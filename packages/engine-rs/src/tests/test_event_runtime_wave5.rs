use crate::events::{typed_events_for_act, EventRuntimeStatus};
use crate::run::{GameAction, RunEngine, RunPhase};

fn typed_event(act: i32, name: &str) -> crate::events::TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn test_event_runtime_wave5_mind_bloom_awake_installs_mark_and_blocks_future_heal() {
    let mut engine = RunEngine::new(23, 20);
    engine.run_state.deck = vec![
        "Strike".to_string(),
        "Defend".to_string(),
        "Vigilance".to_string(),
    ];
    engine.run_state.current_hp = 20;

    let mind_bloom = typed_event(3, "Mind Bloom");
    assert!(matches!(
        mind_bloom.options[1].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(mind_bloom);

    let awake = engine.step_game(&GameAction::EventChoice(1));
    assert!(awake.accepted());
    // MindBloom.java::buttonEffect obtains the relic during this choice; it
    // does not interpose a reward-screen decision.
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);

    assert!(engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "Mark of the Bloom"));
    assert!(engine.run_state.deck.iter().all(|card| card.ends_with('+')));

    let library = typed_event(2, "The Library");
    engine.debug_set_typed_event_state(library);
    let sleep = engine.step_game(&GameAction::EventChoice(1));
    assert!(sleep.accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.current_hp, 20);
}

#[test]
fn mark_of_the_bloom_acquired_from_mind_bloom_blocks_next_combat_heal() {
    // MarkOfTheBloom.java::onPlayerHeal always returns 0. MindBloom.java obtains
    // the relic immediately in "I am Awake", so the next combat must install
    // that behavior without any extra reward-screen action.
    let mut engine = RunEngine::new(29, 0);
    engine.run_state.current_hp = 20;
    engine.debug_set_typed_event_state(typed_event(3, "Mind Bloom"));

    assert!(engine.step_game(&GameAction::EventChoice(1)).accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);

    engine.debug_enter_specific_combat(&["JawWorm"]);
    let combat = engine.debug_combat_engine_mut();
    assert_eq!(combat.state.player.hp, 20);
    combat.state.heal_player(12);
    assert_eq!(combat.state.player.hp, 20);
}

#[test]
fn test_event_runtime_wave5_mind_bloom_rich_adds_gold_and_two_normalities() {
    let mut engine = RunEngine::new(7, 20);
    let gold_before = engine.run_state.gold;
    let deck_before = engine.run_state.deck.len();

    let mind_bloom = typed_event(3, "Mind Bloom");
    assert!(matches!(
        mind_bloom.options[2].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(mind_bloom);

    let rich = engine.step_game(&GameAction::EventChoice(2));
    assert!(rich.accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.gold, gold_before + 999);
    assert_eq!(engine.run_state.deck.len(), deck_before + 2);
    assert_eq!(
        engine
            .run_state
            .deck
            .iter()
            .filter(|card| card.as_str() == "Normality")
            .count(),
        2
    );
}

#[test]
fn test_event_runtime_wave5_moai_head_trades_away_golden_idol_for_gold() {
    let mut engine = RunEngine::new(31, 20);
    engine.run_state.gold = 90;
    engine.run_state.relics = vec!["Golden Idol".to_string()];
    engine
        .run_state
        .relic_flags
        .rebuild(&engine.run_state.relics);

    let moai = typed_event(3, "The Moai Head");
    engine.debug_set_typed_event_state(moai);

    let trade = engine.step_game(&GameAction::EventChoice(1));
    assert!(trade.accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.gold, 423);
    assert!(!engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "Golden Idol" || relic == "GoldenIdol"));
    assert!(!engine
        .run_state
        .relic_flags
        .has(crate::relic_flags::flag::GOLDEN_IDOL));
}
