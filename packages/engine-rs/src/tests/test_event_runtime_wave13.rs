use crate::events::{typed_events_for_act, EventProgramOp, EventRuntimeStatus, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SpireHeart.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn spire_heart_branch_is_supported_and_uses_canonical_final_act_op() {
    let spire_heart = typed_event(3, "Spire Heart");
    assert!(matches!(
        spire_heart.options[0].status,
        EventRuntimeStatus::Supported
    ));
    assert!(matches!(
        spire_heart.options[0].program.ops.as_slice(),
        [EventProgramOp::ResolveFinalAct]
    ));
}

#[test]
fn spire_heart_without_keys_ends_run_on_canonical_terminal_path() {
    let mut engine = RunEngine::new(313, 20);
    engine.debug_set_typed_event_state(typed_event(3, "Spire Heart"));

    let step = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::GameOver);
    assert!(engine.run_state.run_won);
    assert!(engine.run_state.run_over);
    assert!(engine.current_reward_screen().is_none());
}

#[test]
fn spire_heart_with_keys_starts_act_four_on_event_runtime_path() {
    let mut engine = RunEngine::new(313, 20);
    engine.run_state.has_ruby_key = true;
    engine.run_state.has_emerald_key = true;
    engine.run_state.has_sapphire_key = true;
    engine.debug_set_typed_event_state(typed_event(3, "Spire Heart"));

    let step = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    assert_eq!(engine.run_state.act, 4);
    assert_eq!(engine.boss_name(), "CorruptHeart");
    assert_eq!(
        engine.debug_current_enemy_ids(),
        vec!["SpireShield".to_string(), "SpireSpear".to_string()]
    );
    let pending = engine
        .pending_event_combat_summary()
        .expect("final act should queue the Heart after the elite combat");
    assert!(pending.contains("CorruptHeart"));
    assert!(pending.contains("StartBossCombat"));
}

#[test]
fn spire_heart_act_four_chain_reaches_heart_and_ends_without_boss_reward() {
    let mut engine = RunEngine::new(313, 20);
    engine.run_state.has_ruby_key = true;
    engine.run_state.has_emerald_key = true;
    engine.run_state.has_sapphire_key = true;
    engine.debug_set_typed_event_state(typed_event(3, "Spire Heart"));

    let enter = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(enter.action_accepted);
    assert_eq!(engine.debug_current_enemy_ids().len(), 2);

    engine.debug_force_current_combat_outcome(true);
    let shield_spear = engine.debug_resolve_current_combat_outcome();
    assert!(shield_spear > 0.0);
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    assert_eq!(engine.debug_current_enemy_ids(), vec!["CorruptHeart".to_string()]);

    engine.debug_force_current_combat_outcome(true);
    let heart = engine.debug_resolve_current_combat_outcome();
    assert!(heart >= 6.0);
    assert_eq!(engine.current_phase(), RunPhase::GameOver);
    assert!(engine.run_state.run_won);
    assert!(engine.run_state.run_over);
    assert!(engine.current_reward_screen().is_none());
}
