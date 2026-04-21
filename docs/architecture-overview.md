# Architecture Overview

This document is a single-pass tour of the entire repo. It covers the Rust engine, the
Python training stack, the SwiftUI monitor, the orchestration scripts, the parity
infrastructure, and which legacy Python remains as carry-over from the previous
training generation.

The branch shape (recorded in `CLAUDE.md:6-12`):

- engine base: `codex/universal-gameplay-runtime`
- training stacked on top: `codex/training-rebuild`
- this worktree (`sharp-solomon-a1c9ec`) is checked out on the training branch.


## 1. Repo Layout

Top-level (worktree root):

- `packages/` — all production code (Rust engine, Python training, SwiftUI monitor, parity)
- `scripts/` — bash orchestration (`training.sh`, `test_engine_rs.sh`, `app.sh`, `alert.sh`, `play.sh`, `hooks/`)
- `tests/` — legacy pytest suite for `packages/engine/` (Python engine), plus `tests/training/` for `packages/training/`
- `docs/` — `CLAUDE-training.md`, `CLAUDE-data.md`, `work_units/`, `research/`, `vault/`, this file
- `logs/active/` — artifact root convention consumed by SpireMonitor and CLI emitters
- `pyproject.toml` (`pyproject.toml:5-22`) — `setuptools.packages.find` over `packages*`; deps include `mlx`, `torch`, `gymnasium`, `numpy`, `fastapi`
- `CLAUDE.md` — branch shape, active system, and current phase notes

Inside `packages/`:

- `packages/engine-rs/` — canonical Rust engine + PyO3 bindings (compiled to `libsts_engine.dylib`)
- `packages/engine/` — legacy Python engine (still used by `tests/` and `packages/parity/`; not imported from `packages/training/`)
- `packages/training/` — combat-first training stack (corpus, PUCT collection, MLX trainer, CLI, recorded-run replay)
- `packages/parity/` — Java-vs-Python parity verification tools (depends on `packages/engine/`)
- `packages/app/SpireMonitor/` — native macOS SwiftUI monitor (Swift Package, see `packages/app/Package.swift:1-13`)

Decompiled Slay the Spire Java source lives **outside** the worktree at the parent
repo root:

- `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/` — 2,008 `.java` files (CFR-decompiled `desktop-1.0.jar`, see `decompiled/manifest.json`)


## 2. Rust Engine — `packages/engine-rs/src/`

Crate root: `packages/engine-rs/src/lib.rs:1-40` declares 26 modules. PyO3 module
entry is `lib.rs:52-63` which exports `RustCombatEngine`, `PyCombatState`, `PyAction`,
`PyRunEngine`, `StSEngine`, `ActionInfo`, `CombatSolver`, plus a free function
`get_training_entity_catalog`.

### Module map

- `cards/` (`packages/engine-rs/src/cards/mod.rs`, ~99 KB) — per-character subdirs
  `ironclad/`, `silent/`, `defect/`, `watcher/`, plus `colorless/`, `curses.rs`,
  `status.rs`, `temp.rs`, `runtime_meta.rs`. Global card registry exposed via
  `cards::global_registry()` (`lib.rs:70`).
- `relics/` (`packages/engine-rs/src/relics/mod.rs`) — 105 individual `defs/*.rs`
  files (one per relic, e.g. `akabeko.rs`, `burning_blood.rs`, `pure_water.rs`).
- `powers/` (`packages/engine-rs/src/powers/`) — `buffs.rs` (37 KB), `debuffs.rs`,
  `enemy_powers.rs`, `defs/`, `registry.rs`.
- `events/` (`packages/engine-rs/src/events/`) — `exordium.rs`, `city.rs`, `beyond.rs`,
  `shrines.rs`, `mod.rs` (per-act event handlers).
- `gameplay/` (`packages/engine-rs/src/gameplay/`) — universal gameplay runtime
  (`runtime.rs:1-60`, `registry.rs`, `session.rs`, `types.rs`). Provides cross-domain
  effect interpretation that combat/run/decision systems all read through.
- `effects/` (`packages/engine-rs/src/effects/`) — declarative effect interpreter
  (`interpreter.rs` 75 KB, `runtime.rs` 75 KB, `hooks_*.rs` per trigger family).
