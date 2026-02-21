# Test Baseline

Date: 2026-02-21

## Full suite
```bash
uv run pytest tests/ -ra
```

Result:
- `4602 passed`
- `0 skipped`
- `0 failed`

## Notable delta from start of pass
- Replaced mock/placeholder relic suites with engine-backed tests:
  - `tests/test_relic_rest_site.py`
  - `tests/test_relic_acquisition.py`
  - `tests/test_relic_triggers_outofcombat.py`
- Added run-state coverage for on-obtain-card relic effects (Ceramic Fish, Egg relics, Darkstone Periapt).
- Added room-entry/chest parity coverage for Maw Bank and N'loth's Hungry Face.
- Added dedicated egg integration suite covering shop/reward/direct acquisition paths:
  - `tests/test_relic_eggs.py`
- Added potion parity coverage for:
  - Distilled Chaos autoplay semantics
  - Entropic Brew class-pool behavior
  - Smoke Bomb back-attack restriction
  - selection potion `onUsePotion` relic trigger propagation

## Focused verification commands
```bash
uv run pytest tests/test_relic_eggs.py tests/test_relic_rest_site.py tests/test_relic_acquisition.py tests/test_relic_triggers_outofcombat.py -q
```

Focused result:
- `47 passed`
- `0 skipped`
- `0 failed`
