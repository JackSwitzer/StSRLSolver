# Agent Loop

Use this repo-level loop when working on `packages/engine-rs` parity and cleanup:

1. Verify the integrated branch first.
   - Run:
     - `./scripts/test_engine_rs.sh check --lib`
     - `./scripts/test_engine_rs.sh test --lib --no-run`
   - Re-run the focused suites for any just-landed worker slices before crediting progress.

2. Recount the real tail before planning the next wave.
   - Track at minimum:
     - card files with empty `effect_data`
     - card files still using `complex_hook`
     - remaining `EventProgramOp::blocked(...)`
     - remaining relic helper/oracle references in `src/relics/mod.rs` and `src/tests/test_relics_parity.rs`

3. Convert every confirmed gap into one of two things only:
   - a landed engine-path test in the same wave
   - an explicit ignored/queued Java-cited blocker test if a primitive must land first

4. Prefer primitive-centered worker waves over broad content spam.
   - Current priority order:
     - runtime-complete relic cleanup and oracle retirement
     - card-owned state / X-count tails
     - generated-card / generated-choice tails
     - legality / fetch / follow-up sequencing tails
     - only then broad low-risk card ports

5. Keep the coordinator on audit, verification, and worker slicing.
   - Subagents own bounded write scopes plus their focused tests.
   - Do not credit a worker slice until the integrated branch passes its wrapper checks locally.

6. Keep the parity source of truth explicit.
   - Java oracle: `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src`
   - Scorecard: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/AUDIT_PARITY_STATUS.md`
   - Endgame map: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DECOMPILE_PARITY_ENDGAME.md`

7. Do not start broad parity sweeps until all three are true:
   - blocked event ops are zero
   - helper/oracle tests are no longer primary parity evidence
   - the remaining card tail is mostly documented blockers rather than unported easy files

8. Repeat the loop until the tail is exhausted.
   - verify
   - recount
   - spawn bounded waves
   - land engine-path tests
   - delete dead helpers
   - every worker must end with an explicit completion report listing changed files, verification commands/results, and any exact Java-backed blockers so the coordinator can react immediately
   - when a worker finishes, it should send that completion report right away instead of waiting for a follow-up prompt
   - after each accepted slice, immediately queue the next bounded wave unless the user explicitly pauses
   - immediately queue the next bounded wave after every accepted slice so the coordinator never goes idle
