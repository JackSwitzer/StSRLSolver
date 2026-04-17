#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/GamblingChip.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GamblingChipAction.java

use crate::actions::Action;
use crate::effects::runtime::GameEvent;
use crate::effects::trigger::Trigger;
use crate::engine::{ChoiceReason, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, make_deck,
};

fn relic_engine(hand: &[&str], draw: &[&str]) -> crate::engine::CombatEngine {
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    state.relics.push("Gambling Chip".to_string());
    let mut engine = engine_with_state(state);
    engine.state.hand = make_deck(hand);
    engine.state.draw_pile = make_deck(draw);
    engine.state.turn = 1;
    engine
}

fn hand_names(engine: &crate::engine::CombatEngine) -> Vec<String> {
    engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect()
}

#[test]
fn gambling_chip_opens_a_zero_to_many_discard_choice_on_the_first_turn_after_draw() {
    let mut engine = relic_engine(
        &["Strike_G", "Defend_G", "Neutralize"],
        &["Survivor", "Deflect"],
    );

    engine.emit_event(GameEvent {
        kind: Trigger::TurnStartPostDrawLate,
        card_type: None,
        card_inst: None,
        is_first_turn: true,
        target_idx: -1,
        enemy_idx: -1,
        potion_slot: -1,
        status_id: None,
        amount: 0,
        replay_window: false,
    });

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Gambling Chip choice");
    assert_eq!(choice.reason, ChoiceReason::DiscardFromHand);
    assert_eq!(choice.min_picks, 0);
    assert_eq!(choice.max_picks, 3);
    assert_eq!(engine.state.player.status(sid::GAMBLING_CHIP_ACTIVE), 1);
}

#[test]
fn gambling_chip_can_discard_any_number_of_cards_and_redraw_that_many() {
    let mut engine = relic_engine(
        &["Strike_G", "Defend_G", "Neutralize"],
        &["Survivor", "Deflect", "Bash"],
    );

    engine.emit_event(GameEvent {
        kind: Trigger::TurnStartPostDrawLate,
        card_type: None,
        card_inst: None,
        is_first_turn: true,
        target_idx: -1,
        enemy_idx: -1,
        potion_slot: -1,
        status_id: None,
        amount: 0,
        replay_window: false,
    });

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(1));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.player.status(sid::GAMBLING_CHIP_ACTIVE), 0);
    assert_eq!(engine.state.discard_pile.len(), 2);
    assert_eq!(hand_names(&engine).len(), 3);
    assert!(hand_names(&engine).contains(&"Bash".to_string()));
    assert!(hand_names(&engine).contains(&"Deflect".to_string()));
}

#[test]
fn gambling_chip_allows_zero_discards_without_changing_the_hand() {
    let mut engine = relic_engine(
        &["Strike_G", "Defend_G", "Neutralize"],
        &["Survivor", "Deflect", "Bash"],
    );

    engine.emit_event(GameEvent {
        kind: Trigger::TurnStartPostDrawLate,
        card_type: None,
        card_inst: None,
        is_first_turn: true,
        target_idx: -1,
        enemy_idx: -1,
        potion_slot: -1,
        status_id: None,
        amount: 0,
        replay_window: false,
    });

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.player.status(sid::GAMBLING_CHIP_ACTIVE), 0);
    assert_eq!(hand_names(&engine), vec!["Strike_G", "Defend_G", "Neutralize"]);
}

#[path = "test_zone_batch_java_wave1.rs"]
mod test_zone_batch_java_wave1;
