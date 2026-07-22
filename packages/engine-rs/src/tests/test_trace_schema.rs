//! U02 — trace schema (`src/trace.rs`) round-trip + fixture tests.
//!
//! Java refs: schema mirrors `AbstractDungeon` public static RNG fields and
//! `Settings`/`CardCrawlGame` (see doc comments in `trace.rs`); this file
//! only exercises Rust-side (de)serialization, not game behavior.

use std::collections::BTreeMap;

use crate::trace::{
    check_version, parse_masks, parse_script_seed, ActionScript, DivergenceReport,
    DivergenceStatus, EnemyPostState, FieldDiff, FirstDivergence, IntentPostState, Mask,
    MaskedDiff, OrbPostState, PilePostState, PlayerPostState, PostState, PowerPostState,
    RelicPostState, RngSnapshotPair, ScriptStopCondition, TraceAction, TraceHeader, TraceRecord,
    TRACE_SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// Version helper
// ---------------------------------------------------------------------------

#[test]
fn script_seed_parsing_matches_tracelab_decimal_then_display_precedence() {
    // Source: packages/harness-java/src/main/java/tracelab/TraceLabMod.java
    assert_eq!(parse_script_seed("57554006466"), 57_554_006_466);
    assert_eq!(
        parse_script_seed("ABC"),
        crate::seed::seed_from_string("ABC")
    );
    assert_eq!(parse_script_seed("10"), 10);
    assert_eq!(parse_script_seed("-1"), u64::MAX);
}

#[test]
fn check_version_accepts_v1() {
    assert!(check_version(1).is_ok());
    assert_eq!(TRACE_SCHEMA_VERSION, 1);
}

#[test]
fn check_version_rejects_other() {
    let err = check_version(2).expect_err("v2 must be rejected");
    assert!(
        err.contains('2'),
        "error should name the bad version: {err}"
    );
    assert!(
        err.contains('1'),
        "error should name the expected version: {err}"
    );
}

// ---------------------------------------------------------------------------
// Round-trip helpers
// ---------------------------------------------------------------------------

fn sample_rng_map() -> BTreeMap<String, i64> {
    let mut rng = BTreeMap::new();
    rng.insert("card".to_string(), 37);
    rng.insert("ai".to_string(), 4);
    rng.insert("shuffle".to_string(), 12);
    rng.insert("cardRandom".to_string(), 0);
    rng.insert("misc".to_string(), 2);
    rng.insert("monsterHp".to_string(), 6);
    rng.insert("treasure".to_string(), 3);
    rng.insert("relic".to_string(), 11);
    rng.insert("potion".to_string(), 5);
    rng.insert("merchant".to_string(), 0);
    rng.insert("event".to_string(), 1);
    rng.insert("monster".to_string(), 8);
    rng.insert("map".to_string(), 57);
    rng
}

fn sample_post_state() -> PostState {
    PostState {
        player: PlayerPostState {
            hp: 61,
            max_hp: 72,
            block: 7,
            energy: 1,
            stance: "Calm".to_string(),
            gold: 124,
            powers: vec![PowerPostState {
                id: "Vigor".to_string(),
                amt: 8,
            }],
            orbs: vec![],
        },
        enemies: vec![EnemyPostState {
            id: "JawWorm".to_string(),
            idx: 0,
            hp: 30,
            max_hp: 44,
            block: 6,
            intent: IntentPostState {
                move_id: 1,
                name: "CHOMP".to_string(),
                dmg: 12,
                hits: 1,
            },
            powers: vec![],
            move_history: vec![3, 1],
        }],
        piles: PilePostState {
            hand: vec![
                "Strike_P".to_string(),
                "Defend_P".to_string(),
                "Eruption".to_string(),
            ],
            draw_ordered: vec![],
            discard: vec![],
            exhaust: vec![],
        },
        relics: vec![RelicPostState {
            id: "PureWater".to_string(),
            counter: -1,
        }],
        potions: vec!["FairyPotion".to_string()],
        rng: sample_rng_map(),
    }
}

macro_rules! round_trip {
    ($name:ident, $value:expr, $ty:ty) => {
        #[test]
        fn $name() {
            let value: $ty = $value;
            let json = serde_json::to_string(&value).expect("serialize");
            let decoded: $ty = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(value, decoded, "round-trip mismatch via {json}");
        }
    };
}

round_trip!(
    trace_header_round_trip,
    TraceHeader {
        v: 1,
        kind: "header".to_string(),
        seed: "3LGMWP6QYAWB".to_string(),
        seed_long: 57554006466,
        character: "WATCHER".to_string(),
        ascension: 0,
        game_version: "desktop-1.0".to_string(),
        mods: vec![
            "basemod".to_string(),
            "stslib".to_string(),
            "tracelab".to_string()
        ],
    },
    TraceHeader
);

round_trip!(post_state_round_trip, sample_post_state(), PostState);

#[test]
fn post_state_preserves_signed_java_rng_counter_overflow() {
    let mut post = sample_post_state();
    post.rng.insert("card".to_string(), i64::from(i32::MIN));
    let json = serde_json::to_string(&post).unwrap();
    assert!(json.contains("-2147483648"));
    assert_eq!(serde_json::from_str::<PostState>(&json).unwrap(), post);
}

round_trip!(
    trace_record_round_trip,
    TraceRecord {
        v: 1,
        idx: 57,
        floor: 4,
        turn: 3,
        phase: "COMBAT".to_string(),
        action: TraceAction::PlayCard {
            hand_idx: 2,
            target: 0,
            card_id: Some("EmptyBody".to_string()),
        },
        post: sample_post_state(),
    },
    TraceRecord
);

round_trip!(
    action_play_card_round_trip,
    TraceAction::PlayCard {
        hand_idx: 2,
        target: 0,
        card_id: None
    },
    TraceAction
);
round_trip!(
    action_end_turn_round_trip,
    TraceAction::EndTurn,
    TraceAction
);
round_trip!(
    action_use_potion_round_trip,
    TraceAction::UsePotion { idx: 0, target: 0 },
    TraceAction
);
round_trip!(
    action_neow_round_trip,
    TraceAction::Neow { choice: 1 },
    TraceAction
);
round_trip!(
    action_path_round_trip,
    TraceAction::Path { choice: 0 },
    TraceAction
);
round_trip!(
    action_reward_take_round_trip,
    TraceAction::RewardTake { item: 0 },
    TraceAction
);
round_trip!(
    action_reward_skip_round_trip,
    TraceAction::RewardSkip { item: None },
    TraceAction
);
round_trip!(
    action_event_choice_round_trip,
    TraceAction::EventChoice { choice: 1 },
    TraceAction
);
round_trip!(
    action_campfire_round_trip,
    TraceAction::Campfire {
        choice: "REST".to_string(),
        card_idx: None
    },
    TraceAction
);
round_trip!(
    action_shop_buy_round_trip,
    TraceAction::ShopBuy { item: 2 },
    TraceAction
);
round_trip!(
    action_shop_remove_round_trip,
    TraceAction::ShopRemove { deck_idx: 1 },
    TraceAction
);
round_trip!(
    action_shop_leave_round_trip,
    TraceAction::ShopLeave,
    TraceAction
);

round_trip!(
    action_script_round_trip,
    ActionScript {
        v: 1,
        seed: "3LGMWP6QYAWB".to_string(),
        character: "WATCHER".to_string(),
        ascension: 0,
        stop: ScriptStopCondition {
            max_floor: Some(8),
            max_actions: None
        },
        actions: vec![
            TraceAction::Neow { choice: 1 },
            TraceAction::Path { choice: 0 },
            TraceAction::PlayCard {
                hand_idx: 2,
                target: 0,
                card_id: None
            },
            TraceAction::EndTurn,
        ],
    },
    ActionScript
);

round_trip!(
    divergence_report_match_round_trip,
    DivergenceReport {
        status: DivergenceStatus::Match,
        script: "act1-jawworm-3turn".to_string(),
        seed: "3LGMWP6QYAWB".to_string(),
        matched_actions: 112,
        total_actions: 112,
        first_divergence: None,
        masked: vec![],
        secondary: vec![],
    },
    DivergenceReport
);

round_trip!(
    divergence_report_diverged_round_trip,
    DivergenceReport {
        status: DivergenceStatus::Diverged,
        script: "act1-jawworm-3turn".to_string(),
        seed: "3LGMWP6QYAWB".to_string(),
        matched_actions: 56,
        total_actions: 112,
        first_divergence: Some(FirstDivergence {
            idx: 57,
            floor: 4,
            turn: 3,
            path: "post.enemies[0].intent.move_id".to_string(),
            java: serde_json::json!(2),
            rust: serde_json::json!(1),
            rng_at_divergence: RngSnapshotPair {
                java: BTreeMap::from([("ai".to_string(), 9)]),
                rust: BTreeMap::from([("ai".to_string(), 6)]),
            },
        }),
        masked: vec![MaskedDiff {
            path: "post.neow.options".to_string(),
            java: serde_json::json!([1, 2, 3, 4]),
            rust: serde_json::json!([1, 2, 3, 4, 5]),
            id: "DEV-003".to_string(),
        }],
        secondary: vec![FieldDiff {
            path: "post.rng.ai".to_string(),
            java: serde_json::json!(9),
            rust: serde_json::json!(6),
        }],
    },
    DivergenceReport
);

round_trip!(
    mask_round_trip,
    Mask {
        id: "DEV-003".to_string(),
        path: "post.neow.options".to_string(),
        scope: "all".to_string(),
        reason: "engine intentionally exposes 4 Neow options".to_string(),
        register: "docs/work_units/parity-deviations-register.md".to_string(),
    },
    Mask
);

round_trip!(
    orb_post_state_round_trip,
    OrbPostState {
        id: "Lightning".to_string(),
        evoke_amount: 3,
        passive_amount: 1
    },
    OrbPostState
);

// ---------------------------------------------------------------------------
// Masks: DEV- prefix enforcement
// ---------------------------------------------------------------------------

#[test]
fn mask_check_id_accepts_dev_prefix() {
    let mask = Mask {
        id: "DEV-003".to_string(),
        path: "post.neow.options".to_string(),
        scope: "all".to_string(),
        reason: "reason".to_string(),
        register: "docs/work_units/parity-deviations-register.md".to_string(),
    };
    assert!(mask.check_id().is_ok());
}

#[test]
fn mask_check_id_rejects_missing_dev_prefix() {
    let mask = Mask {
        id: "D3".to_string(),
        path: "post.neow.options".to_string(),
        scope: "all".to_string(),
        reason: "reason".to_string(),
        register: "docs/work_units/parity-deviations-register.md".to_string(),
    };
    let err = mask.check_id().expect_err("bare D-id must be rejected");
    assert!(
        err.contains("DEV-"),
        "error should mention DEV- requirement: {err}"
    );
    assert!(err.contains("D3"), "error should name offending id: {err}");
}

#[test]
fn parse_masks_rejects_file_with_any_bad_id() {
    let json = r#"[
        {"id":"DEV-003","path":"post.neow.options","scope":"all","reason":"ok","register":"docs/work_units/parity-deviations-register.md"},
        {"id":"D3","path":"post.other","scope":"all","reason":"bad","register":"docs/work_units/parity-deviations-register.md"}
    ]"#;
    let err = parse_masks(json).expect_err("file with a bad id must fail whole-file");
    assert!(err.contains("DEV-"));
}

