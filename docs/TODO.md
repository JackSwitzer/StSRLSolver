# Active Work Units

Auto-generated from `docs/work_units/` YAML frontmatter. Updated via `/pr`.

## FOUNDATION

### P0
- **[data-pipeline](work_units/data-pipeline.md)** — Consolidate scattered data, assess quality, organize storage, extract data_utils.py, logging spec

### P1
- **[engine-parity](work_units/engine-parity.md)** — Legacy parity checklist; remaining work is a smaller, targeted tail. Not blocking training, matters for the long-run WR target.
- **[runtime-hardening](work_units/runtime-hardening.md)** — Disk monitoring, exception audit, config verification tests, auto-pause, solver budget runtime tests
  - depends on: data-pipeline

## TRAINING

### P0
- **[training-architecture](work_units/training-architecture.md)** — Algorithm choice, solver budget fix, value head, pretrain strategy, 5-day run plan
  - depends on: data-pipeline, runtime-hardening

### Active Engine Parity (sub-items)
- [granular-events.md](work_units/granular-events.md) — legacy checklist, use as reference for the remaining event-tail follow-up
- [granular-powers.md](work_units/granular-powers.md) — legacy checklist, use as reference for the remaining power-tail follow-up
- [granular-relics.md](work_units/granular-relics.md) — legacy checklist, use as reference for the remaining relic-tail follow-up

## VISIBILITY

### P0
- **[tooling](work_units/tooling.md)** — Hooks, skills (/pretrain, /experiment, /training-status, /data-audit), /pr auto-update, session discipline

### P1
- **[dashboard](work_units/dashboard.md)** — Decision quality, convergence, data inventory, run comparison, cleaner UI
  - depends on: data-pipeline

## Completed
See [COMPLETED.md](COMPLETED.md) for closed work units and full history.

## Latest audit note
- 2026-03-25: See `docs/research/optimization-bug-audit-2026-03-25.md` for the current runtime/optimization audit and the remaining parity follow-up notes.
