"""
WebSocket server for streaming Slay the Spire game state.

Accepts WebSocket connections and manages game sessions.  Each connection
can create a new game, query actions/observations, execute actions, and
run auto-play with configurable delay for the visualization frontend.

Start directly:
    uv run python packages/server/ws_server.py --port 8080

Or via module:
    uv run python -m packages.server --port 8080
"""

from __future__ import annotations

import asyncio
import json
import logging
import random
from typing import Any, Dict, Optional

import websockets
from websockets.asyncio.server import ServerConnection

from .game_session import GameSession
from .protocol import (
    MessageType,
    make_game_created,
    make_actions,
    make_action_result,
    make_observation,
    make_step,
    make_game_over,
    make_error,
)

logger = logging.getLogger(__name__)


class GameServer:
    """WebSocket server that manages game sessions and streams state."""

    def __init__(self, host: str = "localhost", port: int = 8080):
        self.host = host
        self.port = port
        # Map connection id -> GameSession
        self.sessions: Dict[int, GameSession] = {}
        # Track auto-play cancellation per connection
        self._auto_play_tasks: Dict[int, asyncio.Task] = {}

    async def start(self) -> None:
        """Start the WebSocket server and run forever."""
        async with websockets.serve(self.handler, self.host, self.port):
            logger.info("Game server running at ws://%s:%d", self.host, self.port)
            print(f"Game server running at ws://{self.host}:{self.port}")
            await asyncio.Future()  # run forever

    async def handler(self, websocket: ServerConnection) -> None:
        """Handle a single WebSocket connection."""
        conn_id = id(websocket)
        logger.info("Client connected: %s", conn_id)
        try:
            async for raw_message in websocket:
                try:
                    data = json.loads(raw_message)
                except json.JSONDecodeError:
                    await websocket.send(json.dumps(make_error("Invalid JSON")))
                    continue

                response = await self._process_message(data, websocket, conn_id)
                if response is not None:
                    await websocket.send(json.dumps(response))
        except websockets.ConnectionClosed:
            logger.info("Client disconnected: %s", conn_id)
        finally:
            # Clean up session and any running auto-play
            self.sessions.pop(conn_id, None)
            task = self._auto_play_tasks.pop(conn_id, None)
            if task and not task.done():
                task.cancel()

    # ------------------------------------------------------------------
    # Message dispatch
    # ------------------------------------------------------------------

    async def _process_message(
        self,
        data: Dict[str, Any],
        websocket: ServerConnection,
        conn_id: int,
    ) -> Optional[Dict[str, Any]]:
        """Route an incoming message to the correct handler."""
        msg_type = data.get("type")

        if msg_type == MessageType.NEW_GAME.value:
            return await self._handle_new_game(data, websocket, conn_id)
        elif msg_type == MessageType.GET_ACTIONS.value:
            return self._handle_get_actions(conn_id)
        elif msg_type == MessageType.TAKE_ACTION.value:
            return self._handle_take_action(data, conn_id)
        elif msg_type == MessageType.GET_OBSERVATION.value:
            return self._handle_get_observation(data, conn_id)
        elif msg_type == MessageType.AUTO_PLAY.value:
            return await self._handle_auto_play(data, websocket, conn_id)
        else:
            return make_error(f"Unknown message type: {msg_type}", msg_type)

    # ------------------------------------------------------------------
    # Handler implementations
    # ------------------------------------------------------------------

    async def _handle_new_game(
        self,
        data: Dict[str, Any],
        websocket: ServerConnection,
        conn_id: int,
    ) -> Dict[str, Any]:
        """Create a new game session."""
        seed = data.get("seed", str(random.randint(0, 2**31)))
        ascension = data.get("ascension", 20)
        character = data.get("character", "Watcher")

        # Cancel any in-flight auto-play for this connection
        self._cancel_auto_play(conn_id)

        session = GameSession(
            seed=seed,
            ascension=ascension,
            character=character,
        )
        self.sessions[conn_id] = session

        obs = session.get_observation()
        logger.info(
            "New game created: session=%s seed=%s asc=%d char=%s",
            session.session_id, seed, ascension, character,
        )
        return make_game_created(session.session_id, obs)

    def _handle_get_actions(self, conn_id: int) -> Dict[str, Any]:
        """Return available actions for the current session."""
        session = self.sessions.get(conn_id)
        if session is None:
            return make_error("No active game session. Send 'new_game' first.", "get_actions")
        return make_actions(session.get_actions())

    def _handle_take_action(
        self, data: Dict[str, Any], conn_id: int
    ) -> Dict[str, Any]:
        """Execute an action in the current session."""
        session = self.sessions.get(conn_id)
        if session is None:
            return make_error("No active game session. Send 'new_game' first.", "take_action")

        action = data.get("action")
        if action is None:
            return make_error("Missing 'action' field in take_action message.", "take_action")

        try:
            result = session.take_action(action)
        except Exception as exc:
            return make_error(f"Action failed: {exc}", "take_action")

        obs = session.get_observation()

        if session.game_over:
            return make_action_result(
                result, obs, game_over=True, game_won=session.game_won
            )
        return make_action_result(result, obs)

    def _handle_get_observation(
        self, data: Dict[str, Any], conn_id: int
    ) -> Dict[str, Any]:
        """Return the current observation."""
        session = self.sessions.get(conn_id)
        if session is None:
            return make_error("No active game session. Send 'new_game' first.", "get_observation")

        profile = data.get("profile", "human")
        return make_observation(session.get_observation(profile=profile))

    async def _handle_auto_play(
        self,
        data: Dict[str, Any],
        websocket: ServerConnection,
        conn_id: int,
    ) -> Optional[Dict[str, Any]]:
        """Auto-play N steps, streaming each step to the client.

        Returns None because responses are sent directly via websocket.
        """
        session = self.sessions.get(conn_id)
        if session is None:
            return make_error("No active game session. Send 'new_game' first.", "auto_play")

        steps = data.get("steps", 100)
        delay_ms = data.get("delay_ms", 200)
        delay_s = max(delay_ms / 1000.0, 0.0)

        # Cancel any previous auto-play for this connection
        self._cancel_auto_play(conn_id)

        async def _run_auto_play() -> None:
            for step in range(1, steps + 1):
                if session.game_over:
                    break

                actions = session.get_actions()
                if not actions:
                    break

                # Pick first action (random agent could be added later)
                action = actions[0]
                try:
                    session.take_action(action)
                except Exception as exc:
                    await websocket.send(json.dumps(
                        make_error(f"Auto-play step {step} failed: {exc}", "auto_play")
                    ))
                    break

                obs = session.get_observation()

                if session.game_over:
                    await websocket.send(json.dumps(
                        make_game_over(session.game_won, obs)
                    ))
                    break

                await websocket.send(json.dumps(
                    make_step(step, obs, action)
                ))

                if delay_s > 0:
                    await asyncio.sleep(delay_s)

        task = asyncio.create_task(_run_auto_play())
        self._auto_play_tasks[conn_id] = task
        return None  # responses streamed directly

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    def _cancel_auto_play(self, conn_id: int) -> None:
        task = self._auto_play_tasks.pop(conn_id, None)
        if task and not task.done():
            task.cancel()


# ---------------------------------------------------------------------------
# Standalone entry point
# ---------------------------------------------------------------------------

def main(host: str = "localhost", port: int = 8080) -> None:
    """Run the game server."""
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    )
    server = GameServer(host=host, port=port)
    asyncio.run(server.start())


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Slay the Spire WebSocket game server")
    parser.add_argument("--host", default="localhost", help="Host to bind to (default: localhost)")
    parser.add_argument("--port", type=int, default=8080, help="Port to listen on (default: 8080)")
    args = parser.parse_args()
    main(host=args.host, port=args.port)
