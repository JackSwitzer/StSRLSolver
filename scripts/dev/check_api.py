#!/usr/bin/env python3
"""
Check STS Dashboard API State

Enhanced developer tool for checking API responses from the STS dashboard server.
Provides detailed diagnostics about what data the server returns.

Usage:
    uv run scripts/dev/check_api.py              # Summary view
    uv run scripts/dev/check_api.py keys         # List all keys
    uv run scripts/dev/check_api.py neow         # Show Neow options
    uv run scripts/dev/check_api.py bosses       # Show boss predictions
    uv run scripts/dev/check_api.py rng          # Show RNG accuracy
    uv run scripts/dev/check_api.py raw          # Full JSON output
    uv run scripts/dev/check_api.py diagnose     # Full diagnostic check
"""

import json
import sys
import urllib.request
from typing import Any, Dict, Optional

API_URL = "http://127.0.0.1:8080/api/state"
EXPECTED_KEYS = [
    "seed", "run", "player", "rng", "player_class", "deck", "deck_count",
    "relics", "potions", "path_taken", "current_position", "map",
    "accessible_nodes", "node_predictions", "boss_relics", "boss_relics_per_act",
    "boss_card_reward", "card_choices", "floor_history", "monster_list",
    "elite_monster_list", "event_list", "boss", "predicted_bosses",
    "future_acts", "shop_prediction", "treasure_prediction", "path_tree",
    "rng_accuracy", "neow_options", "timestamp"
]


def fetch_state() -> Dict[str, Any]:
    """Fetch current state from API."""
    try:
        with urllib.request.urlopen(API_URL, timeout=5) as response:
            return json.loads(response.read().decode())
    except urllib.error.URLError as e:
        return {"error": f"Connection failed: {e.reason}", "server_down": True}
    except Exception as e:
        return {"error": str(e)}


def print_summary(state: Dict[str, Any]) -> None:
    """Print a summary of the game state."""
    if "error" in state:
        print(f"ERROR: {state['error']}")
        if state.get("server_down"):
            print("\nServer is not running. Start it with:")
            print("  uv run scripts/sts.py dashboard")
        return

    # Basic info
    seed = state.get("seed", {}).get("string", "?")
    run = state.get("run", {})
    floor = run.get("floor", 0)
    act = run.get("act", 1)
    ascension = run.get("ascension", 0)
    player = state.get("player", {})
    hp = player.get("hp", 0)
    max_hp = player.get("max_hp", 0)
    gold = player.get("gold", 0)
    player_class = state.get("player_class", "?")

    print(f"Seed: {seed} | {player_class} A{ascension}")
    print(f"Floor {floor} | Act {act} | HP: {hp}/{max_hp} | Gold: {gold}")
    print()

    # Neow options (if at floor 0)
    neow = state.get("neow_options", [])
    if floor == 0 and neow:
        print("NEOW OPTIONS:")
        for i, opt in enumerate(neow):
            if isinstance(opt, dict):
                name = opt.get("name", "?")
                drawback = opt.get("drawback")
                if drawback:
                    print(f"  {i+1}. {name} ({drawback})")
                else:
                    print(f"  {i+1}. {name}")
            else:
                print(f"  {i+1}. {opt}")
        print()

    # Predicted bosses
    bosses = state.get("predicted_bosses", {})
    if bosses:
        print("PREDICTED BOSSES:")
        for act_num in sorted(bosses.keys(), key=lambda x: int(x) if isinstance(x, (int, str)) and str(x).isdigit() else 0):
            boss_list = bosses.get(act_num, [])
            if boss_list:
                boss_str = " + ".join(boss_list) if isinstance(boss_list, list) else str(boss_list)
                marker = " <--" if int(act_num) == act else ""
                print(f"  Act {act_num}: {boss_str}{marker}")
        print()

    # RNG accuracy
    rng_acc = state.get("rng_accuracy", {})
    if rng_acc and "overall" in rng_acc:
        overall = rng_acc["overall"]
        ratio = overall.get("ratio", 0) * 100
        correct = overall.get("correct", 0)
        total = overall.get("total", 0)
        print(f"RNG Accuracy: {ratio:.0f}% ({correct}/{total} predictions correct)")

        # Show card accuracy
        cards = rng_acc.get("cards", {})
        if cards.get("total", 0) > 0:
            card_ratio = cards.get("ratio", 0) * 100
            print(f"  Cards: {card_ratio:.0f}% ({cards['correct']}/{cards['total']})")

        # Show recent mismatches
        mismatches = rng_acc.get("mismatches", [])
        if mismatches:
            print(f"  Recent mismatches: {len(mismatches)}")
        print()

    # Relics
    relics = state.get("relics", [])
    if relics:
        print(f"RELICS ({len(relics)}): {', '.join(relics)}")
        print()

    # Path taken
    path = state.get("path_taken", [])
    if path:
        print(f"PATH: {' '.join(path)}")
        print()

    # Current accessible nodes
    accessible = state.get("accessible_nodes", [])
    if accessible:
        print(f"NEXT NODES ({len(accessible)}):")
        for node in accessible:
            room = node.get("room_type", "?")
            x, y = node.get("x", "?"), node.get("y", "?")
            print(f"  [{x},{y}] {room}")


