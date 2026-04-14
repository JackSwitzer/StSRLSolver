#[cfg(test)]
mod event_runtime_wave6_tests {
    use crate::decision::{RewardItemKind, RewardScreenSource};
    use crate::events::{typed_events_for_act, EventRuntimeStatus, TypedEventDef};
    use crate::run::{RunAction, RunEngine, RunPhase};

    fn typed_event(act: i32, name: &str) -> TypedEventDef {
        typed_events_for_act(act)
            .into_iter()
            .find(|event| event.name == name)
            .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
    }

    #[test]
    fn mushrooms_stomp_branch_enters_scripted_combat_and_continues_to_event_relic_reward() {
        let mut engine = RunEngine::new(61, 20);
        let mushrooms = typed_event(1, "Mushrooms");
        assert!(matches!(
            mushrooms.options[0].status,
            EventRuntimeStatus::Supported
        ));
        engine.debug_set_typed_event_state(mushrooms);

        let start = engine.step_with_result(&RunAction::EventChoice(0));
        assert!(start.action_accepted);
        assert_eq!(engine.current_phase(), RunPhase::Combat);
        let combat = engine.get_combat_engine().expect("event combat");
        assert_eq!(combat.state.enemies.len(), 3);
        assert!(combat.state.enemies.iter().all(|enemy| enemy.id == "FungiBeast"));

        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert_eq!(engine.current_phase(), RunPhase::CardReward);

        let screen = engine.current_reward_screen().expect("event reward screen");
        assert_eq!(screen.source, RewardScreenSource::Event);
        assert_eq!(screen.items.len(), 1);
        assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
        assert_eq!(screen.items[0].label, "Odd Mushroom");

        let claim = engine.step_with_result(&RunAction::SelectRewardItem(0));
        assert!(claim.action_accepted);
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        assert!(engine.run_state.relics.iter().any(|relic| relic == "Odd Mushroom"));
    }

    #[test]
    fn mysterious_sphere_open_branch_enters_scripted_combat_and_keeps_event_owned_reward_flow() {
        let mut engine = RunEngine::new(67, 20);
        let sphere = typed_event(3, "Mysterious Sphere");
        assert!(matches!(
            sphere.options[0].status,
            EventRuntimeStatus::Supported
        ));
        engine.debug_set_typed_event_state(sphere);

        let start = engine.step_with_result(&RunAction::EventChoice(0));
        assert!(start.action_accepted);
        assert_eq!(engine.current_phase(), RunPhase::Combat);
        let combat = engine.get_combat_engine().expect("event combat");
        assert_eq!(combat.state.enemies.len(), 2);
        assert!(combat.state.enemies.iter().all(|enemy| enemy.id == "OrbWalker"));

        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert_eq!(engine.current_phase(), RunPhase::CardReward);

        let screen = engine.current_reward_screen().expect("event reward screen");
        assert_eq!(screen.source, RewardScreenSource::Event);
        assert_eq!(screen.items.len(), 1);
        assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
        assert!(screen.items[0].claimable);
    }
}
