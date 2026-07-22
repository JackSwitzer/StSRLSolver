//! Card play preamble + declarative interpreter dispatch.
//!
//! Handles generic damage/block calculation for all cards, then dispatches
//! to the declarative effect interpreter and optional complex_hook.

use crate::cards::{CardDef, CardTarget, CardType};
use crate::combat_types::CardInstance;
use crate::damage;
use crate::effects::types::DamageModifier;
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn consume_pen_nib_for_attack(engine: &mut CombatEngine) -> bool {
    let Some(relic_slot) = engine.state.relics.iter().position(|id| id == "Pen Nib") else {
        return false;
    };
    let owner = crate::effects::runtime::EffectOwner::PlayerRelic {
        slot: relic_slot as u16,
    };
    // Source: reference/extracted/methods/relic/PenNib.java
    // AbstractRelic.counter persists across fights. Counter 9 means the next
    // ATTACK has PenNibPower; playing it resets the relic counter to zero.
    let counter = engine
        .hidden_effect_value("Pen Nib", owner, 0)
        .max(engine.state.player.status(sid::PEN_NIB_COUNTER));
    let pen_nib_active = engine.state.player.status(sid::PEN_NIB_POWER) > 0;
    let next = if pen_nib_active { 0 } else { counter + 1 };
    let _ = engine.set_hidden_effect_value("Pen Nib", owner, 0, next);
    engine.state.player.set_status(sid::PEN_NIB_COUNTER, next);
    if pen_nib_active {
        engine.state.player.set_status(sid::PEN_NIB_POWER, 0);
        true
    } else {
        if next == 9 {
            // PenNib.onUseCard queues ApplyPowerAction(PenNibPower(1)). The
            // compact engine resolves the queue within the same card step.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/PenNib.java.
            engine.state.player.set_status(sid::PEN_NIB_POWER, 1);
        }
        false
    }
}

fn resolve_damage_modifiers(
    engine: &CombatEngine,
    card: &CardDef,
    card_inst: CardInstance,
) -> DamageModifier {
    crate::effects::card_runtime::resolve_damage_modifiers(engine, card, card_inst)
}

fn pick_random_living_enemy(engine: &mut CombatEngine) -> Option<usize> {
    let living = engine.state.living_enemy_indices();
    if living.is_empty() {
        None
    } else {
        // AttackDamageRandomEnemyAction uses cardRandomRng and consumes a call
        // even when the only possible range is random(0, 0).
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/AttackDamageRandomEnemyAction.java
        let selected = engine
            .card_random_rng
            .random_int_range(0, (living.len() - 1) as i32) as usize;
        Some(living[selected])
    }
}

