use crate::decision::{RewardItemKind, RewardScreenSource};
use crate::events::{typed_events_for_act, EventRuntimeStatus};
use crate::run::{RunAction, RunEngine, RunPhase};

fn typed_event(act: i32, name: &str) -> crate::events::TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn test_event_runtime_wave7_mind_bloom_war_enters_scripted_boss_combat_and_opens_rare_relic_reward() {
    let mut engine = RunEngine::new(73, 20);
    let mind_bloom = typed_event(3, "Mind Bloom");
    assert!(matches!(
        mind_bloom.options[0].status,
        EventRuntimeStatus::Supported
    ));

    engine.debug_set_typed_event_state(mind_bloom);
    let start = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(start.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Combat);

    let combat = engine.get_combat_engine().expect("mind bloom combat");
    assert_eq!(combat.state.enemies.len(), 1);
    let boss_id = combat.state.enemies[0].id.as_str();
    assert!(matches!(boss_id, "TheGuardian" | "Hexaghost" | "SlimeBoss"));

    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();
    assert_eq!(engine.current_phase(), RunPhase::CardReward);

    let screen = engine.current_reward_screen().expect("mind bloom reward screen");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert!(screen.items[0].claimable);
    assert_ne!(screen.items[0].label, "rare relic");

    let relic_before = engine.run_state.relics.len();
    let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(claim.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.relics.len(), relic_before + 1);
}
