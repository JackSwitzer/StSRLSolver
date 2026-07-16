# Slay the Spire RL

A clean-room Rust simulator of *Slay the Spire*, built for deterministic,
high-throughput reinforcement-learning workloads and checked against the
decompiled game.

The source-verification ledger currently records all 667 scoped cards, monsters,
relics, and potions as verified. Full simulation parity is broader than that
content sweep: run generation, RNG topology, events, observation contracts,
trace coverage, and batch throughput still have explicit gaps.

## Repository map

| Path | Purpose |
| --- | --- |
| `packages/engine-rs/` | Canonical Rust simulation engine and its tests |
| `reference/extracted/` | Generated, review-friendly Java source extracts |
| `decompiled/` | Local-only full Java decompilation (gitignored) |
| `data/traces/` | Scripted trace inputs and frozen human-minted goldens |
| `docs/goal/` | Goal contract, known findings, tooling, and ledger |
| `docs/work_units/audit-reports/engine-deep-audit.md` | Ranked post-sweep audit register |
| `docs/work_units/sim-completion-map.md` | Layer-by-layer remaining gap map |

The training, app, and visualization consumers are being rebuilt against the
audited engine boundary. Treat their old launch flows and documentation as
historical, not as supported entry points.

## Start here

Read [`AGENTS.md`](AGENTS.md) before changing engine behavior. It defines the
source-of-truth rules, protected paths, and verification workflow. The canonical
end-state contract is in [`docs/goal/GOAL.md`](docs/goal/GOAL.md), with established
gaps in [`docs/goal/FINDINGS.md`](docs/goal/FINDINGS.md).

```bash
scripts/extract.sh                       # refresh generated source extracts
./scripts/test_engine_rs.sh test --lib   # canonical engine test suite
scripts/ledger.sh status                 # content-verification ledger
```

Java traces are minted only by a human using `scripts/trace_java.sh`; engine work
uses committed goldens through `scripts/trace_diff.sh`.
