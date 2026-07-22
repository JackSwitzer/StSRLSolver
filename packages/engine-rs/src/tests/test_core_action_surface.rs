#![cfg(test)]

use crate::actions::Action;
use crate::decision::{build_combat_context, DecisionKind};
use crate::run::{GameAction, RunEngine, RunPhase};

use super::support::resolve_opening_neow;

#[test]
fn potion_belt_fifth_slot_is_a_canonical_combat_action() {
    // PotionBelt.java::onEquip appends two slots to the standard three.
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Potion Belt".to_string());
    engine.run_state.max_potions = 5;
    engine.run_state.potions.resize(5, String::new());
    engine.run_state.potions[4] = "Fire Potion".to_string();
    resolve_opening_neow(&mut engine);
    engine.debug_enter_specific_combat(&["JawWorm"]);

    assert!(engine
        .get_legal_actions()
        .contains(&GameAction::CombatAction(Action::UsePotion {
            potion_idx: 4,
            target_idx: 0,
        },)));
}

#[test]
fn frozen_eye_alone_exposes_the_actual_draw_order() {
    // FrozenEye.java and DrawPileViewScreen.java::open expose draw order only
    // while Frozen Eye is owned.
    let mut plain = RunEngine::new(42, 20);
    plain.debug_enter_specific_combat(&["JawWorm"]);
    let plain_context = build_combat_context(plain.get_combat_engine().expect("plain combat"));
    assert!(plain_context.draw_order.is_empty());

    let mut frozen = RunEngine::new(42, 20);
    frozen.run_state.relics.push("Frozen Eye".to_string());
    frozen.debug_enter_specific_combat(&["JawWorm"]);
    let combat = frozen.get_combat_engine().expect("Frozen Eye combat");
    let expected: Vec<String> = combat
        .state
        .draw_pile
        .iter()
        .map(|card| combat.card_registry.card_name(card.def_id).to_string())
        .collect();

    assert!(!expected.is_empty());
    assert_eq!(build_combat_context(combat).draw_order, expected);
}

#[test]
fn step_outcome_matches_the_canonical_next_decision() {
    let mut engine = RunEngine::new(7, 20);
    assert_eq!(engine.current_decision_state().kind, DecisionKind::Proceed);
    assert_eq!(engine.get_legal_actions(), vec![GameAction::Proceed]);

    // NeowEvent commits intro, reward choice, and screen-99 exit separately.
    // Java: NeowEvent.java:160-219,236-238.
    let intro = engine.step_game(&GameAction::Proceed);
    assert!(intro.accepted());
    assert_eq!(intro.next_decision.state, engine.current_decision_state());
    assert_eq!(intro.next_decision.state.kind, DecisionKind::NeowChoice);
    assert_eq!(intro.next_decision.legal_actions.len(), 4);

    let choice = engine.step_game(&GameAction::ChooseNeowOption(1));
    assert!(choice.accepted());
    assert_eq!(choice.next_decision.state, engine.current_decision_state());
    assert_eq!(choice.next_decision.state.kind, DecisionKind::Proceed);
    assert_eq!(
        choice.next_decision.legal_actions,
        vec![GameAction::Proceed]
    );

    let exit = engine.step_game(&GameAction::Proceed);
    assert!(exit.accepted());
    assert_eq!(exit.next_decision.state, engine.current_decision_state());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);

    let map_action = engine.get_legal_actions()[0].clone();
    let outcome = engine.step_game(&map_action);

    assert!(outcome.accepted());
    assert_eq!(outcome.next_decision.state, engine.current_decision_state());
    assert_eq!(
        outcome.next_decision.context,
        engine.current_decision_context()
    );
    assert_eq!(
        outcome.next_decision.legal_actions,
        engine.get_legal_actions()
    );
    assert_eq!(outcome.next_decision.state.kind, DecisionKind::CombatAction);
}

#[test]
fn blocked_campfire_options_are_absent_from_core_legality() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Coffee Dripper".to_string());
    engine.run_state.relics.push("Fusion Hammer".to_string());
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);
    engine.run_state.deck = vec!["Strike+".to_string(), "Defend+".to_string()];
    engine.debug_set_campfire_phase();

    assert_eq!(engine.current_phase(), RunPhase::Campfire);
    assert_eq!(engine.get_legal_actions(), vec![GameAction::CampfireRecall]);
    let context = engine
        .current_decision_context()
        .campfire
        .expect("campfire");
    assert!(!context.can_rest);
    assert!(context.upgradable_cards.is_empty());
}

#[test]
fn campfire_with_no_usable_options_completes_immediately() {
    // CampfireUI checks every option after construction and completes the room
    // immediately when none can be used. Coffee Dripper and Fusion Hammer
    // disable Rest and Smith; an owned Ruby key removes Recall.
    // Java: CampfireUI.java, CoffeeDripper.java, and FusionHammer.java.
    let mut engine = RunEngine::new(43, 0);
    engine.run_state.relics.push("Coffee Dripper".to_string());
    engine.run_state.relics.push("Fusion Hammer".to_string());
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);
    engine.run_state.deck = vec!["Strike+".to_string(), "Defend+".to_string()];
    engine.run_state.has_ruby_key = true;

    engine.debug_set_campfire_phase();

    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert!(engine.current_decision_context().campfire.is_none());
}
