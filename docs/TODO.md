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
- 2026-04-10: 8-agent comprehensive audit complete. 26 effect tags migrated (Steps 0-2). 10 broken cards found. Dead code identified (status_keys.rs, 7 dead branches, 4 unused functions). MCTS perf opportunities: static CardRegistry, relic bitset, SmallVec. Parity: cards ~100%, enemies ~95%, relics ~100%, potions 100%, events ~75%, powers ~95%.

## Current Sprint (on `feat/rust-engine`)

### In Progress
- [x] Modular effects scaffold (Steps 0-2: 26 tags migrated to EffectFlags registry)
- [ ] Step 3: Bulk on_play migration (200 tags → 7 hooks files, 8 parallel agents)
- [ ] Fix 10 broken effect tags (Mind Blast, Nightmare, Blizzard, Thunder Strike, Scrape, Normality, Pride, Hand of Greed, Panic Button, Alchemize)
- [ ] Dead code cleanup (status_keys.rs, 7 dead branches, unused vars/imports)

### Blocked on this PR
- [ ] MCTS perf: static CardRegistry (OnceLock), relic bitset ([u64;3]), SmallVec
- [ ] Training integration: replace Python engine with Rust in workers (action layer TBD)

### Merge path to main
1. `feat/rust-engine` — engine polish + modular migration (current)
2. Training PR into `feat/rust-engine` — Rust worker swap + action layer redesign
3. Merge to `main` — replaces Python engine core

## Deferred (post-main-merge)
- [ ] Live combat viewer — N-up grid view of bots playing, real-time card-by-card replay
- [ ] Analytics dashboard — win rate by enemy, time-in-phase, card frequency, solver utilization
- [ ] Auto-research system — interval-based audits + fixes
- [ ] Dashboard frontend — jackswitzer.com page consuming gist data
