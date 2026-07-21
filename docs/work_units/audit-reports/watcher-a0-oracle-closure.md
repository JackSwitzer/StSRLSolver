# Watcher A0 Oracle Closure Scorecard

**Branch:** `codex/watcher-a0-oracle-closure`

**Authority:** decompiled Java for behavior; human-minted Java traces for
end-to-end certification

## Decision Dashboard

| Gate | Result | Evidence |
| --- | ---: | --- |
| Native Rust simulator regression | 3,016 pass, 0 fail, 0 ignore | `./scripts/test_engine_rs.sh test --lib` |
| All Rust targets | green | `./scripts/test_engine_rs.sh check --all-targets` |
| Canonical run actions | 27/27 serialized and documented | `test_trace_schema_v2` |
| Deterministic v2 replay | green; header + 4 transitions + end | `smoke-v2-neow-floor1.json` |
| V1 differ field coverage | complete for every serialized field | one-field mutation tests in `test_trace_oracle` |
| Generic production RNG | zero matches | `rand::`, `RngCore`, `SliceRandom` source scan |
| Committed real-game corpus | 1/11 target goldens, about 9% | protected Java trace directory and B3 target |

## Completion Read

The stack remains approximately **95% complete against currently known native
simulator implementation work**. It is only **about 9% certified by real-game
full-run corpus volume**. The first percentage is a source/test implementation
estimate; the second is an integration oracle measurement. They must never be
combined into a claim that the simulator is 95% Java-certified.

## Fixed And Proven

| Item | Status | Proof |
| --- | --- | --- |
| Full v1 differ surface | FIXED-PROVEN | Record identity/action, complete nested post-state, and RNG-first ordering have corruption tests. |
| Canonical v2 action script | FIXED-PROVEN | Direct `GameAction` serialization, seed identity validation, illegal-action rejection, causal chaining, and deterministic replay pass. |
| V2 CLI replay | FIXED-PROVEN | `trace_replay --script ... --out ...` emits six valid JSONL envelopes for the smoke fixture. |
| Schema/document synchronization | FIXED-PROVEN | A test fails if any serialized `GameAction` variant is absent from `script-schema-v2.md`. |
| False Java-certification guard | FIXED-PROVEN | V2 CLI rejects Java diff flags until a language-neutral projection exists. |

## Remaining Merge Gates

1. Implement the v2 canonical-action adapter in the Java recorder. The Java
   harness is intentionally untouched in this engine stack.
2. Freeze a language-neutral complete oracle-state projection. Rust
   `CoreCheckpoint` remains the correct causal restore artifact but cannot be
   treated as a Java-emittable wire DTO.
3. Have a human select real Watcher history seeds, validate complete scripts,
   and A/B mint roughly ten A0 runs plus seed `1776347657`.
4. Replay every accepted golden offline and fix the earliest unmasked
   Java/Rust divergence until the complete corpus matches.
5. Record all four generated Neow option payloads and the selected payload so
   the intentional four-choice policy is compared semantically, not by gated
   screen index.

## Stacked PR State

| Layer | Branch | State |
| --- | --- | --- |
| Native Java RNG | `codex/rng-core-java` | published draft |
| Run generation | `codex/run-generation-parity` | published draft |
| Canonical actions | `codex/full-run-actions-v2` | published draft |
| Causal checkpoints | `codex/core-checkpoint-normalization` | published draft |
| Pure simulation boundary | `codex/pure-sim-freeze` | published draft |
| Oracle closure | `codex/watcher-a0-oracle-closure` | engine-side complete; human/Java gates open |

## Verdict

The Rust branch is a clean, deterministic base for a separately layered
training rebuild. The complete parity stack must remain draft: no ignored Rust
test or known in-repo trace-field omission remains, but the real-game corpus is
far too small to certify Neow-to-Heart behavior and the Java v2 boundary has not
yet been implemented or minted.
