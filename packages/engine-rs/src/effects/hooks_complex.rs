//! on_play hooks for complex card effects — choices, branching, multi-step operations.
//!
//! Each hook has the signature:
//!   `pub fn hook_<tag>(engine: &mut CombatEngine, ctx: &CardPlayContext)`
//!
//! Logic is copied verbatim from card_effects.rs to preserve exact parity.

use crate::cards::{CardTarget, CardType};
use crate::combat_types::CardInstance;
use crate::damage;
use crate::engine::{CombatEngine, ChoiceOption, ChoiceReason};
use crate::state::Stance;
use crate::status_ids::sid;

use super::types::CardPlayContext;

fn hand_card_options(engine: &CombatEngine) -> Vec<ChoiceOption> {
    (0..engine.state.hand.len())
        .map(ChoiceOption::HandCard)
        .collect()
}

fn discard_card_options(engine: &CombatEngine) -> Vec<ChoiceOption> {
    (0..engine.state.discard_pile.len())
        .map(ChoiceOption::DiscardCard)
        .collect()
}

fn draw_card_options(engine: &CombatEngine) -> Vec<ChoiceOption> {
    (0..engine.state.draw_pile.len())
        .map(ChoiceOption::DrawCard)
        .collect()
}

// =========================================================================
// Stance-branching effects
// =========================================================================

/// Inner Peace: if in Calm, draw N; else enter Calm.
pub fn hook_if_calm_draw_else_calm(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if engine.state.stance == Stance::Calm {
        engine.draw_cards(ctx.card.base_magic);
    } else {
        engine.change_stance(Stance::Calm);
    }
}

/// Fear No Evil: enter Calm if target enemy is attacking.
pub fn hook_calm_if_enemy_attacking(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        if engine.state.enemies[ctx.target_idx as usize].is_attacking() {
            engine.change_stance(Stance::Calm);
        }
    }
}

/// Indignation: if in Wrath, apply Vuln to all; else enter Wrath.
pub fn hook_indignation(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if engine.state.stance == Stance::Wrath {
        let vuln_amount = ctx.card.base_magic.max(1);
        let living = engine.state.living_enemy_indices();
        for idx in living {
            engine.apply_player_debuff_to_enemy(idx, sid::VULNERABLE, vuln_amount);
        }
    } else {
        engine.change_stance(Stance::Wrath);
    }
}

// =========================================================================
// Pressure Points / Judgement / Lesson Learned
// =========================================================================

/// Pressure Points: apply Mark to target, then deal HP loss equal to Mark to all marked enemies.
pub fn hook_pressure_points(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let mark_amount = ctx.card.base_magic.max(1);
        engine.state.enemies[ctx.target_idx as usize]
            .entity
            .add_status(sid::MARK, mark_amount);
    }
    let living = engine.state.living_enemy_indices();
    for idx in living {
        let mark = engine.state.enemies[idx].entity.status(sid::MARK);
        if mark > 0 {
            // Pressure Points deals HP loss equal to Mark -- bypasses block entirely
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

/// Judgement: if enemy HP <= threshold, deal their remaining HP as damage (instakill).
pub fn hook_judgement(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        let threshold = ctx.card.base_magic.max(1);
        if engine.state.enemies[tidx].entity.hp <= threshold {
            let hp = engine.state.enemies[tidx].entity.hp;
            // Route through deal_damage_to_enemy so boss hooks fire
            engine.deal_damage_to_enemy(tidx, hp + engine.state.enemies[tidx].entity.block);
        }
    }
}

/// Lesson Learned: if enemy killed, upgrade a random card in draw pile (or discard).
pub fn hook_lesson_learned(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    // Note: enemy_killed is tracked in the main card_effects path.
    // This hook is called after the damage section, so we check if target died.
    // The caller must only invoke this when enemy_killed is true.
    let mut upgraded = false;
    for c in engine.state.draw_pile.iter_mut() {
        if !c.is_upgraded() {
            let name = engine.card_registry.card_name(c.def_id);
            if !name.starts_with("Strike") && !name.starts_with("Defend") {
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
                if !name.starts_with("Strike") && !name.starts_with("Defend") {
                    engine.card_registry.upgrade_card(c);
                    break;
                }
            }
        }
    }
}

// =========================================================================
// Choice-based effects (trigger AwaitingChoice)
// =========================================================================

/// Meditate: choose cards from discard to return to hand (retained). Ends turn.
pub fn hook_meditate(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1) as usize;
    if !engine.state.discard_pile.is_empty() {
        let options: Vec<_> = engine.state.discard_pile.iter()
            .enumerate()
            .map(|(i, _)| ChoiceOption::DiscardCard(i))
            .collect();
        let max_picks = count.min(options.len());
        engine.begin_choice(ChoiceReason::ReturnFromDiscard, options, 1, max_picks);
    }
}

