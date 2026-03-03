"""Training runner: agents playing continuously with MCTS, streaming to WS clients.

Spawns worker processes running GameRunner + StSAgent in infinite loops.
TrainingCoordinator collects events and pushes to WebSocket subscribers.
Logs all episode data to disk for analysis.
"""

from __future__ import annotations

import asyncio
import json
import logging
import multiprocessing as mp
import os
import resource
import time
from collections import deque
from pathlib import Path
from typing import Any, Deque, Dict, List, Optional, Set

logger = logging.getLogger(__name__)

AGENT_NAMES = [
    "Oracle", "Gambler", "Wanderer", "Wildcard",
    "Sentinel", "Guardian", "Tactician", "Drifter",
    "Spectre", "Pilgrim", "Vanguard", "Mystic",
    "Reaper", "Nomad", "Arbiter", "Seeker",
]

LOG_DIR = Path("logs")

# Card type heuristics (prefix-based)
_ATTACK_CARDS = {"Strike", "Eruption", "Tantrum", "Ragnarok", "Conclude", "FlyingSleeves",
                 "SashWhip", "Wallop", "EmptyFist", "FlurryOfBlows", "Smite", "Pressure",
                 "ReachHeaven", "Blasphemy", "Brilliance", "Crush", "SignatureMove",
                 "TalkToTheHand", "Judgement", "LessonLearned", "WindmillStrike",
                 "WheelKick", "Vault", "CarveReality", "FearNoEvil", "FollowUp", "Weave"}
_POWER_CARDS = {"MentalFortress", "Rushdown", "BattleHymn", "Foresight", "LikeWater",
                "DevaForm", "Devotion", "Establishment", "MasterReality", "Study", "Worship"}


def _adapt_combat_obs(raw: dict) -> dict:
    """Transform engine combat observation into frontend CombatState format."""
    player = raw.get("player", {})
    # Powers: dict → array
    statuses = player.get("statuses", {})
    powers = [{"id": k, "name": k.replace("_", " "), "amount": v} for k, v in statuses.items()] if isinstance(statuses, dict) else []

    # Enemies
    enemies = []
    for e in raw.get("enemies", []):
        eid = e.get("id", "Unknown")
        # Humanize name from ID
        name = eid.replace("_", " ").replace("S ", "(S) ").replace("M ", "(M) ").replace("L ", "(L) ")
        # Size heuristic
        mhp = e.get("max_hp", 0)
        size = "large" if mhp >= 100 else "medium" if mhp >= 40 else "small"
        # Intent
        dmg = e.get("move_damage", -1)
        hits = e.get("move_hits", 1)
        blk = e.get("move_block", 0)
        if dmg > 0:
            intent_type = "attack"
        elif blk > 0:
            intent_type = "defend"
        else:
            intent_type = "buff"
        # Enemy powers
        e_statuses = e.get("statuses", {})
        e_powers = [{"id": k, "name": k.replace("_", " "), "amount": v} for k, v in e_statuses.items()] if isinstance(e_statuses, dict) else []

        enemies.append({
            "id": eid, "name": name,
            "hp": e.get("hp", 0), "max_hp": mhp, "block": e.get("block", 0),
            "size": size,
            "intent": {"type": intent_type, "damage": max(0, dmg), "hits": hits},
            "powers": e_powers,
        })

    # Hand: string IDs → card objects
    hand = []
    card_costs = raw.get("card_costs", {})
    for card_id in raw.get("hand", []):
        base = card_id.rstrip("+")
        upgraded = card_id.endswith("+")
        name = base.replace("_P", "").replace("_", " ")
        # Determine type
        if base in _ATTACK_CARDS or "Strike" in base:
            ctype = "attack"
        elif base in _POWER_CARDS:
            ctype = "power"
        elif base == "AscendersBane" or base == "Injury" or "Curse" in base:
            ctype = "curse"
        else:
            ctype = "skill"
        cost = card_costs.get(card_id, 1)
        hand.append({
            "id": card_id, "name": name, "cost": cost,
            "type": ctype, "upgraded": upgraded, "playable": True,
        })

    return {
        "player": {"hp": player.get("hp", 0), "max_hp": player.get("max_hp", 72), "block": player.get("block", 0), "powers": powers},
        "enemies": enemies,
        "hand": hand,
        "draw_pile_count": len(raw.get("draw_pile", [])),
        "discard_pile_count": len(raw.get("discard_pile", [])),
        "exhaust_pile_count": len(raw.get("exhaust_pile", [])),
        "energy": raw.get("energy", 3),
        "max_energy": raw.get("max_energy", 3),
        "turn": raw.get("turn", 1),
        "stance": raw.get("stance", "Neutral"),
    }


