#!/usr/bin/env python3
"""
STS Dashboard Server Restart Tool

Clean restart of the server with cache clearing.
Use when the server is returning stale/incomplete data.

Usage:
    uv run scripts/dev/restart_server.py            # Stop, clear, start
    uv run scripts/dev/restart_server.py --stop     # Just stop
    uv run scripts/dev/restart_server.py --clear    # Just clear cache
    uv run scripts/dev/restart_server.py --fg       # Start in foreground
"""

import argparse
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.parent
PID_FILE = Path("/tmp/sts_dashboard.pid")
DEFAULT_PORT = 8080
DEFAULT_HOST = "127.0.0.1"


def get_server_pid() -> int | None:
    """Get PID of running server from PID file or lsof."""
    # Try PID file first
    if PID_FILE.exists():
        try:
            pid = int(PID_FILE.read_text().strip())
            # Verify process exists
            os.kill(pid, 0)
            return pid
        except (ValueError, ProcessLookupError, PermissionError):
            pass

    # Try lsof
    try:
        result = subprocess.run(
            ["lsof", "-i", f":{DEFAULT_PORT}", "-t"],
            capture_output=True,
            text=True
        )
        if result.returncode == 0 and result.stdout.strip():
            return int(result.stdout.strip().split('\n')[0])
    except Exception:
        pass

    return None


def stop_server(verbose: bool = True) -> bool:
    """Stop the running server."""
    pid = get_server_pid()

    if pid is None:
        if verbose:
            print("No server running")
        return True

    if verbose:
        print(f"Stopping server (PID {pid})...")

    try:
        os.kill(pid, signal.SIGTERM)

        # Wait for graceful shutdown
        for _ in range(20):  # 2 seconds
            time.sleep(0.1)
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                # Process is dead
                break
        else:
            # Force kill
            if verbose:
                print("  Force killing...")
            os.kill(pid, signal.SIGKILL)
            time.sleep(0.1)

        if verbose:
            print("  Server stopped")
        return True

    except ProcessLookupError:
        if verbose:
            print("  Server already stopped")
        return True
    except PermissionError:
        if verbose:
            print(f"  Permission denied to kill {pid}")
        return False
    finally:
        # Clean up PID file
        PID_FILE.unlink(missing_ok=True)


def clear_cache(verbose: bool = True) -> int:
    """Clear Python cache files. Returns count of files deleted."""
    count = 0

    # Web pycache
    web_cache = PROJECT_ROOT / "web" / "__pycache__"
    if web_cache.exists():
        for f in web_cache.glob("*.pyc"):
            f.unlink()
            count += 1
        if verbose:
            print(f"Cleared web/__pycache__")

    # Core pycache (all subdirectories)
    core_dir = PROJECT_ROOT / "core"
    if core_dir.exists():
        for cache_dir in core_dir.rglob("__pycache__"):
            for f in cache_dir.glob("*.pyc"):
                f.unlink()
                count += 1
        if verbose and count > 0:
            print(f"Cleared core/**/__pycache__")

    # Scripts pycache
    scripts_cache = PROJECT_ROOT / "scripts" / "__pycache__"
    if scripts_cache.exists():
        for f in scripts_cache.glob("*.pyc"):
            f.unlink()
            count += 1

    if verbose:
        print(f"Total: {count} cache files cleared")

    return count


