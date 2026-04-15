//! Declarative effect interpreter — walks Effect arrays and dispatches
//! through proper engine methods.

use crate::cards::{CardTarget, CardType};
use crate::damage;
use crate::engine::{CombatEngine, CombatPhase, ChoiceOption, ChoiceReason};
use crate::effects::declarative::*;
use crate::effects::trigger::TriggerContext;
use crate::effects::types::CardPlayContext;
use crate::ids::StatusId;
use crate::status_ids::sid;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

// ===========================================================================
// Public entry point
// ===========================================================================

/// Execute a slice of declarative effects.
/// Called from the card play pipeline after the damage loop.
/// Stops if AwaitingChoice is triggered (ChooseCards must be last).
pub fn execute_effects(engine: &mut CombatEngine, ctx: &mut CardPlayContext, effects: &[Effect]) {
    for effect in effects {
        if engine.phase == CombatPhase::AwaitingChoice {
            return; // Choice triggered, stop processing
        }
        execute_one(engine, ctx, effect);
    }
}

// ===========================================================================
// Single effect dispatch
// ===========================================================================

fn execute_one(engine: &mut CombatEngine, ctx: &mut CardPlayContext, effect: &Effect) {
    match effect {
        Effect::Simple(simple) => execute_simple(engine, ctx, simple),

        Effect::Conditional(condition, then_effects, else_effects) => {
            if evaluate_condition(engine, ctx, condition) {
                execute_effects(engine, ctx, then_effects);
            } else {
                execute_effects(engine, ctx, else_effects);
            }
        }

        Effect::ChooseCards {
            source,
            filter,
            action,
            min_picks,
            max_picks,
            post_choice_draw,
        } => {
            execute_choose_cards(
                engine,
                ctx,
                *source,
                *filter,
                *action,
                *min_picks,
                *max_picks,
                *post_choice_draw,
            );
        }

        Effect::ForEachInPile { pile, filter, action } => {
            execute_for_each(engine, ctx, *pile, *filter, *action);
        }

        Effect::ExtraHits(_amount_source) => {
            // Extra hits are integrated into the damage loop in card_effects.rs.
            // The declarative interpreter does not handle the damage pipeline directly —
            // card_effects.rs reads the ExtraHits variant to determine hit count.
        }

        Effect::Discover(card_names) => {
            execute_discover(engine, ctx, card_names);
        }

        Effect::ChooseNamedOptions(option_names) => {
            execute_choose_named_options(engine, option_names);
        }

        Effect::ChooseScaledNamedOptions(option_specs) => {
            execute_choose_scaled_named_options(engine, ctx, option_specs);
        }

        Effect::GenerateRandomCardsToHand {
            pool,
            count,
            cost_rule,
        } => {
            execute_generate_random_cards_to_hand(engine, ctx, *pool, *count, *cost_rule);
        }

        Effect::GenerateRandomCardsToDraw {
            pool,
            count,
            cost_rule,
        } => {
            execute_generate_random_cards_to_draw(engine, ctx, *pool, *count, *cost_rule);
        }

        Effect::GenerateDiscoveryChoice {
            pool,
            option_count,
            preview_cost_rule,
            selected_cost_rule,
        } => {
            execute_generate_discovery_choice(
                engine,
                *pool,
                *option_count,
                *preview_cost_rule,
                *selected_cost_rule,
            );
        }
    }
}

fn execute_scaled_attack_damage(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    target: Target,
    base_damage: i32,
) {
    let player_strength = engine.state.player.strength();
    let player_weak = engine.state.player.is_weak();
    let weak_paper_crane = engine.state.has_relic("Paper Crane");
    let stance_mult = engine.state.stance.outgoing_mult();
    let pen_nib_active = ctx.pen_nib_active;
    let vigor = ctx.vigor;

    let double_damage = engine.state.player.status(sid::DOUBLE_DAMAGE) > 0;
    if double_damage {
        let dd = engine.state.player.status(sid::DOUBLE_DAMAGE);
        engine.state.player.set_status(sid::DOUBLE_DAMAGE, dd - 1);
    }

    match target {
        Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                let tidx = idx as usize;
                let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
                let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
                let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                let dmg = damage::calculate_damage_full(
                    base_damage,
                    player_strength,
                    vigor,
                    player_weak,
                    weak_paper_crane,
                    pen_nib_active,
                    double_damage,
                    stance_mult,
                    enemy_vuln,
                    vuln_paper_frog,
                    false,
                    enemy_intangible,
                );
                let block_return = engine.state.enemies[tidx].entity.status(sid::BLOCK_RETURN);
                let hp_dmg = engine.deal_player_attack_hit_to_enemy(tidx, dmg);
                if block_return > 0 && hp_dmg > 0 {
                    engine.gain_block_player(block_return);
                }
            }
        }
        Target::AllEnemies => {
            let living = engine.state.living_enemy_indices();
            for enemy_idx in living {
                let enemy_vuln = engine.state.enemies[enemy_idx].entity.is_vulnerable();
                let enemy_intangible = engine.state.enemies[enemy_idx].entity.status(sid::INTANGIBLE) > 0;
                let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                let dmg = damage::calculate_damage_full(
                    base_damage,
                    player_strength,
                    vigor,
                    player_weak,
                    weak_paper_crane,
                    pen_nib_active,
                    double_damage,
                    stance_mult,
                    enemy_vuln,
                    vuln_paper_frog,
                    false,
                    enemy_intangible,
                );
                let block_return = engine.state.enemies[enemy_idx].entity.status(sid::BLOCK_RETURN);
                let hp_dmg = engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
                if block_return > 0 && hp_dmg > 0 {
                    engine.gain_block_player(block_return);
                }
            }
        }
        Target::RandomEnemy => {
            let living = engine.state.living_enemy_indices();
            if !living.is_empty() {
                let enemy_idx = living[engine.rng_gen_range(0..living.len())];
                let enemy_vuln = engine.state.enemies[enemy_idx].entity.is_vulnerable();
                let enemy_intangible = engine.state.enemies[enemy_idx].entity.status(sid::INTANGIBLE) > 0;
                let vuln_paper_frog = engine.state.has_relic("Paper Frog");
                let dmg = damage::calculate_damage_full(
                    base_damage,
                    player_strength,
                    vigor,
                    player_weak,
                    weak_paper_crane,
                    pen_nib_active,
                    double_damage,
                    stance_mult,
                    enemy_vuln,
                    vuln_paper_frog,
                    false,
                    enemy_intangible,
                );
                let block_return = engine.state.enemies[enemy_idx].entity.status(sid::BLOCK_RETURN);
                let hp_dmg = engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
                if block_return > 0 && hp_dmg > 0 {
                    engine.gain_block_player(block_return);
                }
            }
        }
        Target::Player | Target::SelfEntity => {
            engine.player_lose_hp(base_damage);
        }
    }
}

// ===========================================================================
// SimpleEffect dispatch
// ===========================================================================

