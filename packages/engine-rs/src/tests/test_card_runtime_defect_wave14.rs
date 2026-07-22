#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Seek.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Chaos.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Fission.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Reboot.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Recursion.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Scrape.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::declarative::{
    AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE,
    Target as T,
};
use crate::engine::{ChoiceReason, CombatPhase};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    discard_prefix_count, enemy_no_intent, engine_without_start, force_player_turn,
    hand_prefix_count, make_deck, play_on_enemy, play_self, TEST_SEED,
};

#[test]
fn defect_wave14_registry_exports_seek_on_the_typed_search_surface() {
    let seek = global_registry().get("Seek").expect("Seek");
    assert_eq!(
        seek.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::MoveToHand,
            min_picks: A::Magic,
            max_picks: A::Magic,
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }]
    );
    assert!(seek.complex_hook.is_none());

    let seek_plus = global_registry().get("Seek+").expect("Seek+");
    assert_eq!(
        seek_plus.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::MoveToHand,
            min_picks: A::Magic,
            max_picks: A::Magic,
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }]
    );
    assert!(seek_plus.complex_hook.is_none());

    let chaos = global_registry().get("Chaos").expect("Chaos");
    assert_eq!(
        chaos.effect_data,
        &[E::Simple(SE::ChannelRandomOrb(A::Magic))]
    );
    assert!(chaos.complex_hook.is_none());

    let consume = global_registry().get("Consume").expect("Consume");
    assert_eq!(
        consume.effect_data,
        &[
            E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Magic)),
            E::Simple(SE::RemoveOrbSlot),
        ]
    );
    assert!(consume.complex_hook.is_none());

    let darkness = global_registry().get("Darkness").expect("Darkness");
    assert_eq!(
        darkness.effect_data,
        &[E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1)))]
    );
    assert!(darkness.complex_hook.is_none());

    let darkness_plus = global_registry().get("Darkness+").expect("Darkness+");
    assert_eq!(
        darkness_plus.effect_data,
        &[
            E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1))),
            E::Simple(SE::TriggerDarkPassive),
        ]
    );
    assert!(darkness_plus.complex_hook.is_none());

    let fission = global_registry().get("Fission").expect("Fission");
    assert_eq!(
        fission.effect_data,
        &[E::Simple(SE::ResolveFission { evoke: false })]
    );
    assert!(fission.complex_hook.is_none());

    let fission_plus = global_registry().get("Fission+").expect("Fission+");
    assert_eq!(
        fission_plus.effect_data,
        &[E::Simple(SE::ResolveFission { evoke: true })]
    );
    assert!(fission_plus.complex_hook.is_none());

    let reboot = global_registry().get("Reboot").expect("Reboot");
    assert_eq!(
        reboot.effect_data,
        &[E::Simple(SE::ShuffleAllAndDraw(A::Magic))]
    );
    assert!(reboot.complex_hook.is_none());

    let redo = global_registry().get("Redo").expect("Redo");
    assert_eq!(
        redo.effect_data,
        &[E::Simple(SE::EvokeAndRechannelFrontOrb)]
    );
    assert!(redo.complex_hook.is_none());

    let scrape = global_registry().get("Scrape").expect("Scrape");
    assert_eq!(
        scrape.effect_data,
        &[E::Simple(SE::DrawCardsThenDiscardDrawnNonZeroCost(
            A::Magic
        ))]
    );
    assert!(scrape.complex_hook.is_none());
}

#[test]
fn seek_plus_searches_the_draw_pile_with_the_declarative_choice_surface() {
    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Strike", "Defend", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.hand = make_deck(&["Seek+"]);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;

    assert!(play_self(&mut engine, "Seek+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(
        engine.choice.as_ref().map(|choice| choice.reason.clone()),
        Some(ChoiceReason::SearchDrawPile),
    );
    assert_eq!(
        engine.choice.as_ref().map(|choice| choice.min_picks),
        Some(2)
    );

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(1));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 2);
    assert_eq!(engine.state.draw_pile.len(), 1);
}

