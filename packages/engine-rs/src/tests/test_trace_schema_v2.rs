use crate::actions::Action;
use crate::checkpoint::CoreCheckpoint;
use crate::run::GameAction;
use crate::run::RunEngine;
use crate::trace::v2::{
    replay_action_script_v2, ActionOutcome, ActionScriptV2, Capabilities, EndV2, HeaderV2,
    Producer, SchemaVersion, TraceEnvelopeV2, TracePayloadV2, TransitionV2, TRACE_SCHEMA_MAJOR,
    TRACE_SCHEMA_MINOR, TRACE_SCHEMA_NAME,
};
use serde_json::{json, Value};

fn producer() -> Producer {
    Producer {
        name: "engine-rs".to_string(),
        version: "test".to_string(),
        git_commit: Some("deadbeef".to_string()),
    }
}

fn capabilities() -> Capabilities {
    Capabilities(vec![
        "direct_game_action".to_string(),
        "action_outcomes".to_string(),
        "core_checkpoints".to_string(),
    ])
}

fn checkpoint(seed: u64) -> CoreCheckpoint {
    CoreCheckpoint::capture(&RunEngine::new(seed, 0)).expect("public decision is quiescent")
}

fn all_current_game_actions() -> Vec<GameAction> {
    vec![
        GameAction::ChooseNeowOption(1),
        GameAction::ChoosePath(2),
        GameAction::OpenChest,
        GameAction::LeaveChest,
        GameAction::SelectRewardItem(3),
        GameAction::ChooseRewardOption {
            item_index: 4,
            choice_index: 5,
        },
        GameAction::SkipRewardItem(6),
        GameAction::LeaveRewards,
        GameAction::Proceed,
        GameAction::CampfireRest,
        GameAction::CampfireUpgrade(7),
        GameAction::CampfireToke,
        GameAction::CampfireLift,
        GameAction::CampfireDig,
        GameAction::CampfireRecall,
        GameAction::ShopBuyCard(8),
        GameAction::ShopBuyRelic(9),
        GameAction::ShopBuyPotion(10),
        GameAction::ShopRemoveCard(11),
        GameAction::ShopLeave,
        GameAction::EventChoice(12),
        GameAction::CombatAction(Action::PlayCard {
            card_idx: 13,
            target_idx: 1,
        }),
        GameAction::CombatAction(Action::UsePotion {
            potion_idx: 2,
            target_idx: -1,
        }),
        GameAction::CombatAction(Action::EndTurn),
        GameAction::CombatAction(Action::Choose(14)),
        GameAction::CombatAction(Action::ConfirmSelection),
        GameAction::UsePotion(15),
        GameAction::DiscardPotion(16),
    ]
}

fn tiny_action_script() -> ActionScriptV2 {
    ActionScriptV2 {
        schema: SchemaVersion::current(),
        trace_id: "watcher-a0-v2-smoke".to_string(),
        seed: crate::seed::seed_to_string(4),
        seed_long: 4,
        character: "WATCHER".to_string(),
        ascension: 0,
        actions: vec![
            GameAction::Proceed,
            GameAction::ChooseNeowOption(1),
            GameAction::Proceed,
            GameAction::ChoosePath(0),
            GameAction::CombatAction(Action::PlayCard {
                card_idx: 3,
                target_idx: -1,
            }),
            GameAction::CombatAction(Action::EndTurn),
        ],
    }
}

fn transition_envelope(action: GameAction, outcome: ActionOutcome) -> TraceEnvelopeV2 {
    let pre = checkpoint(41);
    let post = if matches!(&outcome, ActionOutcome::Rejected { .. }) {
        pre.clone()
    } else {
        checkpoint(42)
    };
    TraceEnvelopeV2::new(
        capabilities(),
        producer(),
        TracePayloadV2::Transition(TransitionV2 {
            idx: 42,
            action,
            outcome,
            pre,
            post,
        }),
    )
}

#[test]
fn every_current_game_action_round_trips_directly_through_v2() {
    let actions = all_current_game_actions();
    assert!(actions.contains(&GameAction::Proceed));
    assert!(actions.contains(&GameAction::DiscardPotion(16)));

    for action in actions {
        let envelope = transition_envelope(action.clone(), ActionOutcome::Accepted);
        let encoded = serde_json::to_value(&envelope).expect("v2 envelope must serialize");

        assert_eq!(
            encoded["payload"]["data"]["action"],
            serde_json::to_value(&action).unwrap()
        );

        let decoded: TraceEnvelopeV2 =
            serde_json::from_value(encoded).expect("v2 envelope must deserialize");
        assert_eq!(decoded, envelope);
    }
}

