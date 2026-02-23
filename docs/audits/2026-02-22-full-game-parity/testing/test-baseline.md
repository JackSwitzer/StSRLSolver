# Test Baseline

Date: 2026-02-23

## Full suite baseline
Command:
```bash
uv run pytest tests/ -q
```

Result:
- `4684 passed`
- `5 skipped`
- `0 failed`

Re-verified after `POW-002` dispatch closure and `POW-003A` alias/lifecycle power hook slice.

## Structural inventory
- Test files: `70`
- Static test-function definitions (`def test_`): `4086`

## Executed skip inventory (normal run)
All executed skips are from replay-artifact gating in `tests/test_parity.py`:
- `tests/test_parity.py:614`
- `tests/test_parity.py:620`
- `tests/test_parity.py:628`
- `tests/test_parity.py:639`
- `tests/test_parity.py:669`

Reason:
- `consolidated_seed_run.jsonl` not present in expected logs path.

## Additional contingency skip callsites (not all execute in baseline run)
- `tests/test_agent_api.py` (room reachability fallback skips)
- `tests/test_integration.py` (optional effect-registry availability checks)
- `tests/test_coverage_boost.py` (`RunState.get_starter_relic` fallback skip)

## Quality gates
- Normal CI target remains: `0 skipped, 0 failed`.
- Replay-artifact tests should run in a dedicated parity profile/job, not default CI path.
- Contingency skips in core agent API tests should be replaced with deterministic fixtures.

## Regression rules
- No new skips without a manifest row and explicit burn-down plan.
- Every feature commit runs targeted tests plus full-suite baseline command.
