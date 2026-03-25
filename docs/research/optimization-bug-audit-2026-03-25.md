# Optimization + Bug/Training Regularity Audit (2026-03-25)

## Scope reviewed
- Training orchestration and game collection loop safety.
- Combat action-selection quality safeguards.
- Remaining parity work units (events/powers/relics).

## Key findings

1. **Collect phase budget overshoot risk remained in the inner loop.**
   - Outer sweep constraints were enforced, but collect sub-loop could continue a phase after limits were reached mid-batch.
   - Fix applied: collect loop now checks `sweep_games < n_games` and `total_games < max_games`, and per-result guard stops recording once either cap is reached.

2. **Combat safety-net behavior did not match code comments.**
   - `_pick_combat_action()` promised to avoid immediate `end_turn` when playable cards exist, but previously trusted solver `end_turn` directly.
   - Fix applied: if solver returns `end_turn` and any `play_card` action is legal, a card play is selected instead.
   - Added focused tests for both the override and non-override cases.

3. **Remaining parity work units still show large event/power tail.**
   - `docs/TODO.md` currently tracks:
     - Events: 6/55 checked
     - Powers: 50/59 checked
     - Relics: 55/69 checked
   - Recommendation: next parity PR should prioritize event mechanics first (largest open surface), then power tail.

## Suggested next PRs

1. **Runner hardening follow-up**
   - Non-blocking collect result polling and faster signal responsiveness.
   - Worker slot lifecycle hardening for replacement workers.

2. **Parity closure sprint**
   - Complete event audit items from `granular-events.md`.
   - Close the remaining powers tail from `granular-powers.md`.
   - Add corresponding deterministic parity tests per item.