#[test]
fn fission_reboot_and_scrape_follow_the_current_defect_runtime_paths() {
    let mut fission = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 60, 60)], 3);
    force_player_turn(&mut fission);
    fission.init_defect_orbs(3);
    fission.channel_orb(OrbType::Lightning);
    fission.channel_orb(OrbType::Frost);
    fission.channel_orb(OrbType::Dark);
    fission.state.hand = make_deck(&["Fission"]);
    fission.state.draw_pile = make_deck(&["Strike", "Defend", "Zap", "Dualcast"]);

    assert!(play_self(&mut fission, "Fission"));
    assert_eq!(fission.state.orb_slots.occupied_count(), 0);
    assert_eq!(fission.state.energy, 6);
    assert_eq!(fission.state.hand.len(), 3);

    let mut reboot = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut reboot);
    reboot.state.hand = make_deck(&["Reboot", "Strike", "Defend"]);
    reboot.state.draw_pile.clear();
    reboot.state.discard_pile = make_deck(&["Zap", "Dualcast", "Cold Snap"]);

    assert!(play_self(&mut reboot, "Reboot"));
    assert_eq!(reboot.state.hand.len(), 4);
    assert_eq!(reboot.state.exhaust_pile.len(), 1);
    assert_eq!(
        reboot
            .card_registry
            .card_name(reboot.state.exhaust_pile[0].def_id),
        "Reboot"
    );
    assert_eq!(reboot.state.discard_pile.len(), 0);

    let mut scrape = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut scrape);
    scrape.state.hand = make_deck(&["Scrape"]);
    scrape.state.draw_pile = make_deck(&["Turbo", "Strike"]);

    assert!(play_self(&mut scrape, "Scrape"));
    assert!(hand_prefix_count(&scrape, "Turbo") >= 1);
    assert_eq!(discard_prefix_count(&scrape, "Strike"), 1);
}

#[test]
fn consume_uses_the_typed_orb_slot_removal_surface() {
    // Source: Consume.java queues FocusPower(2) before DecreaseMaxOrbAction(1).
    // AbstractPlayer.decreaseMaxOrbSlots removes the last orb without evoking
    // it, so the removed Plasma grants no energy.
    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Consume"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.hand = make_deck(&["Consume"]);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.init_defect_orbs(3);
    engine.channel_orb(OrbType::Lightning);
    engine.channel_orb(OrbType::Frost);
    engine.channel_orb(OrbType::Plasma);

    assert!(play_self(&mut engine, "Consume"));

    assert_eq!(engine.state.player.focus(), 2);
    assert_eq!(engine.state.orb_slots.get_slot_count(), 2);
    assert_eq!(engine.state.orb_slots.occupied_count(), 2);
    assert_eq!(engine.state.energy, 1);
    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    assert_eq!(engine.state.orb_slots.slots[1].orb_type, OrbType::Frost);
}

#[test]
fn darkness_plus_channels_dark_then_triggers_dark_passive() {
    // Darkness.java queues ChannelAction before DarkImpulseAction. The latter
    // pulses every Dark orb, then pulses the front Dark a second time with
    // Cables (DarkImpulseAction.java).
    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Darkness+"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.hand = make_deck(&["Darkness+"]);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.init_defect_orbs(3);
    engine.channel_orb(OrbType::Dark);
    engine.channel_orb(OrbType::Dark);
    engine.state.relics.push("Cables".to_string());

    assert!(play_self(&mut engine, "Darkness+"));

    assert_eq!(engine.state.orb_slots.occupied_count(), 3);
    assert_eq!(engine.state.orb_slots.slots[0].evoke_amount, 18);
    assert_eq!(engine.state.orb_slots.slots[1].evoke_amount, 12);
    assert_eq!(engine.state.orb_slots.slots[2].evoke_amount, 12);
}

#[test]
fn redo_rechannels_the_same_charged_dark_orb_instance() {
    // RedoAction snapshots the front AbstractOrb object, queues an evoke, then
    // passes that same object to ChannelAction(..., false). Dark.onEvoke does
    // not reset evokeAmount, so the re-channeled Dark retains its charge.
    // Sources: Recursion.java, actions/defect/RedoAction.java,
    // actions/defect/ChannelAction.java, and orbs/Dark.java.
    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Redo"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.hand = make_deck(&["Redo"]);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.init_defect_orbs(3);
    engine.channel_orb(OrbType::Dark);
    engine.state.orb_slots.slots[0].evoke_amount = 18;

    assert!(play_self(&mut engine, "Redo"));

    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
    assert_eq!(engine.state.orb_slots.front_orb_type(), OrbType::Dark);
    assert_eq!(engine.state.orb_slots.slots[0].evoke_amount, 18);
    assert_eq!(engine.state.enemies[0].entity.hp, 22);
}

