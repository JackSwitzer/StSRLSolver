#!/usr/bin/env python3
"""
Real-time log watcher for EVTracker mod.
Can run as socket server (receives live events) or file tailer.

Usage:
    python log_watcher.py              # Socket server on port 9999
    python log_watcher.py --file       # Tail latest log file
    python log_watcher.py --file <path> # Tail specific file
"""

import json
import socket
import sys
import time
from pathlib import Path
from datetime import datetime

LOG_DIR = Path.home() / "Desktop" / "SlayTheSpireRL" / "logs"
SOCKET_PORT = 9999


def format_event(event: dict) -> str:
    """Pretty format an event for terminal display."""
    event_type = event.get("type", "unknown")
    timestamp = event.get("timestamp", 0)
    data = event.get("data", {})

    time_str = datetime.fromtimestamp(timestamp / 1000).strftime("%H:%M:%S")

    # Color codes
    COLORS = {
        "run_start": "\033[92m",      # Green
        "run_end": "\033[91m",        # Red
        "battle_start": "\033[94m",   # Blue
        "battle_end": "\033[96m",     # Cyan
        "turn_start": "\033[93m",     # Yellow
        "card_played": "\033[95m",    # Magenta
        "player_damaged": "\033[91m", # Red
    }
    RESET = "\033[0m"

    color = COLORS.get(event_type, "")

    # Format based on event type
    if event_type == "card_played":
        card = data.get("card", {})
        return f"{color}[{time_str}] CARD: {card.get('name', '?')} (cost={card.get('cost')}, dmg={card.get('damage')}, blk={card.get('block')}){RESET}"

    elif event_type == "turn_start":
        player = data.get("player_state", {})
        monsters = data.get("monsters", [])
        monster_str = ", ".join(f"{m.get('name', '?')}:{m.get('hp')}/{m.get('max_hp')}"
                                for m in monsters[:3])
        return f"{color}[{time_str}] TURN {data.get('turn')}: HP={player.get('hp')}/{player.get('max_hp')} Block={player.get('block')} Energy={data.get('energy')} | {monster_str}{RESET}"

    elif event_type == "battle_start":
        monsters = data.get("monsters", [])
        monster_str = ", ".join(f"{m.get('name', '?')}" for m in monsters)
        return f"{color}[{time_str}] BATTLE START (floor {data.get('floor')}): {monster_str}{RESET}"

    elif event_type == "battle_end":
        player = data.get("player_state", {})
        return f"{color}[{time_str}] BATTLE END: HP={player.get('hp')}/{player.get('max_hp')} Turns={data.get('turns_taken')}{RESET}"

    elif event_type == "player_damaged":
        return f"{color}[{time_str}] DAMAGE: {data.get('damage_amount')} from {data.get('source', '?')}{RESET}"

    elif event_type == "run_start":
        return f"{color}[{time_str}] === RUN START: {data.get('character')} A{data.get('ascension')} ==={RESET}"

    elif event_type == "run_end":
        player = data.get("player_state", {})
        result = "VICTORY" if data.get("victory") else "DEFEAT"
        return f"{color}[{time_str}] === RUN END: {result} Floor {data.get('floor')} HP={player.get('hp')}/{player.get('max_hp')} ==={RESET}"

    else:
        return f"[{time_str}] {event_type}: {json.dumps(data, separators=(',', ':'))[:100]}"


def run_socket_server():
    """Run socket server to receive live events from game."""
    print(f"Starting EVTracker socket server on port {SOCKET_PORT}...")
    print("Launch the game and events will appear here.\n")

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("localhost", SOCKET_PORT))
    server.listen(1)

    while True:
        try:
            print("Waiting for game connection...")
            conn, addr = server.accept()
            print(f"Game connected from {addr}\n")

            buffer = ""
            while True:
                data = conn.recv(4096)
                if not data:
                    print("\nGame disconnected.")
                    break

                buffer += data.decode("utf-8")
                while "\n" in buffer:
                    line, buffer = buffer.split("\n", 1)
                    if line.strip():
                        try:
                            event = json.loads(line)
                            print(format_event(event))
                        except json.JSONDecodeError:
                            print(f"Invalid JSON: {line[:50]}...")

        except KeyboardInterrupt:
            print("\nShutting down...")
            break
        except Exception as e:
            print(f"Error: {e}")
            time.sleep(1)

    server.close()


def tail_file(filepath: Path):
    """Tail a log file for events."""
    print(f"Tailing: {filepath}\n")

    with open(filepath, "r") as f:
        # Go to end
        f.seek(0, 2)

        while True:
            line = f.readline()
            if line:
                try:
                    event = json.loads(line.strip())
                    print(format_event(event))
                except json.JSONDecodeError:
                    pass
            else:
                time.sleep(0.1)


def find_latest_log() -> Path:
    """Find the most recent log file."""
    logs = list(LOG_DIR.glob("evlog_*.jsonl"))
    if not logs:
        print(f"No log files found in {LOG_DIR}")
        sys.exit(1)
    return max(logs, key=lambda p: p.stat().st_mtime)


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "--file":
        if len(sys.argv) > 2:
            filepath = Path(sys.argv[2])
        else:
            filepath = find_latest_log()
        tail_file(filepath)
    else:
        run_socket_server()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nExiting...")
