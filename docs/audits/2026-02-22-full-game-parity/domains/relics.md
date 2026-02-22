# Relics Domain Audit

## Status
- Many behavior fixes are in place (eggs, chest counters, rest-site interactions, on-use triggers).
- Highest-priority remaining work is selection-surface parity for acquisition-time relic decisions.

## Confirmed open gaps
- [ ] `REL-003` Orrery acquisition still auto-picks first card in each 3-card offer set.
- [ ] `REL-004` Bottled relic acquisition still defaults to first eligible card if no explicit selection.
- [ ] `REL-008` Dolly's Mirror acquisition still duplicates deck index 0 when unresolved.
- [ ] `REL-005` deterministic selection ID/validation consistency across equivalent snapshots.
- [ ] `REL-006` alias normalization and Java inventory closure (`Toolbox` confirmed open).
- [ ] `REL-007` residual ordering edge cases in reward/chest transitions.

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
1. `REL-003`
2. `REL-004`
3. `REL-008`
4. `REL-005`
5. `REL-006`