/// Discovery: choose 1 of 3 generated cards to add to hand.
pub fn hook_discovery(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if engine.state.hand.len() < 10 {
        let options = vec![
            ChoiceOption::GeneratedCard(engine.temp_card("Smite")),
            ChoiceOption::GeneratedCard(engine.temp_card("Safety")),
            ChoiceOption::GeneratedCard(engine.temp_card("Insight")),
        ];
        engine.begin_choice(
            ChoiceReason::DiscoverCard,
            options,
            1,
            1,
        );
    }
}

/// Foreign Influence: choose 1 of 3 generated attack cards.
pub fn hook_foreign_influence(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if engine.state.hand.len() < 10 {
        let options = vec![
            ChoiceOption::GeneratedCard(engine.temp_card("Smite")),
            ChoiceOption::GeneratedCard(engine.temp_card("Flying Sleeves")),
            ChoiceOption::GeneratedCard(engine.temp_card("Iron Wave")),
        ];
        engine.begin_choice(
            ChoiceReason::DiscoverCard,
            options,
            1,
            1,
        );
    }
}

/// Omniscience: player picks a card from draw pile to play for free.
pub fn hook_omniscience(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.draw_pile.is_empty() {
        let options: Vec<ChoiceOption> = (0..engine.state.draw_pile.len())
            .map(|i| ChoiceOption::DrawCard(i))
            .collect();
        engine.begin_choice(
            ChoiceReason::PlayCardFreeFromDraw,
            options,
            1,
            1,
        );
    }
}

/// Wish: player picks from Strength / Gold / Plated Armor.
pub fn hook_wish(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let options = vec![
        ChoiceOption::Named("Strength"),
        ChoiceOption::Named("Gold"),
        ChoiceOption::Named("Plated Armor"),
    ];
    engine.begin_choice(
        ChoiceReason::PickOption,
        options,
        1,
        1,
    );
}

/// Seek: player picks card(s) from draw pile to add to hand.
pub fn hook_seek(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1) as usize;
    if !engine.state.draw_pile.is_empty() {
        let options = draw_card_options(engine);
        engine.begin_choice(
            ChoiceReason::PickFromDrawPile,
            options,
            1,
            count.min(engine.state.draw_pile.len()),
        );
    }
}

/// Discard: force player to discard 1 card from hand.
pub fn hook_discard(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.hand.is_empty() {
        let options = hand_card_options(engine);
        engine.begin_choice(
            ChoiceReason::DiscardFromHand,
            options,
            1,
            1,
        );
    }
}

/// Headbutt: player picks a card from discard to put on top of draw pile.
pub fn hook_discard_to_top_of_draw(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.discard_pile.is_empty() {
        let options = discard_card_options(engine);
        engine.begin_choice(
            ChoiceReason::PickFromDiscard,
            options,
            1,
            1,
        );
    }
}

/// Warcry: player picks a card from hand to put on top of draw pile.
pub fn hook_put_card_on_top(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.hand.is_empty() {
        let options: Vec<ChoiceOption> = (0..engine.state.hand.len())
            .map(|i| ChoiceOption::HandCard(i))
            .collect();
        engine.begin_choice(
            ChoiceReason::PutOnTopFromHand,
            options,
            1,
            1,
        );
    }
}

/// True Grit: player chooses 1 card from hand to exhaust.
pub fn hook_exhaust_choose(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.hand.is_empty() {
        let options = hand_card_options(engine);
        engine.begin_choice(
            ChoiceReason::ExhaustFromHand,
            options,
            1,
            1,
        );
    }
}

/// Exhume: pick card from exhaust pile to return to hand.
pub fn hook_exhume(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.exhaust_pile.is_empty() {
        let options: Vec<ChoiceOption> = (0..engine.state.exhaust_pile.len())
            .map(|i| ChoiceOption::ExhaustCard(i))
            .collect();
        engine.begin_choice(
            ChoiceReason::PickFromExhaust,
            options,
            1,
            1,
        );
    }
}

