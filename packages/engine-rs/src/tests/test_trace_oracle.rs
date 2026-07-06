//! U05 — trace oracle tests (`docs/goal/TOOLING.md` § T5, `docs/goal/UNITS.md` U05).
//!
//! No real Java golden exists yet (that's U04/U06's job), so this suite is
//! synthetic-fixture based: it generates a Rust trace in-process for a tiny
//! scripted sequence from a fixed seed, treats a clone of it as the "java"
//! side, and diffs via `sts_engine::trace::diff_records` — the exact same
//! differ `bin/trace_replay.rs` uses. It also replays any committed golden
//! under `data/traces/java/*.jsonl` against its matching script (currently
//! none — the loop below no-ops gracefully when the directory is empty or
//! missing, per T5's "part of the lib suite" requirement).
//!
//! Seed 0 / ascension 0 (Watcher) is used because floor 1's first map node
//! is naturally a `Monster` room (`RoomType::Monster`) reached by
//! `ChoosePath(0)` with no map-state hacking, and Neow option 0 for this
//! seed is "Gain 100 gold" (`NeowChoiceEffect::GainGold`, no cardRng
//! consumption per `docs/vault/rng-system-analysis.md`), keeping the
//! fixture's RNG trajectory simple and reproducible.

use std::fs;

use crate::trace::{diff_records, ActionScript, DivergenceStatus, ScriptStopCondition, TraceAction};

/// The tiny scripted sequence used by both tests below: resolve Neow, take
/// the first map path into floor 1 combat (vs a lone Cultist for seed 0),
/// play the Defend card on ourselves, then end the turn.
fn tiny_fixture_script() -> ActionScript {
    ActionScript {
        v: 1,
        seed: crate::seed::seed_to_string(0),
        character: "WATCHER".to_string(),
        ascension: 0,
        stop: ScriptStopCondition { max_floor: None, max_actions: None },
        actions: vec![
            TraceAction::Neow { choice: 0 },
            TraceAction::Path { choice: 0 },
            TraceAction::PlayCard { hand_idx: 2, target: -1, card_id: Some("Defend_P".to_string()) },
            TraceAction::EndTurn,
        ],
    }
}

#[test]
fn synthetic_self_diff_matches() {
    let script = tiny_fixture_script();
    let rust_records = crate::trace::replay_script(&script).expect("fixture script must replay cleanly");
    assert_eq!(rust_records.len(), 4, "expected one record per scripted action");

    // Treat an identical clone as the "java" side.
    let java_records = rust_records.clone();

    let report = diff_records("synthetic-fixture", &script.seed, &java_records, &rust_records, &[]);
    assert_eq!(report.status, DivergenceStatus::Match, "identical traces must report match: {report:?}");
    assert_eq!(report.matched_actions, 4);
    assert_eq!(report.total_actions, 4);
    assert!(report.first_divergence.is_none());
    assert!(report.masked.is_empty());
    assert!(report.secondary.is_empty());
}

#[test]
fn doctored_hp_and_rng_are_reported_as_first_divergence() {
    let script = tiny_fixture_script();
    let rust_records = crate::trace::replay_script(&script).expect("fixture script must replay cleanly");
    assert!(rust_records.len() >= 2, "fixture must have at least 2 records to doctor the 2nd one");

    // Doctor the "java" side at record index 1 (after PLAY_CARD): bump the
    // player's hp by 1 (an impossible value given the real replay) and the
    // `ai` rng counter by 1. hp is checked first among player fields but
    // rng counters are checked first overall (T3): the report's
    // first_divergence must land on the rng path, not hp.
    let mut java_records = rust_records.clone();
    let doctored = &mut java_records[1];
    let original_hp = doctored.post.player.hp;
    doctored.post.player.hp = original_hp + 1;
    let original_ai = *doctored.post.rng.get("ai").expect("ai counter must be tracked");
    doctored.post.rng.insert("ai".to_string(), original_ai + 1);

    let report = diff_records("synthetic-fixture-doctored", &script.seed, &java_records, &rust_records, &[]);

    assert_eq!(report.status, DivergenceStatus::Diverged);
    assert_eq!(report.matched_actions, 1, "record 0 (after NEOW) is undoctored and must still match");

    let first = report.first_divergence.expect("must report a first_divergence");
    assert_eq!(first.idx, 1);
    // RNG counters are diffed first in canonical order, so the `ai` counter
    // diff must be the reported path, not `post.player.hp` (also diffed,
    // but demoted to `secondary` since it's the same divergent record).
    assert_eq!(first.path, "post.rng.ai", "rng counters must be diagnosed first: {first:?}");
    assert_eq!(first.java, serde_json::json!(original_ai + 1));
    assert_eq!(first.rust, serde_json::json!(original_ai));
    assert_eq!(first.rng_at_divergence.java.get("ai"), Some(&(original_ai + 1)));
    assert_eq!(first.rng_at_divergence.rust.get("ai"), Some(&original_ai));

    // The hp diff at the same record must still surface, just secondary.
    let hp_secondary = report
        .secondary
        .iter()
        .find(|diff| diff.path == "post.player.hp")
        .expect("hp diff must appear in secondary diffs for the same divergent record");
    assert_eq!(hp_secondary.java, serde_json::json!(original_hp + 1));
    assert_eq!(hp_secondary.rust, serde_json::json!(original_hp));
}

