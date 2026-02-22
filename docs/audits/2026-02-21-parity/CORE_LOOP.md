# Core Loop Runbook

Last updated: 2026-02-22

This runbook defines the reusable loop for closing Java parity and agent-action completeness without context drift.

## Loop contract (single feature)
1. **Docs first**
- update one domain audit file and `EXECUTION_QUEUE.md`/`CORE_TODO.md` feature status
- capture Java class/method references and RNG stream expectations

2. **Tests second**
- add tests that fail before implementation for the target feature
- include deterministic assertions for action IDs and RNG counters when applicable

3. **Code third**
- implement only what is necessary for those tests
- avoid unrelated refactors in feature commits

4. **Commit fourth**
- one feature ID per code commit (`POT-xxx`, `REL-xxx`, etc.)
- keep commit scope bounded (prefer <=10 files and <=400 net LOC when possible)

5. **Tracker update fifth**
- update `CORE_TODO.md` checkboxes
- update `docs/audits/2026-02-21-parity/testing/test-baseline.md` if test totals changed

## Parallel lane model (for delegated/subagent work)
- Lane A: action-surface/state-machine features (`packages/engine/game.py`, selection contexts)
- Lane B: potions/relic runtime parity (`registry`, `combat_engine`, room/reward handlers)
- Lane C: events/powers parity (`event_handler.py`, `registry/powers.py`, hook-order tests)
- Lane D: audits/docs/test snapshots (domain docs, parity scripts, baseline updates)

## Integrator rules
- Only merge feature branches after lane-targeted tests pass.
- Run full suite before merge to core branch.
- If overlap exists in `packages/engine/game.py`, rebase non-owner lane branches after owner merge.
- Never include unrelated untracked artifacts in feature commits.

## Definition of done (campaign)
- Every decision/interaction requiring choice has explicit action dict representation.
- Java behavior mismatch list is empty in audit domains.
- Full suite stays green with parity-relevant skips not reintroduced.
- Work-unit checklists and audit docs remain synchronized.
