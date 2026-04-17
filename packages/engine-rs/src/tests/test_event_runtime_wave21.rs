use std::sync::{Mutex, OnceLock};

use crate::decision::RewardScreenSource;
use crate::decision::RewardChoice;
use crate::events::{typed_shrine_events, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/events/shrines/NoteForYourself.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java

fn typed_shrine_event(name: &str) -> TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
}

fn note_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn advance_note_event_to_reward(engine: &mut RunEngine) {
    engine.debug_set_typed_event_state(typed_shrine_event("NoteForYourself"));

    let intro = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(intro.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);

    let take = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(take.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
}

fn reward_choice_card_id(choice: &RewardChoice) -> Option<&str> {
    match choice {
        RewardChoice::Card { card_id, .. } => Some(card_id.as_str()),
        RewardChoice::Named { .. } => None,
    }
}

#[test]
fn note_for_yourself_claims_stored_card_then_saves_selected_deck_card_for_future_runs() {
    let _guard = note_lock().lock().expect("note test lock");

    let mut engine = RunEngine::new(55, 20);
    engine.debug_reset_note_for_yourself_card();
    engine.debug_set_note_for_yourself_card("IronWave+");
    let deck_before = engine.run_state.deck.clone();

    advance_note_event_to_reward(&mut engine);

    let screen = engine
        .current_reward_screen()
        .expect("note event reward screen should exist");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 2);
    assert_eq!(screen.items[0].choices.len(), 1);
    assert_eq!(
        reward_choice_card_id(&screen.items[0].choices[0]),
        Some("IronWave+")
    );

    engine.step(&RunAction::SelectRewardItem(0));
    engine.step(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(engine.run_state.deck.iter().any(|card| card == "IronWave+"));

    engine.step(&RunAction::SelectRewardItem(1));
    engine.step(&RunAction::ChooseRewardOption {
        item_index: 1,
        choice_index: 0,
    });

    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.deck.len(), deck_before.len());
    assert_eq!(
        engine.run_state.deck.iter().filter(|card| card.as_str() == "Strike_P").count(),
        deck_before
            .iter()
            .filter(|card| card.as_str() == "Strike_P")
            .count()
            - 1
    );
    assert_eq!(engine.debug_current_note_for_yourself_card(), "Strike_P");

    let mut next_run = RunEngine::new(56, 20);
    next_run.debug_set_typed_event_state(typed_shrine_event("NoteForYourself"));
    next_run.step(&RunAction::EventChoice(0));
    next_run.step(&RunAction::EventChoice(0));
    let next_screen = next_run
        .current_reward_screen()
        .expect("next run note reward screen should exist");
    assert_eq!(
        reward_choice_card_id(&next_screen.items[0].choices[0]),
        Some("Strike_P")
    );

    next_run.debug_reset_note_for_yourself_card();
}

#[test]
fn note_for_yourself_can_store_the_just_claimed_note_again() {
    let _guard = note_lock().lock().expect("note test lock");

    let mut engine = RunEngine::new(57, 20);
    engine.debug_reset_note_for_yourself_card();
    engine.debug_set_note_for_yourself_card("IronWave+");
    let deck_before = engine.run_state.deck.clone();

    advance_note_event_to_reward(&mut engine);

    engine.step(&RunAction::SelectRewardItem(0));
    engine.step(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });

    let screen = engine
        .current_reward_screen()
        .expect("deck-selection reward should still be active");
    let note_choice_index = screen.items[1].choices.len() - 1;
    assert_eq!(
        reward_choice_card_id(&screen.items[1].choices[note_choice_index]),
        Some("IronWave+")
    );

    engine.step(&RunAction::SelectRewardItem(1));
    engine.step(&RunAction::ChooseRewardOption {
        item_index: 1,
        choice_index: note_choice_index,
    });

    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.deck, deck_before);
    assert_eq!(engine.debug_current_note_for_yourself_card(), "IronWave+");

    engine.debug_reset_note_for_yourself_card();
}
