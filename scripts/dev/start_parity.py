#!/usr/bin/env python3
"""
Parity Testing Companion Script

Handles:
1. Generating predictions for a seed
2. Optionally sending commands to CommunicationMod to start run
3. Monitoring save file and auto-running parity check

Usage:
    # Generate predictions and print instructions
    uv run scripts/dev/start_parity.py --seed TEST123

    # Generate predictions and watch for save file
    uv run scripts/dev/start_parity.py --seed TEST123 --watch

    # Start run via CommunicationMod (requires running bot)
    uv run scripts/dev/start_parity.py --seed TEST123 --comm-mod

    # Full automated test
    uv run scripts/dev/start_parity.py --seed TEST123 --launch --watch
"""

import argparse
import base64
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from datetime import datetime

# Add project root to path
PROJECT_ROOT = Path(__file__).parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT))

# Constants
GAME_DIR = Path.home() / "Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
SAVE_DIR = GAME_DIR / "saves"
COMM_MOD_CONFIG = Path.home() / "Library/Preferences/ModTheSpire/CommunicationMod/config.properties"
XOR_KEY = b"key"

# Import prediction modules
try:
    from core.generation.encounters import predict_all_acts, predict_all_bosses_extended
except ImportError:
    predict_all_acts = None
    predict_all_bosses_extended = None


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


def generate_predictions(seed: str, character: str = "WATCHER", ascension: int = 20) -> dict:
    """Generate all predictions for a seed."""
    if predict_all_acts is None:
        print("Warning: Core modules not available, running via subprocess", file=sys.stderr)
        result = subprocess.run(
            ["uv", "run", str(PROJECT_ROOT / "scripts/dev/test_parity.py"),
             "--seed", seed, "--character", character, "--ascension", str(ascension)],
            capture_output=True,
            text=True,
            cwd=str(PROJECT_ROOT)
        )
        print(result.stdout)
        if result.stderr:
            print(result.stderr, file=sys.stderr)
        return {}

    predictions = {
        "seed": seed,
        "character": character,
        "ascension": ascension,
        "timestamp": datetime.now().isoformat(),
    }

    # Get all act predictions
    all_acts = predict_all_acts(str(seed), include_act4=True)
    predictions["acts"] = all_acts

    # Get boss predictions
    bosses = predict_all_bosses_extended(str(seed), ascension=ascension)
    predictions["bosses"] = bosses

    return predictions


def print_predictions(predictions: dict):
    """Print predictions in readable format."""
    print("=" * 60)
    print(f"PREDICTIONS FOR SEED: {predictions.get('seed', 'Unknown')}")
    print(f"Character: {predictions.get('character', 'WATCHER')} | Ascension: {predictions.get('ascension', 20)}")
    print("=" * 60)

    acts = predictions.get("acts", {})
    for act_num in [1, 2, 3]:
        act_key = f"act{act_num}"
        if act_key not in acts:
            continue

        act_data = acts[act_key]
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

    # All bosses
    bosses = predictions.get("bosses", {})
    if bosses:
        print(f"\n--- ALL BOSSES (A{predictions.get('ascension', 20)}) ---")
        for act_num, boss_list in sorted(bosses.items()):
            if isinstance(boss_list, list):
                boss_str = " + ".join(boss_list)
            else:
                boss_str = str(boss_list)
            print(f"  Act {act_num}: {boss_str}")


