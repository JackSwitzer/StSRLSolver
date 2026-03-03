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
- [x] All card effects that require choices/targets must emit explicit action options. (action: play_card{card_index,target_index})
  - Note: handled by two-step action mechanism (PlayCard with card_idx + target_idx, plus PendingSelectionContext for multi-step actions)

## Checklist
- [x] Implement `InnerPeace` effect key `if_calm_draw_else_calm` in `effects/executor.py`. (action: play_card{card_index})
- [x] Add effect implementation in `effects/cards.py`: (action: none{})
  - If in Calm: draw X (base 3, upgrade +1).
  - Else: enter Calm stance.
- [x] Ensure effect is referenced in `content/cards.py` (Watcher `InnerPeace`). (action: none{})
- [x] Add tests for both branches (already Calm vs not Calm) in `tests/test_cards.py` or a focused new test. (action: none{})
- [x] Verify stance change triggers (`onChangeStance`) fire correctly after Calm entry. (action: none{})

### `CRD-WA-001` closure notes
- Java refs: `cards/purple/InnerPeace.java`, `actions/watcher/InnerPeaceAction.java`.
- Python implementation: `effects/cards.py` has `@effect_simple("if_calm_draw_else_calm")` handler; `content/cards.py` card definition matches Java exactly (cost=1, base_magic=3, upgrade_magic=1, SKILL, UNCOMMON, SELF).
- Executor dispatch: `executor.py` `_EFFECT_HANDLERS` has redundant entry (dead code path since registry handler resolves first).
- Stance triggers verified: `change_stance("Calm")` fires Mental Fortress block, Flurry of Blows, and relic `onChangeStance` hooks.
- Tests: `test_watcher_card_effects.py` has 11 InnerPeace tests covering both branches (Calm draw 3/4, non-Calm enters Calm), stance trigger integration (Mental Fortress, Flurry of Blows), and full EffectExecutor pipeline.
