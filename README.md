# Slay the Spire RL

Reinforcement learning bot for Slay the Spire targeting **Watcher A20 with >96% winrate**.

## Quick Start

```bash
# Install dependencies
uv sync

# Run parity tests (compare Python predictions vs game)
uv run pytest tests/test_parity.py

# Run the full test suite
uv run pytest tests/ -q
```

## Project Structure

```
packages/engine/    # Python game engine (source of truth)
  state/            # RNG system, game state tracking
  content/          # Cards, relics, potions, enemies, powers, stances
  generation/       # Map, encounters, shops, treasures, rewards
  calc/             # Damage/block calculations, CombatSimulator
  game.py           # GameRunner for full run simulation

packages/parity/    # Seed catalog + parity verification tools
tests/              # pytest suite (4100+ tests)
docs/               # Architecture docs + vault mechanics
scripts/            # Utility scripts
```

## Key Features

### RNG Parity
100% parity with Java game for:
- **Encounters**: All monster spawns across all acts
- **Bosses**: Boss selection and ordering
- **Elites**: Elite encounter selection
- **Map Generation**: Full act map recreation
- **Shops**: Shop inventory prediction
- **Card Rewards**: Card pool and selection
- **Potions**: Drop chance and selection
- **Relics**: Pool shuffling and tier rolls

### Save File Integration
- Decrypts and parses STS save files
- Live monitoring for state changes
- Supports all character classes

### Predictions
- Boss relic offerings
- Upcoming encounters per floor
- Card reward pools
- Shop inventory
- Treasure chest contents

### GameRunner
Full run simulation supporting:
- Neow options and rewards
- Path selection
- Combat simulation
- Event resolution
- Shop interactions
- Rest site decisions

## Status

### Working
- RNG system (XorShift128, all 13 streams)
- Map generation (Acts 1-4)
- Encounter prediction (all acts)
- Boss/elite selection
- Shop inventory generation
- Card reward prediction
- Potion drop calculation
- Relic pool management
- Save file decryption
- Web dashboard with live updates

### WIP
- Combat simulator (basic framework done, AI behaviors incomplete)
- Full enemy AI implementation
- Event resolution logic
- RL training pipeline
- Card effect implementation

### Documentation
- [Implementation Spec](docs/IMPLEMENTATION_SPEC.md) - What is implemented vs missing for a full Python clone

## Architecture Notes

### RNG Streams (13 total)
**Persistent** (survive entire run):
- cardRng, monsterRng, eventRng, relicRng, treasureRng, potionRng, merchantRng

**Per-Floor** (reset with seed+floorNum):
- monsterHpRng, aiRng, shuffleRng, cardRandomRng, miscRng

**Special**:
- mapRng (reseeded per act)

### Key Discovery: Act Transition Snapping
cardRng counter snaps at act transitions:
- 1-249 -> 250
- 251-499 -> 500
- 501-749 -> 750

## Development

```bash
# Run tests
uv run pytest tests/

# Watch game state changes
./scripts/dev/watch.sh

# Read current save
./scripts/dev/save.sh

# Launch game with mods
./scripts/dev/launch_sts.sh
```

## References

- [StSRLSolver](https://github.com/JackSwitzer/StSRLSolver) - Existing RL solver
- [CommunicationMod](https://github.com/ForgottenArbiter/CommunicationMod) - Bot communication protocol
- [bottled_ai](https://github.com/xaved88/bottled_ai) - 52% Watcher A0 baseline
