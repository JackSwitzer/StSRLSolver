# Slay the Spire RL

Reinforcement learning bot for Slay the Spire targeting **Watcher A20 with >96% winrate**.

## Quick Start

```bash
# Install dependencies
uv sync

# Run parity tests (compare Python predictions vs game)
./scripts/dev/parity.sh

# Launch web dashboard (STS Oracle)
uv run python web/server.py
# Open http://localhost:8080

# Test engine with specific seed
./scripts/dev/engine.sh 1234567890
```

## Project Structure

```
core/               # Python game engine
  state/            # RNG system, game state tracking
  content/          # Cards, relics, potions, enemies, powers, stances
  generation/       # Map, encounters, shops, treasures, rewards
  calc/             # Damage/block calculations, CombatSimulator
  game.py           # GameRunner for full run simulation

web/                # Live dashboard (STS Oracle)
  server.py         # FastAPI server with SSE for real-time updates
  index.html        # Single-page dashboard

tools/              # Utilities
  game_viewer.py    # Interactive game state viewer
  launcher.py       # Game launcher with mod support
  test_engine.py    # Engine testing tool

scripts/dev/        # Development scripts
  parity.sh         # Run parity tests
  engine.sh         # Test engine with seed
  save.sh           # Read current save file
  launch_sts.sh     # Launch game with mods

vod/                # VOD extraction tools
  run_extraction.py # Extract game data from VODs
  orchestrator.py   # Multi-step extraction pipeline
  verification.py   # RNG verification against extracted data

assets/             # Extracted game assets (cards, relics, enemies, UI)
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
