# Relics Domain Audit

## Status
- Many behavior fixes are in place (eggs, chest counters, rest-site interactions, on-use triggers).
- `REL-003`, `REL-004`, and `REL-008` are complete: Orrery/Bottled/Dolly now use explicit follow-up selection actions in shop/reward action flow.
- Highest-priority remaining work is deterministic selection ID hardening and alias/inventory closure.

## Confirmed open gaps
- [ ] `REL-005` deterministic selection ID/validation consistency across equivalent snapshots.
- [ ] `REL-006` alias normalization and Java inventory closure (`Toolbox` confirmed open).
- [ ] `REL-007` residual ordering edge cases in reward/chest transitions.

## Completed in this batch
- [x] `REL-003` Orrery explicit selection actions.
  - Shop/reward acquisition returns `requires_selection` + `candidate_actions`.
  - Follow-up `select_cards` roundtrip applies one choice per generated offer bundle.
  - Runtime relic application consumes selected indices through `selection_card_indices`.
  - Tests: `tests/test_agent_api.py` Orrery selection flow tests.
- [x] `REL-004` Bottled relic explicit selection actions.
  - Shop/reward acquisition returns `requires_selection` + candidate card picks.
  - Follow-up `select_cards` roundtrip assigns bottled card IDs from selected deck indices.
  - Runtime relic application consumes selected indices through `selection_card_indices`.
  - Tests: `tests/test_agent_api.py` bottled selection flow tests.
- [x] `REL-008` Dolly's Mirror explicit selection actions.
  - Shop/reward acquisition returns `requires_selection` + deck-card candidate picks.
  - Follow-up `select_cards` roundtrip duplicates selected card with upgrade/misc preservation.
  - Runtime relic application consumes selected indices through `selection_card_indices`.
  - Tests: `tests/test_agent_api.py` Dolly selection flow tests.

## Java references
- `com/megacrit/cardcrawl/relics/Orrery.java`
- `com/megacrit/cardcrawl/relics/BottledFlame.java`
- `com/megacrit/cardcrawl/relics/BottledLightning.java`
- `com/megacrit/cardcrawl/relics/BottledTornado.java`
- `com/megacrit/cardcrawl/relics/DollysMirror.java`
- `com/megacrit/cardcrawl/relics/Toolbox.java`

## Python implementation touchpoints
- `packages/engine/state/run.py` (`_on_relic_obtained`, `add_relic`)
- `packages/engine/game.py` (pending selection context, action dict interception)
- `packages/engine/registry/relics.py` (battle triggers and orb-linked TODOs)

## Next commit order
1. `REL-005`
2. `REL-006`
3. `REL-007`