/// Armaments: upgrade one card in hand (choice).
pub fn hook_upgrade_one_card(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let upgradeable: Vec<usize> = engine.state.hand.iter()
        .enumerate()
        .filter(|(_, c)| !c.is_upgraded())
        .map(|(i, _)| i)
        .collect();
    if !upgradeable.is_empty() {
        let options: Vec<ChoiceOption> = upgradeable.iter()
            .map(|&i| ChoiceOption::HandCard(i))
            .collect();
        engine.begin_choice(
            ChoiceReason::UpgradeCard,
            options,
            1,
            1,
        );
    }
}

/// Secret Weapon: search draw pile for an Attack card to add to hand.
pub fn hook_search_attack(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let options: Vec<_> = engine.state.draw_pile.iter()
        .enumerate()
        .filter(|(_, c)| {
            engine.card_registry.card_def_by_id(c.def_id).card_type == CardType::Attack
        })
        .map(|(i, _)| ChoiceOption::DrawCard(i))
        .collect();
    engine.begin_choice(ChoiceReason::SearchDrawPile, options, 1, 1);
}

/// Hologram: return card from discard to hand.
pub fn hook_return_from_discard(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let options: Vec<_> = engine.state.discard_pile.iter()
        .enumerate()
        .map(|(i, _)| ChoiceOption::DiscardCard(i))
        .collect();
    engine.begin_choice(ChoiceReason::ReturnFromDiscard, options, 1, 1);
}

/// Recycle: exhaust 1 card from hand, gain its cost as energy.
pub fn hook_recycle(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let options: Vec<_> = engine.state.hand.iter()
        .enumerate()
        .map(|(i, _)| ChoiceOption::HandCard(i))
        .collect();
    engine.begin_choice(ChoiceReason::RecycleCard, options, 1, 1);
}

/// Concentrate: discard N cards, gain energy.
pub fn hook_discard_gain_energy(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let discard_count = ctx.card.base_magic.max(1) as usize;
    let options: Vec<_> = engine.state.hand.iter()
        .enumerate()
        .map(|(i, _)| ChoiceOption::HandCard(i))
        .collect();
    let actual_picks = discard_count.min(options.len());
    engine.begin_choice(ChoiceReason::DiscardForEffect, options, actual_picks, actual_picks);
}

/// Setup: pick card from hand, set cost 0, put on top of draw.
pub fn hook_setup(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let options: Vec<_> = engine.state.hand.iter()
        .enumerate()
        .map(|(i, _)| ChoiceOption::HandCard(i))
        .collect();
    engine.begin_choice(ChoiceReason::SetupPick, options, 1, 1);
}

/// Thinking Ahead: draw 2, then put 1 card on top of draw.
pub fn hook_thinking_ahead(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.draw_cards(2);
    let options: Vec<_> = engine.state.hand.iter()
        .enumerate()
        .map(|(i, _)| ChoiceOption::HandCard(i))
        .collect();
    engine.begin_choice(ChoiceReason::PutOnTopFromHand, options, 1, 1);
}

// =========================================================================
// X-cost effects
// =========================================================================

/// Doppelganger: next turn draw X + gain X energy (X-cost).
pub fn hook_doppelganger(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.x_value + ctx.card.base_magic.max(0);
    if amount > 0 {
        engine.state.player.add_status(sid::DOPPELGANGER_DRAW, amount);
        engine.state.player.add_status(sid::DOPPELGANGER_ENERGY, amount);
    }
}

// =========================================================================
// Stat manipulation
// =========================================================================

/// Reprogram: lose Focus, gain Strength + Dexterity.
pub fn hook_reprogram(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.card.base_magic.max(1);
    engine.state.player.add_status(sid::FOCUS, -amount);
    engine.state.player.add_status(sid::STRENGTH, amount);
    engine.state.player.add_status(sid::DEXTERITY, amount);
}

// =========================================================================
// Pile manipulation (non-choice)
// =========================================================================

/// Reboot: shuffle hand+discard into draw, draw N cards.
pub fn hook_reboot(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let draw_count = ctx.card.base_magic.max(4);
    let hand_cards: Vec<CardInstance> = engine.state.hand.drain(..).collect();
    engine.state.draw_pile.extend(hand_cards);
    let discard_cards: Vec<CardInstance> = engine.state.discard_pile.drain(..).collect();
    engine.state.draw_pile.extend(discard_cards);
    engine.draw_cards(draw_count);
}

