#!/usr/bin/env python3
"""
STS Dashboard Server Debug Tool

Debug server issues, check module loading, verify functions work correctly.
Useful for diagnosing why the API might return incomplete data.

Usage:
    uv run scripts/dev/server_debug.py              # Full diagnostic
    uv run scripts/dev/server_debug.py modules      # Check module loading
    uv run scripts/dev/server_debug.py functions    # Test key functions
    uv run scripts/dev/server_debug.py server       # Check server process
    uv run scripts/dev/server_debug.py cache        # Check cache status
"""

import importlib
import os
import sys
import subprocess
from pathlib import Path
from typing import Any, Dict, List, Optional

# Project setup
PROJECT_ROOT = Path(__file__).parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT))


def check_server_process() -> Dict[str, Any]:
    """Check if the server is running and get process info."""
    result = {
        "running": False,
        "pid": None,
        "port": 8080,
        "command": None,
        "python_version": None,
    }

    try:
        # Check lsof for port 8080
        lsof_out = subprocess.run(
            ["lsof", "-i", ":8080"],
            capture_output=True,
            text=True
        )
        if lsof_out.returncode == 0 and lsof_out.stdout:
            lines = lsof_out.stdout.strip().split('\n')
            if len(lines) > 1:
                # Parse the process info
                parts = lines[1].split()
                if len(parts) >= 2:
                    result["running"] = True
                    result["pid"] = int(parts[1])

        # Check PID file
        pid_file = Path("/tmp/sts_dashboard.pid")
        if pid_file.exists():
            result["pid_file"] = str(pid_file)
            result["pid_from_file"] = pid_file.read_text().strip()

        # Get command from ps if we have a PID
        if result["pid"]:
            ps_out = subprocess.run(
                ["ps", "-p", str(result["pid"]), "-o", "command="],
                capture_output=True,
                text=True
            )
            if ps_out.returncode == 0:
                result["command"] = ps_out.stdout.strip()

    except Exception as e:
        result["error"] = str(e)

    return result


def check_cache_status() -> Dict[str, Any]:
    """Check Python cache files."""
    result = {
        "web_pycache": [],
        "core_pycache": [],
        "total_size": 0,
    }

    # Web pycache
    web_cache = PROJECT_ROOT / "web" / "__pycache__"
    if web_cache.exists():
        for f in web_cache.glob("*.pyc"):
            stat = f.stat()
            result["web_pycache"].append({
                "name": f.name,
                "size": stat.st_size,
                "mtime": stat.st_mtime,
            })
            result["total_size"] += stat.st_size

    # Core pycache (check multiple directories)
    for core_dir in (PROJECT_ROOT / "core").rglob("__pycache__"):
        for f in core_dir.glob("*.pyc"):
            stat = f.stat()
            rel_path = f.relative_to(PROJECT_ROOT)
            result["core_pycache"].append({
                "name": str(rel_path),
                "size": stat.st_size,
                "mtime": stat.st_mtime,
            })
            result["total_size"] += stat.st_size

    return result


def check_module_loading() -> Dict[str, Any]:
    """Test that all required modules load correctly."""
    result = {
        "modules": {},
        "errors": [],
    }

    modules_to_check = [
        ("web.server", ["app", "process_save_data", "predict_neow_options"]),
        ("core.state.rng", ["Random", "long_to_seed", "seed_to_long"]),
        ("core.state.game_rng", ["GameRNGState", "RNGStream"]),
        ("core.comparison.full_rng_tracker", ["read_save_file", "predict_boss_relics"]),
        ("core.generation.boss", ["predict_all_bosses_extended"]),
        ("core.generation.relics", ["predict_neow_boss_swap", "predict_all_relic_pools"]),
        ("core.generation.shop", ["predict_shop_inventory"]),
        ("core.generation.treasure", ["predict_chest", "predict_full_chest"]),
        ("core.generation.potions", ["predict_potion_drop"]),
    ]

    for module_name, expected_attrs in modules_to_check:
        module_result = {"loaded": False, "attrs": {}, "file": None}

        try:
            # Force reimport
            if module_name in sys.modules:
                del sys.modules[module_name]

            module = importlib.import_module(module_name)
            module_result["loaded"] = True
            module_result["file"] = getattr(module, "__file__", None)

            for attr in expected_attrs:
                module_result["attrs"][attr] = hasattr(module, attr)
                if not hasattr(module, attr):
                    result["errors"].append(f"{module_name} missing {attr}")

        except Exception as e:
            module_result["error"] = str(e)
            result["errors"].append(f"Failed to load {module_name}: {e}")

        result["modules"][module_name] = module_result

    return result