#[test]
fn accepted_and_rejected_outcomes_preserve_pre_and_post_payloads() {
    let accepted = transition_envelope(GameAction::Proceed, ActionOutcome::Accepted);
    let rejected = transition_envelope(
        GameAction::DiscardPotion(2),
        ActionOutcome::Rejected {
            code: "illegal_action".to_string(),
            detail: Some("slot is empty".to_string()),
        },
    );

    for envelope in [accepted, rejected] {
        let encoded = serde_json::to_string(&envelope).unwrap();
        let decoded: TraceEnvelopeV2 = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, envelope);

        let TracePayloadV2::Transition(transition) = decoded.payload else {
            panic!("expected transition payload");
        };
        assert_eq!(transition.pre, checkpoint(41));
        if matches!(&transition.outcome, ActionOutcome::Rejected { .. }) {
            assert_eq!(transition.post, transition.pre);
        } else {
            assert_eq!(transition.post, checkpoint(42));
        }
    }
}

#[test]
fn header_transition_and_end_payloads_round_trip() {
    let payloads = vec![
        TracePayloadV2::Header(HeaderV2 {
            trace_id: "trace-1".to_string(),
            seed: "ABC123".to_string(),
            seed_long: 1,
            character: "WATCHER".to_string(),
            ascension: 0,
            game_version: "desktop-2.3".to_string(),
            mods: vec!["tracelab".to_string()],
            initial_checkpoint: checkpoint(1),
        }),
        transition_envelope(GameAction::Proceed, ActionOutcome::Accepted).payload,
        TracePayloadV2::End(EndV2 {
            transition_count: 99,
            result: "victory".to_string(),
            final_checkpoint: checkpoint(99),
        }),
    ];

    for payload in payloads {
        let envelope = TraceEnvelopeV2::new(capabilities(), producer(), payload);
        let encoded = serde_json::to_string(&envelope).unwrap();
        let decoded: TraceEnvelopeV2 = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, envelope);
    }
}

#[test]
fn v2_accepts_additive_fields_at_the_current_minor() {
    let envelope = transition_envelope(GameAction::Proceed, ActionOutcome::Accepted);
    let mut encoded = serde_json::to_value(envelope).unwrap();
    encoded["future_envelope_field"] = json!({"ignored": true});
    encoded["payload"]["data"]["future_transition_field"] = json!("ignored");

    let decoded: TraceEnvelopeV2 = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded.schema.name, TRACE_SCHEMA_NAME);
    assert_eq!(decoded.schema.major, TRACE_SCHEMA_MAJOR);
    assert_eq!(decoded.schema.minor, TRACE_SCHEMA_MINOR);
    decoded.validate().unwrap();
}

#[test]
fn v2_rejects_an_unknown_future_minor() {
    let envelope = transition_envelope(GameAction::Proceed, ActionOutcome::Accepted);
    let mut encoded = serde_json::to_value(envelope).unwrap();
    encoded["schema"]["minor"] = json!(TRACE_SCHEMA_MINOR + 1);

    let error = serde_json::from_value::<TraceEnvelopeV2>(encoded).unwrap_err();
    assert!(error.to_string().contains("unsupported trace schema minor"));
}

#[test]
fn rejected_transition_requires_identical_pre_and_post_checkpoint() {
    let rejected = transition_envelope(
        GameAction::DiscardPotion(2),
        ActionOutcome::Rejected {
            code: "illegal_action".to_string(),
            detail: None,
        },
    );
    let mut encoded = serde_json::to_value(rejected).unwrap();
    encoded["payload"]["data"]["post"] = serde_json::to_value(checkpoint(42)).unwrap();

    let error = serde_json::from_value::<TraceEnvelopeV2>(encoded).unwrap_err();
    assert!(error
        .to_string()
        .contains("rejected trace transition must preserve"));
}

