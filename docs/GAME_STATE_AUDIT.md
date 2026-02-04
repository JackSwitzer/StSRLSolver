# Game State Audit (Compressed)

This document is a concise audit of game-state coverage: what is implemented, what is tracked, and what still needs work.

## Current State Coverage

### Run-Level State (`RunState`)
Tracked and persisted (via `to_dict`/`from_dict`):
- Seed + ascension + act/floor + character.
- Resources: HP/max HP, gold.
- Deck with per-card instances (`CardInstance`) including upgrades and Searing Blow misc value.
- Relics with counters and per-combat/turn flags (`RelicInstance`).
- Potion slots.
- Map state: generated act maps, current position, visited nodes.
- Reward pool tracking: seen cards/relics, cards obtained this act.
- Act 4 keys.
- RNG counters for save/load determinism.
- Pity timers (card rarity blizzard, potion blizzard).
- Progress stats (floors climbed, combats won, elites/bosses killed, perfect floors).
- Shop purge count, Neow’s Lament, Question Card charges.

Present but partial:
- Relic “on obtain” effects include some HP/potion-slot adjustments, but not all relic side effects are handled (`VioletLotus` is a no-op).
- All four characters have run factories (Watcher/Ironclad/Silent/Defect), but cross-class mechanics are still partial.

### Combat-Level State (`CombatState`)
Tracked:
- Player `EntityState` (HP, block, statuses), energy, max energy, stance.
- Enemy combat states with intent and move history.
- Card piles (hand/draw/discard/exhaust), card cost overrides.
- Potions (combat slots as IDs).
- Turn tracking, card play counts, combat stats.
- Relic counters and relic list for combat checks.
- RNG state snapshots (shuffle/card/ai) for deterministic sim.

Present but partial:
- No orb state/slots or per-orb tracking (Defect system missing).
- Several combat-time relic hooks are still no-ops (see `handlers/combat.py`).

### RNG State (`GameRNGState`, `Random`)
Implemented:
- XorShift128 RNG parity with Java.
- 13 RNG streams, counters, and per-floor reseeding.
- Act transition cardRng snapping.
- Neow cardRng consumption table.

Note:
- Run state `rng_counters` are now synced after RNG-consuming phases (events, shops, rewards, treasure, act transitions).

### Phase/Flow State (`GameRunner`)
Implemented:
- Full phase machine: map navigation, combat, rewards, events, shops, rest, treasure, boss rewards, Neow.
- JSON action layer + observation API for RL integration.

Present but partial:
- `GameRunner` supports Watcher/Ironclad/Silent/Defect run factories.
- Some phase outcomes still depend on missing relic/potion/event behaviors.

### Room/Modal State
Implemented dataclasses:
- `ShopState` (inventory, purge cost, sale card, purchased flags).
- `EventState` (multi-phase tracking, pending rewards).
- Reward types (`CombatRewards`, `CardReward`, etc.).

Present but partial:
- Reward action processing is implemented, but fidelity depends on missing relic/potion/event hooks.
- Event choice generators and handler coverage are incomplete; ID normalization now handled via alias mapping.

## What’s Missing / Needs Work

### Core State Gaps
- **Orb system**: channel/evoke, slots, Focus/Lock‑On, orb-specific state in `CombatState`.
- **Event normalization**: alias mapping added; full content/handler unification and missing choice generators remain.

### Combat/State Hooks
- Combat‑time relic hooks are incomplete (`Snecko Eye`, Ice Cream, etc.).
- Potion effect behaviors are still simplified (Discovery choices, Distilled Chaos auto-play, Entropic Brew RNG parity, Smoke Bomb restrictions).
- Power triggers missing across multiple hooks (see `docs/work_units/granular-powers.md`).

### Data/Pool Consistency
- Legacy Java IDs remain canonical, but modern names are now supported via alias mapping (Rushdown → `Adaptation`, Foresight → `Wireheading`, Wraith Form → `Wraith Form v2`).

### Tests and Known Failures (latest known)
- Skip markers indicate **~138 skipped tests**, mostly relic pickup/rest-site/chest mechanics.
- Targeted readiness tests for JSON actions + observations now pass.

### Watcher RL Readiness (current max)
Safe (high parity):
- RNG streams/counters + cardRng snapping.
- Damage/block calc.
- Enemy AI patterns (all 66 verified).
- Watcher stance system (Neutral/Calm/Wrath/Divinity).

Cautious (partial parity; usable with constraints):
- Potions: several effects simplified (discovery, Distilled Chaos, Liquid Memories, Entropic Brew, Smoke Bomb).
- Powers: many missing triggers; expect combat value drift.
- Relic hooks: many combat-time triggers are no-ops.
- Events: missing choice generators default to leave; pool coverage incomplete.

Risky (training fidelity compromised):
- Defect orb system missing (avoid Prismatic Shard/cross-class reliance).
- Some relic/potion/event behavior gaps can bias learning if not constrained.

Practical constraints to “push max” now:
- Prefer Watcher-only runs; avoid Prismatic Shard and cross-class effects.
- Treat event rooms as low-fidelity (either skip events in training or accept leave-only bias).
- Consider disabling/avoiding potions with missing behavior and relics with missing hooks in training configs.

## Compressed “State Progress” Checklist

Implemented:
- Deterministic RNG + stream counters.
- Run-level persistence + map state.
- Combat state with piles/stance/energy/enemy intent.
- Shop, event, and reward data structures.

Missing / Partial:
- Orb system and Defect state.
- Event ID normalization and missing choice generators.
- Combat relic/potion hooks and power trigger coverage.

## References

- State definitions: `packages/engine/state/run.py`, `packages/engine/state/combat.py`, `packages/engine/state/game_rng.py`, `packages/engine/state/rng.py`
- Game flow: `packages/engine/game.py`, `packages/engine/combat_engine.py`
- Modal state: `packages/engine/handlers/rooms.py`, `packages/engine/handlers/shop_handler.py`, `packages/engine/handlers/event_handler.py`, `packages/engine/handlers/reward_handler.py`
- Unit-sized task breakdowns: `docs/work_units/`
