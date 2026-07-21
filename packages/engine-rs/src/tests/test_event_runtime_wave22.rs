use crate::checkpoint::CoreCheckpoint;
use crate::events::{typed_events_for_act, typed_shrine_events, EventRuntimeStatus, TypedEventDef};
use crate::run::{GameAction, RunEngine, RunPhase};

fn act_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing event {name}"))
}

fn shrine_event(name: &str) -> TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing shrine {name}"))
}

fn relevant_rng(engine: &RunEngine) -> (i64, i64, i64) {
    let counters = engine.rng_counters();
    (counters["card"], counters["cardRandom"], counters["misc"])
}

#[test]
fn vampires_replaces_every_starter_strike_with_five_fixed_bites_without_rng() {
    // Vampires.replaceAttacks removes every STARTER_STRIKE-tagged card, then
    // obtains exactly five Bites. The normal choice loses ceil(30% max HP).
    // Java: events/city/Vampires.java::replaceAttacks/buttonEffect.
    let mut engine = RunEngine::new(731, 0);
    engine.run_state.deck = vec![
        "Strike_P".to_string(),
        "Strike_P+".to_string(),
        "Defend_P".to_string(),
        "Eruption".to_string(),
        "Perfected Strike".to_string(),
    ];
    let rng_before = relevant_rng(&engine);
    engine.debug_set_typed_event_state(act_event(2, "Vampires"));

    assert!(matches!(
        engine.debug_current_event().expect("Vampires").options[1].status,
        EventRuntimeStatus::Blocked { .. }
    ));
    assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());

    assert_eq!(
        (engine.run_state.max_hp, engine.run_state.current_hp),
        (50, 50)
    );
    assert_eq!(
        engine
            .run_state
            .deck
            .iter()
            .filter(|card| card.as_str() == "Bite")
            .count(),
        5
    );
    assert!(!engine
        .run_state
        .deck
        .iter()
        .any(|card| matches!(card.as_str(), "Strike_P" | "Strike_P+")));
    assert!(engine.run_state.deck.iter().any(|card| card == "Defend_P"));
    assert!(engine
        .run_state
        .deck
        .iter()
        .any(|card| card == "Perfected Strike"));
    assert_eq!(relevant_rng(&engine), rng_before);
}

#[test]
fn vampires_blood_vial_choice_is_legal_only_when_owned_and_preserves_max_hp() {
    // Java adds the Blood Vial option conditionally. It removes that relic and
    // performs the same deterministic strike replacement without max-HP loss.
    let mut engine = RunEngine::new(733, 0);
    engine.run_state.deck = vec!["Strike_P".to_string(), "Defend_P".to_string()];
    engine.run_state.relics.push("Blood Vial".to_string());
    engine.debug_set_typed_event_state(act_event(2, "Vampires"));

    assert!(engine
        .get_legal_actions()
        .contains(&GameAction::EventChoice(1)));
    assert!(engine.step_game(&GameAction::EventChoice(1)).accepted());
    assert_eq!(engine.run_state.max_hp, 72);
    assert!(!engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "Blood Vial"));
    assert_eq!(
        engine
            .run_state
            .deck
            .iter()
            .filter(|card| card.as_str() == "Bite")
            .count(),
        5
    );
}

#[test]
fn council_of_ghosts_grants_fixed_apparitions_without_card_rng() {
    // Ghosts.becomeGhost obtains five Apparitions below A15 and three at A15;
    // it does not create card-reward screens or draw card RNG.
    // Java: events/city/Ghosts.java::becomeGhost.
    for (ascension, expected_count) in [(0, 5), (15, 3)] {
        let mut engine = RunEngine::new(739, ascension);
        let initial_max_hp = engine.run_state.max_hp;
        let rng_before = relevant_rng(&engine);
        engine.debug_set_typed_event_state(act_event(2, "Council of Ghosts"));

        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());

        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
        let expected_hp = initial_max_hp / 2;
        assert_eq!(
            (engine.run_state.max_hp, engine.run_state.current_hp),
            (expected_hp, expected_hp)
        );
        assert_eq!(
            engine
                .run_state
                .deck
                .iter()
                .filter(|card| card.as_str() == "Apparition")
                .count(),
            expected_count
        );
        assert_eq!(relevant_rng(&engine), rng_before);
    }
}

