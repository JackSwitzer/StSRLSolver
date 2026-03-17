# Subagent Execution Loop (Canonical)

Last updated: 2026-02-24
Feature ID: `DOC-WFLOW-001`

## Lane ownership
- Lane A: docs + generated audits (`DOC-*`, `AUD-*`)
- Lane B: cards (`CRD-*`)
- Lane C: powers/orbs integration (`POW-*`)
- Lane D: RNG + RL contracts (`RNG-*`, `RL-*`)
- Integrator: rebases, test gates, tracker sync, commit hygiene

## Batch policy
- Target <=10 files and <=400 net LOC per feature.
- One feature ID per commit.
- Region-oriented PRs only.

## Mandatory sequence per feature
1. docs: update audit docs, Java refs, and acceptance criteria
2. tests: add or tighten tests first
3. code: minimal parity-correct implementation
4. tracker: update `TODO.md` and `CORE_TODO.md`
5. verify: targeted tests + full suite
6. commit: single feature ID in message

## Merge gate
- Targeted tests pass for touched domain.
- Full suite passes: `uv run pytest tests/ -q`.
- Audit artifacts regenerated if affected.
- Test delta + skip delta captured in tracker update.

## Conflict policy
- If feature overlaps shared runtime files (`game.py`, `combat_engine.py`, registry glue), integrator merges that chunk first.
- Other lanes rebase before next batch.
- Do not carry stale generated artifacts from old base; regenerate on current base.

## Documentation hygiene rules
- Every changed behavior references Java class/method when available.
- RNG-affecting changes include explicit stream notes.
- If behavior intentionally diverges, include rationale and test lock.

## Done criteria for a feature ID
- Acceptance test(s) exist and pass.
- No undocumented behavior deltas.
- Trackers reflect current truth.
- Feature is merge-ready without hidden prerequisites.
