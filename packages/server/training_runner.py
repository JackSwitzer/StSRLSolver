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
from collections import Counter, deque
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

_GOOD_CARDS = frozenset({
    "Adaptation", "Tantrum", "Ragnarok", "MentalFortress", "TalkToTheHand",
    "InnerPeace", "CutThroughFate", "WheelKick", "Conclude", "Wallop",
    "EmptyFist", "Eruption", "FearNoEvil", "Blasphemy", "Brilliance",
})


def _compute_hand_quality(hand: list) -> float:
    """Compute hand quality score (0-1) for meta-learner state."""
    if not hand:
        return 0.0
    good = sum(1 for c in hand if c.rstrip("+") in _GOOD_CARDS)
    return min(1.0, good / max(len(hand), 1) * 2.5)


def _get_seed(agent_id: int, episode: int, initial_seed: str, plays_per_seed: int = 3) -> str:
    """Seed rotation: play each seed N times, then advance. Agents get offset starting seeds."""
    seed_index = episode // plays_per_seed
    # Offset each agent so they explore different seeds
    seed_index += agent_id * 1000
    if seed_index == 0:
        return initial_seed
    return f"Seed_{seed_index}"


def _agent_worker(
    agent_id: int,
    event_queue: mp.Queue,
    stop_event: mp.Event,
    pause_event: mp.Event,
    config: Dict[str, Any],
) -> None:
    """Run games in an infinite loop, pushing events to queue.

    Uses CombatPlanner (turn-level line search) for combat decisions and
    StrategicPlanner for non-combat decisions. Logs compressed episode data.
    """
    from packages.engine.game import GameRunner, GamePhase, CombatAction
    from packages.training.planner import StrategicPlanner
    from packages.training.combat_planner import CombatPlanner
    from packages.training.meta_learner import CombatMetaLearner, CombatLog

    ascension = config.get("ascension", 20)
    character = config.get("character", "Watcher")
    initial_seed = config.get("initial_seed", "Test123")
    plays_per_seed = config.get("plays_per_seed", 3)

    planner = StrategicPlanner()
    combat_planner = CombatPlanner(top_k=5, lookahead_turns=2)
    meta_learner = CombatMetaLearner()

    # Try loading saved meta-learner
    meta_path = Path(config.get("log_dir", "logs")) / "meta_learner.json"
    meta_learner.load(meta_path)

    episode = config.get("start_episode", 0)
    total_wins = config.get("start_wins", 0)

    # Per-agent log file
    log_dir = Path(config.get("log_dir", "logs"))
    log_dir.mkdir(parents=True, exist_ok=True)
    log_path = log_dir / f"agent_{agent_id}.jsonl"
    log_file = open(log_path, "a")

    # Combat logs buffer for meta-learner updates
    combat_log_buffer: List[CombatLog] = []

    while not stop_event.is_set():
        # Pause support: block here until resumed
        while pause_event.is_set() and not stop_event.is_set():
            time.sleep(0.2)

        seed = _get_seed(agent_id, episode, initial_seed, plays_per_seed)

        try:
            runner = GameRunner(seed=seed, ascension=ascension, character=character, verbose=False)
        except Exception as exc:
            _put_safe(event_queue, {"type": "error", "agent_id": agent_id, "msg": str(exc)})
            time.sleep(2)
            continue

        step = 0
        plan_calls = 0
        plan_total_ms = 0.0
        trivial = 0
        start_time = time.monotonic()
        last_heartbeat = time.monotonic()
        hp_history: List[int] = [getattr(runner.run_state, "current_hp", 72)]
        current_floor = 0
        combats: List[Dict] = []
        decisions: List[Dict] = []
        deck_changes: List[str] = []
        initial_deck = Counter(str(c) for c in getattr(runner.run_state, "deck", []))

        # Per-combat tracking
        combat_start_hp = 0
        combat_damage_dealt = 0
        combat_turns = 0
        combat_enemy_id = ""
        combat_lines_considered = 0
        combat_line_rank = 0
        combat_expected_hp_loss = 0
        combat_used_potion = False
        in_combat = False
        combat_turn_snapshots: List[Dict] = []

        while not runner.game_over and step < 3000 and not stop_event.is_set():
            rs = runner.run_state
            fl = getattr(rs, "floor", 0)
            if fl != current_floor:
                current_floor = fl
                hp_history.append(getattr(rs, "current_hp", 0))

            try:
                actions = runner.get_available_actions()
            except Exception:
                break
            if not actions:
                break

            phase = runner.phase
            phase_str = str(phase).split(".")[-1] if phase else "UNKNOWN"
            planner_data = None

            # --- COMBAT: Use CombatPlanner ---
            if phase == GamePhase.COMBAT:
                engine = runner.current_combat

                # Track combat start
                if not in_combat:
                    in_combat = True
                    combat_start_hp = getattr(rs, "current_hp", 0)
                    combat_damage_dealt = 0
                    combat_turns = 0
                    combat_lines_considered = 0
                    combat_line_rank = 0
                    combat_expected_hp_loss = 0
                    combat_used_potion = False
                    combat_turn_snapshots = []
                    if engine and hasattr(engine, "state") and engine.state.enemies:
                        combat_enemy_id = getattr(engine.state.enemies[0], "id", "Unknown")

                if engine and len(actions) > 1:
                    t0 = time.monotonic()
                    try:
                        plan = combat_planner.plan_turn(engine)
                    except Exception:
                        plan = None
                    elapsed_ms = (time.monotonic() - t0) * 1000

                    if plan and plan.card_sequence:
                        plan_calls += 1
                        plan_total_ms += elapsed_ms
                        combat_lines_considered += plan.lines_considered
                        combat_expected_hp_loss += plan.expected_outcome.damage_taken if plan.expected_outcome else 0
                        combat_turns += 1

                        # Record turn snapshot for meta-learner
                        if engine:
                            state = engine.state
                            player_hp = state.player.hp
                            max_hp = max(state.player.max_hp, 1)
                            live_enemies = [e for e in state.enemies if e.hp > 0]
                            combat_turn_snapshots.append({
                                "hp_pct": round(player_hp / max_hp, 2),
                                "enemy_count": len(live_enemies),
                                "energy": state.energy,
                                "stance": getattr(state, "stance", "Neutral"),
                                "hand_quality": _compute_hand_quality(state.hand),
                                "strategy": 2,  # balanced default
                                "player_hp": player_hp,
                            })

                        # Execute the planned card sequence
                        for card_id, target_idx in plan.card_sequence:
                            # Find card in hand by ID
                            hand = getattr(engine.state, "hand", []) if engine else []
                            card_idx = None
                            for i, h_card in enumerate(hand):
                                if h_card == card_id:
                                    card_idx = i
                                    break

                            if card_idx is not None:
                                t_idx = target_idx if target_idx is not None else -1
                                try:
                                    ca = CombatAction(action_type="play_card", card_idx=card_idx, target_idx=t_idx)
                                    runner.take_action(ca)
                                    step += 1
                                    if plan.expected_outcome:
                                        combat_damage_dealt += plan.expected_outcome.damage_dealt
                                except Exception:
                                    break

                        # End turn after playing all cards (only if still in combat)
                        if not runner.game_over and runner.phase == GamePhase.COMBAT:
                            try:
                                runner.take_action(CombatAction(action_type="end_turn"))
                                step += 1
                            except Exception:
                                pass  # Don't break game loop on end_turn failure

                        # Build planner data for viewer
                        if plan.expected_outcome:
                            planner_data = {
                                "type": "planner_result",
                                "agent_id": agent_id,
                                "elapsed_ms": round(elapsed_ms, 1),
                                "lines_considered": plan.lines_considered,
                                "strategy": plan.strategy,
                                "turns_to_kill": plan.turns_to_kill,
                                "expected_hp_loss": plan.expected_hp_loss,
                                "confidence": round(plan.confidence, 2),
                                "cards": [c[0] for c in plan.card_sequence],
                            }

                        # Send planner data and continue to next iteration
                        if planner_data:
                            _put_safe(event_queue, planner_data)

                        # Heartbeat (check after full turn execution)
                        now = time.monotonic()
                        if now - last_heartbeat > 0.5:
                            last_heartbeat = now
                            _send_heartbeat(event_queue, agent_id, runner, seed, episode, total_wins, step)
                        continue

                # Fallback: single action (trivial or no plan)
                if len(actions) == 1:
                    action = actions[0]
                    trivial += 1
                else:
                    action = actions[0]  # fallback

            # --- NON-COMBAT: Use StrategicPlanner ---
            else:
                # Track combat end
                if in_combat:
                    in_combat = False
                    actual_hp_loss = combat_start_hp - getattr(rs, "current_hp", 0)
                    combats.append({
                        "floor": current_floor,
                        "enemy": combat_enemy_id,
                        "turns": combat_turns,
                        "hp_lost": actual_hp_loss,
                        "damage_dealt": combat_damage_dealt,
                        "used_potion": combat_used_potion,
                        "lines_considered": combat_lines_considered,
                        "expected_hp_loss": combat_expected_hp_loss,
                        "actual_hp_loss": actual_hp_loss,
                    })
                    # Save combat log for meta-learner
                    if combat_turn_snapshots:
                        # Backfill hp_lost_this_turn
                        for i, snap in enumerate(combat_turn_snapshots):
                            if i + 1 < len(combat_turn_snapshots):
                                hp_before = snap.get("player_hp", 0)
                                hp_after = combat_turn_snapshots[i + 1].get("player_hp", 0)
                                snap["hp_lost_this_turn"] = max(0, hp_before - hp_after)
                            else:
                                snap["hp_lost_this_turn"] = max(0, snap.get("player_hp", 0) - getattr(rs, "current_hp", 0))
                            snap["kills_this_turn"] = 0  # simplified

                        clog = CombatLog(
                            floor=current_floor,
                            enemy_id=combat_enemy_id,
                            turns=combat_turns,
                            hp_lost=actual_hp_loss,
                            damage_dealt=combat_damage_dealt,
                            turn_snapshots=combat_turn_snapshots,
                        )
                        combat_log_buffer.append(clog)

                if len(actions) == 1:
                    action = actions[0]
                    trivial += 1
                else:
                    # Use strategic planner for non-combat decisions
                    if phase == GamePhase.MAP_NAVIGATION:
                        idx = planner.plan_path_choice(runner, actions)
                        action = actions[min(idx, len(actions) - 1)]
                        decisions.append({
                            "floor": current_floor, "type": "path",
                            "choice": str(action),
                        })
                    elif phase == GamePhase.REST:
                        idx = planner.plan_rest_site(runner, actions)
                        action = actions[min(idx, len(actions) - 1)]
                        hp_pct = round(getattr(rs, "current_hp", 0) / max(getattr(rs, "max_hp", 72), 1), 2)
                        decisions.append({
                            "floor": current_floor, "type": "rest",
                            "choice": str(action), "hp_pct": hp_pct,
                        })
                    else:
                        action = actions[0]

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
                _send_heartbeat(event_queue, agent_id, runner, seed, episode, total_wins, step)

        # Episode complete
        duration = time.monotonic() - start_time
        won = runner.game_won
        rs = runner.run_state
        final_floor = getattr(rs, "floor", 0)
        final_hp = getattr(rs, "current_hp", 0)

        if won:
            total_wins += 1

        # Detect deck changes (Counter handles duplicates correctly)
        final_deck = Counter(str(c) for c in getattr(rs, "deck", []))
        for card, count in (final_deck - initial_deck).items():
            for _ in range(count):
                deck_changes.append(f"+{card}")
        for card, count in (initial_deck - final_deck).items():
            for _ in range(count):
                deck_changes.append(f"-{card}")

        # Compressed episode summary
        summary = {
            "type": "episode",
            "agent_id": agent_id,
            "seed": seed,
            "won": won,
            "floor": final_floor,
            "floors_reached": final_floor,
            "hp_remaining": final_hp,
            "hp_history": hp_history,
            "combats": combats[:10],  # cap to keep size down
            "decisions": decisions[:15],
            "deck_changes": deck_changes[:20],
            "duration_s": round(duration, 1),
            "duration": round(duration, 1),
            "plan_calls": plan_calls,
            "plan_avg_ms": round(plan_total_ms / max(plan_calls, 1), 1),
            "mcts_calls": plan_calls,
            "mcts_avg_ms": round(plan_total_ms / max(plan_calls, 1), 1),
            "total_steps": step,
            "episode": episode,
            "wins": total_wins,
            "trivial": trivial,
            "deck_size": len(getattr(rs, "deck", [])),
            "relic_count": len(getattr(rs, "relics", [])),
        }
        _put_safe(event_queue, summary)

        # Log to disk
        log_file.write(json.dumps(summary) + "\n")
        log_file.flush()

        # Meta-learner: update every 100 episodes
        if len(combat_log_buffer) >= 50:
            meta_learner.update_batch(combat_log_buffer)
            meta_learner.decay_epsilon()
            # Update combat planner weights
            combat_planner.strategy_weights = meta_learner.strategy_modifiers.get(
                meta_learner.get_strategy(0.5, 1, 3, "Neutral", 0.5), None
            )
            combat_log_buffer.clear()
            # Save periodically
            if episode % 100 == 0:
                try:
                    meta_learner.save(meta_path)
                except Exception:
                    pass

        episode += 1

    log_file.close()


