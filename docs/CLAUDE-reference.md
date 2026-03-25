# Slay the Spire — Reference Material

Detailed game mechanics, engine API, and parity data. See `CLAUDE.md` for rules and workflow.

## Engine API
```python
from packages.engine import GameRunner, GamePhase

runner = GameRunner(seed="SEED", ascension=20)
while not runner.game_over:
    actions = runner.get_available_action_dicts()
    runner.take_action_dict(actions[0])
```

## Java Parity (100% on core mechanics)
| Domain | Status | Notes |
|--------|--------|-------|
| RNG System | 100% | All 13 streams verified |
| Damage/Block | 100% | Vuln 1.5x, Weak 0.75x, floor ops exact |
| Stances | 100% | Wrath/Calm/Divinity/Neutral |
| Enemies | 100% | All 66 verified |
| Map/Shop/Rewards | 100% | Java quirks included |

Remaining gaps (~10 LOW items): `docs/remaining-work-scoped.md`

## Engine Registry Pattern
```python
@relic_trigger("atBattleStart", relic="Vajra")
def vajra_start(ctx): ctx.apply_power_to_player("Strength", 1)

@power_trigger("atDamageGive", power="Strength")
def strength_give(ctx): return ctx.trigger_data.get("value", 0) + ctx.amount
```
Hooks match Java: `atBattleStart`, `onPlayCard`, `wasHPLost`, `atDamageGive`, etc.

## Watcher Mechanics
**Stances**: Wrath (2x out/in), Calm (+2 energy exit), Divinity (3x out, +3 energy)
**Key Cards**: Rushdown, Tantrum, MentalFortress, TalkToTheHand, InnerPeace, CutThroughFate
**Energy**: Base 3, Calm exit +2, Violet Lotus +3, Divinity +3, Deva Form +1/turn
**Scry**: Look top X, choose which to discard. Nirvana: block per scry action.

## RNG System (13 streams)
**Persistent**: cardRng, monsterRng, eventRng, relicRng, treasureRng, potionRng, merchantRng
**Per-Floor**: monsterHpRng, aiRng, shuffleRng, cardRandomRng, miscRng
**Special**: mapRng (reseeded per act), NeowEvent.rng

**Act transition cardRng snapping**: counter 1-249→250, 251-499→500, 501-749→750
**Details**: `docs/vault/rng-system-analysis.md`

## Resource Model
| Resource | Type | Notes |
|----------|------|-------|
| HP | Fungible | Most important. Rest, events, Reaper |
| Energy | Per-turn | 3 base + stance + relics |
| Potions | Consumable | Limited slots |
| Gold | Persistent | Shops, events, removal |
| Deck | Strategic | Composition defines strategy space |

## Reference Projects
- [StSRLSolver](https://github.com/JackSwitzer/StSRLSolver) — Original RL solver
- [bottled_ai](https://github.com/xaved88/bottled_ai) — 52% Watcher A0 (graph traversal)
- [CommunicationMod](https://github.com/ForgottenArbiter/CommunicationMod) — Bot protocol

## EV Framework
Track all decisions: card plays, picks, paths, rest, shop, events, potions.
`EV(decision) = P(win | decision) - P(win | baseline)`

## Vault Docs
- `docs/vault/damage-mechanics.md` — Damage/block formulas
- `docs/vault/rng-system-analysis.md` — Complete RNG analysis
- `docs/vault/modding-infrastructure.md` — Java mod patching
- `docs/vault/direct-launch.md` — Running without Steam
