use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::run::{GameAction, ProfileSnapshot, RunEngine};
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

fn recording_prefix(run_id: &str, actions: usize) -> RecordingBundle {
    let mut bundle = load_recording_bundle(recordings_root().join(run_id)).unwrap();
    bundle.actions.truncate(actions);
    bundle.records.truncate(actions);
    bundle.meta.records = actions;
    bundle.meta.status = "IN_PROGRESS".to_string();
    bundle.run_end = None;
    bundle
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
        let initialization_quarantine = report
            .initialization_quarantine
            .as_ref()
            .expect("legacy bundles must remain initialization-uncertified");
        assert_eq!(
            initialization_quarantine.kind,
            "incomplete_initialization_snapshot"
        );
        assert!(
            initialization_quarantine
                .reason
                .contains("exact non-null Note for Yourself"),
            "the null operator attestation must not become authoritative: {report:?}"
        );
        assert_eq!(
            report.initialization_authority.as_deref(),
            Some("operator attestation data/traces/recordings/profile-attestation.json")
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
        assert!(
            report.comparable_actions >= 3,
            "Neow grid replay must compare the settled deck checkpoint: {report:?}"
        );
        assert!(report.replayed_actions >= 3);
        assert!(
            report
                .inferred_actions
                .iter()
                .any(|inferred| inferred.before_idx == 2
                    && inferred.action == GameAction::SelectRewardItem(0)),
            "Neow grid replay must expose its uniquely legal typed opener: {report:?}"
        );
        assert!(
            report.inferred_actions.iter().any(|inferred| {
                inferred.before_idx == 2
                    && matches!(inferred.action, GameAction::ChooseRewardOption { .. })
            }),
            "Neow grid replay must expose the deck-proven canonical choice: {report:?}"
        );
        assert!(
            report
                .first_divergence
                .as_ref()
                .is_none_or(|divergence| divergence.idx > 2),
            "Neow grid replay may only stop at a later real divergence: {report:?}"
        );
    }
}

#[test]
fn bundle_neow_grid_upgrade_from_ordered_deck_delta() {
    // NeowReward.update upgrades selectedCards[0], clears the grid selection,
    // then returns to Neow's final Continue. Java: NeowReward.java:119-170,
    // 291-293. The recorder preserves the resulting ordered master deck.
    let bundle = recording_prefix("-8362019926497694510-WATCHER-20260720-174739", 3);
    let report = compare_recording_bundle(&bundle).unwrap();

    assert_eq!(report.status, BundleComparisonStatus::Uncertified);
    assert_eq!(report.comparable_actions, 3);
    assert_eq!(report.matched_actions, 3);
    assert_eq!(report.replayed_actions, 3);
    assert!(report.first_divergence.is_none());
    assert_eq!(report.inferred_actions.len(), 2);
    assert_eq!(
        report.inferred_actions[0].action,
        GameAction::SelectRewardItem(0)
    );
    assert_eq!(
        report.inferred_actions[1].action,
        GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 8,
        }
    );
    assert!(report.inferred_actions[1]
        .reason
        .contains("uniquely identifies"));
    assert!(!report.unverified_actions.iter().any(|entry| entry.idx == 2));
}

#[test]
fn bundle_neow_grid_equivalent_duplicate_removal() {
    // NeowReward.update removes selectedCards[0] from masterDeck. Java:
    // NeowReward.java:119-170, 241-246. Removing any one of the four identical,
    // non-bottled Defends yields the same ordered deck but not the same instance.
    let bundle = recording_prefix("3679001076098606203-WATCHER-20260720-190036", 3);
    let report = compare_recording_bundle(&bundle).unwrap();

    assert_eq!(report.status, BundleComparisonStatus::Uncertified);
    assert_eq!(report.comparable_actions, 3);
    assert_eq!(report.matched_actions, 3);
    assert_eq!(report.replayed_actions, 3);
    assert!(report.first_divergence.is_none());
    assert_eq!(report.inferred_actions.len(), 2);
    assert_eq!(
        report.inferred_actions[0].action,
        GameAction::SelectRewardItem(0)
    );
    assert_eq!(
        report.inferred_actions[1].action,
        GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 4,
        }
    );
    assert!(report.inferred_actions[1]
        .reason
        .contains("equivalent duplicate card instance"));
    assert!(report.unverified_actions.iter().any(|entry| {
        entry.idx == 2
            && entry
                .reason
                .contains("does not distinguish equivalent duplicate deck-card instances")
    }));
}