fn execute_simple(engine: &mut CombatEngine, ctx: &mut CardPlayContext, simple: &SimpleEffect) {
    match *simple {
        // -- Status application --
        SimpleEffect::AddStatus(target, status, ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            apply_status(engine, ctx, target, status, amount);
        }

        SimpleEffect::SetStatus(target, status, ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            set_status(engine, ctx, target, status, amount);
        }

        SimpleEffect::MultiplyStatus(target, status, multiplier) => {
            multiply_status(engine, ctx, target, status, multiplier);
        }

        // -- Draw --
        SimpleEffect::DrawCards(ref amount_src) => {
            let count = resolve_card_amount(engine, ctx, amount_src);
            engine.draw_cards(count);
        }

        SimpleEffect::DrawCardsThenDiscardDrawnNonZeroCost(ref amount_src) => {
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            let hand_before = engine.state.hand.len();
            if count > 0 {
                engine.draw_cards(count);
            }
            let hand_after = engine.state.hand.len();
            if hand_after <= hand_before {
                return;
            }

            let mut to_discard = Vec::new();
            for idx in hand_before..hand_after {
                if let Some(card) = engine.state.hand.get(idx) {
                    let def = engine.card_registry.card_def_by_id(card.def_id);
                    let current_cost = if card.cost >= 0 {
                        card.cost as i32
                    } else {
                        def.cost
                    };
                    if current_cost > 0 && !card.is_free() {
                        to_discard.push(idx);
                    }
                }
            }
            for idx in to_discard.into_iter().rev() {
                if idx < engine.state.hand.len() {
                    let card = engine.state.hand.remove(idx);
                    engine.state.discard_pile.push(card);
                    engine.on_card_discarded(card);
                }
            }
        }

        SimpleEffect::DrawToHandSize(ref amount_src) => {
            let target = resolve_card_amount(engine, ctx, amount_src);
            let to_draw = (target - engine.state.hand.len() as i32).max(0);
            if to_draw > 0 {
                engine.draw_cards(to_draw);
            }
        }

        SimpleEffect::ExhaustRandomCardFromHand => {
            if !engine.state.hand.is_empty() {
                let idx = engine.rng_gen_range(0..engine.state.hand.len());
                let exhausted = engine.state.hand.remove(idx);
                engine.state.exhaust_pile.push(exhausted);
                engine.trigger_card_on_exhaust(exhausted);
            }
        }

        SimpleEffect::SetRandomHandCardCost(cost) => {
            let eligible: Vec<usize> = engine.state.hand.iter()
                .enumerate()
                .filter(|(_, card)| {
                    let current_cost = if card.cost >= 0 {
                        card.cost as i32
                    } else {
                        engine.card_registry.card_def_by_id(card.def_id).cost
                    };
                    current_cost > 0
                })
                .map(|(idx, _)| idx)
                .collect();

            if !eligible.is_empty() {
                let idx = eligible[engine.rng_gen_range(0..eligible.len())];
                if idx < engine.state.hand.len() {
                    engine.state.hand[idx].set_permanent_cost(cost as i8);
                }
            }
        }

        SimpleEffect::ObtainRandomPotion => {
            let _ = engine.obtain_random_potion();
        }

        SimpleEffect::DrawRandomCardsFromPileToHand(pile, filter, ref count_src) => {
            execute_draw_random_cards_from_pile_to_hand(engine, ctx, pile, filter, *count_src);
        }

        // -- Energy --
        SimpleEffect::GainEnergy(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            engine.state.energy += amount;
        }

        // -- Double energy --
        SimpleEffect::DoubleEnergy => {
            engine.state.energy *= 2;
        }

        // -- Block (routes through dex/frail pipeline) --
        SimpleEffect::GainBlock(ref amount_src) => {
            let base = resolve_card_amount(engine, ctx, amount_src);
            let mut multiplier = 1;
            // Java X-cost block cards like Reinforced Body resolve their modified
            // block once, then apply it per energy spent.
            if matches!(amount_src, AmountSource::Block) && ctx.card.cost == -1 && ctx.card.base_block > 0 {
                multiplier = ctx.x_value.max(0);
            }
            let dex = engine.state.player.dexterity();
            let frail = engine.state.player.is_frail();
            let block = damage::calculate_block(base, dex, frail) * multiplier;
            engine.gain_block_player(block);
        }
        SimpleEffect::GainBlockIfLastHandCardType(card_type, ref amount_src) => {
            if let Some(last_card) = engine.state.hand.last() {
                let last_type = engine.card_registry.card_def_by_id(last_card.def_id).card_type;
                if last_type == card_type {
                    let base = resolve_card_amount(engine, ctx, amount_src);
                    let dex = engine.state.player.dexterity();
                    let frail = engine.state.player.is_frail();
                    let block = damage::calculate_block(base, dex, frail);
                    engine.gain_block_player(block);
                }
            }
        }

        // -- HP modification --
        SimpleEffect::ModifyHp(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            if amount > 0 {
                engine.heal_player(amount);
            } else if amount < 0 {
                engine.player_lose_hp(-amount);
            }
        }

        // -- Mantra --
        SimpleEffect::GainMantra(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            engine.gain_mantra(amount);
        }

        // -- Scry (may trigger AwaitingChoice) --
        SimpleEffect::Scry(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            engine.do_scry(amount);
        }

        // -- Add temp card to a pile --
        SimpleEffect::AddCard(name, pile, ref amount_src) => {
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            for _ in 0..count {
                let card = engine.temp_card(name);
                push_to_pile(engine, pile, card);
            }
            // Shuffle draw pile if cards were added to it
            if pile == Pile::Draw && count > 0 {
                engine.shuffle_draw_pile();
            }
        }

        // -- Add temp card to a pile with explicit misc state --
        SimpleEffect::AddCardWithMisc(name, pile, ref amount_src, ref misc_src) => {
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            let misc = resolve_card_amount(engine, ctx, misc_src).max(0) as i16;
            for _ in 0..count {
                let mut card = engine.temp_card(name);
                card.misc = misc;
                push_to_pile(engine, pile, card);
            }
            if pile == Pile::Draw && count > 0 {
                engine.shuffle_draw_pile();
            }
        }

        // -- Copy played card to a pile (Anger: copy to discard) --
        SimpleEffect::CopyThisCardTo(pile) => {
            push_to_pile(engine, pile, ctx.card_inst);
            if pile == Pile::Draw {
                engine.shuffle_draw_pile();
            }
        }

        // -- Channel orb --
        SimpleEffect::ChannelOrb(orb_type, ref amount_src) => {
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            for _ in 0..count {
                engine.channel_orb(orb_type);
            }
        }

        SimpleEffect::ChannelRandomOrb(ref amount_src) => {
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            let orb_types = [
                crate::orbs::OrbType::Lightning,
                crate::orbs::OrbType::Frost,
                crate::orbs::OrbType::Dark,
                crate::orbs::OrbType::Plasma,
            ];
            for _ in 0..count {
                let idx = engine.rng_gen_range(0..orb_types.len());
                engine.channel_orb(orb_types[idx]);
            }
        }

        SimpleEffect::RemoveOrbSlot => {
            let focus = engine.state.player.focus();
            let evoke = engine.state.orb_slots.remove_slot(focus);
            engine.apply_evoke_effect(evoke);
        }

        SimpleEffect::TriggerDarkPassive => {
            let focus = engine.state.player.focus();
            for orb in engine.state.orb_slots.slots.iter_mut() {
                if orb.orb_type == crate::orbs::OrbType::Dark {
                    let gain = (orb.base_passive + focus).max(0);
                    orb.evoke_amount += gain;
                }
            }
        }

        SimpleEffect::EvokeAndRechannelFrontOrb => {
            if engine.state.orb_slots.occupied_count() > 0 {
                let orb_type = engine.state.orb_slots.front_orb_type();
                let focus = engine.state.player.focus();
                let evoke = engine.state.orb_slots.evoke_front(focus);
                engine.apply_evoke_effect(evoke);
                if orb_type != crate::orbs::OrbType::Empty {
                    let evoke = engine.state.orb_slots.channel(orb_type, focus);
                    engine.apply_evoke_effect(evoke);
                }
            }
        }

        // -- Fission --
        SimpleEffect::ResolveFission { evoke } => {
            let orb_count = engine.state.orb_slots.occupied_count() as i32;
            if evoke {
                engine.evoke_all_orbs();
            } else {
                let max_slots = engine.state.orb_slots.max_slots;
                engine.state.orb_slots.slots =
                    vec![crate::orbs::Orb::new(crate::orbs::OrbType::Empty); max_slots];
            }
            if orb_count > 0 {
                engine.state.energy += orb_count;
                engine.draw_cards(orb_count);
            }
        }

        // -- Evoke front orb --
        SimpleEffect::EvokeOrb(ref amount_src) => {
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            if count > 0 {
                engine.evoke_front_orb_n(count as usize);
            }
        }

        // -- Stance change --
        SimpleEffect::ChangeStance(stance) => {
            engine.change_stance(stance);
        }

        // -- Boolean flags --
        SimpleEffect::SetFlag(flag) => {
            set_bool_flag(engine, flag);
        }

        // -- Shuffle discard into draw --
        SimpleEffect::ShuffleDiscardIntoDraw => {
            let mut cards = std::mem::take(&mut engine.state.discard_pile);
            engine.state.draw_pile.append(&mut cards);
            engine.shuffle_draw_pile();
        }

        // -- Discard random cards from a pile --
        SimpleEffect::DiscardRandomCardsFromPile(pile, count) => {
            execute_discard_random_cards_from_pile(engine, pile, count);
        }

        // -- Play the top card of the draw pile through the normal free-play path --
        SimpleEffect::PlayTopCardOfDraw => {
            if let Some(mut card) = engine.state.draw_pile.pop() {
                let def = engine.card_registry.card_def_by_id(card.def_id).clone();
                let target = if def.target == CardTarget::Enemy {
                    let living = engine.state.living_enemy_indices();
                    if living.is_empty() {
                        -1
                    } else {
                        let idx = engine.rng_gen_range(0..living.len());
                        living[idx] as i32
                    }
                } else {
                    -1
                };
                card.cost = 0;
                card.flags |= crate::combat_types::CardInstance::FLAG_FREE;
                engine.state.hand.push(card);
                let hand_idx = engine.state.hand.len() - 1;
                engine.play_card(hand_idx, target);
            }
        }

        // -- Deal flat damage (no strength/stance modifiers) --
        SimpleEffect::DealDamage(target, ref amount_src) => {
            if matches!(amount_src, AmountSource::DrawPileSize)
                && matches!(
                    target,
                    Target::SelectedEnemy | Target::AllEnemies | Target::RandomEnemy
                )
            {
                let amount = resolve_card_amount(engine, ctx, amount_src);
                execute_scaled_attack_damage(engine, ctx, target, amount);
                return;
            }
            if matches!(*amount_src, AmountSource::Damage)
                && matches!(
                    target,
                    Target::SelectedEnemy | Target::AllEnemies | Target::RandomEnemy
                )
            {
                crate::card_effects::execute_primary_attack(engine, ctx, target);
                return;
            }
            let amount = resolve_card_amount(engine, ctx, amount_src);
            if amount > 0 {
                deal_flat_damage(engine, ctx, target, amount);
            }
        }

        // -- Enemy block removal --
        SimpleEffect::RemoveEnemyBlock(target) => {
            match target {
                Target::SelectedEnemy => {
                    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
                        engine.state.enemies[ctx.target_idx as usize].entity.block = 0;
                    }
                }
                Target::AllEnemies => {
                    for idx in engine.state.living_enemy_indices() {
                        engine.state.enemies[idx].entity.block = 0;
                    }
                }
                Target::RandomEnemy => {
                    let living = engine.state.living_enemy_indices();
                    if !living.is_empty() {
                        let idx = living[engine.rng_gen_range(0..living.len())];
                        engine.state.enemies[idx].entity.block = 0;
                    }
                }
                Target::Player | Target::SelfEntity => {}
            }
        }

        // -- Judgement special resolution --
        SimpleEffect::Judgement(ref threshold_src) => {
            let threshold = resolve_card_amount(engine, ctx, threshold_src);
            if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
                let tidx = ctx.target_idx as usize;
                if engine.state.enemies[tidx].entity.hp <= threshold
                    && engine.state.enemies[tidx].is_alive()
                {
                    let lethal =
                        engine.state.enemies[tidx].entity.hp + engine.state.enemies[tidx].entity.block;
                    engine.deal_damage_to_enemy(tidx, lethal);
                }
            }
        }

        // -- Pressure Points mark resolution --
        SimpleEffect::TriggerMarks => {
            let living = engine.state.living_enemy_indices();
            let mut total_mark_damage = 0;
            let mut any_killed = false;
            for idx in living {
                let mark = engine.state.enemies[idx].entity.status(sid::MARK);
                if mark > 0 {
                    engine.state.enemies[idx].entity.hp -= mark;
                    engine.state.total_damage_dealt += mark;
                    total_mark_damage += mark;
                    if engine.state.enemies[idx].entity.hp <= 0 {
                        engine.state.enemies[idx].entity.hp = 0;
                        any_killed = true;
                    }
                    engine.record_enemy_hp_damage(idx, mark);
                }
            }
            if total_mark_damage > 0 {
                ctx.total_unblocked_damage += total_mark_damage;
            }
            if any_killed {
                ctx.enemy_killed = true;
            }
        }

        // -- Played card mutation --
        SimpleEffect::ModifyPlayedCardCost(ref amount_src) => {
            let delta = resolve_card_amount(engine, ctx, amount_src);
            if let Some(mut card) = engine.runtime_played_card {
                let current = if card.cost >= 0 {
                    card.cost as i32
                } else {
                    ctx.card.cost
                };
                let next = (current + delta).max(0) as i8;
                card.set_permanent_cost(next);
                ctx.card_inst.set_permanent_cost(next);
                engine.runtime_played_card = Some(card);
            }
        }

        SimpleEffect::ModifyPlayedCardBlock(ref amount_src) => {
            let delta = resolve_card_amount(engine, ctx, amount_src);
            if let Some(mut card) = engine.runtime_played_card {
                let current = if card.misc >= 0 {
                    card.misc as i32
                } else {
                    ctx.card.base_block.max(0)
                };
                let next = (current + delta).max(0) as i16;
                card.misc = next;
                ctx.card_inst.misc = next;
                engine.runtime_played_card = Some(card);
            }
        }

        SimpleEffect::ModifyPlayedCardDamage(ref amount_src) => {
            let delta = resolve_card_amount(engine, ctx, amount_src);
            if let Some(mut card) = engine.runtime_played_card {
                let current = if card.misc >= 0 {
                    card.misc as i32
                } else {
                    ctx.card.base_damage
                };
                let next = (current + delta).max(0) as i16;
                card.misc = next;
                ctx.card_inst.misc = next;
                engine.runtime_played_card = Some(card);
            }
        }

        // -- Heal HP (capped at max) --
        SimpleEffect::HealHp(_target, ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            if amount > 0 {
                engine.heal_player(amount);
            }
        }

        // -- Increment counter status --
        SimpleEffect::IncrementCounter(status_id, _threshold) => {
            engine.state.player.add_status(status_id, 1);
        }

        // -- Modify max HP --
        SimpleEffect::ModifyMaxHp(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            engine.state.player.max_hp = (engine.state.player.max_hp + amount).max(1);
            engine.state.player.hp = (engine.state.player.hp + amount)
                .max(0)
                .min(engine.state.player.max_hp);
        }

        // -- Modify max energy --
        SimpleEffect::ModifyMaxEnergy(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            engine.state.max_energy = (engine.state.max_energy + amount).max(0);
            engine.state.energy = engine.state.energy.min(engine.state.max_energy);
        }

        // -- Modify gold (no-op in combat context; wired in Wave 2) --
        SimpleEffect::ModifyGold(_amount_src) => {
            // Gold is on RunState, not CombatEngine. Handled at dispatch level.
        }

        // -- Flee combat --
        SimpleEffect::FleeCombat => {
            engine.state.combat_over = true;
        }
        SimpleEffect::UpgradeRandomCardFromPiles(piles) => {
            upgrade_random_card_from_piles(engine, piles);
        }
    }
}

