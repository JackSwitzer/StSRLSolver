# Core TODO: Full-Game Java Parity + RL Readiness

Last updated: 2026-02-22

Execution queue: [`EXECUTION_QUEUE.md`](./EXECUTION_QUEUE.md)

## Baseline
- Full suite baseline: `4610 passed, 5 skipped, 0 failed`
- Command: `uv run pytest tests/ -q`
- Current skip source:
  - `tests/test_parity.py` artifact-dependent JSONL checks

## Global gates
- [ ] Canonical traceability manifest exists for all gaps.
- [ ] Every choice interaction uses explicit action dict flow.
- [ ] Normal CI is `0 skipped, 0 failed`.
- [ ] RL readiness checklist is fully green.

## Region order (locked)
1. R0 docs + test scaffolding
2. R1 relic selection surface
3. R2 event selection surface
4. R3 reward flow normalization
5. R4 powers + orbs closure
6. R5 cards long-tail closure
7. R6 final re-audit + RL gate

## Active region: R0 (docs + tests scaffolding)
- [ ] `DOC-001` Create canonical audit suite and legacy pointer policy.
- [ ] `DOC-002` Populate Java/Python inventories and gap manifest.
- [ ] `TST-001` Add traceability matrix tests.
- [ ] `TST-002` Add action-surface completeness tests.

## Next region: R1 (relic selection surface)
- [ ] `REL-003` Orrery explicit selection actions.
- [ ] `REL-004` Bottled relic assignment actions.
- [ ] `REL-008` Dolly's Mirror explicit selection actions.
- [ ] `REL-005` Deterministic selection action IDs/validation.
- [ ] `REL-006` Relic alias normalization and Java ID coverage (`Toolbox` included).
- [ ] `REL-007` Remaining boss/chest/reward ordering regressions.

## High-impact open gaps (confirmed)
- [ ] Event action path still sends `card_idx=None` for selection events.
- [ ] Runtime relic on-acquire still auto-picks for Orrery/bottled/Dolly in model-facing flows.
- [ ] `Toolbox` present in generation inventory but absent from content registry.
- [ ] Power coverage has significant class-level remaining gaps.
- [ ] Orb-linked relic behavior includes placeholder TODO logic.

## Policy reminders
- [ ] Per feature loop: `docs -> tests -> code -> commit -> todo update`.
- [ ] One feature ID per commit.
- [ ] Domain PRs only (one region per PR).
- [ ] Include Java refs + RNG notes + test deltas in every PR.