def compare_with_save(save: dict, predictions: dict = None):
    """Compare predictions with save file data."""
    seed = save['seed']
    ascension = save.get('ascension_level', 0)
    character = save.get('name', 'UNKNOWN')

    print(f"\n{'='*60}")
    print(f"PARITY CHECK - SEED: {seed}")
    print(f"Floor: {save['floor_num']} | Act: {save['level_name']}")
    print(f"Character: {character} | Ascension: {ascension}")
    print(f"{'='*60}")

    # Determine current act
    act_map = {"Exordium": 1, "TheCity": 2, "TheBeyond": 3, "TheEnding": 4}
    current_act = act_map.get(save['level_name'], 1)

    # Generate predictions if not provided
    if predictions is None:
        all_acts = predict_all_acts(str(seed), include_act4=True) if predict_all_acts else {}
    else:
        all_acts = predictions.get("acts", {})

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

    # Compare monsters
    print(f"\n--- MONSTER LIST COMPARISON ---")
    print(f"{'#':<3} {'Predicted':<25} {'Actual':<25} {'Match'}")
    print("-" * 60)

    matches = 0
    total = min(len(predicted_monsters), len(actual_monsters), 13)
    for i in range(total):
        pred = predicted_monsters[i] if i < len(predicted_monsters) else "???"
        act = actual_monsters[i] if i < len(actual_monsters) else "???"
        match = "OK" if pred == act else "FAIL"
        if pred == act:
            matches += 1
        print(f"{i+1:<3} {pred:<25} {act:<25} {match}")

    if total > 0:
        print(f"\nMonster Match Rate: {matches}/{total} ({matches/total*100:.1f}%)")

    # Compare elites
    print(f"\n--- ELITE LIST COMPARISON ---")
    elite_matches = 0
    elite_total = min(len(predicted_elites), len(actual_elites), 10)
    for i in range(elite_total):
        pred = predicted_elites[i] if i < len(predicted_elites) else "???"
        act = actual_elites[i] if i < len(actual_elites) else "???"
        match = "OK" if pred == act else "FAIL"
        if pred == act:
            elite_matches += 1
        print(f"{i+1}. {pred:<25} {act:<25} {match}")

    if elite_total > 0:
        print(f"\nElite Match Rate: {elite_matches}/{elite_total} ({elite_matches/elite_total*100:.1f}%)")

    # Compare boss
    print(f"\n--- BOSS COMPARISON ---")
    boss_match = "OK" if predicted_boss == actual_boss else "FAIL"
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
    total_checks = total + elite_total + 1
    total_matches = matches + elite_matches + (1 if predicted_boss == actual_boss else 0)
    print(f"\n{'='*60}")
    print(f"OVERALL PARITY: {total_matches}/{total_checks} ({total_matches/total_checks*100:.1f}%)")
    print(f"{'='*60}")

    return total_matches == total_checks


def create_autostart_bot(seed: str, character: str, ascension: int) -> Path:
    """Create a temporary bot script for CommunicationMod auto-start."""
    bot_script = Path("/tmp/parity_autostart_bot.py")

    bot_code = f'''#!/usr/bin/env python3
"""Auto-start bot for parity testing via CommunicationMod."""
import json
import sys

SEED = "{seed}"
CHARACTER = "{character}"
ASCENSION = "{ascension}"

def main():
    print("ready", flush=True)
    started = False

    while True:
        try:
            line = sys.stdin.readline()
            if not line:
                break

            state = json.loads(line)

            if "error" in state:
                print("state", flush=True)
                continue

            in_game = state.get("in_game", False)
            game_state = state.get("game_state", {{}})

            # If not in game, start the run
            if not in_game and not started:
                cmd = f"start {{CHARACTER}} {{ASCENSION}} {{SEED}}"
                print(f"[PARITY-BOT] Starting: {{cmd}}", file=sys.stderr)
                print(cmd, flush=True)
                started = True
                continue

            # Check if we're at Neow (floor 0)
            floor = game_state.get("floor", -1)
            screen_type = game_state.get("screen_type", "")

            if floor == 0 and in_game:
                print("[PARITY-BOT] At floor 0 (Neow). Auto-start complete.", file=sys.stderr)
                print("[PARITY-BOT] Game is ready for parity testing.", file=sys.stderr)
                # Keep running to prevent game pause
                # User can now interact manually or run parity check

            # Default: request state update
            print("state", flush=True)

        except json.JSONDecodeError:
            print("state", flush=True)
            continue
        except Exception as e:
            print(f"[PARITY-BOT] Error: {{e}}", file=sys.stderr)
            print("state", flush=True)

if __name__ == "__main__":
    main()
'''
    bot_script.write_text(bot_code)
    bot_script.chmod(0o755)
    return bot_script


def configure_communication_mod(bot_script: Path):
    """Update CommunicationMod config to use the auto-start bot."""
    COMM_MOD_CONFIG.parent.mkdir(parents=True, exist_ok=True)

    # Backup existing config
    if COMM_MOD_CONFIG.exists():
        backup = COMM_MOD_CONFIG.with_suffix('.properties.bak')
        COMM_MOD_CONFIG.rename(backup)
        print(f"Backed up existing config to: {backup}")

    config_content = f"#Parity Test Auto-Start - {datetime.now().isoformat()}\ncommand=python3 {bot_script}\n"
    COMM_MOD_CONFIG.write_text(config_content)
    print(f"Updated CommunicationMod config: {COMM_MOD_CONFIG}")


def launch_sts():
    """Launch Slay the Spire via the launch script."""
    launch_script = PROJECT_ROOT / "scripts/dev/launch_sts.sh"
    if not launch_script.exists():
        print(f"Error: Launch script not found: {launch_script}", file=sys.stderr)
        return False

    print("Launching Slay the Spire...")
    subprocess.Popen([str(launch_script)], cwd=str(PROJECT_ROOT))
    return True


