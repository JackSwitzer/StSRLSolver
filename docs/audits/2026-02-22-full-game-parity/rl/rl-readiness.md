# RL Readiness Checklist

## Preconditions
- [ ] Full-game parity manifest is up to date.
- [ ] All parity-critical gaps are closed or explicitly deferred with justification.
- [ ] Normal CI is `0 skipped, 0 failed`.

## Action/observation contract
- [ ] Every choice interaction is represented as explicit action dicts.
- [ ] No hidden UI-only state transitions remain.
- [ ] Action IDs are deterministic for equivalent snapshots.
- [ ] Observation contract is stable and versioned.

## Determinism
- [ ] RNG stream usage is documented per parity-critical mechanic.
- [ ] RNG advancement tests exist for all audited streams.
- [ ] Replay/seed checks are reproducible in automation.

## Training launch gate
- [ ] Final audit sign-off complete.
- [ ] This file marked complete and reviewed.