fn extra_hits_allow_zero(card_id: &str) -> bool {
    // SkewerAction and WhirlwindAction only queue hits when their final energy
    // effect is positive; zero energy without Chemical X deals no damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/SkewerAction.java
    // and actions/unique/WhirlwindAction.java.
    matches!(
        card_id,
        "Barrage"
            | "Barrage+"
            | "Expunger"
            | "Expunger+"
            | "Finisher"
            | "Finisher+"
            | "Flechettes"
            | "Flechettes+"
            | "Skewer"
            | "Skewer+"
            | "Thunder Strike"
            | "Thunder Strike+"
            | "Whirlwind"
            | "Whirlwind+"
    )
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
    let dmg_mod = resolve_damage_modifiers(engine, card, card_inst);

    let body_slam_damage = dmg_mod.base_damage_override;
    let heavy_blade_mult = dmg_mod.strength_multiplier;
    let mut total_damage_bonus = dmg_mod.base_damage_bonus;
    if card.card_type == CardType::Attack {
        total_damage_bonus += engine.player_attack_base_damage(card, card_inst) - card.base_damage;
    }

    // Shiv.java bakes the current AccuracyPower amount into baseDamage when a
    // Shiv is constructed, while AccuracyPower updates existing Shivs whenever
    // it is applied, stacked, drawn, or discarded. CardDef is immutable here,
    // so adding the current amount at damage resolution is semantically equal.
    if card_id == "Shiv" || card_id == "Shiv+" {
        let accuracy = engine.state.player.status(sid::ACCURACY);
        if accuracy > 0 {
            total_damage_bonus += accuracy;
        }
    }

    let grand_finale_blocked = card.runtime_triggers().iter().any(|trigger| {
        matches!(
            trigger,
            crate::effects::types::CardRuntimeTrigger::CanPlay(
                crate::effects::types::CanPlayRule::OnlyEmptyDraw
            )
        )
    }) && !engine.state.draw_pile.is_empty();
    if dmg_mod.skip_generic_damage || grand_finale_blocked {
        return;
    }

    let effective_base_damage = if body_slam_damage >= 0 {
        body_slam_damage
    } else {
        (card.base_damage + total_damage_bonus).max(0)
    };

    let hits = if let Some(amount_src) = card.declared_extra_hits() {
        let resolved = crate::effects::interpreter::resolve_card_amount(engine, ctx, &amount_src);
        if resolved <= 0 && extra_hits_allow_zero(card_id) {
            0
        } else {
            resolved.max(1)
        }
    } else if card.uses_x_cost() && card.cost == -1 {
        ctx.x_value
    } else if card.uses_multi_hit_hint() && card.base_magic > 0 {
        card.base_magic
    } else {
        1
    };

    let player_strength = engine.state.player.strength() * heavy_blade_mult;
    let player_weak = engine.state.player.is_weak();
    let weak_paper_crane = engine.state.has_relic("Paper Crane");
    let stance_mult = engine.state.stance.outgoing_mult();

    let double_damage = engine.state.player.status(sid::DOUBLE_DAMAGE) > 0;

    match target {
        crate::effects::declarative::Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                let tidx = idx as usize;
                let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
                let enemy_intangible =
                    engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
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
                for _ in 0..hits {
                    let hp_dmg = engine.deal_player_attack_hit_to_enemy(tidx, dmg);
                    total_unblocked_damage += hp_dmg;
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
                let enemy_intangible = engine.state.enemies[enemy_idx]
                    .entity
                    .status(sid::INTANGIBLE)
                    > 0;
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
                for _ in 0..hits {
                    let hp_dmg = engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
                    total_unblocked_damage += hp_dmg;
                    if engine.state.enemies[enemy_idx].entity.is_dead() {
                        enemy_killed = true;
                        break;
                    }
                }
            }
        }
        crate::effects::declarative::Target::RandomEnemy => {
            for _ in 0..hits {
                let Some(enemy_idx) = pick_random_living_enemy(engine) else {
                    break;
                };
                let enemy_vuln = engine.state.enemies[enemy_idx].entity.is_vulnerable();
                let enemy_intangible = engine.state.enemies[enemy_idx]
                    .entity
                    .status(sid::INTANGIBLE)
                    > 0;
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
                let hp_dmg = engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
                total_unblocked_damage += hp_dmg;
                if engine.state.enemies[enemy_idx].entity.is_dead() {
                    enemy_killed = true;
                }
            }
        }
        crate::effects::declarative::Target::Player
        | crate::effects::declarative::Target::SelfEntity => {
            for _ in 0..hits {
                engine.player_lose_hp_from_damage(effective_base_damage);
            }
        }
    }

    ctx.total_unblocked_damage = total_unblocked_damage;
    ctx.enemy_killed = enemy_killed;
}

