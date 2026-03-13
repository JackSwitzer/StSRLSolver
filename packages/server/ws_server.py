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
import os
import random
import subprocess
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import asdict
from typing import Any, Dict, Optional, Set

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


# ---------------------------------------------------------------------------
# System stats collection (psutil-based)
# ---------------------------------------------------------------------------

try:
    import psutil

    _HAS_PSUTIL = True
except ImportError:  # pragma: no cover
    _HAS_PSUTIL = False

# Prime psutil's CPU measurement on import so the first real call returns
# a meaningful value (psutil.cpu_percent needs a prior sample).
if _HAS_PSUTIL:
    psutil.cpu_percent(interval=None)

# GPU detection (cached at module level -- doesn't change at runtime)
_GPU_INFO: Dict[str, Any] = {"available": False, "name": "N/A"}
try:
    import torch

    if torch.backends.mps.is_available():
        _GPU_INFO = {"available": True, "name": "Apple MPS"}
    elif torch.cuda.is_available():
        _GPU_INFO = {"available": True, "name": torch.cuda.get_device_name(0)}
except Exception:
    pass


def _collect_system_stats() -> Dict[str, Any]:
    """Collect CPU%, RAM, GPU, and per-process breakdown.  ~5ms with psutil."""
    if _HAS_PSUTIL:
        cpu_pct = psutil.cpu_percent(interval=None)
        per_cpu = psutil.cpu_percent(interval=None, percpu=True)
        mem = psutil.virtual_memory()
        ram_used_gb = round(mem.used / (1024 ** 3), 2)
        ram_total_gb = round(mem.total / (1024 ** 3), 1)
        ram_pct = round(mem.percent, 1)
        swap = psutil.swap_memory()
        swap_used_gb = round(swap.used / (1024 ** 3), 2)
        swap_total_gb = round(swap.total / (1024 ** 3), 1)

        # Per-process breakdown: group by category
        categories: Dict[str, Dict[str, float]] = {}
        try:
            for proc in psutil.process_iter(["pid", "name", "cmdline", "cpu_percent", "memory_info"]):
                try:
                    info = proc.info
                    name = info.get("name", "")
                    cmdline = " ".join(info.get("cmdline") or [])
                    cpu = info.get("cpu_percent", 0) or 0
                    rss = (info.get("memory_info") or type("", (), {"rss": 0})).rss / (1024 ** 3)

                    # Categorize
                    if "overnight" in cmdline or "training" in cmdline:
                        cat = "RL Training"
                    elif "codex" in cmdline.lower() or "codex" in name.lower():
                        cat = "Codex (GPT 5.4)"
                    elif "claude" in cmdline.lower() or "claude" in name.lower():
                        cat = "Claude Code"
                    elif "node" in name.lower() or "bun" in name.lower() or "vite" in cmdline:
                        cat = "Node/Vite"
                    elif "cargo" in cmdline or "rustc" in name.lower() or "maturin" in cmdline:
                        cat = "Rust Build"
                    elif "python" in name.lower() or "uv" in name.lower():
                        cat = "Python"
                    elif "mds" in name or "spotlight" in name.lower() or "kernel_task" in name:
                        cat = "macOS System"
                    elif cpu > 1.0 or rss > 0.05:
                        cat = "Other"
                    else:
                        continue  # Skip idle tiny processes

                    if cat not in categories:
                        categories[cat] = {"cpu": 0.0, "ram_gb": 0.0, "count": 0}
                    categories[cat]["cpu"] += cpu
                    categories[cat]["ram_gb"] += rss
                    categories[cat]["count"] += 1
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
        except Exception:
            pass

        # Round category values
        proc_breakdown = {
            cat: {
                "cpu": round(vals["cpu"], 1),
                "ram_gb": round(vals["ram_gb"], 2),
                "count": int(vals["count"]),
            }
            for cat, vals in sorted(categories.items(), key=lambda x: -x[1]["cpu"])
        }
    else:
        cpu_pct = 0.0
        per_cpu = []
        ram_used_gb = 0.0
        ram_total_gb = 0.0
        ram_pct = 0.0
        swap_used_gb = 0.0
        swap_total_gb = 0.0
        proc_breakdown = {}

    # GPU memory (Apple Silicon unified memory — check via Metal/torch)
    gpu_mem_used_gb = 0.0
    gpu_mem_allocated_gb = 0.0
    gpu_util_pct = 0.0
    try:
        import torch
        if torch.backends.mps.is_available():
            gpu_mem_allocated_gb = round(torch.mps.current_allocated_memory() / (1024 ** 3), 2)
            # MPS driver memory (includes caches)
            gpu_mem_used_gb = round(torch.mps.driver_allocated_memory() / (1024 ** 3), 2)
    except Exception:
        pass

    return {
        "cpu_pct": round(cpu_pct, 1),
        "per_cpu": per_cpu,
        "cpu_cores": len(per_cpu),
        "ram_pct": ram_pct,
        "ram_used_gb": ram_used_gb,
        "ram_total_gb": ram_total_gb,
        "swap_used_gb": swap_used_gb,
        "swap_total_gb": swap_total_gb,
        "gpu_available": _GPU_INFO["available"],
        "gpu_name": _GPU_INFO["name"],
        "gpu_mem_used_gb": gpu_mem_used_gb,
        "gpu_mem_allocated_gb": gpu_mem_allocated_gb,
        "processes": proc_breakdown,
    }

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

    def __init__(self, host: str = "localhost", port: int = 8080, auto_train: bool = True):
        self.host = host
        self.port = port
        self.auto_train = auto_train
        # Map connection id -> GameSession
        self.sessions: Dict[int, GameSession] = {}
        # All connected websockets (for system_stats broadcast)
        self._connections: Set[ServerConnection] = set()
        # Track auto-play cancellation per connection
        self._auto_play_tasks: Dict[int, asyncio.Task] = {}
        # Training coordinator (shared across connections)
        self._training: Optional["TrainingCoordinator"] = None
        self._training_poll_task: Optional[asyncio.Task] = None
        # Per-connection training WS forwarder tasks
        self._training_forwarders: Dict[int, asyncio.Task] = {}
        # Background system stats broadcast task
        self._system_stats_task: Optional[asyncio.Task] = None

    async def start(self) -> None:
        """Start the WebSocket server and run forever."""
        async with websockets.serve(self.handler, self.host, self.port):
            logger.info("Game server running at ws://%s:%d", self.host, self.port)
            mode = "training" if self.auto_train else "monitor-only"
            print(f"Game server running at ws://{self.host}:{self.port} [{mode}]")
            self._system_stats_task = asyncio.create_task(self._broadcast_system_stats())
            await asyncio.Future()  # run forever

    async def handler(self, websocket: ServerConnection) -> None:
        """Handle a single WebSocket connection."""
        conn_id = id(websocket)
        self._connections.add(websocket)
        logger.info("Client connected: %s", conn_id)

        # Send metrics history immediately on connect
        if self._training and self._training.metrics_history:
            history = list(self._training.metrics_history)
            await websocket.send(json.dumps({
                "type": MessageType.METRICS_HISTORY.value,
                "floor_history": [h.get("floor", 0) for h in history],
                "loss_history": [h.get("loss", 0) for h in history],
                "win_history": [h.get("win_rate", 0) for h in history],
            }))

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
            self._connections.discard(websocket)
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
        elif msg_type == MessageType.TRAINING_CONFIG.value:
            # Alias: route to COMMAND/set_config for backwards compat
            config = data.get("config", data)
            return await self._handle_command(
                {"action": "set_config", "params": config}, websocket, conn_id,
            )
        elif msg_type == MessageType.COMMAND.value:
            return await self._handle_command(data, websocket, conn_id)
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
        """Pause training (preserves coordinator state)."""
        if self._training:
            self._training.pause()
            logger.info("Training paused")
        return {"type": "training_paused"}

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

    async def _handle_command(
        self, data: Dict[str, Any], websocket: ServerConnection, conn_id: int,
    ) -> Optional[Dict[str, Any]]:
        """Handle control commands: pause, resume, stop, set_config."""
        action = data.get("action")
        if action == "pause":
            if self._training:
                self._training.pause()
            return {"type": "command_ack", "action": "pause", "paused": True}

        elif action == "resume":
            if self._training:
                self._training.resume()
            return {"type": "command_ack", "action": "resume", "paused": False}

        elif action == "stop":
            if self._training:
                await self._training.stop()
                if self._training_poll_task:
                    self._training_poll_task.cancel()
                self._training = None
            return {"type": "command_ack", "action": "stop"}

        elif action == "start":
            # Start training via command (for monitor-only mode)
            return await self._handle_training_start(data, websocket, conn_id)

        elif action == "set_config":
            params = data.get("params", {})
            if self._training:
                self._training.set_config(params)
            return {"type": "command_ack", "action": "set_config", "params": params}

        else:
            return make_error(f"Unknown command action: {action}", "command")

    async def _broadcast_system_stats(self) -> None:
        """Broadcast system stats to all connected clients every 5 seconds."""
        while True:
            await asyncio.sleep(5.0)
            if not self._connections:
                continue
            try:
                stats = _collect_system_stats()
                worker_count = 0
                if self._training:
                    try:
                        worker_count = len(self._training.processes)
                    except Exception:
                        worker_count = self._training.num_agents if self._training else 0
                msg = json.dumps({
                    "type": MessageType.SYSTEM_STATS.value,
                    "cpu_pct": stats["cpu_pct"],
                    "per_cpu": stats.get("per_cpu", []),
                    "cpu_cores": stats.get("cpu_cores", 0),
                    "ram_pct": stats["ram_pct"],
                    "ram_used_gb": stats["ram_used_gb"],
                    "ram_total_gb": stats["ram_total_gb"],
                    "swap_used_gb": stats.get("swap_used_gb", 0),
                    "swap_total_gb": stats.get("swap_total_gb", 0),
                    "workers": worker_count,
                    "gpu_available": stats["gpu_available"],
                    "gpu_name": stats.get("gpu_name", "N/A"),
                    "gpu_mem_used_gb": stats.get("gpu_mem_used_gb", 0),
                    "gpu_mem_allocated_gb": stats.get("gpu_mem_allocated_gb", 0),
                    "processes": stats.get("processes", {}),
                    "paused": self._training.paused if self._training else False,
                    "timestamp": round(time.time(), 1),
                })
                dead = set()
                for ws in self._connections:
                    try:
                        await ws.send(msg)
                    except websockets.ConnectionClosed:
                        dead.add(ws)
                self._connections -= dead
            except Exception as exc:
                logger.debug("system_stats broadcast error: %s", exc)

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

def main(host: str = "localhost", port: int = 8080, auto_train: bool = True) -> None:
    """Run the game server."""
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    )
    server = GameServer(host=host, port=port, auto_train=auto_train)
    asyncio.run(server.start())


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
