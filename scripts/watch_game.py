"""Real-time monitor for EVTracker log files.

Tails the latest evlog_*.jsonl and prints formatted summaries as the user plays.

Usage:
    uv run python scripts/watch_game.py
    uv run python scripts/watch_game.py logs/evlog_XXXX.jsonl
"""

import argparse
import json
import sys
import time
from pathlib import Path

LOG_DIR = Path(__file__).parent.parent / "logs"


def find_latest_log() -> Path | None:
    """Find the most recently modified evlog file."""
    logs = sorted(LOG_DIR.glob("evlog_*.jsonl"), key=lambda p: p.stat().st_mtime)
    return logs[-1] if logs else None


def format_event(event: dict) -> str | None:
    """Format a log event into a human-readable summary line."""
    etype = event.get("type", "")
    data = event.get("data", {})

    if etype == "system":
        return f"[SYSTEM] {data.get('message', '')}"

    if etype == "run_start":
        return (
            f"\n{'='*60}\n"
            f"RUN START: {data['character']} A{data['ascension']} "
            f"(seed: {data['seed']})\n"
            f"{'='*60}"
        )

    if etype == "dungeon_init":
        ps = data.get("player_state", {})
        relics = [r["name"] for r in ps.get("relics", data.get("relics", []))]
        deck_size = len(data.get("deck", []))
        return (
            f"[INIT] HP: {ps.get('hp')}/{ps.get('max_hp')} | "
            f"Gold: {ps.get('gold')} | Deck: {deck_size} cards | "
            f"Relics: {', '.join(relics)}"
        )

    if etype == "card_removed":
        card = data["card"]["name"]
        floor = data.get("floor", "?")
        return f"  [REMOVE] Floor {floor}: Removed {card}"

    if etype == "battle_start":
        floor = data["floor"]
        room = data.get("room_type", "?")
        monsters = [m["name"] for m in data["monsters"]]
        return f"\nFloor {floor}: {', '.join(monsters)} ({room})"

    if etype == "turn_start":
        turn = data.get("turn", "?")
        ps = data.get("player_state", {})
        monsters = data.get("monsters", [])
        monster_hp = " | ".join(f"{m['name']}:{m['hp']}/{m['max_hp']}" for m in monsters)
        incoming = data.get("incoming_damage", 0)
        stance = ps.get("stance", "Neutral")
        stance_str = f" [{stance}]" if stance != "Neutral" else ""
        return (
            f"  Turn {turn}{stance_str}: "
            f"HP {ps.get('hp')}/{ps.get('max_hp')} Block:{ps.get('block', 0)} | "
            f"{monster_hp}"
            f"{f' | Incoming:{incoming}' if incoming else ''}"
        )

    if etype == "card_played":
        card = data["card"]
        name = card["name"]
        target = data.get("target", {})
        dmg = data.get("calculated_damage")
        target_name = target.get("name", "")
        parts = [f"    Played {name}"]
        if dmg and dmg > 0:
            parts.append(f"-> {target_name} for {dmg}")
        block = card.get("display_block", -1)
        if block and block > 0:
            parts.append(f"(+{block} block)")
        return " ".join(parts)

    if etype == "player_damaged":
        src = data.get("source", "?")
        amt = data.get("damage_amount", 0)
        block = data.get("player_block", 0)
        hp_before = data.get("player_hp_before", 0)
        actual = max(0, amt - block)
        return f"    TOOK {actual} damage from {src} ({amt} - {block} block) HP: {hp_before} -> {hp_before - actual}"

    if etype == "battle_end":
        result = "WIN" if data.get("victory") else "LOSS"
        floor = data["floor"]
        taken = data.get("total_damage_taken", 0)
        dealt = data.get("total_damage_dealt", 0)
        turns = data.get("turns_taken", 0)
        review = data.get("combat_review", {})
        opt = review.get("optimality_score", "?")
        mistakes = review.get("key_mistakes", [])
        lines = [f"  BATTLE {result} (Floor {floor}): {dealt} dealt, {taken} taken in {turns} turns | Optimality: {opt}%"]
        for m in mistakes:
            lines.append(f"    MISTAKE: {m}")
        return "\n".join(lines)

    if etype == "card_reward_presented":
        floor = data["floor"]
        cards = [c["name"] for c in data["offered_cards"]]
        return f"  Card reward: [{', '.join(cards)}]"

    if etype == "card_obtained":
        card = data["card"]["name"]
        floor = data.get("floor", "?")
        source = data.get("source", "?")
        return f"  -> Chose: {card} (floor {floor}, {source})"

    if etype == "potion_used":
        potion = data.get("potion", {}).get("name", "?")
        return f"    Used potion: {potion}"

    if etype == "run_end":
        result = "VICTORY" if data.get("victory") else "DEFEAT"
        floor = data.get("floor", "?")
        ps = data.get("player_state", {})
        return (
            f"\n{'='*60}\n"
            f"RUN {result} on floor {floor} | "
            f"HP: {ps.get('hp')}/{ps.get('max_hp')}\n"
            f"{'='*60}"
        )

    # Skip turn_start_post_draw and other verbose events
    return None


def tail_log(path: Path):
    """Tail a log file and print formatted events."""
    print(f"Watching: {path}")
    print(f"(Ctrl+C to stop)\n")

    with open(path) as f:
        # Process existing content first
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                event = json.loads(line)
                msg = format_event(event)
                if msg:
                    print(msg)
            except json.JSONDecodeError:
                continue

        # Now tail for new content
        print("\n--- Watching for new events ---\n")
        while True:
            line = f.readline()
            if not line:
                time.sleep(0.1)
                continue
            line = line.strip()
            if not line:
                continue
            try:
                event = json.loads(line)
                msg = format_event(event)
                if msg:
                    print(msg, flush=True)
            except json.JSONDecodeError:
                continue


def main():
    parser = argparse.ArgumentParser(description="Real-time EVTracker log monitor")
    parser.add_argument(
        "logfile",
        nargs="?",
        type=Path,
        help="Path to .jsonl log file (default: latest in logs/)",
    )
    args = parser.parse_args()

    if args.logfile:
        path = args.logfile
    else:
        path = find_latest_log()
        if path is None:
            print(f"No evlog_*.jsonl files found in {LOG_DIR}", file=sys.stderr)
            sys.exit(1)

    if not path.exists():
        print(f"File not found: {path}", file=sys.stderr)
        sys.exit(1)

    try:
        tail_log(path)
    except KeyboardInterrupt:
        print("\nStopped.")


if __name__ == "__main__":
    main()