/// All For One: return zero-cost cards from discard to hand.
pub fn hook_return_zero_cost_from_discard(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
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

// =========================================================================
// Conditional combat effects
// =========================================================================

/// Spot Weakness: if target enemy intending Attack, gain Strength.
pub fn hook_spot_weakness(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        if engine.state.enemies[tidx].is_attacking() {
            let amount = ctx.card.base_magic.max(1);
            engine.state.player.add_status(sid::STRENGTH, amount);
        }
    }
}

// =========================================================================
// Exhaust-all / exhaust-subset effects
// =========================================================================

/// Exhaust non-attacks from hand (e.g. Warcry variant).
pub fn hook_exhaust_non_attacks(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
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

/// Discard non-attacks from hand.
pub fn hook_discard_non_attacks(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
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

// =========================================================================
// Cost manipulation
// =========================================================================

/// Enlightenment: reduce hand card costs to 1 this turn.
pub fn hook_enlightenment(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    for hand_card in &mut engine.state.hand {
        let def = engine.card_registry.card_def_by_id(hand_card.def_id);
        if def.cost > 1 {
            hand_card.cost = 1;
        }
    }
}

// =========================================================================
// Upgrade effects
// =========================================================================

/// Apotheosis: upgrade all cards in hand.
pub fn hook_upgrade_all_cards(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    for hand_card in &mut engine.state.hand {
        if !hand_card.is_upgraded() {
            engine.card_registry.upgrade_card(hand_card);
        }
    }
}

// =========================================================================
// Draw/search effects (non-choice)
// =========================================================================

/// Violence: draw attacks from draw pile.
pub fn hook_draw_attacks_from_draw(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1) as usize;
    for _ in 0..count {
        if engine.state.hand.len() >= 10 {
            break;
        }
        let eligible: Vec<usize> = engine
            .state
            .draw_pile
            .iter()
            .enumerate()
            .filter(|(_, card)| {
                engine.card_registry.card_def_by_id(card.def_id).card_type == CardType::Attack
            })
            .map(|(i, _)| i)
            .collect();
        if eligible.is_empty() {
            break;
        }
        let draw_idx = eligible[engine.rng_gen_range(0..eligible.len())];
        let card = engine.state.draw_pile.remove(draw_idx);
        engine.state.hand.push(card);
    }
}

// =========================================================================
// Havoc (play top card)
// =========================================================================

/// Havoc: play top card of draw pile for free.
pub fn hook_play_top_card(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
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
        crate::card_effects::execute_card_effects(engine, &def, top, target);
        engine.state.discard_pile.push(top);
    }
}

// =========================================================================
// Conditional damage effects
// =========================================================================

/// Bowling Bash: damage per living enemy (extra hits beyond the first).
pub fn hook_damage_per_enemy(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let living_count = engine.state.living_enemy_indices().len() as i32;
    if living_count > 1 && ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        let base_damage = engine.player_attack_base_damage(ctx.card, ctx.card_inst);
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let weak_pc = engine.state.has_relic("Paper Crane");
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
        let vuln_pf = engine.state.has_relic("Paper Frog");
        let has_flight = engine.state.enemies[tidx].entity.status(sid::FLIGHT) > 0;
        // Vigor and Pen Nib already consumed on first hit -- don't apply again
        let dmg = damage::calculate_damage_full(
            base_damage,
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
            engine.deal_player_attack_hit_to_enemy(tidx, dmg);
        }
    }
}

/// Bane: double damage if target is poisoned (deal base damage again).
pub fn hook_double_if_poisoned(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        if engine.state.enemies[tidx].entity.status(sid::POISON) > 0 {
            let base_damage = engine.player_attack_base_damage(ctx.card, ctx.card_inst);
            // Deal base damage again (already dealt once in main damage section)
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
            let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
            let dmg = damage::calculate_damage(
                base_damage, player_strength + ctx.vigor, player_weak,
                stance_mult, enemy_vuln, enemy_intangible,
            );
            engine.deal_player_attack_hit_to_enemy(tidx, dmg);
        }
    }
}