#[test]
fn duplicator_and_transmogrifier_are_agent_selected_card_decisions() {
    // Duplicator copies the selected card with no RNG. Transmogrifier removes
    // the selected purgeable, non-bottled card and uses exactly one miscRng
    // draw for the transformed identity.
    // Java: events/shrines/Duplicator.java and Transmogrifier.java.
    let mut duplicate = RunEngine::new(743, 0);
    duplicate.run_state.deck = vec!["Eruption+".to_string(), "Vigilance".to_string()];
    let duplicate_rng = relevant_rng(&duplicate);
    duplicate.debug_set_typed_event_state(shrine_event("Duplicator"));
    assert!(duplicate.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(duplicate.current_phase(), RunPhase::CardReward);
    assert!(duplicate
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert!(duplicate
        .step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .accepted());
    assert_eq!(
        duplicate
            .run_state
            .deck
            .iter()
            .filter(|card| card.as_str() == "Eruption+")
            .count(),
        2
    );
    assert_eq!(relevant_rng(&duplicate), duplicate_rng);

    let mut transform = RunEngine::new(751, 0);
    transform.run_state.deck = vec!["Eruption".to_string(), "Vigilance".to_string()];
    let transform_rng = relevant_rng(&transform);
    transform.debug_set_typed_event_state(shrine_event("Transmorgrifier"));
    assert!(transform.step_game(&GameAction::EventChoice(0)).accepted());
    assert!(transform
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert!(transform
        .step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .accepted());
    let after = relevant_rng(&transform);
    assert_eq!(after.0, transform_rng.0);
    assert_eq!(after.1, transform_rng.1);
    assert_eq!(after.2, transform_rng.2 + 1);
    assert_eq!(transform.run_state.deck.len(), 2);
    assert!(!transform
        .run_state
        .deck
        .iter()
        .any(|card| card == "Eruption"));
}

#[test]
fn single_card_event_mutations_use_the_canonical_agent_choice_surface() {
    // Purifier and Upgrade Shrine open GridCardSelectScreen over the eligible
    // master-deck cards. The selected identity, not an RNG-picked proxy, owns
    // the mutation.
    // Java: events/shrines/Purifier.java and UpgradeShrine.java.
    let mut remove = RunEngine::new(757, 0);
    remove.run_state.deck = vec!["Eruption".to_string(), "Vigilance".to_string()];
    let remove_rng = relevant_rng(&remove);
    remove.debug_set_typed_event_state(shrine_event("Purifier"));
    assert!(remove.step_game(&GameAction::EventChoice(0)).accepted());
    assert!(remove
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert!(remove
        .step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 1,
        })
        .accepted());
    assert_eq!(remove.run_state.deck, vec!["Eruption"]);
    assert_eq!(relevant_rng(&remove), remove_rng);

    let mut upgrade = RunEngine::new(761, 0);
    upgrade.run_state.deck = vec!["Eruption".to_string(), "Vigilance".to_string()];
    let upgrade_rng = relevant_rng(&upgrade);
    upgrade.debug_set_typed_event_state(shrine_event("Upgrade Shrine"));
    assert!(upgrade.step_game(&GameAction::EventChoice(0)).accepted());
    assert!(upgrade
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert!(upgrade
        .step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 1,
        })
        .accepted());
    assert_eq!(upgrade.run_state.deck, vec!["Eruption", "Vigilance+"]);
    assert_eq!(relevant_rng(&upgrade), upgrade_rng);
}

#[test]
fn multi_card_event_selection_checkpoint_preserves_remaining_choice_count() {
    // DrugDealer opens a two-card transform grid. The canonical action layer
    // resolves it as two persisted selections without allowing a repeated
    // removed identity or fabricating a random deck target.
    let mut original = RunEngine::new(769, 0);
    original.debug_set_typed_event_state(act_event(2, "Drug Dealer"));
    assert!(original.step_game(&GameAction::EventChoice(1)).accepted());
    assert!(original
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert!(original
        .step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .accepted());
    assert_eq!(original.current_phase(), RunPhase::CardReward);

    let checkpoint = CoreCheckpoint::capture(&original).expect("event selection checkpoint");
    let mut restored = checkpoint.restore().expect("restore event selection");
    assert_eq!(restored.get_legal_actions(), original.get_legal_actions());

    for engine in [&mut original, &mut restored] {
        assert!(engine
            .step_game(&GameAction::SelectRewardItem(0))
            .accepted());
        assert!(engine
            .step_game(&GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 0,
            })
            .accepted());
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    }
    assert_eq!(
        CoreCheckpoint::capture(&restored).unwrap(),
        CoreCheckpoint::capture(&original).unwrap()
    );
}
