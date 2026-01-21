#!/usr/bin/env python3
"""
Combat Search Server launcher.

Starts the Python search server on port 9998 for Java mod communication.

Usage:
    python3 run_search_server.py [--port 9998] [--budget 1000] [-v]
"""

import argparse
import logging
import os
import sys
import signal
import time

# Add core to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from core.server.search_server import CombatSearchServer

# Global server reference for signal handler
_server = None


def signal_handler(sig, frame):
    """Handle Ctrl+C gracefully."""
    global _server
    print("\n[SearchServer] Shutting down...")
    if _server:
        _server.stop()
    sys.exit(0)


def main():
    """Main entry point."""
    global _server

    parser = argparse.ArgumentParser(
        description="Combat Search Server for Java ↔ Python integration"
    )
    parser.add_argument(
        "--host",
        default="127.0.0.1",
        help="Host to bind to (default: 127.0.0.1)"
    )
    parser.add_argument(
        "--port",
        type=int,
        default=9998,
        help="Port to listen on (default: 9998)"
    )
    parser.add_argument(
        "--budget",
        type=int,
        default=1000,
        help="MCTS search budget / iterations (default: 1000)"
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=0,
        help="Worker processes, 0=auto (default: 0)"
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Verbose logging"
    )

    args = parser.parse_args()

    # Configure logging
    level = logging.DEBUG if args.verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
        datefmt="%H:%M:%S",
    )

    logger = logging.getLogger("SearchServer")

    # Set up signal handler
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    # Create and start server
    logger.info("=" * 50)
    logger.info("Combat Search Server")
    logger.info("=" * 50)
    logger.info(f"Host: {args.host}")
    logger.info(f"Port: {args.port}")
    logger.info(f"Search budget: {args.budget}")
    logger.info(f"Workers: {args.workers if args.workers > 0 else 'auto'}")
    logger.info("=" * 50)

    _server = CombatSearchServer(
        host=args.host,
        port=args.port,
        search_budget=args.budget,
        n_workers=args.workers,
    )

    try:
        _server.start()
        logger.info("Server started. Press Ctrl+C to stop.")
        logger.info("Waiting for Java client connection...")

        # Keep running
        while True:
            time.sleep(1)

    except KeyboardInterrupt:
        logger.info("Interrupted")
    except Exception as e:
        logger.error(f"Server error: {e}", exc_info=True)
    finally:
        _server.stop()
        logger.info("Server stopped")


if __name__ == "__main__":
    main()