/// Finisher: damage per attack played this turn (extra hits beyond the first).
pub fn hook_finisher(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let attacks = engine.state.attacks_played_this_turn;
    if attacks > 1 && ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        let base_damage = engine.player_attack_base_damage(ctx.card, ctx.card_inst);
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
        let dmg = damage::calculate_damage(
            base_damage, player_strength + ctx.vigor, player_weak,
            stance_mult, enemy_vuln, enemy_intangible,
        );
        // Already dealt 1 hit in main damage; deal (attacks - 1) more
        for _ in 0..(attacks - 1) {
            if engine.state.enemies[tidx].entity.is_dead() { break; }
            engine.deal_player_attack_hit_to_enemy(tidx, dmg);
        }
    }
}

/// Flechettes: damage per Skill in hand.
pub fn hook_flechettes(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let skill_count = engine.state.hand.iter()
        .filter(|c| engine.card_registry.card_def_by_id(c.def_id).card_type == CardType::Skill)
        .count() as i32;
    if skill_count > 0 && ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        let base_damage = engine.player_attack_base_damage(ctx.card, ctx.card_inst);
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
        let dmg = damage::calculate_damage(
            base_damage, player_strength + ctx.vigor, player_weak,
            stance_mult, enemy_vuln, enemy_intangible,
        );
        for _ in 0..skill_count {
            if engine.state.enemies[tidx].entity.is_dead() { break; }
            engine.deal_player_attack_hit_to_enemy(tidx, dmg);
        }
    }
}

// =========================================================================
// Random multi-hit damage (Sword Boomerang, Rip and Tear)
// =========================================================================

/// Sword Boomerang / Rip and Tear: deal base_damage to random enemies base_magic times.
/// The generic damage loop is skipped (damage_random_x_times sets skip_generic_damage).
pub fn hook_damage_random_hits(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let hits = ctx.card.base_magic.max(1);
    let base_damage = engine.player_attack_base_damage(ctx.card, ctx.card_inst);
    let player_strength = engine.state.player.strength();
    let player_weak = engine.state.player.is_weak();
    let weak_pc = engine.state.has_relic("Paper Crane");
    let stance_mult = engine.state.stance.outgoing_mult();
    let double_damage = engine.state.player.status(sid::DOUBLE_DAMAGE) > 0;
    if double_damage {
        let dd = engine.state.player.status(sid::DOUBLE_DAMAGE);
        engine.state.player.set_status(sid::DOUBLE_DAMAGE, dd - 1);
    }

    for i in 0..hits {
        let living = engine.state.living_enemy_indices();
        if living.is_empty() { break; }
        let idx = living[engine.rng_gen_range(0..living.len())];
        let enemy_vuln = engine.state.enemies[idx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[idx].entity.status(sid::INTANGIBLE) > 0;
        let vuln_pf = engine.state.has_relic("Paper Frog");
        let dmg = damage::calculate_damage_full(
            base_damage,
            player_strength,
            if i == 0 { ctx.vigor } else { 0 },
            player_weak,
            weak_pc,
            if i == 0 { ctx.pen_nib_active } else { false },
            double_damage,
            stance_mult,
            enemy_vuln,
            vuln_pf,
            false, // flight
            enemy_intangible,
        );
        engine.deal_player_attack_hit_to_enemy(idx, dmg);
    }
}

// =========================================================================
// Feed: gain max HP on kill
// =========================================================================

/// Feed: if an enemy was killed during the damage loop, increase max HP and heal.
pub fn hook_feed(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.enemy_killed {
        let amount = ctx.card.base_magic.max(1);
        engine.state.player.max_hp += amount;
        engine.state.player.hp = (engine.state.player.hp + amount).min(engine.state.player.max_hp);
    }
}

// =========================================================================
// Reaper: heal for unblocked damage
// =========================================================================

/// Reaper: heal player for total unblocked damage dealt to all enemies.
pub fn hook_reaper(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.total_unblocked_damage > 0 {
        engine.heal_player(ctx.total_unblocked_damage);
    }
}

// =========================================================================
// Escape Plan: draw 1, if skill gain block
// =========================================================================

/// Escape Plan: draw 1 card, if the drawn card is a Skill, gain block.
pub fn hook_escape_plan(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let hand_before = engine.state.hand.len();
    engine.draw_cards(1);
    // Check if a card was actually drawn
    if engine.state.hand.len() > hand_before {
        let drawn_card = &engine.state.hand[engine.state.hand.len() - 1];
        let def = engine.card_registry.card_def_by_id(drawn_card.def_id);
        if def.card_type == CardType::Skill {
            let dex = engine.state.player.dexterity();
            let frail = engine.state.player.is_frail();
            let block = damage::calculate_block(ctx.card.base_block, dex, frail);
            engine.gain_block_player(block);
        }
    }
}

// =========================================================================
// Malaise: apply X weak + reduce X strength
// =========================================================================

/// Malaise: apply (X + base_magic) Weak and reduce (X + base_magic) Strength to target enemy.
pub fn hook_malaise(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.x_value + ctx.card.base_magic.max(0);
    if amount > 0 && ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        engine.apply_player_debuff_to_enemy(tidx, sid::WEAKENED, amount);
        engine.state.enemies[tidx].entity.add_status(sid::STRENGTH, -amount);
    }
}