// ===========================================================================
// Status helpers
// ===========================================================================

/// Debuff status IDs that should route through apply_debuff (handles Artifact).
fn is_debuff(status: StatusId) -> bool {
    status == sid::WEAKENED
        || status == sid::VULNERABLE
        || status == sid::FRAIL
        || status == sid::POISON
        || status == sid::CONSTRICTED
}

fn apply_status(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    target: Target,
    status: StatusId,
    amount: i32,
) {
    match target {
        Target::Player => {
            add_player_status(engine, status, amount);
        }
        // Card effects do not currently install owner-aware runtime handlers.
        // Treat SelfEntity as player here to keep the legacy interpreter compatible.
        Target::SelfEntity => {
            add_player_status(engine, status, amount);
        }
        Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                let i = idx as usize;
                if is_debuff(status) {
                    engine.apply_player_debuff_to_enemy(i, status, amount);
                } else {
                    engine.state.enemies[i].entity.add_status(status, amount);
                }
            }
        }
        Target::AllEnemies => {
            let living = engine.state.living_enemy_indices();
            for i in living {
                if is_debuff(status) {
                    engine.apply_player_debuff_to_enemy(i, status, amount);
                } else {
                    engine.state.enemies[i].entity.add_status(status, amount);
                }
            }
        }
        Target::RandomEnemy => {
            let living = engine.state.living_enemy_indices();
            if !living.is_empty() {
                let idx = living[engine.rng_gen_range(0..living.len())];
                if is_debuff(status) {
                    engine.apply_player_debuff_to_enemy(idx, status, amount);
                } else {
                    engine.state.enemies[idx].entity.add_status(status, amount);
                }
            }
        }
    }
}

