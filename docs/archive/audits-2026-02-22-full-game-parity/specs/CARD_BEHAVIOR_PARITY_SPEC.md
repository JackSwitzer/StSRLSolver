# Card Behavior Parity Spec (CRD-IC/SI/WA/SH/DE)

Last updated: 2026-02-24
Status: spec-lock complete
Parent index: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`

## Objective
Close remaining Java behavior parity for cards now that ID/inventory mapping is complete (`cards missing = 0` in generated diff).

## Source of truth
- Java: `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards`
- Python card data: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/content/cards.py`
- Python card execution:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/effects/cards.py`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/effects/defect_cards.py`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/effects/executor.py`
- Authoritative checklists:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/work_units/granular-cards-ironclad.md`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/work_units/granular-cards-silent.md`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/work_units/granular-cards-watcher.md`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/work_units/granular-cards-defect.md`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/work_units/granular-cards-shared.md`

## Backlog snapshot (2026-02-24)
- Ironclad unchecked rows: `62`
- Silent unchecked rows: `61`
- Watcher unchecked rows: `6`
- Defect unchecked rows: `68`
- Shared unchecked rows: `0`

These counts are execution estimates. Resolution status is authoritative only in work-unit rows plus tests.

## Unit features and acceptance

### `CRD-IC-001` Ironclad behavior closure
Dependencies: `CRD-INV-003B`

Scope:
1. Resolve effect timing and value parity for open Ironclad rows.
2. Ensure selection cards use explicit action flow (`requires_selection` then `select_cards`).
3. Confirm X-cost and random-target handling is Java-consistent.

Acceptance:
- `tests/test_ironclad_cards.py` green.
- Added/updated timing assertions for affected cards.
- Updated rows in `granular-cards-ironclad.md` with `exact` or explicit defer reason.

### `CRD-SI-001` Silent behavior closure
Dependencies: `CRD-INV-003B`

Scope:
1. Resolve discard, retain, and next-turn timing parity.
2. Resolve poison/debuff scaling and transfer behavior.
3. Resolve random target and X-cost card behavior parity.

Acceptance:
- `tests/test_silent_cards.py` green.
- Deterministic selection/discard coverage present.
- Updated rows in `granular-cards-silent.md`.

### `CRD-WA-001` Watcher behavior closure
Dependencies: `CRD-INV-003B`

Scope:
1. Close remaining stance-change branch behavior, including `InnerPeace`.
2. Lock `onChangeStance` trigger ordering parity.
3. Verify base/upgraded branch values.

Acceptance:
- `tests/test_cards.py` and `tests/test_watcher_card_effects.py` include branch-order assertions.
- Updated rows in `granular-cards-watcher.md`.

### `CRD-SH-002` Shared colorless/curse/status closure
Dependencies: `CRD-INV-003B`

Scope:
1. Confirm all shared rows are either exact or explicitly deferred with evidence.
2. Lock curse/status turn-end behavior ordering where still approximate.

Acceptance:
- `tests/test_status_curse.py` and related combat suites green.
- No unresolved shared-row ambiguity in work-unit docs.

### `CRD-DE-001` Defect behavior closure
Dependencies: `POW-003A`, `ORB-001` (already merged)

Scope:
1. Close remaining Defect rows after power/orb behavior is stable.
2. Enforce RNG-stream ownership on random Defect card effects.
3. Ensure selection-required Defect cards use explicit action follow-up.

Acceptance:
- `tests/test_defect_cards.py` and integration suites green.
- Updated rows in `granular-cards-defect.md` with Java refs and RNG notes.

## Cross-cutting invariants
1. Choice mechanics remain explicit in the action API (no hidden UI assumptions).
2. Same seed + same action sequence is deterministic.
3. No direct `random.*` in touched parity-critical runtime paths.
4. Base/upgraded values and timing match Java behavior.

## Required evidence per feature commit
1. Java class and method references for each behavior change.
2. Test delta listing targeted suites and full-suite result.
3. RNG stream note for every touched random path.
4. Tracker updates in `TODO.md`, `CORE_TODO.md`, and `UNIT_CHUNKS.md`.

## Done definition
1. All card feature IDs in this spec are `completed` in `UNIT_CHUNKS.md`.
2. Card work-unit rows have no unresolved parity items without explicit defer rationale.
3. Full suite remains green with `uv run pytest tests/ -q`.
