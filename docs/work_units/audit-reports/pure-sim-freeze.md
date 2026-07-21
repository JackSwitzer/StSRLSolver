# Pure Simulation Freeze Audit

**Snapshot:** `codex/pure-sim-freeze`, updated at stacked oracle-closure tip,
2026-07-19

**Authority:** decompiled Java for behavior; committed Java traces for integration parity

## Decision dashboard

| Gate | Current result | Evidence |
|---|---:|---|
| Source-verified content ledger | 667/667 verified | `scripts/ledger.sh status` |
| Rust core regression suite | 3,016 passed, 0 failed, 0 ignored | `./scripts/test_engine_rs.sh test --lib` |
| All Rust targets | Green | `./scripts/test_engine_rs.sh check --all-targets` |
| Generic gameplay RNG | Zero production uses | source scan for `rand`, `RngCore`, and `SliceRandom` |
| Legacy consumer semantics in core | Zero active references | source scan for PyO3, obs, search, training contract, and legacy action/result types |
| Trace scripts / Java goldens | 2 scripts / 1 golden | v1 and v2 smoke scripts; one protected v1 Java golden |
| Target full-run corpus | 1/11, about 9% | `docs/goal/GOAL.md` B3 target |

The implementation is approximately **95% complete for the currently known
Watcher A0 simulator work**, but only **9% corpus-certified** against the real
game. Those percentages measure different things and must not be blended into a
false “Java-clean” claim. The core is ready to serve as the base for a new
training extension; the stacked parity PR remains draft until oracle closure.

## What this freeze closed

The first five stack layers now provide native Java-exact RNGs and stream
ownership, seeded run generation and pools, one canonical `GameAction` and
training-neutral `StepOutcome`, complete causal checkpoints, and a pure Rust
simulation boundary. Rejected actions preserve causal state and RNG, and every
active test uses the canonical action surface.

The freeze archived and disconnected observation vectors, PUCT/search,
training-contract snapshots, gameplay sessions, and PyO3 wrappers under
`archive/2026-07-engine-consumers`. It also removed their dependencies and the
hidden shaped-reward path that still computed floor, damage, victory, and boss
bonuses after rewards had disappeared from the public result.

The prior suite had 3,058 tests. Fifty-one adapter/search/Python tests were
archived, while four unique game-rule assertions were retained as pure-core
tests: Potion Belt's fifth slot, Frozen Eye draw order, canonical next-decision
continuity, and blocked campfire legality. The resulting total is 3,011.

## Remaining parity work

### Merge-gating for oracle closure

1. The Java recorder needs the canonical v2 action adapter and a
   language-neutral full-state/RNG projection. Rust-private `CoreCheckpoint`
   serialization is continuation proof, not a cross-language oracle format.
2. The human workflow must remint a Watcher A0 trace pack after the action/RNG
   schema changes. Agents must not launch the game or modify protected goldens.
3. Every first divergence from the reminted corpus must be fixed or registered
   as a narrow approved `DEV-` exception. No current unit test can substitute
   for this end-to-end proof.
4. Neow intentionally exposes all four seeded choices on every run. Oracle
   scripts must record the selected option payload rather than assume an old
   progression-gated index mapping.

### Post-core training work

Observations, restrictions/curriculum, rewards, search, batching, logging, and
monitoring are consumer-owned. They should be rebuilt in a separate stacked PR
using `GameAction`, `StepOutcome`, and `CoreCheckpoint`; none should be restored
from the archive as a core dependency.

## Performance snapshot

On the audit host, `full_turn_cycle` measured 4.43-4.45 us,
`clone_for_mcts` 620-624 ns, and `get_legal_actions` 92-93 ns. These remain under
the existing scalar guardrails. Batch APIs, allocation accounting, and
thread-scaling are training-architecture work rather than parity claims.

## Readiness verdict

The branch is **pure-core ready**, **zero-skip regression ready**, and the Rust
v1 differ is complete over its serialized state. It is not
yet **full-run Java certified** or ready to merge the complete stack to `main`.
The only honest next parity layer is the Java v2 recorder/projection followed
by human-minted Watcher A0 oracle closure; training design may proceed in
parallel on top of this immutable core contract.