def print_keys(state: Dict[str, Any]) -> None:
    """Print all keys in the state with types and counts."""
    if "error" in state:
        print(f"ERROR: {state['error']}")
        return

    print(f"API Response Keys ({len(state)} total):")
    for key in sorted(state.keys()):
        val = state[key]
        if isinstance(val, list):
            if val and isinstance(val[0], dict):
                print(f"  {key}: list[{len(val)}] of dicts")
            else:
                print(f"  {key}: list[{len(val)}]")
        elif isinstance(val, dict):
            subkeys = ", ".join(sorted(val.keys())[:3])
            if len(val) > 3:
                subkeys += ", ..."
            print(f"  {key}: dict[{len(val)}] ({subkeys})")
        elif val is None:
            print(f"  {key}: null")
        else:
            print(f"  {key}: {type(val).__name__}")


def print_neow(state: Dict[str, Any]) -> None:
    """Print Neow options in detail."""
    if "error" in state:
        print(f"ERROR: {state['error']}")
        return

    floor = state.get("run", {}).get("floor", -1)
    neow = state.get("neow_options", [])

    print(f"Floor: {floor}")
    print(f"Has neow_options in response: {'neow_options' in state}")

    if floor != 0:
        print("\nNot at floor 0 - Neow options only available at game start")
        return

    if neow:
        print(f"\nNEOW OPTIONS ({len(neow)}):")
        for i, opt in enumerate(neow):
            print(f"\n  Option {i+1}:")
            if isinstance(opt, dict):
                for k, v in opt.items():
                    print(f"    {k}: {v}")
            else:
                print(f"    {opt}")
    else:
        print("\nNo Neow options predicted (check if predict_neow_options is working)")


def print_bosses(state: Dict[str, Any]) -> None:
    """Print boss predictions in detail."""
    if "error" in state:
        print(f"ERROR: {state['error']}")
        return

    act = state.get("run", {}).get("act", 1)
    current_boss = state.get("boss", "Unknown")
    predicted = state.get("predicted_bosses", {})
    per_act = state.get("boss_relics_per_act", {})

    print(f"Current Act: {act}")
    print(f"Current Boss (from save): {current_boss}")
    print()

    print("PREDICTED BOSSES:")
    if predicted:
        for act_num in sorted(predicted.keys(), key=lambda x: int(x) if str(x).isdigit() else 0):
            bosses = predicted[act_num]
            marker = " <-- CURRENT" if int(act_num) == act else ""
            if isinstance(bosses, list):
                print(f"  Act {act_num}: {' + '.join(bosses)}{marker}")
            else:
                print(f"  Act {act_num}: {bosses}{marker}")
    else:
        print("  No boss predictions available")
    print()

    print("BOSS RELICS PER ACT:")
    if per_act:
        for act_num in sorted(per_act.keys(), key=lambda x: int(x) if str(x).isdigit() else 0):
            relics = per_act[act_num]
            if relics:
                print(f"  Act {act_num}: {', '.join(relics)}")
    else:
        print("  No boss relic predictions available")


def print_rng(state: Dict[str, Any]) -> None:
    """Print RNG accuracy in detail."""
    if "error" in state:
        print(f"ERROR: {state['error']}")
        return

    rng = state.get("rng", {})
    rng_acc = state.get("rng_accuracy", {})

    print("RNG COUNTERS:")
    for key in ["card_counter", "card_blizzard", "relic_counter", "potion_counter",
                "monster_counter", "event_counter", "merchant_counter", "treasure_counter"]:
        value = rng.get(key, "N/A")
        print(f"  {key}: {value}")
    print()

    if not rng_acc:
        print("No RNG accuracy data available")
        return

    print("RNG ACCURACY:")
    overall = rng_acc.get("overall", {})
    print(f"  Overall: {overall.get('ratio', 0)*100:.1f}% ({overall.get('correct', 0)}/{overall.get('total', 0)})")

    for category in ["cards", "relics", "potions"]:
        cat_data = rng_acc.get(category, {})
        if cat_data.get("total", 0) > 0:
            print(f"  {category.title()}: {cat_data.get('ratio', 0)*100:.1f}% ({cat_data.get('correct', 0)}/{cat_data.get('total', 0)})")
    print()

    mismatches = rng_acc.get("mismatches", [])
    if mismatches:
        print(f"RECENT MISMATCHES ({len(mismatches)}):")
        for m in mismatches[-5:]:
            floor = m.get("floor", "?")
            mtype = m.get("type", "?")
            if "error" in m:
                print(f"  Floor {floor} ({mtype}): ERROR - {m['error']}")
            else:
                predicted = m.get("predicted", [])
                actual = m.get("actual", [])
                print(f"  Floor {floor} ({mtype}):")
                print(f"    Predicted: {predicted}")
                print(f"    Actual: {actual}")


