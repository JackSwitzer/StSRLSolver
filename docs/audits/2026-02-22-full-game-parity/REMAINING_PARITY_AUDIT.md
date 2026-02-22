# Remaining Parity Audit (Comprehensive)

Last updated: 2026-02-22  
Baseline branch: `main`  
Baseline merge commit: `4fff19ba0dc5ae219adbe0185d7af8a9425ef1c3` (PR #24)

## Solid core base
- Merged PR chain in `main`: [#14](https://github.com/JackSwitzer/StSRLSolver/pull/14) through [#24](https://github.com/JackSwitzer/StSRLSolver/pull/24)
- Stale historical PR [#8](https://github.com/JackSwitzer/StSRLSolver/pull/8) is closed and treated as archival only
- Full suite baseline is stable:
  - `uv run pytest tests/ -q`
  - `4663 passed, 5 skipped, 0 failed`

## Inventory parity snapshot (Java vs Python)

| Domain | Java inventory | Python inventory | Coverage | Notes |
|---|---:|---:|---:|---|
| Relics | 181 | 181 | 100.0% | Inventory parity complete; remaining work is behavior-level (`ORB-001` interactions) |
| Events | 51 | 51 | 100.0% | Definitions, handlers, and choice generators are all 51/51/51 |
| Powers | 149 | 94 | 63.1% | Largest remaining inventory gap |
| Cards (core set) | 361 | 358 overlap | 99.2% overlap | Core Java IDs exclude `optionCards` and `tempCards`; 3 Java IDs still unmatched |
| Potions | unavailable in local Java snapshot | 42 | n/a | Local decompile lacks a reliable potion class inventory root |

### Card ID variance details
- Java-only IDs in current decompile snapshot: `Discipline`, `Gash`, `Impulse`
- Python-only IDs: `Beta`, `Claw`, `Expunger`, `Insight`, `Miracle`, `Omega`, `Safety`, `Shiv`, `Smite`, `ThroughViolence`
- Interpretation:
  - Python-only IDs are mostly generated/temp cards used at runtime
  - `Gash` corresponds to the Java `Claw.java` decompile ID and should be normalized in inventory manifests
  - `Discipline` and `Impulse` require explicit policy labeling (`legacy-decompile`, `intentional-missing`, or implementation target)

## Action/API contract status
- Stable public runner API remains:
  - `GameRunner.get_available_action_dicts()`
  - `GameRunner.take_action_dict()`
  - `GameRunner.get_observation()`
- Explicit decision surfaces already in core for high-risk systems:
  - event follow-up card selections
  - reward/shop relic selections (Orrery, bottled relics, Dolly's Mirror)
  - indexed secondary relic claims (Black Star)
- Remaining explicit-decision closure still required in card/orb/power long-tail interactions.

## Determinism and RNG status
- Phase-0 hardening landed in PR #23:
  - shared effect context RNG helper methods
  - shared registry base-context RNG helper methods
  - card effect random-choice paths moved off Python global RNG
  - power handlers (`Magnetism`, `CreativeAI`, `Study`, `Juggernaut`) moved off Python global RNG
- Remaining direct Python `random` callsites in `packages/engine/`:
  - `packages/engine/registry/relics.py` (11)
  - `packages/engine/effects/defect_cards.py` (4)
  - `packages/engine/effects/orbs.py` (3)
  - `packages/engine/registry/potions.py` (2)
  - `packages/engine/game.py` (2)
  - `packages/engine/calc/combat_sim.py` (1, simulation helper only)

## Remaining work by PR region (locked execution order)

### Region C1: Cards (non-Defect)
Scope:
- Ironclad, Silent, Watcher, and required shared colorless/curse/status behavior

Required outputs:
- card manifest rows: Java class/ID -> Python card/effect mapping -> status (`exact|approximate|missing`)
- tests-first parity closures by mechanic family:
  - selection-required card decisions
  - RNG-sensitive card behavior
  - upgraded/base stat timing parity

Completion gate:
- non-Defect card manifest has no unmapped rows without explicit disposition
- targeted character suites green + full suite green

### Region C2: Orbs + Powers
Scope:
- `ORB-001`, `POW-001`, `POW-002`, `POW-003`

Required outputs:
- close power inventory gap with explicit per-class rows and statuses
- finalize orb timing/runtime semantics used by relic/power/card interactions
- replace remaining orb/power/relic runtime `random` usage with owned streams
- integration tests for orb + power + relic interactions

Completion gate:
- no placeholder orb behavior in active runtime hooks
- power manifest rows all resolved for target scope
- integration suites green

### Region C3: Defect cards
Scope:
- Defect card behavior closure after orb semantics are stable

Required outputs:
- re-audit Defect cards against Java semantics with finalized orbs
- close Defect-only selection and trigger-order gaps

Completion gate:
- Defect card manifest closed to exact/approved states
- Defect + integration suites green

### Region C4: Final audit + RL launch gate
Scope:
- `AUD-001`, `AUD-002`, `AUD-003`

Required outputs:
- clean Java-vs-Python gap manifest
- default CI path with no skips
- versioned observation/action contract markers and RL readiness sign-off

Completion gate:
- `uv run pytest tests/ -q` -> `0 skipped, 0 failed`
- RL checklist fully green

## Test-system cleanup still required
- Move replay-artifact-dependent parity checks out of default test profile
- Remove contingency skips in agent/integration coverage paths by deterministic fixtures
- Convert bug-documentation audit tests into parity-enforcing tests

## Cleanup/deletion policy (targeted only)
- Remove dead placeholder paths only when replacement parity behavior is landed
- Avoid broad doc deletion until parity manifests and gates are fully green
- Keep generated manifest docs canonical; archive superseded ad-hoc notes after each merge
