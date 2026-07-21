use std::fs;
use std::path::{Path, PathBuf};

use crate::trace::bundle::{
    compare_recording_bundle, load_recording_bundle, load_recording_meta, BundleComparisonStatus,
};

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
            report.unverified_action_semantics, 1,
            "the recorder's index-only Neow selection must remain unverified: {report:?}"
        );
        eprintln!(
            "{} matches through {}/{}; first={:?}",
            report.bundle,
            report.matched_actions,
            report.total_actions,
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
        direct_reward.matched_actions >= 17,
        "source-derived replay prefix regressed: {direct_reward:?}"
    );
    if direct_reward.matched_actions == 17 {
        assert_eq!(direct_reward.matched_checkpoints, 15);
        assert_eq!(direct_reward.coupled_actions, 2);
        assert!(
            direct_reward
                .first_divergence
                .as_ref()
                .is_some_and(|divergence| divergence
                    .detail
                    .contains("reward skip/leave action is missing")),
            "the frozen action-17 recorder gap must stay explicit: {direct_reward:?}"
        );
    }

    let grid_followups: Vec<_> = reports
        .iter()
        .filter(|report| report.bundle != direct_reward.bundle)
        .collect();
    assert_eq!(grid_followups.len(), 4);
    for report in grid_followups {
        assert_eq!(report.matched_actions, 2);
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
fn valid_bundle_intake_aligns_script_trace_and_metadata() {
    let path = recordings_root().join("-5884681071377138867-WATCHER-20260720-194423");
    let bundle = load_recording_bundle(path).unwrap();
    assert_eq!(bundle.meta.records, 607);
    assert_eq!(bundle.actions.len(), 607);
    assert_eq!(bundle.records.len(), 607);
    assert_eq!(bundle.actions[0].action_type, "NEOW");
    assert_eq!(bundle.records[606].idx, 606);
}
