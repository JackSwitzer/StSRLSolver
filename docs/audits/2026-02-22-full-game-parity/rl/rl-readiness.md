# RL Readiness Checklist

Last updated: 2026-02-23

## Current status snapshot
- [x] Canonical repo lock + consolidation manifest in place.
- [x] Reward/event/relic action-surface critical fixes integrated.
- [x] Java power inventory mapping closure is complete (manifest: `missing=0`).
- [x] Orb parity foundation (`ORB-001`) integrated.
- [ ] Remaining power hook dispatch closure (`POW-002`) is complete.
- [ ] Normal CI path is `0 skipped, 0 failed`.

## Preconditions
- [x] Script-generated full-game parity manifests exist (`java-inventory`, `python-inventory`, `parity-diff`, `power-hook-coverage`).
- [ ] All parity-critical gaps are closed or explicitly deferred with justification.
- [ ] Gameplay-critical randomness uses owned RNG streams only.

## Action/observation contract
- [x] Core choice interactions in audited domains use explicit action dicts.
- [x] Missing selection params produce explicit candidate actions.
- [x] Action IDs are deterministic for equivalent snapshots.
- [x] Observation contract version fields are emitted (`observation_schema_version`, `action_schema_version`).

## Determinism
- [ ] RNG stream usage is fully documented per parity-critical mechanic.
- [ ] RNG advancement tests exist for all remaining high-impact mechanics.
- [ ] No direct Python `random` in parity-critical engine execution paths.
- [ ] Replay/seed checks are reproducible in automation profile.

## Test quality gate
- [x] Full suite currently green (`4708 passed, 5 skipped, 0 failed`).
- [ ] Default CI has no skips.
- [ ] Replay artifact checks moved to dedicated parity job/profile.
- [ ] Contingency skips in core API tests replaced by deterministic fixtures.

## Launch gate (must all be true)
- [ ] `POW-002` and remaining `POW-003` integration closure complete.
- [ ] `CRD-*` closure accepted for target training scope.
- [ ] `CONS-002` combat runtime unification complete.
- [ ] `AUD-001`/`AUD-002`/`AUD-003` complete.
- [ ] Final sign-off recorded in `GROUND_TRUTH.md`.
