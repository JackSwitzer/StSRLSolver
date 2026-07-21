use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::run::{ProfileSnapshot, RunEngine};
use crate::trace::bundle::{
    compare_recording_bundle, is_distilled_chaos_recorder_only_callback, load_recording_bundle,
    load_recording_meta, BundleComparisonStatus, RecordedTraceRecord, RecorderAction,
    RecordingBundle, RecordingMeta,
};
use crate::trace::TraceProfileSnapshot;

fn recordings_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/traces/recordings")
}

fn bundle_paths() -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(recordings_root())
        .expect("recording corpus must exist")
        .map(|entry| {
            entry
                .expect("recording directory entry must be readable")
                .path()
        })
        .filter(|path| path.is_dir())
        .collect();
    paths.sort();
    paths
}

#[test]
fn committed_recording_inventory_has_parseable_metadata() {
    let paths = bundle_paths();
    assert_eq!(paths.len(), 14, "Wave 3 corpus inventory changed");
    let mut declared_records = 0usize;
    for path in paths {
        let meta = load_recording_meta(&path).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(meta.run_id, path.file_name().unwrap().to_string_lossy());
        declared_records += meta.records;
    }
    assert_eq!(declared_records, 3_108);
}

#[test]
fn complete_a0_bundles_generate_report_only_prefix_results() {
    let mut reports = Vec::new();
    for path in bundle_paths() {
        let meta = load_recording_meta(&path).unwrap();
        if meta.ascension != 0 || !matches!(meta.status.as_str(), "VICTORY" | "DEATH") {
            continue;
        }
        let bundle = load_recording_bundle(&path).unwrap_or_else(|error| panic!("{error}"));
        let report = compare_recording_bundle(&bundle).unwrap_or_else(|error| panic!("{error}"));
        assert_ne!(report.status, BundleComparisonStatus::NoActions);
        assert_eq!(report.total_actions, meta.records);
        assert_eq!(
            report.unverified_action_semantics,
            report.unverified_actions.len() + report.ignored_recorder_actions.len(),
            "every uncertified recorder action must carry structured evidence: {report:?}"
        );
        assert_eq!(
            report
                .initialization_quarantine
                .as_ref()
                .map(|quarantine| quarantine.kind.as_str()),
            Some("missing_profile_snapshot"),
            "legacy corpus must not infer an authoritative all-unlocked profile"
        );
        assert!(
            report.unverified_actions.iter().any(|entry| {
                entry.idx == 1
                    && entry
                        .reason
                        .contains("selected Neow index, not the generated option payload")
            }),
            "the recorder's index-only Neow selection must remain explicit: {report:?}"
        );
        eprintln!(
            "{} compares through {}/{} ({} direct checkpoints); first={:?}",
            report.bundle,
            report.comparable_actions,
            report.total_actions,
            report.matched_actions,
            report
                .first_divergence
                .as_ref()
                .map(|divergence| (&divergence.kind, &divergence.path))
        );
        reports.push(report);
    }
    assert_eq!(
        reports.len(),
        5,
        "expected three A0 victories and two A0 deaths"
    );
    let direct_reward = reports
        .iter()
        .find(|report| report.bundle.starts_with("-5884681071377138867-"))
        .expect("direct Neow reward victory bundle");
    assert!(
        direct_reward.comparable_actions >= 51,
        "source-derived replay prefix regressed: {direct_reward:?}"
    );
    if direct_reward.comparable_actions == 51 {
        assert!(
            direct_reward
                .first_divergence
                .as_ref()
                .is_some_and(|divergence| divergence.path == "relics[1].id"),
            "the current earliest relic-pool mismatch must stay explicit: {direct_reward:?}"
        );
    }

    let grid_followups: Vec<_> = reports
        .iter()
        .filter(|report| report.bundle != direct_reward.bundle)
        .collect();
    assert_eq!(grid_followups.len(), 4);
    for report in grid_followups {
        assert_eq!(report.comparable_actions, 2);
        assert_eq!(report.replayed_actions, 2);
        assert!(
            report
                .first_divergence
                .as_ref()
                .is_some_and(|divergence| divergence
                    .detail
                    .contains("selected deck-card identity")),
            "Neow grid replay must stop rather than guess a card: {report:?}"
        );
    }
}

