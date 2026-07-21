//! `trace_replay` — U05: replay an action script through `RunEngine`, emit a
//! trace JSONL, and diff it against a frozen Java golden trace.
//!
//! Per `docs/goal/TOOLING.md` § T3:
//! ```text
//! trace_replay --script s.json --java-trace t.jsonl [--out rust.jsonl] \
//!              --diff report.json [--masks docs/goal/masks.json]
//! ```
//!
//! Exit codes: `0` on match (or masked-only divergence), `1` on divergence,
//! `2` on error (bad args, unsupported action, parse failure).
//!
//! This is the crate's first `[[bin]]` target — it depends on `sts_engine`
//! as an ordinary external crate (only `pub` items are visible from here;
//! see `RunEngine::rng_counters`/`CombatEngine::rng_counters` and the
//! `sts_engine::trace` replay-support functions added alongside this bin
//! for exactly that reason) and has zero effect on the lib's own
//! `#[cfg(test)]` suite. All actual replay/diff logic lives in
//! `src/trace.rs` so `tests/test_trace_oracle.rs` (part of the lib suite)
//! can exercise the identical code path in-process; this file is CLI + file
//! I/O glue only.

use std::fs;
use std::process::ExitCode;

use sts_engine::trace::{
    self, ActionScript, DivergenceStatus, TraceHeader, TraceRecord,
};
use sts_engine::trace::v2::{replay_action_script_v2, ActionScriptV2, TraceEnvelopeV2};

fn main() -> ExitCode {
    match run() {
        Ok(status) => match status {
            DivergenceStatus::Match => ExitCode::from(0),
            DivergenceStatus::Diverged => ExitCode::from(1),
            DivergenceStatus::Error => ExitCode::from(2),
        },
        Err(msg) => {
            eprintln!("trace_replay: error: {msg}");
            ExitCode::from(2)
        }
    }
}

// ===========================================================================
// CLI args
// ===========================================================================

#[derive(Debug, Default)]
struct Args {
    script: Option<String>,
    java_trace: Option<String>,
    out: Option<String>,
    diff: Option<String>,
    masks: Option<String>,
    help: bool,
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut args = Args::default();
    let mut iter = raw.iter();
    while let Some(flag) = iter.next() {
        let value = |flag: &str, iter: &mut std::slice::Iter<String>| -> Result<String, String> {
            iter.next()
                .cloned()
                .ok_or_else(|| format!("{flag} requires a value"))
        };
        match flag.as_str() {
            "--script" => args.script = Some(value(flag, &mut iter)?),
            "--java-trace" => args.java_trace = Some(value(flag, &mut iter)?),
            "--out" => args.out = Some(value(flag, &mut iter)?),
            "--diff" => args.diff = Some(value(flag, &mut iter)?),
            "--masks" => args.masks = Some(value(flag, &mut iter)?),
            "--help" | "-h" => args.help = true,
            other => return Err(format!("unrecognized flag '{other}'")),
        }
    }
    Ok(args)
}

fn usage() -> &'static str {
    "usage:\n  v1 compare: trace_replay --script <script.json> --java-trace <golden.jsonl> [--out <rust.jsonl>] --diff <report.json> [--masks <masks.json>]\n  v2 replay:  trace_replay --script <script-v2.json> --out <rust-v2.jsonl>"
}

// ===========================================================================
// Top-level flow
// ===========================================================================