- `enemies/`, `potions/`, `orbs.rs`, `damage.rs`, `decision.rs`, `map.rs`,
  `seed.rs`, `state.rs`, `status_ids.rs` (status-effect ID table).
- `engine.rs` (4,589 lines) — `CombatEngine` core
- `run.rs` (4,354 lines) — `RunEngine` (full Act 1+ run simulation)
- `obs.rs` — 480-dim observation encoder (matches Python `state_encoders.py`)
- `search.rs` — PUCT and brute-force search
- `training_contract.rs` — versioned training schemas
- `lib.rs` — PyO3 bindings

### Combat loop entry points

- `CombatEngine::start_combat(&mut self)` — `engine.rs:228-280`. Emits
  `Trigger::CombatStart` event, channels start-of-combat orbs, shuffles draw,
  reorders innate cards to top, then calls `start_player_turn()`.
- `CombatEngine::execute_action(&mut self, action: &Action)` — `engine.rs:365-383`.
  Rebuilds the owner-aware effect dispatch table, then matches on
  `Action::{EndTurn, PlayCard, UsePotion, Choose, ConfirmSelection}`.
- `CombatEngine::get_legal_actions()` — `engine.rs:288-362`. Returns choice actions
  if `phase == AwaitingChoice`, otherwise enumerates legal card plays (per target),
  legal potion uses (per target), and always-legal `EndTurn`.
- `CombatEngine::is_combat_over()` — `engine.rs:283-286` (reads `state.combat_over`).
- Choice flow: `begin_choice` / `begin_choice_with_named_payloads` /
  `begin_discovery_choice` / `begin_choice_with_aux` / `begin_choice_with_action`
  (`engine.rs:410-466`).

Action dispatch is centralised in `execute_action`; per-card behaviour lives in
the declarative effects interpreter (`effects/interpreter.rs`) plus per-card helpers
in `engine.rs` (e.g. `gain_block_player`, `deal_damage_to_enemy`, `change_stance`,
`channel_orb`, `do_scry`, `add_temp_cards_to_hand`, `evoke_front_orb`).

### Training contract — `packages/engine-rs/src/training_contract.rs`

Schema version constants (`training_contract.rs:21-28`):

- `TRAINING_SESSION_SCHEMA_VERSION = 1`
- `COMBAT_OBSERVATION_SCHEMA_VERSION = 1`
- `ACTION_CANDIDATE_SCHEMA_VERSION = 1`
- `GAMEPLAY_EXPORT_SCHEMA_VERSION = 1`
- `REPLAY_EVENT_TRACE_SCHEMA_VERSION = 1`
- `COMBAT_SNAPSHOT_SCHEMA_VERSION = 1`
- `COMBAT_FRONTIER_CAPACITY = 8`
- `COMBAT_PUCT_STABLE_WINDOWS = 3`

Token caps (`training_contract.rs:30-36`): hand 10, enemies 5, player effects 32,
enemy effects 16, orbs 10, relic counters 8, choice options 10.

Key versioned structs:

- `CombatTrainingStateV1` — `training_contract.rs:486` (the canonical training state)
- `CombatObservationSchemaV1` — `training_contract.rs:214` (token-first observation)
- `LegalActionCandidateV1` — `training_contract.rs:268`
- `CombatPuctConfigV1` — `training_contract.rs:338` (with `min_visits`, `hard_visit_cap`,
  `time_cap_ms`, `visit_window`, `best_visit_share_lead_threshold`,
  `root_value_delta_threshold`, `stable_windows_required`, `cpuct`)
- `CombatPuctResultV1` — `training_contract.rs:438`
- `CombatPuctLineV1` — `training_contract.rs:426` (frontier line)
- `CombatSearchStopReasonV1` — `training_contract.rs:329`
  (`Converged | TimeCap | HardVisitCap | TerminalRoot | NoLegalActions`)
- `CombatSnapshotV1` — `training_contract.rs:524` (serialized engine state for solver
  re-instantiation)
- `RunManifestV1` — `training_contract.rs:558`
- `EpisodeStepV1`, `EpisodeLogV1` — `training_contract.rs:574-588`
- `BenchmarkSliceResultV1`, `BenchmarkReportV1` — `training_contract.rs:590-606`

Bridge functions: `combat_training_state_from_combat` (`:607`),
`combat_snapshot_from_combat` (`:658`), `combat_engine_from_snapshot` (`:735`),
`build_legal_action_candidates` (`:1103`), `restricted_legal_decision_actions`
(`:805`).

