//! Card effect execution — the big match on effect tags.
//!
//! Extracted from engine.rs as a pure refactor. All card-specific logic
//! (damage, block, draw, scry, mantra, vigor, pen nib, etc.) lives here.

use crate::cards::{CardDef, CardTarget, CardType};
use crate::damage;
use crate::engine::CombatEngine;
use crate::powers;
use crate::state::Stance;

/// Execute all effects for a card that was just played.
///
/// Called from `CombatEngine::play_card()` after energy payment and hand removal.
pub fn execute_card_effects(engine: &mut CombatEngine, card: &CardDef, card_id: &str, target_idx: i32) {
    // ---- Pen Nib check (before damage) ----
    let pen_nib_active = if card.card_type == CardType::Attack {
        crate::relics::check_pen_nib(&mut engine.state)
    } else {
        false
    };

    // ---- Vigor (consumed on first attack hit) ----
    let vigor = if card.card_type == CardType::Attack {
        let v = engine.state.player.status("Vigor");
        if v > 0 {
            engine.state.player.set_status("Vigor", 0);
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

    // ---- Damage ----
    // Track damage dealt for Wallop (block_from_damage)
    let mut total_unblocked_damage = 0i32;
    let mut enemy_killed = false;

    if card.base_damage >= 0 {
        let is_multi_hit = card.effects.contains(&"multi_hit");
        let hits = if is_multi_hit && card.base_magic > 0 {
            card.base_magic
        } else {
            1
        };

        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let weak_paper_crane = engine.state.has_relic("Paper Crane");
        let stance_mult = engine.state.stance.outgoing_mult();

        match card.target {
            CardTarget::Enemy => {
                if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
                    let tidx = target_idx as usize;
                    let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
                    let enemy_intangible = engine.state.enemies[tidx].entity.status("Intangible") > 0;
                    let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                    let dmg = damage::calculate_damage_full(
                        card.base_damage + brilliance_bonus,
                        player_strength,
                        vigor,
                        player_weak,
                        weak_paper_crane,
                        pen_nib_active,
                        false, // double_damage
                        stance_mult,
                        enemy_vuln,
                        vuln_paper_frog,
                        false, // flight
                        enemy_intangible,
                    );
                    // Talk to the Hand: player gains block per hit ONLY on HP damage
                    let block_return = engine.state.enemies[tidx].entity.status("BlockReturn");
                    for _ in 0..hits {
                        let enemy_block_before = engine.state.enemies[tidx].entity.block;
                        let enemy_hp_before = engine.state.enemies[tidx].entity.hp;
                        engine.deal_damage_to_enemy(tidx, dmg);
                        // Track unblocked damage for Wallop
                        let _blocked = enemy_block_before.min(dmg);
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
                    let enemy_intangible = engine.state.enemies[enemy_idx].entity.status("Intangible") > 0;
                    let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                    let dmg = damage::calculate_damage_full(
                        card.base_damage + brilliance_bonus,
                        player_strength,
                        vigor,
                        player_weak,
                        weak_paper_crane,
                        pen_nib_active,
                        false, // double_damage
                        stance_mult,
                        enemy_vuln,
                        vuln_paper_frog,
                        false, // flight
                        enemy_intangible,
                    );
                    let block_return = engine.state.enemies[enemy_idx].entity.status("BlockReturn");
                    for _ in 0..hits {
                        let enemy_hp_before = engine.state.enemies[enemy_idx].entity.hp;
                        let enemy_block_before = engine.state.enemies[enemy_idx].entity.block;
                        engine.deal_damage_to_enemy(enemy_idx, dmg);
                        total_unblocked_damage += (enemy_hp_before - engine.state.enemies[enemy_idx].entity.hp).max(0);
                        if block_return > 0 {
                            let _blocked = enemy_block_before.min(dmg);
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

    // ---- Block ----
    if card.base_block >= 0 {
        let dex = engine.state.player.dexterity();
        let frail = engine.state.player.is_frail();
        let block = damage::calculate_block(card.base_block, dex, frail);
        engine.state.player.block += block;
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
    if card.effects.contains(&"draw") && card.base_magic > 0 {
        engine.draw_cards(card.base_magic);
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
        engine.state.player.add_status("Vigor", card.base_magic);
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
                    .add_status("Vulnerable", vuln_amount);
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
                    "Weakened",
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
                .add_status("BlockReturn", amount);
        }
    }

    // ---- Ragnarok: deal damage to random enemies X times ----
    if card.effects.contains(&"damage_random_x_times") && card.base_magic > 0 {
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let stance_mult = engine.state.stance.outgoing_mult();
        for _ in 0..(card.base_magic - 1) {
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
                .add_status("Mark", mark_amount);
        }
        // Deal damage to ALL enemies equal to their Mark
        let living = engine.state.living_enemy_indices();
        for idx in living {
            let mark = engine.state.enemies[idx].entity.status("Mark");
            if mark > 0 {
                // Mark damage ignores block (HP loss)
                engine.state.enemies[idx].entity.hp -= mark;
                engine.state.total_damage_dealt += mark;
                if engine.state.enemies[idx].entity.hp <= 0 {
                    engine.state.enemies[idx].entity.hp = 0;
                }
            }
        }
    }

    // ---- Judgement: if enemy HP <= threshold, set HP to 0 ----
    if card.effects.contains(&"judgement") {
        if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
            let tidx = target_idx as usize;
            let threshold = card.base_magic.max(1);
            if engine.state.enemies[tidx].entity.hp <= threshold {
                let hp = engine.state.enemies[tidx].entity.hp;
                engine.state.enemies[tidx].entity.hp = 0;
                engine.state.total_damage_dealt += hp;
            }
        }
    }

    // ---- Lesson Learned: if enemy dies, upgrade a random card in draw/discard ----
    if card.effects.contains(&"lesson_learned") && enemy_killed {
        // Find a non-upgraded card and upgrade it
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
                    "Vulnerable",
                    vuln_amount,
                );
            }
        } else {
            engine.change_stance(Stance::Wrath);
        }
    }

    // ---- Meditate: return cards from discard to hand (MCTS approximation) ----
    if card.effects.contains(&"meditate") {
        let count = card.base_magic.max(1) as usize;
        // Move best cards from discard to hand (simplified: take from end)
        // Returned cards are retained through end-of-turn discard.
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

    // ---- Wave of the Hand: apply WaveOfTheHand status ----
    if card.effects.contains(&"wave_of_the_hand") {
        engine.state.player.add_status("WaveOfTheHand", card.base_magic.max(1));
    }

    // ---- Foreign Influence: MCTS approximation, add a random Smite ----
    if card.effects.contains(&"foreign_influence") {
        let smite_id = engine.temp_card_id("Smite");
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(smite_id);
        }
    }

    // ---- Conjure Blade: create Expunger with X hits ----
    if card.effects.contains(&"conjure_blade") {
        let x_value = engine.state.energy;
        engine.state.energy = 0;
        let expunger_id = engine.temp_card_id("Expunger");
        if x_value > 0 && engine.state.hand.len() < 10 {
            engine.state.hand.push(expunger_id);
            engine.state.player.set_status("ExpungerHits", x_value);
        }
    }

    // ---- Omniscience: MCTS approximation — draw top card and play it ----
    if card.effects.contains(&"omniscience") {
        engine.draw_cards(2);
    }

    // ---- Wish: MCTS approximation — gain Strength (most useful option) ----
    if card.effects.contains(&"wish") {
        let amount = card.base_magic.max(1);
        engine.state.player.add_status("Strength", amount);
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
        engine.state.player.set_status("NextAttackFree", 1);
    }
}
