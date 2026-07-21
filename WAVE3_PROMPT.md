<task>
Wave 3: close the oracle loop on real human data. 14 human-recorded run bundles (3 complete Watcher A0 victories; enumerate via each bundle's meta.json — do not assume seeds) at `data/traces/recordings/` (bundle = meta.json + script.jsonl + trace.jsonl.gz). Your job: make the engine replay and diff them, then fix what diverges. Base: the merged tip of the wave-2 stacked PRs (rng-core-java → run-generation-parity → full-run-actions-v2 → core-checkpoint-normalization → watcher-a0-oracle-closure); create `codex/oracle-loop-wave3` from it.

Ground truth: `decompiled/java-src/` (distillation: `reference/extracted/methods/`). Contracts: `docs/work_units/script-schema-v2.md` (naming is an API commitment), closure scorecard `docs/work_units/audit-reports/watcher-a0-oracle-closure.md` (this wave = its Remaining Merge Gates 2, 4, 5). `packages/harness-java/` stays read-reference only; recorder-side changes are scoped separately for the human operator.

Scope, in order:

1. LANGUAGE-NEUTRAL ORACLE-STATE PROJECTION (gate 2). Freeze a versioned wire DTO for post-action state that BOTH the Java recorder's trace.jsonl and the Rust replay can emit: player vitals/stance/gold, ordered piles, per-enemy hp/block/intent/move_history/powers, relic ids+counters, potions, and all 13 RNG counters. Spec it in `docs/work_units/oracle-state-v2.md`; implement the Rust emitter + serde; add one-field-mutation corruption tests mirroring the v1 differ's. `CoreCheckpoint` stays internal — do not expose it as the DTO.

2. BUNDLE INTAKE + DIFF HARNESS (gate 4 infra). Teach `trace_replay` to consume a recording bundle: parse its script.jsonl (recorder dialect → canonical v2 actions; document any mapping quirks in the schema doc rather than silently coercing), replay through `RunEngine`, project state per (1), and diff against the bundle's trace records for whatever fields the recorder already emits (grow coverage as the recorder catches up — absent fields are skipped-with-count, never silently). Exit codes and first-divergence report per the v1 differ conventions. Wire committed bundles into `test_trace_oracle` (report-only until B2: generate reports, never fail the suite on divergence — fail only on crashes/schema errors).

3. THE DIVERGENCE GRIND (gate 4). Replay the two A0 victory bundles + the A0 death bundle. Take the EARLIEST divergence, root-cause against decompiled Java, fix with citation + source-derived test, flip the relevant `docs/goal/ledger.json` rows via `scripts/ledger.sh`, repeat. Effort-cap per `docs/goal/GOAL.md` Edge-Case Policy (quarantine with DEV-NNN + mask, never stall). Track progress as "bundle X matches through action N/697" in the report — that number only goes up.

4. NEOW SEMANTIC COMPARISON (gate 5, small). Record/compare all four generated Neow option payloads plus the selected payload semantically, per the closure report — engine side + oracle-state DTO field; note the recorder-side requirement in the schema doc for the operator.

5. IF BLOCKED on recorder-emitted data gaps: write precise requirements to `data/traces/requests/wave3-recorder-needs.md` and continue with other scope — never modify the Java harness, never launch the game.

Verification every step: `./scripts/test_engine_rs.sh test --lib` (3,016+, count only up), `./scripts/test_engine_rs.sh check --all-targets`, and the bundle replay reports. Conventions per AGENTS.md (citations, ledger discipline, stacked PRs, commit prefixes). End every session reporting: deepest matched action per bundle, divergences fixed (what was wrong), rows flipped, quarantines, recorder requests filed.
</task>
