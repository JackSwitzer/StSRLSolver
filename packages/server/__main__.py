"""Allow running the server via: uv run python -m packages.server"""

from .ws_server import main

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Slay the Spire WebSocket game server")
    parser.add_argument("--host", default="localhost", help="Host to bind to (default: localhost)")
    parser.add_argument("--port", type=int, default=8080, help="Port to listen on (default: 8080)")
    parser.add_argument(
        "--no-auto-train",
        action="store_true",
        help="Start in monitor-only mode (no training launched automatically)",
    )
    args = parser.parse_args()
    main(host=args.host, port=args.port, auto_train=not args.no_auto_train)
