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
   - Maintain `3` occupied worker slots whenever real work exists.
   - If one worker finishes while others are still running, immediately refill that empty slot with the next disjoint slice instead of waiting for the whole batch.

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
   - keep up to 5 occupied worker slots whenever real work exists:
     - at most 1 shared-runtime implementation worker touching `effects/**`, `engine.rs`, or `run.rs`
     - 1-2 isolated implementation workers on disjoint per-card/test scopes
     - remaining slots can be read-only audits or blocker-inventory passes
   - if all workers are idle or done, that is a notify-worthy loop gap and the next wave should be spawned right away

9. Commit cadence is part of the loop.
   - Do not wait for one giant end-of-cycle commit.
   - After every meaningful verified batch, make a checkpoint commit on the active branch.
   - Default cadence:
     - commit after every 1-3 accepted worker slices, or
     - commit immediately after any real primitive lands, any audited count drop, or any cleanup wave that materially reduces dead-system tail
   - Keep the draft PR updated as those checkpoint commits land so git activity is a visible progress signal, not just local state.
