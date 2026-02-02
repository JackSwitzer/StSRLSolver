#!/usr/bin/env python3
"""
Parity test - compare Python predictions vs game save file.

Usage:
    # Test against current save file
    uv run scripts/dev/test_parity.py

    # Generate predictions for a seed (to compare manually)
    uv run scripts/dev/test_parity.py --seed "1234567890" --character WATCHER
"""
import argparse
import base64
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from core.generation.encounters import predict_all_acts, predict_all_bosses_extended

SAVE_DIR = Path.home() / "Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/saves"
XOR_KEY = b"key"


def decrypt_save(data: bytes) -> dict:
    """Decrypt XOR-encrypted save file."""
    decrypted = bytes([data[i] ^ XOR_KEY[i % len(XOR_KEY)] for i in range(len(data))])
    return json.loads(decrypted.decode('utf-8'))


def load_save(character: str = "WATCHER") -> dict | None:
    """Load save file for character."""
    save_path = SAVE_DIR / f"{character}.autosave"
    if not save_path.exists():
        return None
    with open(save_path, "rb") as f:
        return decrypt_save(base64.b64decode(f.read()))


def predict_for_seed(seed: str, character: str = "WATCHER", ascension: int = 20):
    """Generate predictions for a seed."""
    print(f"\n{'='*60}")
    print(f"PREDICTIONS FOR SEED: {seed}")
    print(f"Character: {character} | Ascension: {ascension}")
    print(f"{'='*60}")

    # Get all act predictions
    all_acts = predict_all_acts(str(seed), include_act4=True)

    for act_num in [1, 2, 3]:
        act_data = all_acts[f"act{act_num}"]
        monsters = act_data.get("monsters", [])
        elites = act_data.get("elites", [])
        boss = act_data.get("boss", "Unknown")

        print(f"\n--- ACT {act_num} MONSTERS ---")
        for i, m in enumerate(monsters[:10], 1):
            print(f"  {i:2}. {m}")

        print(f"\n--- ACT {act_num} ELITES ---")
        for i, e in enumerate(elites[:5], 1):
            print(f"  {i}. {e}")

        print(f"\n--- ACT {act_num} BOSS ---")
        print(f"  {boss}")

    # All bosses (A20 extended)
    print(f"\n--- ALL BOSSES (A{ascension}) ---")
    bosses = predict_all_bosses_extended(str(seed), ascension=ascension)
    for act_num, boss_list in sorted(bosses.items()):
        if isinstance(boss_list, list):
            boss_str = " + ".join(boss_list)
        else:
            boss_str = str(boss_list)
        print(f"  Act {act_num}: {boss_str}")


