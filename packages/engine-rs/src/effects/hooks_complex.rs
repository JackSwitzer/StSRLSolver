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
use crate::powers;
use crate::state::Stance;
use crate::status_ids::sid;

use super::types::CardPlayContext;

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
        let options: Vec<ChoiceOption> = (0..engine.state.draw_pile.len())
            .map(|i| ChoiceOption::DrawCard(i))
            .collect();
        engine.begin_choice(
            ChoiceReason::PickFromDrawPile,
            options,
            1,
            count,
        );
    }
}

/// Discard: force player to discard 1 card from hand.
pub fn hook_discard(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.hand.is_empty() {
        let options: Vec<ChoiceOption> = (0..engine.state.hand.len())
            .map(|i| ChoiceOption::HandCard(i))
            .collect();
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
        let options: Vec<ChoiceOption> = (0..engine.state.discard_pile.len())
            .map(|i| ChoiceOption::DiscardCard(i))
            .collect();
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
        let options: Vec<ChoiceOption> = (0..engine.state.hand.len())
            .map(|i| ChoiceOption::HandCard(i))
            .collect();
        engine.begin_choice(
            ChoiceReason::ExhaustFromHand,
            options,
            1,
            1,
        );
    }
}

/// Exhaust random: exhaust 1 random card from hand.
pub fn hook_exhaust_random(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
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

/// Dual Wield: copy a card from hand (choice).
pub fn hook_dual_wield(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let copies = ctx.card.base_magic.max(1) as usize;
    if !engine.state.hand.is_empty() && engine.state.hand.len() + copies <= 10 {
        let options: Vec<ChoiceOption> = (0..engine.state.hand.len())
            .map(|i| ChoiceOption::HandCard(i))
            .collect();
        engine.begin_choice(
            ChoiceReason::DualWield,
            options,
            1,
            copies,
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

/// Secret Technique: search draw pile for a Skill card to add to hand.
pub fn hook_search_skill(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let options: Vec<_> = engine.state.draw_pile.iter()
        .enumerate()
        .filter(|(_, c)| {
            engine.card_registry.card_def_by_id(c.def_id).card_type == CardType::Skill
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

/// Forethought: put 1 card from hand to bottom of draw at cost 0.
pub fn hook_forethought(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let options: Vec<_> = engine.state.hand.iter()
        .enumerate()
        .map(|(i, _)| ChoiceOption::HandCard(i))
        .collect();
    engine.begin_choice(ChoiceReason::ForethoughtPick, options, 1, 1);
}

/// Forethought+: put ALL hand cards to bottom of draw at cost 0.
pub fn hook_forethought_all(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    // Auto-resolve: move all hand cards to bottom of draw at cost 0
    let hand_cards: Vec<_> = engine.state.hand.drain(..).collect();
    for mut c in hand_cards {
        c.cost = 0;
        engine.state.draw_pile.push(c);
    }
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

/// Purity: exhaust N from hand.
pub fn hook_exhaust_from_hand(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let exhaust_count = ctx.card.base_magic.max(1) as usize;
    let options: Vec<_> = engine.state.hand.iter()
        .enumerate()
        .map(|(i, _)| ChoiceOption::HandCard(i))
        .collect();
    let actual_picks = exhaust_count.min(options.len());
    engine.begin_choice(ChoiceReason::ExhaustFromHand, options, 0, actual_picks);
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

/// Forethought+: all hand cards to bottom at cost 0 (same as hook_forethought_all).
// Already defined above as hook_forethought_all.

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

/// Fiend Fire: exhaust all hand cards, deal damage per card exhausted.
pub fn hook_fiend_fire(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let hand_count = engine.state.hand.len() as i32;
    // Exhaust all cards from hand
    let exhausted_cards: Vec<CardInstance> = engine.state.hand.drain(..).collect();
    engine.state.exhaust_pile.extend(exhausted_cards);
    // Deal base_damage per card exhausted to the target
    if hand_count > 0 && ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let weak_paper_crane = engine.state.has_relic("Paper Crane");
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
        let vuln_paper_frog = engine.state.has_relic("Paper Frog");
        let dmg = damage::calculate_damage_full(
            ctx.card.base_damage,
            player_strength,
            ctx.vigor,
            player_weak,
            weak_paper_crane,
            ctx.pen_nib_active,
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

/// Second Wind: exhaust all non-attack cards in hand, gain block per exhaust.
pub fn hook_second_wind(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let block_per = ctx.card.base_block.max(5);
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

/// Madness: random card in hand costs 0 this combat.
pub fn hook_madness(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
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
            ctx.card.base_damage,
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

/// Bane: double damage if target is poisoned (deal base damage again).
pub fn hook_double_if_poisoned(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        if engine.state.enemies[tidx].entity.status(sid::POISON) > 0 {
            // Deal base damage again (already dealt once in main damage section)
            let player_strength = engine.state.player.strength();
            let player_weak = engine.state.player.is_weak();
            let stance_mult = engine.state.stance.outgoing_mult();
            let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
            let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
            let dmg = damage::calculate_damage(
                ctx.card.base_damage, player_strength + ctx.vigor, player_weak,
                stance_mult, enemy_vuln, enemy_intangible,
            );
            engine.deal_damage_to_enemy(tidx, dmg);
        }
    }
}

/// Finisher: damage per attack played this turn (extra hits beyond the first).
pub fn hook_finisher(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let attacks = engine.state.attacks_played_this_turn;
    if attacks > 1 && ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
        let dmg = damage::calculate_damage(
            ctx.card.base_damage, player_strength + ctx.vigor, player_weak,
            stance_mult, enemy_vuln, enemy_intangible,
        );
        // Already dealt 1 hit in main damage; deal (attacks - 1) more
        for _ in 0..(attacks - 1) {
            if engine.state.enemies[tidx].entity.is_dead() { break; }
            engine.deal_damage_to_enemy(tidx, dmg);
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
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
        let dmg = damage::calculate_damage(
            ctx.card.base_damage, player_strength + ctx.vigor, player_weak,
            stance_mult, enemy_vuln, enemy_intangible,
        );
        for _ in 0..skill_count {
            if engine.state.enemies[tidx].entity.is_dead() { break; }
            engine.deal_damage_to_enemy(tidx, dmg);
        }
    }
}