/// Execute all effects for a card that was just played.
///
/// Called from `CombatEngine::play_card()` after energy payment and hand removal.
pub fn execute_card_effects(
    engine: &mut CombatEngine,
    card: &CardDef,
    card_inst: CardInstance,
    target_idx: i32,
) {
    let card_id = engine.card_registry.card_name(card_inst.def_id);
    // ---- X-cost: consume all remaining energy as X value + Chemical X bonus ----
    // Sources: ChemicalX.java defines BOOST 2; CollectAction.java and
    // ConjureBladeAction.java add exactly 2 when the player owns "Chemical X".
    let x_value = if card.cost == -1 {
        let x = engine
            .runtime_x_energy_override
            .take()
            .unwrap_or(engine.state.energy);
        engine.runtime_last_x_energy_on_use = x;
        let x_is_free = card_inst.is_free()
            || (card.card_type == CardType::Attack
                && engine.state.player.status(sid::NEXT_ATTACK_FREE) > 0)
            || (card.card_type == CardType::Skill
                && engine.state.player.status(sid::CORRUPTION) > 0);
        // MulticastAction only spends EnergyPanel.totalCount inside its
        // `player.hasOrb()` branch. Playing Multi-Cast with no front orb is
        // therefore a legal no-op that preserves all energy.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/MulticastAction.java
        let multi_cast_without_orb = matches!(card_id, "Multi-Cast" | "Multi-Cast+")
            && engine.state.orb_slots.occupied_count() == 0;
        if !x_is_free && !multi_cast_without_orb {
            engine.state.energy = 0;
        }
        x + if engine.state.has_relic("Chemical X") || engine.state.has_relic("ChemicalX") {
            2
        } else {
            0
        }
    } else {
        engine.runtime_last_x_energy_on_use = 0;
        0
    };

    // ---- Pen Nib check (before damage) ----
    let pen_nib_active = if card.card_type == CardType::Attack {
        consume_pen_nib_for_attack(engine)
    } else {
        false
    };

    // ---- Vigor (applies to the full next Attack, then is consumed) ----
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/VigorPower.java
    let vigor = if card.card_type == CardType::Attack {
        let v = engine.state.player.status(sid::VIGOR);
        if v > 0 {
            engine.state.player.set_status(sid::VIGOR, 0);
        }
        v
    } else {
        0
    };

    // ---- Damage modifiers via typed runtime triggers ----
    let dmg_mod = resolve_damage_modifiers(engine, card, card_inst);

    let body_slam_damage = dmg_mod.base_damage_override;
    let heavy_blade_mult = dmg_mod.strength_multiplier;
    // All additive bonuses (brilliance, perfected_strike, rampage, etc.) are merged
    let mut total_damage_bonus = dmg_mod.base_damage_bonus;
    let is_attack = card.card_type == CardType::Attack;
    if is_attack {
        total_damage_bonus += engine.player_attack_base_damage(card, card_inst) - card.base_damage;
    }

    // AccuracyPower updates every existing Shiv to 4/6 + its current amount;
    // immutable CardDefs represent that as the same play-time damage bonus.
    // Java: cards/tempCards/Shiv.java and powers/AccuracyPower.java.
    if card_id == "Shiv" || card_id == "Shiv+" {
        let accuracy = engine.state.player.status(sid::ACCURACY);
        if accuracy > 0 {
            total_damage_bonus += accuracy;
        }
    }

    // ---- Grand Finale: only deal damage if draw pile is empty ----
    let grand_finale_blocked = card.runtime_triggers().iter().any(|trigger| {
        matches!(
            trigger,
            crate::effects::types::CardRuntimeTrigger::CanPlay(
                crate::effects::types::CanPlayRule::OnlyEmptyDraw
            )
        )
    }) && !engine.state.draw_pile.is_empty();

    // ---- Damage ----
    // Track damage dealt for Wallop (block_from_damage) and Reaper (heal)
    let mut total_unblocked_damage = 0i32;
    let mut enemy_killed = false;
    let target_was_attacking = target_idx >= 0
        && engine
            .state
            .enemies
            .get(target_idx as usize)
            .is_some_and(|enemy| enemy.is_attacking());
    let pre_damage_ctx = crate::effects::types::CardPlayContext {
        card,
        card_inst,
        target_idx,
        target_was_attacking,
        x_value,
        pen_nib_active,
        vigor,
        total_unblocked_damage: 0,
        enemy_killed: false,
        hand_size_at_play: engine.state.hand.len(),
        last_bulk_count: 0,
        last_drawn_card_types: Vec::new(),
        deferred_manual_discards: Vec::new(),
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
            let resolved = crate::effects::interpreter::resolve_card_amount(
                engine,
                &pre_damage_ctx,
                &amount_src,
            );
            if resolved <= 0 && extra_hits_allow_zero(card_id) {
                0
            } else {
                resolved.max(1)
            }
        } else if card.uses_x_cost() && card.cost == -1 {
            x_value
        } else if card.uses_multi_hit_hint() && card.base_magic > 0 {
            card.base_magic
        } else {
            1
        };

        let player_strength = engine.state.player.strength() * heavy_blade_mult;
        let player_weak = engine.state.player.is_weak();
        let weak_paper_crane = engine.state.has_relic("Paper Crane");
        let stance_mult = engine.state.stance.outgoing_mult();

        // DoubleDamagePower modifies every NORMAL attack for its full duration;
        // powers/DoubleDamagePower.java decrements it only at end of round.
        let double_damage = engine.state.player.status(sid::DOUBLE_DAMAGE) > 0;

        match card.target {
            CardTarget::Enemy | CardTarget::SelfAndEnemy => {
                if target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
                    let tidx = target_idx as usize;
                    let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
                    let enemy_intangible =
                        engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
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
                    for _ in 0..hits {
                        let hp_dmg = engine.deal_player_attack_hit_to_enemy(tidx, dmg);
                        total_unblocked_damage += hp_dmg;
                        if engine.state.enemies[tidx].entity.is_dead() {
                            enemy_killed = true;
                            break;
                        }
                    }
                }
            }
            CardTarget::AllEnemy => {
                let living = engine.state.living_enemy_indices();
                let mut damage_by_enemy = Vec::with_capacity(living.len());
                for enemy_idx in living {
                    let enemy_vuln = engine.state.enemies[enemy_idx].entity.is_vulnerable();
                    let enemy_intangible = engine.state.enemies[enemy_idx]
                        .entity
                        .status(sid::INTANGIBLE)
                        > 0;
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
                    damage_by_enemy.push((enemy_idx, dmg));
                }
                // WhirlwindAction and Dagger Spray queue one complete
                // DamageAllEnemiesAction per hit. Resolve each whole AoE before
                // starting the next rather than grouping all hits by target.
                // Java: actions/unique/WhirlwindAction.java and
                // cards/green/DaggerSpray.java.
                for _ in 0..hits {
                    for &(enemy_idx, dmg) in &damage_by_enemy {
                        if engine.state.enemies[enemy_idx].entity.is_dead() {
                            continue;
                        }
                        let hp_dmg = engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
                        total_unblocked_damage += hp_dmg;
                        if engine.state.enemies[enemy_idx].entity.is_dead() {
                            enemy_killed = true;
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
        let block_multiplier = if card.has_block_hint(crate::effects::types::CardBlockHint::XTimes)
        {
            x_value
        } else {
            1
        };
        if !card.has_block_hint(crate::effects::types::CardBlockHint::IfSkill)
            && !card.has_block_hint(crate::effects::types::CardBlockHint::IfNoBlock)
            && !card.has_block_hint(crate::effects::types::CardBlockHint::BulkCountTimesBaseBlock)
            && !card.has_block_hint(crate::effects::types::CardBlockHint::ChoicePayloadOnly)
        {
            let current_base_block =
                if card.has_block_hint(crate::effects::types::CardBlockHint::UsesCardMisc) {
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
            let block = damage::calculate_block(current_base_block.max(0), dex, frail);
            engine.gain_block_player(block * block_multiplier);
        }
    }

    // ---- Declarative effect interpreter (always runs) ----
    let mut ctx = crate::effects::types::CardPlayContext {
        card,
        card_inst,
        target_idx,
        target_was_attacking,
        x_value,
        pen_nib_active,
        vigor,
        total_unblocked_damage,
        enemy_killed,
        hand_size_at_play: engine.state.hand.len(),
        last_bulk_count: 0,
        last_drawn_card_types: Vec::new(),
        deferred_manual_discards: Vec::new(),
    };
    let prev_total_unblocked_damage = engine.runtime_card_total_unblocked_damage;
    let prev_enemy_killed = engine.runtime_card_enemy_killed;
    engine.runtime_card_total_unblocked_damage = ctx.total_unblocked_damage;
    engine.runtime_card_enemy_killed = ctx.enemy_killed;
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
            make_deck_n("Strike", 16),
            vec![crate::tests::support::enemy_no_intent("JawWorm", 200, 200)],
            20,
        );
        state.relics.push("Pen Nib".to_string());
        let mut engine = engine_with_state(state);
        engine.state.hand = make_deck_n("Strike", 10);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();

        for expected_counter in 1..=9 {
            let hp_before = engine.state.enemies[0].entity.hp;
            engine.execute_action(&Action::PlayCard {
                card_idx: 0,
                target_idx: 0,
            });
            assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 6);
            assert_eq!(
                engine.state.player.status(sid::PEN_NIB_COUNTER),
                expected_counter
            );
            assert_eq!(
                engine.state.player.status(sid::PEN_NIB_POWER),
                i32::from(expected_counter == 9)
            );
        }

        let hp_before_tenth = engine.state.enemies[0].entity.hp;
        engine.execute_action(&Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        });
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before_tenth - 12);
        assert_eq!(engine.state.player.status(sid::PEN_NIB_COUNTER), 0);
        assert_eq!(engine.state.player.status(sid::PEN_NIB_POWER), 0);

        engine.state.hand = make_deck_n("Strike", 1);
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
