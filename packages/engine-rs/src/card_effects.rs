//! Card effect execution — the big match on effect tags.
//!
//! Extracted from engine.rs as a pure refactor. All card-specific logic
//! (damage, block, draw, scry, mantra, vigor, pen nib, etc.) lives here.

use crate::cards::{CardDef, CardTarget, CardType};
use crate::combat_types::CardInstance;
use crate::damage;
use crate::engine::{CombatEngine, ChoiceOption, ChoiceReason};
use crate::orbs::{self, OrbType};
use crate::powers;
use crate::state::Stance;
use crate::status_ids::sid;

/// Execute all effects for a card that was just played.
///
/// Called from `CombatEngine::play_card()` after energy payment and hand removal.
pub fn execute_card_effects(engine: &mut CombatEngine, card: &CardDef, card_inst: CardInstance, target_idx: i32) {
    let card_id = engine.card_registry.card_name(card_inst.def_id);
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
        let v = engine.state.player.status(sid::VIGOR);
        if v > 0 {
            engine.state.player.set_status(sid::VIGOR, 0);
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
            .filter(|c| engine.card_registry.is_strike(c.def_id))
            .count() as i32;
        per_strike * strike_count
    } else {
        0
    };

    // ---- Per-card scaling bonuses (from status counters) ----
    let scaling_bonus = if card.effects.contains(&"rampage") {
        engine.state.player.status(sid::RAMPAGE_BONUS)
    } else if card.effects.contains(&"glass_knife") {
        -engine.state.player.status(sid::GLASS_KNIFE_PENALTY)
    } else if card.effects.contains(&"ritual_dagger") {
        engine.state.player.status(sid::RITUAL_DAGGER_BONUS)
    } else if card.effects.contains(&"searing_blow") {
        // Searing Blow: base 12, each upgrade adds progressively more
        // Upgraded flag = +4 bonus (simplified for MCTS)
        if card_inst.flags & 0x04 != 0 { 4 } else { 0 }
    } else {
        0
    };

    // ---- Genetic Algorithm: scaling block bonus ----
    let genetic_alg_block_bonus = if card.effects.contains(&"genetic_algorithm") {
        engine.state.player.status(sid::GENETIC_ALG_BONUS)
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
            (card.base_damage + perfected_strike_bonus + scaling_bonus).max(0)
        };

        let is_multi_hit = card.effects.contains(&"multi_hit");

        // X-cost attacks: Whirlwind = X hits AoE, Skewer = X hits single
        let hits = if card_id == "Expunger" || card_id == "Expunger+" {
            // Expunger hits = X from Conjure Blade (stored in ExpungerHits status)
            engine.state.player.status(sid::EXPUNGER_HITS).max(1)
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
        let double_damage = engine.state.player.status(sid::DOUBLE_DAMAGE) > 0;
        if double_damage {
            let dd = engine.state.player.status(sid::DOUBLE_DAMAGE);
            engine.state.player.set_status(sid::DOUBLE_DAMAGE, dd - 1);
        }

        match card.target {
            CardTarget::Enemy => {
                if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
                    let tidx = target_idx as usize;
                    let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
                    let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
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
                    let block_return = engine.state.enemies[tidx].entity.status(sid::BLOCK_RETURN);
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
                                engine.gain_block_player(block_return);
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
                    let enemy_intangible = engine.state.enemies[enemy_idx].entity.status(sid::INTANGIBLE) > 0;
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
                    let block_return = engine.state.enemies[enemy_idx].entity.status(sid::BLOCK_RETURN);
                    for _ in 0..hits {
                        let enemy_hp_before = engine.state.enemies[enemy_idx].entity.hp;
                        let enemy_block_before = engine.state.enemies[enemy_idx].entity.block;
                        engine.deal_damage_to_enemy(enemy_idx, dmg);
                        total_unblocked_damage += (enemy_hp_before - engine.state.enemies[enemy_idx].entity.hp).max(0);
                        if block_return > 0 {
                            let hp_dmg = dmg - enemy_block_before.min(dmg);
                            if hp_dmg > 0 || enemy_hp_before > engine.state.enemies[enemy_idx].entity.hp {
                                engine.gain_block_player(block_return);
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
        engine.gain_block_player(total_unblocked_damage);
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
    // block_if_skill (Escape Plan): block is conditional, handled separately below
    if card.base_block >= 0 && !card.effects.contains(&"block_if_skill") {
        // Reinforced Body (block_x_times): gain base_block X times
        let block_multiplier = if card.effects.contains(&"block_x_times") {
            x_value
        } else {
            1
        };
        let dex = engine.state.player.dexterity();
        let frail = engine.state.player.is_frail();
        let block = damage::calculate_block(card.base_block + genetic_alg_block_bonus, dex, frail);
        engine.gain_block_player(block * block_multiplier);
    }

    // ---- Spirit Shield: gain block per card in hand ----
    if card.effects.contains(&"block_per_card_in_hand") {
        let cards_in_hand = engine.state.hand.len() as i32;
        let per_card = card.base_magic.max(1);
        let dex = engine.state.player.dexterity();
        let frail = engine.state.player.is_frail();
        let block = damage::calculate_block(per_card * cards_in_hand, dex, frail);
        engine.gain_block_player(block);
    }

    // ---- Halt: extra block in Wrath ----
    if card.effects.contains(&"extra_block_in_wrath") && engine.state.stance == Stance::Wrath {
        if card.base_magic > 0 {
            let dex = engine.state.player.dexterity();
            let frail = engine.state.player.is_frail();
            let extra = damage::calculate_block(card.base_magic, dex, frail);
            engine.gain_block_player(extra);
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
        // Scry triggers AwaitingChoice -- pause remaining effects
        if engine.phase == crate::engine::CombatPhase::AwaitingChoice {
            return;
        }
    }

    // ---- Gain Energy (Miracle) ----
    if card.effects.contains(&"gain_energy") && card.base_magic > 0 {
        engine.state.energy += card.base_magic;
    }

    // ---- Vigor (Wreath of Flame) ----
    if card.effects.contains(&"vigor") && card.base_magic > 0 {
        engine.state.player.add_status(sid::VIGOR, card.base_magic);
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
            let tidx = target_idx as usize;
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let weak_pc = engine.state.has_relic("Paper Crane");
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
            let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
            let vuln_pf = engine.state.has_relic("Paper Frog");
            let has_flight = engine.state.enemies[tidx].entity.status(sid::FLIGHT) > 0;
            // Vigor and Pen Nib already consumed on first hit — don't apply again
            let dmg = damage::calculate_damage_full(
                card.base_damage,
                player_strength,
                0, // vigor already applied on first hit
                player_weak,
                weak_pc,
                false, // pen nib already applied on first hit
                false,
                stance_mult,
                enemy_vuln,
                vuln_pf,
                has_flight,
                enemy_intangible,
            );
            for _ in 0..(living_count - 1) {
                if engine.state.enemies[tidx].entity.is_dead() {
                    break;
                }
                engine.deal_damage_to_enemy(tidx, dmg);
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
                    .add_status(sid::VULNERABLE, vuln_amount);
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
                    sid::WEAKENED,
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
                .add_status(sid::BLOCK_RETURN, amount);
        }
    }

    // ---- Ragnarok: deal damage to random enemies X times ----
    if card.effects.contains(&"damage_random_x_times") && card.base_magic > 0 {
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let weak_pc = engine.state.has_relic("Paper Crane");
        let vuln_pf = engine.state.has_relic("Paper Frog");
        let stance_mult = engine.state.stance.outgoing_mult();
        for hit_i in 0..card.base_magic {
            let living = engine.state.living_enemy_indices();
            if living.is_empty() {
                break;
            }
            let idx = living[engine.rng_gen_range(0..living.len())];
            let enemy_vuln = engine.state.enemies[idx].entity.is_vulnerable();
            let enemy_intangible = engine.state.enemies[idx].entity.status(sid::INTANGIBLE) > 0;
            let has_flight = engine.state.enemies[idx].entity.status(sid::FLIGHT) > 0;
            // Vigor and Pen Nib only apply on first hit
            let dmg = damage::calculate_damage_full(
                card.base_damage,
                player_strength,
                if hit_i == 0 { vigor } else { 0 },
                player_weak,
                weak_pc,
                if hit_i == 0 { pen_nib_active } else { false },
                false,
                stance_mult,
                enemy_vuln,
                vuln_pf,
                has_flight,
                enemy_intangible,
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
                .add_status(sid::MARK, mark_amount);
        }
        let living = engine.state.living_enemy_indices();
        for idx in living {
            let mark = engine.state.enemies[idx].entity.status(sid::MARK);
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
        for c in engine.state.draw_pile.iter_mut() {
            if !c.is_upgraded() {
                let name = engine.card_registry.card_name(c.def_id);
                if !name.starts_with("Strike_") && !name.starts_with("Defend_") {
                    engine.card_registry.upgrade_card(c);
                    upgraded = true;
                    break;
                }
            }
        }
        if !upgraded {
            for c in engine.state.discard_pile.iter_mut() {
                if !c.is_upgraded() {
                    let name = engine.card_registry.card_name(c.def_id);
                    if !name.starts_with("Strike_") && !name.starts_with("Defend_") {
                        engine.card_registry.upgrade_card(c);
                        break;
                    }
                }
            }
        }
    }

    // ---- Shuffle self into draw pile (Tantrum) ----
    if card.effects.contains(&"shuffle_self_into_draw") {
        engine.state.draw_pile.push(card_inst);
        engine.shuffle_draw_pile();
    }

    // ---- Add Insight to draw pile (Evaluate) ----
    if card.effects.contains(&"insight_to_draw") {
        let insight_id = engine.temp_card("Insight");
        engine.state.draw_pile.push(insight_id);
        engine.shuffle_draw_pile();
    }

    // ---- Add Smite to hand (Carve Reality) ----
    if card.effects.contains(&"add_smite_to_hand") {
        let smite_id = engine.temp_card("Smite");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(smite_id);
        }
    }

    // ---- Add Safety to hand (Deceive Reality) ----
    if card.effects.contains(&"add_safety_to_hand") {
        let safety_id = engine.temp_card("Safety");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(safety_id);
        }
    }

    // ---- Add Through Violence to draw (Reach Heaven) ----
    if card.effects.contains(&"add_through_violence_to_draw") {
        let tv_id = engine.temp_card("ThroughViolence");
        engine.state.draw_pile.push(tv_id);
        engine.shuffle_draw_pile();
    }

    // ---- Add Beta to draw (Alpha) ----
    if card.effects.contains(&"add_beta_to_draw") {
        let beta_id = engine.temp_card("Beta");
        engine.state.draw_pile.push(beta_id);
        engine.shuffle_draw_pile();
    }

    // ---- Add Omega to draw (Beta) ----
    if card.effects.contains(&"add_omega_to_draw") {
        let omega_id = engine.temp_card("Omega");
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
                    sid::VULNERABLE,
                    vuln_amount,
                );
            }
        } else {
            engine.change_stance(Stance::Wrath);
        }
    }

    // ---- Meditate: choose cards from discard to return to hand (retained) ----
    if card.effects.contains(&"meditate") {
        let count = card.base_magic.max(1) as usize;
        if !engine.state.discard_pile.is_empty() {
            let options: Vec<_> = engine.state.discard_pile.iter()
                .enumerate()
                .map(|(i, _)| ChoiceOption::DiscardCard(i))
                .collect();
            let max_picks = count.min(options.len());
            engine.begin_choice(ChoiceReason::ReturnFromDiscard, options, 1, max_picks);
        }
    }

    // ---- Wave of the Hand ----
    if card.effects.contains(&"wave_of_the_hand") {
        engine.state.player.add_status(sid::WAVE_OF_THE_HAND, card.base_magic.max(1));
    }

    // ---- Rage: gain block when playing Attacks this turn ----
    if card.effects.contains(&"rage") {
        engine.state.player.add_status(sid::RAGE, card.base_magic.max(1));
    }

    // ---- Discovery: choose 1 of 3 generated cards to add to hand ----
    if card.effects.contains(&"discovery") {
        if engine.state.hand.len() < 10 {
            let options = vec![
                crate::engine::ChoiceOption::GeneratedCard(engine.temp_card("Smite")),
                crate::engine::ChoiceOption::GeneratedCard(engine.temp_card("Safety")),
                crate::engine::ChoiceOption::GeneratedCard(engine.temp_card("Insight")),
            ];
            engine.begin_choice(
                crate::engine::ChoiceReason::DiscoverCard,
                options,
                1,
                1,
            );
            return;
        }
    }

    // ---- Foreign Influence: choose 1 of 3 generated attack cards ----
    if card.effects.contains(&"foreign_influence") {
        if engine.state.hand.len() < 10 {
            let options = vec![
                crate::engine::ChoiceOption::GeneratedCard(engine.temp_card("Smite")),
                crate::engine::ChoiceOption::GeneratedCard(engine.temp_card("Flying Sleeves")),
                crate::engine::ChoiceOption::GeneratedCard(engine.temp_card("Iron Wave")),
            ];
            engine.begin_choice(
                crate::engine::ChoiceReason::DiscoverCard,
                options,
                1,
                1,
            );
            return;
        }
    }

    // ---- Conjure Blade: create Expunger with X hits ----
    if card.effects.contains(&"conjure_blade") {
        let expunger_id = engine.temp_card("Expunger");
        if x_value > 0 && engine.state.hand.len() < 10 {
            engine.state.hand.push(expunger_id);
            engine.state.player.set_status(sid::EXPUNGER_HITS, x_value);
        }
    }

    // ---- Omniscience: player picks a card from hand to play for free ----
    if card.effects.contains(&"omniscience") {
        if !engine.state.hand.is_empty() {
            let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.hand.len())
                .map(|i| crate::engine::ChoiceOption::HandCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::PlayCardFree,
                options,
                1,
                1,
            );
            return;
        }
    }

    // ---- Wish: player picks from Strength / Gold / Plated Armor ----
    if card.effects.contains(&"wish") {
        let options = vec![
            crate::engine::ChoiceOption::Named("Strength"),
            crate::engine::ChoiceOption::Named("Gold"),
            crate::engine::ChoiceOption::Named("Plated Armor"),
        ];
        engine.begin_choice(
            crate::engine::ChoiceReason::PickOption,
            options,
            1,
            1,
        );
        return;
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
        engine.state.player.set_status(sid::NEXT_ATTACK_FREE, 1);
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
                sid::VULNERABLE,
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
                sid::VULNERABLE,
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
                sid::WEAKENED,
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
                sid::WEAKENED,
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
            engine.state.player.add_status(sid::STRENGTH, current_str);
        }
    }

    // ---- Catalyst: double target's Poison ----
    if card.effects.contains(&"catalyst_double") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let current_poison = engine.state.enemies[tidx].entity.status(sid::POISON);
            if current_poison > 0 {
                engine.state.enemies[tidx].entity.set_status(sid::POISON, current_poison * 2);
            }
        }
    }

    // ---- Catalyst+: triple target's Poison ----
    if card.effects.contains(&"catalyst_triple") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let current_poison = engine.state.enemies[tidx].entity.status(sid::POISON);
            if current_poison > 0 {
                engine.state.enemies[tidx].entity.set_status(sid::POISON, current_poison * 3);
            }
        }
    }

    // ---- Bullet Time: cards cost 0, no more draw this turn ----
    if card.effects.contains(&"bullet_time") {
        engine.state.player.set_status(sid::BULLET_TIME, 1);
        engine.state.player.set_status(sid::NO_DRAW, 1);
    }

    // ---- Malaise: apply X Weak + X Strength Down (X-cost) ----
    if card.effects.contains(&"malaise") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let amount = x_value + card.base_magic.max(0);
            if amount > 0 {
                powers::apply_debuff(
                    &mut engine.state.enemies[tidx].entity,
                    sid::WEAKENED,
                    amount,
                );
                let current_str = engine.state.enemies[tidx].entity.strength();
                engine.state.enemies[tidx].entity.set_status(sid::STRENGTH, current_str - amount);
            }
        }
    }

    // ---- Doppelganger: next turn draw X + gain X energy (X-cost) ----
    if card.effects.contains(&"doppelganger") {
        let amount = x_value + card.base_magic.max(0);
        if amount > 0 {
            engine.state.player.add_status(sid::DOPPELGANGER_DRAW, amount);
            engine.state.player.add_status(sid::DOPPELGANGER_ENERGY, amount);
        }
    }

    // ---- Corruption: all Skills cost 0 + exhaust on play ----
    if card.effects.contains(&"corruption") {
        engine.state.player.set_status(sid::CORRUPTION, 1);
    }

    // ---- Wraith Form: gain Intangible, -1 Dex per turn ----
    if card.effects.contains(&"wraith_form") {
        let intangible_turns = card.base_magic.max(2);
        engine.state.player.add_status(sid::INTANGIBLE, intangible_turns);
        engine.state.player.add_status(sid::WRAITH_FORM, 1);
    }

    // ---- Echo Form: first card each turn played twice ----
    if card.effects.contains(&"echo_form") {
        engine.state.player.add_status(sid::ECHO_FORM, 1);
    }

    // ---- Creative AI: add random Power to hand each turn ----
    if card.effects.contains(&"creative_ai") {
        engine.state.player.add_status(sid::CREATIVE_AI, card.base_magic.max(1));
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
        engine.state.player.add_status(sid::FOCUS, amount);
    }

    // ---- Lose Focus (Hyperbeam) ----
    if card.effects.contains(&"lose_focus") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sid::FOCUS, -amount);
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
        let power_id = engine.temp_card("Defragment");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(power_id);
        }
    }

    // ---- Gain Artifact ----
    if card.effects.contains(&"gain_artifact") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sid::ARTIFACT, amount);
    }

    // ---- Apply Vulnerable to target (Beam Cell) ----
    if card.effects.contains(&"apply_vulnerable") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            powers::apply_debuff(
                &mut engine.state.enemies[target_idx as usize].entity,
                sid::VULNERABLE,
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
                sid::WEAKENED,
                amount,
            );
        }
    }

    // ---- Reprogram: lose Focus, gain Str + Dex ----
    if card.effects.contains(&"reprogram") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sid::FOCUS, -amount);
        engine.state.player.add_status(sid::STRENGTH, amount);
        engine.state.player.add_status(sid::DEXTERITY, amount);
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
        let mut returned = Vec::new();
        engine.state.discard_pile.retain(|card_inst| {
            let def = engine.card_registry.card_def_by_id(card_inst.def_id);
            if def.cost == 0 && engine.state.hand.len() + returned.len() < 10 {
                returned.push(*card_inst);
                return false;
            }
            true
        });
        engine.state.hand.extend(returned);
    }

    // ---- Reboot: shuffle hand+discard into draw, draw base_magic cards ----
    if card.effects.contains(&"reboot") {
        let draw_count = card.base_magic.max(4);
        let hand_cards: Vec<CardInstance> = engine.state.hand.drain(..).collect();
        engine.state.draw_pile.extend(hand_cards);
        let discard_cards: Vec<CardInstance> = engine.state.discard_pile.drain(..).collect();
        engine.state.draw_pile.extend(discard_cards);
        engine.draw_cards(draw_count);
    }

    // ---- Seek: player picks card(s) from draw pile to add to hand ----
    if card.effects.contains(&"seek") {
        let count = card.base_magic.max(1) as usize;
        if !engine.state.draw_pile.is_empty() {
            let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.draw_pile.len())
                .map(|i| crate::engine::ChoiceOption::DrawCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::PickFromDrawPile,
                options,
                1,
                count,
            );
            return; // Pause execution; remaining effects handled after choice resolves
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
                .add_status(sid::POISON, amount);
        }
    }

    // ---- Poison All: apply Poison to ALL enemies ----
    if card.effects.contains(&"poison_all") {
        let amount = card.base_magic.max(1);
        let living = engine.state.living_enemy_indices();
        for idx in living {
            engine.state.enemies[idx].entity.add_status(sid::POISON, amount);
        }
    }

    // ---- Copy to Discard: add a copy of this card to discard (Anger) ----
    if card.effects.contains(&"copy_to_discard") {
        engine.state.discard_pile.push(card_inst);
    }

    // ---- Discard: force player to discard 1 card from hand ----
    if card.effects.contains(&"discard") {
        if !engine.state.hand.is_empty() {
            let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.hand.len())
                .map(|i| crate::engine::ChoiceOption::HandCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::DiscardFromHand,
                options,
                1,
                1,
            );
            return;
        }
    }

    // ---- Discard to Top of Draw: player picks a card from discard to put on top of draw (Headbutt) ----
    if card.effects.contains(&"discard_to_top_of_draw") {
        if !engine.state.discard_pile.is_empty() {
            let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.discard_pile.len())
                .map(|i| crate::engine::ChoiceOption::DiscardCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::PickFromDiscard,
                options,
                1,
                1,
            );
            return;
        }
    }

    // ---- Put Card On Top: player picks a card from hand to put on top of draw (Warcry) ----
    if card.effects.contains(&"put_card_on_top") {
        if !engine.state.hand.is_empty() {
            let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.hand.len())
                .map(|i| crate::engine::ChoiceOption::HandCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::PutOnTopFromHand,
                options,
                1,
                1,
            );
            return;
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
        engine.state.player.add_status(sid::THORNS, amount);
    }

    // ---- Gain Strength (Inflame) ----
    if card.effects.contains(&"gain_strength") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sid::STRENGTH, amount);
    }

    // ---- Temp Strength: gain Strength this turn, lose at end (Flex) ----
    if card.effects.contains(&"temp_strength") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sid::STRENGTH, amount);
        // Track temporary strength to remove at end of turn
        engine.state.player.add_status(sid::TEMP_STRENGTH, amount);
    }

    // ---- Gain Dexterity (Footwork) ----
    if card.effects.contains(&"gain_dexterity") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sid::DEXTERITY, amount);
    }

    // ---- Reduce Strength: target enemy loses Strength (Disarm) ----
    if card.effects.contains(&"reduce_strength") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            let tidx = target_idx as usize;
            let current = engine.state.enemies[tidx].entity.strength();
            engine.state.enemies[tidx].entity.set_status(sid::STRENGTH, current - amount);
        }
    }

    // ---- Reduce Strength All Temp: all enemies lose Strength temporarily (Piercing Wail) ----
    if card.effects.contains(&"reduce_strength_all_temp") {
        let amount = card.base_magic.max(1);
        let living = engine.state.living_enemy_indices();
        for idx in living {
            let current = engine.state.enemies[idx].entity.strength();
            engine.state.enemies[idx].entity.set_status(sid::STRENGTH, current - amount);
            // Track for restoration at end of turn
            engine.state.enemies[idx].entity.add_status(sid::TEMP_STRENGTH_LOSS, amount);
        }
    }

    // ---- Heal: restore HP (Bandage Up) ----
    if card.effects.contains(&"heal") {
        let amount = card.base_magic.max(1);
        engine.heal_player(amount);
    }

    // ---- Heal on Play: same as heal (Bite) ----
    if card.effects.contains(&"heal_on_play") {
        let amount = card.base_magic.max(1);
        engine.heal_player(amount);
    }

    // ---- Intangible: gain Intangible (Apparition/Ghostly) ----
    if card.effects.contains(&"intangible") {
        engine.state.player.add_status(sid::INTANGIBLE, 1);
    }

    // ---- Exhaust Choose: player chooses 1 card from hand to exhaust ----
    if card.effects.contains(&"exhaust_choose") {
        if !engine.state.hand.is_empty() {
            let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.hand.len())
                .map(|i| crate::engine::ChoiceOption::HandCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::ExhaustFromHand,
                options,
                1,
                1,
            );
            return;
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
                engine.state.player.add_status(sid::STRENGTH, amount);
            }
        }
    }

    // ---- Fiend Fire: exhaust all hand cards, deal damage per card exhausted ----
    if card.effects.contains(&"fiend_fire") {
        let hand_count = engine.state.hand.len() as i32;
        // Exhaust all cards from hand
        let exhausted_cards: Vec<CardInstance> = engine.state.hand.drain(..).collect();
        engine.state.exhaust_pile.extend(exhausted_cards);
        // Deal base_damage per card exhausted to the target
        if hand_count > 0 && target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let weak_paper_crane = engine.state.has_relic("Paper Crane");
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
            let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
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
        engine.state.player.add_status(sid::ENERGIZED, card.base_magic);
    }

    // ---- Next-turn block (Dodge and Roll) ----
    if card.effects.contains(&"next_turn_block") {
        engine.state.player.add_status(sid::NEXT_TURN_BLOCK, card.base_magic);
    }

    // ---- Draw next turn (Predator) ----
    if card.effects.contains(&"draw_next_turn") {
        engine.state.player.add_status(sid::DRAW_CARD, card.base_magic);
    }

    // ---- Double Tap: next Attack played twice ----
    if card.effects.contains(&"double_tap") {
        engine.state.player.set_status(sid::DOUBLE_TAP, card.base_magic.max(1));
    }

    // ---- Burst: next Skill played twice ----
    if card.effects.contains(&"burst") {
        engine.state.player.set_status(sid::BURST, card.base_magic.max(1));
    }

    // ---- Second Wind: exhaust all non-attack cards in hand, gain block per exhaust ----
    if card.effects.contains(&"second_wind") {
        let block_per = card.base_block.max(5);
        let dex = engine.state.player.dexterity();
        let frail = engine.state.player.is_frail();
        let mut to_exhaust = Vec::new();
        let mut remaining = Vec::new();
        for hand_card in engine.state.hand.drain(..) {
            let is_attack = engine.card_registry.card_def_by_id(hand_card.def_id).card_type == CardType::Attack;
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
            engine.gain_block_player(block);
        }
    }

    // ====================================================================
    // Skill/Attack status setters (NOT handled by install_power)
    // Power cards are handled by install_power() via the power registry.
    // ====================================================================

    // ---- Flame Barrier (Skill): deal damage back when hit ----
    if card.effects.contains(&"flame_barrier") {
        engine.state.player.add_status(sid::FLAME_BARRIER, card.base_magic.max(1));
    }

    // ====================================================================
    // Card manipulation effects
    // ====================================================================

    // ---- Calculated Gamble: discard hand, draw same count ----
    if card.effects.contains(&"calculated_gamble") {
        let hand_count = engine.state.hand.len() as i32;
        let discarded: Vec<CardInstance> = engine.state.hand.drain(..).collect();
        engine.state.discard_pile.extend(discarded);
        if hand_count > 0 {
            engine.draw_cards(hand_count);
        }
    }

    // ---- Exhaust non-attacks from hand ----
    if card.effects.contains(&"exhaust_non_attacks") {
        let mut remaining = Vec::new();
        for hand_card in engine.state.hand.drain(..) {
            let def = engine.card_registry.card_def_by_id(hand_card.def_id);
            if def.card_type == CardType::Attack {
                remaining.push(hand_card);
            } else {
                engine.state.exhaust_pile.push(hand_card);
            }
        }
        engine.state.hand = remaining;
    }

    // ---- Discard non-attacks from hand ----
    if card.effects.contains(&"discard_non_attacks") {
        let mut remaining = Vec::new();
        for hand_card in engine.state.hand.drain(..) {
            let def = engine.card_registry.card_def_by_id(hand_card.def_id);
            if def.card_type == CardType::Attack {
                remaining.push(hand_card);
            } else {
                engine.state.discard_pile.push(hand_card);
            }
        }
        engine.state.hand = remaining;
    }

    // ---- Exhume: pick card from exhaust pile to return to hand ----
    if card.effects.contains(&"exhume") {
        if !engine.state.exhaust_pile.is_empty() {
            let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.exhaust_pile.len())
                .map(|i| crate::engine::ChoiceOption::ExhaustCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::PickFromExhaust,
                options,
                1,
                1,
            );
            return;
        }
    }

    // ---- Dual Wield: copy a card from hand ----
    if card.effects.contains(&"dual_wield") {
        let copies = card.base_magic.max(1) as usize;
        if !engine.state.hand.is_empty() && engine.state.hand.len() + copies <= 10 {
            let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.hand.len())
                .map(|i| crate::engine::ChoiceOption::HandCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::DualWield,
                options,
                1,
                copies,
            );
            return;
        }
    }

    // ====================================================================
    // Card generation effects
    // ====================================================================

    // ---- Add Shivs to hand ----
    if card.effects.contains(&"add_shiv_to_hand") || card.effects.contains(&"add_shivs") {
        let count = card.base_magic.max(1);
        for _ in 0..count {
            if engine.state.hand.len() >= 10 { break; }
            let shiv = engine.temp_card("Shiv");
            engine.state.hand.push(shiv);
        }
    }

    // ---- Add Wound to discard ----
    if card.effects.contains(&"add_wound_to_discard") {
        let count = card.base_magic.max(1);
        for _ in 0..count {
            let wound = engine.temp_card("Wound");
            engine.state.discard_pile.push(wound);
        }
    }

    // ---- Add Burn to discard ----
    if card.effects.contains(&"add_burn_to_discard") {
        let count = card.base_magic.max(1);
        for _ in 0..count {
            let burn = engine.temp_card("Burn");
            engine.state.discard_pile.push(burn);
        }
    }

    // ---- Add Dazed to discard ----
    if card.effects.contains(&"add_dazed_to_discard") {
        let count = card.base_magic.max(1);
        for _ in 0..count {
            let dazed = engine.temp_card("Dazed");
            engine.state.discard_pile.push(dazed);
        }
    }

    // ---- Add Wound to DRAW pile (Wild Strike) ----
    if card.effects.contains(&"add_wound_to_draw") {
        let count = card.base_magic.max(1);
        for _ in 0..count {
            let wound = engine.temp_card("Wound");
            engine.state.draw_pile.push(wound);
        }
    }

    // ---- Add Dazed to DRAW pile (Reckless Charge) ----
    if card.effects.contains(&"add_dazed_to_draw") {
        let count = card.base_magic.max(1);
        for _ in 0..count {
            let dazed = engine.temp_card("Dazed");
            engine.state.draw_pile.push(dazed);
        }
    }

    // ---- Add Void to discard ----
    if card.effects.contains(&"add_void_to_discard") {
        let count = card.base_magic.max(1);
        for _ in 0..count {
            let void_card = engine.temp_card("Void");
            engine.state.discard_pile.push(void_card);
        }
    }

    // ---- Storm of Steel: discard hand, add Shiv per card discarded (upgraded: Shiv+) ----
    if card.effects.contains(&"storm_of_steel") {
        let hand_count = engine.state.hand.len();
        let discarded: Vec<CardInstance> = engine.state.hand.drain(..).collect();
        engine.state.discard_pile.extend(discarded);
        let shiv_name = if card.id.ends_with('+') { "Shiv+" } else { "Shiv" };
        for _ in 0..hand_count {
            if engine.state.hand.len() >= 10 { break; }
            let shiv = engine.temp_card(shiv_name);
            engine.state.hand.push(shiv);
        }
    }

    // ====================================================================
    // Conditional damage effects
    // ====================================================================

    // ---- Bane: double damage if target poisoned ----
    if card.effects.contains(&"double_if_poisoned") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            if engine.state.enemies[tidx].entity.status(sid::POISON) > 0 {
                // Deal base damage again (already dealt once in main damage section)
                let player_strength = engine.state.player.strength();
                let player_weak = engine.state.player.is_weak();
                let stance_mult = engine.state.stance.outgoing_mult();
                let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
                let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
                let dmg = damage::calculate_damage(
                    card.base_damage, player_strength + vigor, player_weak,
                    stance_mult, enemy_vuln, enemy_intangible,
                );
                engine.deal_damage_to_enemy(tidx, dmg);
            }
        }
    }

    // ---- Finisher: damage per attack played this turn ----
    if card.effects.contains(&"finisher") {
        let attacks = engine.state.attacks_played_this_turn;
        if attacks > 1 && target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
            let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
            let dmg = damage::calculate_damage(
                card.base_damage, player_strength + vigor, player_weak,
                stance_mult, enemy_vuln, enemy_intangible,
            );
            // Already dealt 1 hit in main damage; deal (attacks - 1) more
            for _ in 0..(attacks - 1) {
                if engine.state.enemies[tidx].entity.is_dead() { break; }
                engine.deal_damage_to_enemy(tidx, dmg);
            }
        }
    }

    // ---- Flechettes: damage per Skill in hand ----
    if card.effects.contains(&"flechettes") {
        let skill_count = engine.state.hand.iter()
            .filter(|c| engine.card_registry.card_def_by_id(c.def_id).card_type == CardType::Skill)
            .count() as i32;
        if skill_count > 0 && target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
            let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
            let dmg = damage::calculate_damage(
                card.base_damage, player_strength + vigor, player_weak,
                stance_mult, enemy_vuln, enemy_intangible,
            );
            for _ in 0..skill_count {
                if engine.state.enemies[tidx].entity.is_dead() { break; }
                engine.deal_damage_to_enemy(tidx, dmg);
            }
        }
    }

    // ====================================================================
    // Energy / cost manipulation
    // ====================================================================

    // ---- Enlightenment: reduce hand card costs to 1 this turn ----
    if card.effects.contains(&"enlightenment") {
        for hand_card in &mut engine.state.hand {
            let def = engine.card_registry.card_def_by_id(hand_card.def_id);
            if def.cost > 1 {
                hand_card.cost = 1;
            }
        }
    }

    // ---- Madness: random card in hand costs 0 this combat ----
    if card.effects.contains(&"madness") {
        let eligible: Vec<usize> = engine.state.hand.iter()
            .enumerate()
            .filter(|(_, c)| {
                let def = engine.card_registry.card_def_by_id(c.def_id);
                def.cost > 0
            })
            .map(|(i, _)| i)
            .collect();
        if !eligible.is_empty() {
            let idx = eligible[engine.rng_gen_range(0..eligible.len())];
            engine.state.hand[idx].cost = 0;
        }
    }

    // ---- Havoc: play top card of draw pile for free ----
    if card.effects.contains(&"play_top_card") {
        if !engine.state.draw_pile.is_empty() {
            let top = engine.state.draw_pile.pop().unwrap();
            let def = engine.card_registry.card_def_by_id(top.def_id).clone();
            // Pick a valid target
            let target = if def.target == CardTarget::Enemy {
                let living = engine.state.living_enemy_indices();
                if living.is_empty() { -1 } else { living[0] as i32 }
            } else {
                -1
            };
            // Execute the card effects directly (free play)
            execute_card_effects(engine, &def, top, target);
            engine.state.discard_pile.push(top);
        }
    }

    // ---- Upgrade all cards in hand (Apotheosis) ----
    if card.effects.contains(&"upgrade_all_cards") {
        for hand_card in &mut engine.state.hand {
            if !hand_card.is_upgraded() {
                engine.card_registry.upgrade_card(hand_card);
            }
        }
    }

    // ---- Upgrade one card in hand (Armaments) -- choice ----
    if card.effects.contains(&"upgrade_one_card") {
        let upgradeable: Vec<usize> = engine.state.hand.iter()
            .enumerate()
            .filter(|(_, c)| !c.is_upgraded())
            .map(|(i, _)| i)
            .collect();
        if !upgradeable.is_empty() {
            let options: Vec<crate::engine::ChoiceOption> = upgradeable.iter()
                .map(|&i| crate::engine::ChoiceOption::HandCard(i))
                .collect();
            engine.begin_choice(
                crate::engine::ChoiceReason::UpgradeCard,
                options,
                1,
                1,
            );
            return;
        }
    }

    // ---- Gain orb slots (Capacitor) — Power cards handled by install_power ----
    // gain_orb_slots is already handled in engine.rs install_power() for Powers.
    // This handles non-Power uses (if any).
    if card.card_type != CardType::Power && card.effects.contains(&"gain_orb_slots") {
        let amount = card.base_magic.max(1);
        for _ in 0..amount {
            engine.state.orb_slots.add_slot();
        }
    }

    // ---- Channel random orb ----
    if card.effects.contains(&"channel_random") {
        let orb_types = [OrbType::Lightning, OrbType::Frost, OrbType::Dark, OrbType::Plasma];
        let idx = engine.rng_gen_range(0..orb_types.len());
        let focus = engine.state.player.focus();
        let evoke = engine.state.orb_slots.channel(orb_types[idx], focus);
        engine.apply_evoke_effect(evoke);
    }

    // ---- Evoke all orbs ----
    if card.effects.contains(&"evoke_all") {
        let focus = engine.state.player.focus();
        let effects = engine.state.orb_slots.evoke_all(focus);
        for effect in effects {
            engine.apply_evoke_effect(effect);
        }
    }

    // ---- Trigger all orb passives ----
    if card.effects.contains(&"trigger_all_passives") {
        let focus = engine.state.player.focus();
        for i in 0..engine.state.orb_slots.slots.len() {
            let orb = &engine.state.orb_slots.slots[i];
            if orb.is_empty() { continue; }
            let passive_val = orb.passive_with_focus(focus);
            match orb.orb_type {
                OrbType::Frost => {
                    engine.state.player.block += passive_val;
                }
                OrbType::Lightning => {
                    let living = engine.state.living_enemy_indices();
                    if let Some(&idx) = living.first() {
                        engine.deal_damage_to_enemy(idx, passive_val);
                    }
                }
                OrbType::Plasma => {
                    engine.state.energy += passive_val;
                }
                OrbType::Dark => {
                    // Dark passive increases its own evoke amount
                    engine.state.orb_slots.slots[i].evoke_amount += passive_val;
                }
                _ => {}
            }
        }
    }

    // ---- Choke: deal damage each time enemy plays card (status) ----
    if card.effects.contains(&"choke") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            engine.state.enemies[target_idx as usize]
                .entity
                .add_status(sid::CONSTRICTED, amount);
        }
    }

    // ---- Plated Armor: gain N Plated Armor ----
    if card.effects.contains(&"plated_armor") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status(sid::PLATED_ARMOR, amount);
    }

    // ---- Apply Lock-On to target (for orb focus bonus) ----
    if card.effects.contains(&"lock_on") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            engine.state.enemies[target_idx as usize]
                .entity
                .add_status(sid::LOCK_ON, amount);
        }
    }

    // ---- Claw scaling: each play increases future Claw damage ----
    if card.effects.contains(&"claw_scaling") {
        engine.state.player.add_status(sid::GENERIC_STRENGTH_UP, 2);
    }

    // ====================================================================
    // PR4: Per-card scaling (post-play updates) + card generation
    // ====================================================================

    // ---- Rampage: +5 bonus damage each play (or +8 upgraded) ----
    if card.effects.contains(&"rampage") {
        let increment = card.base_magic.max(5);
        engine.state.player.add_status(sid::RAMPAGE_BONUS, increment);
    }

    // ---- Glass Knife: -2 damage each play ----
    if card.effects.contains(&"glass_knife") {
        engine.state.player.add_status(sid::GLASS_KNIFE_PENALTY, 2);
    }

    // ---- Genetic Algorithm: +2 block each play (exhaust) ----
    if card.effects.contains(&"genetic_algorithm") {
        engine.state.player.add_status(sid::GENETIC_ALG_BONUS, 2);
    }

    // ---- Ritual Dagger: +3 bonus damage on kill (or +5 upgraded) ----
    if card.effects.contains(&"ritual_dagger") && enemy_killed {
        let increment = card.base_magic.max(3);
        engine.state.player.add_status(sid::RITUAL_DAGGER_BONUS, increment);
    }

    // ---- Reduce cost each play (Streamline): reduce this card's cost by 1 ----
    if card.effects.contains(&"reduce_cost_each_play") {
        // Find matching cards in draw/discard piles and reduce cost
        let def_id = card_inst.def_id;
        for pile_card in engine.state.draw_pile.iter_mut()
            .chain(engine.state.discard_pile.iter_mut())
        {
            if pile_card.def_id == def_id && pile_card.cost > 0 {
                pile_card.cost -= 1;
            }
        }
    }

    // ---- Add random colorless card to hand (Jack of All Trades) ----
    if card.effects.contains(&"add_random_colorless") {
        // MCTS: use Smite as representative colorless attack
        let temp = engine.temp_card("Smite");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(temp);
        }
    }

    // ---- Random attack to hand at 0 cost (Infernal Blade) ----
    if card.effects.contains(&"random_attack_to_hand") {
        // MCTS: use Strike as representative, set cost to 0
        let mut temp = engine.temp_card("Strike_R");
        temp.cost = 0;
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(temp);
        }
    }

    // ---- Random skill to hand at 0 cost (Distraction) ----
    if card.effects.contains(&"random_skill_to_hand") {
        // MCTS: use Defend as representative, set cost to 0
        let mut temp = engine.temp_card("Defend_G");
        temp.cost = 0;
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(temp);
        }
    }

    // ---- Draw attacks from draw pile (Violence) ----
    if card.effects.contains(&"draw_attacks_from_draw") {
        let count = card.base_magic.max(1) as usize;
        let mut drawn = 0;
        // Find attacks in draw pile and move to hand
        let mut i = engine.state.draw_pile.len();
        while i > 0 && drawn < count {
            i -= 1;
            let is_attack = {
                let def = engine.card_registry.card_def_by_id(engine.state.draw_pile[i].def_id);
                def.card_type == CardType::Attack
            };
            if is_attack && engine.state.hand.len() < 10 {
                let c = engine.state.draw_pile.remove(i);
                engine.state.hand.push(c);
                drawn += 1;
            }
        }
    }

    // ---- Add random attacks to draw pile (Metamorphosis) ----
    if card.effects.contains(&"add_random_attacks_to_draw") {
        let count = card.base_magic.max(3);
        for _ in 0..count {
            let temp = engine.temp_card("Strike_R");
            engine.state.draw_pile.push(temp);
        }
    }

    // ---- Add random skills to draw pile (Chrysalis) ----
    if card.effects.contains(&"add_random_skills_to_draw") {
        let count = card.base_magic.max(3);
        for _ in 0..count {
            let temp = engine.temp_card("Defend_G");
            engine.state.draw_pile.push(temp);
        }
    }

    // ---- Transmutation: add X random colorless cards to hand ----
    if card.effects.contains(&"transmutation") {
        let count = if card.cost == -1 { x_value } else { card.base_magic.max(1) };
        for _ in 0..count {
            let temp = engine.temp_card("Smite");
            if engine.state.hand.len() < 10 {
                engine.state.hand.push(temp);
            }
        }
    }

    // ---- Alchemize: gain a random potion (MCTS: no-op, potions are run-level) ----
    // Potions are managed at the run level, not combat level.
    // For MCTS purposes, this is effectively a no-op since potion slots
    // are tracked outside the combat state.

    // ====================================================================
    // PR2: Simple effect handlers (no choices, no hooks needed)
    // ====================================================================

    // ---- Lose HP (Hemokinesis, Offering) ----
    if card.effects.contains(&"lose_hp") {
        engine.player_lose_hp(card.base_magic);
    }

    // ---- Lose HP + gain energy (Bloodletting) ----
    if card.effects.contains(&"lose_hp_gain_energy") {
        engine.player_lose_hp(card.base_magic);
        engine.state.energy += 2;
    }

    // ---- Lose HP + gain Strength (J.A.X.) ----
    if card.effects.contains(&"lose_hp_gain_str") {
        engine.player_lose_hp(card.base_magic);
        engine.state.player.add_status(sid::STRENGTH, card.base_magic);
    }

    // ---- Damage from draw pile size (Mind Blast) ----
    // Note: Mind Blast damage is set pre-damage section, this tag is for the flag
    // The actual damage calc happens above in the pre-damage section if present

    // ---- Draw to N cards in hand (Expertise) ----
    if card.effects.contains(&"draw_to_n") {
        let target = card.base_magic;
        let to_draw = (target - engine.state.hand.len() as i32).max(0);
        if to_draw > 0 {
            engine.draw_cards(to_draw);
        }
    }

    // ---- Draw if no attacks in hand (Impatience) ----
    if card.effects.contains(&"draw_if_no_attacks") {
        let has_attack = engine.state.hand.iter().any(|c| {
            engine.card_registry.card_def_by_id(c.def_id).card_type == CardType::Attack
        });
        if !has_attack {
            engine.draw_cards(card.base_magic);
        }
    }

    // ---- Draw if few cards played this turn (FTL) ----
    if card.effects.contains(&"draw_if_few_cards_played") {
        if engine.state.cards_played_this_turn < 3 {
            engine.draw_cards(card.base_magic);
        }
    }

    // ---- Block from discard pile size (Stack) ----
    if card.effects.contains(&"block_from_discard") {
        let block = engine.state.discard_pile.len() as i32;
        engine.gain_block_player(block);
    }

    // ---- Block only if no block (Auto Shields) ----
    if card.effects.contains(&"block_if_no_block") {
        if engine.state.player.block == 0 {
            engine.gain_block_player(card.base_block);
        }
    }

    // ---- Remove enemy block before damage (Melter) ----
    if card.effects.contains(&"remove_enemy_block") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            engine.state.enemies[target_idx as usize].entity.block = 0;
        }
    }

    // ---- No draw this turn (Battle Trance) ----
    if card.effects.contains(&"no_draw") {
        engine.state.player.set_status(sid::NO_DRAW, 1);
    }

    // ---- Shuffle discard into draw (Deep Breath) ----
    if card.effects.contains(&"shuffle_discard_into_draw") {
        let mut cards = std::mem::take(&mut engine.state.discard_pile);
        engine.state.draw_pile.append(&mut cards);
        engine.shuffle_draw_pile();
    }

    // ---- Energy from draw pile size (Aggregate) ----
    if card.effects.contains(&"energy_per_cards_in_draw") {
        engine.state.energy += engine.state.draw_pile.len() as i32 / 4;
    }

    // ---- Add Wounds to hand (Power Through) ----
    if card.effects.contains(&"add_wounds_to_hand") {
        let count = card.base_magic.max(1);
        for _ in 0..count {
            if engine.state.hand.len() >= 10 { break; }
            let wound = engine.temp_card("Wound");
            engine.state.hand.push(wound);
        }
    }

    // ---- Poison random enemy multiple times (Bouncing Flask) ----
    if card.effects.contains(&"poison_random_multi") {
        let applications = card.base_magic.max(1);
        let poison_per = 3; // Bouncing Flask applies 3 poison per bounce
        for _ in 0..applications {
            let living = engine.state.living_enemy_indices();
            if living.is_empty() { break; }
            let idx = if living.len() == 1 { 0 } else {
                engine.rng_gen_range(0..living.len())
            };
            let target = living[idx];
            engine.state.enemies[target].entity.add_status(sid::POISON, poison_per);
        }
    }

    // ---- Weak if attacking (Go for the Eyes) ----
    if card.effects.contains(&"weak_if_attacking") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let enemy = &engine.state.enemies[target_idx as usize];
            let is_attacking = enemy.move_damage() > 0;
            if is_attacking {
                crate::powers::apply_debuff(
                    &mut engine.state.enemies[target_idx as usize].entity,
                    sid::WEAKENED,
                    card.base_magic,
                );
            }
        }
    }

    // ---- If vulnerable: gain energy + draw (Dropkick) ----
    if card.effects.contains(&"if_vulnerable_energy_draw") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            if engine.state.enemies[target_idx as usize].entity.is_vulnerable() {
                engine.state.energy += 1;
                engine.draw_cards(1);
            }
        }
    }

    // ---- If weak: gain energy + draw (Heel Hook) ----
    if card.effects.contains(&"if_weak_energy_draw") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            if engine.state.enemies[target_idx as usize].entity.status(sid::WEAKENED) > 0 {
                engine.state.energy += 1;
                engine.draw_cards(1);
            }
        }
    }

    // ---- Temporary Strength reduction (Dark Shackles) ----
    if card.effects.contains(&"reduce_str_this_turn") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic;
            engine.state.enemies[target_idx as usize].entity.add_status(sid::STRENGTH, -amount);
            engine.state.enemies[target_idx as usize].entity.add_status(sid::LOSE_STRENGTH, amount);
        }
    }

    // ---- Discard random card (All-Out Attack) ----
    if card.effects.contains(&"discard_random") {
        if !engine.state.hand.is_empty() {
            let idx = engine.rng_gen_range(0..engine.state.hand.len());
            let card = engine.state.hand.remove(idx);
            engine.state.discard_pile.push(card);
        }
    }

    // ---- Retain block / Blur ----
    if card.effects.contains(&"retain_block") {
        engine.state.player.add_status(sid::BLUR, card.base_magic.max(1));
    }

    // ---- The Bomb: install bomb status (countdown hook already in registry) ----
    if card.effects.contains(&"the_bomb") {
        engine.state.player.add_status(sid::THE_BOMB, card.base_magic);
    }

    // ---- Enlightenment this turn (reduce all hand costs to 1 this turn) ----
    if card.effects.contains(&"enlightenment_this_turn") {
        for hand_card in engine.state.hand.iter_mut() {
            if hand_card.cost > 1 {
                hand_card.cost = 1;
            }
        }
    }

    // ---- Enlightenment permanent (reduce all hand costs to 1 permanently) ----
    if card.effects.contains(&"enlightenment_permanent") {
        for hand_card in engine.state.hand.iter_mut() {
            if hand_card.cost > 1 {
                hand_card.cost = 1;
            }
        }
    }

    // ---- Apply Lock-On (Bullseye uses "apply_lock_on" tag) ----
    if card.effects.contains(&"apply_lock_on") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            engine.state.enemies[target_idx as usize]
                .entity
                .add_status(sid::LOCK_ON, amount);
        }
    }

    // ====================================================================
    // PR5: Choice-based card effects
    // ====================================================================

    // ---- Search draw pile for Attack (Secret Weapon) ----
    if card.effects.contains(&"search_attack") {
        let options: Vec<_> = engine.state.draw_pile.iter()
            .enumerate()
            .filter(|(_, c)| {
                engine.card_registry.card_def_by_id(c.def_id).card_type == CardType::Attack
            })
            .map(|(i, _)| ChoiceOption::DrawCard(i))
            .collect();
        engine.begin_choice(ChoiceReason::SearchDrawPile, options, 1, 1);
    }

    // ---- Search draw pile for Skill (Secret Technique) ----
    if card.effects.contains(&"search_skill") {
        let options: Vec<_> = engine.state.draw_pile.iter()
            .enumerate()
            .filter(|(_, c)| {
                engine.card_registry.card_def_by_id(c.def_id).card_type == CardType::Skill
            })
            .map(|(i, _)| ChoiceOption::DrawCard(i))
            .collect();
        engine.begin_choice(ChoiceReason::SearchDrawPile, options, 1, 1);
    }

    // ---- Return card from discard to hand (Hologram) ----
    if card.effects.contains(&"return_from_discard") {
        let options: Vec<_> = engine.state.discard_pile.iter()
            .enumerate()
            .map(|(i, _)| ChoiceOption::DiscardCard(i))
            .collect();
        engine.begin_choice(ChoiceReason::ReturnFromDiscard, options, 1, 1);
    }

    // ---- Forethought: put 1 card from hand to bottom of draw at cost 0 ----
    if card.effects.contains(&"forethought") {
        let options: Vec<_> = engine.state.hand.iter()
            .enumerate()
            .map(|(i, _)| ChoiceOption::HandCard(i))
            .collect();
        engine.begin_choice(ChoiceReason::ForethoughtPick, options, 1, 1);
    }

    // ---- Forethought+: put ALL hand cards to bottom of draw at cost 0 ----
    if card.effects.contains(&"forethought_all") {
        // Auto-resolve: move all hand cards to bottom of draw at cost 0
        let hand_cards: Vec<_> = engine.state.hand.drain(..).collect();
        for mut c in hand_cards {
            c.cost = 0;
            engine.state.draw_pile.push(c);
        }
    }

    // ---- Recycle: exhaust 1 card from hand, gain its cost as energy ----
    if card.effects.contains(&"recycle") {
        let options: Vec<_> = engine.state.hand.iter()
            .enumerate()
            .map(|(i, _)| ChoiceOption::HandCard(i))
            .collect();
        engine.begin_choice(ChoiceReason::RecycleCard, options, 1, 1);
    }

    // ---- Discard N cards, gain energy (Concentrate) ----
    if card.effects.contains(&"discard_gain_energy") {
        let discard_count = card.base_magic.max(1) as usize;
        let options: Vec<_> = engine.state.hand.iter()
            .enumerate()
            .map(|(i, _)| ChoiceOption::HandCard(i))
            .collect();
        let actual_picks = discard_count.min(options.len());
        engine.begin_choice(ChoiceReason::DiscardForEffect, options, actual_picks, actual_picks);
    }

    // ---- Exhaust N from hand (Purity) ----
    if card.effects.contains(&"exhaust_from_hand") {
        let exhaust_count = card.base_magic.max(1) as usize;
        let options: Vec<_> = engine.state.hand.iter()
            .enumerate()
            .map(|(i, _)| ChoiceOption::HandCard(i))
            .collect();
        let actual_picks = exhaust_count.min(options.len());
        engine.begin_choice(ChoiceReason::ExhaustFromHand, options, 0, actual_picks);
    }

    // ---- Setup: pick card from hand, set cost 0, put on top of draw ----
    if card.effects.contains(&"setup") {
        let options: Vec<_> = engine.state.hand.iter()
            .enumerate()
            .map(|(i, _)| ChoiceOption::HandCard(i))
            .collect();
        engine.begin_choice(ChoiceReason::SetupPick, options, 1, 1);
    }

    // ---- Thinking Ahead: draw 2, then put 1 card on top of draw ----
    if card.effects.contains(&"thinking_ahead") {
        engine.draw_cards(2);
        let options: Vec<_> = engine.state.hand.iter()
            .enumerate()
            .map(|(i, _)| ChoiceOption::HandCard(i))
            .collect();
        engine.begin_choice(ChoiceReason::PutOnTopFromHand, options, 1, 1);
    }

    // ====================================================================
    // PR6: Power installs + dynamic cost + misc
    // ====================================================================

    // ---- Phantasmal Killer: set DOUBLE_DAMAGE for next turn ----
    if card.effects.contains(&"phantasmal_killer") {
        engine.state.player.add_status(sid::DOUBLE_DAMAGE, 1);
    }

    // ---- Biased Cognition: gain Focus now, lose 1 Focus each turn ----
    if card.effects.contains(&"lose_focus_each_turn") {
        engine.state.player.add_status(sid::BIASED_COG_FOCUS_LOSS, 1);
    }
    // gain_focus is already handled by existing gain_focus handler

    // ---- Amplify: next Power played this turn is doubled ----
    if card.effects.contains(&"amplify_power") {
        engine.state.player.add_status(sid::AMPLIFY, 1);
    }

    // ---- Self Repair: heal at end of combat ----
    if card.effects.contains(&"heal_end_of_combat") {
        let amount = card.base_magic.max(7);
        engine.state.player.add_status(sid::SELF_REPAIR, amount);
    }

    // ---- Corpse Explosion: mark enemy, on death deal max_hp to all enemies ----
    if card.effects.contains(&"corpse_explosion") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let amount = card.base_magic.max(1);
            engine.state.enemies[target_idx as usize]
                .entity
                .add_status(sid::CORPSE_EXPLOSION, amount);
        }
    }

    // ---- Equilibrium: retain entire hand this turn ----
    if card.effects.contains(&"retain_hand") {
        engine.state.player.set_status(sid::RETAIN_HAND_FLAG, 1);
    }

    // ---- Sentinel: gain energy when this card is exhausted ----
    // Sentinel only exhausts under Corruption (all Skills exhaust on play).
    // If Corruption is active, the card will be routed to exhaust pile and
    // trigger_on_exhaust fires, so we grant energy here proactively.
    if card.effects.contains(&"energy_on_exhaust") {
        if engine.state.player.status(sid::CORRUPTION) > 0 {
            let amount = card.base_magic.max(2);
            engine.state.energy += amount;
        }
    }

    // ---- Escape Plan: draw 1, if Skill gain block ----
    if card.effects.contains(&"block_if_skill") {
        // The "draw" tag already drew a card. Check if the drawn card is a Skill.
        // Look at last card in hand (the one just drawn).
        if !engine.state.hand.is_empty() {
            let last = engine.state.hand.last().unwrap();
            let last_type = engine.card_registry.card_def_by_id(last.def_id).card_type;
            if last_type == CardType::Skill {
                let dex = engine.state.player.dexterity();
                let frail = engine.state.player.is_frail();
                let block = damage::calculate_block(card.base_block.max(0), dex, frail);
                engine.gain_block_player(block);
            }
        }
    }

    // ---- Sneaky Strike: refund energy if discarded this turn ----
    if card.effects.contains(&"refund_energy_on_discard") {
        if engine.state.player.status(sid::DISCARDED_THIS_TURN) > 0 {
            engine.state.energy += 2;
        }
    }
}