# =========================================================================
# Worker process (top-level for pickling)
# =========================================================================

def _agent_worker(
    agent_id: int,
    event_queue: mp.Queue,
    stop_event: mp.Event,
    config: Dict[str, Any],
) -> None:
    """Run games in an infinite loop, pushing events to queue."""
    from packages.engine.game import GameRunner, GamePhase
    from packages.training.planner import StSAgent

    base_sims = config.get("mcts_sims", 64)
    ascension = config.get("ascension", 20)
    character = config.get("character", "Watcher")
    initial_seed = config.get("initial_seed", "Test123")

    agent = StSAgent(combat_sims=base_sims, temperature=0.0)
    episode = config.get("start_episode", 0)
    total_wins = config.get("start_wins", 0)
    conquered_initial = config.get("conquered_initial", False)

    # Per-agent log file
    log_dir = Path(config.get("log_dir", "logs"))
    log_dir.mkdir(parents=True, exist_ok=True)
    log_path = log_dir / f"agent_{agent_id}.jsonl"
    log_file = open(log_path, "a")

    while not stop_event.is_set():
        # Seed strategy
        if not conquered_initial:
            seed = initial_seed
        else:
            seed = f"random_{agent_id}_{episode}"

        try:
            runner = GameRunner(seed=seed, ascension=ascension, character=character, verbose=False)
        except Exception as exc:
            _put_safe(event_queue, {"type": "error", "agent_id": agent_id, "msg": str(exc)})
            time.sleep(2)
            continue

        step = 0
        mcts_calls = 0
        mcts_total_ms = 0.0
        trivial = 0
        start_time = time.monotonic()
        last_heartbeat = time.monotonic()
        hp_at_floor: Dict[int, int] = {}
        current_floor = 0

        while not runner.game_over and step < 3000 and not stop_event.is_set():
            # Track HP per floor
            rs = runner.run_state
            fl = getattr(rs, "floor", 0)
            if fl != current_floor:
                current_floor = fl
                hp_at_floor[fl] = getattr(rs, "current_hp", 0)

            try:
                actions = runner.get_available_actions()
            except Exception:
                break
            if not actions:
                break

            mcts_data = None

            if len(actions) == 1:
                action = actions[0]
                trivial += 1
                time.sleep(0.0005)  # Yield CPU on trivial decisions
            else:
                # Adaptive sims
                n_actions = len(actions)
                if n_actions <= 2:
                    agent.combat_mcts.num_simulations = max(16, base_sims // 4)
                elif n_actions <= 4:
                    agent.combat_mcts.num_simulations = max(32, base_sims // 2)
                else:
                    agent.combat_mcts.num_simulations = base_sims

                t0 = time.monotonic()
                try:
                    action = agent.get_action(runner)
                except Exception:
                    action = actions[0]
                elapsed_ms = (time.monotonic() - t0) * 1000

                if runner.phase == GamePhase.COMBAT and elapsed_ms > 5:
                    mcts_calls += 1
                    mcts_total_ms += elapsed_ms

                    # Extract MCTS data for viewer
                    root = getattr(agent.combat_mcts, '_last_root', None)
                    if root and root.children:
                        top_actions = []
                        total_visits = sum(c.visits for c in root.children.values())
                        for act, child in sorted(
                            root.children.items(), key=lambda x: x[1].visits, reverse=True,
                        )[:8]:
                            top_actions.append({
                                "id": str(act),
                                "visits": child.visits,
                                "pct": round(child.visits / max(total_visits, 1), 3),
                                "q": round(child.value, 3),
                                "selected": False,
                            })
                        if top_actions:
                            top_actions[0]["selected"] = True
                        mcts_data = {
                            "sims": agent.combat_mcts.num_simulations,
                            "elapsed_ms": round(elapsed_ms, 1),
                            "root_value": round(root.value, 3),
                            "actions": top_actions,
                        }

            if action is None:
                action = actions[0]

            try:
                runner.take_action(action)
            except Exception:
                break
            step += 1

            # Heartbeat at ~2Hz
            now = time.monotonic()
            if now - last_heartbeat > 0.5:
                last_heartbeat = now
                rs = runner.run_state
                phase_str = str(runner.phase).split(".")[-1] if runner.phase else "UNKNOWN"
                hb: Dict[str, Any] = {
                    "type": "heartbeat",
                    "agent_id": agent_id,
                    "phase": phase_str,
                    "floor": getattr(rs, "floor", 0),
                    "act": getattr(rs, "act", 1),
                    "hp": getattr(rs, "current_hp", 0),
                    "max_hp": getattr(rs, "max_hp", 72),
                    "seed": seed,
                    "episode": episode,
                    "wins": total_wins,
                    "step": step,
                }
                # Combat snapshot for focused agent rendering
                if "COMBAT" in phase_str:
                    try:
                        obs = runner.get_observation()
                        combat_obs = obs.get("combat")
                        if combat_obs:
                            adapted = _adapt_combat_obs(combat_obs)
                            hb["combat"] = adapted
                            # Compact combat info for grid cards
                            if adapted["enemies"]:
                                e = adapted["enemies"][0]
                                hb["enemy_name"] = e["name"]
                                hb["enemy_hp"] = e["hp"]
                                hb["enemy_max_hp"] = e["max_hp"]
                            hb["hand_size"] = len(adapted["hand"])
                            hb["turn"] = adapted["turn"]
                            hb["stance"] = adapted["stance"]
                    except Exception:
                        pass
                _put_safe(event_queue, hb)

            # Send MCTS result
            if mcts_data:
                _put_safe(event_queue, {"type": "mcts_result", "agent_id": agent_id, **mcts_data})

        # Episode complete
        duration = time.monotonic() - start_time
        won = runner.game_won
        rs = runner.run_state
        final_floor = getattr(rs, "floor", 0)
        final_hp = getattr(rs, "current_hp", 0)

        if won and seed == initial_seed:
            conquered_initial = True
        if won:
            total_wins += 1

        summary = {
            "type": "episode",
            "agent_id": agent_id,
            "seed": seed,
            "won": won,
            "floors_reached": final_floor,
            "hp_remaining": final_hp,
            "total_steps": step,
            "duration": round(duration, 1),
            "episode": episode,
            "wins": total_wins,
            "mcts_calls": mcts_calls,
            "mcts_avg_ms": round(mcts_total_ms / max(mcts_calls, 1), 1),
            "trivial": trivial,
            "deck_size": len(getattr(rs, "deck", [])),
            "relic_count": len(getattr(rs, "relics", [])),
        }
        _put_safe(event_queue, summary)

        # Log to disk
        log_file.write(json.dumps(summary) + "\n")
        log_file.flush()

        episode += 1

    log_file.close()


def _put_safe(queue: mp.Queue, data: Dict) -> None:
    try:
        queue.put_nowait(data)
    except Exception:
        pass


# =========================================================================
# TrainingCoordinator
# =========================================================================

class TrainingCoordinator:
    """Manages agent workers and distributes events to WS clients."""

    def __init__(
        self,
        num_agents: int = 4,
        mcts_sims: int = 64,
        ascension: int = 20,
        initial_seed: str = "Test123",
    ):
        self.num_agents = num_agents
        self.config: Dict[str, Any] = {
            "mcts_sims": mcts_sims,
            "ascension": ascension,
            "character": "Watcher",
            "initial_seed": initial_seed,
            "log_dir": str(LOG_DIR),
        }
        self.event_queue: Optional[mp.Queue] = None
        self.stop_event: Optional[mp.Event] = None
        self.processes: List[mp.Process] = []
        self.running = False

        # Per-agent state
        self.agents: Dict[int, Dict[str, Any]] = {}
        for i in range(num_agents):
            self.agents[i] = {
                "id": i,
                "name": AGENT_NAMES[i] if i < len(AGENT_NAMES) else f"Agent_{i}",
                "phase": "STARTING", "floor": 0, "act": 1,
                "hp": 72, "max_hp": 72, "seed": initial_seed,
                "episode": 0, "wins": 0, "status": "starting",
            }

        # Aggregate stats
        self.total_episodes = 0
        self.total_wins = 0
        self.recent_results: Deque[bool] = deque(maxlen=100)
        self.recent_floors: Deque[int] = deque(maxlen=100)
        self.mcts_times: Deque[float] = deque(maxlen=100)
        self.episode_log: Deque[Dict] = deque(maxlen=200)
        self.start_time = 0.0

        # Latest MCTS result per agent
        self.latest_mcts: Dict[int, Dict] = {}

        # Latest combat state per agent (for immediate send on focus)
        self.latest_combat: Dict[int, Dict] = {}

        # WS subscribers
        self._subscribers: Set[int] = set()
        self._focused: Dict[int, Set[int]] = {}  # conn_id -> set of focused agent_ids
        self._broadcast_queues: Dict[int, asyncio.Queue] = {}

    async def start(self) -> None:
        if self.running:
            return

        LOG_DIR.mkdir(parents=True, exist_ok=True)
        self.event_queue = mp.Queue(maxsize=5000)
        self.stop_event = mp.Event()
        self.start_time = time.time()
        self.running = True

        # Load checkpoints if they exist
        checkpoints = self._load_checkpoints()

        for agent_id in range(self.num_agents):
            cfg = dict(self.config)
            cp = checkpoints.get(agent_id, {})
            cfg["start_episode"] = cp.get("episode", 0)
            cfg["start_wins"] = cp.get("wins", 0)
            cfg["conquered_initial"] = cp.get("conquered_initial", False)

            p = mp.Process(
                target=_agent_worker,
                args=(agent_id, self.event_queue, self.stop_event, cfg),
                daemon=True,
            )
            p.start()
            self.processes.append(p)

        logger.info("Started %d agent workers (lazy MCTS, %d base sims)", len(self.processes), self.config["mcts_sims"])

    async def stop(self) -> None:
        if not self.running:
            return
        self.running = False
        if self.stop_event:
            self.stop_event.set()

        # Save checkpoints
        self._save_checkpoints()

        for p in self.processes:
            p.join(timeout=5)
            if p.is_alive():
                p.kill()
        self.processes.clear()
        logger.info("All workers stopped, checkpoints saved")

    def _load_checkpoints(self) -> Dict[int, Dict]:
        cp_path = LOG_DIR / "checkpoints.json"
        if cp_path.exists():
            try:
                with open(cp_path) as f:
                    return {int(k): v for k, v in json.load(f).items()}
            except Exception:
                pass
        return {}

    def _save_checkpoints(self) -> None:
        cp_path = LOG_DIR / "checkpoints.json"
        data = {}
        for aid, a in self.agents.items():
            data[str(aid)] = {
                "episode": a.get("episode", 0),
                "wins": a.get("wins", 0),
                "conquered_initial": a.get("seed", "") != self.config["initial_seed"],
            }
        try:
            with open(cp_path, "w") as f:
                json.dump(data, f)
        except Exception:
            pass

    def subscribe(self, conn_id: int) -> asyncio.Queue:
        q: asyncio.Queue = asyncio.Queue(maxsize=100)
        self._subscribers.add(conn_id)
        self._broadcast_queues[conn_id] = q
        return q

    def unsubscribe(self, conn_id: int) -> None:
        self._subscribers.discard(conn_id)
        self._broadcast_queues.pop(conn_id, None)
        self._focused.pop(conn_id, None)

    def set_focus(self, conn_id: int, agent_id: int) -> None:
        """Add agent to this connection's focused set and immediately send cached state."""
        if conn_id not in self._focused:
            self._focused[conn_id] = set()
        self._focused[conn_id].add(agent_id)

        # Immediately send cached combat state so client doesn't wait for next heartbeat
        cached_combat = self.latest_combat.get(agent_id)
        if cached_combat:
            self._send_to(conn_id, {"type": "agent_combat", "agent_id": agent_id, "combat": cached_combat})

        # Immediately send cached MCTS result
        cached_mcts = self.latest_mcts.get(agent_id)
        if cached_mcts:
            self._send_to(conn_id, cached_mcts)

    def remove_focus(self, conn_id: int, agent_id: int) -> None:
        """Remove agent from this connection's focused set."""
        focused = self._focused.get(conn_id)
        if focused:
            focused.discard(agent_id)
            if not focused:
                self._focused.pop(conn_id, None)

    def clear_focus(self, conn_id: int) -> None:
        self._focused.pop(conn_id, None)

    async def poll_events(self) -> None:
        """Poll mp.Queue and dispatch to subscribers."""
        last_grid = 0.0
        last_stats = 0.0
        last_resource = 0.0

        while self.running:
            drained = 0
            while drained < 100:
                try:
                    event = self.event_queue.get_nowait()
                except Exception:
                    break
                drained += 1
                self._process_event(event)

            if drained == 0:
                await asyncio.sleep(0.05)
                continue

            now = time.time()

            if now - last_grid > 1.0:
                last_grid = now
                self._broadcast(self._build_grid_update())

            if now - last_stats > 1.0:
                last_stats = now
                self._broadcast(self._build_stats())

            # Resource logging every 30s
            if now - last_resource > 30.0:
                last_resource = now
                self._log_resources()

    def _process_event(self, event: Dict) -> None:
        etype = event.get("type")
        agent_id = event.get("agent_id", -1)

        if etype == "heartbeat":
            if agent_id in self.agents:
                a = self.agents[agent_id]
                for k in ("phase", "floor", "act", "hp", "max_hp", "seed", "episode", "wins",
                          "enemy_name", "enemy_hp", "enemy_max_hp", "hand_size", "turn", "stance"):
                    if k in event:
                        a[k] = event[k]
                a["status"] = "playing"

                # Cache and forward combat snapshot to focused clients
                combat = event.get("combat")
                if combat:
                    self.latest_combat[agent_id] = combat
                    for conn_id, focused_set in self._focused.items():
                        if agent_id in focused_set:
                            self._send_to(conn_id, {"type": "agent_combat", "agent_id": agent_id, "combat": combat})

        elif etype == "episode":
            if agent_id in self.agents:
                a = self.agents[agent_id]
                a["episode"] = event.get("episode", 0) + 1
                a["wins"] = event.get("wins", a.get("wins", 0))
                a["status"] = "restarting"

            self.total_episodes += 1
            if event.get("won"):
                self.total_wins += 1
            self.recent_results.append(event.get("won", False))
            self.recent_floors.append(event.get("floors_reached", 0))
            if event.get("mcts_avg_ms", 0) > 0:
                self.mcts_times.append(event["mcts_avg_ms"])
            self.episode_log.append(event)

            self._broadcast({"type": "agent_episode", **{k: v for k, v in event.items() if k != "type"}})

        elif etype == "mcts_result":
            self.latest_mcts[agent_id] = event
            for conn_id, focused_set in self._focused.items():
                if agent_id in focused_set:
                    self._send_to(conn_id, event)

    def _build_grid_update(self) -> Dict:
        return {
            "type": "grid_update",
            "agents": [dict(a) for a in self.agents.values()],
        }

    def _build_stats(self) -> Dict:
        uptime = time.time() - self.start_time if self.start_time else 0
        wr = sum(self.recent_results) / max(len(self.recent_results), 1)
        af = sum(self.recent_floors) / max(len(self.recent_floors), 1)
        ma = sum(self.mcts_times) / max(len(self.mcts_times), 1) if self.mcts_times else 0
        epm = self.total_episodes / max(uptime / 60, 0.01)
        return {
            "type": "training_stats",
            "total_episodes": self.total_episodes,
            "win_count": self.total_wins,
            "win_rate": round(wr, 3),
            "avg_floor": round(af, 1),
            "mcts_avg_ms": round(ma, 1),
            "eps_per_min": round(epm, 2),
            "uptime": round(uptime, 0),
        }

    def _log_resources(self) -> None:
        try:
            usage = resource.getrusage(resource.RUSAGE_CHILDREN)
            rss_mb = usage.ru_maxrss / 1024 / 1024  # macOS returns bytes
            stats = {
                "ts": time.time(),
                "total_episodes": self.total_episodes,
                "worker_count": len(self.processes),
                "rss_mb": round(rss_mb, 1),
                "user_time_s": round(usage.ru_utime, 1),
                "sys_time_s": round(usage.ru_stime, 1),
            }
            with open(LOG_DIR / "resources.jsonl", "a") as f:
                f.write(json.dumps(stats) + "\n")
        except Exception:
            pass

    def _broadcast(self, msg: Dict) -> None:
        for q in self._broadcast_queues.values():
            try:
                q.put_nowait(msg)
            except asyncio.QueueFull:
                pass

    def _send_to(self, conn_id: int, msg: Dict) -> None:
        q = self._broadcast_queues.get(conn_id)
        if q:
            try:
                q.put_nowait(msg)
            except asyncio.QueueFull:
                pass
