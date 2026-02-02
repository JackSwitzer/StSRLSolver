#!/usr/bin/env python3
"""
STS Dashboard State Watcher

Watch for state changes and display updates in real-time.
Useful for monitoring game progress and debugging state updates.

Usage:
    uv run scripts/dev/watch_state.py               # Watch all changes
    uv run scripts/dev/watch_state.py --floor       # Watch floor changes only
    uv run scripts/dev/watch_state.py --rng         # Watch RNG changes
    uv run scripts/dev/watch_state.py --compact     # Compact display
    uv run scripts/dev/watch_state.py -i 1          # 1 second interval
"""

import argparse
import json
import os
import sys
import time
import urllib.request
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, Optional

API_URL = "http://127.0.0.1:8080/api/state"


def fetch_state() -> Dict[str, Any]:
    """Fetch current state from API."""
    try:
        with urllib.request.urlopen(API_URL, timeout=5) as response:
            return json.loads(response.read().decode())
    except urllib.error.URLError:
        return {"error": "Server not responding"}
    except Exception as e:
        return {"error": str(e)}


def detect_changes(old: Dict, new: Dict, prefix: str = "") -> list:
    """Detect changes between two state dicts."""
    changes = []

    # Keys in new but not old
    for key in new:
        if key not in old:
            changes.append(f"+ {prefix}{key}")
        elif isinstance(new[key], dict) and isinstance(old.get(key), dict):
            changes.extend(detect_changes(old[key], new[key], f"{prefix}{key}."))
        elif new[key] != old.get(key):
            # Summarize change
            old_val = old.get(key)
            new_val = new[key]

            # For lists, show length change
            if isinstance(new_val, list) and isinstance(old_val, list):
                if len(new_val) != len(old_val):
                    changes.append(f"~ {prefix}{key}: len {len(old_val)} -> {len(new_val)}")
            # For simple values, show change
            elif not isinstance(new_val, (dict, list)):
                changes.append(f"~ {prefix}{key}: {old_val} -> {new_val}")
            else:
                changes.append(f"~ {prefix}{key}: (changed)")

    # Keys in old but not new
    for key in old:
        if key not in new:
            changes.append(f"- {prefix}{key}")

    return changes


def format_state_summary(state: Dict) -> str:
    """Format a compact state summary."""
    if "error" in state:
        return f"ERROR: {state['error']}"

    seed = state.get("seed", {}).get("string", "?")
    run = state.get("run", {})
    floor = run.get("floor", 0)
    act = run.get("act", 1)
    room = run.get("room_type", "?")
    player = state.get("player", {})
    hp = player.get("hp", 0)
    max_hp = player.get("max_hp", 0)
    gold = player.get("gold", 0)

    relics = len(state.get("relics", []))
    deck = state.get("deck_count", 0)
    path = state.get("path_taken", [])
    path_str = "".join(path[-10:]) if path else "-"

    return f"F{floor} A{act} [{room}] | HP:{hp}/{max_hp} G:{gold} | R:{relics} D:{deck} | {path_str}"


def format_state_full(state: Dict) -> str:
    """Format a full state display."""
    if "error" in state:
        return f"ERROR: {state['error']}"

    lines = []

    # Header
    seed = state.get("seed", {}).get("string", "?")
    player_class = state.get("player_class", "?")
    lines.append(f"Seed: {seed} | {player_class}")

    # Run info
    run = state.get("run", {})
    floor = run.get("floor", 0)
    act = run.get("act", 1)
    room = run.get("room_type", "?")
    current_room = run.get("current_room", "").split(".")[-1]
    lines.append(f"Floor {floor} | Act {act} | Room: {room} ({current_room})")

    # Player
    player = state.get("player", {})
    hp = player.get("hp", 0)
    max_hp = player.get("max_hp", 0)
    gold = player.get("gold", 0)
    lines.append(f"HP: {hp}/{max_hp} | Gold: {gold}")

    # Deck/Relics
    deck = state.get("deck_count", 0)
    relics = state.get("relics", [])
    lines.append(f"Deck: {deck} | Relics: {len(relics)}")

    # Path
    path = state.get("path_taken", [])
    if path:
        lines.append(f"Path: {' '.join(path[-15:])}")

    # Neow (if at floor 0)
    neow = state.get("neow_options", [])
    if floor == 0 and neow:
        lines.append("")
        lines.append("NEOW OPTIONS:")
        for opt in neow:
            name = opt.get("name", "?") if isinstance(opt, dict) else str(opt)
            drawback = opt.get("drawback") if isinstance(opt, dict) else None
            if drawback:
                lines.append(f"  - {name} ({drawback})")
            else:
                lines.append(f"  - {name}")

    # Bosses
    bosses = state.get("predicted_bosses", {})
    if bosses:
        lines.append("")
        lines.append("BOSSES:")
        for act_num in sorted(bosses.keys(), key=lambda x: int(x) if str(x).isdigit() else 0):
            boss_list = bosses[act_num]
            marker = " <--" if int(act_num) == act else ""
            if isinstance(boss_list, list):
                lines.append(f"  Act {act_num}: {' + '.join(boss_list)}{marker}")

    # RNG
    rng = state.get("rng", {})
    if rng:
        lines.append("")
        lines.append("RNG COUNTERS:")
        for key in ["card_counter", "relic_counter", "potion_counter", "event_counter"]:
            if key in rng:
                lines.append(f"  {key}: {rng[key]}")

    # RNG Accuracy
    rng_acc = state.get("rng_accuracy", {})
    if rng_acc and rng_acc.get("overall", {}).get("total", 0) > 0:
        overall = rng_acc["overall"]
        lines.append("")
        lines.append(f"RNG Accuracy: {overall['ratio']*100:.0f}% ({overall['correct']}/{overall['total']})")

    return "\n".join(lines)