### Search — `packages/engine-rs/src/search.rs`

- `search_combat_puct<E, F, G>(snapshot, config, execution_id_for_action, evaluator)`
  — `search.rs:280-400`. Generic over a leaf evaluator closure
  `F: FnMut(&CombatTrainingStateV1) -> Result<CombatPuctLeafEvaluationV1, E>`. The
  evaluator returns `(value, prior)` for each candidate; in the Python stack this
  is wired to MLX inference (`packages/training/engine_adapter.py` provides
  `build_model_evaluator`).
- Stop conditions (`search.rs:328-399`):
  1. `TimeCap` — elapsed >= `config.time_cap_ms`
  2. `HardVisitCap` — root visits >= `config.hard_visit_cap`
  3. `Converged` — sustained `stable_windows_required` consecutive convergence windows
     (best-action stability + value delta below threshold)
  4. `TerminalRoot` / `NoLegalActions` — degenerate cases handled at the top
- `simulate_puct` (`search.rs:402`), `expand_puct_node` (`:460`), `select_puct_child`
  (`:510`), `puct_score` (`:533`), `backpropagate` (`:546`), `convergence_snapshot`
  (`:576`), `build_puct_result` (`:621`).
- Brute-force planners: `search_combat` (`:242`) using `CombatPlanner` (`:781`),
  `search_run` (`:250`) using `RunPlanner` (`:876`), and `run_seed_suite` (`:258`)
  for batch evaluation.

### Python bindings — `packages/engine-rs/src/lib.rs`

Compiled as a PyO3 extension named `sts_engine`. The Python loader
(`packages/training/engine_module.py:14-87`) auto-builds the dylib via
`cargo build --features extension-module` when missing, then loads it with
`importlib.machinery.ExtensionFileLoader`.

Key Python-exposed classes:

- `RustCombatEngine` (`lib.rs:54`, defined in `engine.rs`) — combat-only engine.
  Constructor takes `(player_hp, max_hp, energy, deck, enemies, seed, relics)`,
  exposes `start_combat`, `get_combat_snapshot_json`, etc. Used by recorded-run
  replay (`run_replay.py:162-171`).
- `CombatSolver` (`lib.rs:158-307`) — cloned combat state for MCTS. Static
  `from_snapshot_json(json)` rehydrates a combat state from `CombatSnapshotV1`.
  Methods: `step(action_id) -> (reward, done)`, `get_legal_actions() -> [int]`,
  `get_legal_action_infos() -> [ActionInfo]`, `is_done`, `is_won`, `get_hp`,
  `get_energy`, `get_turn`, `run_combat_puct(evaluator, config_json)`.
- `StSEngine` (`lib.rs:716-`) — Gym-style API wrapping `RunEngine`. Constructor
  `(seed, ascension=20, character="watcher")`. `step(action) -> (state_dict, reward,
  done, info)`, `reset(seed)`, `get_obs() -> Vec<f32>`, `get_legal_actions()`,
  `clone_combat() -> Optional[CombatSolver]` (`lib.rs:802-809`) for in-run MCTS.
- `PyRunEngine`, `PyCombatState`, `PyAction`, `ActionInfo` — supporting types.
- `get_training_entity_catalog(py) -> dict` (`lib.rs:65-107`) — exposes the cards,
  relics, potions, and enemy registries to Python.


## 3. Python Training Stack — `packages/training/`

Combat-first training built on the Rust engine + PyO3 binding. Public surface in
`packages/training/__init__.py:1-104` (45+ symbols re-exported).

### Module map

- `cli.py` (`packages/training/cli.py`, 31 KB) — argparse subcommands; entry
  point is `python -m packages.training`.
- `corpus.py` — Watcher starter deck constants.
- `encounters.py` — `ENCOUNTER_CATALOG`; `encounter_spec(name)` returns a spec
  with `room_kind`, `to_engine_enemies()`.
- `entity_catalog.py` — canonical card / relic / potion id maps and
  `canonicalize_*` helpers (used by `run_parser.py:32-36`).
- `seed_imports.py` — reconstructed Act 1 Watcher seed scripts;
  `default_imported_act1_scripts()` returns the validation seed list.
- `seed_suite.py` — `default_watcher_validation_seed_suite()`,
  `ValidationSeed`.
