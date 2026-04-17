use crate::decision::{RewardItemKind, RewardScreenSource};
use crate::events::{typed_events_for_act, EventRuntimeStatus, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/city/Colosseum.java
// - decompiled/java-src/com/megacrit/cardcrawl/events/city/CursedTome.java
// - decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SpireHeart.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn colosseum_is_supported_and_uses_event_continuation_plus_two_combats() {
    let mut engine = RunEngine::new(91, 20);
    let colosseum = typed_event(2, "Colosseum");
    assert!(matches!(
        colosseum.options[0].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(colosseum);

    let intro = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(intro.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    let intro_ctx = engine.current_decision_context().event.expect("colosseum follow-up event");
    assert_eq!(intro_ctx.name, "Colosseum");
    assert_eq!(intro_ctx.options.len(), 1);

    let first_fight = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(first_fight.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    let combat = engine.get_combat_engine().expect("slavers combat");
    assert_eq!(combat.state.enemies.len(), 3);
    assert!(combat.state.enemies.iter().any(|enemy| enemy.id == "TaskMaster"));
    assert!(combat.state.enemies.iter().any(|enemy| enemy.id == "SlaverBlue"));
    assert!(combat.state.enemies.iter().any(|enemy| enemy.id == "SlaverRed"));

    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();
    assert_eq!(engine.current_phase(), RunPhase::Event);
    let post_ctx = engine.current_decision_context().event.expect("colosseum post combat");
    assert_eq!(post_ctx.name, "Colosseum");
    assert_eq!(post_ctx.options.len(), 2);

    let second_fight = engine.step_with_result(&RunAction::EventChoice(1));
    assert!(second_fight.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    let combat = engine.get_combat_engine().expect("nobs combat");
    assert_eq!(combat.state.enemies.len(), 2);
    assert!(combat.state.enemies.iter().all(|enemy| enemy.id == "GremlinNob"));

    let gold_before = engine.run_state.gold;
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert_eq!(engine.run_state.gold, gold_before + 100);

    let screen = engine.current_reward_screen().expect("colosseum reward screen");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 2);
    assert!(screen.items.iter().all(|item| item.kind == RewardItemKind::Relic));
}

#[test]
fn cursed_tome_progresses_page_by_page_and_opens_book_reward_on_take() {
    let mut engine = RunEngine::new(93, 20);
    engine.run_state.max_hp = 80;
    engine.run_state.current_hp = 80;
    let cursed_tome = typed_event(2, "Cursed Tome");
    assert!(matches!(
        cursed_tome.options[0].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(cursed_tome);

    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 80);

    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 79);

    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 77);

    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 74);

    let take = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(take.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert_eq!(engine.run_state.current_hp, 59);

    let screen = engine.current_reward_screen().expect("cursed tome reward");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert!(matches!(
        screen.items[0].label.as_str(),
        "Necronomicon" | "Enchiridion" | "Nilry's Codex" | "Circlet"
    ));
}

#[test]
fn cursed_tome_stop_reading_takes_three_and_returns_to_map_without_reward() {
    let mut engine = RunEngine::new(95, 20);
    engine.run_state.max_hp = 80;
    engine.run_state.current_hp = 80;
    engine.debug_set_typed_event_state(typed_event(2, "Cursed Tome"));

    engine.step_with_result(&RunAction::EventChoice(0));
    engine.step_with_result(&RunAction::EventChoice(0));
    engine.step_with_result(&RunAction::EventChoice(0));
    engine.step_with_result(&RunAction::EventChoice(0));
    let stop = engine.step_with_result(&RunAction::EventChoice(1));

    assert!(stop.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.current_hp, 71);
    assert!(engine.current_reward_screen().is_none());
}
