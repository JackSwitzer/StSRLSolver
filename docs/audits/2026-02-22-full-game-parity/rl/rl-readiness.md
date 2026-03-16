# RL Readiness Checklist

Last updated: 2026-02-24

> Update 2026-03-16:
> The canonical engine API is stronger than the current training integrations.
> RL readiness is blocked by:
> - semantic candidate blindness in strategic inference
> - `256` strategic action-cap truncation
> - non-canonical / lossy training artifacts
> - reward/replay behavior that is not aligned with the EV framing
> - runtime parity gaps that leak directly into non-combat training
>
> See `docs/audits/2026-03-16-full-audit-gap-inventory.md` and
> `docs/audits/2026-03-16-rl-contract-matrix.md`.

## Current status snapshot
- [x] Canonical repo lock + consolidation manifest in place.
- [x] Reward/event/relic action-surface critical fixes integrated.
- [x] Java power inventory mapping closure is complete (manifest: `missing=0`).
- [x] Orb parity foundation (`ORB-001`) integrated.
- [x] Power hook dispatch coverage artifact is complete (`registered_not_dispatched = []`).
- [x] Normal CI path is `0 skipped, 0 failed`.

## Preconditions
- [x] Script-generated full-game parity manifests exist (`java-inventory`, `python-inventory`, `parity-diff`, `power-hook-coverage`).
- [ ] All parity-critical behavior gaps are closed or explicitly deferred with justification.
- [ ] Gameplay-critical randomness uses owned RNG streams only.

## Action/observation contract
- [x] Core choice interactions in audited domains use explicit action dicts.
- [x] Missing selection params produce explicit candidate actions.
- [x] Action IDs are deterministic for equivalent snapshots.
- [x] Observation contract version fields are emitted (`observation_schema_version`, `action_schema_version`).
- [x] Canonical action-space spec documented (`action-layer/ACTION_SPACE_SPEC.md`).

## Determinism
- [x] RNG stream usage ownership is documented (`rng/JAVA_RNG_STREAM_SPEC.md`).
- [ ] RNG advancement tests exist for all remaining high-impact mechanics.
- [ ] No direct Python `random` in parity-critical engine execution paths.
- [ ] Replay/seed checks are reproducible in automation profile.

## Test quality gate
- [x] Full suite currently green (`4722 passed, 0 skipped, 0 failed`).
- [x] Default CI has no skips.
- [ ] Replay artifact checks moved to dedicated parity job/profile (if needed by CI).
- [ ] Contingency skips in core API tests replaced by deterministic fixtures.

## Launch gate (must all be true)
- [ ] `POW-002B`/`POW-003*` behavior and ordering closure complete.
- [ ] `CRD-*` behavior closure accepted for target training scope.
- [ ] `RNG-MOD-*` migration complete for parity-critical runtime paths.
- [ ] `RL-ACT`/`RL-OBS` contracts finalized and test-locked.
- [ ] `AUD-003` complete and final sign-off recorded in `GROUND_TRUTH.md`.
