# Relics Domain Audit

## Status
- Many behavior fixes are in place (eggs, chest counters, rest-site interactions, on-use triggers).
- `REL-003` is complete: Orrery now uses explicit follow-up selection actions in shop/reward action flow.
- Highest-priority remaining work is the remaining acquisition-time selection relics.

## Confirmed open gaps
- [ ] `REL-004` Bottled relic acquisition still defaults to first eligible card if no explicit selection.
- [ ] `REL-008` Dolly's Mirror acquisition still duplicates deck index 0 when unresolved.
- [ ] `REL-005` deterministic selection ID/validation consistency across equivalent snapshots.
- [ ] `REL-006` alias normalization and Java inventory closure (`Toolbox` confirmed open).
- [ ] `REL-007` residual ordering edge cases in reward/chest transitions.

## Completed in this batch
- [x] `REL-003` Orrery explicit selection actions.
  - Shop/reward acquisition returns `requires_selection` + `candidate_actions`.
  - Follow-up `select_cards` roundtrip applies one choice per generated offer bundle.
  - Runtime relic application consumes selected indices through `selection_card_indices`.
  - Tests: `tests/test_agent_api.py` Orrery selection flow tests.

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
1. `REL-004`
2. `REL-008`
3. `REL-005`
4. `REL-006`