#[test]
fn bundle_infers_the_uniquely_forced_synthetic_boss_click() {
    // DungeonMapScreen routes controller focus to bossHb when no ordinary map
    // node remains, while TraceLab's path patch observes MapRoomNode clicks.
    // The next combat checkpoint and the sole canonical destination therefore
    // prove this omitted UI action without choosing among alternatives.
    // Java: DungeonMapScreen.java (`nodes.isEmpty()`) and DungeonMap.java.
    let bundle = recording_prefix("-5884681071377138867-WATCHER-20260720-194423", 124);
    let report = compare_recording_bundle(&bundle).unwrap();

    assert!(report.inferred_actions.iter().any(|inferred| {
        inferred.before_idx == 123
            && inferred.action == GameAction::ChoosePath(0)
            && inferred.reason.contains("synthetic boss destination")
    }));
    assert!(
        report
            .first_divergence
            .as_ref()
            .is_none_or(|divergence| divergence.idx > 123),
        "the uniquely forced boss entry must make the recorded potion action reachable: {report:?}"
    );
}

#[test]
fn bundle_maps_boss_relic_identity_through_java_reward_staging() {
    // Boss combat Proceed enters TreasureRoomBoss, BossChest.open opens the
    // one-item BossRelicSelectScreen, and the relic image selects one of its
    // three named choices. The legacy recorder emits only that final identity.
    // Java: ProceedButton.java, BossChest.java, BossRelicSelectScreen.java.
    let bundle = recording_prefix("-5884681071377138867-WATCHER-20260720-194423", 137);
    let report = compare_recording_bundle(&bundle).unwrap();

    let staged = report
        .inferred_actions
        .iter()
        .filter(|inferred| inferred.before_idx == 135)
        .map(|inferred| inferred.action.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        staged,
        vec![
            GameAction::LeaveRewards,
            GameAction::OpenChest,
            GameAction::SelectRewardItem(0),
        ]
    );
    assert!(report.inferred_actions.iter().any(|inferred| {
        inferred.before_idx == 136
            && inferred.action == GameAction::Proceed
            && inferred.reason.contains("DungeonTransitionScreen")
    }));
    assert!(
        report
            .first_divergence
            .as_ref()
            .is_none_or(|divergence| divergence.idx > 136),
        "the recorded VioletLotus and next-act path must resolve the omitted staging: {report:?}"
    );
}

#[test]
fn bundle_uses_following_card_identity_to_open_the_prayer_wheel_reward() {
    // CombatRewardScreen adds a second card item for Prayer Wheel. TraceLab's
    // REWARD_TAKE callback omits its item index, but the following CARD_REWARD
    // identity uniquely proves which offer Java opened.
    // Java: CombatRewardScreen.java::setupItemReward.
    let bundle = recording_prefix("-5884681071377138867-WATCHER-20260720-194423", 278);
    let report = compare_recording_bundle(&bundle).unwrap();

    assert_eq!(report.comparable_actions, 278, "{report:?}");
    assert!(report.first_divergence.is_none(), "{report:?}");
}

