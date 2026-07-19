#![cfg(test)]

use super::*;
use crate::actions::Action;
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

const DEFECT_COMMON_POOL: &[&str] = &[
    "Steam", "Cold Snap", "Leap", "Beam Cell", "Hologram", "Conserve Battery",
    "Sweeping Beam", "Turbo", "Coolheaded", "Gash", "Rebound", "Stack", "Barrage",
    "Compile Driver", "Redo", "Streamline", "Ball Lightning", "Go for the Eyes",
];

const COLORLESS_POOL: &[&str] = &[
    "Madness", "Thinking Ahead", "Mind Blast", "Metamorphosis", "Jack Of All Trades",
    "Swift Strike", "Good Instincts", "Master of Strategy", "Magnetism", "Finesse",
    "Discovery", "Chrysalis", "Transmutation", "Panacea", "Purity", "Enlightenment",
    "Forethought", "Flash of Steel", "HandOfGreed", "Mayhem", "Apotheosis", "Secret Weapon",
    "Panache", "Violence", "Deep Breath", "Secret Technique", "Blind", "The Bomb",
    "Impatience", "Dramatic Entrance", "Trip", "PanicButton", "Sadistic Nature", "Dark Shackles",
];

#[test]
fn hello_world_hook_rolls_every_stack_and_spills_past_hand_limit() {
    // HelloPower.atStartOfTurn calls getCard(COMMON, cardRandomRng) once per
    // stack. MakeTempCardInHandAction then puts overflow into the discard pile.
    // Java: powers/HelloPower.java and actions/common/MakeTempCardInHandAction.java.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.player.set_status(sid::HELLO_WORLD, 3);
    engine.state.hand = make_deck(&[
        "Strike", "Strike", "Strike", "Strike", "Strike",
        "Strike", "Strike", "Strike", "Strike",
    ]);
    let mut oracle = engine.card_random_rng.clone();
    let expected: Vec<&str> = (0..3)
        .map(|_| DEFECT_COMMON_POOL[oracle.random_int((DEFECT_COMMON_POOL.len() - 1) as i32) as usize])
        .collect();

    let mut runtime_state = EffectState::default();
    hook_hello_world(
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
fn magnetism_hook_rolls_one_colorless_per_stack_and_spills_past_hand_limit() {
    // MagnetismPower.atStartOfTurn calls the truly-random Colorless helper once
    // per stack. The source pool/order comes from CardLibrary plus
    // AbstractDungeon.addColorlessCards; MakeTempCardInHandAction spills at ten.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.player.set_status(sid::MAGNETISM, 3);
    engine.state.hand = make_deck(&[
        "Strike", "Strike", "Strike", "Strike", "Strike",
        "Strike", "Strike", "Strike", "Strike",
    ]);
    let mut oracle = engine.card_random_rng.clone();
    let expected: Vec<&str> = (0..3)
        .map(|_| COLORLESS_POOL[oracle.random_int((COLORLESS_POOL.len() - 1) as i32) as usize])
        .collect();

    let mut runtime_state = EffectState::default();
    hook_magnetism(
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
        .map(|_| DEFECT_POWER_POOL[oracle.random_int((DEFECT_POWER_POOL.len() - 1) as i32) as usize])
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
fn mayhem_hook_autoplays_top_draw_cards_for_free_without_exhausting() {
    let mut state = combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.draw_pile = make_deck(&["Strike", "Defend", "Bash"]);

    let mut engine = engine_without_start(state.draw_pile.clone(), state.enemies.clone(), 3);
    engine.phase = CombatPhase::PlayerTurn;
    engine.state.draw_pile = state.draw_pile;
    engine.state.player.set_status(sid::MAYHEM, 2);
    let card_random_before = engine.card_random_rng.counter;

    let mut runtime_state = EffectState::default();
    hook_mayhem(
        &mut engine,
        EffectOwner::PlayerPower,
        &post_draw_event(),
        &mut runtime_state,
    );

    let draw_names: Vec<_> = engine
        .state
        .draw_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect();

    assert_eq!(draw_names, vec!["Strike".to_string()]);
    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.enemies[0].entity.hp, 32);
    assert_eq!(engine.state.energy, 3);
    assert_eq!(engine.state.exhaust_pile.len(), 0);
    let discard_names: Vec<_> = engine
        .state
        .discard_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect();
    assert_eq!(discard_names, vec!["Bash".to_string(), "Defend".to_string()]);
    assert_eq!(engine.card_random_rng.counter, card_random_before + 2);
}

#[test]
fn tools_of_the_trade_hook_draws_and_discards_once_per_stack() {
    // ToolsOfTheTradePower.atStartOfTurnPostDraw queues DrawCardAction(amount)
    // followed by non-random DiscardAction(amount).
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ToolsOfTheTradePower.java
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
    assert_eq!(choice.min_picks, 2);
    assert_eq!(choice.max_picks, 2);
    assert_eq!(choice.options.len(), 3);

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::Choose(1));
    engine.execute_action(&Action::ConfirmSelection);
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.state.discard_pile.len(), 2);
}

#[test]
fn tools_of_the_trade_auto_discards_a_short_hand_with_manual_hooks() {
    // DiscardAction moves the whole hand directly when hand.size() <= amount
    // and invokes triggerOnManualDiscard on every moved card.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DiscardAction.java
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    engine.phase = CombatPhase::PlayerTurn;
    engine.state.draw_pile = make_deck(&["Tactician+"]);
    engine.state.player.set_status(sid::TOOLS_OF_THE_TRADE, 2);

    let mut runtime_state = EffectState::default();
    hook_tools_of_the_trade(
        &mut engine,
        EffectOwner::PlayerPower,
        &post_draw_event(),
        &mut runtime_state,
    );

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert!(engine.choice.is_none());
    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.discard_pile.len(), 1);
    assert_eq!(engine.state.energy, 2);
}

#[test]
fn complex_turn_start_power_defs_use_java_trigger_phases() {
    assert_eq!(DEF_HELLO_WORLD.triggers.len(), 1);
    assert_eq!(DEF_HELLO_WORLD.triggers[0].trigger, Trigger::TurnStart);
    assert!(DEF_HELLO_WORLD.complex_hook.is_some());

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
