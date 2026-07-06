# AGENTS.md — Read This First

Clean-room **Rust simulator of Slay the Spire** (`packages/engine-rs`) for RL training, verified per-action against the real game. The active mission is **Watcher full-run parity**.

## Start here

1. `docs/goal/GOAL.md` — end state, invariants, edge-case (quarantine) policy. **Binding.**
2. `docs/goal/UNITS.md` — the work queue; pick the next `ready` unit.
3. `docs/goal/TOOLING.md` — oracle pipeline contracts. `docs/goal/INVENTORY.md` — what already exists (check before building anything).

## Verify (run these, in this order, before claiming success)

```bash
./scripts/test_engine_rs.sh test --lib            # 2219+ tests, must stay green
./scripts/trace_diff.sh data/traces/scripts/<relevant>.json   # parity oracle (offline)
scripts/goal.sh status                            # ledger/units dashboard (once U07 lands)
uv run pytest tests/training -q                   # if anything near training was touched (it shouldn't be)
```

## Hard rules (violations = bounced work)

- **Never modify**: `data/traces/java/`, `decompiled/`, `packages/training/`, `logs/`, `runs/`. Never `rm -rf` — archive to `archive/` instead.
- **Never launch the game** (`ModTheSpire`, `trace_java.sh`) — golden minting is human-attended.
- Tests only go green; no deleting/skipping tests to pass. Diff masks require a `DEV-NNN` register entry (`docs/goal/GOAL.md` § Edge-Case Policy).
- Stay in the unit's stated file scope. Cite the `decompiled/java-src/...` file for every ported behavior.
- Tooling: bash in `scripts/` (+ `jq`), Rust in `packages/engine-rs`. Python via `uv`, JS via `bun`. No new Python infra files. No docs beyond the goal/register files unless asked.
- Branch `codex/uNN-<slug>` per unit, stacked in dependency order, PR per unit, conventional commit messages prefixed `uNN:`. Never commit to `main`.

## Context

Java reference: `decompiled/java-src/com/megacrit/cardcrawl/` (grep it liberally). Mechanics notes: `docs/vault/`. Known gaps: `docs/work_units/comprehensive-audit-2026-04-17.md`. Deviations: `docs/work_units/parity-deviations-register.md`.
