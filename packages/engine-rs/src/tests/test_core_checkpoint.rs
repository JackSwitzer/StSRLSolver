use crate::checkpoint::{CoreCheckpoint, CORE_CHECKPOINT_MAJOR};
use crate::map::RoomType;
use crate::relic_flags::counter;
use crate::run::{GameAction, RunEngine, RunPhase};
use crate::tests::support::resolve_opening_neow;

fn round_trip(engine: &RunEngine) -> RunEngine {
    let checkpoint = CoreCheckpoint::capture(engine).expect("decision boundary is quiescent");
    let json = checkpoint.to_json().expect("checkpoint must serialize");
    let decoded = CoreCheckpoint::from_json(&json).expect("checkpoint must deserialize");
    assert_eq!(decoded, checkpoint);
    decoded.restore().expect("checkpoint must restore")
}

fn assert_same_continuation(mut original: RunEngine, action: GameAction) {
    let mut restored = round_trip(&original);
    assert_eq!(restored.get_legal_actions(), original.get_legal_actions());

    let original_outcome = original.step_game(&action);
    let restored_outcome = restored.step_game(&action);
    assert_eq!(restored_outcome.status, original_outcome.status);
    assert_eq!(restored_outcome.terminal, original_outcome.terminal);
    assert_eq!(
        restored_outcome.next_decision.legal_actions,
        original_outcome.next_decision.legal_actions
    );

    let original_post = CoreCheckpoint::capture(&original).unwrap();
    let restored_post = CoreCheckpoint::capture(&restored).unwrap();
    assert_eq!(restored_post, original_post);
}

fn active_combat_run() -> RunEngine {
    let mut engine = RunEngine::new_with_ambient_seed(2024, 0, 77);
    resolve_opening_neow(&mut engine);
    let (x, y) = {
        let node = engine.map.get_start_nodes()[0];
        (node.x, node.y)
    };
    engine.map.rows[y][x].room_type = RoomType::Monster;
    engine.step_game(&GameAction::ChoosePath(0));
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    engine
}

#[test]
fn checkpoint_round_trip_continues_identically_from_neow() {
    let engine = RunEngine::new_with_ambient_seed(42, 0, 99);
    assert_same_continuation(engine, GameAction::ChooseNeowOption(1));
}

#[test]
fn checkpoint_preserves_an_open_reward_subchoice() {
    let mut found = None;
    'seed: for seed in 0..256 {
        for neow_choice in 0..4 {
            let mut engine = RunEngine::new(seed, 0);
            engine.step_game(&GameAction::ChooseNeowOption(neow_choice));
            if engine.current_phase() != RunPhase::CardReward {
                continue;
            }
            let Some(select) = engine
                .get_legal_actions()
                .into_iter()
                .find(|action| matches!(action, GameAction::SelectRewardItem(_)))
            else {
                continue;
            };
            engine.step_game(&select);
            if let Some(choice) = engine
                .get_legal_actions()
                .into_iter()
                .find(|action| matches!(action, GameAction::ChooseRewardOption { .. }))
            {
                found = Some((engine, choice));
                break 'seed;
            }
        }
    }

    let (engine, choice) = found.expect("a seeded Neow reward subchoice must exist");
    assert_same_continuation(engine, choice);
}

#[test]
fn checkpoint_round_trip_preserves_active_combat_and_all_combat_rngs() {
    let engine = active_combat_run();

    let action = engine
        .get_legal_actions()
        .into_iter()
        .find(|action| matches!(action, GameAction::CombatAction(_)))
        .expect("combat must expose an action");
    assert_same_continuation(engine, action);
}

#[test]
fn checkpoint_rejects_inconsistent_combat_decision_boundaries() {
    let checkpoint = CoreCheckpoint::capture(&active_combat_run()).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["engine"]["combat_engine"]["phase"] = serde_json::json!("AwaitingChoice");
    value["engine"]["combat_engine"]["choice"] = serde_json::Value::Null;

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("awaiting-choice combat is missing its choice payload"));
}

#[test]
fn checkpoint_rejects_duplicate_independent_live_combat_identity() {
    let checkpoint = CoreCheckpoint::capture(&active_combat_run()).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    let hand_id = value["engine"]["combat_engine"]["state"]["hand"][0]["instance_id"]
        .clone();
    value["engine"]["combat_engine"]["state"]["draw_pile"][0]["instance_id"] = hand_id;

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error.to_string().contains("independent live cards alias"));
}

