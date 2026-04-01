//! Card effect execution — the big match on effect tags.
//!
//! Extracted from engine.rs as a pure refactor. All card-specific logic
//! (damage, block, draw, scry, mantra, vigor, pen nib, etc.) lives here.

use crate::cards::{CardDef, CardTarget, CardType};
use crate::damage;
use crate::engine::CombatEngine;
use crate::orbs::{self, OrbType};
use crate::powers;
use crate::state::Stance;
use crate::status_keys::sk;

/// Execute all effects for a card that was just played.
///
/// Called from `CombatEngine::play_card()` after energy payment and hand removal.
pub fn execute_card_effects(engine: &mut CombatEngine, card: &CardDef, card_id: &str, target_idx: i32) {
    // ---- X-cost: consume all remaining energy as X value + Chemical X bonus ----
    let x_value = if card.cost == -1 {
        let x = engine.state.energy;
        engine.state.energy = 0;
        x + crate::relics::chemical_x_bonus(&engine.state)
    } else {
        0
    };

    // ---- Pen Nib check (before damage) ----
    let pen_nib_active = if card.card_type == CardType::Attack {
        crate::relics::check_pen_nib(&mut engine.state)
    } else {
        false
    };

    // ---- Vigor (consumed on first attack hit) ----
    let vigor = if card.card_type == CardType::Attack {
        let v = engine.state.player.status(sk::VIGOR);
        if v > 0 {
            engine.state.player.set_status(sk::VIGOR, 0);
        }
        v
    } else {
        0
    };

    // ---- Brilliance: extra damage from mantra gained ----
    let brilliance_bonus = if card.effects.contains(&"damage_plus_mantra") {
        engine.state.mantra_gained
    } else {
        0
    };

    // ---- Body Slam: damage = current player block ----
    let body_slam_damage = if card.effects.contains(&"damage_equals_block") {
        engine.state.player.block
    } else {
        -1
    };

    // ---- Grand Finale: only deal damage if draw pile is empty ----
    let grand_finale_blocked = card.effects.contains(&"only_empty_draw")
        && !engine.state.draw_pile.is_empty();

    // ---- Heavy Blade: strength multiplier (3x base, 5x upgraded) ----
    let heavy_blade_mult = if card.effects.contains(&"heavy_blade") {
        card.base_magic.max(1)
    } else {
        1
    };

    // ---- Perfected Strike: +N damage per "Strike" card in all piles ----
    let perfected_strike_bonus = if card.effects.contains(&"perfected_strike") {
        let per_strike = card.base_magic.max(1);
        let strike_count = engine.state.hand.iter()
            .chain(engine.state.draw_pile.iter())
            .chain(engine.state.discard_pile.iter())
            .chain(engine.state.exhaust_pile.iter())
            .filter(|id| {
                let lower = id.to_lowercase();
                lower.contains("strike")
            })
            .count() as i32;
        per_strike * strike_count
    } else {
        0
    };

    // ---- Damage ----
    // Track damage dealt for Wallop (block_from_damage) and Reaper (heal)
    let mut total_unblocked_damage = 0i32;
    let mut enemy_killed = false;

    // Skip generic damage for cards that use damage_random_x_times (they handle their own hits)
    let skip_generic_damage = card.effects.contains(&"damage_random_x_times");

    if !skip_generic_damage && !grand_finale_blocked && (card.base_damage >= 0 || body_slam_damage >= 0) {
        let effective_base_damage = if body_slam_damage >= 0 {
            body_slam_damage
        } else {
            card.base_damage + perfected_strike_bonus
        };

        let is_multi_hit = card.effects.contains(&"multi_hit");

        // X-cost attacks: Whirlwind = X hits AoE, Skewer = X hits single
        let hits = if card_id == "Expunger" || card_id == "Expunger+" {
            // Expunger hits = X from Conjure Blade (stored in ExpungerHits status)
            engine.state.player.status(sk::EXPUNGER_HITS).max(1)
        } else if card.effects.contains(&"x_cost") && card.cost == -1 {
            x_value
        } else if is_multi_hit && card.base_magic > 0 {
            card.base_magic
        } else {
            1
        };

        let player_strength = engine.state.player.strength() * heavy_blade_mult;
        let player_weak = engine.state.player.is_weak();
        let weak_paper_crane = engine.state.has_relic("Paper Crane");
        let stance_mult = engine.state.stance.outgoing_mult();

        // DoubleDamage (Phantasmal Killer, Double Damage potion): consume and double
        let double_damage = engine.state.player.status(sk::DOUBLE_DAMAGE) > 0;
        if double_damage {
            let dd = engine.state.player.status(sk::DOUBLE_DAMAGE);
            engine.state.player.set_status(sk::DOUBLE_DAMAGE, dd - 1);
        }

        match card.target {
            CardTarget::Enemy => {
                if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
                    let tidx = target_idx as usize;
                    let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
                    let enemy_intangible = engine.state.enemies[tidx].entity.status(sk::INTANGIBLE) > 0;
                    let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                    let dmg = damage::calculate_damage_full(
                        effective_base_damage + brilliance_bonus,
                        player_strength,
                        vigor,
                        player_weak,
                        weak_paper_crane,
                        pen_nib_active,
                        double_damage,
                        stance_mult,
                        enemy_vuln,
                        vuln_paper_frog,
                        false, // flight
                        enemy_intangible,
                    );
                    // Talk to the Hand: player gains block per hit ONLY on HP damage
                    let block_return = engine.state.enemies[tidx].entity.status(sk::BLOCK_RETURN);
                    for _ in 0..hits {
                        let enemy_block_before = engine.state.enemies[tidx].entity.block;
                        let enemy_hp_before = engine.state.enemies[tidx].entity.hp;
                        engine.deal_damage_to_enemy(tidx, dmg);
                        // Track unblocked damage for Wallop / Reaper
                        let hp_dmg = dmg - enemy_block_before.min(dmg);
                        total_unblocked_damage += (enemy_hp_before - engine.state.enemies[tidx].entity.hp).max(0);
                        // BlockReturn only triggers on actual HP damage
                        if block_return > 0 {
                            if hp_dmg > 0 || enemy_hp_before > engine.state.enemies[tidx].entity.hp {
                                engine.state.player.block += block_return;
                            }
                        }
                        if engine.state.enemies[tidx].entity.is_dead() {
                            enemy_killed = true;
                            break;
                        }
                    }
                }
            }
            CardTarget::AllEnemy => {
                let living = engine.state.living_enemy_indices();
                for enemy_idx in living {
                    let enemy_vuln = engine.state.enemies[enemy_idx].entity.is_vulnerable();
                    let enemy_intangible = engine.state.enemies[enemy_idx].entity.status(sk::INTANGIBLE) > 0;
                    let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                    let dmg = damage::calculate_damage_full(
                        effective_base_damage + brilliance_bonus,
                        player_strength,
                        vigor,
                        player_weak,
                        weak_paper_crane,
                        pen_nib_active,
                        double_damage,
                        stance_mult,
                        enemy_vuln,
                        vuln_paper_frog,
                        false, // flight
                        enemy_intangible,
                    );
                    let block_return = engine.state.enemies[enemy_idx].entity.status(sk::BLOCK_RETURN);
                    for _ in 0..hits {
                        let enemy_hp_before = engine.state.enemies[enemy_idx].entity.hp;
                        let enemy_block_before = engine.state.enemies[enemy_idx].entity.block;
                        engine.deal_damage_to_enemy(enemy_idx, dmg);
                        total_unblocked_damage += (enemy_hp_before - engine.state.enemies[enemy_idx].entity.hp).max(0);
                        if block_return > 0 {
                            let hp_dmg = dmg - enemy_block_before.min(dmg);
                            if hp_dmg > 0 || enemy_hp_before > engine.state.enemies[enemy_idx].entity.hp {
                                engine.state.player.block += block_return;
                            }
                        }
                        if engine.state.enemies[enemy_idx].entity.is_dead() {
                            enemy_killed = true;
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // ---- Wallop: gain block equal to unblocked damage dealt ----
    if card.effects.contains(&"block_from_damage") {
        engine.state.player.block += total_unblocked_damage;
    }

    // ---- Reaper: heal for total unblocked damage dealt to all enemies ----
    if card.effects.contains(&"reaper") {
        if total_unblocked_damage > 0 {
            engine.state.player.hp = (engine.state.player.hp + total_unblocked_damage)
                .min(engine.state.player.max_hp);
        }
    }

    // ---- Feed: if enemy killed, gain max HP ----
    if card.effects.contains(&"feed") && enemy_killed {
        let hp_gain = card.base_magic.max(3);
        engine.state.player.max_hp += hp_gain;
        engine.state.player.hp += hp_gain;
    }

    // ---- Block ----
    if card.base_block >= 0 {
        // Reinforced Body (block_x_times): gain base_block X times
        let block_multiplier = if card.effects.contains(&"block_x_times") {
            x_value
        } else {
            1
        };
        let dex = engine.state.player.dexterity();
        let frail = engine.state.player.is_frail();
        let block = damage::calculate_block(card.base_block, dex, frail);
        engine.state.player.block += block * block_multiplier;
    }

    // ---- Spirit Shield: gain block per card in hand ----
    if card.effects.contains(&"block_per_card_in_hand") {
        let cards_in_hand = engine.state.hand.len() as i32;
        let per_card = card.base_magic.max(1);
        let dex = engine.state.player.dexterity();
        let frail = engine.state.player.is_frail();
        let block = damage::calculate_block(per_card * cards_in_hand, dex, frail);
        engine.state.player.block += block;
    }

    // ---- Halt: extra block in Wrath ----
    if card.effects.contains(&"extra_block_in_wrath") && engine.state.stance == Stance::Wrath {
        if card.base_magic > 0 {
            let dex = engine.state.player.dexterity();
            let frail = engine.state.player.is_frail();
            let extra = damage::calculate_block(card.base_magic, dex, frail);
            engine.state.player.block += extra;
        }
    }

    // ---- Draw ----
    if card.effects.contains(&"draw") {
        let count = if card.base_magic > 0 { card.base_magic } else { 1 };
        engine.draw_cards(count);
    }

    // ---- Scrawl: draw until hand is 10 ----
    if card.effects.contains(&"draw_to_ten") {
        let cards_to_draw = (10 - engine.state.hand.len() as i32).max(0);
        if cards_to_draw > 0 {
            engine.draw_cards(cards_to_draw);
        }
    }

    // ---- Mantra ----
    if card.effects.contains(&"mantra") && card.base_magic > 0 {
        engine.gain_mantra(card.base_magic);
    }

    // ---- Scry ----
    if card.effects.contains(&"scry") && card.base_magic > 0 {
        engine.do_scry(card.base_magic);
    }

    // ---- Gain Energy (Miracle) ----
    if card.effects.contains(&"gain_energy") && card.base_magic > 0 {
        engine.state.energy += card.base_magic;
    }

    // ---- Vigor (Wreath of Flame) ----
    if card.effects.contains(&"vigor") && card.base_magic > 0 {
        engine.state.player.add_status(sk::VIGOR, card.base_magic);
    }

    // ---- Inner Peace: if in Calm, draw 3; else enter Calm ----
    if card.effects.contains(&"if_calm_draw_else_calm") {
        if engine.state.stance == Stance::Calm {
            engine.draw_cards(card.base_magic);
        } else {
            engine.change_stance(Stance::Calm);
        }
    }

    // ---- BowlingBash: damage per living enemy (extra hits) ----
    if card.effects.contains(&"damage_per_enemy") {
        let living_count = engine.state.living_enemy_indices().len() as i32;
        if living_count > 1 && target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[target_idx as usize].entity.is_vulnerable();
            let dmg = damage::calculate_damage(
                card.base_damage,
                player_strength + vigor,
                player_weak,
                stance_mult,
                enemy_vuln,
                false,
            );
            for _ in 0..(living_count - 1) {
                if engine.state.enemies[target_idx as usize].entity.is_dead() {
                    break;
                }
                engine.deal_damage_to_enemy(target_idx as usize, dmg);
            }
        }
    }

    // ---- CrushJoints: apply Vulnerable if last card played was a Skill ----
    if card.effects.contains(&"vuln_if_last_skill") {
        if engine.state.last_card_type == Some(CardType::Skill) {
            if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
                let vuln_amount = card.base_magic.max(1);
                engine.state.enemies[target_idx as usize]
                    .entity
                    .add_status(sk::VULNERABLE, vuln_amount);
            }
        }
    }

    // ---- SashWhip: apply Weak if last card played was an Attack ----
    if card.effects.contains(&"weak_if_last_attack") {
        if engine.state.last_card_type == Some(CardType::Attack) {
            if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
                let weak_amount = card.base_magic.max(1);
                powers::apply_debuff(
                    &mut engine.state.enemies[target_idx as usize].entity,
                    sk::WEAKENED,
                    weak_amount,
                );
            }
        }
    }

    // ---- FollowUp: gain 1 energy if last card played was an Attack ----
    if card.effects.contains(&"energy_if_last_attack") {
        if engine.state.last_card_type == Some(CardType::Attack) {
            engine.state.energy += 1;
        }
    }

    // ---- TalkToTheHand: apply BlockReturn to target ----
    if card.effects.contains(&"apply_block_return") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            engine.state.enemies[target_idx as usize]
                .entity
                .add_status(sk::BLOCK_RETURN, amount);
        }
    }

    // ---- Ragnarok: deal damage to random enemies X times ----
    if card.effects.contains(&"damage_random_x_times") && card.base_magic > 0 {
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let stance_mult = engine.state.stance.outgoing_mult();
        for _ in 0..card.base_magic {
            let living = engine.state.living_enemy_indices();
            if living.is_empty() {
                break;
            }
            let idx = living[engine.rng_gen_range(0..living.len())];
            let enemy_vuln = engine.state.enemies[idx].entity.is_vulnerable();
            let dmg = damage::calculate_damage(
                card.base_damage,
                player_strength + vigor,
                player_weak,
                stance_mult,
                enemy_vuln,
                false,
            );
            engine.deal_damage_to_enemy(idx, dmg);
        }
    }

    // ---- Pressure Points: apply Mark, then damage all marked enemies ----
    if card.effects.contains(&"pressure_points") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let mark_amount = card.base_magic.max(1);
            engine.state.enemies[target_idx as usize]
                .entity
                .add_status(sk::MARK, mark_amount);
        }
        let living = engine.state.living_enemy_indices();
        for idx in living {
            let mark = engine.state.enemies[idx].entity.status(sk::MARK);
            if mark > 0 {
                // Pressure Points deals HP loss equal to Mark — bypasses block entirely
                engine.state.enemies[idx].entity.hp -= mark;
                engine.state.total_damage_dealt += mark;
                if engine.state.enemies[idx].entity.hp <= 0 {
                    engine.state.enemies[idx].entity.hp = 0;
                }
                // Still fire boss hooks (rebirth, mode shift, etc.)
                crate::combat_hooks::on_enemy_damaged(engine, idx, mark);
            }
        }
    }

    // ---- Judgement: if enemy HP <= threshold, deal their remaining HP as damage ----
    if card.effects.contains(&"judgement") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let threshold = card.base_magic.max(1);
            if engine.state.enemies[tidx].entity.hp <= threshold {
                let hp = engine.state.enemies[tidx].entity.hp;
                // Route through deal_damage_to_enemy so boss hooks fire
                engine.deal_damage_to_enemy(tidx, hp + engine.state.enemies[tidx].entity.block);
            }
        }
    }

    // ---- Lesson Learned: if enemy dies, upgrade a random card ----
    if card.effects.contains(&"lesson_learned") && enemy_killed {
        let mut upgraded = false;
        for card_id in engine.state.draw_pile.iter_mut() {
            if !card_id.ends_with('+') && !card_id.starts_with("Strike_") && !card_id.starts_with("Defend_") {
                card_id.push('+');
                upgraded = true;
                break;
            }
        }
        if !upgraded {
            for card_id in engine.state.discard_pile.iter_mut() {
                if !card_id.ends_with('+') && !card_id.starts_with("Strike_") && !card_id.starts_with("Defend_") {
                    card_id.push('+');
                    break;
                }
            }
        }
    }

    // ---- Shuffle self into draw pile (Tantrum) ----
    if card.effects.contains(&"shuffle_self_into_draw") {
        engine.state.draw_pile.push(card_id.to_string());
        engine.shuffle_draw_pile();
    }

    // ---- Add Insight to draw pile (Evaluate) ----
    if card.effects.contains(&"insight_to_draw") {
        let insight_id = engine.temp_card_id("Insight");
        engine.state.draw_pile.push(insight_id);
        engine.shuffle_draw_pile();
    }

    // ---- Add Smite to hand (Carve Reality) ----
    if card.effects.contains(&"add_smite_to_hand") {
        let smite_id = engine.temp_card_id("Smite");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(smite_id);
        }
    }

    // ---- Add Safety to hand (Deceive Reality) ----
    if card.effects.contains(&"add_safety_to_hand") {
        let safety_id = engine.temp_card_id("Safety");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(safety_id);
        }
    }

    // ---- Add Through Violence to draw (Reach Heaven) ----
    if card.effects.contains(&"add_through_violence_to_draw") {
        let tv_id = engine.temp_card_id("ThroughViolence");
        engine.state.draw_pile.push(tv_id);
        engine.shuffle_draw_pile();
    }

    // ---- Add Beta to draw (Alpha) ----
    if card.effects.contains(&"add_beta_to_draw") {
        let beta_id = engine.temp_card_id("Beta");
        engine.state.draw_pile.push(beta_id);
        engine.shuffle_draw_pile();
    }

    // ---- Add Omega to draw (Beta) ----
    if card.effects.contains(&"add_omega_to_draw") {
        let omega_id = engine.temp_card_id("Omega");
        engine.state.draw_pile.push(omega_id);
        engine.shuffle_draw_pile();
    }

    // ---- Fear No Evil: enter Calm if target enemy is attacking ----
    if card.effects.contains(&"calm_if_enemy_attacking") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            if engine.state.enemies[target_idx as usize].is_attacking() {
                engine.change_stance(Stance::Calm);
            }
        }
    }

    // ---- Indignation: if in Wrath, apply Vuln to all; else enter Wrath ----
    if card.effects.contains(&"indignation") {
        if engine.state.stance == Stance::Wrath {
            let vuln_amount = card.base_magic.max(1);
            let living = engine.state.living_enemy_indices();
            for idx in living {
                powers::apply_debuff(
                    &mut engine.state.enemies[idx].entity,
                    sk::VULNERABLE,
                    vuln_amount,
                );
            }
        } else {
            engine.change_stance(Stance::Wrath);
        }
    }

    // ---- Meditate: return cards from discard to hand ----
    if card.effects.contains(&"meditate") {
        let count = card.base_magic.max(1) as usize;
        for _ in 0..count {
            if engine.state.discard_pile.is_empty() {
                break;
            }
            if engine.state.hand.len() >= 10 {
                break;
            }
            if let Some(returned) = engine.state.discard_pile.pop() {
                engine.state.retained_cards.push(returned.clone());
                engine.state.hand.push(returned);
            }
        }
    }

    // ---- Wave of the Hand ----
    if card.effects.contains(&"wave_of_the_hand") {
        engine.state.player.add_status(sk::WAVE_OF_THE_HAND, card.base_magic.max(1));
    }

    // ---- Rage: gain block when playing Attacks this turn ----
    if card.effects.contains(&"rage") {
        engine.state.player.add_status(sk::RAGE, card.base_magic.max(1));
    }

    // ---- Foreign Influence: MCTS approximation ----
    if card.effects.contains(&"foreign_influence") {
        let smite_id = engine.temp_card_id("Smite");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(smite_id);
        }
    }

    // ---- Conjure Blade: create Expunger with X hits ----
    if card.effects.contains(&"conjure_blade") {
        let expunger_id = engine.temp_card_id("Expunger");
        if x_value > 0 && engine.state.hand.len() < 10 {
            engine.state.hand.push(expunger_id);
            engine.state.player.set_status(sk::EXPUNGER_HITS, x_value);
        }
    }

    // ---- Omniscience: MCTS approximation ----
    if card.effects.contains(&"omniscience") {
        engine.draw_cards(2);
    }

    // ---- Wish: MCTS approximation ----
    if card.effects.contains(&"wish") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::STRENGTH, amount);
    }

    // ---- Blasphemy: die_next_turn flag ----
    if card.effects.contains(&"die_next_turn") {
        engine.state.blasphemy_active = true;
    }

    // ---- Skip enemy turn (Vault) ----
    if card.effects.contains(&"skip_enemy_turn") {
        engine.state.skip_enemy_turn = true;
    }

    // ---- Swivel: next_attack_free ----
    if card.effects.contains(&"next_attack_free") {
        engine.state.player.set_status(sk::NEXT_ATTACK_FREE, 1);
    }

    // ====================================================================
    // Ironclad / Silent — newly implemented effects
    // ====================================================================

    // ---- Apply Vulnerable to single target ----
    if card.effects.contains(&"vulnerable") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            powers::apply_debuff(
                &mut engine.state.enemies[target_idx as usize].entity,
                sk::VULNERABLE,
                amount,
            );
        }
    }

    // ---- Apply Vulnerable to ALL enemies ----
    if card.effects.contains(&"vulnerable_all") {
        let amount = card.base_magic.max(1);
        let living = engine.state.living_enemy_indices();
        for idx in living {
            powers::apply_debuff(
                &mut engine.state.enemies[idx].entity,
                sk::VULNERABLE,
                amount,
            );
        }
    }

    // ---- Apply Weak to single target ----
    if card.effects.contains(&"weak") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            powers::apply_debuff(
                &mut engine.state.enemies[target_idx as usize].entity,
                sk::WEAKENED,
                amount,
            );
        }
    }

    // ---- Apply Weak to ALL enemies ----
    if card.effects.contains(&"weak_all") {
        let amount = card.base_magic.max(1);
        let living = engine.state.living_enemy_indices();
        for idx in living {
            powers::apply_debuff(
                &mut engine.state.enemies[idx].entity,
                sk::WEAKENED,
                amount,
            );
        }
    }

    // ---- Gain exactly 1 energy (Adrenaline) ----
    if card.effects.contains(&"gain_energy_1") {
        engine.state.energy += 1;
    }

    // ---- Limit Break: double current Strength ----
    if card.effects.contains(&"double_strength") {
        let current_str = engine.state.player.strength();
        if current_str > 0 {
            engine.state.player.add_status(sk::STRENGTH, current_str);
        }
    }

    // ---- Catalyst: double target's Poison ----
    if card.effects.contains(&"catalyst_double") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let current_poison = engine.state.enemies[tidx].entity.status(sk::POISON);
            if current_poison > 0 {
                engine.state.enemies[tidx].entity.set_status(sk::POISON, current_poison * 2);
            }
        }
    }

    // ---- Catalyst+: triple target's Poison ----
    if card.effects.contains(&"catalyst_triple") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let current_poison = engine.state.enemies[tidx].entity.status(sk::POISON);
            if current_poison > 0 {
                engine.state.enemies[tidx].entity.set_status(sk::POISON, current_poison * 3);
            }
        }
    }

    // ---- Bullet Time: cards cost 0, no more draw this turn ----
    if card.effects.contains(&"bullet_time") {
        engine.state.player.set_status(sk::BULLET_TIME, 1);
        engine.state.player.set_status(sk::NO_DRAW, 1);
    }

    // ---- Malaise: apply X Weak + X Strength Down (X-cost) ----
    if card.effects.contains(&"malaise") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let amount = x_value + card.base_magic.max(0);
            if amount > 0 {
                powers::apply_debuff(
                    &mut engine.state.enemies[tidx].entity,
                    sk::WEAKENED,
                    amount,
                );
                let current_str = engine.state.enemies[tidx].entity.strength();
                engine.state.enemies[tidx].entity.set_status(sk::STRENGTH, current_str - amount);
            }
        }
    }

    // ---- Doppelganger: next turn draw X + gain X energy (X-cost) ----
    if card.effects.contains(&"doppelganger") {
        let amount = x_value + card.base_magic.max(0);
        if amount > 0 {
            engine.state.player.add_status(sk::DOPPELGANGER_DRAW, amount);
            engine.state.player.add_status(sk::DOPPELGANGER_ENERGY, amount);
        }
    }

    // ---- Corruption: all Skills cost 0 + exhaust on play ----
    if card.effects.contains(&"corruption") {
        engine.state.player.set_status(sk::CORRUPTION, 1);
    }

    // ---- Wraith Form: gain Intangible, -1 Dex per turn ----
    if card.effects.contains(&"wraith_form") {
        let intangible_turns = card.base_magic.max(2);
        engine.state.player.add_status(sk::INTANGIBLE, intangible_turns);
        engine.state.player.add_status(sk::WRAITH_FORM, 1);
    }

    // ---- Echo Form: first card each turn played twice ----
    if card.effects.contains(&"echo_form") {
        engine.state.player.add_status(sk::ECHO_FORM, 1);
    }

    // ---- Creative AI: add random Power to hand each turn ----
    if card.effects.contains(&"creative_ai") {
        engine.state.player.add_status(sk::CREATIVE_AI, card.base_magic.max(1));
    }

    // ====================================================================
    // Defect orb effects
    // ====================================================================

    // ---- Channel Lightning ----
    if card.effects.contains(&"channel_lightning") {
        let count = card.base_magic.max(1);
        let focus = engine.state.player.focus();
        for _ in 0..count {
            let evoke = engine.state.orb_slots.channel(OrbType::Lightning, focus);
            engine.apply_evoke_effect(evoke);
        }
    }

    // ---- Channel Frost ----
    if card.effects.contains(&"channel_frost") {
        let count = card.base_magic.max(1);
        let focus = engine.state.player.focus();
        for _ in 0..count {
            let evoke = engine.state.orb_slots.channel(OrbType::Frost, focus);
            engine.apply_evoke_effect(evoke);
        }
    }

    // ---- Channel Dark ----
    if card.effects.contains(&"channel_dark") {
        let count = card.base_magic.max(1);
        let focus = engine.state.player.focus();
        for _ in 0..count {
            let evoke = engine.state.orb_slots.channel(OrbType::Dark, focus);
            engine.apply_evoke_effect(evoke);
        }
    }

    // ---- Channel Plasma ----
    if card.effects.contains(&"channel_plasma") {
        let count = card.base_magic.max(1);
        let focus = engine.state.player.focus();
        for _ in 0..count {
            let evoke = engine.state.orb_slots.channel(OrbType::Plasma, focus);
            engine.apply_evoke_effect(evoke);
        }
    }

    // ---- Channel Frost per enemy (Chill) ----
    if card.effects.contains(&"channel_frost_per_enemy") {
        let count = engine.state.living_enemy_indices().len() as i32;
        let focus = engine.state.player.focus();
        for _ in 0..count {
            let evoke = engine.state.orb_slots.channel(OrbType::Frost, focus);
            engine.apply_evoke_effect(evoke);
        }
    }

    // ---- Evoke orb (Dualcast) ----
    {
        let evoke_count = card.effects.iter().filter(|&&e| e == "evoke_orb").count();
        if evoke_count > 0 {
            let focus = engine.state.player.focus();
            for _ in 0..evoke_count {
                let effect = engine.state.orb_slots.evoke_front(focus);
                engine.apply_evoke_effect(effect);
            }
        }
    }

    // ---- Gain Focus ----
    if card.effects.contains(&"gain_focus") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::FOCUS, amount);
    }

    // ---- Lose Focus (Hyperbeam) ----
    if card.effects.contains(&"lose_focus") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::FOCUS, -amount);
    }

    // ---- Lose orb slot (Consume) ----
    if card.effects.contains(&"lose_orb_slot") {
        let focus = engine.state.player.focus();
        let evoke = engine.state.orb_slots.remove_slot(focus);
        engine.apply_evoke_effect(evoke);
    }

    // ---- Tempest: channel X Lightning (X-cost) ----
    if card.effects.contains(&"channel_lightning_x") {
        let count = x_value;
        let focus = engine.state.player.focus();
        for _ in 0..count {
            let evoke = engine.state.orb_slots.channel(OrbType::Lightning, focus);
            engine.apply_evoke_effect(evoke);
        }
    }

    // ---- Tempest+: channel X+1 Lightning (X-cost) ----
    if card.effects.contains(&"channel_lightning_x_plus_1") {
        let count = x_value + 1;
        let focus = engine.state.player.focus();
        for _ in 0..count {
            let evoke = engine.state.orb_slots.channel(OrbType::Lightning, focus);
            engine.apply_evoke_effect(evoke);
        }
    }

    // ---- Multi-Cast: evoke front orb X times (X-cost) ----
    if card.effects.contains(&"evoke_orb_x") {
        let count = x_value;
        let focus = engine.state.player.focus();
        let effects = engine.state.orb_slots.evoke_front_n(count as usize, focus);
        for effect in effects {
            engine.apply_evoke_effect(effect);
        }
    }

    // ---- Multi-Cast+: evoke front orb X+1 times (X-cost) ----
    if card.effects.contains(&"evoke_orb_x_plus_1") {
        let count = x_value + 1;
        let focus = engine.state.player.focus();
        let effects = engine.state.orb_slots.evoke_front_n(count as usize, focus);
        for effect in effects {
            engine.apply_evoke_effect(effect);
        }
    }

    // ---- Trigger Dark passive (Darkness+) ----
    if card.effects.contains(&"trigger_dark_passive") {
        let focus = engine.state.player.focus();
        for orb in engine.state.orb_slots.slots.iter_mut() {
            if orb.orb_type == OrbType::Dark {
                let gain = (orb.base_passive + focus).max(0);
                orb.evoke_amount += gain;
            }
        }
    }

    // ---- Double Energy ----
    if card.effects.contains(&"double_energy") {
        engine.state.energy *= 2;
    }

    // ---- Add random Power to hand (White Noise) ----
    if card.effects.contains(&"add_random_power") {
        let power_id = engine.temp_card_id("Defragment");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(power_id);
        }
    }

    // ---- Gain Artifact ----
    if card.effects.contains(&"gain_artifact") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::ARTIFACT, amount);
    }

    // ---- Apply Vulnerable to target (Beam Cell) ----
    if card.effects.contains(&"apply_vulnerable") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            powers::apply_debuff(
                &mut engine.state.enemies[target_idx as usize].entity,
                sk::VULNERABLE,
                amount,
            );
        }
    }

    // ---- Apply Weak (Undo) ----
    if card.effects.contains(&"apply_weak") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            powers::apply_debuff(
                &mut engine.state.enemies[target_idx as usize].entity,
                sk::WEAKENED,
                amount,
            );
        }
    }

    // ---- Reprogram: lose Focus, gain Str + Dex ----
    if card.effects.contains(&"reprogram") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::FOCUS, -amount);
        engine.state.player.add_status(sk::STRENGTH, amount);
        engine.state.player.add_status(sk::DEXTERITY, amount);
    }

    // ---- Damage per orb (Barrage) ----
    if card.effects.contains(&"damage_per_orb") {
        let orb_count = engine.state.orb_slots.occupied_count() as i32;
        if orb_count > 1 && target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
            let dmg = damage::calculate_damage(
                card.base_damage,
                player_strength + vigor,
                player_weak,
                stance_mult,
                enemy_vuln,
                false,
            );
            for _ in 0..(orb_count - 1) {
                if engine.state.enemies[tidx].entity.is_dead() {
                    break;
                }
                engine.deal_damage_to_enemy(tidx, dmg);
            }
        }
    }

    // ---- Draw per unique orb (Compile Driver) ----
    if card.effects.contains(&"draw_per_unique_orb") {
        let mut types = std::collections::HashSet::new();
        for orb in &engine.state.orb_slots.slots {
            if !orb.is_empty() {
                types.insert(orb.orb_type);
            }
        }
        let draw_count = types.len() as i32;
        if draw_count > 0 {
            engine.draw_cards(draw_count);
        }
    }

    // ---- Fission: remove all orbs, gain energy + draw per orb ----
    if card.effects.contains(&"fission") {
        let orb_count = engine.state.orb_slots.occupied_count() as i32;
        engine.state.orb_slots.slots = vec![orbs::Orb::new(OrbType::Empty); engine.state.orb_slots.max_slots];
        if orb_count > 0 {
            engine.state.energy += orb_count;
            engine.draw_cards(orb_count);
        }
    }

    // ---- Fission+: evoke all orbs, gain energy + draw per orb ----
    if card.effects.contains(&"fission_evoke") {
        let orb_count = engine.state.orb_slots.occupied_count() as i32;
        let focus = engine.state.player.focus();
        let effects = engine.state.orb_slots.evoke_all(focus);
        for effect in effects {
            engine.apply_evoke_effect(effect);
        }
        if orb_count > 0 {
            engine.state.energy += orb_count;
            engine.draw_cards(orb_count);
        }
    }

    // ---- Energy on Kill (Sunder) ----
    if card.effects.contains(&"energy_on_kill") && enemy_killed {
        engine.state.energy += 3;
    }

    // ---- Return zero-cost cards from discard to hand (All For One) ----
    if card.effects.contains(&"return_zero_cost_from_discard") {
        let registry = crate::cards::CardRegistry::new();
        let mut returned = Vec::new();
        engine.state.discard_pile.retain(|card_id| {
            if let Some(def) = registry.get(card_id) {
                if def.cost == 0 && engine.state.hand.len() + returned.len() < 10 {
                    returned.push(card_id.clone());
                    return false;
                }
            }
            true
        });
        engine.state.hand.extend(returned);
    }

    // ---- Reboot: shuffle hand+discard into draw, draw base_magic cards ----
    if card.effects.contains(&"reboot") {
        let draw_count = card.base_magic.max(4);
        let hand_cards: Vec<String> = engine.state.hand.drain(..).collect();
        engine.state.draw_pile.extend(hand_cards);
        let discard_cards: Vec<String> = engine.state.discard_pile.drain(..).collect();
        engine.state.draw_pile.extend(discard_cards);
        engine.draw_cards(draw_count);
    }

    // ---- Seek: tutor base_magic cards from draw pile to hand ----
    if card.effects.contains(&"seek") {
        let count = card.base_magic.max(1) as usize;
        let to_move = count.min(engine.state.draw_pile.len()).min(10 - engine.state.hand.len());
        for _ in 0..to_move {
            if let Some(card_id) = engine.state.draw_pile.pop() {
                engine.state.hand.push(card_id);
            }
        }
    }

    // ====================================================================
    // Newly implemented effect handlers
    // ====================================================================

    // ---- Poison: apply Poison to single target ----
    if card.effects.contains(&"poison") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            engine.state.enemies[target_idx as usize]
                .entity
                .add_status(sk::POISON, amount);
        }
    }

    // ---- Poison All: apply Poison to ALL enemies ----
    if card.effects.contains(&"poison_all") {
        let amount = card.base_magic.max(1);
        let living = engine.state.living_enemy_indices();
        for idx in living {
            engine.state.enemies[idx].entity.add_status(sk::POISON, amount);
        }
    }

    // ---- Copy to Discard: add a copy of this card to discard (Anger) ----
    if card.effects.contains(&"copy_to_discard") {
        engine.state.discard_pile.push(card_id.to_string());
    }

    // ---- Discard: force player to discard 1 card from hand (MCTS: discard random) ----
    if card.effects.contains(&"discard") {
        if !engine.state.hand.is_empty() {
            let idx = engine.rng_gen_range(0..engine.state.hand.len());
            let discarded = engine.state.hand.remove(idx);
            engine.state.discard_pile.push(discarded);
        }
    }

    // ---- Discard to Top of Draw: move a card from discard to top of draw (Headbutt) ----
    if card.effects.contains(&"discard_to_top_of_draw") {
        if !engine.state.discard_pile.is_empty() {
            // MCTS approximation: move the last discarded card to top of draw
            if let Some(moved) = engine.state.discard_pile.pop() {
                engine.state.draw_pile.push(moved);
            }
        }
    }

    // ---- Double Block: double current player block (Entrench) ----
    if card.effects.contains(&"double_block") {
        engine.state.player.block *= 2;
    }

    // ---- Offering: lose 6 HP, gain 2 energy, draw N cards ----
    if card.effects.contains(&"offering") {
        engine.state.player.hp = (engine.state.player.hp - 6).max(0);
        engine.state.energy += 2;
        let draw_count = card.base_magic.max(3);
        engine.draw_cards(draw_count);
    }

    // ---- Thorns: apply Thorns to self (Caltrops) ----
    if card.effects.contains(&"thorns") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::THORNS, amount);
    }

    // ---- Gain Strength (Inflame) ----
    if card.effects.contains(&"gain_strength") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::STRENGTH, amount);
    }

    // ---- Temp Strength: gain Strength this turn, lose at end (Flex) ----
    if card.effects.contains(&"temp_strength") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::STRENGTH, amount);
        // Track temporary strength to remove at end of turn
        engine.state.player.add_status(sk::TEMP_STRENGTH, amount);
    }

    // ---- Gain Dexterity (Footwork) ----
    if card.effects.contains(&"gain_dexterity") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sk::DEXTERITY, amount);
    }

    // ---- Reduce Strength: target enemy loses Strength (Disarm) ----
    if card.effects.contains(&"reduce_strength") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            let tidx = target_idx as usize;
            let current = engine.state.enemies[tidx].entity.strength();
            engine.state.enemies[tidx].entity.set_status(sk::STRENGTH, current - amount);
        }
    }

    // ---- Reduce Strength All Temp: all enemies lose Strength temporarily (Piercing Wail) ----
    if card.effects.contains(&"reduce_strength_all_temp") {
        let amount = card.base_magic.max(1);
        let living = engine.state.living_enemy_indices();
        for idx in living {
            let current = engine.state.enemies[idx].entity.strength();
            engine.state.enemies[idx].entity.set_status(sk::STRENGTH, current - amount);
            // Track for restoration at end of turn
            engine.state.enemies[idx].entity.add_status(sk::TEMP_STRENGTH_LOSS, amount);
        }
    }

    // ---- Heal: restore HP (Bandage Up) ----
    if card.effects.contains(&"heal") {
        let amount = card.base_magic.max(1);
        engine.state.player.hp = (engine.state.player.hp + amount).min(engine.state.player.max_hp);
    }

    // ---- Heal on Play: same as heal (Bite) ----
    if card.effects.contains(&"heal_on_play") {
        let amount = card.base_magic.max(1);
        engine.state.player.hp = (engine.state.player.hp + amount).min(engine.state.player.max_hp);
    }

    // ---- Intangible: gain Intangible (Apparition/Ghostly) ----
    if card.effects.contains(&"intangible") {
        engine.state.player.add_status(sk::INTANGIBLE, 1);
    }

    // ---- Exhaust Choose: player chooses N cards from hand to exhaust (MCTS: random) ----
    if card.effects.contains(&"exhaust_choose") {
        let count = 1; // Standard: exhaust 1 card
        for _ in 0..count {
            if engine.state.hand.is_empty() {
                break;
            }
            let idx = engine.rng_gen_range(0..engine.state.hand.len());
            let exhausted = engine.state.hand.remove(idx);
            engine.state.exhaust_pile.push(exhausted);
        }
    }

    // ---- Exhaust Random: exhaust N random cards from hand ----
    if card.effects.contains(&"exhaust_random") {
        let count = 1; // Standard: exhaust 1 random card
        for _ in 0..count {
            if engine.state.hand.is_empty() {
                break;
            }
            let idx = engine.rng_gen_range(0..engine.state.hand.len());
            let exhausted = engine.state.hand.remove(idx);
            engine.state.exhaust_pile.push(exhausted);
        }
    }

    // ---- Spot Weakness: if target enemy intending Attack, gain Strength ----
    if card.effects.contains(&"spot_weakness") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            if engine.state.enemies[tidx].is_attacking() {
                let amount = card.base_magic.max(1);
                engine.state.player.add_status(sk::STRENGTH, amount);
            }
        }
    }

    // ---- Fiend Fire: exhaust all hand cards, deal damage per card exhausted ----
    if card.effects.contains(&"fiend_fire") {
        let hand_count = engine.state.hand.len() as i32;
        // Exhaust all cards from hand
        let exhausted_cards: Vec<String> = engine.state.hand.drain(..).collect();
        engine.state.exhaust_pile.extend(exhausted_cards);
        // Deal base_damage per card exhausted to the target
        if hand_count > 0 && target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let weak_paper_crane = engine.state.has_relic("Paper Crane");
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
            let enemy_intangible = engine.state.enemies[tidx].entity.status(sk::INTANGIBLE) > 0;
            let vuln_paper_frog = engine.state.has_relic("Paper Frog");
            let dmg = damage::calculate_damage_full(
                card.base_damage,
                player_strength,
                vigor,
                player_weak,
                weak_paper_crane,
                pen_nib_active,
                false,
                stance_mult,
                enemy_vuln,
                vuln_paper_frog,
                false,
                enemy_intangible,
            );
            for _ in 0..hand_count {
                if engine.state.enemies[tidx].entity.is_dead() {
                    break;
                }
                engine.deal_damage_to_enemy(tidx, dmg);
            }
        }
    }

    // ---- Next-turn energy (Conserve Battery, Outmaneuver, Flying Knee) ----
    if card.effects.contains(&"next_turn_energy") {
        engine.state.player.add_status(sk::ENERGIZED, card.base_magic);
    }

    // ---- Next-turn block (Dodge and Roll) ----
    if card.effects.contains(&"next_turn_block") {
        engine.state.player.add_status(sk::NEXT_TURN_BLOCK, card.base_magic);
    }

    // ---- Draw next turn (Predator) ----
    if card.effects.contains(&"draw_next_turn") {
        engine.state.player.add_status(sk::DRAW_CARD, card.base_magic);
    }

    // ---- Double Tap: next Attack played twice ----
    if card.effects.contains(&"double_tap") {
        engine.state.player.set_status(sk::DOUBLE_TAP, card.base_magic.max(1));
    }

    // ---- Burst: next Skill played twice ----
    if card.effects.contains(&"burst") {
        engine.state.player.set_status(sk::BURST, card.base_magic.max(1));
    }

    // ---- Second Wind: exhaust all non-attack cards in hand, gain block per exhaust ----
    if card.effects.contains(&"second_wind") {
        let registry = crate::cards::CardRegistry::new();
        let block_per = card.base_block.max(5);
        let dex = engine.state.player.dexterity();
        let frail = engine.state.player.is_frail();
        let mut to_exhaust = Vec::new();
        let mut remaining = Vec::new();
        for hand_card in engine.state.hand.drain(..) {
            let is_attack = registry.get(&hand_card)
                .map(|def| def.card_type == CardType::Attack)
                .unwrap_or(false);
            if is_attack {
                remaining.push(hand_card);
            } else {
                to_exhaust.push(hand_card);
            }
        }
        let exhaust_count = to_exhaust.len() as i32;
        engine.state.exhaust_pile.extend(to_exhaust);
        engine.state.hand = remaining;
        if exhaust_count > 0 {
            let block = damage::calculate_block(block_per * exhaust_count, dex, frail);
            engine.state.player.block += block;
        }
    }
}