#[test]
fn redo_does_not_rechannel_after_a_lethal_evoke() {
    // Lightning.onEvoke queues damage ahead of RedoAction's ChannelAction.
    // DamageAction clears post-combat non-damage actions after the last enemy
    // dies, and ChannelAction is therefore removed.
    // Sources: actions/defect/RedoAction.java, orbs/Lightning.java,
    // actions/common/DamageAction.java, and actions/GameActionManager.java.
    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Redo"]),
        vec![enemy_no_intent("JawWorm", 8, 8)],
        3,
    );
    state.hand = make_deck(&["Redo"]);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.init_defect_orbs(3);
    engine.channel_orb(OrbType::Lightning);

    assert!(play_self(&mut engine, "Redo"));

    assert!(engine.state.is_victory());
    assert_eq!(engine.state.orb_slots.occupied_count(), 0);
}

#[test]
fn reboot_moves_remaining_hand_and_discard_into_draw_then_draws_and_exhausts() {
    // Reboot.java queues ShuffleAllAction, ShuffleAction(drawPile, false), then
    // DrawCardAction(magicNumber). ShuffleAllAction delegates the hand to
    // PutOnDeckAction rather than DiscardAction: this consumes one cardRandomRng
    // tick per remaining hand card and never fires discard callbacks. Its
    // discard shuffle and Reboot's explicit combined shuffle each consume one
    // shuffleRng randomLong, while relic onShuffle hooks fire only once.
    // Sources: reference/extracted/methods/card/Reboot.java;
    // decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ShuffleAllAction.java;
    // decompiled/java-src/com/megacrit/cardcrawl/actions/common/PutOnDeckAction.java;
    // decompiled/java-src/com/megacrit/cardcrawl/actions/common/ShuffleAction.java;
    // and decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java.
    let mut reboot = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut reboot);
    reboot.state.hand = make_deck(&["Reboot+", "Strike", "Defend"]);
    reboot.state.draw_pile = make_deck(&["Zap"]);
    reboot.state.discard_pile = make_deck(&["Dualcast", "Cold Snap", "Gash"]);
    reboot.clear_event_log();
    let shuffle_before = reboot.shuffle_rng.counter;
    let card_random_before = reboot.card_random_rng.counter;

    assert!(play_self(&mut reboot, "Reboot+"));

    let hand_names: Vec<_> = reboot
        .state
        .hand
        .iter()
        .map(|card| reboot.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(
        hand_names,
        vec!["Defend", "Strike", "Zap", "Cold Snap", "Dualcast", "Gash"]
    );
    assert_eq!(reboot.shuffle_rng.counter, shuffle_before + 2);
    assert_eq!(reboot.card_random_rng.counter, card_random_before + 2);
    assert_eq!(reboot.state.player.status(sid::DISCARDED_THIS_TURN), 0);
    assert_eq!(
        reboot
            .event_log
            .iter()
            .filter(|record| record.event == crate::effects::trigger::Trigger::OnShuffle)
            .count(),
        1
    );
    assert!(reboot.state.draw_pile.is_empty());
    assert_eq!(reboot.state.exhaust_pile.len(), 1);
    assert_eq!(
        reboot
            .card_registry
            .card_name(reboot.state.exhaust_pile[0].def_id),
        "Reboot+"
    );
    assert_eq!(reboot.state.discard_pile.len(), 0);
}

#[test]
fn reboot_waits_for_melange_scry_before_gathering_and_shuffling_piles() {
    // ShuffleAllAction calls relic.onShuffle() in its constructor, before the
    // action itself is added to the queue. Melange.onShuffle queues ScryAction,
    // so that Scry resolves before Reboot moves the hand or shuffles discard.
    // Sources: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ShuffleAllAction.java,
    // decompiled/java-src/com/megacrit/cardcrawl/relics/Melange.java, and
    // reference/extracted/methods/card/Reboot.java.
    let mut reboot = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut reboot);
    reboot.state.relics.push("Melange".to_string());
    reboot.rebuild_effect_runtime();
    reboot.state.hand = make_deck(&["Reboot+", "Strike"]);
    reboot.state.draw_pile = make_deck(&["Zap", "Dualcast", "Cold Snap", "Weave"]);
    reboot.state.discard_pile = make_deck(&["Bash"]);
    let shuffle_before = reboot.shuffle_rng.counter;
    let card_random_before = reboot.card_random_rng.counter;

    assert!(play_self(&mut reboot, "Reboot+"));

    assert_eq!(reboot.phase, CombatPhase::AwaitingChoice);
    assert_eq!(
        reboot.state.hand.len(),
        1,
        "ShuffleAllAction is still queued"
    );
    assert_eq!(
        reboot.state.draw_pile.len(),
        1,
        "Melange exposed three cards"
    );
    assert_eq!(reboot.state.discard_pile.len(), 1);
    assert_eq!(reboot.shuffle_rng.counter, shuffle_before);
    assert_eq!(reboot.card_random_rng.counter, card_random_before);
    assert_eq!(
        reboot
            .choice
            .as_ref()
            .and_then(|choice| choice.deferred_shuffle_all_draw),
        Some(6)
    );

    // Select the exposed Weave. Java queues its DiscardToHandAction after the
    // already-queued Reboot actions, so Reboot shuffles it out of discard first.
    reboot.execute_action(&Action::Choose(2));
    reboot.execute_action(&Action::ConfirmSelection);

    assert_eq!(reboot.phase, CombatPhase::PlayerTurn);
    assert_eq!(reboot.state.hand.len(), 6);
    assert!(reboot.state.draw_pile.is_empty());
    assert!(reboot.state.discard_pile.is_empty());
    assert_eq!(reboot.shuffle_rng.counter, shuffle_before + 2);
    assert_eq!(reboot.card_random_rng.counter, card_random_before + 1);
    assert_eq!(reboot.state.exhaust_pile.len(), 1);
    assert_eq!(
        reboot
            .card_registry
            .card_name(reboot.state.exhaust_pile[0].def_id),
        "Reboot+"
    );
}