def watch_save_file(character: str, predictions: dict = None, callback=None):
    """Watch for save file changes and run parity check."""
    save_path = SAVE_DIR / f"{character}.autosave"
    print(f"Watching: {save_path}")
    print("Press Ctrl+C to stop")
    print()

    initial_mtime = save_path.stat().st_mtime if save_path.exists() else 0

    try:
        while True:
            if save_path.exists():
                current_mtime = save_path.stat().st_mtime
                if current_mtime != initial_mtime:
                    print(f"\n[{datetime.now().strftime('%H:%M:%S')}] Save file updated!")

                    # Load and check
                    save = load_save(character)
                    if save:
                        # Check if we're at floor 0 (Neow)
                        if save.get('floor_num', -1) == 0:
                            print("At Neow (floor 0) - Running parity check...")
                            success = compare_with_save(save, predictions)
                            if callback:
                                callback(success)
                        else:
                            print(f"At floor {save.get('floor_num')} - waiting for Neow...")

                    initial_mtime = current_mtime

            time.sleep(2)
    except KeyboardInterrupt:
        print("\nStopped watching.")


def print_manual_instructions(seed: str, character: str, ascension: int):
    """Print instructions for manual seed entry."""
    print()
    print("=" * 60)
    print("MANUAL START INSTRUCTIONS")
    print("=" * 60)
    print()
    print("Option 1 - Using console (requires BaseMod + EVTracker):")
    print("  1. Press ~ (tilde) to open console")
    print(f"  2. Type: evseed {seed}")
    print(f"  3. Start new run: {character}, Ascension {ascension}")
    print()
    print("Option 2 - Using seeded run option:")
    print("  1. Main Menu -> Standard -> Seeded")
    print(f"  2. Enter seed: {seed}")
    print(f"  3. Select: {character}, Ascension {ascension}")
    print()
    print("After starting, run parity check with:")
    print(f"  uv run scripts/dev/test_parity.py --character {character}")
    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(
        description="Parity Testing - Launch STS and verify predictions",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate predictions only
  %(prog)s --seed TEST123 --predictions-only

  # Generate predictions and watch for save file
  %(prog)s --seed TEST123 --watch

  # Full automated: configure CommunicationMod, launch, and watch
  %(prog)s --seed TEST123 --auto --launch --watch
        """
    )
    parser.add_argument("--seed", "-s", type=str, default="1234567890",
                        help="Seed to use (default: 1234567890)")
    parser.add_argument("--character", "-c", type=str, default="WATCHER",
                        help="Character class (default: WATCHER)")
    parser.add_argument("--ascension", "-a", type=int, default=20,
                        help="Ascension level (default: 20)")
    parser.add_argument("--auto", action="store_true",
                        help="Configure CommunicationMod for auto-start")
    parser.add_argument("--launch", "-l", action="store_true",
                        help="Launch Slay the Spire")
    parser.add_argument("--watch", "-w", action="store_true",
                        help="Watch for save file and run parity")
    parser.add_argument("--predictions-only", "-p", action="store_true",
                        help="Only generate and print predictions")
    parser.add_argument("--output", "-o", type=str,
                        help="Save predictions to JSON file")

    args = parser.parse_args()

    # Normalize character name
    character = args.character.upper()

    # Generate predictions
    print("Generating predictions...")
    predictions = generate_predictions(args.seed, character, args.ascension)

    if predictions:
        print_predictions(predictions)

        # Save to file if requested
        if args.output:
            output_path = Path(args.output)
            output_path.write_text(json.dumps(predictions, indent=2))
            print(f"\nPredictions saved to: {output_path}")

    if args.predictions_only:
        return 0

    # Configure auto-start if requested
    if args.auto:
        print("\nConfiguring auto-start...")
        bot_script = create_autostart_bot(args.seed, character, args.ascension)
        configure_communication_mod(bot_script)
        print(f"Bot script: {bot_script}")

    # Launch if requested
    if args.launch:
        print()
        if not launch_sts():
            return 1

    # Print manual instructions if not auto-starting
    if not args.auto:
        print_manual_instructions(args.seed, character, args.ascension)

    # Watch mode
    if args.watch:
        print("\nStarting watch mode...")
        watch_save_file(character, predictions)
    else:
        print("\nTo run parity check after starting:")
        print(f"  uv run scripts/dev/test_parity.py --character {character}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
