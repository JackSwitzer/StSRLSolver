"""
Tests for the WebSocket game server.

Unit tests for protocol constructors and GameSession run synchronously.
Integration tests start a real server on an ephemeral port, connect via
the websockets client library, and verify the full request/response protocol.
"""

from __future__ import annotations

import asyncio
from collections import deque
import json
import socket
from types import SimpleNamespace

import pytest
import websockets

from packages.server.game_session import GameSession
from packages.server.protocol import (
    make_game_created,
    make_actions,
    make_action_result,
    make_observation,
    make_step,
    make_game_over,
    make_error,
    make_metrics_history,
)
from packages.server.ws_server import GameServer


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _find_free_port() -> int:
    """Find an available TCP port."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("", 0))
        return s.getsockname()[1]


async def _run_with_server(coro_factory):
    """Start a server, run *coro_factory(url)* against it, then tear down."""
    port = _find_free_port()
    server = GameServer(host="localhost", port=port)
    ws_server = await websockets.serve(server.handler, server.host, server.port)
    url = f"ws://localhost:{port}"
    try:
        await coro_factory(url)
    finally:
        ws_server.close()
        await ws_server.wait_closed()


# ---------------------------------------------------------------------------
# Unit tests: protocol message constructors
# ---------------------------------------------------------------------------


class TestProtocol:
    """Test protocol message constructors produce valid JSON-serializable dicts."""

    def test_make_game_created(self):
        msg = make_game_created("sess123", {"phase": "neow"})
        assert msg["type"] == "game_created"
        assert msg["session_id"] == "sess123"
        assert msg["observation"]["phase"] == "neow"

    def test_make_actions(self):
        msg = make_actions([{"id": "end_turn", "type": "end_turn"}])
        assert msg["type"] == "actions"
        assert len(msg["actions"]) == 1

    def test_make_action_result(self):
        msg = make_action_result({"success": True}, {"phase": "combat"})
        assert msg["type"] == "action_result"
        assert "game_over" not in msg

    def test_make_action_result_game_over(self):
        msg = make_action_result(
            {"success": True},
            {"phase": "run_complete"},
            game_over=True,
            game_won=True,
        )
        assert msg["game_over"] is True
        assert msg["won"] is True

    def test_make_observation(self):
        msg = make_observation({"phase": "map"})
        assert msg["type"] == "observation"
        assert msg["observation"]["phase"] == "map"

    def test_make_step(self):
        msg = make_step(5, {"phase": "combat"}, {"id": "end_turn"})
        assert msg["type"] == "step"
        assert msg["step"] == 5

    def test_make_game_over(self):
        msg = make_game_over(True, {"phase": "run_complete"})
        assert msg["type"] == "game_over"
        assert msg["won"] is True

    def test_make_error(self):
        msg = make_error("something broke", "take_action")
        assert msg["type"] == "error"
        assert "something broke" in msg["error"]
        assert msg["request_type"] == "take_action"

    def test_make_error_no_request_type(self):
        msg = make_error("bad")
        assert msg["type"] == "error"
        assert "request_type" not in msg

    def test_make_metrics_history(self):
        msg = make_metrics_history([1.0, 2.0], [0.3], [0.0, 1.0])
        assert msg["type"] == "metrics_history"
        assert msg["floor_history"] == [1.0, 2.0]
        assert msg["loss_history"] == [0.3]
        assert msg["win_history"] == [0.0, 1.0]

    def test_all_messages_json_serializable(self):
        """Every constructor must produce JSON-serializable output."""
        messages = [
            make_game_created("id", {}),
            make_actions([]),
            make_action_result({}, {}),
            make_observation({}),
            make_step(1, {}, {}),
            make_game_over(False, {}),
            make_error("err"),
            make_metrics_history([], [], []),
        ]
        for msg in messages:
            json.dumps(msg)  # must not raise


# ---------------------------------------------------------------------------
# Unit tests: GameSession
# ---------------------------------------------------------------------------


class TestGameSession:
    """Test GameSession wrapping GameRunner."""

    def test_create_session(self):
        session = GameSession(seed="TEST", ascension=0, character="Watcher")
        assert session.session_id is not None
        assert session.step_count == 0
        assert not session.game_over

    def test_custom_session_id(self):
        session = GameSession(seed="TEST", ascension=0, session_id="custom42")
        assert session.session_id == "custom42"

    def test_get_observation(self):
        session = GameSession(seed="TEST", ascension=0, character="Watcher")
        obs = session.get_observation()
        assert "phase" in obs
        assert "run" in obs

    def test_get_actions(self):
        session = GameSession(seed="TEST", ascension=0, character="Watcher")
        actions = session.get_actions()
        assert isinstance(actions, list)
        assert len(actions) > 0

    def test_take_action_increments_step(self):
        session = GameSession(seed="TEST", ascension=0, character="Watcher")
        actions = session.get_actions()
        assert len(actions) > 0
        session.take_action(actions[0])
        assert session.step_count == 1
        assert len(session.action_history) == 1

    def test_game_won_starts_false(self):
        session = GameSession(seed="TEST", ascension=0)
        assert session.game_won is False


# ---------------------------------------------------------------------------
# Integration tests: WebSocket server
# ---------------------------------------------------------------------------


def test_server_starts_and_accepts_connection():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({"type": "get_observation"}))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            # No session yet -> error
            assert resp["type"] == "error"

    asyncio.run(_run_with_server(_test))


def test_new_game_returns_observation():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({
                "type": "new_game",
                "seed": "TEST123",
                "ascension": 0,
                "character": "Watcher",
            }))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "game_created"
            assert "session_id" in resp
            assert "observation" in resp
            assert "phase" in resp["observation"]

    asyncio.run(_run_with_server(_test))


def test_get_actions_after_new_game():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({
                "type": "new_game",
                "seed": "ABC",
                "ascension": 0,
            }))
            await asyncio.wait_for(ws.recv(), timeout=5)

            await ws.send(json.dumps({"type": "get_actions"}))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "actions"
            assert isinstance(resp["actions"], list)
            assert len(resp["actions"]) > 0

    asyncio.run(_run_with_server(_test))


def test_take_action_returns_result():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({
                "type": "new_game",
                "seed": "XYZ",
                "ascension": 0,
            }))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "game_created"

            await ws.send(json.dumps({"type": "get_actions"}))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            actions = resp["actions"]
            assert len(actions) > 0

            await ws.send(json.dumps({
                "type": "take_action",
                "action": actions[0],
            }))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "action_result"
            assert "observation" in resp

    asyncio.run(_run_with_server(_test))


def test_metrics_history_bootstrap_matches_dashboard_shape():
    async def _test():
        port = _find_free_port()
        server = GameServer(host="localhost", port=port)
        server._training = SimpleNamespace(
            metrics_history=deque(
                [
                    {"floor": 6.5, "loss": 0.25, "win_rate": 0.0},
                    {"floor": 8.0, "loss": 0.15, "win_rate": 1.0},
                ],
                maxlen=1000,
            ),
        )
        ws_server = await websockets.serve(server.handler, server.host, server.port)
        try:
            async with websockets.connect(f"ws://localhost:{port}") as ws:
                resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
                assert resp == {
                    "type": "metrics_history",
                    "floor_history": [6.5, 8.0],
                    "loss_history": [0.25, 0.15],
                    "win_history": [0.0, 1.0],
                }
        finally:
            ws_server.close()
            await ws_server.wait_closed()

    asyncio.run(_test())


def test_get_observation():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({
                "type": "new_game",
                "seed": "OBS",
                "ascension": 0,
            }))
            await asyncio.wait_for(ws.recv(), timeout=5)

            await ws.send(json.dumps({"type": "get_observation"}))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "observation"
            assert "observation" in resp
            assert "run" in resp["observation"]

    asyncio.run(_run_with_server(_test))


def test_auto_play_streams_steps():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({
                "type": "new_game",
                "seed": "AUTO",
                "ascension": 0,
            }))
            await asyncio.wait_for(ws.recv(), timeout=5)

            await ws.send(json.dumps({
                "type": "auto_play",
                "steps": 3,
                "delay_ms": 0,
            }))

            messages = []
            for _ in range(3):
                try:
                    raw = await asyncio.wait_for(ws.recv(), timeout=5)
                    msg = json.loads(raw)
                    messages.append(msg)
                    if msg["type"] == "game_over":
                        break
                except asyncio.TimeoutError:
                    break

            assert len(messages) > 0
            for msg in messages:
                assert msg["type"] in ("step", "game_over", "error")

    asyncio.run(_run_with_server(_test))


def test_game_over_sends_terminal():
    """Play enough steps to reach a terminal state (win or loss)."""

    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({
                "type": "new_game",
                "seed": "TERMINAL",
                "ascension": 0,
            }))
            await asyncio.wait_for(ws.recv(), timeout=5)

            await ws.send(json.dumps({
                "type": "auto_play",
                "steps": 5000,
                "delay_ms": 0,
            }))

            last_msg = None
            while True:
                try:
                    raw = await asyncio.wait_for(ws.recv(), timeout=30)
                    last_msg = json.loads(raw)
                    if last_msg.get("type") == "game_over":
                        break
                    if last_msg.get("type") == "error":
                        break
                except asyncio.TimeoutError:
                    break

            assert last_msg is not None
            assert last_msg["type"] == "game_over"
            assert "won" in last_msg
            assert "observation" in last_msg

    asyncio.run(_run_with_server(_test))


def test_invalid_json_returns_error():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send("not valid json {{{")
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "error"

    asyncio.run(_run_with_server(_test))


def test_unknown_message_type_returns_error():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({"type": "foobar"}))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "error"
            assert "foobar" in resp["error"]

    asyncio.run(_run_with_server(_test))


def test_take_action_without_session():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({
                "type": "take_action",
                "action": {"id": "end_turn"},
            }))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "error"
            assert "No active game session" in resp["error"]

    asyncio.run(_run_with_server(_test))


def test_take_action_missing_action_field():
    async def _test(url):
        async with websockets.connect(url) as ws:
            await ws.send(json.dumps({
                "type": "new_game",
                "seed": "MISS",
                "ascension": 0,
            }))
            await asyncio.wait_for(ws.recv(), timeout=5)

            await ws.send(json.dumps({"type": "take_action"}))
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
            assert resp["type"] == "error"
            assert "Missing" in resp["error"]

    asyncio.run(_run_with_server(_test))
