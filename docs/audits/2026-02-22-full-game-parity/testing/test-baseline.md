# Test Baseline

Date: 2026-02-22

## Full suite baseline
Command:
```bash
uv run pytest tests/ -q
```

Result:
- `4610 passed`
- `5 skipped`
- `0 failed`

## Skip inventory (normal run)
Current known skip source:
- `tests/test_parity.py` skips when `consolidated_seed_run.jsonl` is missing.

## Cleanup target
- Normal CI gate target: `0 skipped, 0 failed`.
- Artifact-dependent replay tests must be split from default CI profile.

## Regression rules
- No new skips without explicit manifest row and temporary expiry plan.
- Any skip introduced in a feature must be removed before region sign-off.
