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
- 2026-03-30: 5 PRs merged (#75-79). F16 wall analysis complete. Next: value head normalization, Rust engine scoping, auto-research system.

## Immediate (next session)
- [ ] Value head return normalization — bimodal rewards (0-2 vs 13-15) make value head unlearnable
- [ ] Codex plugin dead code audit — use new codex/review plugin for systematic cleanup
- [ ] Hyperparam sweep setup — configs in sweep_config.py, A/B test via `training.sh experiment`
- [ ] Refine iMessage text system — test on branch, better formatting

## Medium Term
- [ ] Auto-research system — interval-based audits + fixes via /loop + /schedule
- [ ] Rust engine migration scoping — audit Java source, estimate effort, plan 2-agent approach
- [ ] Dashboard frontend — jackswitzer.com page consuming gist data (gist: 6fd47c26377bb8b9e16802f85d771348)