def format_rng_state(state: Dict) -> str:
    """Format RNG-focused state display."""
    if "error" in state:
        return f"ERROR: {state['error']}"

    lines = []

    rng = state.get("rng", {})
    lines.append("RNG COUNTERS:")
    for key in sorted(rng.keys()):
        lines.append(f"  {key}: {rng[key]}")

    rng_acc = state.get("rng_accuracy", {})
    if rng_acc:
        lines.append("")
        lines.append("RNG ACCURACY:")
        overall = rng_acc.get("overall", {})
        lines.append(f"  Overall: {overall.get('ratio', 0)*100:.0f}% ({overall.get('correct', 0)}/{overall.get('total', 0)})")

        for cat in ["cards", "relics", "potions"]:
            cat_data = rng_acc.get(cat, {})
            if cat_data.get("total", 0) > 0:
                lines.append(f"  {cat.title()}: {cat_data['ratio']*100:.0f}% ({cat_data['correct']}/{cat_data['total']})")

        mismatches = rng_acc.get("mismatches", [])
        if mismatches:
            lines.append(f"\nMISMATCHES ({len(mismatches)}):")
            for m in mismatches[-3:]:
                floor = m.get("floor", "?")
                mtype = m.get("type", "?")
                if "error" in m:
                    lines.append(f"  F{floor} ({mtype}): ERROR")
                else:
                    lines.append(f"  F{floor} ({mtype}): predicted {m.get('predicted')} vs actual {m.get('actual')}")

    return "\n".join(lines)


def watch(
    interval: float = 2.0,
    compact: bool = False,
    floor_only: bool = False,
    rng_only: bool = False,
    show_changes: bool = True,
) -> None:
    """Watch for state changes."""
    last_state = None
    last_floor = None

    while True:
        state = fetch_state()
        now = datetime.now().strftime("%H:%M:%S")

        # Check for floor change
        current_floor = state.get("run", {}).get("floor", -1)
        floor_changed = last_floor is not None and current_floor != last_floor

        # Determine if we should display
        should_display = False
        if last_state is None:
            should_display = True
        elif floor_only:
            should_display = floor_changed
        else:
            should_display = state != last_state

        if should_display:
            # Clear screen
            os.system('clear')

            # Header
            print(f"[{now}] STS State Watcher")
            print("=" * 50)

            # Display appropriate format
            if compact:
                print(format_state_summary(state))
            elif rng_only:
                print(format_rng_state(state))
            else:
                print(format_state_full(state))

            # Show changes
            if show_changes and last_state and not compact:
                changes = detect_changes(last_state, state)
                if changes:
                    print()
                    print("CHANGES:")
                    for change in changes[:10]:
                        print(f"  {change}")
                    if len(changes) > 10:
                        print(f"  ... and {len(changes) - 10} more")

            print()
            print(f"Watching... (Ctrl+C to stop)")

        last_state = state
        last_floor = current_floor

        try:
            time.sleep(interval)
        except KeyboardInterrupt:
            break

    print("\nStopped.")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Watch STS dashboard state changes",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    uv run scripts/dev/watch_state.py               # Watch all changes
    uv run scripts/dev/watch_state.py --compact     # Compact single-line display
    uv run scripts/dev/watch_state.py --floor       # Only update on floor changes
    uv run scripts/dev/watch_state.py --rng         # Focus on RNG state
    uv run scripts/dev/watch_state.py -i 0.5        # Fast polling (0.5s)
"""
    )

    parser.add_argument("-i", "--interval", type=float, default=2.0,
                        help="Polling interval in seconds (default: 2)")
    parser.add_argument("--compact", action="store_true",
                        help="Compact single-line display")
    parser.add_argument("--floor", action="store_true",
                        help="Only update display on floor changes")
    parser.add_argument("--rng", action="store_true",
                        help="Focus on RNG state and accuracy")
    parser.add_argument("--no-changes", action="store_true",
                        help="Don't show change detection")

    args = parser.parse_args()

    # Check server is running
    state = fetch_state()
    if "error" in state and "Server not responding" in state["error"]:
        print("Server is not running. Start it with:")
        print("  uv run scripts/sts.py dashboard")
        return 1

    print("Starting state watcher...")
    print(f"Interval: {args.interval}s")
    print("Press Ctrl+C to stop")
    print()

    watch(
        interval=args.interval,
        compact=args.compact,
        floor_only=args.floor,
        rng_only=args.rng,
        show_changes=not args.no_changes,
    )

    return 0


if __name__ == "__main__":
    sys.exit(main())