def test_functions() -> Dict[str, Any]:
    """Test that key server functions work correctly."""
    result = {
        "functions": {},
        "errors": [],
    }

    # Test process_save_data with mock data
    try:
        from web.server import process_save_data

        mock_save = {
            'seed': 12345,
            'floor_num': 0,
            'act_num': 1,
            'ascension_level': 20,
            'current_health': 72,
            'max_health': 72,
            'gold': 99,
            'class': 'WATCHER',
            'cards': [],
            'relics': ['PureWater'],
            'potions': [],
            'room_x': 0,
            'room_y': -1,
            'current_room': 'NeowRoom',
            'card_seed_count': 0,
            'card_random_seed_randomizer': 5,
            'relic_seed_count': 0,
            'potion_seed_count': 0,
            'monster_seed_count': 0,
            'event_seed_count': 0,
            'merchant_seed_count': 0,
            'treasure_seed_count': 0,
            'potion_chance': 0,
            'metric_path_per_floor': [],
            'path_x': [],
            'path_y': [],
            'monster_list': ['Jaw Worm'],
            'elite_monster_list': ['Gremlin Nob'],
            'event_list': ['Big Fish'],
            'boss': 'Hexaghost',
            'metric_card_choices': [],
            'metric_boss_relics': [],
            'current_hp_per_floor': [],
            'max_hp_per_floor': [],
            'gold_per_floor': [],
            'damage_taken': [],
            'event_choices': [],
            'campfire_choices': [],
            'boss_relics': [],
            'potions_obtained': [],
            'relics_obtained': [],
            'items_purchased': [],
            'items_purged': [],
        }

        output = process_save_data(mock_save)
        result["functions"]["process_save_data"] = {
            "ok": True,
            "keys": len(output),
            "has_neow": "neow_options" in output and bool(output["neow_options"]),
            "has_bosses": "predicted_bosses" in output and bool(output["predicted_bosses"]),
            "has_rng_accuracy": "rng_accuracy" in output and bool(output["rng_accuracy"]),
        }

    except Exception as e:
        result["functions"]["process_save_data"] = {"ok": False, "error": str(e)}
        result["errors"].append(f"process_save_data failed: {e}")

    # Test predict_neow_options
    try:
        from web.server import predict_neow_options

        options = predict_neow_options(12345, "WATCHER")
        result["functions"]["predict_neow_options"] = {
            "ok": True,
            "count": len(options),
            "sample": options[0] if options else None,
        }

    except Exception as e:
        result["functions"]["predict_neow_options"] = {"ok": False, "error": str(e)}
        result["errors"].append(f"predict_neow_options failed: {e}")

    # Test predict_all_bosses_extended
    try:
        from core.generation.encounters import predict_all_bosses_extended

        bosses = predict_all_bosses_extended(12345, ascension=20)
        result["functions"]["predict_all_bosses_extended"] = {
            "ok": True,
            "acts": list(bosses.keys()),
        }

    except Exception as e:
        result["functions"]["predict_all_bosses_extended"] = {"ok": False, "error": str(e)}
        result["errors"].append(f"predict_all_bosses_extended failed: {e}")

    # Test calculate_rng_accuracy
    try:
        from web.server import calculate_rng_accuracy

        accuracy = calculate_rng_accuracy(mock_save, "ABC12", "WATCHER")
        result["functions"]["calculate_rng_accuracy"] = {
            "ok": True,
            "overall_ratio": accuracy.get("overall", {}).get("ratio", 0),
        }

    except Exception as e:
        result["functions"]["calculate_rng_accuracy"] = {"ok": False, "error": str(e)}
        result["errors"].append(f"calculate_rng_accuracy failed: {e}")

    return result