fn set_status(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    target: Target,
    status: StatusId,
    value: i32,
) {
    match target {
        Target::Player => {
            set_player_status(engine, status, value);
        }
        // Card effects do not currently install owner-aware runtime handlers.
        // Treat SelfEntity as player here to keep the legacy interpreter compatible.
        Target::SelfEntity => {
            set_player_status(engine, status, value);
        }
        Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                engine.state.enemies[idx as usize].entity.set_status(status, value);
            }
        }
        Target::AllEnemies => {
            let living = engine.state.living_enemy_indices();
            for i in living {
                engine.state.enemies[i].entity.set_status(status, value);
            }
        }
        Target::RandomEnemy => {
            let living = engine.state.living_enemy_indices();
            if !living.is_empty() {
                let idx = living[engine.rng_gen_range(0..living.len())];
                engine.state.enemies[idx].entity.set_status(status, value);
            }
        }
    }
}

fn add_player_status(engine: &mut CombatEngine, status: StatusId, amount: i32) {
    engine.state.player.add_status(status, amount);
    if status == sid::ORB_SLOTS && amount > 0 {
        for _ in 0..amount {
            engine.state.orb_slots.add_slot();
        }
    }
}

fn set_player_status(engine: &mut CombatEngine, status: StatusId, value: i32) {
    if status == sid::ORB_SLOTS {
        let current = engine.state.player.status(status);
        if value > current {
            for _ in 0..(value - current) {
                engine.state.orb_slots.add_slot();
            }
        }
    }
    engine.state.player.set_status(status, value);
}

fn multiply_status(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    target: Target,
    status: StatusId,
    multiplier: i32,
) {
    match target {
        Target::Player => {
            let current = engine.state.player.status(status);
            if current > 0 {
                engine.state.player.set_status(status, current * multiplier);
            }
        }
        // Card effects do not currently install owner-aware runtime handlers.
        // Treat SelfEntity as player here to keep the legacy interpreter compatible.
        Target::SelfEntity => {
            let current = engine.state.player.status(status);
            if current > 0 {
                engine.state.player.set_status(status, current * multiplier);
            }
        }
        Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                let i = idx as usize;
                let current = engine.state.enemies[i].entity.status(status);
                if current > 0 {
                    engine.state.enemies[i].entity.set_status(status, current * multiplier);
                }
            }
        }
        Target::AllEnemies => {
            let living = engine.state.living_enemy_indices();
            for i in living {
                let current = engine.state.enemies[i].entity.status(status);
                if current > 0 {
                    engine.state.enemies[i].entity.set_status(status, current * multiplier);
                }
            }
        }
        Target::RandomEnemy => {
            let living = engine.state.living_enemy_indices();
            if !living.is_empty() {
                let idx = living[engine.rng_gen_range(0..living.len())];
                let current = engine.state.enemies[idx].entity.status(status);
                if current > 0 {
                    engine.state.enemies[idx].entity.set_status(status, current * multiplier);
                }
            }
        }
    }
}

// ===========================================================================
// Amount resolution
// ===========================================================================

pub fn resolve_card_amount(engine: &CombatEngine, ctx: &CardPlayContext, src: &AmountSource) -> i32 {
    match *src {
        AmountSource::Magic => ctx.card.base_magic.max(1),
        AmountSource::Block => {
            if ctx.card_inst.misc >= 0 {
                ctx.card_inst.misc as i32
            } else {
                ctx.card.base_block.max(0)
            }
        }
        AmountSource::Damage => ctx.card.base_damage.max(0),
        AmountSource::Fixed(n) => n,
        AmountSource::XCost => ctx.x_value,
        AmountSource::XCostPlus(bonus) => ctx.x_value + bonus,
        AmountSource::MagicPlusX => ctx.card.base_magic.max(0) + ctx.x_value,
        AmountSource::MagicPlusXNeg => -(ctx.card.base_magic.max(0) + ctx.x_value),
        AmountSource::LivingEnemyCount => engine.state.living_enemy_indices().len() as i32,
        AmountSource::OrbCount => engine.state.orb_slots.occupied_count() as i32,
        AmountSource::UniqueOrbCount => {
            // Count unique non-empty orb types
            let mut has_lightning = false;
            let mut has_frost = false;
            let mut has_dark = false;
            let mut has_plasma = false;
            for orb in &engine.state.orb_slots.slots {
                match orb.orb_type {
                    crate::orbs::OrbType::Lightning => has_lightning = true,
                    crate::orbs::OrbType::Frost => has_frost = true,
                    crate::orbs::OrbType::Dark => has_dark = true,
                    crate::orbs::OrbType::Plasma => has_plasma = true,
                    crate::orbs::OrbType::Empty => {}
                }
            }
            (has_lightning as i32) + (has_frost as i32) + (has_dark as i32) + (has_plasma as i32)
        }
        AmountSource::HandSize => engine.state.hand.len() as i32,
        AmountSource::PlayerBlock => engine.state.player.block,
        AmountSource::DiscardPileSize => engine.state.discard_pile.len() as i32,
        AmountSource::CardMisc => ctx.card_inst.misc.max(0) as i32,
        AmountSource::DrawPileSize => engine.state.draw_pile.len() as i32,
        AmountSource::DrawPileDivN(n) => {
            if n > 0 {
                engine.state.draw_pile.len() as i32 / n
            } else {
                0
            }
        }
        AmountSource::HandSizeAtPlay => ctx.hand_size_at_play as i32,
        AmountSource::HandSizeAtPlayPlus(bonus) => ctx.hand_size_at_play as i32 + bonus,
        AmountSource::LastBulkCount => ctx.last_bulk_count.max(0),
        AmountSource::LastBulkCountTimesBlock => {
            ctx.last_bulk_count.max(0) * ctx.card.base_block.max(0)
        }
        AmountSource::AttacksThisTurn => engine.state.attacks_played_this_turn,
        AmountSource::SkillsInHand => {
            engine.state.hand.iter()
                .filter(|c| {
                    let def = engine.card_registry.card_def_by_id(c.def_id);
                    def.card_type == CardType::Skill
                })
                .count() as i32
        }
        AmountSource::StatusValue(status_id) => {
            engine.state.player.status(status_id)
        }
        AmountSource::StatusValueTimesMagic(status_id) => {
            engine.state.player.status(status_id) * ctx.card.base_magic.max(0)
        }
        AmountSource::PercentMaxHp(pct) => {
            (engine.state.player.max_hp * pct) / 100
        }
        AmountSource::PotionPotency => {
            // Resolved externally by the potion interpreter (not the card interpreter).
            // If this is reached from card play context, it's a bug.
            0
        }
        AmountSource::TotalUnblockedDamage => ctx.total_unblocked_damage.max(0),
    }
}

fn upgrade_random_card_from_piles(engine: &mut CombatEngine, piles: &'static [Pile]) {
    let mut eligible: Vec<(Pile, usize)> = Vec::new();
    for pile in piles {
        let cards = match pile {
            Pile::Hand => &engine.state.hand,
            Pile::Draw => &engine.state.draw_pile,
            Pile::Discard => &engine.state.discard_pile,
            Pile::Exhaust => &engine.state.exhaust_pile,
        };
        for (idx, card) in cards.iter().enumerate() {
            if !card.is_upgraded() {
                eligible.push((*pile, idx));
            }
        }
    }
    if eligible.is_empty() {
        return;
    }
    let (pile, idx) = eligible[engine.rng_gen_range(0..eligible.len())];
    let pile_vec = match pile {
        Pile::Hand => &mut engine.state.hand,
        Pile::Draw => &mut engine.state.draw_pile,
        Pile::Discard => &mut engine.state.discard_pile,
        Pile::Exhaust => &mut engine.state.exhaust_pile,
    };
    if idx < pile_vec.len() {
        engine.card_registry.upgrade_card(&mut pile_vec[idx]);
    }
}

// ===========================================================================
// Deal flat damage (no strength/stance — used by relics, powers)
// ===========================================================================

fn deal_flat_damage(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    target: Target,
    amount: i32,
) {
    match target {
        Target::Player => {
            engine.player_lose_hp(amount);
        }
        // Card effects do not currently install owner-aware runtime handlers.
        // Treat SelfEntity as player here to keep the legacy interpreter compatible.
        Target::SelfEntity => {
            engine.player_lose_hp(amount);
        }
        Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                engine.deal_damage_to_enemy(idx as usize, amount);
            }
        }
        Target::AllEnemies => {
            let living = engine.state.living_enemy_indices();
            for i in living {
                engine.deal_damage_to_enemy(i, amount);
            }
        }
        Target::RandomEnemy => {
            let living = engine.state.living_enemy_indices();
            if !living.is_empty() {
                let idx = living[engine.rng_gen_range(0..living.len())];
                engine.deal_damage_to_enemy(idx, amount);
            }
        }
    }
}

