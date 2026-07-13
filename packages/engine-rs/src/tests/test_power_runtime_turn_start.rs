#![cfg(test)]

use super::*;
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::Trigger;
use crate::engine::{CombatPhase, ChoiceReason};
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_without_start, make_deck};

fn post_draw_event() -> GameEvent {
    GameEvent::empty(Trigger::TurnStartPostDraw)
}

const DEFECT_POWER_POOL: &[&str] = &[
    "Defragment", "Capacitor", "Heatsinks", "Static Discharge", "Loop", "Hello World", "Storm",
    "Biased Cognition", "Machine Learning", "Electrodynamics", "Buffer", "Echo Form", "Creative AI",
];

#[test]
fn creative_ai_hook_rolls_every_stack_and_spills_past_hand_limit() {
    // CreativeAIPower.java rolls one source-pool Power per stack before
    // queuing MakeTempCardInHandAction; each roll consumes cardRandomRng, and
    // MakeTempCardInHandAction spills cards past ten into the discard pile.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.player.set_status(sid::CREATIVE_AI, 3);
    engine.state.hand = make_deck(&[
        "Strike", "Strike", "Strike", "Strike", "Strike",
        "Strike", "Strike", "Strike", "Strike",
    ]);
    let mut oracle = engine.card_random_rng.clone();
    let expected: Vec<&str> = (0..3)
        .map(|_| DEFECT_POWER_POOL[oracle.random((DEFECT_POWER_POOL.len() - 1) as i32) as usize])
        .collect();

    let mut runtime_state = EffectState::default();
    hook_creative_ai(
        &mut engine,
        EffectOwner::PlayerPower,
        &post_draw_event(),
        &mut runtime_state,
    );

    assert_eq!(engine.state.hand.len(), 10);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[9].def_id), expected[0]);
    assert_eq!(engine.state.discard_pile.len(), 2);
    assert_eq!(engine.card_registry.card_name(engine.state.discard_pile[0].def_id), expected[1]);
    assert_eq!(engine.card_registry.card_name(engine.state.discard_pile[1].def_id), expected[2]);
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
}

#[test]
fn enter_divinity_hook_clears_flag_and_enters_divinity() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.energy = 2;
    engine.state.stance = Stance::Neutral;
    engine.state.player.set_status(sid::ENTER_DIVINITY, 1);

    let mut runtime_state = EffectState::default();
    hook_enter_divinity(
        &mut engine,
        EffectOwner::PlayerPower,
        &post_draw_event(),
        &mut runtime_state,
    );

    assert_eq!(engine.state.player.status(sid::ENTER_DIVINITY), 0);
    assert_eq!(engine.state.stance, Stance::Divinity);
    assert_eq!(engine.state.energy, 5);
}

#[test]
fn mayhem_hook_moves_top_draw_cards_into_hand() {
    let mut state = combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.draw_pile = make_deck(&["Strike", "Defend", "Bash"]);

    let mut engine = engine_without_start(state.draw_pile.clone(), state.enemies.clone(), 3);
    engine.state.draw_pile = state.draw_pile;
    engine.state.player.set_status(sid::MAYHEM, 2);

    let mut runtime_state = EffectState::default();
    hook_mayhem(
        &mut engine,
        EffectOwner::PlayerPower,
        &post_draw_event(),
        &mut runtime_state,
    );

    let hand_names: Vec<_> = engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect();
    let draw_names: Vec<_> = engine
        .state
        .draw_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect();

    assert_eq!(hand_names, vec!["Bash".to_string(), "Defend".to_string()]);
    assert_eq!(draw_names, vec!["Strike".to_string()]);
}

#[test]
fn tools_of_the_trade_hook_draws_and_opens_single_discard_choice() {
    let mut state = combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.hand = make_deck(&["Strike"]);
    state.draw_pile = make_deck(&["Defend", "Bash", "Inflame"]);

    let mut engine = engine_without_start(Vec::new(), state.enemies.clone(), 3);
    engine.phase = CombatPhase::PlayerTurn;
    engine.state.hand = state.hand;
    engine.state.draw_pile = state.draw_pile;
    engine.state.player.set_status(sid::TOOLS_OF_THE_TRADE, 2);

    let mut runtime_state = EffectState::default();
    hook_tools_of_the_trade(
        &mut engine,
        EffectOwner::PlayerPower,
        &post_draw_event(),
        &mut runtime_state,
    );

    assert_eq!(engine.state.hand.len(), 3);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("tools of the trade should open a discard choice");
    assert_eq!(choice.reason, ChoiceReason::DiscardFromHand);
    assert_eq!(choice.min_picks, 1);
    assert_eq!(choice.max_picks, 1);
    assert_eq!(choice.options.len(), 3);
}

#[test]
fn complex_turn_start_power_defs_use_java_trigger_phases() {
    assert_eq!(DEF_CREATIVE_AI.triggers.len(), 1);
    assert_eq!(DEF_CREATIVE_AI.triggers[0].trigger, Trigger::TurnStart);
    assert!(DEF_CREATIVE_AI.complex_hook.is_some());

    for def in [
        &DEF_ENTER_DIVINITY,
        &DEF_MAYHEM,
        &DEF_TOOLS_OF_THE_TRADE,
    ] {
        assert_eq!(def.triggers.len(), 1);
        assert_eq!(def.triggers[0].trigger, Trigger::TurnStartPostDraw);
        assert!(def.complex_hook.is_some());
    }
}
