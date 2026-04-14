//! Card play preamble + declarative interpreter dispatch.
//!
//! Handles generic damage/block calculation for all cards, then dispatches
//! to the declarative effect interpreter and optional complex_hook.

use crate::cards::{CardDef, CardTarget, CardType};
use crate::combat_types::CardInstance;
use crate::damage;
use crate::engine::CombatEngine;
use crate::effects::types::DamageModifier;
use crate::status_ids::sid;

fn consume_pen_nib_for_attack(engine: &mut CombatEngine) -> bool {
    if !engine.state.has_relic("Pen Nib") {
        return false;
    }

    let counter = engine.state.player.status(sid::PEN_NIB_COUNTER);
    if counter >= 9 {
        engine.state.player.set_status(sid::PEN_NIB_COUNTER, 0);
        true
    } else {
        engine.state
            .player
            .set_status(sid::PEN_NIB_COUNTER, counter + 1);
        false
    }
}

fn resolve_damage_modifiers(
    engine: &CombatEngine,
    card: &CardDef,
    card_inst: CardInstance,
    card_flags: crate::effects::EffectFlags,
) -> DamageModifier {
    use crate::effects::hooks_damage;
    use crate::effects::registry as bits;

    let mut out = DamageModifier::default();
    if card_flags.has(bits::BIT_HEAVY_BLADE) {
        out.merge(hooks_damage::hook_heavy_blade(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_DAMAGE_EQUALS_BLOCK) {
        out.merge(hooks_damage::hook_damage_equals_block(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_DAMAGE_PLUS_MANTRA) {
        out.merge(hooks_damage::hook_damage_plus_mantra(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_PERFECTED_STRIKE) {
        out.merge(hooks_damage::hook_perfected_strike(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_RAMPAGE) {
        out.merge(hooks_damage::hook_rampage(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_GLASS_KNIFE) {
        out.merge(hooks_damage::hook_glass_knife(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_RITUAL_DAGGER) {
        out.merge(hooks_damage::hook_ritual_dagger(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_SEARING_BLOW) {
        out.merge(hooks_damage::hook_searing_blow(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_DAMAGE_RANDOM_X_TIMES) {
        out.merge(hooks_damage::hook_damage_random_x_times(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_GROW_DAMAGE_ON_RETAIN) {
        out.merge(hooks_damage::hook_windmill_strike_damage(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_CLAW_SCALING) {
        out.merge(hooks_damage::hook_claw_damage(engine, card, card_inst));
    }
    if card_flags.has(bits::BIT_DAMAGE_PER_FROST)
        || card_flags.has(bits::BIT_DAMAGE_PER_LIGHTNING)
        || card_flags.has(bits::BIT_DAMAGE_FROM_DRAW_PILE)
    {
        out.merge(hooks_damage::hook_skip_generic_damage(engine, card, card_inst));
    }
    out
}

pub(crate) fn execute_primary_attack(
    engine: &mut CombatEngine,
    ctx: &mut crate::effects::types::CardPlayContext,
    target: crate::effects::declarative::Target,
) {
    let mut total_unblocked_damage = 0i32;
    let mut enemy_killed = false;
    let card = ctx.card;
    let card_inst = ctx.card_inst;
    let card_id = engine.card_registry.card_name(card_inst.def_id);
    let card_flags = engine.card_registry.effect_flags(card_inst.def_id);
    let dmg_mod = resolve_damage_modifiers(engine, card, card_inst, card_flags);

    let body_slam_damage = dmg_mod.base_damage_override;
    let heavy_blade_mult = dmg_mod.strength_multiplier;
    let mut total_damage_bonus = dmg_mod.base_damage_bonus;
    if card.card_type == CardType::Attack {
        total_damage_bonus += engine.player_attack_base_damage(card, card_inst) - card.base_damage;
    }

    if card_id == "Shiv" || card_id == "Shiv+" {
        let accuracy = engine.state.player.status(sid::ACCURACY);
        if accuracy > 0 {
            total_damage_bonus += accuracy;
        }
    }

    let grand_finale_blocked = card_flags.has(crate::effects::registry::BIT_ONLY_EMPTY_DRAW)
        && !engine.state.draw_pile.is_empty();
    if dmg_mod.skip_generic_damage || grand_finale_blocked {
        return;
    }

    let effective_base_damage = if body_slam_damage >= 0 {
        body_slam_damage
    } else {
        (card.base_damage + total_damage_bonus).max(0)
    };

    let hits = if let Some(amount_src) = card.declared_extra_hits() {
        crate::effects::interpreter::resolve_card_amount(engine, ctx, &amount_src).max(1)
    } else if card.effects.contains(&"x_cost") && card.cost == -1 {
        ctx.x_value
    } else if card.effects.contains(&"multi_hit") && card.base_magic > 0 {
        card.base_magic
    } else {
        1
    };

    let player_strength = engine.state.player.strength() * heavy_blade_mult;
    let player_weak = engine.state.player.is_weak();
    let weak_paper_crane = engine.state.has_relic("Paper Crane");
    let stance_mult = engine.state.stance.outgoing_mult();

    let double_damage = engine.state.player.status(sid::DOUBLE_DAMAGE) > 0;
    if double_damage {
        let dd = engine.state.player.status(sid::DOUBLE_DAMAGE);
        engine.state.player.set_status(sid::DOUBLE_DAMAGE, dd - 1);
    }

    match target {
        crate::effects::declarative::Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                let tidx = idx as usize;
                let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
                let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
                let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                let dmg = damage::calculate_damage_full(
                    effective_base_damage,
                    player_strength,
                    ctx.vigor,
                    player_weak,
                    weak_paper_crane,
                    ctx.pen_nib_active,
                    double_damage,
                    stance_mult,
                    enemy_vuln,
                    vuln_paper_frog,
                    false,
                    enemy_intangible,
                );
                let block_return = engine.state.enemies[tidx].entity.status(sid::BLOCK_RETURN);
                for _ in 0..hits {
                    let hp_dmg = engine.deal_player_attack_hit_to_enemy(tidx, dmg);
                    total_unblocked_damage += hp_dmg;
                    if block_return > 0 && hp_dmg > 0 {
                        engine.gain_block_player(block_return);
                    }
                    if engine.state.enemies[tidx].entity.is_dead() {
                        enemy_killed = true;
                        break;
                    }
                }
            }
        }
        crate::effects::declarative::Target::AllEnemies => {
            let living = engine.state.living_enemy_indices();
            for enemy_idx in living {
                let enemy_vuln = engine.state.enemies[enemy_idx].entity.is_vulnerable();
                let enemy_intangible = engine.state.enemies[enemy_idx].entity.status(sid::INTANGIBLE) > 0;
                let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                let dmg = damage::calculate_damage_full(
                    effective_base_damage,
                    player_strength,
                    ctx.vigor,
                    player_weak,
                    weak_paper_crane,
                    ctx.pen_nib_active,
                    double_damage,
                    stance_mult,
                    enemy_vuln,
                    vuln_paper_frog,
                    false,
                    enemy_intangible,
                );
                let block_return = engine.state.enemies[enemy_idx].entity.status(sid::BLOCK_RETURN);
                for _ in 0..hits {
                    let hp_dmg = engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
                    total_unblocked_damage += hp_dmg;
                    if block_return > 0 && hp_dmg > 0 {
                        engine.gain_block_player(block_return);
                    }
                    if engine.state.enemies[enemy_idx].entity.is_dead() {
                        enemy_killed = true;
                        break;
                    }
                }
            }
        }
        crate::effects::declarative::Target::RandomEnemy => {
            let living = engine.state.living_enemy_indices();
            if !living.is_empty() {
                let enemy_idx = living[engine.rng_gen_range(0..living.len())];
                let enemy_vuln = engine.state.enemies[enemy_idx].entity.is_vulnerable();
                let enemy_intangible = engine.state.enemies[enemy_idx].entity.status(sid::INTANGIBLE) > 0;
                let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                let dmg = damage::calculate_damage_full(
                    effective_base_damage,
                    player_strength,
                    ctx.vigor,
                    player_weak,
                    weak_paper_crane,
                    ctx.pen_nib_active,
                    double_damage,
                    stance_mult,
                    enemy_vuln,
                    vuln_paper_frog,
                    false,
                    enemy_intangible,
                );
                let block_return = engine.state.enemies[enemy_idx].entity.status(sid::BLOCK_RETURN);
                for _ in 0..hits {
                    let hp_dmg = engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
                    total_unblocked_damage += hp_dmg;
                    if block_return > 0 && hp_dmg > 0 {
                        engine.gain_block_player(block_return);
                    }
                    if engine.state.enemies[enemy_idx].entity.is_dead() {
                        enemy_killed = true;
                        break;
                    }
                }
            }
        }
        crate::effects::declarative::Target::Player | crate::effects::declarative::Target::SelfEntity => {
            for _ in 0..hits {
                engine.player_lose_hp(effective_base_damage);
            }
        }
    }

    ctx.total_unblocked_damage = total_unblocked_damage;
    ctx.enemy_killed = enemy_killed;
}

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
        consume_pen_nib_for_attack(engine)
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
    let dmg_mod = resolve_damage_modifiers(engine, card, card_inst, card_flags);

    let body_slam_damage = dmg_mod.base_damage_override;
    let heavy_blade_mult = dmg_mod.strength_multiplier;
    // All additive bonuses (brilliance, perfected_strike, rampage, etc.) are merged
    let mut total_damage_bonus = dmg_mod.base_damage_bonus;
    let is_attack = card.card_type == CardType::Attack;
    if is_attack {
        total_damage_bonus += engine.player_attack_base_damage(card, card_inst) - card.base_damage;
    }

    // Accuracy: +N damage to Shiv cards
    if card_id == "Shiv" || card_id == "Shiv+" {
        let accuracy = engine.state.player.status(sid::ACCURACY);
        if accuracy > 0 {
            total_damage_bonus += accuracy;
        }
    }

    // ---- Grand Finale: only deal damage if draw pile is empty ----
    let grand_finale_blocked = card_flags.has(crate::effects::registry::BIT_ONLY_EMPTY_DRAW)
        && !engine.state.draw_pile.is_empty();

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
    let pre_damage_ctx = crate::effects::types::CardPlayContext {
        card,
        card_inst,
        target_idx,
        x_value,
        pen_nib_active,
        vigor,
        total_unblocked_damage: 0,
        enemy_killed: false,
    };

    // Skip generic damage for cards that use damage_random_x_times (they handle their own hits)
    let skip_generic_damage = dmg_mod.skip_generic_damage;

    let has_typed_primary_attack = card.declared_primary_attack_target();
    let has_typed_primary_block = card.declared_primary_block();

    if has_typed_primary_attack.is_none()
        && !skip_generic_damage
        && !grand_finale_blocked
        && (card.base_damage >= 0 || body_slam_damage >= 0)
    {
        let effective_base_damage = if body_slam_damage >= 0 {
            body_slam_damage
        } else {
            // total_damage_bonus includes all additive modifiers (brilliance, perfected_strike, scaling, etc.)
            (card.base_damage + total_damage_bonus).max(0)
        };

        // X-cost attacks: Whirlwind = X hits AoE, Skewer = X hits single
        let hits = if let Some(amount_src) = card.declared_extra_hits() {
            crate::effects::interpreter::resolve_card_amount(engine, &pre_damage_ctx, &amount_src).max(1)
        } else if card.effects.contains(&"x_cost") && card.cost == -1 {
            x_value
        } else if card.effects.contains(&"multi_hit") && card.base_magic > 0 {
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
                        let hp_dmg = engine.deal_player_attack_hit_to_enemy(tidx, dmg);
                        total_unblocked_damage += hp_dmg;
                        // BlockReturn only triggers on actual HP damage
                        if block_return > 0 && hp_dmg > 0 {
                            engine.gain_block_player(block_return);
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
                        let hp_dmg = engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
                        total_unblocked_damage += hp_dmg;
                        if block_return > 0 && hp_dmg > 0 {
                            engine.gain_block_player(block_return);
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
    if card.base_block >= 0 && !has_typed_primary_block {
        let block_multiplier = if card.effects.contains(&"block_x_times") {
            x_value
        } else {
            1
        };
        if !card.effects.contains(&"block_if_skill") && !card.effects.contains(&"block_if_no_block") && !card.effects.contains(&"second_wind") {
            let current_base_block = if card.effects.contains(&"genetic_algorithm")
                || card.effects.contains(&"lose_block_each_play")
            {
                if card_inst.misc >= 0 {
                    card_inst.misc as i32
                } else {
                    card.base_block
                }
            } else {
                card.base_block
            };
            let dex = engine.state.player.dexterity();
            let frail = engine.state.player.is_frail();
            let block = damage::calculate_block(
                (current_base_block + perseverance_block_bonus).max(0),
                dex, frail,
            );
            engine.gain_block_player(block * block_multiplier);
        }
    }

    // ---- Declarative effect interpreter (always runs) ----
    let mut ctx = crate::effects::types::CardPlayContext {
        card,
        card_inst,
        target_idx,
        x_value,
        pen_nib_active,
        vigor,
        total_unblocked_damage,
        enemy_killed,
    };
    let prev_total_unblocked_damage = engine.runtime_card_total_unblocked_damage;
    let prev_enemy_killed = engine.runtime_card_enemy_killed;
    engine.runtime_card_total_unblocked_damage = ctx.total_unblocked_damage;
    engine.runtime_card_enemy_killed = ctx.enemy_killed;
    // ---- Exhaust random: True Grit (base) exhausts 1 random card from hand ----
    if card.effects.contains(&"exhaust_random") {
        crate::effects::hooks_complex::hook_exhaust_random(engine, &ctx);
    }

    crate::effects::interpreter::execute_effects(engine, &mut ctx, card.effect_data);
    if let Some(hook) = card.complex_hook {
        hook(engine, &ctx);
    }
    engine.runtime_card_total_unblocked_damage = prev_total_unblocked_damage;
    engine.runtime_card_enemy_killed = prev_enemy_killed;
}

#[cfg(test)]
mod test_runtime_inline_cutover_wave3 {
    use crate::actions::Action;
    use crate::status_ids::sid;
    use crate::tests::support::{combat_state_with, engine_with_state, make_deck_n};

    #[test]
    fn pen_nib_doubles_exactly_on_tenth_attack_and_resets_after_firing() {
        let mut state = combat_state_with(
            make_deck_n("Strike_R", 16),
            vec![crate::tests::support::enemy_no_intent("JawWorm", 200, 200)],
            20,
        );
        state.relics.push("Pen Nib".to_string());
        let mut engine = engine_with_state(state);
        engine.state.hand = make_deck_n("Strike_R", 10);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();

        for expected_counter in 1..=9 {
            let hp_before = engine.state.enemies[0].entity.hp;
            engine.execute_action(&Action::PlayCard {
                card_idx: 0,
                target_idx: 0,
            });
            assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 6);
            assert_eq!(engine.state.player.status(sid::PEN_NIB_COUNTER), expected_counter);
        }

        let hp_before_tenth = engine.state.enemies[0].entity.hp;
        engine.execute_action(&Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        });
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before_tenth - 12);
        assert_eq!(engine.state.player.status(sid::PEN_NIB_COUNTER), 0);

        engine.state.hand = make_deck_n("Strike_R", 1);
        let hp_before_eleventh = engine.state.enemies[0].entity.hp;
        engine.execute_action(&Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        });
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before_eleventh - 6);
        assert_eq!(engine.state.player.status(sid::PEN_NIB_COUNTER), 1);
    }
}

#[cfg(test)]
#[path = "tests/test_runtime_inline_cutover_wave4.rs"]
mod test_runtime_inline_cutover_wave4;