#[test]
fn coupled_or_incomplete_legacy_prefix_is_never_reported_as_a_match() {
    let path = recordings_root().join("-5884681071377138867-WATCHER-20260720-194423");
    let mut bundle = load_recording_bundle(path).unwrap();
    bundle.actions.truncate(51);
    bundle.records.truncate(51);
    bundle.meta.records = 51;
    bundle.meta.profile = Some(TraceProfileSnapshot {
        v: 1,
        note_for_yourself_card: "IronWave".to_string(),
        highest_unlocked_ascension: 20,
        is_daily_run: false,
        final_act_available: true,
        bosses_seen: vec![
            "GUARDIAN".to_string(),
            "GHOST".to_string(),
            "SLIME".to_string(),
            "CHAMP".to_string(),
            "AUTOMATON".to_string(),
            "COLLECTOR".to_string(),
            "CROW".to_string(),
            "DONUT".to_string(),
            "WIZARD".to_string(),
        ],
        locked_cards: Vec::new(),
        locked_relics: Vec::new(),
    });

    let report = compare_recording_bundle(&bundle).unwrap();
    assert_eq!(report.status, BundleComparisonStatus::Uncertified);
    assert_eq!(report.comparable_actions, 51);
    assert!(report.coupled_actions > 0);
    assert!(report.matched_actions < report.comparable_actions);
    assert!(report.initialization_quarantine.is_none());
}

#[test]
fn valid_bundle_intake_aligns_script_trace_and_metadata() {
    let path = recordings_root().join("-5884681071377138867-WATCHER-20260720-194423");
    let bundle = load_recording_bundle(path).unwrap();
    assert_eq!(bundle.meta.records, 607);
    assert_eq!(bundle.actions.len(), 607);
    assert_eq!(bundle.records.len(), 607);
    assert_eq!(bundle.actions[0].action_type, "NEOW");
    assert_eq!(bundle.records[606].idx, 606);
}

#[test]
fn ordinary_profile_json_defaults_missing_unlock_state_to_all_unlocked() {
    let profile = ProfileSnapshot::all_unlocked();
    let mut value = serde_json::to_value(&profile).unwrap();
    value.as_object_mut().unwrap().remove("locked_cards");
    value.as_object_mut().unwrap().remove("locked_relics");

    let decoded: ProfileSnapshot = serde_json::from_value(value).unwrap();
    assert!(decoded.locked_cards.is_empty());
    assert!(decoded.locked_relics.is_empty());
}

#[test]
fn recorder_profile_contract_requires_explicit_unlock_sets() {
    let profile = TraceProfileSnapshot {
        v: 1,
        note_for_yourself_card: "IronWave".to_string(),
        highest_unlocked_ascension: 20,
        is_daily_run: false,
        final_act_available: true,
        bosses_seen: vec!["GUARDIAN".to_string(), "GHOST".to_string()],
        locked_cards: vec!["Prostrate".to_string()],
        locked_relics: vec!["Omamori".to_string(), "Turnip".to_string()],
    };
    profile.check_version().unwrap();
    let value = serde_json::to_value(&profile).unwrap();
    for required_field in ["locked_cards", "locked_relics"] {
        let mut missing = value.clone();
        missing.as_object_mut().unwrap().remove(required_field);

        let error = serde_json::from_value::<TraceProfileSnapshot>(missing).unwrap_err();
        assert!(error.to_string().contains(required_field));
    }

    let mut duplicate = profile;
    duplicate.locked_relics.push("Turnip".to_string());
    assert!(duplicate.check_version().unwrap_err().contains("duplicate"));
}

#[test]
fn recorded_profile_lock_state_filters_card_pools_before_use() {
    // Watcher.getCardPool delegates to CardLibrary.addPurpleCards, which drops
    // UnlockTracker-locked cards before AbstractDungeon builds working/source
    // pools. Java: Watcher.java:179-190; CardLibrary.java:1172-1179.
    let all_unlocked = RunEngine::new_with_profile(7, 0, ProfileSnapshot::all_unlocked());
    let mut profile = ProfileSnapshot::all_unlocked();
    profile.locked_cards = vec![
        "Prostrate".to_string(),
        "ForeignInfluence".to_string(),
        "Blasphemy".to_string(),
    ];
    let filtered = RunEngine::new_with_profile(7, 0, profile);

    let (all_common, all_uncommon, all_rare) = all_unlocked.debug_card_pool_lengths();
    assert_eq!(
        filtered.debug_card_pool_lengths(),
        (all_common - 1, all_uncommon - 1, all_rare - 1)
    );
    for locked in ["Prostrate", "ForeignInfluence", "Blasphemy"] {
        assert!(all_unlocked.debug_card_pool_contains(locked), "{locked}");
        assert!(!filtered.debug_card_pool_contains(locked), "{locked}");
    }
}

