# Full-Game Parity Audit (2026-02-22)

This is the canonical parity campaign for Java-exact behavior and model-facing action completeness.

## Canonical workspace
- All campaign edits happen in:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees/parity-core-loop`
- Do not split implementation across multiple local clones/worktrees for this campaign.

## Campaign goals
1. Close Java vs Python behavior gaps across full game systems.
2. Make every choice interaction explicit in action dict APIs.
3. Reach clean CI with zero skips in normal runs.
4. Clear RL readiness gate only after parity/action closure.

## Core entry points
- Core TODO: [`CORE_TODO.md`](./CORE_TODO.md)
- Execution queue: [`EXECUTION_QUEUE.md`](./EXECUTION_QUEUE.md)
- Test baseline: [`testing/test-baseline.md`](./testing/test-baseline.md)
- RL readiness checklist: [`rl/rl-readiness.md`](./rl/rl-readiness.md)

## Traceability (source of truth)
- Java inventory: [`traceability/java-inventory.md`](./traceability/java-inventory.md)
- Python coverage inventory: [`traceability/python-coverage.md`](./traceability/python-coverage.md)
- Gap manifest: [`traceability/gap-manifest.md`](./traceability/gap-manifest.md)

## Domain audits
- Potions: [`domains/potions.md`](./domains/potions.md)
- Relics: [`domains/relics.md`](./domains/relics.md)
- Events: [`domains/events.md`](./domains/events.md)
- Powers: [`domains/powers.md`](./domains/powers.md)
- Cards: [`domains/cards.md`](./domains/cards.md)
- Rewards/Shops/Rest/Map: [`domains/rewards-shops-rest-map.md`](./domains/rewards-shops-rest-map.md)
- Orbs/System support: [`domains/orbs.md`](./domains/orbs.md)

## Legacy suite status
- Legacy audit retained for history only:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees/parity-core-loop/docs/audits/2026-02-21-parity`
- New work must be tracked in this 2026-02-22 suite.
