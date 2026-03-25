# Optimization + Bug/Training Regularity Audit (2026-03-25)

## Scope reviewed
- Training orchestration and game collection loop safety.
- Combat action-selection quality safeguards.
- Remaining parity notes are doc-only follow-up items, not a live completion counter.

## Key findings

1. **Collect phase overshoot is now fixed in the umbrella branch.**
   - The inner collect loop now checks `sweep_games < n_games` and `total_games < max_games`, and the per-result guard stops recording once either cap is reached.

2. **Combat safety-net behavior now matches the comment in the umbrella branch.**
   - `_pick_combat_action()` now prefers a legal `play_card` if the solver returns `end_turn` while playable cards exist.
   - The focused regression test is carried forward with the fix.

3. **Parity docs need a rebaseline.**
   - `docs/TODO.md` and the granular parity docs still read like a broad open-gap audit.
   - They should be treated as legacy references for a smaller, targeted remaining tail.

## Suggested next PRs

1. **Runner hardening follow-up**
   - Non-blocking collect result polling and faster signal responsiveness.
   - Worker slot lifecycle hardening for replacement workers.

2. **Parity closure sprint**
   - Rebaseline the parity docs so they stop overstating the remaining tail.
   - Promote only the truly remaining parity items into code work.
