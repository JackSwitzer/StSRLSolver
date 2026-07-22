use crate::actions::Action;
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

fn paused_generated_turn_start_run() -> RunEngine {
    let mut engine = active_combat_run();
    {
        let combat = engine.debug_combat_engine_mut();
        // Foresight and Hello have equal Java priority. Applying Foresight
        // first makes its queued ScryAction pause before Hello's already
        // generated MakeTempCardInHandAction drains.
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::FORESIGHT, 1);
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::HELLO_WORLD, 1);
    }
    assert!(engine
        .step_game(&GameAction::CombatAction(Action::EndTurn))
        .accepted());
    let combat = engine.get_combat_engine().expect("active combat");
    assert_eq!(combat.phase, crate::engine::CombatPhase::AwaitingChoice);
    assert_eq!(
        combat.choice.as_ref().expect("Scry choice").reason,
        crate::engine::ChoiceReason::Scry
    );
    assert!(combat.pending_turn_start_resume);
    assert!(combat.turn_start_actions.iter().any(|action| matches!(
        action,
        crate::engine::TurnStartQueuedAction::AddCardToHand(_)
    )));
    engine
}

#[test]
fn checkpoint_round_trip_continues_identically_from_neow_intro() {
    let engine = RunEngine::new_with_ambient_seed(42, 0, 99);
    assert_same_continuation(engine, GameAction::Proceed);
}

#[test]
fn checkpoint_round_trip_continues_identically_from_neow_choice() {
    let mut engine = RunEngine::new_with_ambient_seed(42, 0, 99);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert_same_continuation(engine, GameAction::ChooseNeowOption(1));
}