- `stage2_pipeline.py` (`packages/training/stage2_pipeline.py`, 31 KB) —
  snapshot-corpus generation (`SnapshotCase`, `_config_for_room`,
  `write_snapshot_corpus`, `load_snapshot_corpus`), Rust PUCT collection
  (`PuctCollectionRecord`, `write_puct_collection`), seed-validation report
  builder, frontier-points extractor.
- `inference_service.py` — `CombatInferenceService`, `CombatPolicyValueTrainer`,
  `CombatSearchConfig`, `PolicyValueEpochSummary`, `TrainingConfig`.
- `combat_model.py` (20 KB) — `MLXCombatModel` (MLX-only inference + training),
  `CombatBatchPredictions`, `LegalCombatCandidate`.
- `engine_module.py` (`packages/training/engine_module.py:1-87`) — builds and
  loads the `sts_engine` extension (`build_engine_extension`, `load_engine_module`).
- `engine_adapter.py` — bridges PyO3 candidates to MLX evaluator
  (`build_model_evaluator`, `action_id_for_candidate`,
  `build_search_request_from_training_state`).
- `bridge.py` — `load_combat_training_state`, `run_combat_puct`,
  `parse_combat_puct_result`, `parse_combat_training_state`.
- `contracts.py` (17 KB) — Python mirrors of the Rust `*V1` schemas.
- `manifests.py` — `TrainingRunManifest`, `GitSnapshot`, `OvernightSearchSnapshot`,
  `SearchBudgetSnapshot`, `TrainingConfigSnapshot`, `build_run_manifest`.
- `run_logging.py` — `TrainingArtifacts(root)`, `TrainingRunLogger.append_event`
  (writes JSONL into `events.jsonl`/`metrics.jsonl`).
- `benchmarking.py` / `benchmark.py` — `BenchmarkConfig`, `frontier_score`,
  `BenchmarkFrontierPoint`, `FrontierReport`, `pareto_frontier`,
  `build_frontier_report`.
- `selector.py` — `select_frontier`, `rank_frontier_lines`.
- `value_targets.py` — `CombatValueTarget`, `PHASE1_POTION_VOCAB`,
  `PHASE1_VALUE_HEAD_NAMES`.
- `shared_memory.py` — `CombatPuctTargetExample`, `CombatSearchRequest`,
  `CombatSharedMemoryBatch`, `CombatSharedMemoryBatcher`, `SharedMemoryConfig`.
- `restrictions.py` — restriction-policy enforcement.
- `system_stats.py` — `SystemStatsSampler` (CPU/RAM/process samples logged into
  artifacts).
- `episode_log.py` — episode-step logger.
- `_serde.py` — JSON helpers.
- `run_parser.py` (added 2026-04-17, 20 KB) — see §3 Recorded-run replay.
- `run_replay.py` (added 2026-04-17, 10 KB) — see §3 Recorded-run replay.

### Data flow

```
encounter spec + relics + deck   ->  SnapshotCase           (stage2_pipeline.py)
SnapshotCase                      ->  CombatSnapshotV1 JSON  (Rust contract)
CombatSnapshotV1 JSON             ->  CombatSolver.from_snapshot_json
                                                            (lib.rs:166-177)
CombatSolver + MLX evaluator     ->  CombatPuctResultV1     (search.rs:280)
CombatPuctResultV1                ->  PuctCollectionRecord   (stage2_pipeline.py)
PuctCollectionRecord+             ->  CombatPuctTargetExample(records_to_puct_targets)
CombatPuctTargetExample           ->  MLXCombatModel.train  (combat_model.py)
                                  ->  MLX checkpoint (.safetensors / .npz)
checkpoint                        ->  next iteration evaluator
```

### CLI surface — `packages/training/cli.py`

`subparsers.add_parser` calls (`cli.py:59-119`):

- `print-corpus-plan` — describes the `watcher_a0_act1_snapshot` plan
- `print-seed-suite` — prints the reconstructed Act 1 validation seeds
- `generate-phase1-corpus --output-dir --target-cases`
- `collect-puct-targets --input --output-dir --collection-passes`
- `train-puct-checkpoint --input-dir --output-dir --epochs --learning-rate --top-k --checkpoint --no-update`
- `validate-seed-suite --output-dir --checkpoint`
- `validate-recorded-run --run-file --output-dir --tolerance --checkpoint --alert-script`
- `run-phase1-puct-overnight --output-dir --target-cases --collection-passes --epochs --learning-rate --top-k --seed`

