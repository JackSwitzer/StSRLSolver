# Ground Truth: Java Parity + Agent Contract

> Update 2026-03-16:
> Inventory closure and handler coverage are no longer enough to claim runtime
> parity or RL-readiness. See
> `docs/audits/2026-03-16-full-audit-gap-inventory.md` for the currently known
> blockers.

Last updated: 2026-02-24
Working branch: `codex/parity-d0-d2-foundation`

## Baseline
- Command: `uv run pytest tests/ -q`
- Result: `4722 passed, 0 skipped, 0 failed`
- No skips executed in the current baseline run.

## Canonical sources
- Java reference root:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl`
- Canonical Python repo:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL`
- Script-generated parity artifacts:
  - `docs/audits/2026-02-22-full-game-parity/traceability/java-inventory.json`
  - `docs/audits/2026-02-22-full-game-parity/traceability/python-inventory.json`
  - `docs/audits/2026-02-22-full-game-parity/traceability/parity-diff.json`
  - `docs/audits/2026-02-22-full-game-parity/traceability/power-hook-coverage.json`

## Inventory and mapping snapshot (generated)

| Domain | Java count | Python count | Exact | Alias | Missing | Notes |
|---|---:|---:|---:|---:|---:|---|
| cards | 361 | 370 | 228 | 133 | 0 | class-name and in-game-ID rows normalized via aliases |
| relics | 181 | 181 | 75 | 106 | 0 | inventory parity closed |
| events | 51 | 51 | 40 | 11 | 0 | regular event inventory closed (`SpireHeart` excluded from regular pools) |
| powers | 149 | 148 | 125 | 24 | 0 | full Java class mapping achieved |
| potions | 42 | 42 | 28 | 14 | 0 | fallback class-artifact inventory source enabled |

## Power hook dispatch snapshot (generated)
- Registry hooks: `25`
- Runtime-dispatched hooks: `25`
- Registered-but-undispatched hooks: `0`

## Agent contract snapshot

### Stable public API
- `GameRunner.get_available_action_dicts()`
- `GameRunner.take_action_dict()`
- `GameRunner.get_observation()`

### Observation/action schema markers
- `observation_schema_version` emitted at observation root.
- `action_schema_version` emitted at observation root.

### Canonical action-surface policy
- Environment API remains primitive-action only.
- Selection-required mechanics are explicit two-step flows via `select_cards` / `select_stance`.
- Invalid actions are hard-rejected and expected to be mask-pruned by caller.
- Full spec: `docs/audits/2026-02-22-full-game-parity/action-layer/ACTION_SPACE_SPEC.md`.

## Consolidation state
- Canonical repo lock file: `REPO_CANONICAL.md`.
- Training-wrapper migration manifest:
  - `docs/audits/2026-02-22-full-game-parity/traceability/repo-consolidation-manifest.md`
- Curated training utilities migrated into:
  - `packages/training/`
- Desktop one-folder realignment verified:
  - `docs/audits/2026-02-22-full-game-parity/traceability/desktop-realignment-2026-02-23.md`
- Combat runtime unification completed for duplicated implementation removal:
  - `handlers/combat.py` is compatibility shim + helper surface only
  - `CombatEngine` is runtime owner

## Priority remaining blockers
1. Card behavior parity closure (inventory now closed; behavior deltas remain).
2. Power behavior/order parity closure beyond dispatch inventory closure.
3. RNG normalization migration in parity-critical runtime modules.
4. Runtime-loop parity closure:
   - event combat handoff
   - question-room resolution
   - burning-elite wiring
   - campfire action-surface fidelity
   - replay beyond floor 0
5. RL readiness gates (`RL-ACT-*`, `RL-OBS-*`, dashboard/search layers) and final audit sign-off.
6. Artifact-schema unification for run/episode/replay persistence.
