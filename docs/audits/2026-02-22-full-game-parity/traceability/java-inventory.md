# Java Inventory

Track canonical Java sources for parity-critical systems.

| domain | java package path | inventory status | notes |
|---|---|---|---|
| potions | `com/megacrit/cardcrawl/helpers/PotionHelper.java` + potion classes | seed-complete | content ID set matched in prior pass |
| relics | `com/megacrit/cardcrawl/relics/*.java` | partial | `Toolbox` mismatch confirmed |
| events | `com/megacrit/cardcrawl/events/**` | partial | naming aliases and selection flow gaps remain |
| powers | `com/megacrit/cardcrawl/powers/**` | partial | significant class-level residuals |
| cards | `com/megacrit/cardcrawl/cards/**` | open | long-tail checklist region |
| rewards/rooms/shop | `rewards/**`, `rooms/**`, `shop/**` | partial | action-surface normalization pending |
| orbs | card/relic/power interactions across classes | open | infrastructure closure pending |

## Intake checklist
- [ ] Export event class inventory and map aliases to Python IDs.
- [ ] Export relic class inventory and resolve ID canonicalization rules.
- [ ] Export power class inventory and normalize alias mapping.
- [ ] Link each manifest row to exact Java class/method reference.