def diagnose(state: Dict[str, Any]) -> None:
    """Run full diagnostics on API response."""
    if "error" in state:
        print(f"ERROR: {state['error']}")
        if state.get("server_down"):
            print("\nDIAGNOSIS: Server is not running")
            print("FIX: Run 'uv run scripts/sts.py dashboard' to start")
        return

    print("=" * 60)
    print("STS DASHBOARD API DIAGNOSTIC REPORT")
    print("=" * 60)
    print()

    # Check keys
    returned_keys = set(state.keys())
    expected_set = set(EXPECTED_KEYS)
    missing = expected_set - returned_keys
    extra = returned_keys - expected_set

    print(f"KEYS: {len(returned_keys)} returned, {len(EXPECTED_KEYS)} expected")
    if missing:
        print(f"  MISSING ({len(missing)}): {', '.join(sorted(missing))}")
    if extra:
        print(f"  EXTRA ({len(extra)}): {', '.join(sorted(extra))}")
    if not missing and not extra:
        print("  All expected keys present")
    print()

    # Check critical features
    print("FEATURE AVAILABILITY:")
    features = {
        "Neow Options": bool(state.get("neow_options")),
        "Boss Predictions": bool(state.get("predicted_bosses")),
        "Boss Relics per Act": bool(state.get("boss_relics_per_act")),
        "RNG Accuracy": bool(state.get("rng_accuracy")),
        "Path Tree": bool(state.get("path_tree")),
        "Future Acts": bool(state.get("future_acts")),
        "Floor History": bool(state.get("floor_history")),
        "Shop Prediction": state.get("shop_prediction") is not None,
        "Treasure Prediction": state.get("treasure_prediction") is not None,
    }
    for feature, available in features.items():
        status = "OK" if available else "N/A"
        print(f"  [{status}] {feature}")
    print()

    # Check RNG counters
    rng = state.get("rng", {})
    print("RNG COUNTERS:")
    expected_counters = ["card_counter", "card_blizzard", "relic_counter", "potion_counter",
                        "monster_counter", "event_counter", "merchant_counter", "treasure_counter", "blizzard_mod"]
    for counter in expected_counters:
        value = rng.get(counter, "MISSING")
        status = "OK" if counter in rng else "MISSING"
        print(f"  [{status}] {counter}: {value}")
    print()

    # Summary
    print("SUMMARY:")
    if missing:
        print(f"  WARNING: {len(missing)} expected keys are missing!")
        print("  This may indicate the server is running an old version.")
        print("  FIX: Run 'uv run scripts/dev/restart_server.py'")
    else:
        print("  API is returning all expected data")

    # Floor-specific check
    floor = state.get("run", {}).get("floor", -1)
    if floor == 0:
        neow = state.get("neow_options", [])
        if neow:
            print(f"  Floor 0: Neow options available ({len(neow)} options)")
        else:
            print("  WARNING: At floor 0 but no Neow options!")


def main() -> int:
    state = fetch_state()

    if len(sys.argv) > 1:
        cmd = sys.argv[1].lower()
        if cmd == "keys":
            print_keys(state)
        elif cmd == "neow":
            print_neow(state)
        elif cmd == "bosses":
            print_bosses(state)
        elif cmd == "rng":
            print_rng(state)
        elif cmd == "raw":
            if "error" in state:
                print(f"ERROR: {state['error']}")
                return 1
            print(json.dumps(state, indent=2))
        elif cmd == "diagnose":
            diagnose(state)
        elif cmd in ["-h", "--help", "help"]:
            print(__doc__)
        else:
            print(f"Unknown command: {cmd}")
            print("Usage: check_api.py [keys|neow|bosses|rng|raw|diagnose]")
            return 1
    else:
        print_summary(state)

    return 0 if "error" not in state else 1


if __name__ == "__main__":
    sys.exit(main())
