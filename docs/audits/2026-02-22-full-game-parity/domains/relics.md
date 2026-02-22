# Relics Domain Audit

## Status
- Critical progress done for several pickup/rest/chest behaviors.
- Remaining P0 work is selection-surface completeness plus ID normalization.

## Confirmed gaps
- [ ] Orrery explicit card-pick actions (`REL-003`).
- [ ] Bottled relic assignment explicit actions (`REL-004`).
- [ ] Dolly's Mirror explicit duplicate selection (`REL-008`).
- [ ] Deterministic selection IDs/validation (`REL-005`).
- [ ] Alias and Java inventory normalization including `Toolbox` (`REL-006`).
- [ ] Remaining edge-order regressions (`REL-007`).

## Java reference focus
- `com/megacrit/cardcrawl/relics/Orrery.java`
- `com/megacrit/cardcrawl/relics/BottledFlame.java`
- `com/megacrit/cardcrawl/relics/BottledLightning.java`
- `com/megacrit/cardcrawl/relics/BottledTornado.java`
- `com/megacrit/cardcrawl/relics/DollysMirror.java`
- `com/megacrit/cardcrawl/relics/Toolbox.java`