// =========================================================================
// Wraith Form: gain Intangible + set Wraith Form power
// =========================================================================

/// Wraith Form: gain base_magic Intangible, set Wraith Form power to 1
/// (Wraith Form power causes -1 Dex per turn via the power system).
pub fn hook_wraith_form(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.card.base_magic.max(1);
    engine.state.player.add_status(sid::INTANGIBLE, amount);
    engine.state.player.set_status(sid::WRAITH_FORM, 1);
}

// =========================================================================
// Doppelganger: set next-turn draw/energy bonuses
// =========================================================================

/// Doppelganger: set draw and energy bonuses for next turn (X + base_magic).
pub fn hook_doppelganger_set_bonuses(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.x_value + ctx.card.base_magic.max(0);
    if amount > 0 {
        engine.state.player.add_status(sid::DOPPELGANGER_DRAW, amount);
        engine.state.player.add_status(sid::DOPPELGANGER_ENERGY, amount);
    }
}

// =========================================================================
// Defect: Thunder Strike — deal base_damage per lightning channeled to random enemies
// =========================================================================

/// Thunder Strike: deal base_damage to a random enemy for each Lightning channeled.
pub fn hook_thunder_strike(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let lightning_count = engine.state.player.status(sid::LIGHTNING_CHANNELED);
    if lightning_count <= 0 { return; }
    let base_damage = engine.player_attack_base_damage(ctx.card, ctx.card_inst);
    let player_strength = engine.state.player.strength();
    let player_weak = engine.state.player.is_weak();
    let stance_mult = engine.state.stance.outgoing_mult();
    for _ in 0..lightning_count {
        let living = engine.state.living_enemy_indices();
        if living.is_empty() { break; }
        let idx = living[engine.rng_gen_range(0..living.len())];
        let enemy_vuln = engine.state.enemies[idx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[idx].entity.status(sid::INTANGIBLE) > 0;
        let dmg = damage::calculate_damage(
            base_damage, player_strength, player_weak,
            stance_mult, enemy_vuln, enemy_intangible,
        );
        engine.deal_player_attack_hit_to_enemy(idx, dmg);
    }
}

// Defect: Recursion (Redo) — evoke front orb, channel same type
// =========================================================================

/// Recursion: evoke the front orb, then channel a new orb of the same type.
pub fn hook_recursion(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    // Peek at front orb type before evoking
    let front_type = engine.state.orb_slots.front_orb_type();
    if front_type == crate::orbs::OrbType::Empty {
        return;
    }
    // Evoke the front orb
    engine.evoke_front_orb();
    // Channel a new orb of the same type
    engine.channel_orb(front_type);
}

// =========================================================================
// Defect: Claw — increment CLAW_BONUS status (all Claws gain +2 damage)
// =========================================================================

/// Claw: after dealing damage, add base_magic (2) to CLAW_BONUS for future Claws.
pub fn hook_claw(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let bonus = ctx.card.base_magic.max(2);
    engine.state.player.add_status(sid::CLAW_BONUS, bonus);
}

// =========================================================================
// Defect: Chaos upgrade — channel N random orbs
// =========================================================================

/// Chaos: channel base_magic random orbs (base=1, upgrade=2).
pub fn hook_chaos(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1);
    let orb_types = [crate::orbs::OrbType::Lightning, crate::orbs::OrbType::Frost,
                     crate::orbs::OrbType::Dark, crate::orbs::OrbType::Plasma];
    for _ in 0..count {
        let idx = engine.rng_gen_range(0..orb_types.len());
        engine.channel_orb(orb_types[idx]);
    }
}

