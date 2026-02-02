# SlayTheSpireRL Tools and Scripts

This document covers all development tools, scripts, and utilities in the SlayTheSpireRL project.

---

## Table of Contents

- [Development Scripts](#development-scripts)
  - [parity.sh / test_parity.py](#paritysh--test_paritypy---parity-testing)
  - [engine.sh](#enginesh---interactive-engine-tester)
  - [launch_sts.sh](#launch_stssh---launch-slay-the-spire)
  - [read_save.py / save.sh](#read_savepy--savesh---save-file-reader)
- [GUI Tools](#gui-tools)
  - [game_viewer.py](#game_viewerpy---game-state-viewer)
  - [launcher.py](#launcherpy---simulation-launcher)
  - [test_engine.py](#test_enginepy---interactive-python-engine)
- [Web Dashboard](#web-dashboard)
  - [server.py](#serverpy---live-dashboard-server)
  - [index.html](#indexhtml---sts-oracle-ui)
- [When to Use Each Tool](#when-to-use-each-tool)

---

## Development Scripts

Location: `/Users/jackswitzer/Desktop/SlayTheSpireRL/scripts/dev/`

### parity.sh / test_parity.py - Parity Testing

**Purpose:** Compare Python RNG predictions against actual game data from save files. Validates that the Python implementation matches Java behavior.

**Usage:**

```bash
# Compare predictions with current save file
./scripts/dev/parity.sh

# Or directly with Python
uv run scripts/dev/test_parity.py

# Generate predictions for a specific seed (for manual comparison)
uv run scripts/dev/test_parity.py --seed "1234567890" --character WATCHER

# Test with different ascension
uv run scripts/dev/test_parity.py --seed "ABCDEF" --character WATCHER --ascension 20
```

**Example Output:**

```
============================================================
PARITY CHECK - SEED: 1234567890
Floor: 5 | Act: Exordium
Ascension: 20
============================================================

--- MONSTER LIST COMPARISON ---
Monsters fought so far: 2
#   Predicted                 Actual (remaining)        Match
-----------------------------------------------------------------
3   2 Louse                   2 Louse                   ✓
4   3 Slime                   3 Slime                   ✓
5   Cultist                   Cultist                   ✓

Monster Match Rate: 3/3 (100.0%)

--- ELITE LIST COMPARISON ---
Elites fought so far: 0
#   Predicted                 Actual (remaining)        Match
-----------------------------------------------------------------
1.  Lagavulin                 Lagavulin                 ✓

Elite Match Rate: 1/1 (100.0%)

--- BOSS COMPARISON ---
Predicted: Hexaghost
Actual:    Hexaghost
Match:     ✓

============================================================
OVERALL PARITY: 5/5 (100.0%)
STATUS: PERFECT PARITY ✓
============================================================
```

---

### engine.sh - Interactive Engine Tester

**Purpose:** Launches the Python game engine in interactive mode for side-by-side testing with the actual Java game.

**Usage:**

```bash
# Start with default seed
./scripts/dev/engine.sh

# Start with specific seed
./scripts/dev/engine.sh 1234567890
```

**Example Output:**

```
Starting Python Engine with seed: 1234567890
Compare with STS using the same seed.

============================================================
PYTHON ENGINE TESTER
============================================================
Seed (alpha): ABCDEF
Seed (numeric): 1234567890
Ascension: 20

Start STS with this seed to compare.
============================================================

ENCOUNTER PREDICTIONS
============================================================

Act 1:
  Monsters: Cultist, Jaw Worm, 2 Louse, Small Slimes, Gremlin Gang
  Elites: Lagavulin, 3 Sentries, Gremlin Nob
  Boss: Hexaghost
```

---

### launch_sts.sh - Launch Slay the Spire

**Purpose:** Launch Slay the Spire with ModTheSpire and CommunicationMod configured for development/testing.

**Usage:**

```bash
# Default: Launch with CommunicationMod
./scripts/dev/launch_sts.sh

# Launch vanilla (no mods)
./scripts/dev/launch_sts.sh --no-mods

# Custom mod list
./scripts/dev/launch_sts.sh --mods "basemod,stslib,evtracker"

# Show intro video
./scripts/dev/launch_sts.sh --no-skip-intro

# Custom memory allocation
./scripts/dev/launch_sts.sh --memory 2G

# Run in foreground (blocking, shows output)
./scripts/dev/launch_sts.sh --fg

# Show help
./scripts/dev/launch_sts.sh --help
```

**Options:**

| Option | Description |
|--------|-------------|
| `--no-mods`, `--vanilla` | Launch without mods |
| `--mods "mod1,mod2"` | Custom mod list (default: basemod,CommunicationMod) |
| `--no-skip-intro` | Show intro video |
| `--no-skip-launcher` | Show ModTheSpire launcher GUI |
| `--memory SIZE`, `-m` | Memory allocation (default: 1G) |
| `--fg`, `--foreground` | Run in foreground (blocking) |

**Example Output:**

```
=== Launching Slay the Spire (Modded) ===
Mods: basemod,CommunicationMod
Memory: 1G
Skip Intro: true
Skip Launcher: true

Launching in background...
Log: /Users/.../SlayTheSpireRL/logs/sts_20240128_123456.log

Launched! (PID: 12345)

Monitor with: tail -f "/Users/.../logs/sts_20240128_123456.log"
Kill with:    kill 12345
```

---

### read_save.py / save.sh - Save File Reader

**Purpose:** Read and display the current STS save file in a human-readable format. Decrypts the XOR-encrypted autosave and shows run state.

**Usage:**

```bash
# Read WATCHER save (default)
./scripts/dev/save.sh

# Read specific character save
./scripts/dev/save.sh IRONCLAD
./scripts/dev/save.sh SILENT
./scripts/dev/save.sh DEFECT

# Or directly with Python
uv run scripts/dev/read_save.py WATCHER
```

**Example Output:**

```
==================================================
SEED: 1234567890
FLOOR: 12 | ACT: Exordium
HP: 65/80 | GOLD: 234
NEOW: RANDOM_COMMON_RELIC
BOSS: Hexaghost
ASCENSION: 20
==================================================

RELICS (4):
  PureWater, Vajra, Boot, Anchor

DECK (15):
  Strike_P
  Strike_P
  Strike_P
  Strike_P
  Defend_P
  Defend_P
  Defend_P
  Defend_P
  Vigilance
  Eruption+
  Crescendo
  WheelKick

POTIONS:
  Fire Potion
  Block Potion

PATH: M -> M -> ? -> M -> E -> R -> M -> M -> $ -> M -> R -> M

MONSTER LIST (upcoming):
  1. 2 Fungi Beasts
  2. Large Slime
  3. Looter

ELITE LIST (upcoming):
  1. Gremlin Nob

RNG COUNTERS:
  card_seed_count: 27
  relic_seed_count: 3
  potion_seed_count: 12
  event_seed_count: 4
  monster_seed_count: 8
```

---

## GUI Tools

Location: `/Users/jackswitzer/Desktop/SlayTheSpireRL/tools/`

### game_viewer.py - Game State Viewer

**Purpose:** DearPyGui-based graphical viewer that watches save files in real-time and displays game state with RNG predictions. Features parity checking between predictions and actual game data.

**Usage:**

```bash
uv run tools/game_viewer.py
```

**Features:**

- **Real-time save file monitoring** - Auto-updates when save changes
- **Current state panel** - Floor, HP, gold, relics, potions, deck
- **Predictions panel** - Monster queue, elite queue, boss predictions per act
- **Parity checking** - Compare predicted vs actual encounters
- **RNG counters** - Display current card/relic/potion RNG states
- **Manual seed input** - Generate predictions for any seed

**Example Window Layout:**

```
+------------------+-------------------+------------------+
| CURRENT STATE    | PREDICTIONS       | PARITY CHECK     |
|------------------|-------------------|------------------|
| Floor 5 | Act 1  | Act 1 | Act 2 | 3 | PARITY: 5/5 100% |
| HP: 65/80        |                   |                  |
| Gold: 234        | BOSS: Hexaghost   | Monster Compare: |
|                  |                   | Cultist ✓        |
| RELICS           | MONSTERS (10):    | 2 Louse ✓        |
| PureWater, ...   | 1. Cultist        |                  |
|                  | 2. 2 Louse        | Elite Compare:   |
| POTIONS          | 3. 3 Slime        | Lagavulin ✓      |
| Fire Potion      | ...               |                  |
|                  |                   | Boss: ✓          |
| DECK (15)        | ELITES:           |                  |
| Strike, ...      | 1. Lagavulin      | RNG COUNTERS     |
|                  | 2. 3 Sentries     | card: 27         |
| PATH             |                   | monster: 8       |
| M -> M -> ? ...  |                   | relic: 3         |
+------------------+-------------------+------------------+
```

---

### launcher.py - Simulation Launcher

**Purpose:** DearPyGui-based GUI for configuring and launching batch simulations. Allows setting character, ascension, seed counts, and mode.

**Usage:**

```bash
uv run tools/launcher.py
```

**Features:**

- **Character selection** - Ironclad, Silent, Defect, Watcher
- **Ascension slider** - 0-20
- **Batch configuration** - Total seeds, runs per seed
- **Visual/Headless mode** - Toggle for faster batch runs
- **Progress tracking** - Real-time progress bars, win/loss counts
- **Start/Stop controls** - Threaded execution with interruption support

**Configuration Options:**

| Option | Range | Default |
|--------|-------|---------|
| Character | IRONCLAD/SILENT/DEFECT/WATCHER | WATCHER |
| Ascension | 0-20 | 20 |
| Total Seeds | 1-10000 | 1 |
| Runs/Seed | 1-100 | 1 |
| Mode | Visual/Headless | Visual |

**Note:** Currently contains placeholder simulation logic. Replace `_run_simulation()` with actual engine integration.

---

### test_engine.py - Interactive Python Engine

**Purpose:** Run the Python game engine interactively in the terminal. Allows step-by-step gameplay with action selection, useful for comparing behavior with the Java game.

**Usage:**

```bash
# Default seed
uv run tools/test_engine.py

# Specific seed
uv run tools/test_engine.py --seed 1234567890

# Different ascension
uv run tools/test_engine.py --seed "ABCDEF" --ascension 15
```

**Interactive Commands:**

| Input | Action |
|-------|--------|
| `0-N` | Select action by index |
| `p` | Show encounter predictions |
| `q` | Quit |

**Example Session:**

```
============================================================
PYTHON ENGINE STATE
============================================================
Seed: ABCDEF
Floor: 0 | Act: 1
HP: 72/72
Gold: 99
Phase: NEOW

Deck (10 cards):
  4x Strike_P
  4x Defend_P
  1x Vigilance
  1x Eruption

Relics (1):
  - PureWater

Potions: []

AVAILABLE ACTIONS:
  [0] NEOW_THREE_CARDS
  [1] NEOW_HUNDRED_GOLD
  [2] NEOW_RANDOM_RARE_RELIC (drawback: TEN_PERCENT_HP_LOSS)
  [3] NEOW_BOSS_RELIC

Enter action number (or 'q' to quit, 'p' for predictions):
```

---

## Web Dashboard

Location: `/Users/jackswitzer/Desktop/SlayTheSpireRL/web/`

### server.py - Live Dashboard Server

**Purpose:** FastAPI-based web server providing a real-time dashboard for game state monitoring. Uses Server-Sent Events (SSE) for live updates.

**Usage:**

```bash
# Start the server
uv run python web/server.py

# Then open browser to:
# http://localhost:8080
```

**API Endpoints:**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Serve the dashboard HTML |
| `/api/state` | GET | Get current game state as JSON |
| `/api/stream` | GET | SSE stream for live updates |
| `/api/predict/cards` | GET | Predict card rewards |
| `/api/predict/boss-relics` | GET | Predict boss relic offerings |
| `/api/predict/path` | POST | Predict cumulative rewards for a path |

**Query Parameters for `/api/predict/cards`:**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `seed` | string | required | Seed string |
| `counter` | int | 0 | Card RNG counter |
| `blizzard` | int | 5 | Card blizzard value |
| `act` | int | 1 | Current act |
| `room_type` | string | "normal" | "normal" or "elite" |

**Features:**

- **Real-time updates** - 2-second polling of save file
- **Map generation** - Shows current act's map
- **Path tree** - Displays accessible nodes and 3-floor lookahead
- **Full predictions** - Card rewards, boss relics, shop inventory, treasure
- **RNG accuracy tracking** - Compares predictions with actual results
- **Floor history** - Complete run timeline with all choices
- **Neow options** - Predicts Neow's blessing options at floor 0

---

### index.html - STS Oracle UI

**Purpose:** Single-page web application providing a rich dashboard interface for the STS Oracle. Dark theme styled after the game's aesthetic.

**Features:**

- **Top bar** - Seed, floor, HP, gold display
- **Map visualization** - Interactive map with path highlighting
- **Prediction panels** - Card rewards, relics, potions
- **Floor timeline** - Historical view of run progress
- **RNG state display** - Current counter values
- **Live connection** - SSE-based real-time updates

**Styling:**

- Cinzel font for headers (matches STS aesthetic)
- Dark purple/gold color scheme
- Responsive layout with multiple panels

**Access:**

Open `http://localhost:8080` after starting `server.py`

---

## When to Use Each Tool

| Task | Tool |
|------|------|
| **Verify RNG implementation** | `parity.sh` / `test_parity.py` |
| **Debug encounter predictions** | `parity.sh --seed SEED` |
| **Quick save file inspection** | `save.sh` |
| **Real-time game monitoring (GUI)** | `game_viewer.py` |
| **Real-time game monitoring (Web)** | `server.py` + browser |
| **Test Python engine step-by-step** | `test_engine.py` |
| **Launch STS with mods** | `launch_sts.sh` |
| **Batch simulation runs** | `launcher.py` |
| **Compare Python vs Java side-by-side** | `engine.sh` + `launch_sts.sh --seed SEED` |

### Typical Workflows

**Validating RNG Implementation:**
```bash
# 1. Start a run in STS with known seed
./scripts/dev/launch_sts.sh

# 2. After progressing, check parity
./scripts/dev/parity.sh

# 3. If mismatches, investigate with viewer
uv run tools/game_viewer.py
```

**Live Dashboard During Play:**
```bash
# 1. Start dashboard server
uv run python web/server.py

# 2. Open browser to http://localhost:8080

# 3. Start STS
./scripts/dev/launch_sts.sh

# 4. Dashboard auto-updates as you play
```

**Testing Engine Changes:**
```bash
# 1. Generate predictions for a seed
uv run scripts/dev/test_parity.py --seed "TESTSEED"

# 2. Run same seed in interactive engine
uv run tools/test_engine.py --seed "TESTSEED"

# 3. Compare behavior with actual game
./scripts/dev/launch_sts.sh
# (Start custom seed run in STS)
```