def print_server_status(server: Dict[str, Any]) -> None:
    """Print server process status."""
    print("SERVER PROCESS:")
    if server["running"]:
        print(f"  Status: RUNNING")
        print(f"  PID: {server['pid']}")
        if server.get("command"):
            print(f"  Command: {server['command'][:80]}...")
    else:
        print("  Status: NOT RUNNING")

    if server.get("pid_file"):
        print(f"  PID file: {server['pid_file']} (value: {server.get('pid_from_file')})")

    if server.get("error"):
        print(f"  Error: {server['error']}")
    print()


def print_cache_status(cache: Dict[str, Any]) -> None:
    """Print cache status."""
    print("PYTHON CACHE:")
    print(f"  Total size: {cache['total_size'] / 1024:.1f} KB")

    if cache["web_pycache"]:
        print(f"  Web pycache files: {len(cache['web_pycache'])}")
        for f in cache["web_pycache"]:
            from datetime import datetime
            mtime = datetime.fromtimestamp(f["mtime"]).strftime("%Y-%m-%d %H:%M:%S")
            print(f"    {f['name']}: {f['size']} bytes ({mtime})")

    if cache["core_pycache"]:
        print(f"  Core pycache files: {len(cache['core_pycache'])}")
    print()


def print_module_status(modules: Dict[str, Any]) -> None:
    """Print module loading status."""
    print("MODULE LOADING:")
    for mod_name, mod_info in modules["modules"].items():
        if mod_info["loaded"]:
            missing = [k for k, v in mod_info["attrs"].items() if not v]
            if missing:
                print(f"  [WARN] {mod_name}: loaded but missing {missing}")
            else:
                print(f"  [ OK ] {mod_name}")
        else:
            print(f"  [FAIL] {mod_name}: {mod_info.get('error', 'unknown error')}")
    print()


def print_function_status(functions: Dict[str, Any]) -> None:
    """Print function test status."""
    print("FUNCTION TESTS:")
    for func_name, func_info in functions["functions"].items():
        if func_info["ok"]:
            details = ", ".join(f"{k}={v}" for k, v in func_info.items() if k != "ok")
            print(f"  [ OK ] {func_name}: {details}")
        else:
            print(f"  [FAIL] {func_name}: {func_info.get('error', 'unknown error')}")

    if functions["errors"]:
        print("\n  ERRORS:")
        for err in functions["errors"]:
            print(f"    - {err}")
    print()


def main() -> int:
    if len(sys.argv) > 1:
        cmd = sys.argv[1].lower()

        if cmd == "server":
            server = check_server_process()
            print_server_status(server)
            return 0 if server["running"] else 1

        elif cmd == "cache":
            cache = check_cache_status()
            print_cache_status(cache)
            return 0

        elif cmd == "modules":
            modules = check_module_loading()
            print_module_status(modules)
            return 0 if not modules["errors"] else 1

        elif cmd == "functions":
            functions = test_functions()
            print_function_status(functions)
            return 0 if not functions["errors"] else 1

        elif cmd in ["-h", "--help", "help"]:
            print(__doc__)
            return 0

        else:
            print(f"Unknown command: {cmd}")
            print("Usage: server_debug.py [server|cache|modules|functions]")
            return 1

    # Full diagnostic
    print("=" * 60)
    print("STS DASHBOARD SERVER DIAGNOSTIC")
    print("=" * 60)
    print()

    # Server process
    server = check_server_process()
    print_server_status(server)

    # Cache status
    cache = check_cache_status()
    print_cache_status(cache)

    # Module loading
    modules = check_module_loading()
    print_module_status(modules)

    # Function tests
    functions = test_functions()
    print_function_status(functions)

    # Summary
    print("=" * 60)
    print("SUMMARY:")
    issues = []

    if not server["running"]:
        issues.append("Server is not running")

    if modules["errors"]:
        issues.append(f"{len(modules['errors'])} module loading errors")

    if functions["errors"]:
        issues.append(f"{len(functions['errors'])} function test failures")

    if issues:
        print("  ISSUES FOUND:")
        for issue in issues:
            print(f"    - {issue}")
        print("\n  RECOMMENDED ACTIONS:")
        if not server["running"]:
            print("    - Start server: uv run scripts/sts.py dashboard")
        if modules["errors"] or functions["errors"]:
            print("    - Clear cache and restart: uv run scripts/dev/restart_server.py")
    else:
        print("  All checks passed!")

    return 0 if not issues else 1


if __name__ == "__main__":
    sys.exit(main())