// =========================================================================
// Colorless: Mind Blast — damage equal to draw pile size
// =========================================================================

/// Mind Blast: deal flat damage equal to draw pile size to target enemy.
/// The preamble base_damage is 0, so we handle all damage here.
pub fn hook_mind_blast(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let draw_pile_size = engine.state.draw_pile.len() as i32;
    if draw_pile_size > 0 && ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
        let dmg = damage::calculate_damage(
            draw_pile_size, player_strength, player_weak,
            stance_mult, enemy_vuln, enemy_intangible,
        );
        engine.deal_player_attack_hit_to_enemy(tidx, dmg);
    }
}

// =========================================================================
// Colorless: Enlightenment+ — permanently set all hand card costs to 1
// =========================================================================

/// Enlightenment+: permanently reduce hand card costs to 1.
/// Unlike base Enlightenment (this-turn only), the cost stays at 1.
pub fn hook_enlightenment_permanent(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    for hand_card in &mut engine.state.hand {
        let def = engine.card_registry.card_def_by_id(hand_card.def_id);
        if def.cost > 1 {
            hand_card.set_permanent_cost(1);
        }
    }
}

// =========================================================================
// Colorless: Chrysalis — add Deflect cards (MCTS approx for random Skills)
// =========================================================================

/// Chrysalis: add base_magic Deflect cards (cost 0) to draw pile and shuffle.
pub fn hook_chrysalis(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1);
    for _ in 0..count {
        let card = engine.temp_card("Deflect");
        engine.state.draw_pile.push(card);
    }
    engine.shuffle_draw_pile();
}

// =========================================================================
// Colorless: Metamorphosis — add Strike cards (MCTS approx for random Attacks)
// =========================================================================

/// Metamorphosis: add base_magic Strike temp cards (cost 0) to draw pile and shuffle.
pub fn hook_metamorphosis(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1);
    for _ in 0..count {
        let card = engine.temp_card("Smite");
        engine.state.draw_pile.push(card);
    }
    engine.shuffle_draw_pile();
}

// =========================================================================
// Colorless: Jack of All Trades — add Finesse to hand (MCTS approx)
// =========================================================================

/// Jack of All Trades: add base_magic Finesse cards to hand.
pub fn hook_jack_of_all_trades(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1);
    for _ in 0..count {
        if engine.state.hand.len() >= 10 { break; }
        let card = engine.temp_card("Finesse");
        engine.state.hand.push(card);
    }
}

// =========================================================================
// All-Out Attack: discard 1 random card from hand
// =========================================================================

/// All-Out Attack: discard 1 random card from hand.
pub fn hook_discard_random(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.hand.is_empty() {
        let idx = engine.rng_gen_range(0..engine.state.hand.len());
        let card = engine.state.hand.remove(idx);
        engine.state.discard_pile.push(card);
        engine.on_card_discarded(card);
    }
}

// =========================================================================
// Calculated Gamble: discard entire hand, draw that many
// =========================================================================

/// Calculated Gamble: discard entire hand, draw that many cards.
pub fn hook_calculated_gamble(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let hand_count = engine.state.hand.len() as i32;
    let cards: Vec<CardInstance> = engine.state.hand.drain(..).collect();
    for card in cards {
        engine.state.discard_pile.push(card);
        engine.on_card_discarded(card);
    }
    let draw_count = if _ctx.card.id.ends_with('+') {
        hand_count + 1
    } else {
        hand_count
    };
    if draw_count > 0 {
        engine.draw_cards(draw_count);
    }
}

// =========================================================================
// Expertise: draw until hand has N cards
// =========================================================================

/// Expertise: draw until hand has base_magic cards.
pub fn hook_expertise(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let target_size = ctx.card.base_magic.max(1);
    let to_draw = (target_size - engine.state.hand.len() as i32).max(0);
    if to_draw > 0 {
        engine.draw_cards(to_draw);
    }
}

// =========================================================================
// Bouncing Flask: apply 3 Poison to random enemies N times
// =========================================================================

/// Bouncing Flask: apply 3 Poison to a random enemy, repeated base_magic times.
pub fn hook_bouncing_flask(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let bounces = ctx.card.base_magic.max(1);
    let poison_per_bounce = 3;
    for _ in 0..bounces {
        let living = engine.state.living_enemy_indices();
        if living.is_empty() { break; }
        let idx = living[engine.rng_gen_range(0..living.len())];
        engine.apply_player_debuff_to_enemy(idx, sid::POISON, poison_per_bounce);
    }
}

