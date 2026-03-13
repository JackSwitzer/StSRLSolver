"""Training runner: agents playing continuously with MCTS, streaming to WS clients.

Spawns worker processes running GameRunner + StSAgent in infinite loops.
TrainingCoordinator collects events and pushes to WebSocket subscribers.
Logs all episode data to disk for analysis.

Run management:
  Each training session gets a unique run_id (YYYYMMDD_HHMMSS).
  Logs go to logs/runs/{run_id}/agent_{id}.jsonl.
  Run metadata in logs/runs/{run_id}/meta.json.
  All runs indexed in logs/runs/manifest.jsonl.
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
RUNS_DIR = LOG_DIR / "runs"


def _generate_run_id() -> str:
    """Generate a unique run ID from current timestamp."""
    return time.strftime("%Y%m%d_%H%M%S")


def _write_atomic(path: Path, data: dict) -> None:
    """Write JSON atomically via tmp file + rename."""
    tmp = path.with_suffix(".tmp")
    with open(tmp, "w") as f:
        json.dump(data, f, indent=2)
        f.flush()
        os.fsync(f.fileno())
    tmp.rename(path)


def _append_manifest(run_meta: dict) -> None:
    """Append a run summary to the manifest index."""
    manifest = RUNS_DIR / "manifest.jsonl"
    RUNS_DIR.mkdir(parents=True, exist_ok=True)
    with open(manifest, "a") as f:
        f.write(json.dumps(run_meta) + "\n")
        f.flush()
        os.fsync(f.fileno())

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


def _get_seed(agent_id: int, episode: int, initial_seed: str, plays_per_seed: int = 3, same_seed: bool = True) -> str:
    """Seed rotation: play each seed N times, then advance.

    If same_seed=True, all agents share the same seed sequence (good for comparing decisions).
    If same_seed=False, agents get offset starting seeds for seed diversity.
    """
    seed_index = episode // plays_per_seed
    if not same_seed:
        seed_index += agent_id * 1000
    if seed_index == 0:
        return initial_seed
    return f"Seed_{seed_index}"


def _make_combat_policy_fn(model, encoder, action_space, runner, action_dim):
    """Create a policy function bridging CombatEngine → neural network.

    Same pattern as self_play._make_policy_fn but adapted for the training
    runner context.  Snapshots run-level observation once per combat.

    Returns:
        Callable (CombatEngine) -> (action_priors dict, value float)
    """
    import numpy as np
    import torch
    import torch.nn.functional as F
    from packages.engine.rl_observations import STANCE_IDS, _stance_to_index
    from packages.engine.state.combat import PlayCard, UsePotion, EndTurn

    try:
        run_obs = runner.get_observation()
    except Exception:
        run_obs = {"run": {}, "combat": None}

    _run_arr_cache = [None]

    def _get_run_arr():
        if _run_arr_cache[0] is None:
            _run_arr_cache[0] = encoder.observation_to_array(run_obs)
        return _run_arr_cache[0]

    def policy_fn(engine):
        base_arr = _get_run_arr().copy()

        # Overlay combat features
        state = engine.state
        player = state.player
        off_cs = encoder._off_combat_scalars
        base_arr[off_cs] = float(state.energy)
        base_arr[off_cs + 1] = float(player.block)
        base_arr[off_cs + 2] = float(state.turn)
        base_arr[off_cs + 3] = float(len(state.hand))
        base_arr[off_cs + 4] = float(len(state.draw_pile))
        base_arr[off_cs + 5] = float(len(state.discard_pile))
        base_arr[off_cs + 6] = float(len(getattr(state, "exhaust_pile", [])))

        # Stance one-hot
        off_st = encoder._off_combat_stance
        for i in range(len(STANCE_IDS)):
            base_arr[off_st + i] = 0.0
        stance_idx = _stance_to_index(getattr(state, "stance", "Neutral"))
        base_arr[off_st + stance_idx] = 1.0

        # Enemies
        off_e = encoder._off_combat_enemies
        n_per = encoder.n_per_enemy
        for i in range(encoder.max_enemies):
            ebase = off_e + i * n_per
            base_arr[ebase:ebase + n_per] = 0.0
        for ei, enemy in enumerate(state.enemies):
            if ei >= encoder.max_enemies:
                break
            ebase = off_e + ei * n_per
            emax = max(getattr(enemy, "max_hp", 1), 1)
            base_arr[ebase] = enemy.hp / emax
            base_arr[ebase + 1] = float(emax)
            base_arr[ebase + 2] = float(enemy.block)
            base_arr[ebase + 3] = float(getattr(enemy, "move_damage", 0) or 0)
            base_arr[ebase + 4] = float(getattr(enemy, "move_hits", 0) or 0)
            base_arr[ebase + 5] = 1.0 if enemy.hp > 0 else 0.0

        # Get legal actions and build action mask
        legal_actions = engine.get_legal_actions()
        if not legal_actions:
            return {}, 0.0

        action_ids = {}
        for act in legal_actions:
            if isinstance(act, PlayCard):
                parts = ["play_card", f"card_index={act.card_idx}"]
                if act.target_idx >= 0:
                    parts.append(f"target_index={act.target_idx}")
                aid = "|".join(parts)
            elif isinstance(act, UsePotion):
                parts = ["use_potion", f"potion_slot={act.potion_idx}"]
                if act.target_idx >= 0:
                    parts.append(f"target_index={act.target_idx}")
                aid = "|".join(parts)
            elif isinstance(act, EndTurn):
                aid = "end_turn"
            else:
                aid = str(act)
            action_ids[id(act)] = aid

        mask = np.zeros(action_dim, dtype=np.bool_)
        action_to_mask_idx = {}
        for act in legal_actions:
            aid = action_ids[id(act)]
            idx = action_space.register(aid)
            if idx < action_dim:
                mask[idx] = True
                action_to_mask_idx[id(act)] = idx

        # Forward pass
        obs_tensor = torch.tensor(base_arr, dtype=torch.float32).unsqueeze(0)
        mask_tensor = torch.tensor(mask, dtype=torch.bool).unsqueeze(0)

        with torch.no_grad():
            logits, value, _ = model(obs_tensor, mask_tensor)

        probs = F.softmax(logits[0], dim=-1).numpy()
        val = value.item()

        # Map back to Action objects
        action_priors = {}
        for act in legal_actions:
            midx = action_to_mask_idx.get(id(act))
            if midx is not None and midx < len(probs):
                action_priors[act] = float(probs[midx])
            else:
                action_priors[act] = 1e-6

        return action_priors, val

    return policy_fn


def _agent_worker(
    agent_id: int,
    event_queue: mp.Queue,
    stop_event: mp.Event,
    pause_event: mp.Event,
    config: Dict[str, Any],
) -> None:
    """Run games in an infinite loop, pushing events to queue.

    Uses GumbelMCTS (neural-guided) for combat decisions with CombatPlanner
    as fallback, and StrategicPlanner for non-combat decisions.
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

    # --- Load combat model + GumbelMCTS for neural-guided search ---
    combat_model = None
    gumbel_mcts = None
    encoder = None
    action_space = None
    combat_sims = config.get("combat_sims", 32)
    deep_sims = config.get("deep_sims", 64)
    deep_prob = config.get("deep_prob", 0.25)
    use_mcts = config.get("use_mcts", True)

    if use_mcts:
        try:
            import numpy as np
            import torch
            import torch.nn.functional as F
            from packages.training.torch_policy_net import StSPolicyValueNet
            from packages.training.gumbel_mcts import GumbelMCTS
            from packages.engine.rl_observations import ObservationEncoder
            from packages.engine.rl_masks import ActionSpace

            obs_dim = config.get("obs_dim", 1186)
            action_dim = config.get("action_dim", 2048)
            hidden_dim = config.get("hidden_dim", 256)
            num_layers = config.get("num_layers", 3)

            combat_model = StSPolicyValueNet(
                obs_dim=obs_dim, action_dim=action_dim,
                hidden_dim=hidden_dim, num_layers=num_layers,
            )

            # Try loading checkpoint
            from pathlib import Path as _Path
            model_path = config.get("combat_model_path")
            if model_path and _Path(model_path).exists():
                checkpoint = torch.load(model_path, map_location="cpu", weights_only=True)
                combat_model.load_state_dict(checkpoint["model_state_dict"])
            combat_model.eval()

            encoder = ObservationEncoder()
            action_space = ActionSpace()
            gumbel_mcts = GumbelMCTS(num_simulations=combat_sims)
            logger.info("Agent %d: combat model loaded (MCTS enabled)", agent_id)
        except Exception as exc:
            logger.warning("Agent %d: MCTS init failed (%s), using CombatPlanner only", agent_id, exc)
            combat_model = None
            gumbel_mcts = None

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

        same_seed = config.get("same_seed", True)
        seed = _get_seed(agent_id, episode, initial_seed, plays_per_seed, same_seed=same_seed)

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

            # --- COMBAT: GumbelMCTS (primary) with CombatPlanner fallback ---
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

                    # Refresh MCTS policy function at combat start
                    if gumbel_mcts is not None and combat_model is not None:
                        try:
                            _policy_fn = _make_combat_policy_fn(
                                combat_model, encoder, action_space, runner,
                                combat_model.action_dim,
                            )
                            gumbel_mcts.policy_fn = _policy_fn
                        except Exception:
                            pass

                if engine and len(actions) > 1:
                    mcts_used = False
                    t0 = time.monotonic()

                    # --- Try GumbelMCTS first ---
                    if gumbel_mcts is not None and combat_model is not None:
                        try:
                            import numpy as np
                            from packages.engine.state.combat import PlayCard, UsePotion, EndTurn

                            # KataGo playout cap: deep vs shallow
                            use_deep = np.random.random() < deep_prob
                            sims = deep_sims if use_deep else combat_sims
                            gumbel_mcts.num_simulations = sims

                            action_probs = gumbel_mcts.search(engine)

                            if action_probs:
                                best_action = max(action_probs, key=action_probs.get)
                                elapsed_ms = (time.monotonic() - t0) * 1000
                                plan_calls += 1
                                plan_total_ms += elapsed_ms
                                combat_turns += 1
                                mcts_used = True

                                # Record turn snapshot for meta-learner
                                state = engine.state
                                player_hp = state.player.hp
                                max_hp_val = max(state.player.max_hp, 1)
                                live_enemies = [e for e in state.enemies if e.hp > 0]
                                combat_turn_snapshots.append({
                                    "hp_pct": round(player_hp / max_hp_val, 2),
                                    "enemy_count": len(live_enemies),
                                    "energy": state.energy,
                                    "stance": getattr(state, "stance", "Neutral"),
                                    "hand_quality": _compute_hand_quality(state.hand),
                                    "strategy": 3,  # MCTS
                                    "player_hp": player_hp,
                                })

                                # Execute best MCTS action
                                if isinstance(best_action, PlayCard):
                                    ga = CombatAction(action_type="play_card", card_idx=best_action.card_idx, target_idx=best_action.target_idx)
                                elif isinstance(best_action, UsePotion):
                                    ga = CombatAction(action_type="use_potion", potion_idx=best_action.potion_idx, target_idx=best_action.target_idx)
                                else:
                                    ga = CombatAction(action_type="end_turn")
                                runner.take_action(ga)
                                step += 1

                                # Build planner data for viewer
                                planner_data = {
                                    "type": "planner_result",
                                    "agent_id": agent_id,
                                    "elapsed_ms": round(elapsed_ms, 1),
                                    "lines_considered": sims,
                                    "strategy": "mcts",
                                    "turns_to_kill": 0,
                                    "expected_hp_loss": 0,
                                    "confidence": 0.0,
                                    "cards": [str(best_action)],
                                }
                                _put_safe(event_queue, planner_data)

                                # Heartbeat
                                now = time.monotonic()
                                if now - last_heartbeat > 0.5:
                                    last_heartbeat = now
                                    _send_heartbeat(event_queue, agent_id, runner, seed, episode, total_wins, step)
                                continue
                        except Exception:
                            pass  # Fall through to CombatPlanner

                    # --- Fallback: CombatPlanner (turn-level line search) ---
                    if not mcts_used:
                        t0_plan = time.monotonic()
                        try:
                            plan = combat_planner.plan_turn(engine)
                        except Exception:
                            plan = None
                        elapsed_ms = (time.monotonic() - t0_plan) * 1000

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
                                max_hp_val = max(state.player.max_hp, 1)
                                live_enemies = [e for e in state.enemies if e.hp > 0]
                                combat_turn_snapshots.append({
                                    "hp_pct": round(player_hp / max_hp_val, 2),
                                    "enemy_count": len(live_enemies),
                                    "energy": state.energy,
                                    "stance": getattr(state, "stance", "Neutral"),
                                    "hand_quality": _compute_hand_quality(state.hand),
                                    "strategy": 2,  # CombatPlanner
                                    "player_hp": player_hp,
                                })

                            # Execute the planned card sequence
                            for card_id, target_idx in plan.card_sequence:
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

                            # End turn after playing all cards
                            if not runner.game_over and runner.phase == GamePhase.COMBAT:
                                try:
                                    runner.take_action(CombatAction(action_type="end_turn"))
                                    step += 1
                                except Exception:
                                    pass

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

                            if planner_data:
                                _put_safe(event_queue, planner_data)

                            # Heartbeat
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
                    # Count stance usage from turn snapshots
                    stance_counts: Dict[str, int] = {}
                    for snap in combat_turn_snapshots:
                        s = snap.get("stance", "Neutral")
                        stance_counts[s] = stance_counts.get(s, 0) + 1
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
                        "stances": stance_counts,
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
                        # Pass map nodes so planner can see room types
                        map_nodes = rs.get_available_paths() if hasattr(rs, "get_available_paths") else []
                        idx = planner.plan_path_choice(runner, map_nodes if map_nodes else actions)
                        action = actions[min(idx, len(actions) - 1)]
                        room_type = ""
                        if idx < len(map_nodes):
                            rt = getattr(map_nodes[idx], "room_type", None)
                            room_type = rt.value if hasattr(rt, "value") else str(rt) if rt else ""
                        decisions.append({
                            "floor": current_floor, "type": "path",
                            "choice": room_type or str(action),
                        })
                    elif phase == GamePhase.REST:
                        idx = planner.plan_rest_site(runner, actions)
                        action = actions[min(idx, len(actions) - 1)]
                        hp_pct = round(getattr(rs, "current_hp", 0) / max(getattr(rs, "max_hp", 72), 1), 2)
                        decisions.append({
                            "floor": current_floor, "type": "rest",
                            "choice": str(action), "hp_pct": hp_pct,
                        })
                    elif phase == GamePhase.SHOP:
                        idx = planner.plan_shop_action(runner, actions)
                        action = actions[min(idx, len(actions) - 1)]
                        decisions.append({
                            "floor": current_floor, "type": "shop",
                            "choice": str(action),
                        })
                    elif phase == GamePhase.EVENT:
                        idx = planner.plan_event_choice(runner, actions)
                        action = actions[min(idx, len(actions) - 1)]
                        # Try to get event name
                        event_id = ""
                        if hasattr(rs, "current_event"):
                            event_id = getattr(rs.current_event, "id", getattr(rs.current_event, "name", ""))
                        elif hasattr(rs, "event_id"):
                            event_id = str(rs.event_id)
                        decisions.append({
                            "floor": current_floor, "type": "event",
                            "choice": str(action), "event_id": event_id,
                        })
                    elif phase in (GamePhase.COMBAT_REWARDS, GamePhase.BOSS_REWARDS):
                        idx = planner.plan_card_pick(runner, actions)
                        action = actions[min(idx, len(actions) - 1)]
                        # Log alternatives
                        alternatives = [str(a) for a in actions if a != action][:3]
                        decisions.append({
                            "floor": current_floor, "type": "card_pick",
                            "choice": str(action), "alternatives": alternatives,
                        })
                    elif phase == GamePhase.NEOW:
                        # Neow: just pick first for now, but log it
                        action = actions[0]
                        decisions.append({
                            "floor": 0, "type": "neow",
                            "choice": str(action),
                        })
                    else:
                        # Truly unhandled phases (TREASURE, etc.)
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

        # Determine death cause (last combat enemy if died)
        death_floor = 0
        death_enemy = ""
        if not won and combats:
            last_combat = combats[-1]
            death_floor = last_combat.get("floor", final_floor)
            death_enemy = last_combat.get("enemy", "Unknown")

        # Compressed episode summary
        summary = {
            "type": "episode",
            "run_id": config.get("run_id", ""),
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
            "death_floor": death_floor,
            "death_enemy": death_enemy,
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


def _serialize_map(runner: Any) -> Optional[Dict[str, Any]]:
    """Serialize current act's map for frontend visualization."""
    rs = runner.run_state
    current_map = rs.get_current_map()
    if not current_map:
        return None

    nodes = []
    for row in current_map:
        for node in row:
            if node and node.room_type:
                edges = [{"dx": e.dst_x, "dy": e.dst_y} for e in node.edges]
                nd: Dict[str, Any] = {
                    "x": node.x,
                    "y": node.y,
                    "type": node.room_type.value,
                    "edges": edges,
                }
                if node.has_emerald_key:
                    nd["key"] = True
                nodes.append(nd)

    pos = {"x": rs.map_position.x, "y": rs.map_position.y}
    visited = [{"act": v[0], "x": v[1], "y": v[2]} for v in rs.visited_nodes]

    result: Dict[str, Any] = {
        "act": rs.act,
        "nodes": nodes,
        "position": pos,
        "visited": visited,
    }

    # Available paths with scores
    try:
        paths = rs.get_available_paths()
        hp_pct = rs.current_hp / max(rs.max_hp, 1)
        available = []
        for p in paths:
            rt = p.room_type
            type_str = rt.value if rt else "?"
            # Inline scoring (matches StrategicPlanner._score_room)
            score = _quick_score_room(type_str, hp_pct, rs)
            available.append({
                "x": p.x,
                "y": p.y,
                "type": type_str,
                "score": round(score, 1),
            })
        result["available"] = available
    except Exception:
        pass

    return result


def _quick_score_room(room_type: str, hp_pct: float, rs: Any) -> float:
    """Quick room scoring matching StrategicPlanner logic."""
    rt = room_type.lower().strip()
    if rt in ("r",):
        return 4.0 if hp_pct < 0.4 else 2.5 if hp_pct < 0.6 else 0.5
    if rt in ("m",):
        return 2.0 if hp_pct > 0.5 else 0.5
    if rt in ("e",):
        return 2.5 if hp_pct > 0.75 else -2.0
    if rt in ("?",):
        return 1.5
    if rt in ("$",):
        gold = getattr(rs, "gold", 0)
        return 2.5 if gold > 100 else 1.0
    if rt in ("t",):
        return 2.5
    return 0.0


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
    # Compact counts for grid display (always sent)
    deck_size = len(getattr(rs, "deck", []))
    relic_count = len(getattr(rs, "relics", []))
    potions_raw = getattr(rs, "potion_slots", [])
    potion_count = sum(1 for p in potions_raw if getattr(p, "potion_id", None))
    potion_max = len(potions_raw)
    # Stance from combat state if available
    cs = getattr(rs, "combat_state", None)
    current_stance = getattr(cs, "stance", "Neutral") if cs else "Neutral"

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
        "gold": getattr(rs, "gold", 0),
        "deck_size": deck_size,
        "relic_count": relic_count,
        "potion_count": potion_count,
        "potion_max": potion_max,
        "stance": current_stance,
    }
    # Always include deck/relics/potions so Run tab has data
    try:
        from packages.engine.content.cards import ALL_CARDS
        deck = getattr(rs, "deck", [])
        deck_list = []
        for c in deck:
            card_def = ALL_CARDS.get(c.id)
            ct = "skill"
            if card_def and hasattr(card_def, "card_type"):
                ct = card_def.card_type.value.lower()
            name = card_def.name if card_def else c.id
            cost = card_def.cost if card_def else 0
            deck_list.append({
                "id": c.id,
                "name": f"{name}+" if c.upgraded else name,
                "cost": cost,
                "type": ct,
                "upgraded": c.upgraded,
            })
        hb["deck"] = deck_list
        relics = getattr(rs, "relics", [])
        hb["relics"] = [{"id": r.id, "name": r.id, "counter": r.counter if r.counter >= 0 else None} for r in relics]
        potions = getattr(rs, "potion_slots", [])
        hb["potions"] = [{"id": getattr(p, "potion_id", None), "name": getattr(p, "potion_id", None)} for p in potions]
    except Exception:
        pass
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
                hb["energy"] = adapted.get("energy", 3)
                hb["max_energy"] = adapted.get("max_energy", 3)
        except Exception:
            pass
    # Map snapshot for non-combat phases
    if phase_str not in ("COMBAT", "UNKNOWN"):
        try:
            map_data = _serialize_map(runner)
            if map_data:
                hb["map"] = map_data
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
        ascension: int = 0,
        initial_seed: str = "Test123",
        headless_after_min: Optional[int] = None,
        visual_at: Optional[str] = None,
    ):
        self.num_agents = num_agents
        self.headless_after_min = headless_after_min
        self.visual_at = visual_at
        self._headless = False
        self._last_status_write = 0.0

        # Run management: each start() creates a new run
        self.run_id: Optional[str] = None
        self.run_dir: Optional[Path] = None

        self.config: Dict[str, Any] = {
            "mcts_sims": mcts_sims,
            "ascension": ascension,
            "character": "Watcher",
            "initial_seed": initial_seed,
            "log_dir": str(LOG_DIR),
            "same_seed": False,
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
        # Latest map state per agent (for immediate send on focus)
        self.latest_map: Dict[int, Dict] = {}
        # Latest run state per agent (deck/relics/potions/gold) for immediate send on focus
        self.latest_run_state: Dict[int, Dict] = {}

        # WS subscribers
        self._subscribers: Set[int] = set()
        self._focused: Dict[int, Set[int]] = {}  # conn_id -> set of focused agent_ids
        self._broadcast_queues: Dict[int, asyncio.Queue] = {}

    async def start(self) -> None:
        if self.running:
            return

        # Create new run
        self.run_id = _generate_run_id()
        self.run_dir = RUNS_DIR / self.run_id
        self.run_dir.mkdir(parents=True, exist_ok=True)

        # Point worker logs to this run's directory
        self.config["log_dir"] = str(self.run_dir)
        self.config["run_id"] = self.run_id

        # Write initial run metadata
        self._run_meta = {
            "run_id": self.run_id,
            "started_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "started_epoch": time.time(),
            "config": {k: v for k, v in self.config.items() if k != "log_dir"},
            "num_agents": self.num_agents,
            "status": "running",
        }
        _write_atomic(self.run_dir / "meta.json", self._run_meta)

        # Also keep legacy LOG_DIR for backwards compat
        LOG_DIR.mkdir(parents=True, exist_ok=True)

        self.event_queue = mp.Queue(maxsize=5000)
        self.stop_event = mp.Event()
        self.pause_event = mp.Event()
        self.start_time = time.time()
        self.running = True

        # Reset aggregate stats for new run
        self.total_episodes = 0
        self.total_wins = 0
        self.recent_results.clear()
        self.recent_floors.clear()
        self.mcts_times.clear()
        self.episode_log.clear()
        self.metrics_history.clear()
        self._last_episode_count_for_metrics = 0
        self.latest_mcts.clear()
        self.latest_combat.clear()
        self.latest_map.clear()
        self.latest_run_state.clear()

        # Reset agent states
        for i in range(self.num_agents):
            self.agents[i] = {
                "id": i,
                "name": AGENT_NAMES[i] if i < len(AGENT_NAMES) else f"Agent_{i}",
                "phase": "STARTING", "floor": 0, "act": 1,
                "hp": 72, "max_hp": 72, "seed": self.config.get("initial_seed", ""),
                "episode": 0, "wins": 0, "status": "starting",
            }

        # Load checkpoints from this run dir (empty on fresh start)
        checkpoints = self._load_checkpoints()

        for agent_id in range(self.num_agents):
            self._start_worker(agent_id, checkpoints.get(agent_id, {}))

        logger.info("Run %s started: %d agents → %s", self.run_id, len(self.processes), self.run_dir)

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

        # Finalize run metadata
        self._finalize_run()
        logger.info("Run %s stopped, checkpoints saved", self.run_id)

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

    def check_headless_schedule(self) -> None:
        """Check if headless mode should be toggled based on time schedule."""
        now = time.time()

        # Check headless_after_min: go headless after N minutes of running
        if not self._headless and self.headless_after_min is not None:
            elapsed_min = (now - self.start_time) / 60.0
            if elapsed_min >= self.headless_after_min:
                self._headless = True
                logger.info("Entering headless mode after %.1f min", elapsed_min)

        # Check visual_at: resume broadcasting at HH:MM
        if self._headless and self.visual_at is not None:
            current_hhmm = time.strftime("%H:%M")
            if current_hhmm == self.visual_at:
                self._headless = False
                logger.info("Exiting headless mode at %s", self.visual_at)

        # In headless mode, write status.json every 60s
        if self._headless and now - self._last_status_write >= 60.0:
            self._last_status_write = now
            self._write_status_json()

    def _write_status_json(self) -> None:
        """Write current training stats to status.json for headless monitoring."""
        status_dir = self.run_dir if self.run_dir else RUNS_DIR
        status_dir.mkdir(parents=True, exist_ok=True)
        elapsed = time.time() - self.start_time if self.start_time else 0
        status = {
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "elapsed_s": round(elapsed, 1),
            "total_episodes": self.total_episodes,
            "total_wins": self.total_wins,
            "win_rate": round(self.total_wins / max(self.total_episodes, 1) * 100, 2),
            "recent_win_rate": round(sum(self.recent_results) / max(len(self.recent_results), 1) * 100, 2),
            "avg_floor": round(sum(self.recent_floors) / max(len(self.recent_floors), 1), 1),
            "games_per_hr": round(self.total_episodes / max(elapsed / 3600, 0.001), 1),
            "headless": self._headless,
            "paused": self.paused,
            "num_agents": self.num_agents,
        }
        _write_atomic(status_dir / "status.json", status)

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
        """Apply runtime config changes.

        Handles both in-process config (sims, workers) and hot-reload params
        (entropy_coeff, lr, temperature, turn_solver_ms) that get written to
        reload.json and signalled to the overnight training process via SIGUSR1.
        """
        import signal
        import subprocess as _sp

        if "sims" in params:
            self.config["mcts_sims"] = params["sims"]
        if "workers" in params:
            self.set_workers(params["workers"])

        # Hot-reload params forwarded to the overnight training process
        _HOT_RELOAD_KEYS = ("entropy_coeff", "lr", "temperature", "turn_solver_ms")
        reload_params: Dict[str, Any] = {}
        for key in _HOT_RELOAD_KEYS:
            if key in params:
                reload_params[key] = params[key]

        if reload_params:
            run_dir = self.run_dir if self.run_dir else Path("logs/weekend-run")
            reload_path = run_dir / "reload.json"
            try:
                reload_path.parent.mkdir(parents=True, exist_ok=True)
                reload_path.write_text(json.dumps(reload_params))
                logger.info("Wrote hot-reload params to %s: %s", reload_path, reload_params)
            except Exception as exc:
                logger.warning("Failed to write reload.json: %s", exc)

            # Signal overnight process to pick up the new config
            try:
                result = _sp.run(
                    ["pgrep", "-f", "overnight"],
                    capture_output=True, text=True, timeout=5,
                )
                for pid_str in result.stdout.strip().split("\n"):
                    pid_str = pid_str.strip()
                    if pid_str:
                        os.kill(int(pid_str), signal.SIGUSR1)
                        logger.info("Sent SIGUSR1 to overnight pid %s", pid_str)
            except Exception as exc:
                logger.debug("Could not signal overnight process: %s", exc)

    def _checkpoint_path(self) -> Path:
        """Checkpoint goes into the current run directory."""
        if self.run_dir:
            return self.run_dir / "checkpoints.json"
        return LOG_DIR / "checkpoints.json"

    def _load_checkpoints(self) -> Dict[int, Dict]:
        cp_path = self._checkpoint_path()
        if cp_path.exists():
            try:
                with open(cp_path) as f:
                    return {int(k): v for k, v in json.load(f).items()}
            except Exception:
                pass
        return {}

    def _save_checkpoints(self) -> None:
        cp_path = self._checkpoint_path()
        data = {}
        for aid, a in self.agents.items():
            data[str(aid)] = {
                "episode": a.get("episode", 0),
                "wins": a.get("wins", 0),
                "conquered_initial": a.get("seed", "") != self.config["initial_seed"],
            }
        try:
            _write_atomic(cp_path, data)
        except Exception:
            pass

    def _finalize_run(self) -> None:
        """Write final run stats to meta.json and append to manifest."""
        if not self.run_dir or not hasattr(self, "_run_meta"):
            return

        uptime = time.time() - self.start_time if self.start_time else 0
        wr = sum(self.recent_results) / max(len(self.recent_results), 1)
        af = sum(self.recent_floors) / max(len(self.recent_floors), 1)

        self._run_meta.update({
            "status": "completed",
            "stopped_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "stopped_epoch": time.time(),
            "duration_s": round(uptime, 1),
            "total_episodes": self.total_episodes,
            "total_wins": self.total_wins,
            "win_rate": round(wr, 4),
            "avg_floor": round(af, 2),
            "eps_per_min": round(self.total_episodes / max(uptime / 60, 0.01), 2),
            "per_agent": {
                str(aid): {
                    "episodes": a.get("episode", 0),
                    "wins": a.get("wins", 0),
                }
                for aid, a in self.agents.items()
            },
        })

        try:
            _write_atomic(self.run_dir / "meta.json", self._run_meta)
        except Exception:
            logger.warning("Failed to write run metadata")

        # Append to manifest (compact summary)
        manifest_entry = {
            "run_id": self.run_id,
            "started_at": self._run_meta.get("started_at"),
            "stopped_at": self._run_meta.get("stopped_at"),
            "duration_s": self._run_meta.get("duration_s"),
            "num_agents": self.num_agents,
            "total_episodes": self.total_episodes,
            "win_rate": round(wr, 4),
            "avg_floor": round(af, 2),
            "ascension": self.config.get("ascension", 20),
        }
        try:
            _append_manifest(manifest_entry)
        except Exception:
            logger.warning("Failed to append to manifest")

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

        # Immediately send cached map state
        cached_map = self.latest_map.get(agent_id)
        if cached_map:
            self._send_to(conn_id, {"type": "agent_map", "agent_id": agent_id, "map": cached_map})

        # Immediately send cached run state (deck/relics/potions/gold)
        cached_run_state = self.latest_run_state.get(agent_id)
        if cached_run_state:
            self._send_to(conn_id, cached_run_state)

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
            # Check headless schedule each iteration
            self.check_headless_schedule()

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

            # Skip WS broadcasting in headless mode
            if not self._headless:
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
                          "enemy_name", "enemy_hp", "enemy_max_hp", "hand_size", "turn", "stance",
                          "energy", "max_energy", "gold", "deck", "relics", "potions",
                          "deck_size", "relic_count", "potion_count", "potion_max"):
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

                # Cache and forward map snapshot to focused clients
                map_data = event.get("map")
                if map_data:
                    self.latest_map[agent_id] = map_data
                    for conn_id, focused_set in self._focused.items():
                        if agent_id in focused_set:
                            self._send_to(conn_id, {"type": "agent_map", "agent_id": agent_id, "map": map_data})

                # Forward run state (deck/relics/potions/gold) to focused clients
                deck = event.get("deck")
                if deck is not None:
                    run_state_msg = {
                        "type": "agent_run_state",
                        "agent_id": agent_id,
                        "deck": deck,
                        "relics": event.get("relics", []),
                        "potions": event.get("potions", []),
                        "gold": event.get("gold", 0),
                    }
                    # Cache for immediate send on focus
                    self.latest_run_state[agent_id] = run_state_msg
                    for conn_id, focused_set in self._focused.items():
                        if agent_id in focused_set:
                            self._send_to(conn_id, run_state_msg)

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

            # Include rich episode data (combats, decisions, hp_history, death info)
            episode_msg = {"type": "agent_episode"}
            for k, v in event.items():
                if k != "type":
                    episode_msg[k] = v
            self._broadcast(episode_msg)

        elif etype in ("mcts_result", "planner_result"):
            self.latest_mcts[agent_id] = event
            for conn_id, focused_set in self._focused.items():
                if agent_id in focused_set:
                    self._send_to(conn_id, event)

    # Keys too large for grid broadcast (sent via focused agent path instead)
    _GRID_EXCLUDE_KEYS = {"deck", "relics", "potions"}

    def _build_grid_update(self) -> Dict:
        agents_data = []
        for a in self.agents.values():
            agent_dict = {k: v for k, v in a.items() if k not in self._GRID_EXCLUDE_KEYS}
            # Add compact combat summary if agent is in combat
            if a.get("phase") == "COMBAT" and a.get("enemy_name"):
                agent_dict["combat_summary"] = {
                    "enemy_name": a.get("enemy_name", ""),
                    "enemy_hp": a.get("enemy_hp", 0),
                    "enemy_max_hp": a.get("enemy_max_hp", 0),
                    "stance": a.get("stance", "Neutral"),
                    "hand_size": a.get("hand_size", 0),
                    "energy": a.get("energy", 3),
                    "max_energy": a.get("max_energy", 3),
                    "turn": a.get("turn", 1),
                }
            agents_data.append(agent_dict)
        return {
            "type": "grid_update",
            "run_id": self.run_id,
            "agents": agents_data,
        }

    @staticmethod
    def list_runs(limit: int = 50) -> List[Dict]:
        """List past training runs from manifest, most recent first."""
        manifest = RUNS_DIR / "manifest.jsonl"
        if not manifest.exists():
            return []
        runs = []
        try:
            with open(manifest) as f:
                for line in f:
                    line = line.strip()
                    if line:
                        runs.append(json.loads(line))
        except Exception:
            pass
        return list(reversed(runs[-limit:]))

    @staticmethod
    def get_run_meta(run_id: str) -> Optional[Dict]:
        """Load full metadata for a specific run."""
        meta_path = RUNS_DIR / run_id / "meta.json"
        if meta_path.exists():
            try:
                with open(meta_path) as f:
                    return json.load(f)
            except Exception:
                pass
        return None

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

        mf = max(self.recent_floors) if self.recent_floors else 0

        return {
            "type": "training_stats",
            "run_id": self.run_id,
            "total_episodes": self.total_episodes,
            "win_count": self.total_wins,
            "win_rate": round(wr, 3),
            "avg_floor": round(af, 1),
            "max_floor": mf,
            "mcts_avg_ms": round(ma, 1),
            "eps_per_min": round(epm, 2),
            "uptime": round(uptime, 0),
            "paused": self.paused,
            "worker_count": len(self.processes),
        }

    def _log_resources(self) -> None:
        try:
            usage = resource.getrusage(resource.RUSAGE_CHILDREN)
            rss_mb = usage.ru_maxrss / 1024 / 1024  # macOS returns bytes (actually pages on macOS)
            stats = {
                "ts": time.time(),
                "total_episodes": self.total_episodes,
                "worker_count": len(self.processes),
                "rss_mb": round(rss_mb, 1),
                "user_time_s": round(usage.ru_utime, 1),
                "sys_time_s": round(usage.ru_stime, 1),
            }
            res_path = self.run_dir / "resources.jsonl" if self.run_dir else LOG_DIR / "resources.jsonl"
            with open(res_path, "a") as f:
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