#[test]
fn header_seed_and_ascension_must_match_initial_checkpoint() {
    let header = TraceEnvelopeV2::new(
        capabilities(),
        producer(),
        TracePayloadV2::Header(HeaderV2 {
            trace_id: "trace-header-validation".to_string(),
            seed: "1".to_string(),
            seed_long: 2,
            character: "WATCHER".to_string(),
            ascension: 0,
            game_version: "desktop-2.3".to_string(),
            mods: Vec::new(),
            initial_checkpoint: checkpoint(1),
        }),
    );
    let error = serde_json::from_value::<TraceEnvelopeV2>(serde_json::to_value(header).unwrap())
        .unwrap_err();
    assert!(error.to_string().contains("trace header seed"));

    let header = TraceEnvelopeV2::new(
        capabilities(),
        producer(),
        TracePayloadV2::Header(HeaderV2 {
            trace_id: "trace-header-validation".to_string(),
            seed: "1".to_string(),
            seed_long: 1,
            character: "WATCHER".to_string(),
            ascension: 1,
            game_version: "desktop-2.3".to_string(),
            mods: Vec::new(),
            initial_checkpoint: checkpoint(1),
        }),
    );
    assert!(header
        .validate()
        .unwrap_err()
        .contains("trace header ascension"));
}

#[test]
fn v2_rejects_unknown_major_during_deserialization() {
    let envelope = transition_envelope(GameAction::Proceed, ActionOutcome::Accepted);
    let mut encoded = serde_json::to_value(envelope).unwrap();
    encoded["schema"]["major"] = json!(TRACE_SCHEMA_MAJOR + 1);

    let error = serde_json::from_value::<TraceEnvelopeV2>(encoded).unwrap_err();
    assert!(error.to_string().contains("unsupported trace schema major"));
}

#[test]
fn v2_schema_identity_is_explicit_and_does_not_replace_v1() {
    let version = SchemaVersion::current();
    assert_eq!(version.name, TRACE_SCHEMA_NAME);
    assert_eq!(version.major, TRACE_SCHEMA_MAJOR);
    assert_eq!(version.minor, TRACE_SCHEMA_MINOR);
    assert_eq!(TRACE_SCHEMA_NAME, "sts.trace");
    assert_eq!(TRACE_SCHEMA_MAJOR, 2);

    // The additive v2 module must not mutate the frozen v1 version constant.
    assert_eq!(crate::trace::TRACE_SCHEMA_VERSION, 1);

    let value: Value = serde_json::to_value(version).unwrap();
    assert_eq!(value, json!({"name": "sts.trace", "major": 2, "minor": 0}));
}

#[test]
fn v2_action_script_replays_deterministically_with_causal_chain() {
    let script = tiny_action_script();
    let first = replay_action_script_v2(&script).expect("v2 script must replay");
    let second = replay_action_script_v2(&script).expect("v2 script must replay twice");

    assert_eq!(first.len(), script.actions.len() + 2);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );

    let TracePayloadV2::Header(header) = &first[0].payload else {
        panic!("first envelope must be a header");
    };
    assert_eq!(header.seed_long, 4);

    let mut previous = &header.initial_checkpoint;
    for (idx, envelope) in first[1..=script.actions.len()].iter().enumerate() {
        let TracePayloadV2::Transition(transition) = &envelope.payload else {
            panic!("action envelope must be a transition");
        };
        assert_eq!(transition.idx, idx as u64);
        assert_eq!(transition.action, script.actions[idx]);
        assert_eq!(&transition.pre, previous);
        previous = &transition.post;
    }

    let TracePayloadV2::End(end) = &first.last().unwrap().payload else {
        panic!("last envelope must be an end record");
    };
    assert_eq!(end.transition_count, script.actions.len() as u64);
    assert_eq!(end.result, "incomplete");
    assert_eq!(&end.final_checkpoint, previous);
}

#[test]
fn v2_action_script_rejects_identity_drift_and_illegal_actions() {
    let mut script = tiny_action_script();
    script.seed_long = 5;
    assert!(replay_action_script_v2(&script)
        .unwrap_err()
        .contains("seed and seed_long disagree"));

    let mut script = tiny_action_script();
    script.actions[0] = GameAction::ChoosePath(0);
    let error = replay_action_script_v2(&script).unwrap_err();
    assert!(error.contains("action 0"));
    assert!(error.contains("not legal"));
}

#[test]
fn script_schema_document_names_every_serialized_game_action_variant() {
    let contract = include_str!("../../../../docs/work_units/script-schema-v2.md");
    for action in all_current_game_actions() {
        let encoded = serde_json::to_value(action).unwrap();
        let variant = match encoded {
            Value::String(name) => name,
            Value::Object(fields) => fields.into_iter().next().unwrap().0,
            other => panic!("unexpected GameAction serialization: {other}"),
        };
        assert!(
            contract.contains(&format!("| `{variant}` |")),
            "script schema document is missing {variant}"
        );
    }
}