#[test]
fn bundle_quarantines_unindexed_card_preview_before_retrying_full_potion_reward() {
    // A full inventory makes Ambrosia's first RewardItem.claimReward return
    // false. The player then opens and skips the second Prayer Wheel card
    // reward, uses/discards a potion, and clicks the still-present Ambrosia.
    // TraceLab does not observe SkipCardButton.closeCurrentScreen.
    // Java: RewardItem.java::claimReward, SkipCardButton.java::update, and
    // CardRewardScreen.java::takeReward.
    let bundle = recording_prefix("-5884681071377138867-WATCHER-20260720-194423", 318);
    let report = compare_recording_bundle(&bundle).unwrap();

    assert_eq!(report.comparable_actions, 318, "{report:?}");
    assert!(report.first_divergence.is_none(), "{report:?}");
    assert!(report
        .ignored_recorder_actions
        .iter()
        .any(|ignored| { ignored.idx == 314 && ignored.reason.contains("skipped preview") }));
}

#[test]
fn bundle_quarantines_unindexed_prayer_wheel_previews_before_final_card_pick() {
    // Two identical-state REWARD_TAKE CARD callbacks are followed by another
    // opener and the persistent EmptyMind pick. RewardItem.claimReward omits
    // the item index and SkipCardButton has no TraceLab hook, so only the final
    // opener is uniquely tied to a card offer.
    let bundle = recording_prefix("-5884681071377138867-WATCHER-20260720-194423", 337);
    let report = compare_recording_bundle(&bundle).unwrap();

    assert_eq!(report.comparable_actions, 337, "{report:?}");
    assert!(report.first_divergence.is_none(), "{report:?}");
    assert_eq!(
        report
            .ignored_recorder_actions
            .iter()
            .filter(|ignored| matches!(ignored.idx, 333 | 334))
            .count(),
        2
    );
    assert!(report.unverified_actions.iter().any(|entry| {
        entry.idx == 335 && entry.reason.contains("multiple Prayer Wheel offers")
    }));
}

#[test]
fn bundle_quarantines_both_unindexed_prayer_wheel_previews_when_all_are_skipped() {
    // Both card items are previewed and then the player leaves without a
    // CARD_REWARD pick. The unchanged ordered deck proves the net skip, while
    // the legacy schema still cannot identify either RewardItem index.
    let bundle = recording_prefix("-5884681071377138867-WATCHER-20260720-194423", 356);
    let report = compare_recording_bundle(&bundle).unwrap();

    assert_eq!(report.comparable_actions, 356, "{report:?}");
    assert!(report.first_divergence.is_none(), "{report:?}");
    assert!(report
        .ignored_recorder_actions
        .iter()
        .any(|ignored| ignored.idx == 354 && ignored.reason.contains("skipped preview")));
}

#[test]
fn bundle_quarantines_canceled_smith_grid_before_following_campfire_choice() {
    // SmithOption only queues CampfireSmithEffect; the actual upgrade occurs
    // after GridSelectScreen returns a card. The unchanged deck and immediate
    // Recall prove the grid was canceled, which TraceLab does not record.
    // Java: SmithOption.java::useOption and CampfireSmithEffect.java::update.
    let bundle = recording_prefix("-5884681071377138867-WATCHER-20260720-194423", 396);
    let report = compare_recording_bundle(&bundle).unwrap();

    assert_eq!(report.comparable_actions, 396, "{report:?}");
    assert!(report.first_divergence.is_none(), "{report:?}");
    assert!(report
        .ignored_recorder_actions
        .iter()
        .any(|ignored| { ignored.idx == 394 && ignored.reason.contains("upgrade grid") }));
}