#[test]
fn checkpoint_preserves_duplicate_mutable_cards_and_relic_counters() {
    let mut engine = RunEngine::new(7, 0);
    let registry = crate::cards::global_registry();
    let mut first = registry
        .make_card("Genetic Algorithm")
        .with_instance_id(100);
    first.misc = 11;
    let mut second = registry
        .make_card("Genetic Algorithm")
        .with_instance_id(101);
    second.misc = 23;
    engine.run_state.deck = vec!["Genetic Algorithm".into(), "Genetic Algorithm".into()];
    engine.run_state.deck_card_states = vec![first, second];
    engine.run_state.next_card_instance_id = 102;
    engine.run_state.relic_flags.counters[counter::NUNCHAKU] = 9;
    engine.run_state.relic_flags.counters[counter::NEOWS_LAMENT] = 1;

    let restored = round_trip(&engine);
    assert_eq!(restored.run_state.deck_card_states[0].misc, 11);
    assert_eq!(restored.run_state.deck_card_states[1].misc, 23);
    assert_ne!(
        restored.run_state.deck_card_states[0].instance_id,
        restored.run_state.deck_card_states[1].instance_id
    );
    assert_eq!(
        restored.run_state.relic_flags.counters[counter::NUNCHAKU],
        9
    );
    assert_eq!(
        restored.run_state.relic_flags.counters[counter::NEOWS_LAMENT],
        1
    );
}

#[test]
fn rejected_action_has_identical_pre_and_post_checkpoint() {
    let mut engine = RunEngine::new(42, 0);
    let before = CoreCheckpoint::capture(&engine).unwrap();
    let outcome = engine.step_game(&GameAction::ShopLeave);
    let after = CoreCheckpoint::capture(&engine).unwrap();
    assert_eq!(outcome.status, crate::run::ActionStatus::Rejected);
    assert_eq!(after, before);
}

#[test]
fn checkpoint_rejects_an_unknown_major_version() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["schema"]["major"] = serde_json::json!(CORE_CHECKPOINT_MAJOR + 1);
    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported checkpoint schema major"));
}

#[test]
fn checkpoint_rejects_an_unknown_future_minor() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["schema"]["minor"] = serde_json::json!(1);
    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported checkpoint schema minor"));
}

#[test]
fn checkpoint_rejects_a_different_core_semantics_fingerprint() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["semantics_fingerprint"] = serde_json::json!("different-ruleset");

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("checkpoint semantics fingerprint mismatch"));
}

#[test]
fn checkpoint_rejects_missing_master_deck_state_before_normalization() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["engine"]["run_state"]
        .as_object_mut()
        .unwrap()
        .remove("deck_card_states");

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("master-deck names and instances are misaligned"));
}

#[test]
fn checkpoint_rejects_silently_defaultable_causal_field_loss() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["engine"]["run_state"]
        .as_object_mut()
        .unwrap()
        .remove("event_monster_chance");

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("complete canonical causal representation"));
}

#[test]
fn checkpoint_rejects_valid_state_edits_without_a_matching_hash() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["engine"]["run_state"]["gold"] = serde_json::json!(1234);

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error.to_string().contains("checkpoint causal hash mismatch"));
}

#[test]
fn checkpoint_rejects_unknown_wire_fields() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["unexpected"] = serde_json::json!(true);

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn checkpoint_rejects_missing_phase_owned_payloads() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["engine"]["phase"] = serde_json::json!("Shop");
    value["engine"]["current_shop"] = serde_json::Value::Null;

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error.to_string().contains("missing its shop inventory"));
}

#[test]
fn card_instance_allocator_exhausts_instead_of_reusing_an_identity() {
    let mut state = crate::state::CombatState::new(80, 80, Vec::new(), Vec::new(), 3);
    state.next_card_instance_id = u64::from(u32::MAX);
    assert_eq!(state.allocate_card_instance_id(), u32::MAX);
    let exhausted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.allocate_card_instance_id()
    }));
    assert!(exhausted.is_err());
}

#[test]
fn checkpoint_rejects_duplicate_master_card_identity() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    let first_id = value["engine"]["run_state"]["deck_card_states"][0]["instance_id"]
        .clone();
    value["engine"]["run_state"]["deck_card_states"][1]["instance_id"] = first_id;

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("master-deck card instance ids must be unique"));
}