def start_server(foreground: bool = False, verbose: bool = True) -> bool:
    """Start the server."""
    url = f"http://{DEFAULT_HOST}:{DEFAULT_PORT}"

    if foreground:
        if verbose:
            print(f"Starting server at {url} (foreground mode)")
            print("Press Ctrl+C to stop")
            print()

        try:
            import uvicorn

            # Add project to path
            sys.path.insert(0, str(PROJECT_ROOT))
            from web.server import app

            uvicorn.run(app, host=DEFAULT_HOST, port=DEFAULT_PORT, log_level="warning")
        except KeyboardInterrupt:
            print("\nStopped.")
        return True

    else:
        if verbose:
            print(f"Starting server at {url}...")

        # Start as background process
        cmd = [
            sys.executable, "-c",
            f"""
import sys
sys.path.insert(0, '{PROJECT_ROOT}')
import uvicorn
from web.server import app
uvicorn.run(app, host='{DEFAULT_HOST}', port={DEFAULT_PORT}, log_level='warning')
"""
        ]

        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )

        # Save PID
        PID_FILE.write_text(str(proc.pid))

        # Wait and verify
        time.sleep(1.5)

        if get_server_pid():
            if verbose:
                print(f"  Server started (PID {proc.pid})")
                print(f"  Dashboard: {url}")
            return True
        else:
            if verbose:
                print("  Failed to start server")
            return False


def verify_api(verbose: bool = True) -> bool:
    """Verify the API is returning expected data."""
    import json
    import urllib.request

    try:
        with urllib.request.urlopen(f"http://{DEFAULT_HOST}:{DEFAULT_PORT}/api/state", timeout=5) as resp:
            data = json.loads(resp.read().decode())

        # Check for expected keys
        expected = ["neow_options", "predicted_bosses", "rng_accuracy", "boss_relics_per_act"]
        missing = [k for k in expected if k not in data]

        if "error" in data:
            if verbose:
                print(f"  API returned error: {data['error']}")
                # This is OK if no save file exists
                if "No save file" in data.get("error", "") or "No such file" in str(data.get("path", "")):
                    print("  (This is expected if no game is in progress)")
                    return True
            return False

        if missing:
            if verbose:
                print(f"  WARNING: API missing keys: {missing}")
                print("  Server may be running old code")
            return False

        if verbose:
            print(f"  API OK: {len(data)} keys returned")
        return True

    except Exception as e:
        if verbose:
            print(f"  API check failed: {e}")
        return False


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Clean restart of STS dashboard server",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    uv run scripts/dev/restart_server.py            # Full restart with cache clear
    uv run scripts/dev/restart_server.py --stop     # Just stop the server
    uv run scripts/dev/restart_server.py --clear    # Just clear cache
    uv run scripts/dev/restart_server.py --fg       # Start in foreground mode
"""
    )

    parser.add_argument("--stop", action="store_true", help="Only stop the server")
    parser.add_argument("--clear", action="store_true", help="Only clear cache")
    parser.add_argument("--fg", "--foreground", action="store_true", dest="foreground",
                        help="Start in foreground mode")
    parser.add_argument("--no-verify", action="store_true", help="Skip API verification")
    parser.add_argument("-q", "--quiet", action="store_true", help="Minimal output")

    args = parser.parse_args()
    verbose = not args.quiet

    if verbose:
        print("=" * 50)
        print("STS Dashboard Server Restart")
        print("=" * 50)
        print()

    # Just stop
    if args.stop:
        return 0 if stop_server(verbose) else 1

    # Just clear
    if args.clear:
        clear_cache(verbose)
        return 0

    # Full restart: stop -> clear -> start -> verify

    # Step 1: Stop
    stop_server(verbose)
    print()

    # Step 2: Clear cache
    clear_cache(verbose)
    print()

    # Step 3: Start
    if not start_server(args.foreground, verbose):
        return 1

    if args.foreground:
        return 0  # Foreground mode doesn't return until stopped

    print()

    # Step 4: Verify
    if not args.no_verify:
        if verbose:
            print("Verifying API...")
        time.sleep(0.5)  # Extra wait for server to be fully ready
        verify_api(verbose)

    if verbose:
        print()
        print("Done! Server is running.")
        print(f"Dashboard: http://{DEFAULT_HOST}:{DEFAULT_PORT}")
        print()
        print("To stop: uv run scripts/dev/restart_server.py --stop")

    return 0


if __name__ == "__main__":
    sys.exit(main())