#[test]
fn coupled_or_incomplete_legacy_prefix_is_never_reported_as_a_match() {
    let path = recordings_root().join("-5884681071377138867-WATCHER-20260720-194423");
    let mut bundle = load_recording_bundle(path).unwrap();
    bundle.actions.truncate(51);
    bundle.records.truncate(51);
    bundle.meta.records = 51;
    // A prefix cannot retain the full run's later RUN_END lifecycle. Keeping
    // it would correctly assert that action 51 must already be terminal and
    // therefore produce a terminal-lifecycle divergence rather than testing
    // the intended incomplete-prefix status.
    bundle.run_end = None;
    bundle.meta.status = "IN_PROGRESS".to_string();

    let report = compare_recording_bundle(&bundle).unwrap();
    assert_eq!(report.status, BundleComparisonStatus::Uncertified);
    assert_eq!(report.comparable_actions, 51);
    assert!(report.coupled_actions > 0);
    assert!(report.matched_actions < report.comparable_actions);
    assert_eq!(
        report
            .initialization_quarantine
            .as_ref()
            .map(|quarantine| quarantine.kind.as_str()),
        Some("incomplete_initialization_snapshot")
    );
    assert_eq!(
        report.initialization_authority.as_deref(),
        Some("operator attestation data/traces/recordings/profile-attestation.json")
    );
}

#[test]
fn valid_bundle_intake_aligns_script_trace_and_metadata() {
    let path = recordings_root().join("-5884681071377138867-WATCHER-20260720-194423");
    let bundle = load_recording_bundle(path).unwrap();
    assert_eq!(bundle.meta.records, 607);
    assert_eq!(bundle.actions.len(), 607);
    assert_eq!(bundle.records.len(), 607);
    assert_eq!(
        bundle.meta.profile_authority.as_deref(),
        Some("operator attestation data/traces/recordings/profile-attestation.json")
    );
    assert_eq!(
        bundle
            .meta
            .profile
            .as_ref()
            .map(|profile| profile.note_for_yourself_card.as_str()),
        Some("IronWave")
    );
    assert_eq!(
        bundle
            .meta
            .profile_quarantine
            .as_ref()
            .map(|quarantine| quarantine.kind.as_str()),
        Some("incomplete_operator_profile_attestation")
    );
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
        note_for_yourself_upgrades: 0,
        highest_unlocked_ascension: 20,
        is_daily_run: false,
        is_trial: false,
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

    let mut upgraded_note = duplicate.clone();
    upgraded_note.locked_relics.pop();
    upgraded_note.note_for_yourself_upgrades = 1;
    assert_eq!(
        upgraded_note
            .to_engine_profile()
            .unwrap()
            .note_for_yourself_card,
        "IronWave+"
    );
    upgraded_note.note_for_yourself_upgrades = 2;
    assert!(upgraded_note
        .check_version()
        .unwrap_err()
        .contains("not currently replayable"));
    upgraded_note.note_for_yourself_upgrades = 0;
    upgraded_note.is_daily_run = true;
    assert!(upgraded_note
        .check_version()
        .unwrap_err()
        .contains("standard-run certification"));
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
fn profile_presence_alone_cannot_certify_run_initialization() {
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
        initial: None,
        environment: None,
        profile_authority: None,
        profile_quarantine: None,
    };
    let bundle = RecordingBundle {
        path: PathBuf::new(),
        meta: meta.clone(),
        actions: Vec::new(),
        records: Vec::new(),
        run_end: None,
        lifecycle: Default::default(),
    };
    let legacy = compare_recording_bundle(&bundle).unwrap();
    assert_eq!(
        legacy
            .initialization_quarantine
            .as_ref()
            .map(|quarantine| quarantine.kind.as_str()),
        Some("incomplete_initialization_snapshot")
    );

    let authoritative = RecordingBundle {
        meta: RecordingMeta {
            profile: Some(TraceProfileSnapshot {
                v: 1,
                note_for_yourself_card: "IronWave".to_string(),
                note_for_yourself_upgrades: 0,
                highest_unlocked_ascension: 20,
                is_daily_run: false,
                is_trial: false,
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
    let report = compare_recording_bundle(&authoritative).unwrap();
    let quarantine = report
        .initialization_quarantine
        .expect("an exact profile does not replace the process-global envelope");
    assert_eq!(quarantine.kind, "incomplete_initialization_snapshot");
    assert!(quarantine.reason.contains("post-generation state"));
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
