//! Card play preamble + declarative interpreter dispatch.
//!
//! Handles generic damage/block calculation for all cards, then dispatches
//! to the declarative effect interpreter and optional complex_hook.

use crate::cards::{CardDef, CardTarget, CardType};
use crate::combat_types::CardInstance;
use crate::damage;
use crate::engine::CombatEngine;
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

    // ---- Damage modifiers via registry dispatch ----
    let card_flags = engine.card_registry.effect_flags(card_inst.def_id);
    let dmg_mod = crate::effects::dispatch_modify_damage(engine, card, card_inst, card_flags);

    let body_slam_damage = dmg_mod.base_damage_override;
    let heavy_blade_mult = dmg_mod.strength_multiplier;
    // All additive bonuses (brilliance, perfected_strike, rampage, etc.) are merged
    let total_damage_bonus = dmg_mod.base_damage_bonus;

    // ---- Grand Finale: only deal damage if draw pile is empty ----
    let grand_finale_blocked = card_flags.has(crate::effects::registry::BIT_ONLY_EMPTY_DRAW)
        && !engine.state.draw_pile.is_empty();

    // ---- Genetic Algorithm: scaling block bonus ----
    let genetic_alg_block_bonus = if card.effects.contains(&"genetic_algorithm") {
        engine.state.player.status(sid::GENETIC_ALG_BONUS)
    } else {
        0
    };

    // ---- Perseverance: scaling block bonus from retaining ----
    let perseverance_block_bonus = if card_flags.has(crate::effects::registry::BIT_GROW_BLOCK_ON_RETAIN) {
        engine.state.player.status(sid::PERSEVERANCE_BONUS)
    } else {
        0
    };

    // ---- Damage ----
    // Track damage dealt for Wallop (block_from_damage) and Reaper (heal)
    let mut total_unblocked_damage = 0i32;
    let mut enemy_killed = false;

    // Skip generic damage for cards that use damage_random_x_times (they handle their own hits)
    let skip_generic_damage = dmg_mod.skip_generic_damage;

    if !skip_generic_damage && !grand_finale_blocked && (card.base_damage >= 0 || body_slam_damage >= 0) {
        let effective_base_damage = if body_slam_damage >= 0 {
            body_slam_damage
        } else {
            // total_damage_bonus includes all additive modifiers (brilliance, perfected_strike, scaling, etc.)
            (card.base_damage + total_damage_bonus).max(0)
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
                        effective_base_damage,
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
                        effective_base_damage,
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

    // ---- Standard block calculation (preamble, runs for ALL cards) ----
    // This runs BEFORE the interpreter fallthrough because block from base_block
    // is a preamble operation, not a post-damage effect.
    if card.base_block >= 0 {
        let block_multiplier = if card.effects.contains(&"block_x_times") {
            x_value
        } else {
            1
        };
        if !card.effects.contains(&"block_if_skill") {
            let dex = engine.state.player.dexterity();
            let frail = engine.state.player.is_frail();
            let block = damage::calculate_block(
                card.base_block + genetic_alg_block_bonus + perseverance_block_bonus,
                dex, frail,
            );
            engine.gain_block_player(block * block_multiplier);
        }
    }

    // ---- Declarative effect interpreter (always runs) ----
    let ctx = crate::effects::types::CardPlayContext {
        card,
        card_inst,
        target_idx,
        x_value,
        pen_nib_active,
        vigor,
        total_unblocked_damage,
        enemy_killed,
    };
    crate::effects::interpreter::execute_effects(engine, &ctx, card.effect_data);
    if let Some(hook) = card.complex_hook {
        hook(engine, &ctx);
    }
}
