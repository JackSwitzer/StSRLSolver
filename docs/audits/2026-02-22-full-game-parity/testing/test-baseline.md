# Test Baseline

Date: 2026-02-23

## Full suite baseline
Command:
```bash
uv run pytest tests/ -q
```

Result:
- `4715 passed`
- `0 skipped`
- `0 failed`

Re-verified after `CONS-DESKTOP-001` (one-folder desktop realignment) and `CONS-002A` (CombatRunner facade over CombatEngine).

## Structural inventory
- Test files: `75`
- Static test-function definitions (`def test_`): `4137`

## Quality gates
- Current local baseline satisfies `0 skipped, 0 failed`.
- Keep parity replay checks in a dedicated profile/job if external artifacts are required.

## Regression rules
- No new skips without an explicit manifest row and burn-down plan.
- Every feature commit runs targeted tests plus the full-suite baseline command.
