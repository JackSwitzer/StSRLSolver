"""
Combat Search Server - TCP server for Java ↔ Python communication.

Listens on port 9998 for search requests from Java mod.
Uses the existing ParallelSimulator for MCTS-based tree search.
"""

from __future__ import annotations

import json
import logging
import os
import socket
import struct
import threading
import time
from typing import Any, Callable, Dict, List, Optional

from .protocol import (
    SearchRequest,
    SearchResponse,
    VerifyStateRequest,
    VerifyResponse,
    parse_message,
    create_response,
    BestLine,
    ActionInfo,
    ExpectedOutcome,
)
from .state_converter import (
    json_to_combat_state,
    request_to_combat_state,
    extract_rng_state,
    compare_states,
)


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger("SearchServer")


# Default port for search server
DEFAULT_PORT = 9998
DEFAULT_HOST = "127.0.0.1"


class CombatSearchServer:
    """
    TCP server for combat search requests.

    Accepts connections from Java mod, parses search requests,
    runs MCTS search, and returns best plays.
    """

    def __init__(
        self,
        host: str = DEFAULT_HOST,
        port: int = DEFAULT_PORT,
        search_budget: int = 1000,
        n_workers: int = 0,
    ):
        """
        Initialize the search server.

        Args:
            host: Host to bind to (default: localhost)
            port: Port to listen on (default: 9998)
            search_budget: Default MCTS iterations per search
            n_workers: Number of worker processes (0 = auto)
        """
        self.host = host
        self.port = port
        self.search_budget = search_budget
        self.n_workers = n_workers

        self._socket: Optional[socket.socket] = None
        self._running = False
        self._simulator = None
        self._accept_thread: Optional[threading.Thread] = None
        self._client_threads: List[threading.Thread] = []
        self._card_registry: Dict[str, dict] = {}

        # Stats
        self._total_requests = 0
        self._total_search_time_ms = 0.0

        # Verification logging
        self._log_dir = os.path.expanduser("~/Desktop/SlayTheSpireRL/logs/verification")
        os.makedirs(self._log_dir, exist_ok=True)

    def start(self):
        """Start the server."""
        if self._running:
            logger.warning("Server already running")
            return

        # Initialize simulator
        from ..simulation.engine import ParallelSimulator, SimulationConfig

        config = SimulationConfig(n_workers=self.n_workers)
        self._simulator = ParallelSimulator(config=config)
        logger.info(f"Initialized ParallelSimulator with {config.n_workers} workers")

        # Create socket
        self._socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

        try:
            self._socket.bind((self.host, self.port))
            self._socket.listen(5)
            self._running = True

            logger.info(f"Search server listening on {self.host}:{self.port}")

            # Start accept thread
            self._accept_thread = threading.Thread(target=self._accept_loop, daemon=True)
            self._accept_thread.start()

        except OSError as e:
            logger.error(f"Failed to bind to {self.host}:{self.port}: {e}")
            raise

    def stop(self):
        """Stop the server."""
        self._running = False

        if self._socket:
            self._socket.close()
            self._socket = None

        if self._simulator:
            self._simulator.shutdown()
            self._simulator = None

        logger.info("Search server stopped")

    def _accept_loop(self):
        """Accept incoming connections."""
        while self._running:
            try:
                client_socket, addr = self._socket.accept()
                logger.info(f"Client connected from {addr}")

                # Handle client in new thread
                client_thread = threading.Thread(
                    target=self._handle_client,
                    args=(client_socket, addr),
                    daemon=True,
                )
                client_thread.start()
                self._client_threads.append(client_thread)

            except OSError:
                if self._running:
                    logger.error("Accept failed")
                break

    def _handle_client(self, client_socket: socket.socket, addr):
        """Handle a connected client."""
        try:
            while self._running:
                # Read message (length-prefixed)
                data = self._read_message(client_socket)
                if data is None:
                    break

                # Parse request
                request = parse_message(data)
                if request is None:
                    logger.warning(f"Failed to parse message: {data[:200]}...")
                    continue

                # Handle request
                if isinstance(request, SearchRequest):
                    response = self._handle_search_request(request)
                elif isinstance(request, VerifyStateRequest):
                    response = self._handle_verify_request(request)
                else:
                    logger.warning(f"Unknown request type: {type(request)}")
                    continue

                # Send response
                self._send_message(client_socket, response.to_json())

        except (ConnectionResetError, BrokenPipeError):
            logger.info(f"Client {addr} disconnected")
        except Exception as e:
            logger.error(f"Error handling client {addr}: {e}", exc_info=True)
        finally:
            client_socket.close()

    def _read_message(self, sock: socket.socket) -> Optional[str]:
        """
        Read a length-prefixed message from socket.

        Protocol: 4-byte big-endian length prefix + JSON payload
        """
        try:
            # Read length prefix
            length_bytes = self._recv_all(sock, 4)
            if not length_bytes:
                return None

            length = struct.unpack(">I", length_bytes)[0]

            # Sanity check
            if length > 10_000_000:  # 10 MB max
                logger.error(f"Message too large: {length} bytes")
                return None

            # Read payload
            payload = self._recv_all(sock, length)
            if not payload:
                return None

            return payload.decode("utf-8")

        except (socket.error, struct.error) as e:
            logger.debug(f"Read error: {e}")
            return None

    def _recv_all(self, sock: socket.socket, length: int) -> Optional[bytes]:
        """Receive exactly `length` bytes from socket."""
        data = b""
        while len(data) < length:
            chunk = sock.recv(length - len(data))
            if not chunk:
                return None
            data += chunk
        return data

    def _send_message(self, sock: socket.socket, message: str):
        """
        Send a length-prefixed message to socket.

        Protocol: 4-byte big-endian length prefix + JSON payload
        """
        payload = message.encode("utf-8")
        length = struct.pack(">I", len(payload))
        sock.sendall(length + payload)

    def _handle_search_request(self, request: SearchRequest) -> SearchResponse:
        """Handle a search request."""
        start_time = time.perf_counter()
        self._total_requests += 1

        logger.info(f"Search request {request.request_id}")

        try:
            # Convert to CombatState
            combat_state = request_to_combat_state(request, self._card_registry)

            # Log state summary
            logger.debug(
                f"State: HP={combat_state.player.hp}, "
                f"Energy={combat_state.energy}, "
                f"Hand={len(combat_state.hand)}, "
                f"Enemies={len(combat_state.enemies)}"
            )

            # Get search budget from request or use default
            budget = request.search_params.get("budget_ms", self.search_budget)
            budget_iterations = request.search_params.get("iterations", budget)

            # Run MCTS search
            result = self._simulator.find_best_play(
                combat_state=combat_state,
                search_budget=budget_iterations,
            )

            elapsed_ms = (time.perf_counter() - start_time) * 1000
            self._total_search_time_ms += elapsed_ms

            # Create response
            response = create_response(
                request_id=request.request_id,
                best_action=result.best_action,
                action_scores=result.action_scores,
                search_time_ms=elapsed_ms,
                nodes_explored=result.nodes_explored,
                combat_state=combat_state,
                hand=combat_state.hand,
                enemies=combat_state.enemies,
            )

            # Enrich with expected outcome from simulation
            if response.best_line and result.best_action:
                outcome = self._simulate_action_outcome(
                    combat_state, result.best_action
                )
                response.best_line.expected_outcome = outcome
                response.best_line.display_text = self._format_display_text(
                    response.best_line, outcome
                )

            logger.info(
                f"Search complete: {response.best_line.display_text if response.best_line else 'No action'} "
                f"({elapsed_ms:.0f}ms, {result.nodes_explored} nodes)"
            )

            return response

        except Exception as e:
            logger.error(f"Search failed: {e}", exc_info=True)
            return SearchResponse(
                request_id=request.request_id,
                error=str(e),
                search_time_ms=(time.perf_counter() - start_time) * 1000,
            )

    def _simulate_action_outcome(
        self, state, action
    ) -> ExpectedOutcome:
        """Simulate a single action to get expected outcome."""
        from ..state.combat import PlayCard, UsePotion, EndTurn

        initial_hp = state.player.hp
        initial_block = state.player.block
        total_enemy_hp = sum(e.hp for e in state.enemies if not e.is_dead)

        # Run single-action simulation
        result = self._simulator.simulate_combat_single(
            combat_state=state,
            actions=[action] if not isinstance(action, EndTurn) else [],
            max_turns=1,
        )

        final_enemy_hp = sum(
            e.hp for e in result.final_state.enemies if not e.is_dead
        ) if result.final_state else total_enemy_hp

        return ExpectedOutcome(
            hp_lost=initial_hp - (result.final_state.player.hp if result.final_state else initial_hp),
            damage_dealt=total_enemy_hp - final_enemy_hp,
            enemy_killed=result.victory if result else False,
            player_dead=result.final_state.player.hp <= 0 if result.final_state else False,
            block_remaining=result.final_state.player.block if result.final_state else 0,
            energy_remaining=result.final_state.energy if result.final_state else 0,
        )

    def _format_display_text(self, line: BestLine, outcome: ExpectedOutcome) -> str:
        """Format display text for overlay."""
        parts = []

        # Action description
        for action in line.actions:
            if action.action_type == "card":
                text = action.card_id or "Card"
                if action.target_name:
                    text += f" -> {action.target_name}"
                parts.append(text)
            elif action.action_type == "potion":
                text = f"Potion [{action.potion_slot}]"
                if action.target_name:
                    text += f" -> {action.target_name}"
                parts.append(text)
            elif action.action_type == "end_turn":
                parts.append("End Turn")

        action_text = " + ".join(parts) if parts else "?"

        # Outcome
        if outcome.enemy_killed:
            return f"{action_text} = KILL"
        elif outcome.hp_lost == 0:
            return f"{action_text} = 0 damage taken"
        elif outcome.hp_lost > 0:
            return f"{action_text} = {outcome.hp_lost} damage taken"
        else:
            return f"{action_text} = {outcome.damage_dealt} dealt"

    def _handle_verify_request(self, request: VerifyStateRequest) -> VerifyResponse:
        """Handle a state verification request."""
        logger.info(f"Verify request {request.request_id}")

        # Compare predicted vs actual
        predicted_state = json_to_combat_state(request.predicted_state)
        mismatches = compare_states(predicted_state, request.actual_state)

        if mismatches:
            # Log mismatches
            self._log_mismatch(request, mismatches)

            diagnosis = "; ".join(m.get("diagnosis", "") for m in mismatches[:3])

            logger.warning(
                f"State mismatch: {len(mismatches)} differences. "
                f"First: {mismatches[0]}"
            )

            return VerifyResponse(
                request_id=request.request_id,
                matches=False,
                mismatches=mismatches,
                diagnosis=diagnosis,
            )

        return VerifyResponse(
            request_id=request.request_id,
            matches=True,
        )

    def _log_mismatch(
        self, request: VerifyStateRequest, mismatches: List[Dict]
    ):
        """Log mismatch to file for analysis."""
        timestamp = int(time.time())
        filename = f"mismatch_{self._total_requests:04d}_{timestamp}.json"
        filepath = os.path.join(self._log_dir, filename)

        log_data = {
            "request_id": request.request_id,
            "timestamp": timestamp,
            "action_taken": request.action_taken,
            "predicted_state": request.predicted_state,
            "actual_state": request.actual_state,
            "mismatches": mismatches,
        }

        try:
            with open(filepath, "w") as f:
                json.dump(log_data, f, indent=2)
            logger.info(f"Mismatch logged to {filepath}")
        except Exception as e:
            logger.error(f"Failed to log mismatch: {e}")

    def get_stats(self) -> Dict[str, Any]:
        """Get server statistics."""
        avg_time = (
            self._total_search_time_ms / self._total_requests
            if self._total_requests > 0 else 0
        )
        return {
            "total_requests": self._total_requests,
            "total_search_time_ms": self._total_search_time_ms,
            "avg_search_time_ms": avg_time,
            "port": self.port,
            "running": self._running,
        }


# =============================================================================
# CLI Entry Point
# =============================================================================


def main():
    """CLI entry point for running the server."""
    import argparse

    parser = argparse.ArgumentParser(description="Combat Search Server")
    parser.add_argument("--host", default=DEFAULT_HOST, help="Host to bind to")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="Port to listen on")
    parser.add_argument("--budget", type=int, default=1000, help="MCTS search budget")
    parser.add_argument("--workers", type=int, default=0, help="Worker processes (0=auto)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose logging")

    args = parser.parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    server = CombatSearchServer(
        host=args.host,
        port=args.port,
        search_budget=args.budget,
        n_workers=args.workers,
    )

    try:
        server.start()
        logger.info("Press Ctrl+C to stop")
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        logger.info("Shutting down...")
    finally:
        server.stop()


if __name__ == "__main__":
    main()