### Recorded-run replay (added 2026-04-17)

- `run_parser.py` — parses Steam `.run` JSON files (one per completed run). Builds
  `RecordedFloor` (per-floor join of `card_choices`, `relics_obtained`,
  `event_choices`, `campfire_choices`, `potions_obtained`, `items_purchased`,
  `items_purged`, `boss_relics`) and `RecordedCombatCase` with forward-simulated
  entry deck/relics/potions/HP. Best-effort reconstruction: ambiguous cases (Neow
  removes, shop-item type, mid-combat potion timing) emit warnings into
  `RecordedRun.reconstruction_warnings` and the final deck is sanity-checked
  against `master_deck`.
- `run_replay.py:94-258` — `replay_recorded_run(run, output_dir, tolerance_base,
  checkpoint_path)`. For each combat case: looks up `encounter_spec` (skip as
  `unsupported` if missing), constructs `RustCombatEngine`, calls `start_combat`,
  serialises to snapshot JSON, hands to `CombatSolver`, runs `run_combat_puct`
  with the MLX evaluator, computes pass/fail by
  `solver_hp_loss <= recorded_hp_loss + max(base_tol, 0.10 * max_hp)`
  (`run_replay.py:86-87`).
- Emits live events to `events.jsonl`: `run_started`, `combat_started`,
  `combat_solved`, `combat_failed`, `combat_unsupported`, `combat_error`,
  `run_complete`. Final report goes to `recorded_run_replay_report.json`.
- CLI subcommand entry: `cli.py:96-107` →
  `cli.py:_validate_recorded_run` (`cli.py:205-259`), which optionally invokes
  `scripts/alert.sh` on `combat_failed` / `run_complete`.


## 4. SpireMonitor — `packages/app/SpireMonitor/`

Native macOS SwiftUI app, Swift Package (`packages/app/Package.swift:1-13`,
`platforms: [.macOS(.v14)]`).

Layout:

- `App/` — `SpireMonitorApp.swift`, `AppState.swift`
- `Settings/` — `AppConfig.swift` (configurable `logsPath`), `SettingsView.swift`
- `DataLayer/`
  - `ArtifactLoaders.swift` — pure async loaders for each artifact type
  - `DataStore.swift` — `@MainActor` observable store
  - `StatusPoller.swift` — actor that polls every 2.5 s
    (`StatusPoller.swift:26-31`) and pushes a `LoadedArtifactBundle` into the
    store
  - `SystemMonitor.swift`
- `Models/TrainingArtifacts.swift` — Decodable mirrors of the Rust/Python
  artifacts
- `Views/Artifacts/` — `ActiveRunSummaryView`, `ArtifactStreamsView`,
  `BenchmarkSliceDashboardView`, `FrontierInspectorView`,
  `SeedValidationReportView`, `ArtifactAnalysisView`
- `Views/Live/` — `LiveView`, `SystemStatsBar`
- `Theme/`, `Utilities/`

Artifacts consumed (resolved relative to `config.logsPath`, e.g. `logs/active/`):

- `manifest.json` — `ManifestLoader.load` (`ArtifactLoaders.swift:15-22`)
- `frontier_report.json` — `FrontierReportLoader`
- `seed_validation_report*.json` — `SeedValidationReportLoader.loadAll`
- `checkpoint_comparison*.json` — `SeedCheckpointComparisonLoader`
- `benchmark_report*.json` — `BenchmarkReportLoader`
- `episodes*.jsonl` — `ArtifactEpisodeLogLoader.loadAll` (`ArtifactLoaders.swift:107-141`)
- `events.jsonl` — `EventStreamLoader.load` (`ArtifactLoaders.swift:143-164`),
  decoded as `TrainingEventRecord` with extras as `JSONValue`
- `metrics.jsonl` — `MetricStreamLoader.load`

Polling: `StatusPoller.pollLoop` reloads everything from disk every 2.5 s
(simple but adequate for the artifact volume).

Build/run via `scripts/app.sh` (`packages/app/.build/.../SpireMonitor`):

