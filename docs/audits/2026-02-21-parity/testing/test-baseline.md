# Test Baseline

Date: 2026-02-22

## Full suite
```bash
uv run pytest tests/ -ra
```

Result:
- `4610 passed`
- `5 skipped`
- `0 failed`

Skip details:
- `tests/test_parity.py` skips when `consolidated_seed_run.jsonl` is absent (5 skips).

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
- Added potion action-surface completeness checks for full hand-subset candidates:
  - `tests/test_agent_api.py` (Gamblers Brew + Elixir selection enumeration)
- Added RNG stream advancement checks for potion runtime:
  - `tests/test_potion_rng_streams.py`
- Added Fairy defeat-prevention invariant coverage:
  - `tests/test_potion_sacred_bark.py`
- Added runtime/registry de-dup coverage:
  - `tests/test_potion_runtime_dedup.py`
- Added relic action-surface coverage:
  - `tests/test_agent_api.py` (Astrolabe boss-relic pick requires and resolves `select_cards` flow)
  - `tests/test_agent_api.py` (Empty Cage boss-relic pick requires and resolves `select_cards` flow)

## Focused verification commands
```bash
uv run pytest tests/test_relic_eggs.py tests/test_relic_rest_site.py tests/test_relic_acquisition.py tests/test_relic_triggers_outofcombat.py -q
```

Focused result:
- `47 passed`
- `0 skipped`
- `0 failed`
