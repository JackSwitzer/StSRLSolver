<task>
Overnight fix wave. Base: branch `codex/engine-deep-audit` (2,883 lib tests green + 11 `#[ignore]`d audit reproducers). Create branch `codex/eda-fix-wave1` from it and work there. The findings register is `docs/work_units/audit-reports/engine-deep-audit.md` (EDA-001..035); the sequencing map is `docs/work_units/sim-completion-map.md`. Ground truth for all behavior is `decompiled/java-src/com/megacrit/cardcrawl/` (agent distillation: `reference/extracted/methods/`).

Objective: convert confirmed findings into fixed, Java-cited, test-proven engine behavior — maximizing the SCOREBOARD below, never gaming it. You are a swarm: parallelize independent findings, serialize conflicting ones, and end with a judge pass that grades the night honestly.
</task>

<scoreboard>
All gates are commands with exit codes — no judgment calls. Final score = the scorecard table produced by the judge pass.

- G1 (per fix): the finding's reproducer in `packages/engine-rs/src/tests/test_audit_repros.rs` runs WITHOUT `#[ignore]` and passes. Prove red→green: show the repro failing at the base commit (`git stash` / run at branch point) and passing after the fix, and record both command outputs in the commit message or scorecard.
- G2 (always): `./scripts/test_engine_rs.sh test --lib` fully green after EVERY commit. Baseline 2,883 passing / 11 ignored. Passing count only moves up (un-ignored repros + new family tests). Ignored count only moves down.
- G3 (oracle): `scripts/trace_diff.sh data/traces/scripts/smoke-neow-floor1.json` ends with `"status": "match"` and `matched_actions == total_actions` in `logs/traces/smoke-neow-floor1/report.json`. This is the acceptance test for the Wave C run-layer items (F2 counter exposure, F4 Neow mapping, F5 potion placeholder).
- G4 (perf guard): `DYLD_FRAMEWORK_PATH=/Applications/Xcode.app/Contents/Developer/Library/Frameworks cargo bench --manifest-path packages/engine-rs/Cargo.toml --bench combat_bench` — clone_for_mcts < 800ns, full_turn_cycle < 5.2µs, get_legal_actions < 130ns (≈20% headroom over the 2026-07-15 baseline). A fix that blows a bound is reworked or reverted.
- G5 (hygiene): `cargo check --all-targets` clean; no new warnings; protected paths untouched (`data/traces/java/`, `decompiled/`, `packages/training/`, `logs/`, `runs/`, `docs/goal/` except FINDINGS cross-refs); every behavior commit cites the decompiled Java file+lines it implements.
</scoreboard>

<work_queue>
One commit per finding: `fix(EDA-NNN): <what changed>` (or `fix(F2): ...` for FINDINGS items). Priority order below; within a wave, items are independent and parallelizable unless noted.

WAVE A — S/M-effort combat P0s (do first, highest confidence):
EDA-001 victory dispatch truncation (S) · EDA-006 nested event dispatch (M) · EDA-007 Devotion Mantra remainder · EDA-009 enemy Ritual trigger timing · EDA-010 Static Discharge orb pipeline · EDA-011 Spore Cloud after final enemy death · EDA-013 player Poison owner phase · EDA-008 Necronomicon card-use semantics (M) · EDA-012 combat shuffle algorithm + stream accounting (M; touches shuffle paths — coordinate with EDA-004) · EDA-004 per-floor combat RNG seeding/topology (M; do after or with EDA-012, same subsystem).

WAVE B — L-effort P0s (start once Wave A is merging; each is a single dedicated worker):
EDA-003 + EDA-002 integer widening (i16→i32 for statuses and card misc; ONE worker, mechanical sweep, run LAST in merge order — it touches files every other fix touches) · EDA-005 seeded weighted encounter queues (port `MonsterInfo`/`populateMonsterList`/first-strong exclusions/repeat rules from Exordium/TheCity/TheBeyond + `AbstractDungeon.java:1047-1084`, on a dedicated persistent monsterRng).

WAVE C — run-layer oracle smalls (small, high-leverage; drives G3):
F2 full 13-stream counter exposure from `RunEngine::rng_counters` · F4 Neow option→blessing mapping · F5 potion empty-slot placeholder `"Potion Slot"` · trace emission of real relic counters (replaces the hardcoded `-1` in `trace.rs::build_post_state`).

STRETCH (only if waves A-C are done and gates hold):
EDA-019 Note for Yourself global → state-local (S) · EDA-021 PyO3 behind a cargo feature (default off for the core build; keep the python surface compiling under the feature) · EDA-022 additive-tolerant boundary schema versioning.

DO NOT attempt overnight: EDA-014 snapshot redesign, EDA-017 batched PUCT, EDA-027 RunEngine normalization, EDA-020/026 representation unification — these are design-heavy daytime work; the map sequences them after P0 closure.
</work_queue>

<per_fix_protocol>
1. Re-read the register entry AND the cited Java before writing code. If the Java contradicts the register, the Java wins — record the correction in the scorecard and fix to the Java.
2. Red first: run the finding's repro un-ignored at base; capture the failure.
3. Fix through the production engine path. No special-casing the repro's inputs.
4. Family sweep: search for sibling instances of the same defect class (same wrong stream elsewhere, same narrowing cast, same early-exit dispatch pattern) — fix or add a register note for each; add at least one boundary/sibling test beyond the repro.
5. Green: repro passes, full suite green, relevant bench bound holds.
6. Self-review the diff against the Java one final time before committing.
</per_fix_protocol>

<anti_gaming_rules>
- Never edit expected values in `test_audit_repros.rs` except with a decompiled-Java citation proving the audit itself misread the source; record any such correction prominently in the scorecard.
- Never delete or weaken an existing test without a contradicting Java citation in the same commit.
- Never add a trace mask to make G3 pass; masks require a DEV- register entry and none are authorized tonight.
- A fix that cannot honestly pass its gates is REVERTED before morning, with the attempt documented. A reverted honest attempt scores better than a gamed pass.
</anti_gaming_rules>

<judge_pass>
Reserve the final ~15% of the budget. The judge re-verifies every claim independently: re-runs G1 per fixed finding, G2-G5 once on the final tree, and writes `docs/work_units/audit-reports/wave1-scorecard.md`:
- header: final commit, suite counts, bench numbers, G3 verdict;
- table: one row per queue item — verdict FIXED-PROVEN / FIXED-UNPROVEN / ATTEMPTED-REVERTED / NOT-ATTEMPTED, the exact command run, exit status, and one-line note;
- a "corrections to the register" section for any EDA entry the night proved wrong;
- a "handoff" section: the 3 highest-value next actions for the morning, with evidence.
FIXED-UNPROVEN (claimed but a gate fails on re-run) is a failure state: the judge reverts that commit and re-runs G2 before finishing. Commit the scorecard as the final commit of the night.
</judge_pass>

<default_follow_through_policy>
Do not stop for routine questions. If a finding is blocked (ambiguous source, missing context), mark it ATTEMPTED-REVERTED or NOT-ATTEMPTED with the blocker documented, and move to the next item. Only halt entirely if the base branch itself is broken.
</default_follow_through_policy>

<progress_updates>
Keep updates brief and outcome-based: wave transitions, gate failures, reverts. No narration.
</progress_updates>