#[test]
fn truncated_trace_is_never_reported_as_match() {
    // A "java" trace that stopped early (e.g. the golden mint truncated,
    // or a script only partially replayed on one side) must never be
    // silently reported as "match" even though every record it does have
    // matches exactly.
    let script = tiny_fixture_script();
    let rust_records = crate::trace::replay_script(&script).expect("fixture script must replay cleanly");
    assert!(rust_records.len() > 1);

    let java_records = rust_records[..1].to_vec();
    let report = diff_records("synthetic-fixture-truncated", &script.seed, &java_records, &rust_records, &[]);

    assert_eq!(
        report.status,
        DivergenceStatus::Diverged,
        "a length mismatch must never report match: {report:?}"
    );
    assert_eq!(report.matched_actions, 1);
    assert_eq!(report.total_actions, rust_records.len() as u64);
    assert!(report.first_divergence.is_some());
}

/// If `data/traces/java/*.jsonl` goldens exist (minted by U04/U06), replay
/// each against its matching `data/traces/scripts/<stem>.json` and diff —
/// per T5, "part of the lib suite". Today the directory doesn't exist yet,
/// so this must no-op cleanly rather than fail.
#[test]
fn replays_committed_goldens_if_any_exist() {
    let repo_root = repo_root();
    let goldens_dir = repo_root.join("data/traces/java");
    let scripts_dir = repo_root.join("data/traces/scripts");

    let Ok(entries) = fs::read_dir(&goldens_dir) else {
        // No goldens minted yet (U04/U06 not landed) — graceful no-op.
        return;
    };

    let mut checked = 0usize;
    for entry in entries {
        let entry = entry.expect("readable dir entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("golden filename must have a stem")
            .to_string();
        let script_path = scripts_dir.join(format!("{stem}.json"));
        assert!(
            script_path.exists(),
            "golden {path:?} has no matching script at {script_path:?}"
        );

        let script_text = fs::read_to_string(&script_path).expect("script must be readable");
        let script: ActionScript = serde_json::from_str(&script_text).expect("script must parse");
        script.check_version().expect("script version must be supported");

        let java_text = fs::read_to_string(&path).expect("golden must be readable");
        let mut lines = java_text.lines().filter(|l| !l.trim().is_empty());
        let header_line = lines.next().expect("golden must have a header line");
        let header: crate::trace::TraceHeader =
            serde_json::from_str(header_line).expect("golden header must parse");
        header.check_version().expect("golden header version must be supported");
        let java_records: Vec<crate::trace::TraceRecord> = lines
            .map(|line| serde_json::from_str(line).expect("golden record must parse"))
            .collect();

        let rust_records = crate::trace::replay_script(&script)
            .unwrap_or_else(|err| panic!("script {stem} failed to replay: {err}"));

        // Divergences against real goldens are U08+'s backlog (per U06's
        // acceptance criteria) — this test only asserts the pipeline runs
        // end-to-end without panicking, not that it matches yet.
        let _report = diff_records(&stem, &script.seed, &java_records, &rust_records, &[]);
        checked += 1;
    }

    let _ = checked; // informational; zero is a valid outcome today
}

fn repo_root() -> std::path::PathBuf {
    // src/tests/test_trace_oracle.rs -> packages/engine-rs/src/tests -> ../../../.. = repo root
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR must have packages/ and repo root ancestors")
        .to_path_buf()
}
