# RL Readiness

## Current verdict
Engine is test-clean and runnable, but not yet full-parity-safe for high-confidence RL training due selection/action-model gaps (mostly potions + relic pickup flows).

## Minimum gate before large training runs
- Keep full suite green: `uv run pytest tests/ -ra`
- Close P0 items in [`../CORE_TODO.md`](../CORE_TODO.md)
- Unskip critical relic/potion selection suites or replace with equivalent mandatory tests.

## Safe-to-run now (for infra smoke tests)
- Headless short runs
- Combat-heavy evaluation where unresolved event/potion branches are minimized

## Suggested temporary guardrails
- Avoid or de-prioritize unresolved potion decision branches (`Discovery`, `DistilledChaos`, `LiquidMemories`, `GamblersBrew`, `Elixir`, `StancePotion`).
- Treat complex relic pickup choices as deterministic fallback until selection APIs are implemented.
- Keep event policy conservative while action-level parity is incomplete.

## Runbook: verify game loop executes
```bash
uv run python - <<'PY'
from packages.engine.game import run_headless

result = run_headless(seed=123456789, ascension=20, max_actions=3000)
print({
    "seed": result.seed,
    "victory": result.victory,
    "floor": result.floor_reached,
    "hp": result.hp_remaining,
    "combats_won": result.combats_won,
})
PY
```

## Runbook: multi-seed smoke test
```bash
uv run python - <<'PY'
from packages.engine.game import run_parallel

seeds = [111, 222, 333, 444, 555]
results = run_parallel(seeds, ascension=20, max_workers=2)
for r in results:
    print(r.seed, r.victory, r.floor_reached, r.combats_won)
PY
```
