//! Declarative effect interpreter — walks Effect arrays and dispatches
//! through proper engine methods.

use crate::cards::CardType;
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
                engine.deal_player_attack_hit_to_enemy(tidx, dmg);
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
                engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
            }
        }
        Target::RandomEnemy => {
            let living = engine.state.living_enemy_indices();
            if !living.is_empty() {
                // AttackDamageRandomEnemyAction selects through cardRandomRng
                // for every hit, including the one-enemy random(0, 0) case.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/AttackDamageRandomEnemyAction.java
                let selected = engine
                    .card_random_rng
                    .random_range(0, (living.len() - 1) as i32) as usize;
                let enemy_idx = living[selected];
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
                engine.deal_player_attack_hit_to_enemy(enemy_idx, dmg);
            }
        }
        Target::Player | Target::SelfEntity => {
            engine.player_lose_hp_from_damage(base_damage);
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
            // DamageAction clears queued non-damage/non-heal/non-block actions
            // when the last monster dies. ApplyPowerAction is among those
            // removed, so effects such as Flying Knee's Energized do not land
            // after a lethal preceding hit.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
            if engine.state.combat_over || engine.state.is_victory() {
                return;
            }
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
            // DamageAction clears queued non-combat actions when the final
            // monster dies, so Wheel Kick's following DrawCardAction does not run.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
            if engine.state.combat_over || engine.state.is_victory() {
                return;
            }
            let count = resolve_card_amount(engine, ctx, amount_src);
            ctx.last_drawn_card_types = engine.draw_cards(count);
        }

        SimpleEffect::DrawCardsThenDiscardDrawnNonZeroCost(ref amount_src) => {
            // DamageAction removes queued non-combat actions after a lethal hit,
            // so Scrape's following DrawCardAction never resolves on victory.
            // Java: Scrape.java, DamageAction.java, and GameActionManager.java.
            if engine.state.combat_over || engine.state.is_victory() {
                return;
            }
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            let hand_before = engine.state.hand.len();
            let directly_drawn = if count > 0 {
                // draw_cards returns only the requested DrawCardAction's cards;
                // Evolve's recursively queued draws are intentionally excluded.
                engine.draw_cards(count).len()
            } else {
                0
            };
            if directly_drawn == 0 {
                return;
            }
            let direct_end = (hand_before + directly_drawn).min(engine.state.hand.len());

            let mut to_discard = Vec::new();
            for idx in hand_before..direct_end {
                if let Some(card) = engine.state.hand.get(idx) {
                    let def = engine.card_registry.card_def_by_id(card.def_id);
                    let current_cost = if card.cost >= 0 {
                        card.cost as i32
                    } else {
                        def.cost
                    };
                    // ScrapeFollowUpAction keeps only costForTurn == 0 or
                    // freeToPlayOnce. X-cost (-1), Status/Curse (-2), and
                    // positive-cost cards are all manually discarded.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/
                    // defect/ScrapeFollowUpAction.java
                    if current_cost != 0 && !card.is_free() {
                        to_discard.push(idx);
                    }
                }
            }
            // Java walks DrawCardAction.drawnCards in draw order. Adjust each
            // original index for earlier removals so discard hooks and RNG fire
            // in that same order.
            for (removed, original_idx) in to_discard.into_iter().enumerate() {
                let idx = original_idx - removed;
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
            engine.apply_madness_action(cost as i8);
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
            let mut base = resolve_card_amount(engine, ctx, amount_src);
            let mut multiplier = 1;
            // Java X-cost block cards like Reinforced Body resolve their modified
            // block once, then apply it per energy spent.
            if matches!(amount_src, AmountSource::Block) && ctx.card.cost == -1 && ctx.card.base_block > 0 {
                multiplier = ctx.x_value.max(0);
            }
            // BlockPerNonAttackAction queues one GainBlockAction(this.block) per
            // snapshotted non-Attack. Resolve the card's block separately and
            // retain one block-gain event per exhausted card.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/
            // BlockPerNonAttackAction.java
            if matches!(amount_src, AmountSource::LastBulkCountTimesBlock) {
                base = ctx.card.base_block;
                multiplier = ctx.last_bulk_count.max(0);
            }
            // WallopAction passes target.lastDamageTaken straight to a
            // GainBlockAction, and DoubleYourBlockAction directly adds the
            // player's current Block. Neither amount re-enters the card
            // Dexterity/Frail pipeline.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/WallopAction.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DoubleYourBlockAction.java
            let block = if matches!(amount_src, AmountSource::TotalUnblockedDamage | AmountSource::PlayerBlock) {
                base
            } else {
                let dex = engine.state.player.dexterity();
                let frail = engine.state.player.is_frail();
                damage::calculate_block(base, dex, frail)
            };
            // ReinforcedBodyAction and BlockPerNonAttackAction queue separate
            // GainBlockActions. Keep those as distinct events so onGainedBlock
            // powers such as Juggernaut and Wave of the Hand fire each time.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/
            // ReinforcedBodyAction.java
            for _ in 0..multiplier {
                engine.gain_block_player(block);
            }
        }
        SimpleEffect::GainBlockIfLastDrawnCardType(card_type, ref amount_src) => {
            if ctx.last_drawn_card_types.contains(&card_type) {
                let base = resolve_card_amount(engine, ctx, amount_src);
                let dex = engine.state.player.dexterity();
                let frail = engine.state.player.is_frail();
                let block = damage::calculate_block(base, dex, frail);
                engine.gain_block_player(block);
            }
        }

        // -- HP modification --
        SimpleEffect::ModifyHp(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            if amount > 0 {
                engine.heal_player(amount);
            } else if amount < 0 {
                // LoseHPAction uses HP_LOSS DamageInfo: it bypasses block but
                // still runs Intangible, Buffer, and Tungsten Rod reductions.
                // Sources: LoseHPAction.java, IntangiblePlayerPower.java,
                // BufferPower.java, TungstenRod.java, and AbstractPlayer.java::damage.
                engine.player_lose_hp_from_damage(-amount);
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
            // DamageAction and DamageAllEnemiesAction clear queued card-
            // manipulation actions after the final enemy dies. AddCard models
            // MakeTempCard actions, so it must not survive a preceding lethal.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAllEnemiesAction.java
            if engine.state.combat_over || engine.state.is_victory() {
                return;
            }
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            if pile == Pile::Hand {
                // MakeTempCardInHandAction sends cards beyond the ten-card hand
                // cap to discard rather than deleting them, and applies Master
                // Reality to each copy. Source: MakeTempCardInHandAction.java.
                engine.add_temp_cards_to_hand(name, count);
            } else {
                for _ in 0..count {
                    let card = engine.temp_card(name);
                    push_to_pile(engine, pile, card);
                }
            }
            // Shuffle draw pile if cards were added to it
            if pile == Pile::Draw && count > 0 {
                engine.shuffle_draw_pile();
            }
        }

        SimpleEffect::AddCardToRandomDrawSpot(name, ref amount_src) => {
            // A preceding lethal DamageAction clears queued card-manipulation
            // actions, including MakeTempCardInDrawPileAction.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
            if engine.state.combat_over || engine.state.is_victory() {
                return;
            }
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            for _ in 0..count {
                let card = engine.temp_card(name);
                if engine.state.draw_pile.is_empty() {
                    engine.state.draw_pile.push(card);
                } else {
                    let idx = engine.card_random_rng.random_range(
                        0,
                        (engine.state.draw_pile.len() - 1) as i32,
                    ) as usize;
                    engine.state.draw_pile.insert(idx, card);
                }
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
            let mut copy = ctx.card_inst;
            // AbstractCard.makeStatEquivalentCopy preserves upgrade, current
            // costs, misc, freeToPlayOnce, and bottle flags, but not transient
            // retain, purge-on-use, or exhaust-on-use state.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
            copy.flags &= crate::combat_types::CardInstance::FLAG_UPGRADED
                | crate::combat_types::CardInstance::FLAG_FREE
                | crate::combat_types::CardInstance::FLAG_INNATE;
            push_to_pile(engine, pile, copy);
            if pile == Pile::Draw {
                engine.shuffle_draw_pile();
            }
        }

        // -- Channel orb --
        SimpleEffect::ChannelOrb(orb_type, ref amount_src) => {
            // DamageAction removes later non-damage actions when the last
            // monster dies. Damage-then-channel cards (Meteor Strike, Ball
            // Lightning, Cold Snap, Doom and Gloom) therefore do not channel
            // after a combat-ending hit.
            // Java: actions/common/DamageAction.java and actions/GameActionManager.java.
            if engine.state.combat_over || engine.state.is_victory() {
                return;
            }
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            for _ in 0..count {
                engine.channel_orb(orb_type);
            }
        }

        SimpleEffect::ChannelRandomOrb(ref amount_src) => {
            let count = resolve_card_amount(engine, ctx, amount_src).max(0);
            // AbstractOrb.getRandomOrb(true) uses this exact list order and
            // cardRandomRng.random(3), consuming one counter tick per orb.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/orbs/AbstractOrb.java
            let orb_types = [
                crate::orbs::OrbType::Dark,
                crate::orbs::OrbType::Frost,
                crate::orbs::OrbType::Lightning,
                crate::orbs::OrbType::Plasma,
            ];
            for _ in 0..count {
                let idx = engine.card_random_rng.random(3) as usize;
                engine.channel_orb(orb_types[idx]);
            }
        }

        SimpleEffect::RemoveOrbSlot => {
            let focus = engine.state.player.focus();
            let evoke = engine.state.orb_slots.remove_slot(focus);
            engine.apply_evoke_effect(evoke);
        }

        SimpleEffect::TriggerDarkPassive => {
            engine.trigger_dark_impulse();
        }

        SimpleEffect::TriggerAllOrbPassives => {
            engine.trigger_orb_impulse();
        }

        SimpleEffect::EvokeAndRechannelFrontOrb => {
            engine.evoke_and_rechannel_front_orb();
        }

        SimpleEffect::EvokeOrbWithoutRemoving => {
            engine.evoke_front_orb_without_removing();
        }

        // -- Fission --
        SimpleEffect::ResolveFission { evoke } => {
            // FissionAction snapshots filledOrbCount, removes/evokes all orbs,
            // then gains that much energy before drawing that many cards.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/FissionAction.java
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
                if matches!(ctx.card.id, "Multi-Cast" | "Multi-Cast+") {
                    engine.multicast_front_orb_n(count as usize);
                } else {
                    engine.evoke_front_orb_n(count as usize);
                }
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

        SimpleEffect::ShuffleAllAndDraw(ref amount_src) => {
            let draw_count = resolve_card_amount(engine, ctx, amount_src);
            engine.shuffle_all_and_draw(draw_count);
        }

        // -- Discard random cards from a pile --
        SimpleEffect::DiscardRandomCardsFromPile(pile, count) => {
            execute_discard_random_cards_from_pile(engine, pile, count);
        }

        // -- Play the top card of the draw pile through the normal free-play path --
        SimpleEffect::PlayTopCardOfDraw => {
            engine.play_top_card_of_draw(true);
        }

        // -- Deal flat damage (no strength/stance modifiers) --
        SimpleEffect::DealDamage(target, ref amount_src) => {
            if matches!(
                amount_src,
                AmountSource::DrawPileSize
                    | AmountSource::StatusValueTimesMagic(_)
                    | AmountSource::PlayerBlock
            )
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
                    if engine.instant_kill_enemy(tidx) {
                        ctx.enemy_killed = true;
                    }
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
                    let hp_damage = engine.enemy_lose_hp_from_damage(idx, mark);
                    total_mark_damage += hp_damage;
                    if !engine.state.enemies[idx].is_alive() {
                        any_killed = true;
                    }
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
                let before = card;
                let current = if card.misc >= 0 {
                    card.misc as i32
                } else {
                    ctx.card.base_block.max(0)
                };
                let next = (current + delta).max(0) as i16;
                card.misc = next;
                ctx.card_inst.misc = next;
                engine.runtime_played_card = Some(card);
                if matches!(ctx.card.id, "Genetic Algorithm" | "Genetic Algorithm+") {
                    engine.sync_genetic_algorithm_master_deck(before, next);
                }
            }
        }

        SimpleEffect::ModifyPlayedCardDamage(ref amount_src) => {
            // ModifyDamageAction is CARD_MANIPULATION, so DamageAction removes
            // it from the queue when the hit kills the final monster. The
            // played card still grows when only its target dies and another
            // monster remains alive.
            // Java: cards/red/Rampage.java, cards/green/GlassKnife.java,
            // actions/common/DamageAction.java, and actions/GameActionManager.java.
            if matches!(
                ctx.card.id,
                "Rampage" | "Rampage+" | "Glass Knife" | "Glass Knife+"
            ) && (engine.state.combat_over || engine.state.is_victory())
            {
                return;
            }
            let delta = resolve_card_amount(engine, ctx, amount_src);
            if let Some(mut card) = engine.runtime_played_card {
                let before = card;
                let current = if card.misc >= 0 {
                    card.misc as i32
                } else {
                    ctx.card.base_damage
                };
                let next = (current + delta).max(0) as i16;
                card.misc = next;
                ctx.card_inst.misc = next;
                engine.runtime_played_card = Some(card);
                if matches!(ctx.card.id, "RitualDagger" | "RitualDagger+") {
                    engine.sync_ritual_dagger_master_deck(before, next);
                }
            }
        }

        SimpleEffect::IncreaseAllClawDamage(ref amount_src) => {
            let delta = resolve_card_amount(engine, ctx, amount_src);
            engine.increase_all_claw_damage(delta);
            if let Some(updated) = engine.runtime_played_card {
                ctx.card_inst = updated;
            }
        }

        // -- Heal HP (capped at max) --
        SimpleEffect::HealHp(_target, ref amount_src) => {
            // Post-combat cleanup explicitly preserves HealAction. Reaper's
            // VampireDamageAllEnemiesAction queues its accumulated heal before
            // cleanup, so terminal Reaper/Bite hits still heal the player.
            // Java: actions/unique/VampireDamageAllEnemiesAction.java and
            // actions/GameActionManager.java::clearPostCombatActions.
            let amount = resolve_card_amount(engine, ctx, amount_src);
            if amount > 0 {
                engine.heal_player(amount);
            }
        }

        // -- Increment counter status --
        SimpleEffect::IncrementCounter(status_id, threshold) => {
            let next = engine.state.player.status(status_id) + 1;
            let next = if threshold > 1 { next.min(threshold) } else { next };
            engine.state.player.set_status(status_id, next);
        }

        // -- Modify max HP --
        SimpleEffect::ModifyMaxHp(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            engine.state.player.max_hp = (engine.state.player.max_hp + amount).max(1);
            if amount > 0 {
                // AbstractCreature.java::increaseMaxHp raises maxHealth first,
                // then routes the same amount through heal(), so Mark of the
                // Bloom and Magic Flower still modify the current-HP increase.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
                engine.state.heal_player(amount);
            } else {
                engine.state.player.hp = engine.state.player.hp.min(engine.state.player.max_hp);
            }
        }

        // -- Modify max energy --
        SimpleEffect::ModifyMaxEnergy(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            engine.state.max_energy = (engine.state.max_energy + amount).max(0);
            engine.state.energy = engine.state.energy.min(engine.state.max_energy);
        }

        // -- Modify live run gold --
        SimpleEffect::ModifyGold(ref amount_src) => {
            let amount = resolve_card_amount(engine, ctx, amount_src);
            engine.gain_run_gold(amount);
        }

        // -- Flee combat --
        SimpleEffect::FleeCombat => {
            engine.state.combat_over = true;
        }
        SimpleEffect::UpgradeRandomMasterDeckCard => {
            engine.upgrade_random_master_deck_card();
        }
    }
}

// ===========================================================================
// Status helpers
// ===========================================================================

/// Debuff status IDs that should route through apply_debuff (handles Artifact).
fn is_debuff(status: StatusId, amount: i32) -> bool {
    status == sid::WEAKENED
        || status == sid::VULNERABLE
        || status == sid::FRAIL
        || status == sid::POISON
        || status == sid::CONSTRICTED
        // LockOnPower declares DEBUFF, so Artifact blocks the card before its
        // orb-damage modifier can become active.
        || status == sid::LOCK_ON
        // CorpseExplosionPower.java declares PowerType.DEBUFF, so Artifact blocks it.
        || status == sid::CORPSE_EXPLOSION
        || status == sid::NO_DRAW
        // NoBlockPower declares DEBUFF, so Artifact consumes a charge and
        // blocks Panic Button's restriction after its block action resolves.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoBlockPower.java
        || status == sid::NO_BLOCK
        || status == sid::BIASED_COG_FOCUS_LOSS
        // LoseStrengthPower is explicitly PowerType.DEBUFF, so Artifact can
        // make Flex's Strength gain permanent.
        || status == sid::LOSE_STRENGTH
        // FocusPower.updateDescription classifies negative applications as
        // DEBUFF and positive applications as BUFF.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FocusPower.java
        || (status == sid::FOCUS && amount < 0)
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
            if is_debuff(status, amount) {
                crate::powers::apply_debuff(&mut engine.state.player, status, amount);
            } else {
                add_player_status(engine, status, amount);
            }
        }
        // Card effects do not currently install owner-aware runtime handlers.
        // Treat SelfEntity as player until self-owned status handlers become explicit.
        Target::SelfEntity => {
            if is_debuff(status, amount) {
                crate::powers::apply_debuff(&mut engine.state.player, status, amount);
            } else {
                add_player_status(engine, status, amount);
            }
        }
        Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0
                && (idx as usize) < engine.state.enemies.len()
                && engine.state.enemies[idx as usize].is_targetable()
            {
                let i = idx as usize;
                // ApplyPowerAction immediately cancels for a dead or escaped
                // target. This matters for damage-then-debuff cards whose hit
                // kills the selected monster, such as Neutralize.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ApplyPowerAction.java
                if is_debuff(status, amount) {
                    engine.apply_player_debuff_to_enemy(i, status, amount);
                } else {
                    engine.state.enemies[i].entity.add_status(status, amount);
                }
            }
        }
        Target::AllEnemies => {
            let living = engine.state.living_enemy_indices();
            for i in living {
                if is_debuff(status, amount) {
                    engine.apply_player_debuff_to_enemy(i, status, amount);
                } else {
                    engine.state.enemies[i].entity.add_status(status, amount);
                }
            }
        }
        Target::RandomEnemy => {
            let living = engine.state.living_enemy_indices();
            if !living.is_empty() {
                // Card-owned random debuff targeting uses cardRandomRng. In
                // particular, Bouncing Flask calls MonsterGroup.getRandomMonster
                // once for the initial target and once per recursive bounce.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/BouncingFlask.java
                // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/BouncingFlaskAction.java
                // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
                let selected = engine
                    .card_random_rng
                    .random_range(0, (living.len() - 1) as i32) as usize;
                let idx = living[selected];
                if is_debuff(status, amount) {
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
        // Treat SelfEntity as player until self-owned status handlers become explicit.
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
    if status == sid::ORB_SLOTS && amount > 0 {
        let before = engine.state.orb_slots.max_slots;
        for _ in 0..amount {
            engine.state.orb_slots.add_slot();
        }
        let gained = engine.state.orb_slots.max_slots.saturating_sub(before) as i32;
        engine.state.player.add_status(status, gained);
        return;
    }
    if matches!(status, sid::COLLECT_MIRACLES | sid::LIKE_WATER | sid::ENERGIZED) {
        // Java CollectPower.stackPower(), LikeWaterPower.stackPower(), and
        // EnergizedPower.stackPower()/EnergizedBluePower.stackPower() cap at 999.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CollectPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/LikeWaterPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EnergizedPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EnergizedBluePower.java
        let next = (engine.state.player.status(status) + amount).min(999);
        engine.state.player.set_status(status, next);
    } else {
        engine.state.player.add_status(status, amount);
    }
    // ReboundPower is applied during Rebound.use, before UseCardAction invokes
    // onAfterUseCard. Install its runtime handler immediately so the new power
    // can ignore that first callback via its Java `justEvoked` flag.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ReboundPower.java
    if status == sid::REBOUND {
        engine.rebuild_effect_runtime();
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
            if current != 0 {
                let multiplied = current.saturating_mul(multiplier);
                // LimitBreakAction reapplies the current signed Strength via
                // StrengthPower.stackPower, which clamps its result to ±999.
                // Java: actions/unique/LimitBreakAction.java and
                // powers/StrengthPower.java.
                let next = if status == sid::STRENGTH {
                    multiplied.clamp(-999, 999)
                } else {
                    multiplied
                };
                engine.state.player.set_status(status, next);
            }
        }
        // Card effects do not currently install owner-aware runtime handlers.
        // Treat SelfEntity as player until self-owned status handlers become explicit.
        Target::SelfEntity => {
            let current = engine.state.player.status(status);
            if current != 0 {
                let multiplied = current.saturating_mul(multiplier);
                let next = if status == sid::STRENGTH {
                    multiplied.clamp(-999, 999)
                } else {
                    multiplied
                };
                engine.state.player.set_status(status, next);
            }
        }
        Target::SelectedEnemy => {
            let idx = ctx.target_idx;
            if idx >= 0 && (idx as usize) < engine.state.enemies.len() {
                let i = idx as usize;
                let current = engine.state.enemies[i].entity.status(status);
                if current > 0 {
                    if status == sid::POISON {
                        // DoublePoisonAction / TriplePoisonAction do not set the
                        // final amount directly: they enqueue ApplyPowerAction
                        // for the additional current or current*2 Poison. That
                        // application is blocked by Artifact and receives Snake
                        // Skull's +1 Poison constructor bonus.
                        // Java: actions/unique/DoublePoisonAction.java,
                        // TriplePoisonAction.java, and common/ApplyPowerAction.java.
                        let additional = current * (multiplier - 1).max(0);
                        engine.apply_player_debuff_to_enemy(i, status, additional);
                    } else {
                        engine.state.enemies[i].entity.set_status(status, current * multiplier);
                    }
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
        AmountSource::ModifiedBlock => {
            let base = if ctx.card_inst.misc >= 0 {
                ctx.card_inst.misc as i32
            } else {
                ctx.card.base_block.max(0)
            };
            damage::calculate_block(
                base,
                engine.state.player.dexterity(),
                engine.state.player.is_frail(),
            )
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
        AmountSource::PriorAttacksThisTurn => {
            (engine.state.attacks_played_this_turn - 1).max(0)
        }
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
            engine.player_lose_hp_from_damage(amount);
        }
        // Card effects do not currently install owner-aware runtime handlers.
        // Treat SelfEntity as player until self-owned status handlers become explicit.
        Target::SelfEntity => {
            engine.player_lose_hp_from_damage(amount);
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
    static EMPTY_CARD: OnceLock<crate::cards::CardDef> = OnceLock::new();
    let empty_card = EMPTY_CARD.get_or_init(|| crate::cards::CardDef {
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
        metadata: crate::effects::types::CardMetadata::default(),
        effect_data: &[],
        complex_hook: None,
    });

    let mut ctx = CardPlayContext {
        card: empty_card,
        card_inst: crate::combat_types::CardInstance::new(0),
        target_idx: trigger_ctx.target_idx,
        x_value: 0,
        pen_nib_active: false,
        vigor: 0,
        total_unblocked_damage: 0,
        enemy_killed: false,
        hand_size_at_play: 0,
        last_bulk_count: 0,
        last_drawn_card_types: Vec::new(),
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
                // HeelHookAction/DropkickAction queue damage before their
                // reward actions. A final kill clears those queued follow-ups,
                // while killing one target in a multi-monster fight does not.
                !engine.state.is_victory()
                    && engine.state.enemies[idx as usize].entity.status(status) > 0
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

        Condition::EnemyKilledNonMinion => {
            ctx.enemy_killed
                && ctx.target_idx >= 0
                && (ctx.target_idx as usize) < engine.state.enemies.len()
                && !engine.state.enemies[ctx.target_idx as usize].is_minion
                && engine.state.enemies[ctx.target_idx as usize]
                    .entity
                    .status(sid::REBIRTH_PENDING)
                    == 0
        }

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
    // DamageAction clears queued post-combat actions after a lethal final hit.
    // Card-manipulation actions such as Headbutt therefore never open after
    // the room has entered its battle-ending state.
    if engine.state.is_victory() {
        return;
    }

    let pile = get_pile(engine, source);
    let post_choice_draw = if post_choice_draw_src == AmountSource::Fixed(0) {
        0
    } else {
        resolve_card_amount(engine, ctx, &post_choice_draw_src).max(0)
    };

    // Build options from the source pile, applying filter
    let mut options: Vec<ChoiceOption> = pile.iter()
        .enumerate()
        .filter(|(_, card)| matches_filter(engine, card, filter))
        .map(|(i, _)| make_choice_option(source, i))
        .collect();

    if matches!(ctx.card.id, "Omniscience" | "Omniscience+")
        && matches!(source, Pile::Draw)
        && matches!(action, ChoiceAction::PlayForFree)
    {
        // OmniscienceAction copies the draw pile, then stably sorts by name,
        // rarity descending, and finally moves Status cards to the end.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/OmniscienceAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
        options.sort_by(|left, right| {
            let ChoiceOption::DrawCard(left_idx) = left else {
                return std::cmp::Ordering::Equal;
            };
            let ChoiceOption::DrawCard(right_idx) = right else {
                return std::cmp::Ordering::Equal;
            };
            let left_card = pile[*left_idx];
            let right_card = pile[*right_idx];
            let left_def = engine.card_registry.card_def_by_id(left_card.def_id);
            let right_def = engine.card_registry.card_def_by_id(right_card.def_id);
            let left_status = left_def.card_type == CardType::Status;
            let right_status = right_def.card_type == CardType::Status;
            left_status
                .cmp(&right_status)
                .then_with(|| {
                    omniscience_rarity_rank(right_def.id)
                        .cmp(&omniscience_rarity_rank(left_def.id))
                })
                .then_with(|| left_def.name.cmp(right_def.name))
        });
    }

    if options.is_empty() {
        // ExhaustAction with an empty hand completes immediately; actions
        // queued after it (Burning Pact's DrawCardAction) still resolve.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
        if post_choice_draw > 0 {
            engine.draw_cards(post_choice_draw);
        }
        return;
    }

    let is_meditate = matches!(ctx.card.id, "Meditate" | "Meditate+")
        && matches!(source, Pile::Discard)
        && matches!(action, ChoiceAction::MoveToHand);
    let requested_min = resolve_card_amount(engine, ctx, &min_picks_src).max(0) as usize;
    let requested_max = resolve_card_amount(engine, ctx, &max_picks_src).max(0) as usize;
    let max_picks = requested_max.min(options.len());

    if max_picks == 0 {
        return;
    }

    // Base Forethought skips hand selection when exactly one card remains.
    // Forethought+ still opens the any-number screen and permits choosing zero.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
    if ctx.card.id == "Forethought"
        && source == Pile::Hand
        && action == ChoiceAction::PutOnBottomFreeIfCostly
        && options.len() == 1
    {
        if let ChoiceOption::HandCard(index) = options[0] {
            engine.move_forethought_cards_to_bottom(&[index]);
        }
        return;
    }

    if ctx.card.id == "Armaments"
        && source == Pile::Hand
        && action == ChoiceAction::Upgrade
        && options.len() == 1
    {
        if let ChoiceOption::HandCard(index) = options[0] {
            engine.card_registry.upgrade_card(&mut engine.state.hand[index]);
        }
        return;
    }

    // DiscardPileToTopOfDeckAction moves a singleton discard pile directly;
    // grid selection is used only when more than one card is available.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscardPileToTopOfDeckAction.java
    if matches!(ctx.card.id, "Headbutt" | "Headbutt+")
        && source == Pile::Discard
        && action == ChoiceAction::PutOnTopOfDraw
        && options.len() == 1
    {
        if let ChoiceOption::DiscardCard(index) = options[0] {
            let card = engine.state.discard_pile.remove(index);
            engine.state.draw_pile.push(card);
        }
        return;
    }

    // BetterDiscardPileToHandAction directly moves the entire discard pile
    // when its size is at most the mandatory request. Hologram requests one,
    // so a singleton never opens grid selection.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDiscardPileToHandAction.java
    if matches!(ctx.card.id, "Hologram" | "Hologram+")
        && source == Pile::Discard
        && action == ChoiceAction::MoveToHand
        && options.len() == 1
    {
        if engine.state.hand.len() < 10 {
            if let ChoiceOption::DiscardCard(index) = options[0] {
                let card = engine.state.discard_pile.remove(index);
                engine.state.hand.push(card);
            }
        }
        return;
    }

    // The Skill/AttackFromDeckToHandAction pair skips grid selection when
    // exactly one matching card is eligible and moves it directly into the hand
    // (or discard if the hand is already full).
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/SkillFromDeckToHandAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/AttackFromDeckToHandAction.java
    if matches!(
        ctx.card.id,
        "Secret Technique" | "Secret Technique+" | "Secret Weapon" | "Secret Weapon+"
    )
        && source == Pile::Draw
        && action == ChoiceAction::MoveToHand
        && options.len() == 1
    {
        if let ChoiceOption::DrawCard(index) = options[0] {
            let card = engine.state.draw_pile.remove(index);
            if engine.state.hand.len() >= 10 {
                engine.state.discard_pile.push(card);
            } else {
                engine.state.hand.push(card);
            }
        }
        return;
    }

    // MeditateAction is mandatory. When the whole discard pile fits, Java
    // moves it without opening grid select.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/MeditateAction.java
    if is_meditate && options.len() <= max_picks {
        for card in &mut engine.state.discard_pile {
            card.set_retained(true);
        }
        let move_count = options
            .len()
            .min(10usize.saturating_sub(engine.state.hand.len()));
        for _ in 0..move_count {
            let card = engine.state.discard_pile.remove(0);
            engine.state.hand.push(card);
        }
        return;
    }

    let min_picks = if is_meditate {
        max_picks
    } else {
        requested_min
    };

    // A fixed-count DiscardAction automatically discards the whole hand when
    // hand.size() <= amount, firing each card's manual-discard hooks. It opens
    // hand selection only when more cards remain than the requested amount.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DiscardAction.java
    if action == ChoiceAction::Discard
        && source == Pile::Hand
        && filter == CardFilter::All
        && requested_min == requested_max
        && options.len() <= requested_max
    {
        let mut indices: Vec<usize> = options
            .iter()
            .filter_map(|option| match option {
                ChoiceOption::HandCard(index) => Some(*index),
                _ => None,
            })
            .collect();
        indices.sort_unstable_by(|left, right| right.cmp(left));
        let mut discarded_cards = Vec::with_capacity(indices.len());
        for index in indices {
            let card = engine.state.hand.remove(index);
            engine.state.discard_pile.push(card);
            discarded_cards.push(card);
        }
        for card in discarded_cards {
            engine.on_card_discarded(card);
        }
        if post_choice_draw > 0 {
            engine.draw_cards(post_choice_draw);
        }
        return;
    }

    // Non-random, fixed-count ExhaustAction automatically exhausts the whole
    // hand when hand.size() <= amount instead of opening the selection screen.
    // This covers Burning Pact/True Grit with a singleton remaining hand while
    // preserving any-number choices such as Purity (min != max).
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
    if action == ChoiceAction::Exhaust
        && source == Pile::Hand
        && requested_min == requested_max
        && options.len() <= requested_max
    {
        let mut indices: Vec<usize> = options
            .iter()
            .filter_map(|option| match option {
                ChoiceOption::HandCard(index) => Some(*index),
                _ => None,
            })
            .collect();
        indices.sort_unstable_by(|left, right| right.cmp(left));
        for index in indices {
            let card = engine.state.hand.remove(index);
            engine.state.exhaust_pile.push(card);
            engine.trigger_card_on_exhaust(card);
        }
        if post_choice_draw > 0 {
            engine.draw_cards(post_choice_draw);
        }
        return;
    }

    // RecycleAction directly resolves the only remaining hand card. It opens
    // hand selection only when two or more cards remain after Recycle itself
    // has left the hand.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/RecycleAction.java
    if action == ChoiceAction::ExhaustAndGainEnergy
        && source == Pile::Hand
        && options.len() == 1
    {
        if let ChoiceOption::HandCard(index) = options[0] {
            engine.recycle_hand_card(index);
        }
        return;
    }

    // DualWieldAction skips the hand-select screen when exactly one Attack or
    // Power is eligible and immediately creates its copies.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DualWieldAction.java
    if action == ChoiceAction::CopyToHand && options.len() == 1 {
        if let ChoiceOption::HandCard(index) = options[0] {
            let card = engine.state.hand[index];
            engine.add_dual_wield_copies(card, ctx.card.base_magic.max(1) as usize);
        }
        return;
    }

    // NightmareAction immediately applies NightmarePower when the hand left
    // after playing Night Terror contains exactly one card.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/NightmareAction.java
    if action == ChoiceAction::StoreCardForNextTurnCopies && options.len() == 1 {
        if let ChoiceOption::HandCard(index) = options[0] {
            let card = engine.state.hand[index];
            engine.store_nightmare_copies(card, ctx.card.base_magic.max(1) as usize);
        }
        return;
    }

    // ExhumeAction moves a lone non-Exhume card immediately. With a larger
    // original exhaust pile it still opens grid select after hiding Exhumes,
    // even when that leaves only one eligible option.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ExhumeAction.java
    let is_exhume = matches!(ctx.card.id, "Exhume" | "Exhume+")
        && source == Pile::Exhaust
        && action == ChoiceAction::MoveToHand;
    if is_exhume && pile.len() == 1 && options.len() == 1 {
        if engine.state.hand.len() < 10 {
            if let ChoiceOption::ExhaustCard(index) = options[0] {
                let card = engine.state.exhaust_pile.remove(index);
                engine.state.hand.push(card);
            }
        }
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
    if post_choice_draw > 0 {
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
        CardFilter::NonExhume => {
            let id = engine.card_registry.card_def_by_id(card.def_id).id;
            id.strip_suffix('+').unwrap_or(id) != "Exhume"
        }
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
            let current_cost = if card.cost >= 0 {
                card.cost as i32
            } else {
                def.cost
            };
            current_cost == 0 || card.is_free()
        }
        CardFilter::Upgradeable => {
            engine.card_registry.can_upgrade_card(card)
        }
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
        ChoiceAction::PutOnBottomFreeIfCostly => ChoiceReason::ForethoughtPick,
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

        BulkAction::ExhaustRandom => {
            // FiendFireAction queues one random ExhaustAction per card. Each
            // getRandomCard call consumes cardRandomRng, including the final
            // random(0) call with one card remaining.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/FiendFireAction.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
            let pile_ref = get_pile_mut(engine, pile);
            let mut candidates = Vec::with_capacity(matching.len());
            for &index in matching.iter().rev() {
                candidates.push(pile_ref.remove(index));
            }
            candidates.reverse();

            let mut exhausted = Vec::with_capacity(candidates.len());
            while !candidates.is_empty() {
                let index = engine
                    .card_random_rng
                    .random((candidates.len() - 1) as i32) as usize;
                exhausted.push(candidates.remove(index));
            }

            let exhausted_cards = exhausted.clone();
            engine.state.exhaust_pile.extend(exhausted);
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
            for &i in &matching {
                let current_cost = get_pile(engine, pile).get(i).map(|card| {
                    let def = engine.card_registry.card_def_by_id(card.def_id);
                    if card.cost >= 0 {
                        card.cost as i32
                    } else if card.base_cost >= 0 {
                        card.base_cost as i32
                    } else {
                        def.cost
                    }
                });
                if current_cost.is_some_and(|current| current > cost) {
                    get_pile_mut(engine, pile)[i].set_cost_for_turn(cost as i8);
                }
            }
        }

        BulkAction::SetCost(cost) => {
            for &i in &matching {
                let costs = get_pile(engine, pile).get(i).map(|card| {
                    let def = engine.card_registry.card_def_by_id(card.def_id);
                    let permanent = if card.base_cost >= 0 {
                        card.base_cost as i32
                    } else {
                        def.cost
                    };
                    let current = if card.cost >= 0 {
                        card.cost as i32
                    } else {
                        permanent
                    };
                    (current, permanent)
                });
                if let Some((current, permanent)) = costs {
                    let card = &mut get_pile_mut(engine, pile)[i];
                    // EnlightenmentAction always applies the turn-only branch,
                    // then (upgraded) changes the base cost independently.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
                    if current > cost {
                        card.set_cost_for_turn(cost as i8);
                    }
                    if permanent > cost {
                        card.base_cost = cost as i8;
                    }
                }
            }
        }

        BulkAction::MoveToHand => {
            if pile == Pile::Hand {
                return; // No-op: already in hand
            }
            // DiscardToHandAction resolves queued cards in discard scan order,
            // stopping naturally when the hand reaches ten.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/AllCostToHandAction.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DiscardToHandAction.java
            let hand_capacity = 10 - engine.state.hand.len();
            let pile_ref = get_pile_mut(engine, pile);
            let selected = matching.into_iter().take(hand_capacity).collect::<Vec<_>>();
            let moved = selected
                .iter()
                .filter_map(|index| pile_ref.get(*index).copied())
                .collect::<Vec<_>>();
            for index in selected.into_iter().rev() {
                pile_ref.remove(index);
            }
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

    // DiscardAction takes a no-RNG whole-hand branch when hand size is at
    // most the requested amount. All Out Attack requests exactly one.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DiscardAction.java
    if pile == Pile::Hand && engine.state.hand.len() <= count {
        while let Some(card) = engine.state.hand.pop() {
            engine.state.discard_pile.push(card);
            engine.on_card_discarded(card);
        }
        return;
    }

    for _ in 0..count {
        let len = get_pile_mut(engine, pile).len();
        if len == 0 {
            break;
        }
        let idx = engine.card_random_rng.random((len - 1) as i32) as usize;
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
    generate_random_cards_to_hand(engine, pool, count, cost_rule);
}

pub(crate) fn generate_random_cards_to_hand(
    engine: &mut CombatEngine,
    pool: GeneratedCardPool,
    count: usize,
    cost_rule: GeneratedCostRule,
) {
    for _ in 0..count {
        if let Some(mut card) = generate_random_card(engine, pool) {
            apply_generated_upgrade_rule(
                engine,
                &mut card,
                upgrade_rule_from_cost_rule(cost_rule),
            );
            apply_generated_cost_rule(&mut card, cost_rule);
            // MakeTempCardInHandAction still creates a card when the hand is
            // full, sending that copy to discard. Cards such as Jack Of All
            // Trades choose each random card before those queued actions
            // resolve, so every requested copy also consumes its RNG roll.
            // Java: cards/colorless/JackOfAllTrades.java and
            // actions/common/MakeTempCardInHandAction.java.
            if engine.state.hand.len() < 10 {
                engine.state.hand.push(card);
            } else {
                engine.state.discard_pile.push(card);
            }
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
    let mut generated = Vec::with_capacity(count);
    for _ in 0..count {
        let at_hand_cap = matches!(destination, GeneratedDestination::Hand)
            && engine.state.hand.len() + generated.len() >= 10;
        if at_hand_cap {
            break;
        }
        if let Some(mut card) = generate_random_card(engine, pool) {
            let master_reality_rule = if engine.state.player.status(sid::MASTER_REALITY) > 0 {
                GeneratedUpgradeRule::Upgrade
            } else {
                GeneratedUpgradeRule::Base
            };
            apply_generated_upgrade_rule(
                engine,
                &mut card,
                combine_generated_upgrade_rules(
                    combine_generated_upgrade_rules(upgrade_rule, master_reality_rule),
                    upgrade_rule_from_cost_rule(cost_rule),
                ),
            );
            apply_generated_cost_rule(&mut card, cost_rule);
            generated.push(card);
        }
    }

    match destination {
        GeneratedDestination::Hand => engine.state.hand.extend(generated),
        GeneratedDestination::Draw => {
            // Chrysalis/Metamorphosis select every random card immediately in
            // card.use, then their queued MakeTempCardInDrawPileActions resolve
            // in order. `addToRandomSpot` leaves the existing pile order intact,
            // consuming cardRandomRng only when the pile is non-empty.
            // Java: cards/colorless/Chrysalis.java and
            // cards/CardGroup.java::addToRandomSpot.
            for card in generated {
                if engine.state.draw_pile.is_empty() {
                    engine.state.draw_pile.push(card);
                } else {
                    let idx = engine
                        .card_random_rng
                        .random((engine.state.draw_pile.len() - 1) as i32)
                        as usize;
                    engine.state.draw_pile.insert(idx, card);
                }
            }
        }
    }
}

pub(crate) fn generate_random_card(
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
    let choice_index = if matches!(
        pool,
        GeneratedCardPool::Colorless
            | GeneratedCardPool::Attack
            | GeneratedCardPool::Skill
            | GeneratedCardPool::DefectCommon
            | GeneratedCardPool::DefectPower
            | GeneratedCardPool::WatcherPower
            | GeneratedCardPool::WatcherAny
    ) {
        // AbstractDungeon.returnTrulyRandomCardInCombat(type) selects from
        // the source color pools with cardRandomRng.
        engine.card_random_rng.random((pool_cards.len() - 1) as i32) as usize
    } else {
        engine.rng_gen_range(0..pool_cards.len())
    };
    let choice = pool_cards[choice_index];
    Some(engine.temp_card(choice))
}

pub(crate) fn generate_unique_random_cards(
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
        let choice = pool_cards
            [engine.card_random_rng.random((pool_cards.len() - 1) as i32) as usize];
        if seen.insert(choice) {
            // DiscoveryAction previews base copies; Master Reality upgrades
            // only the selected copy during resolution.
            picked.push(engine.card_registry.make_card(choice));
        }
    }
    picked
}

pub(crate) fn generated_card_pool(
    engine: &CombatEngine,
    pool: GeneratedCardPool,
) -> Vec<&'static str> {
    match pool {
        GeneratedCardPool::Colorless => COLORLESS_GENERATION_POOL.to_vec(),
        GeneratedCardPool::Attack => WATCHER_ATTACK_GENERATION_POOL.to_vec(),
        GeneratedCardPool::Skill => WATCHER_SKILL_GENERATION_POOL.to_vec(),
        GeneratedCardPool::Power => engine
            .card_registry
            .all_card_defs()
            .iter()
            .filter(|def| def.card_type == CardType::Power && !def.id.ends_with('+'))
            .map(|def| def.id)
            .collect(),
        GeneratedCardPool::DefectCommon => DEFECT_COMMON_GENERATION_POOL.to_vec(),
        GeneratedCardPool::DefectPower => DEFECT_POWER_GENERATION_POOL.to_vec(),
        GeneratedCardPool::WatcherPower => engine
            .card_registry
            .all_card_defs()
            .iter()
            .filter(|def| !def.id.ends_with('+'))
            .filter(|def| {
                matches!(
                    generated_card_meta(def.id),
                    Some(GeneratedCardMeta {
                        card_type: CardType::Power,
                        rarity: GeneratedPoolRarity::Common
                            | GeneratedPoolRarity::Uncommon
                            | GeneratedPoolRarity::Rare,
                        watcher: true,
                    })
                )
            })
            .map(|def| def.id)
            .collect(),
        GeneratedCardPool::WatcherAny => engine
            .card_registry
            .all_card_defs()
            .iter()
            .filter(|def| !def.id.ends_with('+'))
            .filter(|def| {
                matches!(
                    generated_card_meta(def.id),
                    Some(GeneratedCardMeta {
                        rarity: GeneratedPoolRarity::Common
                            | GeneratedPoolRarity::Uncommon
                            | GeneratedPoolRarity::Rare,
                        watcher: true,
                        ..
                    })
                )
            })
            // returnTrulyRandomCardInCombat excludes HEALING-tagged cards.
            // Lesson Learned is the Watcher's only normal-rarity healing card.
            .filter(|def| def.id != "LessonLearned")
            .map(|def| def.id)
            .collect(),
        GeneratedCardPool::AnyColorAttackRarityWeighted => weighted_any_color_attack_ids(engine)
            .into_iter()
            .collect(),
    }
}

// Watcher's srcCommon/srcUncommon/srcRare pools in Java iteration order,
// excluding BASIC/SPECIAL cards and Wish's HEALING tag. CardLibrary stores the
// registered game cards in a 512-bucket HashMap; initializeCardPools preserves
// that bucket order within each rarity, then reverses it while copying each
// source pool through CardGroup.addToBottom.
// Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
const WATCHER_SKILL_GENERATION_POOL: &[&str] = &[
    "Prostrate", "Evaluate", "PathToVictory", "EmptyBody", "ClearTheMind", "Crescendo",
    "ThirdEye", "Protect", "Halt", "Pray", "EmptyMind", "Worship", "Swivel",
    "Perseverance", "Meditate", "WaveOfTheHand", "DeceiveReality", "InnerPeace", "Collect",
    "WreathOfFlame", "ForeignInfluence", "Indignation", "Sanctity", "Vengeance", "Judgement",
    "ConjureBlade", "Blasphemy", "Scrawl", "Vault", "Alpha", "Omniscience", "SpiritShield",
    "DeusExMachina",
];

// Watcher's srcCommon/srcUncommon/srcRare Attack pools in Java HashMap
// iteration order. Lesson Learned is excluded by its HEALING tag.
// Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
const WATCHER_ATTACK_GENERATION_POOL: &[&str] = &[
    "EmptyFist", "CrushJoints", "FollowUp", "CutThroughFate", "SashWhip",
    "FlurryOfBlows", "JustLucky", "FlyingSleeves", "BowlingBash", "Consecrate",
    "SignatureMove", "Weave", "Tantrum", "Conclude", "SandsOfTime", "FearNoEvil",
    "ReachHeaven", "Wallop", "CarveReality", "WindmillStrike", "TalkToTheHand",
    "WheelKick", "Brilliance", "Ragnarok",
];

// Defect's srcCommonCardPool in Java HashMap iteration order. CardLibrary
// inserts the unlocked blue cards into tmpPool, initializeCardPools adds each
// common to the top of commonCardPool, and getCard(COMMON, cardRandomRng)
// selects directly from that resulting order.
// Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
const DEFECT_COMMON_GENERATION_POOL: &[&str] = &[
    "Steam", "Cold Snap", "Leap", "Beam Cell", "Hologram", "Conserve Battery",
    "Sweeping Beam", "Turbo", "Coolheaded", "Gash", "Rebound", "Stack", "Barrage",
    "Compile Driver", "Redo", "Streamline", "Ball Lightning", "Go for the Eyes",
];

// Defect's srcUncommon/srcRare Power pools in Java HashMap iteration order.
// Self Repair is omitted because returnTrulyRandomCardInCombat excludes
// HEALING-tagged cards. There are no normal-rarity common Defect Powers.
// Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
const DEFECT_POWER_GENERATION_POOL: &[&str] = &[
    "Defragment", "Capacitor", "Heatsinks", "Static Discharge", "Loop", "Hello World", "Storm",
    "Biased Cognition", "Machine Learning", "Electrodynamics", "Buffer", "Echo Form", "Creative AI",
];

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
            // Fresh CardInstances use cost=-1 as "read base_cost". Java's
            // generated-card actions test the card's real cost, not this Rust
            // sentinel, before setting a turn-only zero cost.
            let current_cost = if card.cost >= 0 { card.cost } else { card.base_cost };
            if current_cost > 0 {
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
    Basic,
    Special,
    Common,
    Uncommon,
    Rare,
    Curse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GeneratedCardMeta {
    card_type: CardType,
    rarity: GeneratedPoolRarity,
    watcher: bool,
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
    let mut watcher = false;
    for line in include_str!("../../content/generated-cards.txt").lines() {
        if line.contains("============ WATCHER CARDS ============") {
            watcher = true;
        } else if line.starts_with("# ============") && line.contains("CARDS ============") {
            watcher = false;
        }
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
            "BASIC" => GeneratedPoolRarity::Basic,
            "SPECIAL" => GeneratedPoolRarity::Special,
            "COMMON" => GeneratedPoolRarity::Common,
            "UNCOMMON" => GeneratedPoolRarity::Uncommon,
            "RARE" => GeneratedPoolRarity::Rare,
            "CURSE" => GeneratedPoolRarity::Curse,
            _ => continue,
        };
        map.insert(
            card_id.to_string(),
            GeneratedCardMeta {
                card_type,
                rarity,
                watcher,
            },
        );
    }
    map
}

fn omniscience_rarity_rank(card_id: &str) -> u8 {
    match generated_card_meta(card_id).map(|meta| meta.rarity) {
        Some(GeneratedPoolRarity::Basic) => 0,
        Some(GeneratedPoolRarity::Special) => 1,
        Some(GeneratedPoolRarity::Common) => 2,
        Some(GeneratedPoolRarity::Uncommon) => 3,
        Some(GeneratedPoolRarity::Rare) => 4,
        Some(GeneratedPoolRarity::Curse) => 5,
        None => 0,
    }
}

fn weighted_any_color_attack_ids(engine: &CombatEngine) -> Vec<&'static str> {
    engine
        .card_registry
        .all_card_defs()
        .iter()
        .filter(|def| !def.id.ends_with('+'))
        .filter(|def| !foreign_influence_excludes_healing_attack(def.id))
        .filter(|def| {
            matches!(
                generated_card_meta(def.id),
                Some(GeneratedCardMeta {
                    card_type: CardType::Attack,
                    rarity: GeneratedPoolRarity::Common
                        | GeneratedPoolRarity::Uncommon
                        | GeneratedPoolRarity::Rare,
                    ..
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
        .filter(|def| !foreign_influence_excludes_healing_attack(def.id))
        .filter(|def| {
            matches!(
                generated_card_meta(def.id),
                Some(GeneratedCardMeta {
                    card_type: CardType::Attack,
                    rarity: card_rarity,
                    ..
                }) if card_rarity == rarity
            )
        })
        .map(|def| def.id)
        .collect()
}

fn foreign_influence_excludes_healing_attack(card_id: &str) -> bool {
    // CardLibrary.getAnyColorCard(type, rarity) rejects CardTags.HEALING.
    // These are the normal-rarity attacks carrying that tag in the Java card
    // library; Bite is SPECIAL and is already excluded by the rarity filter.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/{red/Feed.java,red/Reaper.java,purple/LessonLearned.java}
    matches!(card_id, "Feed" | "Reaper" | "LessonLearned")
}

fn roll_generated_attack_rarity(engine: &mut CombatEngine) -> GeneratedPoolRarity {
    // ForeignInfluenceAction.generateCardChoices uses cardRandomRng.random(99).
    let roll = engine.card_random_rng.random(99) as usize;
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
    // CardLibrary.getAnyColorCard first shuffles with a seed obtained from
    // cardRandomRng.randomLong(), then getRandomCard(true, rarity) sorts by
    // cardID and selects with cardRng. The shuffle therefore changes no card
    // ordering, but its RNG tick is observable and must still be consumed.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    let _shuffle_seed = engine.card_random_rng.random_long();
    let idx = engine.rng.random((bucket.len() - 1) as i32) as usize;
    // Foreign Influence presents base copies. Master Reality upgrades only the
    // selected copy when it is added to hand/discard.
    Some(engine.card_registry.make_card(bucket[idx]))
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

    while picked.len() < target {
        if let Some(card) = generate_weighted_any_color_attack_card(engine) {
            let card_name = engine.card_registry.card_name(card.def_id);
            if seen.insert(card_name) {
                picked.push(card);
            }
        }
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

// srcColorlessCardPool order produced by CardLibrary's 512-bucket HashMap and
// AbstractDungeon.addColorlessCards adding each normal-rarity card to the top.
// Bandage Up is then excluded by its HEALING tag.
// Java: helpers/CardLibrary.java and dungeons/AbstractDungeon.java.
const COLORLESS_GENERATION_POOL: &[&str] = &[
    "Madness", "Thinking Ahead", "Mind Blast", "Metamorphosis", "Jack Of All Trades",
    "Swift Strike", "Good Instincts", "Master of Strategy", "Magnetism", "Finesse",
    "Discovery", "Chrysalis", "Transmutation", "Panacea", "Purity", "Enlightenment",
    "Forethought", "Flash of Steel", "HandOfGreed", "Mayhem", "Apotheosis", "Secret Weapon",
    "Panache", "Violence", "Deep Breath", "Secret Technique", "Blind", "The Bomb",
    "Impatience", "Dramatic Entrance", "Trip", "PanicButton", "Sadistic Nature", "Dark Shackles",
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
            // FreeAttackPower stacks and consumes one charge per Attack.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/FreeAttackPower.java
            engine.state.player.add_status(sid::NEXT_ATTACK_FREE, 1);
        }
        BoolFlag::Blasphemy => {
            engine.state.blasphemy_active = true;
        }
        BoolFlag::BulletTime => {
            engine.apply_bullet_time();
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
        assert!(is_debuff(sid::WEAKENED, 1));
        assert!(is_debuff(sid::VULNERABLE, 1));
        assert!(is_debuff(sid::FRAIL, 1));
        assert!(is_debuff(sid::POISON, 1));
        assert!(is_debuff(sid::CORPSE_EXPLOSION, 1));
        assert!(is_debuff(sid::LOSE_STRENGTH, 1));
        assert!(is_debuff(sid::FOCUS, -3));
        assert!(!is_debuff(sid::FOCUS, 3));
        assert!(!is_debuff(sid::STRENGTH, 1));
        assert!(!is_debuff(sid::VIGOR, 1));
    }
}

#[cfg(test)]
#[path = "../tests/test_generated_choice_java_wave3.rs"]
mod test_generated_choice_java_wave3;