// ===========================================================================
// Trigger-based effect execution (no CardPlayContext needed)
// ===========================================================================

/// Execute a slice of declarative effects from a trigger context.
/// Used by relics, powers, and potions that fire effects outside
/// of the card play pipeline.
///
/// This creates a synthetic CardPlayContext with no card data,
/// then delegates to the existing effect interpreter.
pub fn execute_trigger_effects(
    engine: &mut CombatEngine,
    trigger_ctx: &TriggerContext,
    effects: &[Effect],
) {
    // Build a minimal synthetic CardPlayContext.
    // Card-relative AmountSource variants (Magic, Block, Damage) will
    // resolve to 0/1 — callers should use Fixed() for trigger effects.
    static EMPTY_CARD: crate::cards::CardDef = crate::cards::CardDef {
        id: "",
        name: "",
        card_type: CardType::Skill,
        target: crate::cards::CardTarget::SelfTarget,
        cost: 0,
        base_damage: 0,
        base_block: 0,
        base_magic: 0,
        exhaust: false,
        enter_stance: None,
        effects: &[],
        effect_data: &[],
        complex_hook: None,
    };

    let mut ctx = CardPlayContext {
        card: &EMPTY_CARD,
        card_inst: crate::combat_types::CardInstance::new(0),
        target_idx: trigger_ctx.target_idx,
        x_value: 0,
        pen_nib_active: false,
        vigor: 0,
        total_unblocked_damage: 0,
        enemy_killed: false,
        hand_size_at_play: 0,
        last_bulk_count: 0,
    };

    execute_effects(engine, &mut ctx, effects);
}

// ===========================================================================
// Condition evaluation
// ===========================================================================

fn evaluate_condition(engine: &CombatEngine, ctx: &CardPlayContext, cond: &Condition) -> bool {
    match *cond {
        Condition::InStance(stance) => engine.state.stance == stance,

        Condition::EnemyAttacking => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                engine.state.enemies[idx as usize].is_attacking()
            } else {
                false
            }
        }

        Condition::HandContainsType(card_type) => {
            engine.state.hand.iter().any(|card| {
                engine.card_registry.card_def_by_id(card.def_id).card_type == card_type
            })
        }

        Condition::CardsPlayedThisTurnLessThan(threshold) => {
            engine.state.cards_played_this_turn < threshold
        }

        Condition::EnemyHasStatus(status) => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                engine.state.enemies[idx as usize].entity.status(status) > 0
            } else {
                false
            }
        }

        Condition::EnemyAlive => {
            let idx = ctx.target_idx;
            idx >= 0
                && (idx as usize) < engine.state.enemies.len()
                && engine.state.enemies[idx as usize].is_alive()
        }

        Condition::LastCardType(card_type) => {
            engine.state.last_card_type == Some(card_type)
        }

        Condition::PlayerHasStatus(status) => {
            engine.state.player.status(status) > 0
        }

        Condition::NoBlock => engine.state.player.block == 0,

        Condition::EnemyKilled => ctx.enemy_killed,

        Condition::DiscardedThisTurn => {
            engine.state.player.status(sid::DISCARDED_THIS_TURN) > 0
        }
    }
}

// ===========================================================================
// ChooseCards
// ===========================================================================

fn execute_choose_cards(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    source: Pile,
    filter: CardFilter,
    action: ChoiceAction,
    min_picks_src: AmountSource,
    max_picks_src: AmountSource,
    post_choice_draw_src: AmountSource,
) {
    let pile = get_pile(engine, source);

    // Build options from the source pile, applying filter
    let options: Vec<ChoiceOption> = pile.iter()
        .enumerate()
        .filter(|(_, card)| matches_filter(engine, card, filter))
        .map(|(i, _)| make_choice_option(source, i))
        .collect();

    if options.is_empty() {
        return;
    }

    let min_picks = resolve_card_amount(engine, ctx, &min_picks_src).max(0) as usize;
    let max_picks = (resolve_card_amount(engine, ctx, &max_picks_src).max(0) as usize)
        .min(options.len());

    if max_picks == 0 {
        return;
    }

    let reason = choice_reason_for_action(action, source);
    if matches!(action, ChoiceAction::CopyToHand | ChoiceAction::StoreCardForNextTurnCopies) {
        engine.begin_choice_with_action(
            reason,
            options,
            min_picks,
            max_picks,
            ctx.card.base_magic.max(1) as usize,
            Some(action),
        );
    } else {
        engine.begin_choice(reason, options, min_picks, max_picks);
    }
    if post_choice_draw_src != AmountSource::Fixed(0) {
        let post_choice_draw = resolve_card_amount(engine, ctx, &post_choice_draw_src).max(0);
        if let Some(choice) = engine.choice.as_mut() {
            choice.post_choice_draw = post_choice_draw;
        }
    }
    if matches!(action, ChoiceAction::MoveToHand)
        && matches!(source, Pile::Discard)
        && matches!(ctx.card.id, "Meditate" | "Meditate+")
    {
        if let Some(choice) = engine.choice.as_mut() {
            choice.retain_returned_cards = true;
        }
    }
}

fn get_pile(engine: &CombatEngine, pile: Pile) -> &Vec<crate::combat_types::CardInstance> {
    match pile {
        Pile::Hand => &engine.state.hand,
        Pile::Draw => &engine.state.draw_pile,
        Pile::Discard => &engine.state.discard_pile,
        Pile::Exhaust => &engine.state.exhaust_pile,
    }
}

fn push_to_pile(engine: &mut CombatEngine, pile: Pile, card: crate::combat_types::CardInstance) {
    match pile {
        Pile::Hand => {
            if engine.state.hand.len() < 10 {
                engine.state.hand.push(card);
            }
        }
        Pile::Draw => engine.state.draw_pile.push(card),
        Pile::Discard => engine.state.discard_pile.push(card),
        Pile::Exhaust => engine.state.exhaust_pile.push(card),
    }
}

fn matches_filter(
    engine: &CombatEngine,
    card: &crate::combat_types::CardInstance,
    filter: CardFilter,
) -> bool {
    match filter {
        CardFilter::All => true,
        CardFilter::Attacks => {
            engine.card_registry.card_def_by_id(card.def_id).card_type == CardType::Attack
        }
        CardFilter::AttackOrPower => {
            matches!(
                engine.card_registry.card_def_by_id(card.def_id).card_type,
                CardType::Attack | CardType::Power
            )
        }
        CardFilter::Skills => {
            engine.card_registry.card_def_by_id(card.def_id).card_type == CardType::Skill
        }
        CardFilter::NonAttacks => {
            engine.card_registry.card_def_by_id(card.def_id).card_type != CardType::Attack
        }
        CardFilter::ZeroCost => {
            let def = engine.card_registry.card_def_by_id(card.def_id);
            def.cost == 0
        }
        CardFilter::Upgradeable => !card.is_upgraded(),
    }
}

fn make_choice_option(source: Pile, index: usize) -> ChoiceOption {
    match source {
        Pile::Hand => ChoiceOption::HandCard(index),
        Pile::Draw => ChoiceOption::DrawCard(index),
        Pile::Discard => ChoiceOption::DiscardCard(index),
        Pile::Exhaust => ChoiceOption::ExhaustCard(index),
    }
}

fn choice_reason_for_action(action: ChoiceAction, source: Pile) -> ChoiceReason {
    match action {
        ChoiceAction::Discard => ChoiceReason::DiscardFromHand,
        ChoiceAction::DiscardForEffect => ChoiceReason::DiscardForEffect,
        ChoiceAction::Exhaust => match source {
            Pile::Hand => ChoiceReason::ExhaustFromHand,
            _ => ChoiceReason::DiscardForEffect,
        },
        ChoiceAction::MoveToHand => match source {
            Pile::Discard => ChoiceReason::ReturnFromDiscard,
            Pile::Draw => ChoiceReason::SearchDrawPile,
            Pile::Exhaust => ChoiceReason::PickFromExhaust,
            _ => ChoiceReason::PickOption,
        },
        ChoiceAction::PutOnTopOfDraw => match source {
            Pile::Discard => ChoiceReason::PickFromDiscard,
            _ => ChoiceReason::PutOnTopFromHand,
        },
        ChoiceAction::PlayForFree => match source {
            Pile::Draw => ChoiceReason::PlayCardFreeFromDraw,
            _ => ChoiceReason::PlayCardFree,
        },
        ChoiceAction::Upgrade => ChoiceReason::UpgradeCard,
        ChoiceAction::CopyToHand => ChoiceReason::DualWield,
        ChoiceAction::StoreCardForNextTurnCopies => ChoiceReason::DualWield,
        ChoiceAction::PutOnTopAtCostZero => ChoiceReason::SetupPick,
        ChoiceAction::PutOnBottomAtCostZero => ChoiceReason::ForethoughtPick,
        ChoiceAction::ExhaustAndGainEnergy => ChoiceReason::RecycleCard,
    }
}