fn run() -> Result<DivergenceStatus, String> {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let args = parse_args(&raw_args)?;

    if args.help || raw_args.is_empty() {
        println!("{}", usage());
        return Ok(if args.help {
            DivergenceStatus::Match
        } else {
            DivergenceStatus::Error
        });
    }

    let script_path = args
        .script
        .as_ref()
        .cloned()
        .ok_or_else(|| format!("--script is required\n{}", usage()))?;
    let script_text = fs::read_to_string(&script_path)
        .map_err(|err| format!("failed to read script '{script_path}': {err}"))?;
    let script_value: serde_json::Value = serde_json::from_str(&script_text)
        .map_err(|err| format!("failed to parse script '{script_path}': {err}"))?;
    if script_value.get("schema").is_some() {
        return replay_v2_script(&args, &script_path, &script_text);
    }

    let java_trace_path =
        args.java_trace.ok_or_else(|| format!("--java-trace is required\n{}", usage()))?;
    let diff_path = args.diff.ok_or_else(|| format!("--diff is required\n{}", usage()))?;
    let script: ActionScript = serde_json::from_str(&script_text)
        .map_err(|err| format!("failed to parse script '{script_path}': {err}"))?;
    script.check_version()?;

    let java_trace_text = fs::read_to_string(&java_trace_path)
        .map_err(|err| format!("failed to read java trace '{java_trace_path}': {err}"))?;
    let (java_header, java_records) = parse_trace_jsonl(&java_trace_text)?;
    let _ = java_header; // header is validated but not otherwise consulted below

    let masks = match &args.masks {
        Some(path) => {
            let text = fs::read_to_string(path)
                .map_err(|err| format!("failed to read masks '{path}': {err}"))?;
            trace::parse_masks(&text)?
        }
        None => Vec::new(),
    };

    let rust_records = trace::replay_script(&script)?;

    if let Some(out_path) = &args.out {
        write_jsonl(out_path, &script, &rust_records)?;
    }

    let script_stem = script_stem_name(&script_path);
    let report = trace::diff_records(&script_stem, &script.seed, &java_records, &rust_records, &masks);

    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|err| format!("failed to serialize report: {err}"))?;
    fs::write(&diff_path, report_json)
        .map_err(|err| format!("failed to write report '{diff_path}': {err}"))?;

    Ok(report.status)
}

fn replay_v2_script(
    args: &Args,
    script_path: &str,
    script_text: &str,
) -> Result<DivergenceStatus, String> {
    if args.java_trace.is_some() || args.diff.is_some() || args.masks.is_some() {
        return Err(
            "v2 Java comparison is not available until the language-neutral oracle projection is frozen; replay with --out only"
                .to_string(),
        );
    }
    let out_path = args
        .out
        .as_deref()
        .ok_or_else(|| format!("--out is required for a v2 replay\n{}", usage()))?;
    let script: ActionScriptV2 = serde_json::from_str(script_text)
        .map_err(|err| format!("failed to parse v2 script '{script_path}': {err}"))?;
    let envelopes = replay_action_script_v2(&script)?;
    write_v2_jsonl(out_path, &envelopes)?;
    Ok(DivergenceStatus::Match)
}

fn script_stem_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn write_jsonl(path: &str, script: &ActionScript, records: &[TraceRecord]) -> Result<(), String> {
    let header = trace::header_for_script(script);
    let mut out = serde_json::to_string(&header).map_err(|err| err.to_string())?;
    out.push('\n');
    for record in records {
        out.push_str(&serde_json::to_string(record).map_err(|err| err.to_string())?);
        out.push('\n');
    }
    fs::write(path, out).map_err(|err| format!("failed to write '{path}': {err}"))
}

fn write_v2_jsonl(path: &str, envelopes: &[TraceEnvelopeV2]) -> Result<(), String> {
    let mut out = String::new();
    for envelope in envelopes {
        out.push_str(&serde_json::to_string(envelope).map_err(|err| err.to_string())?);
        out.push('\n');
    }
    fs::write(path, out).map_err(|err| format!("failed to write '{path}': {err}"))
}

// ===========================================================================
// Trace JSONL parsing (header + records)
// ===========================================================================

#[derive(serde::Deserialize)]
struct MetaLine {
    #[serde(default)]
    kind: Option<String>,
}

fn parse_trace_jsonl(text: &str) -> Result<(TraceHeader, Vec<TraceRecord>), String> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines.next().ok_or("trace file is empty")?;
    let header: TraceHeader =
        serde_json::from_str(header_line).map_err(|err| format!("bad trace header: {err}"))?;
    header.check_version()?;

    let mut records = Vec::new();
    for (line_no, line) in lines.enumerate() {
        // Skip meta records (header/end sentinels carry a `kind`; data records do not).
        if serde_json::from_str::<MetaLine>(line)
            .map(|meta| meta.kind.is_some())
            .unwrap_or(false)
        {
            continue;
        }
        let record: TraceRecord = serde_json::from_str(line)
            .map_err(|err| format!("bad trace record on line {}: {err}", line_no + 2))?;
        record.check_version()?;
        records.push(record);
    }
    Ok((header, records))
}
