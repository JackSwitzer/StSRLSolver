#![cfg(test)]

use crate::actions::Action;
use crate::effects::runtime::{EffectOwner, EventRecordPhase};
use crate::effects::trigger::Trigger;
use crate::engine::{ChoiceReason, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_self};

fn hand_names(engine: &crate::engine::CombatEngine) -> Vec<String> {
    engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect()
}

#[test]
fn thousand_cuts_hits_all_enemies_via_engine_play_path() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 35, 35),
    ];
    let mut engine = engine_with_state(combat_state_with(Vec::new(), enemies, 3));
    engine.state.player.set_status(sid::THOUSAND_CUTS, 2);
    engine.state.hand = make_deck(&["Defend_R"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.clear_event_log();

    assert!(play_self(&mut engine, "Defend_R"));

    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.enemies[0].entity.hp, 38);
    assert_eq!(engine.state.enemies[1].entity.hp, 33);

    let events = engine.take_event_log();
    assert!(events.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.event == Trigger::OnAnyCardPlayed
            && record.def_id == Some("thousand_cuts")
    }));
}

#[test]
fn panache_tracks_cards_and_bursts_on_the_fifth_real_play() {
    let enemies = vec![
        enemy_no_intent("JawWorm", 50, 50),
        enemy_no_intent("Cultist", 45, 45),
    ];
    let mut engine = engine_with_state(combat_state_with(Vec::new(), enemies, 10));
    engine.state.player.set_status(sid::PANACHE, 7);
    engine.state.hand = make_deck(&[
        "Defend_R",
        "Defend_R",
        "Defend_R",
        "Defend_R",
        "Defend_R",
    ]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    for plays in 1..=4 {
        assert!(play_self(&mut engine, "Defend_R"));
        assert_eq!(engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0), plays);
        assert_eq!(engine.state.enemies[0].entity.hp, 50);
        assert_eq!(engine.state.enemies[1].entity.hp, 45);
    }

    assert!(play_self(&mut engine, "Defend_R"));

    assert_eq!(engine.hidden_effect_value("panache", EffectOwner::PlayerPower, 0), 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 43);
    assert_eq!(engine.state.enemies[1].entity.hp, 38);
}

#[test]
fn creative_ai_adds_its_generated_card_on_real_turn_start() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.player.set_status(sid::CREATIVE_AI, 1);
    engine.state.hand.clear();
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.clear_event_log();

    engine.execute_action(&Action::EndTurn);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(hand_names(&engine), vec!["Smite".to_string()]);

    let events = engine.take_event_log();
    assert!(events.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.event == Trigger::TurnStartPostDraw
            && record.def_id == Some("creative_ai")
    }));
}

#[test]
fn mayhem_moves_the_remaining_top_draw_card_into_hand_on_turn_start() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.player.set_status(sid::MAYHEM, 1);
    engine.state.hand.clear();
    engine.state.discard_pile.clear();
    engine.state.draw_pile = make_deck(&[
        "Strike_R",
        "Defend_R",
        "Bash",
        "Shrug It Off",
        "Inflame",
        "Flex",
    ]);
    engine.clear_event_log();

    engine.execute_action(&Action::EndTurn);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 6);
    assert!(hand_names(&engine).contains(&"Strike_R".to_string()));
    assert!(engine.state.draw_pile.is_empty());

    let events = engine.take_event_log();
    assert!(events.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.event == Trigger::TurnStartPostDraw
            && record.def_id == Some("mayhem")
    }));
}

#[test]
fn tools_of_the_trade_draws_then_opens_and_resolves_a_real_discard_choice() {
    let mut engine = engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.player.set_status(sid::TOOLS_OF_THE_TRADE, 1);
    engine.state.hand.clear();
    engine.state.discard_pile.clear();
    engine.state.draw_pile = make_deck(&[
        "Strike_R",
        "Defend_R",
        "Bash",
        "Shrug It Off",
        "Inflame",
        "Flex",
    ]);
    engine.clear_event_log();

    engine.execute_action(&Action::EndTurn);

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("tools of the trade should open a discard choice");
    assert_eq!(choice.reason, ChoiceReason::DiscardFromHand);
    assert_eq!(choice.options.len(), 6);
    assert_eq!(engine.state.hand.len(), 6);

    engine.execute_action(&Action::Choose(0));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 5);
    assert_eq!(engine.state.discard_pile.len(), 1);

    let events = engine.take_event_log();
    assert!(events.iter().any(|record| {
        record.phase == EventRecordPhase::Handled
            && record.event == Trigger::TurnStartPostDraw
            && record.def_id == Some("tools_of_the_trade")
    }));
}
