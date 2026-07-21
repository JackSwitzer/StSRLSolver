use crate::run::{GameAction, RunEngine};
use crate::trace::oracle_v2::{
    diff_oracle_states, diff_partial_oracle_state, project_oracle_state, OracleOrbStateV2,
    OraclePowerStateV2, OracleStateV2,
};

fn combat_state() -> OracleStateV2 {
    let mut engine = RunEngine::new(4, 0);
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert!(engine
        .step_game(&GameAction::ChooseNeowOption(1))
        .accepted());
    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert!(engine.step_game(&GameAction::ChoosePath(0)).accepted());
    project_oracle_state(&engine).expect("combat state must project")
}

fn assert_mutation_path(
    expected_path: &str,
    prepare: impl FnOnce(&mut OracleStateV2),
    mutate: impl FnOnce(&mut OracleStateV2),
) {
    let mut expected = combat_state();
    prepare(&mut expected);
    let mut actual = expected.clone();
    mutate(&mut actual);
    let diffs = diff_oracle_states(&expected, &actual);
    assert_eq!(
        diffs.first().map(|diff| diff.path.as_str()),
        Some(expected_path),
        "unexpected mutation report: {diffs:?}"
    );
}

#[test]
fn oracle_state_v2_projects_complete_run_combat_and_rng_state() {
    let state = combat_state();
    state.validate().unwrap();
    assert_eq!(state.floor, 1);
    assert_eq!(state.act, 1);
    assert_eq!(state.turn, 1);
    assert_eq!(state.phase, "COMBAT");
    assert_eq!(state.deck.len(), 10);
    assert_eq!(
        &state.deck[..5],
        &["Strike_P", "Strike_P", "Strike_P", "Strike_P", "Defend_P"]
    );
    assert!(!state.enemies.is_empty());
    assert_eq!(state.rng.monster_hp, 1);

    let encoded = serde_json::to_value(&state).unwrap();
    let rng = encoded["rng"].as_object().unwrap();
    assert_eq!(rng.len(), 16);
    assert!(rng.contains_key("monsterHp"));
    assert!(rng.contains_key("cardRandom"));
    assert!(rng.contains_key("ambientMath"));
    assert!(rng.contains_key("javaCollections"));
    assert!(rng.contains_key("rawStates"));
    assert_eq!(state.rng.raw_states.len(), 14);
    let decoded: OracleStateV2 = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, state);
}

#[test]
fn oracle_intent_damage_keeps_java_float_order_until_final_floor() {
    // AbstractMonster.calculateDamage: enemy Weak -> player Vulnerable ->
    // stance -> BackAttack cast -> final modifiers (Intangible) -> floor.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
    let mut engine = RunEngine::new(91, 0);
    engine.debug_enter_specific_combat(&["JawWorm"]);
    {
        let combat = engine.debug_combat_engine_mut();
        combat.state.enemies[0].set_move(1, 5, 1, 0);
        combat.state.enemies[0]
            .entity
            .set_status(crate::status_ids::sid::WEAKENED, 1);
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::VULNERABLE, 1);
    }
    let projected = project_oracle_state(&engine).unwrap();
    assert_eq!(projected.enemies[0].intent.dmg, 5);

    {
        let combat = engine.debug_combat_engine_mut();
        combat.state.enemies[0].back_attack = true;
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::INTANGIBLE, 1);
    }
    let projected = project_oracle_state(&engine).unwrap();
    assert_eq!(projected.enemies[0].intent.dmg, 1);
}

#[test]
fn oracle_state_v2_rejects_a_missing_rng_stream() {
    let mut encoded = serde_json::to_value(combat_state()).unwrap();
    encoded["rng"].as_object_mut().unwrap().remove("merchant");
    let error = serde_json::from_value::<OracleStateV2>(encoded).unwrap_err();
    assert!(error.to_string().contains("merchant"));
}

