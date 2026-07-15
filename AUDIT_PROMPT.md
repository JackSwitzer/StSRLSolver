<task>
You are auditing `packages/engine-rs` (a clean-room Rust simulator of Slay the Spire, ported from decompiled Java at `decompiled/java-src/com/megacrit/cardcrawl/`) in this worktree, branch `codex/engine-deep-audit` (state = PR #162, the 667-row verification sweep; 2,989 lib tests green).

Context you must internalize before starting:
- Read `docs/goal/GOAL.md`, `docs/goal/FINDINGS.md`, and `AGENTS.md` first. The decompiled Java is ground truth for behavior.
- The engine's future consumers are being REBUILT from scratch: the SwiftUI monitor app and the Python training stack (`packages/training/`) are both out of date and will be replaced. Therefore existing training/obs/monitor contracts are NOT constraints — you may propose breaking redesigns of `training_contract.rs`, `obs.rs`, and the PyO3 surface. Do not spend effort preserving compatibility with `packages/training/`.
- The primary future workload: PUCT/MCTS self-play where ONE immutable content registry serves MANY combat/run states stepped in parallel (multi-thread batch rollouts on an M4 Mac), plus trace replay for parity checking.

Audit for, in priority order:
1. BUGS and edge cases — behavior that deviates from the decompiled Java, RNG stream misuse (wrong stream, missing/extra draws), action-queue ordering errors, i16 status saturation/overflow, death/exhaust/power-removal reentrancy, X-cost and free-play edge cases, intangible/artifact ordering, stance-change loops.
2. STALE/WEAK TESTS — tests that (a) encode pre-sweep misremembered behavior instead of source-derived values (see FINDINGS F1 for the known failure pattern), (b) are tautological (assert what the implementation does rather than what Java does), (c) couple to private internals (e.g. `hidden_effect_value`, direct state pokes that skip the engine pipeline), (d) duplicate coverage across the `test_*_wave*.rs` files (some are 5,000+ lines), or (e) would stay green if the behavior regressed.
3. POOR ABSTRACTIONS — known suspects to evaluate and extend: `RunEngine` god-struct with per-relic `pending_*` bool fields (run.rs ~line 758); dual relic representation (`state.relics: Vec<String>` vs `relic_flags` bitset/counters); string IDs in hot combat state (`enemy.id: String`, relic strings compared by `has_relic("...")` every damage tick); the 24 remaining `complex_hook` cards vs the declarative effect interpreter; single out-of-combat RNG stream on `RunEngine` (Java has 13 distinct persistent streams — this is FINDINGS F2 and blocks run-level parity); trace emission hardcoding relic `counter: -1` (trace.rs ~line 796).
4. BATCHING/PARALLELISM READINESS — design review for "1 engine definition, N states": verify content/registry is fully immutable and shareable (&'static or Arc, no interior mutability), state clone is minimal (bench says 569ns — find what's still heap-allocating: Strings, Vecs, move_history), propose a `step`-many API shape for parallel PUCT workers, identify allocation churn in the hot turn loop, confirm determinism holds per-state with no thread-shared RNG or globals. Quantify with the existing criterion benches (`benches/`; binaries need `DYLD_FRAMEWORK_PATH=/Applications/Xcode.app/Contents/Developer/Library/Frameworks` because PyO3 is currently an unconditional dependency — itself a finding: it should be feature-gated).
5. CONTRACT REDESIGN — sketch a v2 boundary: sim-core crate with zero Python/obs/search deps; versioned, additive-tolerant serialization for obs/training tokens and the trace schema, so a new field never breaks a consumer with a TypeError again (this exact failure exists today: `power_cards_played_this_combat` broke the old Python contract).

Known findings to NOT re-derive (already registered): FINDINGS F1-F8, Neow blessing mapping (F4), potion-slot placeholder (F5), differ blind spots (F6). Reference them if related, don't re-report them.
</task>

<structured_output_contract>
Write the full register to `docs/work_units/audit-reports/engine-deep-audit.md` in this worktree. Format: one entry per finding, ID'd `EDA-NNN`, banded P0 (bug/correctness) / P1 (perf, batching, contract) / P2 (test hygiene, abstraction debt). Each entry: file:line evidence, one-paragraph explanation, smallest repro or failing-test sketch, proposed fix, effort S/M/L. Open the file with a ranked top-10 summary table.
In your final chat answer, return ONLY the top-10 table plus counts per band. Highest-value findings first.
</structured_output_contract>

<action_safety>
This is register-and-defer: do NOT fix findings. The only code you may add is minimal failing-repro tests in a single new file `packages/engine-rs/src/tests/test_audit_repros.rs`, each `#[ignore]`d with its EDA id in a comment. Never modify: `data/traces/java/`, `decompiled/`, `packages/training/`, `logs/`, `runs/`, existing tests. `./scripts/test_engine_rs.sh test --lib` must remain 2,989 green when you finish.
</action_safety>

<grounding_rules>
Every P0 claim must cite the decompiled Java file that contradicts the Rust, or a deterministic repro. Do not present inferences as facts; label hypotheses. A suspicion without evidence goes in a final "leads not run down" list, not the register.
</grounding_rules>

<dig_deeper_nudge>
After each confirmed issue, check its whole family: the same RNG stream misuse in sibling enemies, the same saturation risk in every i16 status write, the same String-compare pattern in every relic hook. Family sweeps are where this codebase hides bugs.
</dig_deeper_nudge>

<completeness_contract>
Cover all five audit areas before finalizing; do not exhaust the time budget on area 1. If an area yields nothing, say so explicitly with what you checked.
</completeness_contract>
