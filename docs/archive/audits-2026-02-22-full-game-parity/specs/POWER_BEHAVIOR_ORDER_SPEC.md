# Power Behavior + Hook Order Spec (POW-002B / POW-003A / POW-003B)

Last updated: 2026-02-24
Status: spec-lock complete
Parent index: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`

## Objective
Close remaining power parity after inventory and hook-dispatch inventory closure. Focus is hook ordering and per-power behavior semantics.

## Source of truth
- Java powers: `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers`
- Python power registry: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/registry/powers.py`
- Runtime dispatch owner: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/combat_engine.py`
- Runtime helper hooks: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/effects/registry.py`
- Work-unit list: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/work_units/granular-powers.md`

## Backlog snapshot (2026-02-24)
- Dispatch inventory: `25/25` hooks dispatched.
- Remaining unchecked rows in `granular-powers.md`: `43`.
- Remaining work is ordering + behavior semantics, not inventory mapping.

## Canonical runtime ownership
- `CombatEngine` is the implementation owner.
- `CombatRunner` is compatibility shim only.
- New behavior should not be implemented in `handlers/combat.py` except delegation/shim compatibility.

## Hook order contract (locked)

### Start of player turn
1. Relic `atTurnStart`
2. Power `atStartOfTurn` (player)
3. Orb passives at start of turn
4. Power `onEnergyRecharge`
5. Draw
6. Relic `atTurnStartPostDraw`
7. Power `atStartOfTurnPostDraw`

### Card play chain
1. Relic `onPlayCard`
2. Card effect resolution
3. Card destination handling
4. Power `onUseCard`
5. Power `onAfterUseCard` (player then enemies)
6. Power `onAfterCardPlayed` (player then enemies)

### Incoming damage chain
1. Attacker `atDamageGive`
2. Defender `atDamageReceive`
3. Defender `atDamageFinalReceive`
4. Defender `onAttackedToChangeDamage`
5. Apply block and HP loss
6. Relic `wasHPLost`
7. Power `wasHPLost`
8. Attacker `onAttack`
9. Defender `onAttacked`

### End turn and end round
1. Relic `onPlayerEndTurn`
2. Power `atEndOfTurnPreEndTurnCards` (player)
3. Power `atEndOfTurn` (player then enemies)
4. Enemy turns
5. Power `atEndOfRound` (player then enemies)

## Unit features and acceptance

### `POW-002B` hook ordering and trigger-count closure
Dependencies: `POW-002A`

Scope:
1. Add strict order assertions for start-turn, card-play, damage, and end-turn chains.
2. Add trigger-count assertions for representative powers per hook family.
3. Remove undocumented order divergences or document exceptions with Java references.

Acceptance:
- `tests/test_power_registry_integration.py` order assertions green.
- `tests/test_audit_power_dispatch.py` stays green.
- `power-hook-coverage.json` remains clean for dispatch inventory.

### `POW-003A` behavior closure by hook family
Dependencies: `POW-002B`

Scope:
1. Close unresolved rows in `granular-powers.md` by behavior family:
   - damage and block modifiers,
   - card-use hooks,
   - turn hooks,
   - lifecycle hooks.
2. Close high-impact enemy/boss powers that alter combat legality or math.
3. Ensure all changed status application paths use canonical power ID normalization.

Acceptance:
- `tests/test_powers.py` and `tests/test_power_edge_cases.py` green.
- Updated rows in `granular-powers.md` with Java refs and resolved status.

### `POW-003B` cross-system integration lock
Dependencies: `POW-003A`

Scope:
1. Add integration tests for powers + cards + relics + orbs interactions.
2. Validate deterministic replay for mixed hook chains.

Acceptance:
- `tests/test_effects_and_combat.py` and related integration suites include multi-system assertions.
- Same seed + same action sequence yields identical outcomes.

## Required evidence per feature commit
1. Java class and overridden hook references.
2. Trigger-order evidence from tests (expected sequence and observed sequence).
3. Runtime ownership note confirming `CombatEngine` path.
4. Tracker updates in `TODO.md`, `CORE_TODO.md`, and `UNIT_CHUNKS.md`.

## Done definition
1. `POW-002B`, `POW-003A`, and `POW-003B` are `completed` in `UNIT_CHUNKS.md`.
2. `granular-powers.md` contains no unresolved behavior rows without explicit defer reason.
3. Full suite remains green with `uv run pytest tests/ -q`.
