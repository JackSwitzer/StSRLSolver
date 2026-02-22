# Java Parity Audit (2026-02-21)

This audit consolidates remaining Java -> Python parity work before RL training.

> Legacy note: this suite is archived for historical context. Active campaign tracking now lives in `docs/audits/2026-02-22-full-game-parity/`.

## Entry points
- Core TODO list: [`CORE_TODO.md`](./CORE_TODO.md)
- Test baseline and commands: [`testing/test-baseline.md`](./testing/test-baseline.md)
- RL readiness and runbook: [`rl/rl-readiness.md`](./rl/rl-readiness.md)

## Domain deep-dives
- Potions: [`domains/potions.md`](./domains/potions.md)
- Events: [`domains/events.md`](./domains/events.md)
- Relics: [`domains/relics.md`](./domains/relics.md)
- Powers: [`domains/powers.md`](./domains/powers.md)

## Inputs inherited from prior audits
- Composer 1.5 update (split spawn/action-result/test status and parity gap list)
- Work-unit specs in `docs/work_units/granular-*.md`
- Current code + test baseline from `uv run pytest tests/ -ra`