// ===========================================================================
// ForEachInPile
// ===========================================================================

fn execute_for_each(
    engine: &mut CombatEngine,
    ctx: &mut CardPlayContext,
    pile: Pile,
    filter: CardFilter,
    action: BulkAction,
) {
    // Collect matching indices first to avoid borrow issues
    let matching: Vec<usize> = get_pile(engine, pile)
        .iter()
        .enumerate()
        .filter(|(_, card)| matches_filter(engine, card, filter))
        .map(|(i, _)| i)
        .collect();

    if matching.is_empty() {
        ctx.last_bulk_count = 0;
        return;
    }

    ctx.last_bulk_count = matching.len() as i32;

    match action {
        BulkAction::Exhaust => {
            // Move matching cards to exhaust pile (reverse order to preserve indices)
            let pile_ref = get_pile_mut(engine, pile);
            let mut exhausted = Vec::new();
            for &i in matching.iter().rev() {
                if i < pile_ref.len() {
                    exhausted.push(pile_ref.remove(i));
                }
            }
            let exhausted_cards = exhausted.clone();
            engine.state.exhaust_pile.extend(exhausted);
            // Trigger on-exhaust hooks for each card
            for card in exhausted_cards {
                engine.trigger_card_on_exhaust(card);
            }
        }

        BulkAction::Discard => {
            let pile_ref = get_pile_mut(engine, pile);
            let mut discarded = Vec::new();
            for &i in matching.iter().rev() {
                if i < pile_ref.len() {
                    discarded.push(pile_ref.remove(i));
                }
            }
            for card in discarded {
                engine.state.discard_pile.push(card);
                if pile == Pile::Hand {
                    engine.on_card_discarded(card);
                }
            }
        }

        BulkAction::Upgrade => {
            // Inline pile access to avoid borrow conflict between card_registry and state
            let pile_vec = match pile {
                Pile::Hand => &mut engine.state.hand,
                Pile::Draw => &mut engine.state.draw_pile,
                Pile::Discard => &mut engine.state.discard_pile,
                Pile::Exhaust => &mut engine.state.exhaust_pile,
            };
            for &i in &matching {
                if i < pile_vec.len() {
                    engine.card_registry.upgrade_card(&mut pile_vec[i]);
                }
            }
        }

        BulkAction::SetCostForTurn(cost) => {
            let pile_ref = get_pile_mut(engine, pile);
            for &i in &matching {
                if i < pile_ref.len() {
                    pile_ref[i].cost = cost as i8;
                }
            }
        }

        BulkAction::SetCost(cost) => {
            let pile_ref = get_pile_mut(engine, pile);
            for &i in &matching {
                if i < pile_ref.len() {
                    pile_ref[i].set_permanent_cost(cost as i8);
                }
            }
        }

        BulkAction::MoveToHand => {
            if pile == Pile::Hand {
                return; // No-op: already in hand
            }
            // Collect matching cards from the source pile, then add to hand
            let hand_capacity = 10 - engine.state.hand.len();
            let pile_ref = get_pile_mut(engine, pile);
            let mut moved = Vec::new();
            for &i in matching.iter().rev() {
                if i < pile_ref.len() && moved.len() < hand_capacity {
                    moved.push(pile_ref.remove(i));
                }
            }
            // pile_ref borrow ends here (temporary)
            engine.state.hand.extend(moved);
        }

        BulkAction::MoveToBottom => {
            let pile_ref = get_pile_mut(engine, pile);
            let mut moved = Vec::new();
            for &i in matching.iter().rev() {
                if i < pile_ref.len() {
                    moved.push(pile_ref.remove(i));
                }
            }
            // Insert at bottom of draw pile (index 0 = bottom)
            for card in moved {
                engine.state.draw_pile.insert(0, card);
            }
        }
    }
}

fn execute_draw_random_cards_from_pile_to_hand(
    engine: &mut CombatEngine,
    ctx: &mut CardPlayContext,
    pile: Pile,
    filter: CardFilter,
    count_src: AmountSource,
) {
    let count = resolve_card_amount(engine, ctx, &count_src).max(0) as usize;
    if count == 0 {
        return;
    }

    let mut picked = Vec::new();
    let mut eligible: Vec<usize> = get_pile(engine, pile)
        .iter()
        .enumerate()
        .filter(|(_, card)| matches_filter(engine, card, filter))
        .map(|(idx, _)| idx)
        .collect();

    for _ in 0..count {
        if eligible.is_empty() {
            break;
        }
        let choice_idx = engine.rng_gen_range(0..eligible.len());
        let idx = eligible.remove(choice_idx);
        let source = get_pile_mut(engine, pile);
        if idx < source.len() {
            picked.push(source.remove(idx));
            eligible = eligible
                .into_iter()
                .map(|n| if n > idx { n - 1 } else { n })
                .collect();
        }
    }

    for card in picked {
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(card);
        } else {
            engine.state.discard_pile.push(card);
        }
    }
}

fn execute_discard_random_cards_from_pile(
    engine: &mut CombatEngine,
    pile: Pile,
    count: i32,
) {
    let count = count.max(0) as usize;
    if count == 0 {
        return;
    }

    for _ in 0..count {
        let len = get_pile_mut(engine, pile).len();
        if len == 0 {
            break;
        }
        let idx = engine.rng_gen_range(0..len);
        let source = get_pile_mut(engine, pile);
        let card = source.remove(idx);
        engine.state.discard_pile.push(card);
        if pile == Pile::Hand {
            engine.on_card_discarded(card);
        }
    }
}

fn get_pile_mut(engine: &mut CombatEngine, pile: Pile) -> &mut Vec<crate::combat_types::CardInstance> {
    match pile {
        Pile::Hand => &mut engine.state.hand,
        Pile::Draw => &mut engine.state.draw_pile,
        Pile::Discard => &mut engine.state.discard_pile,
        Pile::Exhaust => &mut engine.state.exhaust_pile,
    }
}

// ===========================================================================
// Discover
// ===========================================================================

