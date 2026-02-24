# Test Baseline

Date: 2026-02-24

## Full suite baseline
Command:
```bash
uv run pytest tests/ -q
```

Result:
- `4722 passed`
- `0 skipped`
- `0 failed`

Re-verified after `DOC-TODO-001`, `DOC-ACTION-001`, `DOC-WFLOW-001`, `AUD-GEN-001/002/003`, and `CRD-INV-003A/B` updates.

## Structural inventory
- Test files: `76`
- Static test-function definitions (`def test_`): `4144`

## Quality gates
- Current local baseline satisfies `0 skipped, 0 failed`.
- Keep parity replay checks in a dedicated profile/job if external artifacts are required.

## Regression rules
- No new skips without an explicit manifest row and burn-down plan.
- Every feature commit runs targeted tests plus the full-suite baseline command.