#[test]
fn oracle_state_v2_requires_and_round_trips_process_global_rng_states() {
    let ambient = (0x0123_4567_89AB_CDEF, 0xFEDC_BA98_7654_3210);
    let collections = 0x1234_5678_9ABC;
    let engine = RunEngine::new_with_ambient_states(4, 0, ambient, collections);
    let state = project_oracle_state(&engine).unwrap();

    assert_eq!(state.rng.ambient_math.seed0, "0123456789abcdef");
    assert_eq!(state.rng.ambient_math.seed1, "fedcba9876543210");
    assert_eq!(state.rng.java_collections, "123456789abc");

    let encoded = serde_json::to_value(&state).unwrap();
    assert_eq!(
        serde_json::from_value::<OracleStateV2>(encoded.clone()).unwrap(),
        state
    );

    let mut missing_math = encoded.clone();
    missing_math["rng"]
        .as_object_mut()
        .unwrap()
        .remove("ambientMath");
    assert!(serde_json::from_value::<OracleStateV2>(missing_math)
        .unwrap_err()
        .to_string()
        .contains("ambientMath"));

    let mut missing_collections = encoded;
    missing_collections["rng"]
        .as_object_mut()
        .unwrap()
        .remove("javaCollections");
    assert!(serde_json::from_value::<OracleStateV2>(missing_collections)
        .unwrap_err()
        .to_string()
        .contains("javaCollections"));
}

#[test]
fn oracle_state_v2_rejects_missing_or_divergent_raw_stream_state() {
    let state = combat_state();
    let mut missing = serde_json::to_value(&state).unwrap();
    missing["rng"]["rawStates"]
        .as_object_mut()
        .unwrap()
        .remove("shuffle");
    assert!(serde_json::from_value::<OracleStateV2>(missing)
        .unwrap_err()
        .to_string()
        .contains("shuffle"));

    let mut divergent = state.clone();
    let card = divergent.rng.raw_states.get_mut("card").unwrap();
    let unchanged_counter = card.counter;
    card.seed0 = "0000000000000000".to_string();
    assert_eq!(card.counter, unchanged_counter);
    assert_eq!(
        diff_oracle_states(&state, &divergent)[0].path,
        "rng.rawStates.card.seed0"
    );

    let mut inconsistent = serde_json::to_value(&state).unwrap();
    inconsistent["rng"]["rawStates"]["card"]["counter"] = serde_json::json!(999);
    assert!(serde_json::from_value::<OracleStateV2>(inconsistent)
        .unwrap_err()
        .to_string()
        .contains("disagrees"));
}

#[test]
fn oracle_state_v2_requires_and_validates_schema_during_deserialization() {
    let encoded = serde_json::to_value(combat_state()).unwrap();

    let mut missing = encoded.clone();
    missing.as_object_mut().unwrap().remove("schema");
    assert!(serde_json::from_value::<OracleStateV2>(missing)
        .unwrap_err()
        .to_string()
        .contains("schema"));

    let mut wrong_major = encoded.clone();
    wrong_major["schema"]["major"] = serde_json::json!(3);
    assert!(serde_json::from_value::<OracleStateV2>(wrong_major)
        .unwrap_err()
        .to_string()
        .contains("major"));

    let current_minor = encoded["schema"]["minor"]
        .as_u64()
        .expect("projected schema minor must be numeric");
    let mut future_minor = encoded;
    future_minor["schema"]["minor"] = serde_json::json!(current_minor + 1);
    assert!(serde_json::from_value::<OracleStateV2>(future_minor)
        .unwrap_err()
        .to_string()
        .contains("minor"));
}

