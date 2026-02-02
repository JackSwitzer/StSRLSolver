#!/usr/bin/env python3
"""
Run the live dashboard server.

Usage:
    uv run scripts/run_dashboard.py
    uv run scripts/run_dashboard.py --port 8080
"""

import argparse
import subprocess
import sys
import os

# Ensure we're in the project root
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(PROJECT_ROOT)
sys.path.insert(0, PROJECT_ROOT)


def main():
    parser = argparse.ArgumentParser(description="Run the Slay the Spire live dashboard")
    parser.add_argument("--port", type=int, default=8080, help="Port to run server on")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    args = parser.parse_args()

    print(f"Starting dashboard at http://{args.host}:{args.port}")
    print("Press Ctrl+C to stop")

    # Run the server
    import uvicorn
    from web.server import app

    uvicorn.run(app, host=args.host, port=args.port, log_level="warning")


if __name__ == "__main__":
    main()