def _send_heartbeat(
    event_queue: mp.Queue,
    agent_id: int,
    runner: Any,
    seed: str,
    episode: int,
    total_wins: int,
    step: int,
) -> None:
    """Send a heartbeat event with current agent state."""
    from packages.engine.game import GamePhase

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
        self.pause_event: Optional[mp.Event] = None
        self.processes: List[mp.Process] = []
        self.running = False
        self.paused = False

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
        # Rolling metrics history (1000 entries, ~1 per stats broadcast)
        self.metrics_history: Deque[Dict] = deque(maxlen=1000)
        self._last_episode_count_for_metrics = 0

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
        self.pause_event = mp.Event()
        self.start_time = time.time()
        self.running = True

        # Load checkpoints if they exist
        checkpoints = self._load_checkpoints()

        for agent_id in range(self.num_agents):
            self._start_worker(agent_id, checkpoints.get(agent_id, {}))

        logger.info("Started %d agent workers (CombatPlanner + seed rotation)", len(self.processes))

    def _start_worker(self, agent_id: int, checkpoint: Dict) -> None:
        cfg = dict(self.config)
        cfg["start_episode"] = checkpoint.get("episode", 0)
        cfg["start_wins"] = checkpoint.get("wins", 0)
        cfg["conquered_initial"] = checkpoint.get("conquered_initial", False)

        p = mp.Process(
            target=_agent_worker,
            args=(agent_id, self.event_queue, self.stop_event, self.pause_event, cfg),
            daemon=True,
        )
        p.start()
        self.processes.append(p)

    async def stop(self) -> None:
        if not self.running:
            return
        self.running = False
        if self.stop_event:
            self.stop_event.set()
        # Unblock any paused workers so they can see stop_event
        if self.pause_event:
            self.pause_event.clear()

        # Save checkpoints
        self._save_checkpoints()

        for p in self.processes:
            p.join(timeout=5)
            if p.is_alive():
                p.kill()
        self.processes.clear()
        logger.info("All workers stopped, checkpoints saved")

    def pause(self) -> None:
        """Pause all worker processes."""
        if self.pause_event and not self.paused:
            self.pause_event.set()
            self.paused = True
            logger.info("Training paused")

    def resume(self) -> None:
        """Resume paused worker processes."""
        if self.pause_event and self.paused:
            self.pause_event.clear()
            self.paused = False
            logger.info("Training resumed")

    def set_workers(self, new_count: int) -> None:
        """Gracefully adjust the number of worker processes."""
        new_count = max(1, min(new_count, 32))
        current = len(self.processes)
        if new_count == current:
            return

        self.num_agents = new_count

        if new_count > current:
            # Add workers
            checkpoints = self._load_checkpoints()
            for agent_id in range(current, new_count):
                if agent_id not in self.agents:
                    self.agents[agent_id] = {
                        "id": agent_id,
                        "name": AGENT_NAMES[agent_id] if agent_id < len(AGENT_NAMES) else f"Agent_{agent_id}",
                        "phase": "STARTING", "floor": 0, "act": 1,
                        "hp": 72, "max_hp": 72, "seed": self.config.get("initial_seed", ""),
                        "episode": 0, "wins": 0, "status": "starting",
                    }
                self._start_worker(agent_id, checkpoints.get(agent_id, {}))
            logger.info("Added %d workers (total: %d)", new_count - current, new_count)
        else:
            # Remove excess workers by killing them
            to_remove = self.processes[new_count:]
            self.processes = self.processes[:new_count]
            for p in to_remove:
                if p.is_alive():
                    p.kill()
            logger.info("Removed %d workers (total: %d)", current - new_count, new_count)

    def set_config(self, params: Dict[str, Any]) -> None:
        """Apply runtime config changes."""
        if "sims" in params:
            self.config["mcts_sims"] = params["sims"]
        if "workers" in params:
            self.set_workers(params["workers"])

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
            self.recent_floors.append(event.get("floor", event.get("floors_reached", 0)))
            # Support both old mcts_avg_ms and new plan_avg_ms
            plan_ms = event.get("plan_avg_ms", event.get("mcts_avg_ms", 0))
            if plan_ms > 0:
                self.mcts_times.append(plan_ms)
            self.episode_log.append(event)

            self._broadcast({"type": "agent_episode", **{k: v for k, v in event.items() if k != "type"}})

        elif etype in ("mcts_result", "planner_result"):
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

        # Record into metrics history when we have new episodes
        if self.total_episodes != self._last_episode_count_for_metrics:
            self._last_episode_count_for_metrics = self.total_episodes
            throughput_gph = epm * 60
            self.metrics_history.append({
                "timestamp": round(time.time(), 1),
                "floor": round(af, 1),
                "win_rate": round(wr, 4),
                "loss": 0.0,  # placeholder until neural net training is wired
                "throughput": round(throughput_gph, 1),
            })

        return {
            "type": "training_stats",
            "total_episodes": self.total_episodes,
            "win_count": self.total_wins,
            "win_rate": round(wr, 3),
            "avg_floor": round(af, 1),
            "mcts_avg_ms": round(ma, 1),
            "eps_per_min": round(epm, 2),
            "uptime": round(uptime, 0),
            "paused": self.paused,
            "worker_count": len(self.processes),
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
