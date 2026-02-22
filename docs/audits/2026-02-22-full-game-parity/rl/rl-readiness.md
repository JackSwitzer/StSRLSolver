# RL Readiness Checklist

Last updated: 2026-02-22

## Current status snapshot
- [x] Merged parity chain for `R1` to `R3` is on `main` (through PR #25).
- [x] Reward/event/relic action-surface critical fixes are integrated.
- [ ] Powers inventory and long-tail behavior parity are complete.
- [ ] Orb infrastructure parity is complete.
- [ ] Normal CI path is `0 skipped, 0 failed`.

## Preconditions
- [ ] Full-game parity manifest is complete for remaining powers/cards/orbs.
- [ ] All parity-critical gaps are closed or explicitly deferred with justification.
- [ ] Gameplay-critical randomness uses owned RNG streams only.

## Action/observation contract
- [x] Core choice interactions in audited domains use explicit action dicts.
- [x] Missing selection params produce explicit candidate actions.
- [x] Action IDs are deterministic for equivalent snapshots.
- [ ] Observation contract is frozen/versioned for training artifacts.

## Determinism
- [ ] RNG stream usage is fully documented per parity-critical mechanic.
- [ ] RNG advancement tests exist for all remaining high-impact mechanics.
- [ ] No direct Python `random` in parity-critical engine execution paths.
- [ ] Replay/seed checks are reproducible in automation profile.

## Test quality gate
- [x] Full suite currently green (`4669 passed, 5 skipped, 0 failed`).
- [ ] Default CI has no skips.
- [ ] Replay artifact checks moved to dedicated parity job/profile.
- [ ] Contingency skips in core API tests replaced by deterministic fixtures.

## Launch gate (must all be true)
- [ ] `POW-001`, `POW-002`, `POW-003` closed.
- [ ] `ORB-001` closed.
- [ ] `CRD-*` closure accepted for target training scope.
- [ ] `AUD-001`, `AUD-002`, `AUD-003` complete.
- [ ] Final sign-off recorded in `GROUND_TRUTH.md`.
