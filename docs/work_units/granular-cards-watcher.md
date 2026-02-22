# Ultra-Granular Work Units: Cards (Watcher)

## Current parity source
- Non-Defect manifest row source: `docs/audits/2026-02-22-full-game-parity/domains/cards-manifest-non-defect.md` (`purple` section)
- Phase target: close Watcher rows from `approximate` to Java-audited `exact`

## Inventory closure slice (`CRD-INV-002`)
- [x] Add missing Java-ID card `Discipline` (Watcher) with power hook coverage and tests.

### `CRD-INV-002` closure notes
- Java refs: `cards/purple/Discipline.java`, `powers/deprecated/DEPRECATEDDisciplinePower.java`.
- Python implementation: `content/cards.py` adds `Discipline`; `effects/cards.py` adds `apply_discipline_power`; `registry/powers.py` adds `DisciplinePower` hooks at `atEndOfTurn` and `atStartOfTurn`.
- Tests: `tests/test_cards.py` (`Discipline` stats), `tests/test_power_registry_integration.py` (registration + behavior).

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