def compare_with_save(save: dict):
    """Compare predictions with save file data."""
    seed = save['seed']
    ascension = save.get('ascension_level', 0)

    print(f"\n{'='*60}")
    print(f"PARITY CHECK - SEED: {seed}")
    print(f"Floor: {save['floor_num']} | Act: {save['level_name']}")
    print(f"Ascension: {ascension}")
    print(f"{'='*60}")

    # Determine current act
    act_map = {"Exordium": 1, "TheCity": 2, "TheBeyond": 3, "TheEnding": 4}
    current_act = act_map.get(save['level_name'], 1)

    # Generate predictions for current act
    all_acts = predict_all_acts(str(seed), include_act4=True)
    act_key = f"act{current_act}"

    if act_key in all_acts:
        act_data = all_acts[act_key]
        predicted_monsters = act_data.get("monsters", [])
        predicted_elites = act_data.get("elites", [])
        predicted_boss = act_data.get("boss", "")
    else:
        predicted_monsters = []
        predicted_elites = []
        predicted_boss = ""

    # Get actual from save
    actual_monsters = save.get('monster_list', [])
    actual_elites = save.get('elite_list', save.get('elite_monster_list', []))
    actual_boss = save.get('boss', '')

    # Compare monsters - account for already-fought monsters
    # The save file's monster_list is the REMAINING queue, not the full list
    # Find the offset by checking where actual matches as a suffix of predicted
    print(f"\n--- MONSTER LIST COMPARISON ---")

    fought_count = len(predicted_monsters) - len(actual_monsters)
    if fought_count < 0:
        fought_count = 0

    # Check if remaining actual matches predicted at the offset
    matches = 0
    total = len(actual_monsters)
    all_match = True

    print(f"Monsters fought so far: {fought_count}")
    print(f"{'#':<3} {'Predicted':<25} {'Actual (remaining)':<25} {'Match'}")
    print("-" * 65)

    for i in range(min(total, 13)):
        pred_idx = fought_count + i
        pred = predicted_monsters[pred_idx] if pred_idx < len(predicted_monsters) else "???"
        act = actual_monsters[i] if i < len(actual_monsters) else "???"
        match = "✓" if pred == act else "✗"
        if pred == act:
            matches += 1
        else:
            all_match = False
        print(f"{pred_idx+1:<3} {pred:<25} {act:<25} {match}")

    if total > 0:
        print(f"\nMonster Match Rate: {matches}/{min(total,13)} ({matches/min(total,13)*100:.1f}%)")
    else:
        print(f"\nNo monsters remaining in queue")

    # Compare elites - account for already-fought elites
    print(f"\n--- ELITE LIST COMPARISON ---")

    elite_fought = len(predicted_elites) - len(actual_elites)
    if elite_fought < 0:
        elite_fought = 0

    elite_matches = 0
    elite_total = len(actual_elites)

    print(f"Elites fought so far: {elite_fought}")
    print(f"{'#':<3} {'Predicted':<25} {'Actual (remaining)':<25} {'Match'}")
    print("-" * 65)

    for i in range(min(elite_total, 10)):
        pred_idx = elite_fought + i
        pred = predicted_elites[pred_idx] if pred_idx < len(predicted_elites) else "???"
        act = actual_elites[i] if i < len(actual_elites) else "???"
        match = "✓" if pred == act else "✗"
        if pred == act:
            elite_matches += 1
        print(f"{pred_idx+1}. {pred:<25} {act:<25} {match}")

    if elite_total > 0:
        print(f"\nElite Match Rate: {elite_matches}/{min(elite_total,10)} ({elite_matches/min(elite_total,10)*100:.1f}%)")
    else:
        print(f"\nNo elites remaining in queue")

    # Compare boss
    print(f"\n--- BOSS COMPARISON ---")
    boss_match = "✓" if predicted_boss == actual_boss else "✗"
    print(f"Predicted: {predicted_boss}")
    print(f"Actual:    {actual_boss}")
    print(f"Match:     {boss_match}")

    # RNG counters
    print(f"\n--- RNG COUNTERS (from save) ---")
    print(f"  card_seed_count:    {save.get('card_seed_count', 'N/A')}")
    print(f"  monster_seed_count: {save.get('monster_seed_count', 'N/A')}")
    print(f"  relic_seed_count:   {save.get('relic_seed_count', 'N/A')}")
    print(f"  potion_seed_count:  {save.get('potion_seed_count', 'N/A')}")
    print(f"  event_seed_count:   {save.get('event_seed_count', 'N/A')}")

    # Summary
    monster_checked = min(total, 13)
    elite_checked = min(elite_total, 10)
    total_checks = monster_checked + elite_checked + 1
    total_matches = matches + elite_matches + (1 if predicted_boss == actual_boss else 0)

    print(f"\n{'='*60}")
    if total_checks > 0:
        pct = total_matches/total_checks*100
        print(f"OVERALL PARITY: {total_matches}/{total_checks} ({pct:.1f}%)")
        if pct == 100:
            print("STATUS: PERFECT PARITY ✓")
        else:
            print("STATUS: MISMATCH DETECTED ✗")
    else:
        print("OVERALL PARITY: No data to compare")
    print(f"{'='*60}")

    return total_matches == total_checks


def main():
    parser = argparse.ArgumentParser(description="Test Python vs Java parity")
    parser.add_argument("--seed", type=str, help="Seed to test (alphanumeric or numeric)")
    parser.add_argument("--character", type=str, default="WATCHER", help="Character class")
    parser.add_argument("--ascension", type=int, default=20, help="Ascension level")
    args = parser.parse_args()

    if args.seed:
        # Generate predictions for given seed
        predict_for_seed(args.seed, args.character, args.ascension)
    else:
        # Compare with current save file
        save = load_save(args.character)
        if save:
            success = compare_with_save(save)
            sys.exit(0 if success else 1)
        else:
            print(f"No save file found for {args.character}")
            print(f"Looking in: {SAVE_DIR}")
            print("\nTo generate predictions for a specific seed:")
            print(f"  uv run {__file__} --seed YOUR_SEED")
            sys.exit(1)


if __name__ == "__main__":
    main()