#[test]
fn parse_masks_rejects_unsupported_scope() {
    // The differ only implements scope "all" today; a narrower scope string
    // must fail loudly instead of being silently applied to every record.
    let json = r#"[{"id":"DEV-004","path":"post.rng.ai","scope":"3LGMWP6QYAWB@f4","reason":"scoped","register":"docs/work_units/parity-deviations-register.md"}]"#;
    let err = parse_masks(json).expect_err("non-'all' scope must be rejected");
    assert!(err.contains("scope"), "error should mention scope: {err}");
    assert!(
        err.contains("DEV-004"),
        "error should name offending mask: {err}"
    );
}

#[test]
fn parse_masks_accepts_well_formed_file() {
    let json = r#"[{"id":"DEV-003","path":"post.neow.options","scope":"all","reason":"ok","register":"docs/work_units/parity-deviations-register.md"}]"#;
    let masks = parse_masks(json).expect("well-formed masks file should parse");
    assert_eq!(masks.len(), 1);
    assert_eq!(masks[0].id, "DEV-003");
}

// ---------------------------------------------------------------------------
// Hand-written multi-line JSONL fixture (header + 2 records) — schema-drift
// tripwire per docs/goal/TOOLING.md § T1.
// ---------------------------------------------------------------------------

const JSONL_FIXTURE: &str = r#"{"v":1,"kind":"header","seed":"3LGMWP6QYAWB","seed_long":57554006466,"character":"WATCHER","ascension":0,"game_version":"desktop-1.0","mods":["basemod","stslib","tracelab"]}
{"v":1,"idx":0,"floor":1,"turn":1,"phase":"COMBAT","action":{"type":"PLAY_CARD","hand_idx":2,"target":0,"card_id":"EmptyBody"},"post":{"player":{"hp":72,"max_hp":72,"block":0,"energy":2,"stance":"Neutral","gold":99,"powers":[],"orbs":[]},"enemies":[{"id":"Cultist","idx":0,"hp":40,"max_hp":48,"block":0,"intent":{"move_id":1,"name":"INCANTATION","dmg":0,"hits":0},"powers":[],"move_history":[1]}],"piles":{"hand":["Strike","Strike","Defend"],"draw_ordered":["Strike","Defend"],"discard":["EmptyBody"],"exhaust":[]},"relics":[{"id":"PureWater","counter":-1}],"potions":[],"rng":{"ai":1,"card":0}}}
{"v":1,"idx":1,"floor":1,"turn":1,"phase":"COMBAT","action":{"type":"END_TURN"},"post":{"player":{"hp":66,"max_hp":72,"block":0,"energy":3,"stance":"Neutral","gold":99,"powers":[],"orbs":[]},"enemies":[{"id":"Cultist","idx":0,"hp":40,"max_hp":48,"block":0,"intent":{"move_id":2,"name":"DARK_STRIKE","dmg":6,"hits":1},"powers":[{"id":"Ritual","amt":3}],"move_history":[1,2]}],"piles":{"hand":["Strike","Strike","Defend","Strike","Defend"],"draw_ordered":[],"discard":["EmptyBody"],"exhaust":[]},"relics":[{"id":"PureWater","counter":-1}],"potions":[],"rng":{"ai":2,"card":0}}}
"#;

