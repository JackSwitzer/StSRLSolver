use crate::actions::Action;
use crate::run::GameAction;
use crate::trace::v2::{
    ActionOutcome, Capabilities, EndV2, HeaderV2, Producer, SchemaVersion, TraceEnvelopeV2,
    TracePayloadV2, TransitionV2, TRACE_SCHEMA_MAJOR, TRACE_SCHEMA_MINOR, TRACE_SCHEMA_NAME,
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
        "opaque_checkpoints".to_string(),
    ])
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

fn transition_envelope(action: GameAction, outcome: ActionOutcome) -> TraceEnvelopeV2 {
    TraceEnvelopeV2::new(
        capabilities(),
        producer(),
        TracePayloadV2::Transition(TransitionV2 {
            idx: 42,
            action,
            outcome,
            pre: json!({"opaque": "before"}),
            post: json!({"opaque": "after"}),
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
        assert_eq!(transition.pre, json!({"opaque": "before"}));
        assert_eq!(transition.post, json!({"opaque": "after"}));
    }
}

#[test]
fn header_transition_and_end_payloads_round_trip() {
    let payloads = vec![
        TracePayloadV2::Header(HeaderV2 {
            trace_id: "trace-1".to_string(),
            seed: "ABC123".to_string(),
            seed_long: 12345,
            character: "WATCHER".to_string(),
            ascension: 0,
            game_version: "desktop-2.3".to_string(),
            mods: vec!["tracelab".to_string()],
            initial_checkpoint: json!({"opaque": "initial"}),
        }),
        transition_envelope(GameAction::Proceed, ActionOutcome::Accepted).payload,
        TracePayloadV2::End(EndV2 {
            transition_count: 99,
            result: "victory".to_string(),
            final_checkpoint: json!({"opaque": "final"}),
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
fn v2_accepts_unknown_minor_and_additive_fields() {
    let envelope = transition_envelope(GameAction::Proceed, ActionOutcome::Accepted);
    let mut encoded = serde_json::to_value(envelope).unwrap();
    encoded["schema"]["minor"] = json!(999);
    encoded["future_envelope_field"] = json!({"ignored": true});
    encoded["payload"]["data"]["future_transition_field"] = json!("ignored");

    let decoded: TraceEnvelopeV2 = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded.schema.name, TRACE_SCHEMA_NAME);
    assert_eq!(decoded.schema.major, TRACE_SCHEMA_MAJOR);
    assert_eq!(decoded.schema.minor, 999);
    decoded.validate().unwrap();
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
