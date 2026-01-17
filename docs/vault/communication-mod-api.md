# CommunicationMod API Reference

External control protocol for Slay the Spire bot development.

## Overview

CommunicationMod enables external processes to control STS gameplay through stdin/stdout JSON communication. Launches specified command and exchanges game state with that process.

**Repository**: https://github.com/ForgottenArbiter/CommunicationMod

## Setup

1. Place `CommunicationMod.jar` in mods directory
2. Enable via ModTheSpire
3. Edit `SpireConfig` to set command:
   ```
   command=python /path/to/bot.py
   ```

## Protocol Flow

```
1. Mod launches external process
2. Process sends: "ready\n"
3. Mod waits for game state stability
4. Mod sends: JSON state object
5. Process sends: command string
6. Repeat from step 3
```

**Timeout**: ~60 seconds for "ready" signal, then process killed.

## State JSON Structure

```json
{
  "available_commands": ["play", "end", "potion"],
  "ready_for_command": true,
  "in_game": true,
  "game_state": {
    "current_hp": 45,
    "max_hp": 60,
    "floor": 3,
    "act": 1,
    "gold": 120,
    "seed": 1234567890,
    "class": "WATCHER",
    "ascension_level": 20,
    "relics": [{"id": "PenNib", "counter": 0}],
    "deck": [{"id": "Strike_P", "upgrades": 0}],
    "potions": [{"id": "StrengthPotion", "can_use": true}],
    "combat_state": {
      "player": {"current_hp": 45, "block": 8, "energy": 3, "powers": []},
      "monsters": [{"name": "Cultist", "current_hp": 18, "intent": "ATTACK", "move_adjusted_damage": 7}],
      "hand": [{"id": "Strike_P", "cost": 1, "is_playable": true}],
      "draw_pile": [...],
      "discard_pile": [],
      "turn": 1
    }
  }
}
```

## Commands

### Game Control

| Command | Syntax | Description |
|---------|--------|-------------|
| `START` | `START WATCHER [ascension] [seed]` | Begin new run |
| `PLAY` | `PLAY card_index [target_index]` | Play card (1-indexed) |
| `END` | `END` | End turn |
| `POTION` | `POTION use\|discard slot [target]` | Use/discard potion |

### Navigation

| Command | Syntax | Description |
|---------|--------|-------------|
| `CHOOSE` | `CHOOSE index\|name` | Select menu option |
| `PROCEED` | `PROCEED` | Advance screen |
| `RETURN` | `RETURN` | Go back |
| `STATE` | `STATE` | Request immediate state |
| `WAIT` | `WAIT timeout_ms` | Pause without action |

## Monster Intent Types

```
ATTACK, ATTACK_BUFF, ATTACK_DEBUFF, ATTACK_DEFEND,
BUFF, DEBUFF, STRONG_DEBUFF, DEFEND, DEFEND_BUFF,
DEFEND_DEBUFF, ESCAPE, MAGIC, NONE, SLEEP, STUN, UNKNOWN
```

## Python Bot Template

```python
import json, sys

def main():
    print("ready", flush=True)
    while True:
        state = json.loads(sys.stdin.readline())
        if "error" in state:
            print("state", flush=True)
            continue
        if not state.get("in_game"):
            print("start WATCHER 20", flush=True)
            continue

        gs = state["game_state"]
        if "play" in state["available_commands"]:
            print("play 1", flush=True)
        elif "end" in state["available_commands"]:
            print("end", flush=True)
        else:
            print("state", flush=True)

if __name__ == "__main__":
    main()
```

## Known Limitations

- Match and Keep event state incomplete
- No feedback when potion inventory full
- Card deselection unsupported in hand select
- Requires `fast mode` enabled
- Config file requires manual editing