#[test]
fn scrape_draws_then_discards_only_newly_drawn_nonzero_cost_cards() {
    let mut scrape = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut scrape);
    scrape.state.hand = make_deck(&["Scrape", "Strike"]);
    scrape.state.draw_pile = make_deck(&["Strike", "Turbo"]);

    assert!(play_self(&mut scrape, "Scrape"));
    assert_eq!(scrape.state.hand.len(), 2);
    assert_eq!(hand_prefix_count(&scrape, "Strike"), 1);
    assert_eq!(hand_prefix_count(&scrape, "Turbo"), 1);
    assert_eq!(discard_prefix_count(&scrape, "Strike"), 1);
    assert_eq!(discard_prefix_count(&scrape, "Scrape"), 1);
    assert_eq!(discard_prefix_count(&scrape, "Turbo"), 0);
    assert_eq!(scrape.state.discard_pile.len(), 2);
}

#[test]
fn scrape_follow_up_discards_direct_nonzero_costs_in_order_before_evolve_draws() {
    // Scrape.java deals 7 then draws 4. ScrapeFollowUpAction iterates that
    // DrawCardAction's drawnCards in order and manually discards everything
    // except costForTurn == 0 or freeToPlayOnce. EvolvePower queues its extra
    // draw behind the follow-up, so the Defend drawn for Burn is not inspected.
    // Java: reference/extracted/methods/card/Scrape.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/
    // ScrapeFollowUpAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EvolvePower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 1);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Scrape"]);
    // Draw pile top is the final element: direct draws are Burn, Reinforced
    // Body, Turbo, Strike; Evolve then draws the remaining Defend.
    engine.state.draw_pile = make_deck(&["Defend", "Strike", "Turbo", "Reinforced Body", "Burn"]);
    engine.state.player.set_status(sid::EVOLVE, 1);

    assert!(play_on_enemy(&mut engine, "Scrape", 0));

    assert_eq!(engine.state.enemies[0].entity.hp, 33);
    assert_eq!(hand_prefix_count(&engine, "Turbo"), 1);
    assert_eq!(hand_prefix_count(&engine, "Defend"), 1);
    assert_eq!(hand_prefix_count(&engine, "Burn"), 0);
    assert_eq!(hand_prefix_count(&engine, "Reinforced Body"), 0);
    assert_eq!(hand_prefix_count(&engine, "Strike"), 0);
    let discard_names: Vec<_> = engine
        .state
        .discard_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect();
    assert_eq!(
        discard_names,
        vec!["Burn", "Reinforced Body", "Strike", "Scrape"]
    );

    // DamageAction clears the queued DrawCardAction when the opening hit ends
    // combat, so even upgraded Scrape leaves the draw pile untouched on lethal.
    let mut lethal = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 10, 10)], 1);
    force_player_turn(&mut lethal);
    lethal.state.hand = make_deck(&["Scrape+"]);
    lethal.state.draw_pile = make_deck(&["Strike", "Defend"]);

    assert!(play_on_enemy(&mut lethal, "Scrape+", 0));
    assert!(lethal.state.combat_over);
    assert!(lethal.state.hand.is_empty());
    assert_eq!(lethal.state.draw_pile.len(), 2);
}
