> **Superseded (2026-07-06):** the Python engine is archived under archive/2026-07-python-engine/; active spec is docs/goal/GOAL.md.

# Python → Rust Training Migration — Next-Agent TODO

Last updated: 2026-04-18.
Status: scoped, not started. Hand-off doc for the next agent.

## Why this doc exists

The training stack at `packages/training/` is **~7,049 LOC across 29 modules**. An audit (full transcript in conversation log on branch `claude/archive-legacy-python-engine`) showed only **~1,150 LOC (3 files)** are truly anchored to Python: the numpy + MLX kernel in `combat_model.py`, `inference_service.py`, `shared_memory.py`. Everything else (~5,900 LOC) is pure orchestration, JSON I/O, dataclass mirrors, hand-curated data, or thin PyO3 bridges — all portable to Rust without losing capability.

We're Watcher-A0-only for the foreseeable training horizon, the Rust engine is mature (2219/2219 lib tests, supported-scope parity audited), and the Python side keeps growing (recent commits added run_parser.py 700 LOC, run_replay.py 380 LOC, etc.). Time to invert the trend.

**MLX-via-FFI is explicitly out of scope.** No Rust binding exists for MLX (Apple's Metal-backed ML framework); rewriting the model in `candle` / `burn` / `tch` is a separate, larger project. The migration described here halves the Python footprint without touching the MLX kernel.

## Categorization rubric

Every `.py` file in `packages/training/` was placed in exactly one of:

- **A — must stay Python**: imports `numpy` heavily or wraps MLX. Cannot leave without an MLX rewrite.
- **B — pure orchestration / portable**: stdlib only (json, dataclasses, hashlib, datetime). Trivial to port.
- **C — engine PyO3 boundary**: bridges Python ↔ Rust extension. Goes away once consumers move to Rust binaries calling the engine crate directly.
- **D — mixed pipeline**: combines A + B + C (e.g. `cli.py`, `stage2_pipeline.py`, `run_replay.py`). Slimmed naturally as A/B/C below it migrates.

| Cat | Files | LOC | Share |
|----:|------:|----:|------:|
| A | 3 | 1,147 | 16% |
| B | 19 | 3,422 | 49% |
| C | 4 | 541 | 8% |
| D | 3 | 1,939 | 27% |

Anchored Python floor (Cat A): **~1,150 LOC**. Migration target: ~50% LOC reduction → ~3,500 LOC out → final ~3,500 LOC Python (kernel + minimal glue).

## File inventory

| File | LOC | Cat | Notes |
|------|----:|----:|-------|
| `__init__.py` | 104 | B | Public re-export surface |
| `__main__.py` | 5 | B | `python -m training` entry shim |
| `_serde.py` | 46 | B | Stable JSON encode + sha256 |
| `benchmark.py` | 31 | B | Frontier scoring formula |
| `benchmarking.py` | 227 | B | Pareto reports over benchmark slices |
| `bridge.py` | 74 | C | PyO3 session calls (`run_combat_puct`, `get_combat_training_state`) |
| `cli.py` | 835 | D | argparse for all phase-1 commands |
| `combat_model.py` | 490 | **A** | `MLXCombatModel`; deferred `_mlx()` import; numpy throughout |
| `config.py` | 52 | B | Frozen dataclass configs |
| `contracts.py` | 616 | B | **Redundant**: dataclass mirror of Rust contract types |
| `corpus.py` | 21 | B | `WATCHER_STARTER_DECK` constant |
| `encounters.py` | 317 | B | Static encounter catalog |
| `engine_adapter.py` | 258 | C | Hand-engineered candidate features; **no dedicated test file** |
| `engine_module.py` | 87 | C | `cargo build` + `dlopen` of `libsts_engine.dylib` |
| `entity_catalog.py` | 122 | C | Canonical-ID lookups via engine PyO3 |
| `episode_log.py` | 48 | B | JSONL append-only writer |
| `inference_service.py` | 272 | **A** | Batch inference + `CombatPolicyValueTrainer` |
| `manifests.py` | 173 | B | Run-manifest builder + sha256 hash |
| `restrictions.py` | 151 | B | Restriction-rule dataclasses + decision-surface enum |
| `run_logging.py` | 134 | B | `TrainingArtifacts` paths + writers |
| `run_parser.py` | 700 | B | `.run` parser + Neow forward-sim + master_deck reconciliation |
| `run_replay.py` | 380 | D | Replay recorded `.run` combat-by-combat |
| `seed_imports.py` | 294 | B | Hand-curated Act 1 floor scripts (Baalor / Steam) |
| `seed_suite.py` | 165 | B | `ValidationSeed` dataclasses + suite report |
| `selector.py` | 56 | B | Lexicographic frontier-line tie-breaker |
| `shared_memory.py` | 385 | **A** | numpy batch tensors |
| `stage2_pipeline.py` | 778 | D | Synthetic snapshot-corpus + PUCT + benchmark conversion |
| `system_stats.py` | 122 | B | psutil + `powermetrics` GPU sampler |
| `value_targets.py` | 106 | B | `CombatValueTarget` multi-head + potion vocab |

Test suite at `tests/training/` — 17 modules, 1,613 LOC. Most ratify the data + restrictions + selectors and would translate to Rust integration tests.

## Phase 1 — Pure data port (fresh branch, low risk)

**Goal**: extract pure-data + pure-stdlib modules into a new Rust crate `training-data` (or fold into `sts-engine` as `training_data` mod). All have backing tests; mechanical port + parallel Rust tests.

Files:
- `encounters.py` (317) — already a static `HashMap<&str, EncounterSpec>` shape; ~50 LOC of `lazy_static` Rust constants.
- `seed_imports.py` (294) — frozen dataclass payloads.
- `seed_suite.py` (165) — small enum + report struct.
- `corpus.py` (21) — single tuple constant.
- `value_targets.py` (106) — multi-head dataclass + vocab.
- `selector.py` (56) — lexicographic sort.
- `benchmark.py` (31) — single weighted-sum function.
- `_serde.py` (46) — stable JSON + sha256.
- `manifests.py` (173) — sha256 config hash + git-snapshot dataclass.
- `restrictions.py` (151) — restriction enum + rule struct.

Total: **~1,360 LOC out, ~600 LOC Rust in**.

Tests to preserve (port to Rust integration tests in the same PR):
- `tests/training/test_encounter_catalog_completeness.py` (4 cases)
- `tests/training/test_seed_suite.py`
- `tests/training/test_restrictions.py`
- `tests/training/test_selector.py`
- `tests/training/test_manifests.py`

Acceptance:
1. New Rust module `training_data` (or crate) with all 10 file equivalents.
2. Python files **kept temporarily** as shim re-exports from PyO3 (`from sts_engine import EncounterSpec as ...`) so downstream code (cli.py, stage2_pipeline.py) doesn't break.
3. Rust + Python test suites both green.
4. PR description lists each file's Rust equivalent.

Estimated effort: 1 focused PR, **~3 days agent time**.

## Phase 2 — Delete `contracts.py` (~616 LOC)

**Goal**: the dataclass mirror is *literally redundant* with the Rust contract types in `packages/engine-rs/src/contract.rs`. Replace with `serde_json::Value -> typed struct` round-trips on the Rust side. Removes the drift hazard between Rust ↔ Python types.

Mechanic:
1. Audit every `from packages.training.contracts import ...` site (mostly `stage2_pipeline.py`, `run_replay.py`, `bridge.py`).
2. For each consumer: replace with a PyO3 shim that converts the engine's already-typed result to a Python dict (or keep dataclass mirror temporarily but have it auto-derived from the Rust contract via `pyo3 = { features = ["serde"] }`).
3. Delete `contracts.py`.
4. Update `tests/training/test_contracts.py` — port assertions to a Rust integration test.

Tests to preserve:
- `tests/training/test_contracts.py` (~95 cases)

Acceptance:
1. `packages/training/contracts.py` deleted.
2. All consumers compile + tests green.
3. Rust contract types are now the single source of truth; `serde` round-trip verified.

Estimated effort: **~2 days agent time**.

## Phase 3 — `run_parser.py` + `entity_catalog.py` (~822 LOC out)

**Goal**: port together (`run_parser` calls `entity_catalog.canonicalize_*`). The largest non-MLX file in the stack and the deepest schema work; protected by `tests/training/test_run_parser_reconciliation.py` (5 cases) and the manual smoke runs.

Mechanic:
1. Port `entity_catalog.py` first — it's just lookup tables built from `engine_module.get_training_entity_catalog()`. Becomes a `pub fn canonicalize_card_id(...)` etc. in the engine crate.
2. Port `run_parser.py` — JSON parsing, Neow forward-sim heuristics, master_deck reconciliation. Care: the Pandora-output-then-purged transient injection logic (`failed_removes` tracking) is non-obvious; preserve the test cases for it.
3. Expose the parser as a Rust binary `cargo run --bin parse-run -- <path>` (replacement for `python -m packages.training.run_parser <path>`).
4. The `RecordedCombatCase` struct becomes a Rust type the replayer consumes directly.

Tests to preserve:
- `tests/training/test_run_parser_reconciliation.py` (5 cases — Adaptation upgrade, Pandora outputs, Strike/Defend post-Pandora removal, failed_removes tracking, F55 master_deck size match).

Acceptance:
1. `parse-run` Rust binary parses the WATCHER A0 golden seed and outputs the same per-combat decks (verify against Python output via diff).
2. Reconciliation produces identical results (Adaptation upgraded, WaveOfTheHand injected F18-F20, Strike/Defend stripped post-Pandora).
3. Python tests pass via shim OR are ported to Rust integration tests.

Estimated effort: **~3-4 days agent time** (this is the trickiest phase).

## Phase 4 — Orchestration to Rust `[[bin]]` target (~1,200 LOC out)

**Goal**: `cli.py` (835), `bridge.py` (74), `engine_module.py` (87) all become a Rust binary calling the `sts-engine` crate directly. PyO3 disappears for these paths. Python side keeps only the MLX kernel + a small wrapper that talks to the binary via JSON over stdio (or a UDS socket).

Mechanic:
1. New `[[bin]]` target in `packages/engine-rs/Cargo.toml`: `name = "training-cli"`.
2. Port `cli.py`'s argparse to `clap` with the same subcommand surface (`print-corpus-plan`, `collect-puct-targets`, `train-puct-checkpoint`, `validate-seed-suite`, `validate-recorded-run`, `run-phase1-puct-overnight`).
3. The `train-puct-checkpoint` subcommand is the only one that needs to call into MLX; it shells out to `python -m sts_training.train --checkpoint <path> ...` (a thin Python entry point that just runs the MLX trainer and writes the checkpoint).
4. Delete `cli.py`, `bridge.py`, `engine_module.py` from `packages/training/`.
5. Update `scripts/training.sh` to invoke the Rust binary instead of `python -m packages.training`.

Acceptance:
1. `./scripts/training.sh validate-recorded-run --run-file <path> --output-dir logs/active/test` runs end-to-end via the Rust binary.
2. `./scripts/training.sh run-phase1-puct-overnight ...` produces identical artifact shapes (manifest.json, events.jsonl, recorded_run_replay_report.json).
3. Continuous training script (`scripts/continuous_training.sh`) still works — possibly with one-line changes to invoke the binary instead of the Python module.
4. PyO3 cdylib still exists (used by the Python MLX trainer), but most non-training code paths skip it.

Estimated effort: **~3-4 days agent time**.

## What stays Python forever (until MLX gets a Rust binding)

Cat A files:
- `combat_model.py` (490) — MLX policy/value head. Could be split: ~200 LOC of dataclass scaffolding + numpy ops are portable; ~290 LOC of MLX `_forward` / softmax / training methods are not.
- `inference_service.py` (272) — numpy-heavy batch service. Could thin to ~100 LOC if Rust owned the batcher.
- `shared_memory.py` (385) — numpy batch tensors.

After P1-P4: these three plus a small wrapper script become the entire Python footprint. Estimated post-migration LOC: **~1,150 LOC** (the MLX/numpy kernel itself).

## Hidden costs

- **New Rust deps**: `clap` (CLI), `chrono` (timestamps), `sysinfo` (psutil replacement; macOS GPU sampler shells `powermetrics`, sudo concern), `sha2` (already transitive). All maintained, no risk.
- **New cargo target**: `[[bin]]` entry alongside the existing `cdylib`. Straightforward.
- **`engine_adapter.py` (258 LOC) lacks dedicated tests**: only exercised through `stage2_pipeline.py` and `run_replay.py`. **Write characterization tests in Rust before porting** — otherwise the candidate-feature math is non-falsifiable mid-migration.
- **`system_stats.py` (122 LOC) lacks tests**: same caution. The `powermetrics` GPU sampler in particular is platform-specific and needs a stub on non-macOS dev machines.
- **`episode_log.py` (48 LOC) lacks tests**: trivial JSONL writer, low risk.
- **Cross-language schema drift**: P2 explicitly fixes this by making Rust the source of truth. Avoid re-introducing typed-mirror Python during P1.

## Tests to preserve (port progressively to Rust integration tests)

| Test file | Cases | Phase |
|-----------|------:|------:|
| `test_encounter_catalog_completeness.py` | 4 | P1 |
| `test_seed_suite.py` | ~6 | P1 |
| `test_restrictions.py` | ~10 | P1 |
| `test_selector.py` | ~8 | P1 |
| `test_manifests.py` | ~6 | P1 |
| `test_contracts.py` | ~95 | P2 |
| `test_run_parser_reconciliation.py` | 5 | P3 |
| `test_run_logging.py` | ~6 | P1 |
| `test_benchmark.py` + `test_benchmarking.py` | ~12 | P1 |
| `test_stage2_pipeline.py` | ~10 | P3-P4 |
| `test_cli_phase1.py` | ~8 | P4 |
| `test_bridge.py` | ~6 | P4 |
| `test_policy_value_trainer.py` | ~10 | **stays Python** (MLX) |
| `test_puct_targets.py` | ~10 | P3 |
| `test_seed_imports.py` | ~6 | P1 |
| `test_training_imports.py` | smoke | P1 |
| `test_package_skeleton.py` | smoke | P4 (becomes Rust integration test) |

Total ~200 cases; ~190 portable, ~10 stay Python (MLX-touching).

## Phasing summary

| Phase | LOC out | LOC in (Rust) | Agent days | Cumulative reduction |
|------:|--------:|--------------:|-----------:|---------------------:|
| P1 | 1,360 | 600 | 3 | -19% |
| P2 | 616 | 0 (delete) | 2 | -28% |
| P3 | 822 | 400 | 3-4 | -40% |
| P4 | 1,200 | 800 | 3-4 | -57% |
| **Total** | **~4,000 out** | **~1,800 in** | **11-13 days** | **~57%** |

Anchored Python floor after P4: ~3,000 LOC (MLX kernel + minimal MLX-trainer entry script + the `~1,150 LOC` Cat-A core).

## How to start (for the next agent)

1. **Read this doc.** Read the audit transcript in the conversation log of branch `claude/archive-legacy-python-engine` for the methodology behind the categorization.
2. **Confirm with the user**: which phase to start, whether to fold the new Rust modules into `sts-engine` or create a new `training-data` crate, whether the `train-puct-checkpoint` Python entry point is OK as the MLX shim or if you want a different handoff shape.
3. **Branch off main** (after #135 + #136 merge). Don't continue this archive branch.
4. **One PR per phase**. Don't combine. Each PR ships with parallel Rust tests + the original Python tests still passing (via shim) until that file is fully removed.
5. **Update this doc as you go** — flip phase rows from "scoped" to "in-flight" to "shipped, see commit XXX".

## Cross-references

- **Engine parity work** (orthogonal, separate branch): [docs/work_units/parity-deviations-register.md](./parity-deviations-register.md) — D48/D52/D62/D67 etc. continue on a Rust-only branch.
- **Comprehensive audit baseline**: [docs/work_units/comprehensive-audit-2026-04-17.md](./comprehensive-audit-2026-04-17.md).
- **Architecture map**: [docs/architecture-overview.md](../architecture-overview.md) — describes the current Python+Rust+SwiftUI stack; will need an update after P4.
- **Plan file (this hand-off was approved here)**: `~/.claude/plans/so-i-was-working-purring-wreath.md`.
