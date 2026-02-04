# Ultra-Granular Work Units: Cards (Watcher)

## Model-facing actions (no UI)
- [ ] All card effects that require choices/targets must emit explicit action options. (action: play_card{card_index,target_index})

## Checklist
- [ ] Implement `InnerPeace` effect key `if_calm_draw_else_calm` in `effects/executor.py`. (action: play_card{card_index})
- [ ] Add effect implementation in `effects/cards.py`: (action: none{})
  - If in Calm: draw X (base 3, upgrade +1).
  - Else: enter Calm stance.
- [ ] Ensure effect is referenced in `content/cards.py` (Watcher `InnerPeace`). (action: none{})
- [ ] Add tests for both branches (already Calm vs not Calm) in `tests/test_cards.py` or a focused new test. (action: none{})
- [ ] Verify stance change triggers (`onChangeStance`) fire correctly after Calm entry. (action: none{})
