#!/usr/bin/env python3
"""
Slay the Spire Tools - Unified Command Line Interface

Main entry point for all STS RNG prediction tools.

Usage:
    uv run scripts/sts.py dashboard        # Start web dashboard (background)
    uv run scripts/sts.py dashboard --fg   # Start in foreground
    uv run scripts/sts.py state            # One-time state check
    uv run scripts/sts.py watch            # Continuous CLI monitoring
    uv run scripts/sts.py stop             # Stop running dashboard
    uv run scripts/sts.py restart          # Restart dashboard
"""

import argparse
import os
import signal
import subprocess
import sys
import time
import webbrowser
from pathlib import Path

# Project setup
PROJECT_ROOT = Path(__file__).parent.parent
os.chdir(PROJECT_ROOT)
sys.path.insert(0, str(PROJECT_ROOT))

# Constants
PID_FILE = Path("/tmp/sts_dashboard.pid")
DEFAULT_PORT = 8080
DEFAULT_HOST = "127.0.0.1"


# =============================================================================
# UTILITY FUNCTIONS
# =============================================================================

def get_running_pid() -> int | None:
    """Get PID of running dashboard server, or None if not running."""
    if not PID_FILE.exists():
        return None

    try:
        pid = int(PID_FILE.read_text().strip())
        # Check if process is actually running
        os.kill(pid, 0)
        return pid
    except (ValueError, ProcessLookupError, PermissionError):
        # Process not running or invalid PID
        PID_FILE.unlink(missing_ok=True)
        return None


def save_pid(pid: int) -> None:
    """Save PID to file."""
    PID_FILE.write_text(str(pid))


def kill_process(pid: int) -> bool:
    """Kill a process by PID. Returns True if successful."""
    try:
        os.kill(pid, signal.SIGTERM)
        # Wait for process to terminate
        for _ in range(10):
            time.sleep(0.1)
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                return True
        # Force kill if still running
        os.kill(pid, signal.SIGKILL)
        return True
    except ProcessLookupError:
        return True  # Already dead
    except PermissionError:
        return False


# =============================================================================
# DASHBOARD COMMANDS
# =============================================================================

def cmd_dashboard(args: argparse.Namespace) -> int:
    """Start the web dashboard server."""
    port = args.port
    foreground = args.foreground

    # Check if already running
    existing_pid = get_running_pid()
    if existing_pid:
        print(f"Dashboard already running (PID {existing_pid})")
        print(f"Use 'sts.py stop' to stop it, or 'sts.py restart' to restart")
        return 1

    url = f"http://{DEFAULT_HOST}:{port}"

    if foreground:
        # Run in foreground
        print(f"Starting dashboard at {url}")
        print("Press Ctrl+C to stop")
        print()

        try:
            import uvicorn
            from web.server import app

            # Open browser after a short delay
            if not args.no_browser:
                import threading
                def open_browser():
                    time.sleep(1.5)
                    webbrowser.open(url)
                threading.Thread(target=open_browser, daemon=True).start()

            uvicorn.run(app, host=DEFAULT_HOST, port=port, log_level="warning")
        except KeyboardInterrupt:
            print("\nStopped.")
        return 0

    else:
        # Run in background
        print(f"Starting dashboard in background at {url}")

        # Launch as background process
        cmd = [
            sys.executable, "-c",
            f"""
import sys
sys.path.insert(0, '{PROJECT_ROOT}')
import uvicorn
from web.server import app
uvicorn.run(app, host='{DEFAULT_HOST}', port={port}, log_level='warning')
"""
        ]

        # Start process detached
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )

        # Save PID
        save_pid(proc.pid)

        # Wait a moment and verify it started
        time.sleep(1.0)
        if get_running_pid() is None:
            print("Failed to start dashboard server")
            return 1

        print(f"Dashboard started (PID {proc.pid})")

        # Open browser
        if not args.no_browser:
            time.sleep(0.5)
            webbrowser.open(url)
            print(f"Opened {url} in browser")

        print(f"\nUse 'uv run scripts/sts.py stop' to stop the server")
        return 0


def cmd_stop(args: argparse.Namespace) -> int:
    """Stop the running dashboard server."""
    pid = get_running_pid()

    if pid is None:
        print("No dashboard server running")
        return 0

    print(f"Stopping dashboard (PID {pid})...")

    if kill_process(pid):
        PID_FILE.unlink(missing_ok=True)
        print("Dashboard stopped")
        return 0
    else:
        print(f"Failed to stop process {pid}")
        return 1


def cmd_restart(args: argparse.Namespace) -> int:
    """Restart the dashboard server."""
    # Stop if running
    pid = get_running_pid()
    if pid:
        print(f"Stopping existing dashboard (PID {pid})...")
        kill_process(pid)
        PID_FILE.unlink(missing_ok=True)
        time.sleep(0.5)

    # Start new instance
    return cmd_dashboard(args)