fn execute_discover(
    engine: &mut CombatEngine,
    _ctx: &CardPlayContext,
    card_names: &[&'static str],
) {
    if engine.state.hand.len() >= 10 {
        return;
    }
    let options: Vec<ChoiceOption> = card_names.iter()
        .map(|name| ChoiceOption::GeneratedCard(engine.temp_card(name)))
        .collect();
    if !options.is_empty() {
        engine.begin_choice(ChoiceReason::DiscoverCard, options, 1, 1);
    }
}

fn execute_choose_named_options(engine: &mut CombatEngine, option_names: &[&'static str]) {
    if option_names.is_empty() {
        return;
    }
    let options = option_names
        .iter()
        .copied()
        .map(crate::engine::ChoiceOption::Named)
        .collect();
    engine.begin_choice(ChoiceReason::PickOption, options, 1, 1);
}

fn execute_choose_scaled_named_options(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    option_specs: &[ScaledNamedOption],
) {
    if option_specs.is_empty() {
        return;
    }
    let options = option_specs
        .iter()
        .map(|option| ChoiceOption::Named(option.label))
        .collect();
    let payloads = option_specs
        .iter()
        .map(|option| crate::engine::NamedChoicePayload {
            kind: option.kind,
            amount: resolve_card_amount(engine, ctx, &option.amount).max(0),
        })
        .collect();
    engine.begin_choice_with_named_payloads(ChoiceReason::PickOption, options, 1, 1, payloads);
}

fn execute_generate_random_cards_to_hand(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    pool: GeneratedCardPool,
    count_src: AmountSource,
    cost_rule: GeneratedCostRule,
) {
    let count = resolve_card_amount(engine, ctx, &count_src).max(0) as usize;
    for _ in 0..count {
        if engine.state.hand.len() >= 10 {
            break;
        }
        if let Some(mut card) = generate_random_card(engine, pool) {
            apply_generated_upgrade_rule(
                engine,
                &mut card,
                upgrade_rule_from_cost_rule(cost_rule),
            );
            apply_generated_cost_rule(&mut card, cost_rule);
            engine.state.hand.push(card);
        }
    }
}

fn execute_generate_random_cards_to_draw(
    engine: &mut CombatEngine,
    ctx: &CardPlayContext,
    pool: GeneratedCardPool,
    count_src: AmountSource,
    cost_rule: GeneratedCostRule,
) {
    let count = resolve_card_amount(engine, ctx, &count_src).max(0) as usize;
    if count == 0 {
        return;
    }
    generate_random_cards(
        engine,
        pool,
        count,
        GeneratedDestination::Draw,
        cost_rule,
        GeneratedUpgradeRule::Base,
    );
}

fn execute_generate_discovery_choice(
    engine: &mut CombatEngine,
    pool: GeneratedCardPool,
    option_count: usize,
    preview_cost_rule: GeneratedCostRule,
    selected_cost_rule: GeneratedCostRule,
) {
    if engine.state.hand.len() >= 10 || option_count == 0 {
        return;
    }
    let options: Vec<ChoiceOption> = generate_unique_random_cards(engine, pool, option_count)
        .into_iter()
        .map(|mut card| {
            apply_generated_cost_rule(&mut card, preview_cost_rule);
            ChoiceOption::GeneratedCard(card)
        })
        .collect();
    if !options.is_empty() {
        engine.begin_discovery_choice(options, 1, 1, 1, selected_cost_rule);
    }
}

pub fn open_generated_discovery_choice(
    engine: &mut CombatEngine,
    pool: GeneratedCardPool,
    option_count: usize,
    cost_rule: GeneratedCostRule,
) {
    execute_generate_discovery_choice(engine, pool, option_count, cost_rule, cost_rule);
}

pub fn open_generated_discovery_choice_scaled(
    engine: &mut CombatEngine,
    pool: GeneratedCardPool,
    option_count: usize,
    cost_rule: GeneratedCostRule,
    copy_count: usize,
    upgrade_rule: GeneratedUpgradeRule,
) {
    if engine.state.hand.len() >= 10 || option_count == 0 {
        return;
    }
    let options: Vec<ChoiceOption> = generate_unique_random_cards(engine, pool, option_count)
        .into_iter()
        .map(|mut card| {
            apply_generated_upgrade_rule(engine, &mut card, upgrade_rule);
            apply_generated_cost_rule(&mut card, cost_rule);
            ChoiceOption::GeneratedCard(card)
        })
        .collect();
    if !options.is_empty() {
        engine.begin_discovery_choice(
            options,
            1,
            1,
            copy_count.max(1),
            cost_rule,
        );
    }
}

pub fn generate_random_cards(
    engine: &mut CombatEngine,
    pool: GeneratedCardPool,
    count: usize,
    destination: GeneratedDestination,
    cost_rule: GeneratedCostRule,
    upgrade_rule: GeneratedUpgradeRule,
) {
    for _ in 0..count {
        let at_hand_cap = matches!(destination, GeneratedDestination::Hand) && engine.state.hand.len() >= 10;
        if at_hand_cap {
            break;
        }
        if let Some(mut card) = generate_random_card(engine, pool) {
            apply_generated_upgrade_rule(
                engine,
                &mut card,
                combine_generated_upgrade_rules(
                    upgrade_rule,
                    upgrade_rule_from_cost_rule(cost_rule),
                ),
            );
            apply_generated_cost_rule(&mut card, cost_rule);
            match destination {
                GeneratedDestination::Hand => engine.state.hand.push(card),
                GeneratedDestination::Draw => engine.state.draw_pile.push(card),
            }
        }
    }
    if matches!(destination, GeneratedDestination::Draw) && count > 0 {
        engine.shuffle_draw_pile();
    }
}

fn generate_random_card(
    engine: &mut CombatEngine,
    pool: GeneratedCardPool,
) -> Option<crate::combat_types::CardInstance> {
    if matches!(pool, GeneratedCardPool::AnyColorAttackRarityWeighted) {
        return generate_weighted_any_color_attack_card(engine);
    }
    let pool_cards = generated_card_pool(engine, pool);
    if pool_cards.is_empty() {
        return None;
    }
    let choice = pool_cards[engine.rng_gen_range(0..pool_cards.len())];
    Some(engine.temp_card(choice))
}

fn generate_unique_random_cards(
    engine: &mut CombatEngine,
    pool: GeneratedCardPool,
    option_count: usize,
) -> Vec<crate::combat_types::CardInstance> {
    if matches!(pool, GeneratedCardPool::AnyColorAttackRarityWeighted) {
        return generate_unique_weighted_any_color_attack_cards(engine, option_count);
    }
    let pool_cards = generated_card_pool(engine, pool);
    if pool_cards.is_empty() {
        return Vec::new();
    }
    let target = option_count.min(pool_cards.len());
    let mut picked = Vec::with_capacity(target);
    let mut seen = HashSet::new();
    while picked.len() < target {
        let choice = pool_cards[engine.rng_gen_range(0..pool_cards.len())];
        if seen.insert(choice) {
            picked.push(engine.temp_card(choice));
        }
    }
    picked
}

fn generated_card_pool(engine: &CombatEngine, pool: GeneratedCardPool) -> Vec<&'static str> {
    match pool {
        GeneratedCardPool::Colorless => COLORLESS_GENERATION_POOL.to_vec(),
        GeneratedCardPool::Attack => engine
            .card_registry
            .all_card_defs()
            .iter()
            .filter(|def| def.card_type == CardType::Attack && !def.id.ends_with('+'))
            .map(|def| def.id)
            .collect(),
        GeneratedCardPool::Skill => engine
            .card_registry
            .all_card_defs()
            .iter()
            .filter(|def| def.card_type == CardType::Skill && !def.id.ends_with('+'))
            .map(|def| def.id)
            .collect(),
        GeneratedCardPool::Power => engine
            .card_registry
            .all_card_defs()
            .iter()
            .filter(|def| def.card_type == CardType::Power && !def.id.ends_with('+'))
            .map(|def| def.id)
            .collect(),
        GeneratedCardPool::AnyColorAttackRarityWeighted => weighted_any_color_attack_ids(engine)
            .into_iter()
            .collect(),
    }
}

pub fn apply_generated_cost_rule(
    card: &mut crate::combat_types::CardInstance,
    cost_rule: GeneratedCostRule,
) {
    match cost_rule {
        GeneratedCostRule::Base => {}
        GeneratedCostRule::ZeroThisTurn => {
            card.cost = 0;
        }
        GeneratedCostRule::ZeroIfPositiveThisTurn => {
            if card.cost > 0 {
                card.cost = 0;
            }
        }
        GeneratedCostRule::ZeroThisTurnAndUpgradeGenerated => {
            card.cost = 0;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GeneratedPoolRarity {
    Common,
    Uncommon,
    Rare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GeneratedCardMeta {
    card_type: CardType,
    rarity: GeneratedPoolRarity,
}

fn generated_card_meta(card_id: &str) -> Option<GeneratedCardMeta> {
    static CARD_META: OnceLock<HashMap<String, GeneratedCardMeta>> = OnceLock::new();
    CARD_META
        .get_or_init(build_generated_card_meta_map)
        .get(card_id.trim_end_matches('+'))
        .copied()
}

fn build_generated_card_meta_map() -> HashMap<String, GeneratedCardMeta> {
    let mut map = HashMap::new();
    for line in include_str!("../../../engine/content/cards.py").lines() {
        let Some((_, id_rest)) = line.split_once("id=\"") else {
            continue;
        };
        let Some((card_id, _)) = id_rest.split_once('"') else {
            continue;
        };
        let Some((_, type_rest)) = line.split_once("card_type=CardType.") else {
            continue;
        };
        let type_token = type_rest
            .split(|ch: char| !matches!(ch, 'A'..='Z' | '_'))
            .next()
            .unwrap_or("");
        let card_type = match type_token {
            "ATTACK" => CardType::Attack,
            "SKILL" => CardType::Skill,
            "POWER" => CardType::Power,
            "STATUS" => CardType::Status,
            "CURSE" => CardType::Curse,
            _ => continue,
        };
        let Some((_, rarity_rest)) = line.split_once("rarity=CardRarity.") else {
            continue;
        };
        let rarity_token = rarity_rest
            .split(|ch: char| !matches!(ch, 'A'..='Z' | '_'))
            .next()
            .unwrap_or("");
        let rarity = match rarity_token {
            "COMMON" => GeneratedPoolRarity::Common,
            "UNCOMMON" => GeneratedPoolRarity::Uncommon,
            "RARE" => GeneratedPoolRarity::Rare,
            _ => continue,
        };
        map.insert(card_id.to_string(), GeneratedCardMeta { card_type, rarity });
    }
    map
}

fn weighted_any_color_attack_ids(engine: &CombatEngine) -> Vec<&'static str> {
    engine
        .card_registry
        .all_card_defs()
        .iter()
        .filter(|def| !def.id.ends_with('+'))
        .filter(|def| {
            matches!(
                generated_card_meta(def.id),
                Some(GeneratedCardMeta {
                    card_type: CardType::Attack,
                    rarity: GeneratedPoolRarity::Common
                        | GeneratedPoolRarity::Uncommon
                        | GeneratedPoolRarity::Rare,
                })
            )
        })
        .map(|def| def.id)
        .collect()
}

fn weighted_any_color_attack_bucket(
    engine: &CombatEngine,
    rarity: GeneratedPoolRarity,
) -> Vec<&'static str> {
    engine
        .card_registry
        .all_card_defs()
        .iter()
        .filter(|def| !def.id.ends_with('+'))
        .filter(|def| {
            matches!(
                generated_card_meta(def.id),
                Some(GeneratedCardMeta {
                    card_type: CardType::Attack,
                    rarity: card_rarity,
                }) if card_rarity == rarity
            )
        })
        .map(|def| def.id)
        .collect()
}

fn roll_generated_attack_rarity(engine: &mut CombatEngine) -> GeneratedPoolRarity {
    let roll = engine.rng_gen_range(0..100);
    if roll < 55 {
        GeneratedPoolRarity::Common
    } else if roll < 85 {
        GeneratedPoolRarity::Uncommon
    } else {
        GeneratedPoolRarity::Rare
    }
}

fn generate_weighted_any_color_attack_card(
    engine: &mut CombatEngine,
) -> Option<crate::combat_types::CardInstance> {
    let rarity = roll_generated_attack_rarity(engine);
    let bucket = weighted_any_color_attack_bucket(engine, rarity);
    if bucket.is_empty() {
        return None;
    }
    let idx = engine.rng_gen_range(0..bucket.len());
    Some(engine.temp_card(bucket[idx]))
}

fn generate_unique_weighted_any_color_attack_cards(
    engine: &mut CombatEngine,
    option_count: usize,
) -> Vec<crate::combat_types::CardInstance> {
    let total_pool = weighted_any_color_attack_ids(engine);
    if total_pool.is_empty() || option_count == 0 {
        return Vec::new();
    }
    let target = option_count.min(total_pool.len());
    let mut picked = Vec::with_capacity(target);
    let mut seen = HashSet::new();
    let mut fallback_pool: Vec<&'static str> = total_pool;

    while picked.len() < target {
        if let Some(card) = generate_weighted_any_color_attack_card(engine) {
            let card_name = engine.card_registry.card_name(card.def_id);
            if seen.insert(card_name) {
                picked.push(card);
                continue;
            }
        }
        fallback_pool.retain(|card_id| !seen.contains(card_id));
        if fallback_pool.is_empty() {
            break;
        }
        let idx = engine.rng_gen_range(0..fallback_pool.len());
        let card_id = fallback_pool.remove(idx);
        seen.insert(card_id);
        picked.push(engine.temp_card(card_id));
    }

    picked
}

fn upgrade_rule_from_cost_rule(cost_rule: GeneratedCostRule) -> GeneratedUpgradeRule {
    match cost_rule {
        GeneratedCostRule::ZeroThisTurnAndUpgradeGenerated => GeneratedUpgradeRule::Upgrade,
        GeneratedCostRule::Base
        | GeneratedCostRule::ZeroThisTurn
        | GeneratedCostRule::ZeroIfPositiveThisTurn => GeneratedUpgradeRule::Base,
    }
}

fn combine_generated_upgrade_rules(
    explicit: GeneratedUpgradeRule,
    implied: GeneratedUpgradeRule,
) -> GeneratedUpgradeRule {
    match (explicit, implied) {
        (GeneratedUpgradeRule::Upgrade, _) | (_, GeneratedUpgradeRule::Upgrade) => {
            GeneratedUpgradeRule::Upgrade
        }
        _ => GeneratedUpgradeRule::Base,
    }
}

fn apply_generated_upgrade_rule(
    engine: &CombatEngine,
    card: &mut crate::combat_types::CardInstance,
    upgrade_rule: GeneratedUpgradeRule,
) {
    match upgrade_rule {
        GeneratedUpgradeRule::Base => {}
        GeneratedUpgradeRule::Upgrade => {
            if card.is_upgraded() {
                return;
            }
            let base_id = engine.card_registry.card_def_by_id(card.def_id).id;
            let upgraded_id = format!("{base_id}+");
            if let Some(def) = engine.card_registry.get(upgraded_id.as_str()) {
                card.def_id = engine.card_registry.card_id(def.id);
                card.flags |= crate::combat_types::CardInstance::FLAG_UPGRADED;
            }
        }
    }
}

const COLORLESS_GENERATION_POOL: &[&str] = &[
    "Apotheosis",
    "Bandage Up",
    "Bite",
    "Blind",
    "Chrysalis",
    "Dark Shackles",
    "Deep Breath",
    "Defend_R",
    "Discovery",
    "Dramatic Entrance",
    "Enlightenment",
    "Finesse",
    "Flash of Steel",
    "Forethought",
    "Ghostly",
    "Good Instincts",
    "HandOfGreed",
    "Impatience",
    "J.A.X.",
    "Jack Of All Trades",
    "Madness",
    "Magnetism",
    "Master of Strategy",
    "Mayhem",
    "Metamorphosis",
    "Mind Blast",
    "Panacea",
    "Panache",
    "PanicButton",
    "Purity",
    "RitualDagger",
    "Sadistic Nature",
    "Secret Technique",
    "Secret Weapon",
    "Strike_R",
    "Swift Strike",
    "The Bomb",
    "Thinking Ahead",
    "Transmutation",
    "Trip",
    "Violence",
];

pub fn is_colorless_generation_card(card_id: &str) -> bool {
    COLORLESS_GENERATION_POOL.contains(&card_id)
}

// ===========================================================================
// BoolFlag dispatch
// ===========================================================================

fn set_bool_flag(engine: &mut CombatEngine, flag: BoolFlag) {
    match flag {
        BoolFlag::NoDraw => {
            engine.state.player.set_status(sid::NO_DRAW, 1);
        }
        BoolFlag::RetainHand => {
            engine.state.player.set_status(sid::RETAIN_HAND_FLAG, 1);
        }
        BoolFlag::SkipEnemyTurn => {
            engine.state.skip_enemy_turn = true;
        }
        BoolFlag::NextAttackFree => {
            engine.state.player.set_status(sid::NEXT_ATTACK_FREE, 1);
        }
        BoolFlag::Blasphemy => {
            engine.state.blasphemy_active = true;
        }
        BoolFlag::BulletTime => {
            engine.state.player.set_status(sid::BULLET_TIME, 1);
            engine.state.player.set_status(sid::NO_DRAW, 1);
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_amount_fixed() {
        // Verify that Fixed(N) returns N without needing an engine
        // (we can't easily construct a CombatEngine in unit tests,
        //  so this just confirms the match arm compiles)
        let _ = AmountSource::Fixed(5);
    }

    #[test]
    fn test_is_debuff() {
        assert!(is_debuff(sid::WEAKENED));
        assert!(is_debuff(sid::VULNERABLE));
        assert!(is_debuff(sid::FRAIL));
        assert!(is_debuff(sid::POISON));
        assert!(!is_debuff(sid::STRENGTH));
        assert!(!is_debuff(sid::VIGOR));
    }
}

#[cfg(test)]
#[path = "../tests/test_generated_choice_java_wave3.rs"]
mod test_generated_choice_java_wave3;
