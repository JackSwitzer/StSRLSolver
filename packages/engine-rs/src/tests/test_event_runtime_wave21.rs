use crate::decision::RewardChoice;
use crate::decision::RewardScreenSource;
use crate::events::{typed_shrine_events, TypedEventDef};
use crate::run::{
    ProfileSnapshot, ProfileUpdate, RunAction, RunEngine, RunPhase,
};

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/events/shrines/NoteForYourself.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java

fn typed_shrine_event(name: &str) -> TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
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
    let profile = ProfileSnapshot::with_note_for_yourself_card("IronWave+");
    let mut engine = RunEngine::new_with_profile(55, 20, profile.clone());
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
        engine.run_state.deck.iter().filter(|card| card.as_str() == "Strike").count(),
        deck_before
            .iter()
            .filter(|card| card.as_str() == "Strike")
            .count()
            - 1
    );
    assert_eq!(
        engine.profile_updates(),
        &[ProfileUpdate::StoreNoteForYourselfCard {
            card_id: "Strike".to_string(),
        }]
    );

    let mut next_profile = profile;
    for update in engine.profile_updates() {
        next_profile.apply_update(update);
    }
    let mut next_run = RunEngine::new_with_profile(56, 20, next_profile);
    next_run.debug_set_typed_event_state(typed_shrine_event("NoteForYourself"));
    next_run.step(&RunAction::EventChoice(0));
    next_run.step(&RunAction::EventChoice(0));
    let next_screen = next_run
        .current_reward_screen()
        .expect("next run note reward screen should exist");
    assert_eq!(
        reward_choice_card_id(&next_screen.items[0].choices[0]),
        Some("Strike")
    );
}

#[test]
fn note_for_yourself_can_store_the_just_claimed_note_again() {
    let profile = ProfileSnapshot::with_note_for_yourself_card("IronWave+");
    let mut engine = RunEngine::new_with_profile(57, 20, profile);
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
    assert_eq!(
        engine.profile_updates(),
        &[ProfileUpdate::StoreNoteForYourselfCard {
            card_id: "IronWave+".to_string(),
        }]
    );
}

#[test]
fn note_for_yourself_profile_inputs_are_isolated_between_simulation_roots() {
    // Java reads NOTE_CARD/NOTE_UPGRADE from the player profile when the event
    // is constructed. Rust supplies the equivalent immutable input per root so
    // concurrent rollouts cannot overwrite one another.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/events/shrines/NoteForYourself.java:88-97
    let mut iron_wave_run = RunEngine::new_with_profile(
        58,
        20,
        ProfileSnapshot::with_note_for_yourself_card("IronWave+"),
    );
    let mut defend_run = RunEngine::new_with_profile(
        59,
        20,
        ProfileSnapshot::with_note_for_yourself_card("Defend+"),
    );

    advance_note_event_to_reward(&mut iron_wave_run);
    advance_note_event_to_reward(&mut defend_run);

    let iron_wave_screen = iron_wave_run
        .current_reward_screen()
        .expect("first root should keep its own profile card");
    let defend_screen = defend_run
        .current_reward_screen()
        .expect("second root should keep its own profile card");
    assert_eq!(
        reward_choice_card_id(&iron_wave_screen.items[0].choices[0]),
        Some("IronWave+")
    );
    assert_eq!(
        reward_choice_card_id(&defend_screen.items[0].choices[0]),
        Some("Defend+")
    );
    assert!(iron_wave_run.profile_updates().is_empty());
    assert!(defend_run.profile_updates().is_empty());
}