#[test]
fn fresh_profile_matches_java_initial_unlock_tracker_lists() {
    // UnlockTracker.refresh adds these preference-backed IDs to the locked
    // lists on a new profile. Java: UnlockTracker.java:173-240,484-487.
    let fresh = ProfileSnapshot::fresh();
    let expected_cards = [
        "Havoc",
        "Sentinel",
        "Exhume",
        "Wild Strike",
        "Evolve",
        "Immolate",
        "Heavy Blade",
        "Spot Weakness",
        "Limit Break",
        "Concentrate",
        "Setup",
        "Grand Finale",
        "Cloak And Dagger",
        "Accuracy",
        "Storm of Steel",
        "Bane",
        "Catalyst",
        "Corpse Explosion",
        "Rebound",
        "Undo",
        "Echo Form",
        "Turbo",
        "Sunder",
        "Meteor Strike",
        "Hyperbeam",
        "Recycle",
        "Core Surge",
        "Prostrate",
        "Blasphemy",
        "Devotion",
        "ForeignInfluence",
        "Alpha",
        "MentalFortress",
        "SpiritShield",
        "Wish",
        "Wireheading",
    ];
    let expected_relics = [
        "Omamori",
        "Prayer Wheel",
        "Shovel",
        "Art of War",
        "The Courier",
        "Pandora's Box",
        "Blue Candle",
        "Dead Branch",
        "Singing Bowl",
        "Du-Vu Doll",
        "Smiling Mask",
        "Tiny Chest",
        "Cables",
        "DataDisk",
        "Emotion Chip",
        "Runic Capacitor",
        "Turnip",
        "Symbiotic Virus",
        "Akabeko",
        "Yang",
        "CeramicFish",
        "StrikeDummy",
        "TeardropLocket",
        "CloakClasp",
    ];
    assert_eq!(
        fresh.locked_cards,
        expected_cards
            .iter()
            .map(|id| (*id).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        fresh.locked_relics,
        expected_relics
            .iter()
            .map(|id| (*id).to_string())
            .collect::<Vec<_>>()
    );

    let all_unlocked = ProfileSnapshot::all_unlocked();
    assert!(all_unlocked.locked_cards.is_empty());
    assert!(all_unlocked.locked_relics.is_empty());

    let fresh_engine = RunEngine::new_with_profile(7, 0, fresh);
    let all_unlocked_engine = RunEngine::new_with_profile(7, 0, all_unlocked);
    for card_id in &expected_cards[27..] {
        assert!(
            all_unlocked_engine.debug_card_pool_contains(card_id),
            "{card_id}"
        );
        assert!(!fresh_engine.debug_card_pool_contains(card_id), "{card_id}");
    }
    for relic_id in expected_relics {
        if all_unlocked_engine.debug_relic_pool_contains(relic_id) {
            assert!(
                !fresh_engine.debug_relic_pool_contains(relic_id),
                "{relic_id}"
            );
        }
    }
}

#[test]
fn recorded_profile_lock_state_filters_relic_pools_before_shuffle() {
    // RelicLibrary.populateRelicPool excludes every UnlockTracker-locked relic,
    // then AbstractDungeon.initializeRelicList consumes one shuffle seed per
    // tier. Java: RelicLibrary.java:628-662; AbstractDungeon.java:1211-1231.
    let mut profile = ProfileSnapshot::all_unlocked();
    profile.locked_relics = vec![
        "Omamori".to_string(),
        "Turnip".to_string(),
        "VioletLotus".to_string(),
    ];
    let engine = RunEngine::new_with_profile(7, 0, profile);

    assert_eq!(engine.debug_relic_pool_lengths(), (32, 30, 26, 17, 20));
    for locked in ["Omamori", "Turnip", "VioletLotus"] {
        assert!(!engine.debug_relic_pool_contains(locked), "{locked}");
    }
    assert_eq!(engine.rng_counters()["relic"], 5);
}

#[test]
fn daily_profile_bypasses_card_and_relic_locks_like_java() {
    // Settings.treatEverythingAsUnlocked is true for daily runs, and both
    // CardLibrary and RelicLibrary guard their lock checks with it.
    // Java: Settings.java:629-635; CardLibrary.java:1172-1179;
    // RelicLibrary.java:628-662.
    let all_unlocked = RunEngine::new_with_profile(7, 0, ProfileSnapshot::all_unlocked());
    let mut daily = ProfileSnapshot::fresh();
    daily.is_daily_run = true;
    let daily = RunEngine::new_with_profile(7, 0, daily);

    assert_eq!(
        daily.debug_card_pool_lengths(),
        all_unlocked.debug_card_pool_lengths()
    );
    assert_eq!(
        daily.debug_relic_pool_lengths(),
        all_unlocked.debug_relic_pool_lengths()
    );
}

#[test]
fn profile_presence_controls_bundle_initialization_quarantine() {
    let meta = RecordingMeta {
        v: 1,
        run_id: "profile-contract".to_string(),
        seed_long: 7,
        seed_display: "7".to_string(),
        character: "WATCHER".to_string(),
        ascension: 0,
        game_version: "desktop-1.0".to_string(),
        status: "IN_PROGRESS".to_string(),
        records: 0,
        sittings: Vec::new(),
        profile: None,
    };
    let bundle = RecordingBundle {
        path: PathBuf::new(),
        meta: meta.clone(),
        actions: Vec::new(),
        records: Vec::new(),
    };
    let legacy = compare_recording_bundle(&bundle).unwrap();
    assert_eq!(
        legacy
            .initialization_quarantine
            .as_ref()
            .map(|quarantine| quarantine.kind.as_str()),
        Some("missing_profile_snapshot")
    );

    let authoritative = RecordingBundle {
        meta: RecordingMeta {
            profile: Some(TraceProfileSnapshot {
                v: 1,
                note_for_yourself_card: "IronWave".to_string(),
                highest_unlocked_ascension: 20,
                is_daily_run: false,
                final_act_available: true,
                bosses_seen: vec![
                    "GUARDIAN".to_string(),
                    "GHOST".to_string(),
                    "SLIME".to_string(),
                    "CHAMP".to_string(),
                    "AUTOMATON".to_string(),
                    "COLLECTOR".to_string(),
                    "CROW".to_string(),
                    "DONUT".to_string(),
                    "WIZARD".to_string(),
                ],
                locked_cards: Vec::new(),
                locked_relics: Vec::new(),
            }),
            ..meta
        },
        ..bundle
    };
    assert!(compare_recording_bundle(&authoritative)
        .unwrap()
        .initialization_quarantine
        .is_none());
}

#[test]
fn distilled_chaos_callbacks_require_the_complete_identical_state_witness() {
    fn action(idx: usize, payload: serde_json::Value) -> RecorderAction {
        RecorderAction {
            idx,
            action_type: payload["type"].as_str().unwrap().to_string(),
            raw: payload,
        }
    }
    fn record(idx: usize, action: &RecorderAction, state_marker: i32) -> RecordedTraceRecord {
        RecordedTraceRecord {
            idx,
            floor: 1,
            act: 1,
            turn: 1,
            phase: "COMBAT".to_string(),
            screen: "NONE".to_string(),
            map: json!({"x": 0, "y": 0}),
            action: action.raw.clone(),
            post: json!({"state_marker": state_marker}),
            deck: json!(["Strike_P"]),
        }
    }

    let mut actions = vec![action(
        0,
        json!({"type": "USE_POTION", "potion_id": "DistilledChaos", "slot": 0}),
    )];
    for idx in 1..=4 {
        actions.push(action(
            idx,
            json!({"type": "PLAY_CARD", "card_id": "Strike_P", "hand_idx": -1}),
        ));
    }
    let records = actions
        .iter()
        .enumerate()
        .map(|(idx, action)| record(idx, action, 1))
        .collect::<Vec<_>>();

    for position in 1..=3 {
        assert!(is_distilled_chaos_recorder_only_callback(
            &actions, &records, position
        ));
    }
    assert!(!is_distilled_chaos_recorder_only_callback(
        &actions, &records, 4
    ));

    let mut changed_state = records.clone();
    changed_state[2].post = json!({"state_marker": 2});
    assert!(!is_distilled_chaos_recorder_only_callback(
        &actions,
        &changed_state,
        2
    ));

    let mut ordinary_hand_play = actions.clone();
    ordinary_hand_play[1].raw["hand_idx"] = json!(0);
    assert!(!is_distilled_chaos_recorder_only_callback(
        &ordinary_hand_play,
        &records,
        1
    ));

    let mut no_potion_witness = actions;
    no_potion_witness[0].raw["potion_id"] = json!("Energy Potion");
    assert!(!is_distilled_chaos_recorder_only_callback(
        &no_potion_witness,
        &records,
        1
    ));
}
