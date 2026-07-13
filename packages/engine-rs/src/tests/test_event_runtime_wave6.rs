#[cfg(test)]
mod event_runtime_wave6_tests {
    use crate::decision::{RewardItemKind, RewardScreenSource};
    use crate::events::{typed_events_for_act, EventRuntimeStatus, TypedEventDef};
    use crate::run::{RunAction, RunEngine, RunPhase};
    use crate::status_ids::sid;

    fn typed_event(act: i32, name: &str) -> TypedEventDef {
        typed_events_for_act(act)
            .into_iter()
            .find(|event| event.name == name)
            .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
    }

    #[test]
    fn mushrooms_stomp_branch_enters_scripted_combat_and_continues_to_event_relic_reward() {
        let mut engine = RunEngine::new(61, 20);
        let gold_before = engine.run_state.gold;
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
        assert!((gold_before + 20..=gold_before + 30).contains(&engine.run_state.gold));

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
    fn odd_mushroom_from_event_reduces_vulnerable_damage_and_duplicates_to_circlet() {
        // OddMushroom.java defines vulnerability effectiveness as 1.25.
        // VulnerablePower.java applies that multiplier to NORMAL damage against
        // the player, and Mushrooms.java substitutes Circlet on duplicate reward.
        let mushrooms = typed_event(1, "Mushrooms");
        let mut engine = RunEngine::new(71, 20);
        engine.debug_set_typed_event_state(mushrooms.clone());
        assert!(engine
            .step_with_result(&RunAction::EventChoice(0))
            .action_accepted);
        engine.debug_force_current_combat_outcome(true);
        engine.debug_resolve_current_combat_outcome();
        assert!(engine
            .step_with_result(&RunAction::SelectRewardItem(0))
            .action_accepted);

        engine.debug_enter_specific_combat(&["JawWorm"]);
        let combat = engine.debug_combat_engine_mut();
        combat.state.player.set_status(sid::VULNERABLE, 1);
        let hp_before = combat.state.player.hp;
        assert_eq!(combat.state.enemies[0].move_damage(), 12);
        crate::combat_hooks::do_enemy_turns(combat);
        assert_eq!(combat.state.player.hp, hp_before - 15);

        let mut duplicate = RunEngine::new(73, 20);
        duplicate.run_state.relics.push("Odd Mushroom".to_string());
        duplicate.debug_set_typed_event_state(mushrooms);
        assert!(duplicate
            .step_with_result(&RunAction::EventChoice(0))
            .action_accepted);
        duplicate.debug_force_current_combat_outcome(true);
        duplicate.debug_resolve_current_combat_outcome();
        let screen = duplicate.current_reward_screen().expect("duplicate reward");
        assert_eq!(screen.items[0].label, "Circlet");
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
        assert!(combat.state.enemies.iter().all(|enemy| enemy.id == "Orb Walker"));

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