// =========================================================================
// Storm of Steel: discard hand, add 1 Shiv per card discarded
// =========================================================================

/// Storm of Steel: discard entire hand, add 1 Shiv per card discarded.
/// Upgraded version adds Shiv+ instead of Shiv.
pub fn hook_storm_of_steel(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let hand_count = engine.state.hand.len();
    let cards: Vec<CardInstance> = engine.state.hand.drain(..).collect();
    for card in cards {
        engine.state.discard_pile.push(card);
        engine.on_card_discarded(card);
    }
    let shiv_name = if ctx.card.id.ends_with('+') { "Shiv+" } else { "Shiv" };
    for _ in 0..hand_count {
        if engine.state.hand.len() >= 10 { break; }
        let shiv = engine.temp_card(shiv_name);
        engine.state.hand.push(shiv);
    }
}

// =========================================================================
// Nightmare: choose a card in hand, add N copies to hand (MCTS simplified)
// =========================================================================

/// Nightmare: choose a card in hand, add base_magic copies to hand.
/// MCTS simplification: adds copies immediately (real game adds next turn).
pub fn hook_nightmare(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let copies = ctx.card.base_magic.max(1) as usize;
    if !engine.state.hand.is_empty() {
        let options = hand_card_options(engine);
        engine.begin_choice_with_aux(ChoiceReason::DualWield, options, 1, 1, copies);
    }
}

// =========================================================================
// Distraction: add random Skill to hand at cost 0 (MCTS approx)
// =========================================================================

/// Distraction: MCTS approximation -- add Backflip at cost 0 to hand.
pub fn hook_distraction(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if engine.state.hand.len() < 10 {
        let mut card = engine.temp_card("Backflip");
        card.cost = 0;
        engine.state.hand.push(card);
    }
}

// =========================================================================
// Defect: Streamline — reduce cost of all copies by 1 each play
// =========================================================================

fn with_runtime_played_card_mut<F>(engine: &mut CombatEngine, f: F)
where
    F: FnOnce(&mut CardInstance),
{
    if let Some(mut card) = engine.runtime_played_card {
        f(&mut card);
        engine.runtime_played_card = Some(card);
    }
}

/// Streamline: after playing, reduce the instance cost of the played copy by 1.
///
/// Java targets the played card by UUID. We model that with the current played
/// instance state that gets written back after the effect pipeline resolves.
pub fn hook_streamline(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let current_cost = if ctx.card_inst.cost >= 0 {
        ctx.card_inst.cost as i32
    } else {
        ctx.card.cost
    };
    with_runtime_played_card_mut(engine, |card| {
        card.set_permanent_cost((current_cost - 1).max(0) as i8);
    });
}

/// Steam Barrier: reduce the played instance's current block by 1.
pub fn hook_steam_barrier(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let current_block = if ctx.card_inst.misc >= 0 {
        ctx.card_inst.misc as i32
    } else {
        ctx.card.base_block
    };
    with_runtime_played_card_mut(engine, |card| {
        card.misc = (current_block - 1).max(0) as i16;
    });
}

/// Genetic Algorithm: increase the played instance's current block by misc.
pub fn hook_genetic_algorithm(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let current_block = if ctx.card_inst.misc >= 0 {
        ctx.card_inst.misc as i32
    } else {
        ctx.card.base_block
    };
    with_runtime_played_card_mut(engine, |card| {
        card.misc = (current_block + ctx.card.base_magic.max(1)) as i16;
    });
}

/// Rampage: increase the played instance's current damage by misc.
pub fn hook_rampage(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let current_damage = if ctx.card_inst.misc >= 0 {
        ctx.card_inst.misc as i32
    } else {
        ctx.card.base_damage
    };
    with_runtime_played_card_mut(engine, |card| {
        card.misc = (current_damage + ctx.card.base_magic.max(1)) as i16;
    });
}

/// Glass Knife: reduce the played instance's current damage by 2.
pub fn hook_glass_knife(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let current_damage = if ctx.card_inst.misc >= 0 {
        ctx.card_inst.misc as i32
    } else {
        ctx.card.base_damage
    };
    with_runtime_played_card_mut(engine, |card| {
        card.misc = (current_damage - 2).max(0) as i16;
    });
}