```
./scripts/app.sh           # default: stop -> swift build -> launch
./scripts/app.sh --build   # release build only
./scripts/app.sh --stop    # kill running instance (PID file at .run/spire-monitor.pid)
./scripts/app.sh --status
```


## 5. Glue and Orchestration — `scripts/`

- `scripts/training.sh` (`scripts/training.sh:1-42`) — runs
  `uv run python -m packages.training "$@"`. The `launch` subcommand
  (`scripts/training.sh:8-39`) detaches via `nohup caffeinate -dimsu` (keeps
  the Mac awake), writes log + PID file (defaults
  `logs/active/training-launcher.log` and `logs/active/training-launcher.pid`).
- `scripts/test_engine_rs.sh` (`scripts/test_engine_rs.sh:1-37`) — wraps
  `cargo {test|check|build} --manifest-path packages/engine-rs/Cargo.toml`.
  Sets `PYO3_PYTHON=.venv/bin/python3` and `DYLD_FRAMEWORK_PATH` so PyO3-linked
  test binaries resolve.
- `scripts/alert.sh` (`scripts/alert.sh:1-38`) — sends iMessage to
  `+14166293183` via AppleScript and appends to `logs/active/alerts.log`.
  Severity validated against `info|warn|critical`. Used by
  `validate-recorded-run --alert-script` (`cli.py:241-259`).
- `scripts/app.sh` — see §4.
- `scripts/play.sh` — interactive play harness (less central).
- `scripts/hooks/` — Claude session lifecycle hooks
  (`session-create.sh`, `session-end.sh`, `session-beacon-update.sh`,
  `log-bash.sh`).

`logs/active/` convention: every CLI run writes its artifacts (manifest, events,
metrics, reports, episodes) into a single output dir, and SpireMonitor points
at that dir via `AppConfig.logsPath`. The current worktree has
`logs/active/recorded-run-20260417-212735/` and `logs/active/smoke/` from
recent test runs.


## 6. Parity Infrastructure

### Java decompile

- Location: `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/`
  (parent repo root, not in worktree)
- Source: CFR 0.152 against Steam `desktop-1.0.jar`
  (`decompiled/manifest.json` lists SHA256, java binary, filtered class count
  2,306, output count 2,008 `.java` files)

### Parity test suites

- Rust parity tests in `packages/engine-rs/src/tests/`:
  - `test_run_parity.rs`
  - `test_events_parity.rs`
  - Plus 200+ wave-grouped runtime tests
    (`test_card_runtime_*_wave*.rs`, `test_event_runtime_wave*.rs`,
    `test_card_play_timing_java_wave1.rs`, `test_card_legality_wave1.rs`)
- Python parity tests in `tests/`:
  - `tests/test_parity.py`
  - `tests/test_engine_parity_tail.py`
  - `tests/test_rng_parity.py`
  - `tests/test_enemy_ai_parity.py`
- `packages/parity/` (`packages/parity/__init__.py:1`) — verification tools
  built on the legacy `packages/engine` Python engine:
  - `replay_runner.py` — uses `from packages.engine.game import GameRunner`
  - `seed_catalog.py`
  - `comparison/` — `game_simulator.py`, `state_comparator.py`,
    `live_monitor.py`, `live_tracker.py`, `interactive_verifier.py`,
    `map_explorer.py`, `save_reader.py`, `seed_verifier.py`,
    `full_rng_tracker.py`
  - `verified_runs/` — captured ground-truth runs

### Recently archived