# =============================================================================
# STATE/WATCH COMMANDS
# =============================================================================

def cmd_state(args: argparse.Namespace) -> int:
    """One-time state check with predictions."""
    from core.comparison.full_rng_tracker import (
        read_save_file, SAVE_PATH, predict_boss_relics, CLASS_STARTER_RELICS
    )
    from core.state.rng import long_to_seed, Random
    from core.state.game_rng import GameRNGState, RNGStream
    from core.generation.map import MapGenerator, MapGeneratorConfig, get_map_seed_offset
    from core.generation.rewards import generate_card_rewards, RewardState

    try:
        save = read_save_file()
    except FileNotFoundError:
        print(f"No save file found at: {SAVE_PATH}")
        print("Start a Slay the Spire run to generate a save file.")
        return 1
    except Exception as e:
        print(f"Error reading save file: {e}")
        return 1

    # Extract save data
    seed_long = save.get('seed', 0)
    seed_str = long_to_seed(seed_long)
    floor = save.get('floor_num', 0)
    act = save.get('act_num', 1)
    hp = save.get('current_health', 0)
    max_hp = save.get('max_health', 0)
    gold = save.get('gold', 0)

    # RNG state
    card_counter = save.get('card_seed_count', 0)
    blizzard = save.get('card_random_seed_randomizer', 5)
    relic_counter = save.get('relic_seed_count', 0)
    potion_counter = save.get('potion_seed_count', 0)
    monster_counter = save.get('monster_seed_count', 0)
    event_counter = save.get('event_seed_count', 0)

    # Collections
    relics = save.get('relics', [])
    path = [p for p in save.get('metric_path_per_floor', []) if p]
    room_x = save.get('room_x', 0)
    room_y = save.get('room_y', 0)

    # Print header
    print()
    print("=" * 70)
    print(f"SEED: {seed_str} | Floor {floor} | Act {act}")
    print("=" * 70)

    # Basic stats
    print(f"HP: {hp}/{max_hp} | Gold: {gold}")
    print(f"Position: ({room_x}, {room_y})")
    print()

    # RNG streams
    print("-" * 70)
    print("RNG STREAMS")
    print("-" * 70)
    print(f"  cardRng:    {card_counter:4d} (blizzard: {blizzard})")
    print(f"  relicRng:   {relic_counter:4d}")
    print(f"  potionRng:  {potion_counter:4d}")
    print(f"  monsterRng: {monster_counter:4d}")
    print(f"  eventRng:   {event_counter:4d}")
    print()

    # Relics
    print("-" * 70)
    print("RELICS")
    print("-" * 70)
    print(f"  {', '.join(relics) if relics else '(none)'}")
    print()

    # Path taken
    print("-" * 70)
    print("PATH")
    print("-" * 70)
    print(f"  {' '.join(path) if path else '(starting)'}")
    print()

    # Map and next node predictions
    try:
        map_seed = seed_long + get_map_seed_offset(act)
        config = MapGeneratorConfig(ascension_level=20)
        rng = Random(map_seed)
        gen = MapGenerator(rng, config)
        dungeon_map = gen.generate()

        # Build node lookup
        node_lookup = {}
        for row in dungeon_map:
            for node in row:
                if node:
                    node_lookup[(node.x, node.y)] = node

        current_node = node_lookup.get((room_x, room_y))

        # Pre-generated lists
        monster_list = save.get('monster_list', [])
        event_list = save.get('event_list', [])
        elite_list = save.get('elite_monster_list', [])

        # Count consumption
        monsters_seen = sum(1 for p in path if p == 'M')
        events_seen = sum(1 for p in path if p == '?')
        elites_seen = sum(1 for p in path if p == 'E')

        print("-" * 70)
        print("NEXT NODE OPTIONS")
        print("-" * 70)

        def predict_reward(room_type: str):
            state = GameRNGState(seed_str)
            state.set_counter(RNGStream.CARD, card_counter)
            reward_state = RewardState()
            reward_state.card_blizzard.offset = blizzard
            card_rng = state.get_rng(RNGStream.CARD)
            cards = generate_card_rewards(
                rng=card_rng, reward_state=reward_state, act=act,
                player_class='WATCHER', ascension=20,
                room_type='elite' if room_type == 'E' else 'normal', num_cards=3,
            )
            return [c.name for c in cards]

        if current_node and current_node.edges:
            for edge in sorted(current_node.edges, key=lambda e: e.dst_x):
                next_node = node_lookup.get((edge.dst_x, edge.dst_y))
                if next_node:
                    sym = next_node.room_type.value if next_node.room_type else '?'
                    print()
                    print(f"  [{edge.dst_x},{edge.dst_y}] {sym}", end="")

                    if sym == 'M':
                        monster = monster_list[monsters_seen] if monsters_seen < len(monster_list) else '???'
                        reward = predict_reward('M')
                        print(f" - {monster}")
                        print(f"      Cards: {reward}")
                    elif sym == 'E':
                        elite = elite_list[elites_seen] if elites_seen < len(elite_list) else '???'
                        reward = predict_reward('E')
                        print(f" - ELITE: {elite}")
                        print(f"      Cards: {reward}")
                    elif sym == '?':
                        event = event_list[events_seen] if events_seen < len(event_list) else '???'
                        print(f" - EVENT: {event}")
                    elif sym == '$':
                        print(" - SHOP")
                    elif sym == 'R':
                        print(" - REST")
                    elif sym == 'T':
                        print(" - TREASURE")
                    else:
                        print()
        else:
            # At boss floor or no edges
            if floor >= 15:
                boss = save.get('boss', 'Unknown')
                print(f"\n  BOSS: {boss}")

                has_starter = any(r in relics for r in CLASS_STARTER_RELICS.values())
                boss_picked = len(save.get('metric_boss_relics', []))
                boss_pred = predict_boss_relics(seed_long, 'WATCHER', has_starter, boss_picked, relics)
                print(f"  Boss Relics: {boss_pred}")

        print()
    except Exception as e:
        print(f"  (Map generation error: {e})")
        print()

    return 0


