# StSRLSolver: Slay the Spire RL Agent

AI agent for achieving >96% winrate on Slay the Spire Watcher at Ascension 20.

## Setup

### 1. Install Dependencies

```bash
# Clone this repo
git clone https://github.com/JackSwitzer/StSRLSolver.git
cd StSRLSolver

# Install spirecomm (game communication library)
git clone https://github.com/ForgottenArbiter/spirecomm.git

# Install Python dependencies
uv sync
```

### 2. Install Game Mods (Steam Workshop)

Subscribe to these mods on Steam Workshop:
- [ModTheSpire](https://steamcommunity.com/workshop/filedetails/?id=1605060445)
- [BaseMod](https://steamcommunity.com/workshop/filedetails/?id=1605833019)
- [StSLib](https://steamcommunity.com/workshop/filedetails/?id=1609158507)
- [CommunicationMod](https://steamcommunity.com/workshop/filedetails/?id=2131373661)

### 3. Configure CommunicationMod

Create/edit `~/Library/Preferences/ModTheSpire/CommunicationMod/config.properties`:

```properties
command=/path/to/StSRLSolver/run_agent.sh
```

### 4. Launch

```bash
./launch_game.sh
```

## Features

### Line Evaluation
Instead of raw card encodings, the agent pre-computes strategic features:
- **Kill probability** per enemy and for lethal lines
- **Damage predictions** accounting for stance math (Wrath = 2x)
- **Block efficiency** and incoming damage calculations
- **Deck cycling** awareness

### Self-Play Training
```bash
# Switch to self-play mode
./switch_mode.sh selfplay

# Run with sleep prevention (for long training)
./run_vacation_selfplay.sh

# Monitor progress
./monitor_selfplay.sh
```

### Models
- `models/line_evaluator.py` - Simulates card play sequences
- `models/strategic_features.py` - 44 high-level strategic features
- `models/combat_calculator.py` - Combat math pre-computation
- `models/enemy_database.py` - Enemy difficulty ratings

## Project Structure

```
StSRLSolver/
├── agent.py                 # Main agent entry point
├── watcher_agent.py         # Priority-based Watcher logic
├── self_play_trainer.py     # Self-play learning loop
├── models/
│   ├── line_evaluator.py    # Combat sequence simulation
│   ├── strategic_features.py # Feature extraction
│   ├── combat_calculator.py # Combat math
│   ├── enemy_database.py    # Enemy info
│   └── bc_model.py          # Neural network
├── train_bc.py              # Behavioral cloning training
├── launch_game.sh           # Game launcher
├── run_agent.sh             # Agent wrapper
├── run_selfplay.sh          # Self-play wrapper
└── switch_mode.sh           # Toggle agent/selfplay mode
```

## Watcher Strategy

The agent manages Watcher's stance system:
- **Wrath**: 2x damage dealt and taken - use for lethal turns
- **Calm**: +2 energy on exit - use for setup
- **Divinity**: 3x damage - rare, powerful state

Key cards prioritized: Rushdown, Tantrum, Ragnarok, MentalFortress, TalkToTheHand

## Data

Training data from the [77M official run dataset](https://github.com/alexdriedger/SlayTheSpireData) (2018-2020).

## References

- [CommunicationMod](https://github.com/ForgottenArbiter/CommunicationMod) - Game communication protocol
- [spirecomm](https://github.com/ForgottenArbiter/spirecomm) - Python wrapper
- [Is Every Seed Winnable?](https://forgottenarbiter.github.io/Is-Every-Seed-Winnable/) - ForgottenArbiter's analysis
