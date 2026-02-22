# Test Baseline

Date: 2026-02-22

## Full suite baseline
Command:
```bash
uv run pytest tests/ -q
```

Result:
- `4638 passed`
- `5 skipped`
- `0 failed`

## Skip inventory (normal run)
Current skip source:
- `tests/test_parity.py` skips when `consolidated_seed_run.jsonl` is missing.

## Quality notes (open debt)
- Some audit tests still document known bugs instead of asserting Java parity behavior after a fix.
- Known examples:
  - `tests/test_audit_damage.py` (Torii ordering note)

## Cleanup targets
- Normal CI gate target: `0 skipped, 0 failed`.
- Artifact-dependent replay checks should run in a separate profile/job.
- Convert bug-documentation tests to strict parity assertions as each feature lands.

## Regression rules
- No new skips without manifest row and expiry plan.
- Every feature commit must run targeted tests and full suite before merge.
