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
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import asdict
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
    make_conquerer_path_result,
    make_conquerer_complete,
)

logger = logging.getLogger(__name__)

_STRATEGY_NAMES = {
    0: "greedy",
    1: "random_0.5",
    2: "random_1.0",
    3: "random_2.0",
    4: "heuristic_attack",
    5: "heuristic_block",
    6: "heuristic_balanced",
    7: "weighted_7",
    8: "weighted_8",
    9: "weighted_9",
}


def _strategy_name(path_id: int) -> str:
    return _STRATEGY_NAMES.get(path_id, f"path_{path_id}")


class GameServer:
    """WebSocket server that manages game sessions and streams state."""

    def __init__(self, host: str = "localhost", port: int = 8080):
        self.host = host
        self.port = port
        # Map connection id -> GameSession
        self.sessions: Dict[int, GameSession] = {}
        # Track auto-play cancellation per connection
        self._auto_play_tasks: Dict[int, asyncio.Task] = {}
        # Training coordinator (shared across connections)
        self._training: Optional["TrainingCoordinator"] = None
        self._training_poll_task: Optional[asyncio.Task] = None
        # Per-connection training WS forwarder tasks
        self._training_forwarders: Dict[int, asyncio.Task] = {}

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
        elif msg_type == MessageType.CONQUERER_RUN.value:
            return await self._handle_conquerer_run(data, websocket, conn_id)
        elif msg_type == MessageType.TRAINING_START.value:
            return await self._handle_training_start(data, websocket, conn_id)
        elif msg_type == MessageType.TRAINING_STOP.value:
            return await self._handle_training_stop(data, websocket, conn_id)
        elif msg_type == MessageType.TRAINING_RESUME.value:
            return await self._handle_training_start(data, websocket, conn_id)
        elif msg_type == MessageType.TRAINING_FOCUS.value:
            return self._handle_training_focus(data, conn_id)
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

    async def _handle_conquerer_run(
        self,
        data: Dict[str, Any],
        websocket: ServerConnection,
        conn_id: int,
    ) -> Optional[Dict[str, Any]]:
        """Run SeedConquerer and stream results as paths complete.

        Client sends: {"type": "conquerer_run", "seed": "...", "num_paths": 10}
        Server streams: conquerer_path_result per path, then conquerer_complete.
        """
        seed = data.get("seed", str(random.randint(0, 2**31)))
        num_paths = data.get("num_paths", 10)
        ascension = data.get("ascension", 20)

        # Cancel any previous auto-play / conquerer for this connection
        self._cancel_auto_play(conn_id)

        async def _run_conquerer() -> None:
            import time as _time
            from packages.training.conquerer import _run_path, SeedConquerer

            start_time = _time.time()
            loop = asyncio.get_running_loop()
            results = []
            active = num_paths

            # Run paths in thread pool to avoid blocking the event loop
            with ProcessPoolExecutor(max_workers=min(num_paths, 4)) as executor:
                futures = {
                    loop.run_in_executor(
                        executor,
                        _run_path,
                        seed,
                        path_id,
                        ascension,
                        "Watcher",
                        3000,
                    ): path_id
                    for path_id in range(num_paths)
                }

                for coro in asyncio.as_completed(futures):
                    try:
                        path_result = await coro
                    except Exception as exc:
                        logger.error("Conquerer path failed: %s", exc)
                        active -= 1
                        continue

                    results.append(path_result)
                    active -= 1

                    # Stream this path result
                    path_dict = {
                        "path_id": path_result.path_id,
                        "seed": path_result.seed,
                        "won": path_result.won,
                        "floors_reached": path_result.floors_reached,
                        "hp_remaining": path_result.hp_remaining,
                        "total_reward": path_result.total_reward,
                        "strategy": _strategy_name(path_result.path_id),
                    }
                    try:
                        await websocket.send(json.dumps(
                            make_conquerer_path_result(path_dict, active)
                        ))
                    except websockets.ConnectionClosed:
                        return

            # Build final result using SeedConquerer's post-processing
            conq = SeedConquerer(num_paths=num_paths, ascension=ascension, parallel=False)
            results.sort(key=lambda r: r.path_id)

            # Compute divergence and select best
            if results:
                from packages.training.conquerer import _find_divergence_points
                baseline_log = results[0].decision_log
                for r in results[1:]:
                    r.divergence_points = _find_divergence_points(baseline_log, r.decision_log)

            best = conq._select_best(results) if results else None
            tree = conq._build_divergence_tree(results) if results else {}

            complete_msg = {
                "type": MessageType.CONQUERER_COMPLETE.value,
                "seed": seed,
                "paths": [
                    {
                        "path_id": r.path_id,
                        "seed": r.seed,
                        "won": r.won,
                        "floors_reached": r.floors_reached,
                        "hp_remaining": r.hp_remaining,
                        "total_reward": r.total_reward,
                        "strategy": _strategy_name(r.path_id),
                    }
                    for r in results
                ],
                "best_path_id": best.path_id if best else 0,
                "win_count": sum(1 for r in results if r.won),
                "max_floor": max((r.floors_reached for r in results), default=0),
                "active_paths": 0,
                "elapsed_seconds": _time.time() - start_time,
                "divergence_tree": tree,
            }
            try:
                await websocket.send(json.dumps(complete_msg))
            except websockets.ConnectionClosed:
                pass

        task = asyncio.create_task(_run_conquerer())
        self._auto_play_tasks[conn_id] = task
        return None  # responses streamed directly

    # ------------------------------------------------------------------
    # Training handlers
    # ------------------------------------------------------------------

    async def _handle_training_start(
        self, data: Dict[str, Any], websocket: ServerConnection, conn_id: int,
    ) -> Optional[Dict[str, Any]]:
        """Start training with agents."""
        from .training_runner import TrainingCoordinator

        config = data.get("config", {})
        num_agents = config.get("num_agents", 4)
        mcts_sims = config.get("mcts_sims", 64)
        ascension = config.get("ascension", 20)
        seed = config.get("seed", "Test123")

        if self._training is None:
            self._training = TrainingCoordinator(
                num_agents=num_agents,
                mcts_sims=mcts_sims,
                ascension=ascension,
                initial_seed=seed,
            )
            await self._training.start()
            self._training_poll_task = asyncio.create_task(self._training.poll_events())
            logger.info("Training started: %d agents, %d sims, seed=%s", num_agents, mcts_sims, seed)

        # Subscribe this connection
        q = self._training.subscribe(conn_id)

        # Start forwarder task to push events from queue to websocket
        async def _forward():
            while True:
                try:
                    msg = await q.get()
                    await websocket.send(json.dumps(msg))
                except (websockets.ConnectionClosed, asyncio.CancelledError):
                    break
                except Exception:
                    continue

        task = asyncio.create_task(_forward())
        self._training_forwarders[conn_id] = task

        return {"type": "training_started", "num_agents": num_agents}

    async def _handle_training_stop(
        self, data: Dict[str, Any], websocket: ServerConnection, conn_id: int,
    ) -> Optional[Dict[str, Any]]:
        """Stop training."""
        if self._training:
            await self._training.stop()
            if self._training_poll_task:
                self._training_poll_task.cancel()
            self._training = None
            logger.info("Training stopped")
        return {"type": "training_stopped"}

    def _handle_training_focus(self, data: Dict[str, Any], conn_id: int) -> Optional[Dict[str, Any]]:
        """Toggle focus on a specific agent (add/remove from focused set)."""
        agent_id = data.get("agent_id")
        if self._training and agent_id is not None:
            # Toggle: if already focused, remove; otherwise add
            focused = self._training._focused.get(conn_id)
            if focused and agent_id in focused:
                self._training.remove_focus(conn_id, agent_id)
            else:
                self._training.set_focus(conn_id, agent_id)
        return None

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    def _cancel_auto_play(self, conn_id: int) -> None:
        task = self._auto_play_tasks.pop(conn_id, None)
        if task and not task.done():
            task.cancel()
        # Also clean up training forwarder
        fwd = self._training_forwarders.pop(conn_id, None)
        if fwd and not fwd.done():
            fwd.cancel()
        if self._training:
            self._training.unsubscribe(conn_id)


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