- `docs/research/engine-rs-audits/` (post PR #134) contains the merged audit
  artifacts:
  - `AUDIT_PARITY_STATUS.md`
  - `COMPLEX_HOOK_AUDIT.md`
  - `DECOMPILE_PARITY_ENDGAME.md`
  - `INCONSISTENCY_REPORT.md`


## 7. Legacy Python (carry-over candidates)

The active training stack (`packages/training/`) does not import
`packages.engine` at all (verified via grep — zero hits). The only consumers of
`packages/engine/` left in the worktree are:

- `packages/parity/` (4 files) — `parity/replay_runner.py`,
  `parity/comparison/game_simulator.py`, `parity/comparison/map_explorer.py`,
  and friends. All do `from packages.engine.game import GameRunner` /
  `from packages.engine.state.rng import Random` etc.
- `tests/` (90+ files at top level) — every one of `test_powers.py`,
  `test_relics.py`, `test_combat.py`, `test_potions.py`, `test_cards.py`, etc.
  imports `from engine.* / from packages.engine.*`. These are the legacy
  pytest suite for the Python engine.

### Candidates to archive

Last-modified dates (from `git checkout` time on this worktree, all
`2026-04-17`; the underlying code is older):

- `packages/engine/` (full Python engine implementation)
  - `__init__.py` (`packages/engine/__init__.py:1-140`) advertises itself as
    "A faithful Python recreation of the game's mechanics based on decompiled
    Java source"
  - `combat_engine.py` (123 KB), `game.py` (182 KB) — the original Python
    `CombatEngine` and `GameRunner`
  - Subdirs: `calc/`, `content/`, `effects/`, `generation/`, `handlers/`,
    `registry/`, `state/`, `utils/`
  - `agent_api.py`, `api.py`, `rl_masks.py`, `rl_observations.py` — old RL
    surface
  - Status: not imported from `packages/training/`; only used by `packages/parity/`
    and the legacy `tests/` suite. Archiving requires migrating parity tooling
    onto the Rust engine first (or accepting a parity-tooling freeze).
- `tests/` top-level (excluding `tests/training/` which is the active suite)
  - 90+ Python test files all targeting `packages/engine`
  - `tests/conftest.py` plus `tests/test_*.py` for combat, cards, relics,
    potions, powers, RNG, events, ascension, enemies, etc.
  - These are still wired into pytest defaults (`pyproject.toml:59-61`,
    `testpaths = ["tests"]`) and would all break if `packages/engine/` is
    removed.

There is no `src/` directory at the top of this worktree (the parent
non-worktree checkout has one, but it is not part of the training-rebuild
branch). No top-level `scripts/training/` legacy directory either — the only
`scripts/` content is the orchestration set listed in §5.


## 8. Data Flow End-to-End

```
                                                   +------------------+
   .run file (Steam save)                          |  scripts/        |
   /Library/Application Support/.../runs/*.run     |   alert.sh       |
            |                                      +---------+--------+
            v                                                |
   packages/training/run_parser.py                            |  iMessage
   parse_run_file()                                           |  +14166293183
            |                                                 ^
            v                                                 |
   RecordedRun  ->  RecordedCombatCase[]                      |
            |                                                 |
            v                                                 |
   packages/training/run_replay.py                             |
   replay_recorded_run(run, output_dir, tolerance, ckpt)       |
            |                                                 |
            v                                                 |
   for each RecordedCombatCase:                               |
     RustCombatEngine(hp,deck,enemies,relics)                  |
     engine.start_combat()                                    |
     engine.get_combat_snapshot_json()                         |
            |                                                 |
            v                                                 |
     CombatSnapshotV1 JSON                                    |
            |                                                 |
            v                                                 |
     CombatSolver.from_snapshot_json (PyO3, lib.rs:166-177)    |
            |                                                 |
            v                                                 |
     solver.run_combat_puct(evaluator, config_json)            |
            |   +- evaluator: build_model_evaluator(MLXCombatModel)
            v   +- search: search.rs:280-400 (PUCT)
     CombatPuctResultV1 JSON                                  |
            |                                                 |
            v                                                 |
     parse_combat_puct_result -> solver_hp_loss               |
     status = solved | failed | unsupported | error            |
            |                                                 |
            +-> events.jsonl (combat_started, combat_solved,  |
            |    combat_failed, combat_unsupported,           |
            |    combat_error, run_complete)                  |
            +-> recorded_run_replay_report.json               |
            +-> alert.sh on failure/completion ----------------+
            |
            v
   logs/active/<run-name>/  <----- polled every 2.5 s ----->  SpireMonitor
                                                              (StatusPoller -> 
                                                               MonitorArtifactLoader -> 
                                                               DataStore -> SwiftUI views)
```

The same artifact root convention applies to `generate-phase1-corpus`,
`collect-puct-targets`, `train-puct-checkpoint`, `validate-seed-suite`, and
`run-phase1-puct-overnight`: each writes
`{snapshot_corpus.jsonl, puct_collection.jsonl, manifest.json,
benchmark_report.json, frontier_report.json, episodes*.jsonl, events.jsonl,
metrics.jsonl}` into its `--output-dir`, which SpireMonitor reads through the
loaders in `packages/app/SpireMonitor/DataLayer/ArtifactLoaders.swift`.
