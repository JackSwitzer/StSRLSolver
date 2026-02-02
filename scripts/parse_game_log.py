"""Parse an EVTracker .jsonl log file into a seed_catalog.py VERIFIED_SEEDS entry.

Usage:
    uv run python scripts/parse_game_log.py logs/evlog_XXXX.jsonl
    uv run python scripts/parse_game_log.py --help
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def parse_log(path: Path) -> dict[str, Any]:
    """Parse a .jsonl log file into a structured seed verification record."""
    events: list[dict] = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line:
                events.append(json.loads(line))

    # Extract run metadata
    run_start = next((e for e in events if e["type"] == "run_start"), None)
    if not run_start:
        print("ERROR: No run_start event found in log.", file=sys.stderr)
        sys.exit(1)

    data = run_start["data"]
    seed_numeric = data["seed"]
    character = data["character"]
    ascension = data["ascension"]
    run_id = data["run_id"]

    # Filter to this run
    run_events = [e for e in events if e.get("data", {}).get("run_id") == run_id]

    # Dungeon init (deck, relics, potions at start)
    dungeon_init = next((e for e in run_events if e["type"] == "dungeon_init"), None)

    # Neow actions (card_removed/card_obtained at floor 0)
    neow_events = [
        e for e in run_events
        if e["type"] in ("card_removed", "card_obtained") and e["data"].get("floor") == 0
    ]
    neow_info = {}
    if neow_events:
        removes = [e for e in neow_events if e["type"] == "card_removed"]
        obtains = [e for e in neow_events if e["type"] == "card_obtained"]
        if removes:
            neow_info["neow_removes"] = [e["data"]["card"]["name"] for e in removes]
        if obtains:
            neow_info["neow_obtains"] = [e["data"]["card"]["name"] for e in obtains]

    # Per-floor data
    floors: dict[int, dict] = {}

    # Battle starts -> encounter info
    for e in run_events:
        if e["type"] == "battle_start":
            d = e["data"]
            floor = d["floor"]
            monsters = [m["name"] for m in d["monsters"]]
            floors.setdefault(floor, {})
            floors[floor]["encounter"] = monsters
            floors[floor]["room_type"] = d["room_type"]

    # Battle ends -> outcome per floor
    for e in run_events:
        if e["type"] == "battle_end":
            d = e["data"]
            floor = d["floor"]
            floors.setdefault(floor, {})
            floors[floor]["victory"] = d["victory"]
            floors[floor]["damage_taken"] = d["total_damage_taken"]
            floors[floor]["damage_dealt"] = d["total_damage_dealt"]
            floors[floor]["turns"] = d["turns_taken"]
            if "combat_review" in d:
                review = d["combat_review"]
                floors[floor]["hp_lost"] = review.get("hp_lost", 0)
                floors[floor]["optimality"] = review.get("optimality_score", 0)

    # Card rewards offered
    for e in run_events:
        if e["type"] == "card_reward_presented":
            d = e["data"]
            floor = d["floor"]
            floors.setdefault(floor, {})
            floors[floor]["card_rewards"] = [c["name"] for c in d["offered_cards"]]

    # Card obtained (chosen)
    for e in run_events:
        if e["type"] == "card_obtained":
            d = e["data"]
            floor = d["floor"]
            if floor == 0:
                continue  # Neow, handled above
            floors.setdefault(floor, {})
            floors[floor]["card_chosen"] = d["card"]["name"]

    # Potion usage
    for e in run_events:
        if e["type"] == "potion_used":
            d = e["data"]
            floor = d.get("floor")
            if floor is not None:
                floors.setdefault(floor, {})
                floors[floor].setdefault("potions_used", []).append(
                    d.get("potion", {}).get("name", "unknown")
                )

    # Run end
    run_end = next((e for e in run_events if e["type"] == "run_end"), None)
    outcome = {}
    if run_end:
        rd = run_end["data"]
        outcome["victory"] = rd["victory"]
        outcome["floor_reached"] = rd["floor"]
        outcome["total_damage_taken"] = rd["total_damage_taken"]
        outcome["total_damage_dealt"] = rd["total_damage_dealt"]

    # Build card_rewards dict matching seed_catalog format
    card_rewards = {}
    for floor_num in sorted(floors.keys()):
        if "card_rewards" in floors[floor_num]:
            card_rewards[floor_num] = floors[floor_num]["card_rewards"]

    # Build result
    result: dict[str, Any] = {
        "numeric_seed": seed_numeric,
        "character": character,
        "ascension": ascension,
    }

    if neow_info:
        result.update(neow_info)

    if card_rewards:
        result["card_rewards"] = card_rewards

    # Per-floor encounters
    encounters = {}
    for floor_num in sorted(floors.keys()):
        f = floors[floor_num]
        if "encounter" in f:
            names = f["encounter"]
            encounters[floor_num] = names[0] if len(names) == 1 else ", ".join(names)
    if encounters:
        result["encounters"] = encounters

    # Cards chosen
    cards_chosen = {}
    for floor_num in sorted(floors.keys()):
        if "card_chosen" in floors[floor_num]:
            cards_chosen[floor_num] = floors[floor_num]["card_chosen"]
    if cards_chosen:
        result["cards_chosen"] = cards_chosen

    if outcome:
        result["outcome"] = outcome

    result["floors_verified"] = max(floors.keys()) if floors else 0

    return result


def format_result(result: dict) -> str:
    """Format the result dict as pasteable Python code."""
    lines = ['    "SEED_STRING": {']
    for key, val in result.items():
        if isinstance(val, dict):
            lines.append(f"        {key!r}: {{")
            for k2, v2 in val.items():
                lines.append(f"            {k2!r}: {v2!r},")
            lines.append("        },")
        else:
            lines.append(f"        {key!r}: {val!r},")
    lines.append("    },")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Parse EVTracker .jsonl log into seed_catalog.py format"
    )
    parser.add_argument(
        "logfile",
        nargs="?",
        type=Path,
        help="Path to .jsonl log file",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output raw JSON instead of Python dict format",
    )
    args = parser.parse_args()

    if args.logfile is None:
        parser.print_help()
        sys.exit(0)

    if not args.logfile.exists():
        print(f"ERROR: File not found: {args.logfile}", file=sys.stderr)
        sys.exit(1)

    result = parse_log(args.logfile)

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print("# Paste into VERIFIED_SEEDS in packages/parity/seed_catalog.py:")
        print(format_result(result))


if __name__ == "__main__":
    main()
