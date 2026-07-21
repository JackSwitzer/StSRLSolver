# Slay the Spire RL — Agent Orientation

Read `AGENTS.md` first. Its source-of-truth, protected-path, testing, and branch
rules are binding.

## Current state

- `packages/engine-rs/` is the canonical simulator.
- The 667-row card/monster/relic/potion source-verification sweep is complete.
- Content verification is not full systems parity. Consult
  `docs/work_units/audit-reports/engine-deep-audit.md`,
  `docs/work_units/sim-completion-map.md`, and `docs/goal/FINDINGS.md` before
  choosing work.
- Training, app, and visualization consumers are being rebuilt. Do not restore or
  advertise their superseded launch flows.

## Working rules

1. Derive expected behavior from `reference/extracted/` or the full local
   `decompiled/java-src/` tree, never from existing Rust alone.
2. Preserve exact action ordering and RNG consumption.
3. Add a source-cited test for every behavior change.
4. Keep changes focused, preserve unrelated worktree edits, and do not touch the
   protected paths listed in `AGENTS.md`.
5. Run the full library suite for engine commits:

```bash
./scripts/test_engine_rs.sh test --lib
```

Use `scripts/ledger.sh status` for content coverage and
`scripts/trace_diff.sh <script.json>` only when a committed golden covers the
scenario. Agents do not run `scripts/trace_java.sh` or otherwise launch the game.

## Canonical references

- `docs/goal/GOAL.md` — end state and invariants
- `docs/goal/UNITS.md` — infrastructure queue
- `docs/goal/TOOLING.md` — trace/oracle contract
- `docs/goal/BENCHMARKS.md` — benchmark ladder
- `docs/goal/FINDINGS.md` — known baseline gaps
- `docs/work_units/audit-reports/engine-deep-audit.md` — ranked audit findings
- `docs/work_units/sim-completion-map.md` — gap map and sequencing
