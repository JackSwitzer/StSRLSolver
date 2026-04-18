# Archive — Legacy Python combat engine

Contents archived 2026-04-18 on branch `claude/archive-legacy-python-engine`. The active gameplay engine is now [packages/engine-rs/](../packages/engine-rs/) (Rust, PyO3 bindings); the active training stack is [packages/training/](../packages/training/) which calls the Rust engine through `engine_module.py`.

## What's here

- `packages-engine/` — the legacy Python combat engine (`combat_engine.py`, 123 KB; `game.py`, 182 KB; `state/`, `calc/`, `content/`, `effects/`, `generation/`, `handlers/`, `agent_api.py`, `rl_observations.py`, `rl_masks.py`, `registry/`). Pre-Rust era; superseded by `packages/engine-rs/` which has reached audited Java parity on the supported scope (`docs/research/engine-rs-audits/AUDIT_PARITY_STATUS.md`).
- `packages-parity/` — the legacy Java↔Python parity comparison tools (`replay_runner.py`, `comparison/{game_simulator,map_explorer,seed_verifier,live_monitor,full_rng_tracker,state_comparator,interactive_verifier,live_tracker}.py`). All depended on `packages/engine/`; ported only as far as needed to bootstrap the Rust engine. Rust now self-tests via `packages/engine-rs/src/tests/test_*_parity.rs` against the Java decomp.
- `tests-legacy/` — 91 top-level test files (`test_*.py`) plus the legacy `conftest.py` and `__init__.py`. All imported from `packages.engine.*`. The active test suites are [tests/training/](../tests/training/) (training stack) and [packages/engine-rs/src/tests/](../packages/engine-rs/src/tests/) (Rust engine, ~2207 tests).

## Why archive instead of delete

- Historical reference: the Python engine encoded a lot of hard-won understanding of edge cases (status timing, RNG stream allocation, multi-hit damage application, stance interactions). When the Rust engine surfaces a parity gap, the Python implementation is often the fastest second source of truth before going to the Java decomp.
- Audit trail: the `tests-legacy/` suites were the original parity bar. Keeping them alongside their target makes it easy to recreate the exact assertions in Rust.
- One-cycle deletion rule: per [docs/work_units/parity-deviations-register.md](../docs/work_units/parity-deviations-register.md), closed rows stay in the doc for one release cycle so we can see history. Same principle applies to entire packages — archive first, delete later if truly unused.

## Reactivating

If you ever need to run the legacy suites:

1. Move the directory back: `git mv archive/packages-engine packages/engine` (and `packages-parity` and `tests-legacy/* tests/`).
2. Restore `pyproject.toml` `[tool.pytest.ini_options].testpaths` to `["tests"]`.
3. `uv run pytest tests/` (will pick up everything).

Nothing else has to change — the legacy code is self-contained.

## What's NOT archived

- `packages/engine-rs/` — the active Rust engine.
- `packages/training/` — the active training stack (Python wrappers + MLX + PUCT).
- `packages/app/SpireMonitor/` — the active SwiftUI monitor.
- `tests/training/` — the active Python tests (training stack only).
- `decompiled/java-src/` — the Java decomp, which is the ground-truth parity reference. (To be removed after the parallel parity ultra-review surfaces and registers all remaining divergences.)
