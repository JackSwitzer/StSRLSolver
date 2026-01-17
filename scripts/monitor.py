#!/usr/bin/env python3
"""
Quick monitoring script - outputs latest game state for Claude to read.
Run: python3 monitor.py [--watch]
"""

import json
import sys
import time
from pathlib import Path

LOG_DIR = Path.home() / "Desktop" / "SlayTheSpireRL" / "logs"

def get_latest_log():
    logs = list(LOG_DIR.glob("evlog_*.jsonl"))
    if not logs:
        return None
    return max(logs, key=lambda p: p.stat().st_mtime)

def get_recent_events(n=20):
    log = get_latest_log()
    if not log:
        return []

    with open(log) as f:
        lines = f.readlines()

    events = []
    for line in lines[-n:]:
        try:
            events.append(json.loads(line.strip()))
        except:
            pass
    return events

def get_game_state():
    """Extract current game state from recent events."""
    events = get_recent_events(50)

    state = {
        "run_active": False,
        "in_combat": False,
        "floor": 0,
        "hp": 0,
        "max_hp": 0,
        "last_events": []
    }

    for e in events:
        etype = e.get("type")
        data = e.get("data", {})

        if etype == "run_start":
            state["run_active"] = True
            state["character"] = data.get("character")
            state["ascension"] = data.get("ascension")
            state["seed"] = data.get("seed")

        elif etype == "run_end":
            state["run_active"] = False
            state["victory"] = data.get("victory")

        elif etype == "battle_start":
            state["in_combat"] = True
            state["floor"] = data.get("floor")
            ps = data.get("player_state", {})
            state["hp"] = ps.get("hp")
            state["max_hp"] = ps.get("max_hp")
            state["monsters"] = [m.get("name") for m in data.get("monsters", [])]

        elif etype == "battle_end":
            state["in_combat"] = False
            ps = data.get("player_state", {})
            state["hp"] = ps.get("hp")
            state["max_hp"] = ps.get("max_hp")

        elif etype == "turn_start":
            state["turn"] = data.get("turn")
            state["energy"] = data.get("energy")
            ps = data.get("player_state", {})
            state["hp"] = ps.get("hp")
            state["block"] = ps.get("block")
            state["stance"] = ps.get("stance")
            state["hand"] = [c.get("name") for c in data.get("hand", [])]

        elif etype == "player_damaged":
            state["last_damage"] = data.get("damage_amount")
            state["damage_source"] = data.get("source")

    # Get last 5 events summary
    for e in events[-5:]:
        etype = e.get("type")
        data = e.get("data", {})
        if etype == "card_played":
            state["last_events"].append(f"Played {data.get('card', {}).get('name')}")
        elif etype == "player_damaged":
            state["last_events"].append(f"Took {data.get('damage_amount')} from {data.get('source')}")
        elif etype == "battle_start":
            monsters = [m.get("name") for m in data.get("monsters", [])]
            state["last_events"].append(f"Battle: {', '.join(monsters)}")
        elif etype == "battle_end":
            state["last_events"].append("Battle won")

    return state

def main():
    watch = "--watch" in sys.argv

    if watch:
        print("Watching for game events... (Ctrl+C to stop)")
        last_count = 0
        while True:
            events = get_recent_events(100)
            if len(events) > last_count:
                for e in events[last_count:]:
                    print(json.dumps(e, separators=(',', ':')))
                last_count = len(events)
            time.sleep(0.5)
    else:
        state = get_game_state()
        print(json.dumps(state, indent=2))

if __name__ == "__main__":
    main()