def cmd_watch(args: argparse.Namespace) -> int:
    """Continuous monitoring mode."""
    from core.comparison.full_rng_tracker import SAVE_PATH

    interval = args.interval
    print(f"Watching for save file changes every {interval}s...")
    print("Press Ctrl+C to stop")
    print()

    last_mtime = 0

    try:
        while True:
            try:
                mtime = os.path.getmtime(SAVE_PATH)
                if mtime != last_mtime:
                    last_mtime = mtime
                    os.system('clear')
                    print(f"[Updated at {time.strftime('%H:%M:%S')}]")
                    cmd_state(args)
            except FileNotFoundError:
                if last_mtime != -1:
                    os.system('clear')
                    print("Waiting for save file...")
                    print(f"Expected: {SAVE_PATH}")
                    last_mtime = -1

            time.sleep(interval)
    except KeyboardInterrupt:
        print("\nStopped.")

    return 0


# =============================================================================
# MAIN
# =============================================================================

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Slay the Spire Tools - Unified CLI",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  uv run scripts/sts.py dashboard         Start web dashboard (background)
  uv run scripts/sts.py dashboard --fg    Start dashboard in foreground
  uv run scripts/sts.py state             One-time state check
  uv run scripts/sts.py watch             Continuous monitoring
  uv run scripts/sts.py watch -i 5        Watch with 5 second interval
  uv run scripts/sts.py stop              Stop running dashboard
  uv run scripts/sts.py restart           Restart the dashboard
"""
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # dashboard command
    dashboard_parser = subparsers.add_parser(
        "dashboard",
        help="Start the web dashboard server"
    )
    dashboard_parser.add_argument(
        "--port", "-p",
        type=int,
        default=DEFAULT_PORT,
        help=f"Port to run server on (default: {DEFAULT_PORT})"
    )
    dashboard_parser.add_argument(
        "--foreground", "--fg", "-f",
        action="store_true",
        help="Run in foreground instead of background"
    )
    dashboard_parser.add_argument(
        "--no-browser",
        action="store_true",
        help="Don't auto-open browser"
    )

    # stop command
    subparsers.add_parser(
        "stop",
        help="Stop the running dashboard server"
    )

    # restart command
    restart_parser = subparsers.add_parser(
        "restart",
        help="Restart the dashboard server"
    )
    restart_parser.add_argument(
        "--port", "-p",
        type=int,
        default=DEFAULT_PORT,
        help=f"Port to run server on (default: {DEFAULT_PORT})"
    )
    restart_parser.add_argument(
        "--no-browser",
        action="store_true",
        help="Don't auto-open browser"
    )

    # state command
    subparsers.add_parser(
        "state",
        help="One-time game state check with predictions"
    )

    # watch command
    watch_parser = subparsers.add_parser(
        "watch",
        help="Continuous CLI monitoring"
    )
    watch_parser.add_argument(
        "--interval", "-i",
        type=int,
        default=10,
        help="Check interval in seconds (default: 10)"
    )

    args = parser.parse_args()

    if args.command is None:
        parser.print_help()
        return 0

    # Dispatch to command handlers
    if args.command == "dashboard":
        return cmd_dashboard(args)
    elif args.command == "stop":
        return cmd_stop(args)
    elif args.command == "restart":
        # Add default flags for restart
        if not hasattr(args, 'foreground'):
            args.foreground = False
        return cmd_restart(args)
    elif args.command == "state":
        return cmd_state(args)
    elif args.command == "watch":
        return cmd_watch(args)
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