#[test]
fn oracle_state_v2_diff_covers_every_required_state_family() {
    assert_mutation_path("rng.ai", |_| {}, |state| state.rng.ai += 1);
    assert_mutation_path(
        "rng.ambientMath.seed0",
        |_| {},
        |state| state.rng.ambient_math.seed0 = "0000000000000000".to_string(),
    );
    assert_mutation_path(
        "rng.javaCollections",
        |_| {},
        |state| state.rng.java_collections = "000000000000".to_string(),
    );
    assert_mutation_path(
        "rng.rawStates.card.seed1",
        |_| {},
        |state| {
            state.rng.raw_states.get_mut("card").unwrap().seed1 = "0000000000000000".to_string()
        },
    );
    assert_mutation_path("floor", |_| {}, |state| state.floor += 1);
    assert_mutation_path("act", |_| {}, |state| state.act += 1);
    assert_mutation_path("turn", |_| {}, |state| state.turn += 1);
    assert_mutation_path("phase", |_| {}, |state| state.phase.push_str("_BAD"));
    assert_mutation_path("map.x", |_| {}, |state| state.map.x += 1);
    assert_mutation_path("keys.ruby", |_| {}, |state| state.keys.ruby = true);
    assert_mutation_path("player.hp", |_| {}, |state| state.player.hp -= 1);
    assert_mutation_path(
        "player.powers[0].amt",
        |state| {
            state.player.powers = vec![OraclePowerStateV2 {
                id: "Strength".to_string(),
                amt: 2,
            }]
        },
        |state| state.player.powers[0].amt = 3,
    );
    assert_mutation_path(
        "player.orbs[0].passive_amount",
        |state| {
            state.player.orbs = vec![OracleOrbStateV2 {
                id: "Lightning".to_string(),
                evoke_amount: 8,
                passive_amount: 3,
            }]
        },
        |state| state.player.orbs[0].passive_amount = 4,
    );
    assert_mutation_path(
        "enemies[0].dead",
        |_| {},
        |state| state.enemies[0].dead = !state.enemies[0].dead,
    );
    assert_mutation_path(
        "enemies[0].intent.name",
        |_| {},
        |state| state.enemies[0].intent.name.push_str("_BAD"),
    );
    assert_mutation_path(
        "enemies[0].powers[0].id",
        |state| {
            state.enemies[0].powers = vec![OraclePowerStateV2 {
                id: "Ritual".to_string(),
                amt: 3,
            }]
        },
        |state| state.enemies[0].powers[0].id = "Strength".to_string(),
    );
    assert_mutation_path(
        "enemies[0].move_history[0]",
        |state| state.enemies[0].move_history = vec![3],
        |state| state.enemies[0].move_history[0] += 1,
    );
    assert_mutation_path(
        "piles.draw_ordered[0]",
        |_| {},
        |state| state.piles.draw_ordered[0].push_str("+"),
    );
    assert_mutation_path("deck[0]", |_| {}, |state| state.deck[0].push_str("+"));
    assert_mutation_path(
        "relics[0].counter",
        |_| {},
        |state| state.relics[0].counter += 1,
    );
    assert_mutation_path(
        "potions[0]",
        |_| {},
        |state| state.potions[0] = "EnergyPotion".to_string(),
    );
}

#[test]
fn oracle_state_v2_exposes_all_four_semantic_neow_payloads() {
    let mut engine = RunEngine::new(4, 0);
    let intro = project_oracle_state(&engine).unwrap();
    assert!(intro.neow.expect("Neow intro witness").options.is_empty());
    assert!(engine.step_game(&GameAction::Proceed).accepted());

    let choice = project_oracle_state(&engine).unwrap();
    let neow = choice
        .neow
        .expect("Neow state must be visible at selection");
    assert_eq!(neow.mode, "four_choices");
    assert_eq!(neow.options.len(), 4);
    for (category, option) in neow.options.iter().enumerate() {
        assert_eq!(option.category, category as u8);
        assert!(!option.reward_id.is_empty());
        assert!(!option.drawback_id.is_empty());
    }

    let expected_selected = neow.options[1].clone();
    assert!(engine
        .step_game(&GameAction::ChooseNeowOption(1))
        .accepted());
    let expected = project_oracle_state(&engine).unwrap();
    let selected = expected
        .neow
        .as_ref()
        .and_then(|neow| neow.selected.as_ref())
        .expect("post-selection state must retain the semantic Neow witness");
    assert_eq!(selected, &expected_selected);
    assert_eq!(expected.neow.as_ref().unwrap().options, neow.options);

    assert!(engine.step_game(&GameAction::Proceed).accepted());
    assert!(project_oracle_state(&engine).unwrap().neow.is_none());

    let mut actual = expected.clone();
    actual
        .neow
        .as_mut()
        .unwrap()
        .selected
        .as_mut()
        .unwrap()
        .reward_id = "UPGRADE_CARD".to_string();
    assert_eq!(
        diff_oracle_states(&expected, &actual)[0].path,
        "neow.selected.reward_id"
    );
}

#[test]
fn partial_oracle_diff_counts_absent_fields_and_still_prioritizes_rng() {
    let actual = combat_state();
    let mut partial = serde_json::to_value(&actual).unwrap();
    let object = partial.as_object_mut().unwrap();
    object.remove("schema");
    object.remove("keys");
    object.remove("neow");
    partial["rng"]["ai"] = serde_json::json!(actual.rng.ai + 1);

    let result = diff_partial_oracle_state(&partial, &actual);
    assert_eq!(result.diffs[0].path, "rng.ai");
    assert_eq!(result.skipped_fields_total(), 6);
    assert_eq!(result.skipped_fields_by_path["keys.ruby"], 1);
    assert_eq!(result.skipped_fields_by_path["schema.major"], 1);
}
