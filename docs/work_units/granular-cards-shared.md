# Ultra-Granular Work Units: Cards (Shared Colorless/Curse/Status)

## Current parity source
- Manifest source: `docs/audits/2026-02-22-full-game-parity/domains/cards-manifest-non-defect.md`
- Gap row: `GAP-CRD-INV-001`

## Feature slice: `CRD-SH-001`
- [x] Add engine-runtime handlers for curse/status end-of-turn effects:
  - `end_of_turn_take_damage` (Burn)
  - `end_of_turn_take_2_damage` (Decay)
  - `end_of_turn_gain_weak_1` (Doubt)
  - `end_of_turn_gain_frail_1` (Shame)
  - `end_of_turn_lose_hp_equal_to_hand_size` (Regret)
  - `end_of_turn_add_copy_to_draw` (Pride)
- [x] Ensure end-of-turn hand-card effects execute before hand discard/exhaust in combat turn flow.
- [x] Add effect handler for `lose_1_energy_when_drawn` and lock with draw-path tests.
- [x] Add targeted tests in `tests/test_effects_and_combat.py` covering the full runtime path.
- [x] Update manifest row statuses and unresolved-handler counts after implementation.