#[test]
fn checkpoint_preserves_an_open_reward_subchoice() {
    let mut found = None;
    'seed: for seed in 0..256 {
        for neow_choice in 0..4 {
            let mut engine = RunEngine::new(seed, 0);
            assert!(engine.step_game(&GameAction::Proceed).accepted());
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
fn checkpoint_round_trip_preserves_paused_generated_turn_start_card() {
    let engine = paused_generated_turn_start_run();
    assert_same_continuation(engine, GameAction::CombatAction(Action::ConfirmSelection));
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
fn checkpoint_rejects_inconsistent_turn_start_continuation_boundaries() {
    let checkpoint = CoreCheckpoint::capture(&active_combat_run()).unwrap();
    let canonical = serde_json::to_value(checkpoint).unwrap();

    let mut collecting = canonical.clone();
    collecting["engine"]["combat_engine"]["collecting_turn_start_actions"] =
        serde_json::json!(true);
    assert!(serde_json::from_value::<CoreCheckpoint>(collecting)
        .unwrap_err()
        .to_string()
        .contains("collecting turn-start actions"));

    let mut stale_player_queue = canonical;
    stale_player_queue["engine"]["combat_engine"]["turn_start_actions"] =
        serde_json::json!(["OpeningEnergy"]);
    assert!(serde_json::from_value::<CoreCheckpoint>(stale_player_queue)
        .unwrap_err()
        .to_string()
        .contains("stale turn-start actions"));

    let checkpoint = CoreCheckpoint::capture(&paused_generated_turn_start_run()).unwrap();
    let mut missing_resume = serde_json::to_value(checkpoint).unwrap();
    missing_resume["engine"]["combat_engine"]["pending_turn_start_resume"] =
        serde_json::json!(false);
    assert!(serde_json::from_value::<CoreCheckpoint>(missing_resume)
        .unwrap_err()
        .to_string()
        .contains("turn-start actions without a resume continuation"));
}

#[test]
fn checkpoint_rejects_duplicate_independent_live_combat_identity() {
    let checkpoint = CoreCheckpoint::capture(&active_combat_run()).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    let hand_id = value["engine"]["combat_engine"]["state"]["hand"][0]["instance_id"].clone();
    value["engine"]["combat_engine"]["state"]["draw_pile"][0]["instance_id"] = hand_id;

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error.to_string().contains("independent live cards alias"));
}

#[test]
fn checkpoint_rejects_queued_turn_start_card_identity_alias() {
    let checkpoint = CoreCheckpoint::capture(&paused_generated_turn_start_run()).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    let hand_id = value["engine"]["combat_engine"]["state"]["hand"][0]["instance_id"].clone();
    let actions = value["engine"]["combat_engine"]["turn_start_actions"]
        .as_array_mut()
        .expect("turn-start action array");
    let queued = actions
        .iter_mut()
        .find_map(|action| action.get_mut("AddCardToHand"))
        .expect("queued generated card");
    queued["instance_id"] = hand_id;

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
fn checkpoint_rejects_the_pre_dynamic_power_major_version() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["schema"]["major"] = serde_json::json!(2);
    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported checkpoint schema major 2"));
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
fn checkpoint_rejects_the_prior_causal_semantics_revision() {
    let checkpoint = CoreCheckpoint::capture(&RunEngine::new(42, 0)).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    let prior = crate::checkpoint::core_semantics_fingerprint_for_revision(
        "java-rng-actions-v2-checkpoint-v6",
    );
    value["semantics_fingerprint"] = serde_json::json!(prior);

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("checkpoint semantics fingerprint mismatch"));
}

#[test]
fn checkpoint_requires_complete_valid_power_order() {
    let mut engine = active_combat_run();
    engine
        .debug_combat_engine_mut()
        .state
        .player
        .set_status(crate::status_ids::sid::STRENGTH, 2);
    let checkpoint = CoreCheckpoint::capture(&engine).unwrap();
    let canonical = serde_json::to_value(checkpoint).unwrap();

    let mut missing_field = canonical.clone();
    missing_field["engine"]["combat_engine"]["state"]["player"]
        .as_object_mut()
        .unwrap()
        .remove("power_order");
    assert!(serde_json::from_value::<CoreCheckpoint>(missing_field).is_err());

    let mut duplicate = canonical.clone();
    let order = duplicate["engine"]["combat_engine"]["state"]["player"]["power_order"]
        .as_array_mut()
        .unwrap();
    order.push(order[0].clone());
    assert!(serde_json::from_value::<CoreCheckpoint>(duplicate)
        .unwrap_err()
        .to_string()
        .contains("duplicate status id"));

    let mut missing_active = canonical;
    missing_active["engine"]["combat_engine"]["state"]["player"]["power_order"]
        .as_array_mut()
        .unwrap()
        .clear();
    assert!(serde_json::from_value::<CoreCheckpoint>(missing_active)
        .unwrap_err()
        .to_string()
        .contains("missing from power order"));
}

#[test]
fn checkpoint_rejects_stasis_power_without_its_held_card_payload() {
    let mut engine = active_combat_run();
    {
        let combat = engine.debug_combat_engine_mut();
        let card = combat.state.draw_pile.pop().expect("fixture draw card");
        combat.state.enemies[0].set_stasis_card(card);
    }
    let checkpoint = CoreCheckpoint::capture(&engine).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["engine"]["combat_engine"]["state"]["enemies"][0]["stasis_card"] =
        serde_json::Value::Null;

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("Stasis card and power presence disagree"));
}

#[test]
fn checkpoint_rejects_the_bomb_power_without_its_dynamic_payload() {
    let mut engine = active_combat_run();
    {
        let combat = engine.debug_combat_engine_mut();
        combat.restore_the_bomb_id_offset(19);
        combat.schedule_the_bomb(3, 40);
    }
    CoreCheckpoint::capture(&engine).expect("consistent The Bomb state");

    engine.debug_combat_engine_mut().state.pending_bombs.clear();
    let error = CoreCheckpoint::capture(&engine).unwrap_err();
    assert!(error
        .to_string()
        .contains("The Bomb power order/payload state disagrees"));
}

#[test]
fn checkpoint_rejects_pen_nib_counter_without_its_semantic_power() {
    let mut engine = active_combat_run();
    {
        let combat = engine.debug_combat_engine_mut();
        combat.state.relics.push("Pen Nib".to_string());
        combat.rebuild_effect_runtime();
        let owner = crate::effects::runtime::EffectOwner::PlayerRelic {
            slot: (combat.state.relics.len() - 1) as u16,
        };
        assert!(combat.set_hidden_effect_value("Pen Nib", owner, 0, 9));
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::PEN_NIB_COUNTER, 9);
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::PEN_NIB_POWER, 1);
    }
    CoreCheckpoint::capture(&engine).expect("consistent Pen Nib state");

    engine
        .debug_combat_engine_mut()
        .state
        .player
        .set_status(crate::status_ids::sid::PEN_NIB_POWER, 0);
    let error = CoreCheckpoint::capture(&engine).unwrap_err();
    assert!(error
        .to_string()
        .contains("Pen Nib counter/power state disagrees"));
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
    assert!(error
        .to_string()
        .contains("checkpoint causal hash mismatch"));
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
fn checkpoint_rejects_an_invalid_neow_selection_witness() {
    let mut engine = RunEngine::new(42, 0);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert!(engine
        .step_game(&GameAction::ChooseNeowOption(1))
        .accepted());
    let checkpoint = CoreCheckpoint::capture(&engine).unwrap();
    let mut value = serde_json::to_value(checkpoint).unwrap();
    value["engine"]["neow_selection_witness"]["selected_index"] = serde_json::json!(4);

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("Neow selection witness index is out of bounds"));
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
    let first_id = value["engine"]["run_state"]["deck_card_states"][0]["instance_id"].clone();
    value["engine"]["run_state"]["deck_card_states"][1]["instance_id"] = first_id;

    let error = serde_json::from_value::<CoreCheckpoint>(value).unwrap_err();
    assert!(error
        .to_string()
        .contains("master-deck card instance ids must be unique"));
}
