# Remaining Parity Audit (Comprehensive)

Last updated: 2026-02-24
Baseline branch: `codex/parity-d0-d2-foundation`

## Verified baseline
- Command: `uv run pytest tests/ -q`
- Result: `4722 passed, 0 skipped, 0 failed`

## Script-generated inventory/mapping snapshot
Source artifact: `docs/archive/audits-2026-02-22-full-game-parity/traceability/parity-diff.json`

| Domain | Java | Python | Exact | Alias | Missing | Status summary |
|---|---:|---:|---:|---:|---:|---|
| cards | 361 | 370 | 228 | 133 | 0 | inventory mapping closure complete; behavior parity remains |
| relics | 181 | 181 | 75 | 106 | 0 | inventory parity closed |
| events | 51 | 51 | 40 | 11 | 0 | regular event inventory parity closed |
| powers | 149 | 148 | 125 | 24 | 0 | Java inventory mapping closed |
| potions | 42 | 42 | 28 | 14 | 0 | inventory parity closed with fallback class-source extraction |

## Power dispatch parity snapshot
Source artifact: `docs/archive/audits-2026-02-22-full-game-parity/traceability/power-hook-coverage.json`

- Registry hooks: `25`
- Runtime-dispatched hooks: `25`
- Registered-but-undispatched hooks: `0`

## Consolidation status
- Canonical repo lock documented in `REPO_CANONICAL.md`.
- Wrapper migration decisions documented in `traceability/repo-consolidation-manifest.md`.
- Curated deterministic training utilities migrated into `packages/training/` with tests.
- Action-space and workflow specs are now canonicalized in:
  - `action-layer/ACTION_SPACE_SPEC.md`
  - `process/SUBAGENT_EXECUTION_LOOP.md`
  - `traceability/UNIT_CHUNKS.md`
  - `rng/JAVA_RNG_STREAM_SPEC.md`

## Remaining closure order (locked)
1. `POW-002B` and `POW-003*`: enforce hook ordering and behavior parity tests.
2. `CRD-IC/SI/WA/SH/DE`: close remaining card behavior deltas (inventory already mapped).
3. `RNG-MOD-*`: remove direct `random.*` usage from parity-critical runtime paths.
4. `RL-ACT/OBS/DASH/SEARCH`: lock training contract and local runboard/deep-dive tooling.
5. `AUD-001`/`AUD-003`: final parity sign-off and RL launch gate.
