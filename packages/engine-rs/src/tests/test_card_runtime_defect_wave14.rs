#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Seek.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Chaos.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Fission.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Reboot.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Redo.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Scrape.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::engine::{ChoiceReason, CombatPhase};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    discard_prefix_count, enemy_no_intent, engine_without_start, force_player_turn, hand_prefix_count,
    make_deck, play_self, TEST_SEED,
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
            min_picks: A::Fixed(1),
            max_picks: A::Magic,
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
            min_picks: A::Fixed(1),
            max_picks: A::Magic,
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
    assert!(fission.effect_data.is_empty());
    assert!(fission.complex_hook.is_some());

    let reboot = global_registry().get("Reboot").expect("Reboot");
    assert!(reboot.effect_data.is_empty());
    assert!(reboot.complex_hook.is_some());

    let redo = global_registry().get("Redo").expect("Redo");
    assert_eq!(
        redo.effect_data,
        &[E::Simple(SE::EvokeAndRechannelFrontOrb)]
    );
    assert!(redo.complex_hook.is_none());

    let scrape = global_registry().get("Scrape").expect("Scrape");
    assert!(scrape.effect_data.is_empty());
    assert!(scrape.complex_hook.is_some());
}

#[test]
fn seek_plus_searches_the_draw_pile_with_the_declarative_choice_surface() {
    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Strike_B", "Defend_B", "Zap"]),
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

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.state.draw_pile.len(), 2);
}

#[test]
fn fission_reboot_and_scrape_follow_the_current_hooked_defect_runtime_paths() {
    let mut fission = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut fission);
    fission.init_defect_orbs(3);
    fission.channel_orb(OrbType::Lightning);
    fission.channel_orb(OrbType::Frost);
    fission.channel_orb(OrbType::Dark);
    fission.state.hand = make_deck(&["Fission"]);
    fission.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap", "Dualcast"]);

    assert!(play_self(&mut fission, "Fission"));
    assert_eq!(fission.state.orb_slots.occupied_count(), 0);
    assert_eq!(fission.state.energy, 6);
    assert_eq!(fission.state.hand.len(), 3);

    let mut reboot = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut reboot);
    reboot.state.hand = make_deck(&["Reboot", "Strike_B", "Defend_B"]);
    reboot.state.draw_pile.clear();
    reboot.state.discard_pile = make_deck(&["Zap", "Dualcast", "Cold Snap"]);

    assert!(play_self(&mut reboot, "Reboot"));
    assert_eq!(reboot.state.hand.len(), 4);
    assert_eq!(reboot.state.exhaust_pile.len(), 1);
    assert_eq!(
        reboot.card_registry.card_name(reboot.state.exhaust_pile[0].def_id),
        "Reboot"
    );
    assert_eq!(reboot.state.discard_pile.len(), 0);

    let mut scrape = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut scrape);
    scrape.state.hand = make_deck(&["Scrape"]);
    scrape.state.draw_pile = make_deck(&["Turbo", "Strike_B"]);

    assert!(play_self(&mut scrape, "Scrape"));
    assert!(hand_prefix_count(&scrape, "Turbo") >= 1);
    assert_eq!(discard_prefix_count(&scrape, "Strike_B"), 1);
}

#[test]
fn consume_uses_the_typed_orb_slot_removal_surface() {
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
    assert_eq!(engine.state.energy, 3);
    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    assert_eq!(engine.state.orb_slots.slots[1].orb_type, OrbType::Frost);
}

#[test]
fn darkness_plus_channels_dark_then_triggers_dark_passive() {
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

    assert!(play_self(&mut engine, "Darkness+"));

    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Dark);
    assert_eq!(engine.state.orb_slots.slots[0].evoke_amount, 12);
}

#[test]
fn redo_reuses_the_front_orb_type_on_the_typed_surface() {
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
    engine.channel_orb(OrbType::Plasma);

    assert!(play_self(&mut engine, "Redo"));

    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
    assert_eq!(engine.state.orb_slots.front_orb_type(), OrbType::Plasma);
    assert_eq!(engine.state.energy, 4);
}

#[test]
#[ignore = "Fission still needs a remove-all-orbs primitive before the energy/draw payload can be typed; Java /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/FissionAction.java removes or evokes all orbs first, with /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Fission.java as the card entry point."]
fn fission_still_needs_remove_all_orbs_before_payload() {}

#[test]
#[ignore = "Reboot still needs a shuffle-hand-and-discard-into-draw primitive; Java /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ShuffleAllAction.java moves the whole hand and discard before drawing, with /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Reboot.java as the card entry point."]
fn reboot_still_needs_shuffle_hand_and_discard_into_draw() {}

#[test]
#[ignore = "Scrape still needs a draw-then-discard-non-zero-cost follow-up primitive; Java /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ScrapeAction.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ScrapeFollowUpAction.java split the draw and follow-up discard after resolution."]
fn scrape_still_needs_draw_then_discard_non_zero_cost_follow_up() {}