#[test]
fn jsonl_fixture_parses_as_header_then_records() {
    let mut lines = JSONL_FIXTURE.lines();

    let header_line = lines.next().expect("fixture must have a header line");
    let header: TraceHeader = serde_json::from_str(header_line).expect("header must parse");
    header
        .check_version()
        .expect("header version must be supported");
    assert_eq!(header.kind, "header");
    assert_eq!(header.character, "WATCHER");
    assert_eq!(header.seed_long, 57554006466);

    let mut records = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let record: TraceRecord = serde_json::from_str(line).expect("record must parse");
        record
            .check_version()
            .expect("record version must be supported");
        records.push(record);
    }

    assert_eq!(
        records.len(),
        2,
        "fixture must contain exactly 2 records after the header"
    );
    assert_eq!(records[0].idx, 0);
    assert_eq!(records[1].idx, 1);
    assert_eq!(records[0].phase, "COMBAT");
    match &records[0].action {
        TraceAction::PlayCard {
            hand_idx,
            target,
            card_id,
        } => {
            assert_eq!(*hand_idx, 2);
            assert_eq!(*target, 0);
            assert_eq!(card_id.as_deref(), Some("EmptyBody"));
        }
        other => panic!("expected PLAY_CARD, got {other:?}"),
    }
    match &records[1].action {
        TraceAction::EndTurn => {}
        other => panic!("expected END_TURN, got {other:?}"),
    }

    // rng counters must survive as a plain map, ordered keys (BTreeMap).
    assert_eq!(records[0].post.rng.get("ai"), Some(&1));
    assert_eq!(records[1].post.rng.get("ai"), Some(&2));

    // Second record shows a power appearing (Ritual) — sanity-check nested
    // Vec<PowerPostState> decodes.
    assert_eq!(records[1].post.enemies[0].powers.len(), 1);
    assert_eq!(records[1].post.enemies[0].powers[0].id, "Ritual");
    assert_eq!(records[1].post.enemies[0].powers[0].amt, 3);
}

#[test]
fn jsonl_fixture_round_trips_byte_stable_keys() {
    // Re-serializing a parsed record should still parse identically
    // (guards against accidental serde attribute drift).
    let line = JSONL_FIXTURE.lines().nth(1).unwrap();
    let record: TraceRecord = serde_json::from_str(line).unwrap();
    let re_encoded = serde_json::to_string(&record).unwrap();
    let re_decoded: TraceRecord = serde_json::from_str(&re_encoded).unwrap();
    assert_eq!(record, re_decoded);
}
