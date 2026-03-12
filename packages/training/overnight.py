"""
Overnight training runner with scheduling and hyperparameter sweep.

Manages long-running training sessions with:
- Time-based headless/visual mode switching
- Hyperparameter sweep scheduling
- Status file writing for monitoring
- Integration with StrategicTrainer + combat self-play
- Multiprocessing for parallel game execution (Phase 2A)
- Centralized GPU inference server (Phase 2B)

Phase 2A changes (2026-03-12):
- ProcessPoolExecutor for parallel game execution (8 workers -> 15+ games/min)
- PBRS (Potential-Based Reward Shaping) for dense rewards
- episodes.jsonl logging per game
- --batch-size CLI arg for PPO mini-batch size
- Temperature-based exploration during training

Phase 2B changes (centralized inference server):
- Workers are torch-free; all forward passes batched in main process via InferenceServer
- Persistent mp.Pool (no executor teardown per batch)
- Weight sync after each PPO update via server.sync_strategic_from_pytorch()
"""

from __future__ import annotations

import json
import logging
import multiprocessing as mp
import signal
import time
from collections import deque
from datetime import datetime
from pathlib import Path
from typing import Any, Deque, Dict, List, Optional

import numpy as np

logger = logging.getLogger(__name__)

DEFAULT_SWEEP_CONFIGS = [
    # --- Pure on-policy (no heuristic override, NN generates all data) ---
    {"name": "pure_med", "epsilon_mode": "none",
     "lr": 3e-4, "lr_schedule": "cosine", "lr_T_max": 30000,
     "batch_size": 256, "entropy_coeff": 0.05, "temperature": 1.0},
    {"name": "pure_low_lr", "epsilon_mode": "none",
     "lr": 1e-4, "lr_schedule": "cosine", "lr_T_max": 30000,
     "batch_size": 512, "entropy_coeff": 0.03, "temperature": 0.8},
    {"name": "pure_high_lr", "epsilon_mode": "none",
     "lr": 1e-3, "lr_schedule": "cosine", "lr_T_max": 30000,
     "batch_size": 256, "entropy_coeff": 0.05, "temperature": 1.0},
    {"name": "pure_restarts", "epsilon_mode": "none",
     "lr": 3e-4, "lr_schedule": "cosine_warm_restarts", "lr_T_0": 5000,
     "batch_size": 256, "entropy_coeff": 0.04, "temperature": 1.0},

    # --- Importance-weighted epsilon-greedy (correct behavior policy) ---
    {"name": "iw_explore", "epsilon_mode": "importance_weighted",
     "epsilon_start": 0.8, "epsilon_end": 0.3, "epsilon_decay": 50000,
     "lr": 3e-4, "lr_schedule": "cosine", "lr_T_max": 30000,
     "batch_size": 256, "entropy_coeff": 0.05, "temperature": 1.0},
    {"name": "iw_low_lr", "epsilon_mode": "importance_weighted",
     "epsilon_start": 0.7, "epsilon_end": 0.2, "epsilon_decay": 40000,
     "lr": 1e-4, "lr_schedule": "cosine", "lr_T_max": 30000,
     "batch_size": 512, "entropy_coeff": 0.03, "temperature": 0.8},
    {"name": "iw_high_lr", "epsilon_mode": "importance_weighted",
     "epsilon_start": 0.5, "epsilon_end": 0.2, "epsilon_decay": 30000,
     "lr": 1e-3, "lr_schedule": "linear_decay",
     "batch_size": 256, "entropy_coeff": 0.02, "temperature": 0.7},
    {"name": "iw_restarts", "epsilon_mode": "importance_weighted",
     "epsilon_start": 0.6, "epsilon_end": 0.25, "epsilon_decay": 40000,
     "lr": 3e-4, "lr_schedule": "cosine_warm_restarts", "lr_T_0": 5000,
     "batch_size": 256, "entropy_coeff": 0.04, "temperature": 1.0},
]

# Adaptive ascension breakpoints: (min_avg_floor, min_win_rate, target_ascension)
ASCENSION_BREAKPOINTS = [
    (17, 0.05, 1),   # Clearing Act 1 somewhat reliably -> A1
    (17, 0.15, 3),   # 15% WR -> A3
    (17, 0.30, 5),   # 30% WR -> A5
    (33, 0.10, 7),   # Reaching Act 2 boss at 10% -> A7
    (33, 0.25, 10),  # 25% WR past Act 2 -> A10
]


# ---------------------------------------------------------------------------
# PBRS potential function
# ---------------------------------------------------------------------------

def compute_potential(run_state) -> float:
    """Compute the potential Phi(s) for PBRS.

    Components:
    - floor_pct: progress through the run (floor / 55)
    - hp_pct: current health percentage
    - deck_quality: heuristic for deck composition quality

    Returns a scalar potential value.
    """
    hp_pct = run_state.current_hp / max(run_state.max_hp, 1)
    floor_pct = getattr(run_state, "floor", 0) / 55.0
    deck_size = len(getattr(run_state, "deck", []))
    # Ideal deck is 12-25 cards; penalize bloat
    if 12 <= deck_size <= 25:
        deck_quality = 1.0
    elif deck_size < 12:
        deck_quality = 0.8
    else:
        deck_quality = max(0.5, 1.0 - (deck_size - 25) * 0.02)

    # Relic count bonus (relics are always positive)
    relic_count = len(getattr(run_state, "relics", []))
    relic_bonus = min(relic_count * 0.02, 0.15)

    return 0.45 * floor_pct + 0.30 * hp_pct + 0.15 * deck_quality + 0.10 * relic_bonus


# Event rewards scaled by HP efficiency
EVENT_REWARDS = {
    "combat_win": 0.05,
    "elite_win": 0.15,
    "boss_win": 0.40,
}


# ---------------------------------------------------------------------------
# Lightweight combat heuristic (no simulation, ~0ms per call)
# ---------------------------------------------------------------------------

# Card data: {card_id: (cost, base_dmg, base_block, is_attack)} for scoring
# Only needs core Watcher cards — unknown cards get defaults
_CARD_STATS = {
    "Strike_P": (1, 6, 0, True), "Strike_P+": (1, 9, 0, True),
    "Defend_P": (1, 0, 5, False), "Defend_P+": (1, 0, 8, False),
    "Eruption": (2, 9, 0, True), "Eruption+": (1, 9, 0, True),
    "Vigilance": (2, 0, 8, False), "Vigilance+": (2, 0, 12, False),
    "BowlingBash": (1, 7, 0, True), "BowlingBash+": (1, 10, 0, True),
    "CrushJoints": (1, 8, 0, True), "CrushJoints+": (1, 10, 0, True),
    "CutThroughFate": (1, 7, 0, True), "CutThroughFate+": (1, 9, 0, True),
    "EmptyBody": (1, 0, 7, False), "EmptyBody+": (1, 0, 11, False),
    "EmptyFist": (1, 9, 0, True), "EmptyFist+": (1, 14, 0, True),
    "Flurry": (0, 4, 0, True), "Flurry+": (0, 6, 0, True),
    "FlyingSleeves": (1, 4, 0, True), "FlyingSleeves+": (1, 6, 0, True),
    "FollowUp": (1, 7, 0, True), "FollowUp+": (1, 11, 0, True),
    "Halt": (0, 0, 3, False), "Halt+": (0, 0, 4, False),
    "Prostrate": (0, 0, 4, False), "Prostrate+": (0, 0, 4, False),
    "Tantrum": (1, 3, 0, True), "Tantrum+": (1, 3, 0, True),
    "InnerPeace": (1, 0, 0, False), "InnerPeace+": (1, 0, 0, False),
    "Crescendo": (1, 0, 0, False), "Crescendo+": (0, 0, 0, False),
    "Tranquility": (1, 0, 0, False), "Tranquility+": (0, 0, 0, False),
    "WheelKick": (2, 15, 0, True), "WheelKick+": (2, 20, 0, True),
    "Conclude": (1, 12, 0, True), "Conclude+": (1, 16, 0, True),
    "Ragnarok": (3, 5, 0, True), "Ragnarok+": (3, 5, 0, True),
    "SashWhip": (1, 8, 0, True), "SashWhip+": (1, 11, 0, True),
    "JustLucky": (0, 3, 0, True), "JustLucky+": (0, 4, 0, True),
    "TalkToTheHand": (1, 5, 0, True), "TalkToTheHand+": (1, 7, 0, True),
    "Wallop": (2, 9, 9, True), "Wallop+": (2, 12, 12, True),
    "Miracle": (0, 0, 0, False), "Miracle+": (0, 0, 0, False),
    "Smite": (1, 12, 0, True), "Smite+": (1, 16, 0, True),
    "Protect": (2, 0, 12, False), "Protect+": (2, 0, 16, False),
    "Worship": (2, 0, 0, False), "Worship+": (2, 0, 0, False),
    "PressurePoints": (1, 0, 0, False), "PressurePoints+": (1, 0, 0, False),
    # Status/Curse
    "Slimed": (1, 0, 0, False), "Wound": (-2, 0, 0, False),
    "Daze": (-2, 0, 0, False), "Burn": (-2, 0, 0, False),
    "AscendersBane": (-2, 0, 0, False),
}

# Stance-changing cards
_WRATH_CARDS = {"Eruption", "Eruption+", "Tantrum", "Tantrum+", "Crescendo", "Crescendo+"}
_CALM_CARDS = {"Vigilance", "Vigilance+", "InnerPeace", "InnerPeace+", "Tranquility", "Tranquility+"}
_EXIT_STANCE_CARDS = {"EmptyBody", "EmptyBody+", "EmptyFist", "EmptyFist+"}


# Per-turn plan cache for CombatPlanner (reset per combat turn)
_turn_plan_cache: Dict[str, Any] = {}  # {seed: (turn_num, card_queue, cards_played)}


def _try_combat_planner(actions, runner, combat_planner, state):
    """Try CombatPlanner for boss/elite. Returns action or None to fall back to heuristic.

    Re-plans every turn (not cached across turns). Handles infinite turns by
    switching to finish-fast mode after 50 cards in one turn.
    """
    from packages.training.combat_planner import CombatPlanner, TurnPlan

    seed = getattr(runner, "seed", "?")
    turn_num = getattr(state, "turn", 0)
    cache_key = seed

    # Check if we have a cached plan for this turn
    cached = _turn_plan_cache.get(cache_key)
    if cached is not None:
        cached_turn, card_queue, cards_played = cached
        if cached_turn == turn_num and card_queue:
            # Infinite loop detection: >50 cards in one turn = finish-fast mode
            if cards_played > 50:
                _turn_plan_cache.pop(cache_key, None)
                return None  # Fall back to heuristic (which prioritizes damage)

            # Try to match next planned card to available actions
            next_card_id, next_target = card_queue[0]
            for a in actions:
                if a.action_type == "play_card":
                    if a.card_idx < len(state.hand):
                        hand_card = str(state.hand[a.card_idx])
                        if hand_card == next_card_id:
                            target_match = (next_target is None or a.target_idx == next_target)
                            if target_match:
                                card_queue.pop(0)
                                _turn_plan_cache[cache_key] = (turn_num, card_queue, cards_played + 1)
                                return a

            # Planned card not in hand anymore (drawn new cards, etc.) — re-plan
            _turn_plan_cache.pop(cache_key, None)
        elif cached_turn != turn_num:
            # New turn — clear old plan
            _turn_plan_cache.pop(cache_key, None)

    # Plan this turn from scratch
    try:
        engine = runner.current_combat
        if engine is None:
            return None
        plan = combat_planner.plan_turn(engine)
        if plan and plan.card_sequence:
            card_queue = list(plan.card_sequence)
            next_card_id, next_target = card_queue[0]

            # Match first card to available actions
            for a in actions:
                if a.action_type == "play_card":
                    if a.card_idx < len(state.hand):
                        hand_card = str(state.hand[a.card_idx])
                        if hand_card == next_card_id:
                            target_match = (next_target is None or a.target_idx == next_target)
                            if target_match:
                                card_queue.pop(0)
                                _turn_plan_cache[cache_key] = (turn_num, card_queue, 1)
                                return a

            # No match found for first card — fall back to heuristic
            return None
    except Exception:
        return None

    return None


def _pick_combat_action(actions, runner, combat_planner=None, turn_solver_adapter=None):
    """Score combat actions and pick the best one.

    Priority: TurnSolver (all fights) > CombatPlanner (boss/elite) > heuristic.
    """
    if len(actions) <= 1:
        return actions[0]

    combat = runner.current_combat
    if combat is None:
        return actions[0]

    state = combat.state
    player = state.player
    enemies = state.enemies
    room_type = getattr(runner, "current_room_type", "monster")

    # TurnSolver: works for all fight types
    if turn_solver_adapter is not None:
        try:
            result = turn_solver_adapter.pick_action(actions, runner, room_type)
            if result is not None:
                return result
        except Exception:
            pass  # Fall through to heuristic

    # CombatPlanner fallback for boss/elite
    if combat_planner is not None and room_type in ("boss", "elite"):
        planned = _try_combat_planner(actions, runner, combat_planner, state)
        if planned is not None:
            return planned

    # Compute incoming damage
    incoming = 0
    for e in enemies:
        if e.hp > 0 and getattr(e, "move_damage", 0) > 0:
            incoming += e.move_damage * getattr(e, "move_hits", 1)

    stance = getattr(state, "stance", "Neutral")
    in_wrath = stance == "Wrath"
    in_calm = stance == "Calm"
    live_enemies = [e for e in enemies if e.hp > 0]
    n_live_enemies = len(live_enemies)
    total_enemy_hp = sum(e.hp for e in live_enemies)
    strength = player.statuses.get("Strength", 0)
    dexterity = player.statuses.get("Dexterity", 0)
    room_type = getattr(runner, "current_room_type", "monster")
    is_boss_or_elite = room_type in ("boss", "elite")
    player_hp = player.hp

    best_action = actions[0]
    best_score = -1000.0

    for a in actions:
        score = 0.0

        if a.action_type == "end_turn":
            score = -1.0 if state.energy > 0 else 5.0

        elif a.action_type == "play_card":
            if a.card_idx < 0 or a.card_idx >= len(state.hand):
                continue
            card_id = str(state.hand[a.card_idx])
            cost, base_dmg, base_block, is_attack = _CARD_STATS.get(
                card_id, (1, 6 if "Strike" in card_id else 0, 5 if "Defend" in card_id else 0, "Strike" in card_id)
            )

            # Damage scoring
            if base_dmg > 0:
                dmg = base_dmg + strength
                if in_wrath:
                    dmg = int(dmg * 2)
                # Vulnerable: 1.5x damage
                if 0 <= a.target_idx < len(enemies):
                    target = enemies[a.target_idx]
                    if target.statuses.get("Vulnerable", 0) > 0:
                        dmg = int(dmg * 1.5)
                # Lethal bonus
                if 0 <= a.target_idx < len(enemies) and enemies[a.target_idx].hp > 0:
                    if dmg >= enemies[a.target_idx].hp:
                        score += 20.0
                score += dmg * 1.5

            # Block scoring (weighted by incoming damage)
            if base_block > 0:
                block = base_block + dexterity
                useful_block = min(block, max(incoming, 1))
                score += useful_block * 2.0 + max(0, block - incoming) * 0.3
                # Panic block: triple priority when low HP and big incoming
                if player_hp < 15 and incoming > 10:
                    score *= 3.0

            # Focus-fire: bonus for targeting lowest-HP enemy (reduces incoming faster)
            if base_dmg > 0 and 0 <= a.target_idx < len(enemies):
                target = enemies[a.target_idx]
                if target.hp > 0 and n_live_enemies > 1:
                    # Extra bonus for killing weakest enemy this turn
                    if dmg >= target.hp:
                        score += 3.0  # On top of existing +20 lethal
                    # Slight preference for lower HP targets
                    if target.hp == min(e.hp for e in live_enemies):
                        score += 2.0

            # AoE bonus: prefer multi-target cards when 2+ enemies
            if n_live_enemies >= 2 and hasattr(a, "targets_all"):
                score += 4.0 * (n_live_enemies - 1)

            # Stance management (Watcher-critical)
            if card_id in _WRATH_CARDS:
                if incoming == 0 or total_enemy_hp <= state.energy * 10:
                    score += 15.0  # Safe Wrath
                elif in_wrath:
                    score += 5.0  # Already in Wrath, this is fine
                else:
                    score -= 10.0  # Dangerous to enter Wrath
            elif card_id in _CALM_CARDS:
                if in_wrath and incoming > 0:
                    score += 25.0  # Exit Wrath under fire
                elif in_wrath and incoming > player_hp // 2:
                    score += 30.0  # URGENT: exit Wrath, incoming > half HP
                elif not in_calm:
                    score += 5.0  # Bank energy
            elif card_id in _EXIT_STANCE_CARDS:
                if in_wrath and incoming > 0:
                    score += 20.0
                if in_wrath and incoming > player_hp // 2:
                    score += 25.0  # URGENT exit

            # Miracle: gain energy, always good to play early
            if card_id in ("Miracle", "Miracle+"):
                score += 12.0

            # Zero-cost cards: almost always worth playing
            if cost == 0:
                score += 8.0

            # Can't afford
            if cost > state.energy:
                score -= 100.0

        elif a.action_type == "use_potion":
            if is_boss_or_elite:
                # Boss/elite: use potions aggressively
                potion_id = ""
                if 0 <= a.potion_idx < len(runner.run_state.potion_slots):
                    potion_id = runner.run_state.potion_slots[a.potion_idx].potion_id or ""
                if any(k in potion_id for k in ("Fire", "Explosive", "Attack", "Strength")):
                    score = 18.0
                elif any(k in potion_id for k in ("Block", "Energy")):
                    score = 14.0
                else:
                    score = 8.0
            else:
                score = 3.0  # Save potions for boss/elite

        if score > best_score:
            best_score = score
            best_action = a

    return best_action


# ---------------------------------------------------------------------------
# Worker initializer — called once per worker process by mp.Pool
# ---------------------------------------------------------------------------

def _worker_init(request_q, response_qs, slot_q):
    """Called once per worker process to set up InferenceClient.

    Pops a unique slot_id from slot_q so each worker knows which
    response queue to listen on. If slot acquisition fails, the worker
    runs without an InferenceClient (falls back to heuristic planner).
    """
    from packages.training.inference_server import InferenceClient
    try:
        slot_id = slot_q.get(timeout=10)
    except Exception:
        # No slot available — worker runs without inference server.
        # This is safer than defaulting to slot 0 (which would collide).
        logger.warning("Worker failed to acquire slot from slot_q — running heuristic-only")
        return
    InferenceClient.setup_worker(request_q, response_qs[slot_id], slot_id)


# ---------------------------------------------------------------------------
# Worker function — runs in subprocess via mp.Pool
# ---------------------------------------------------------------------------

def _play_one_game(
    seed: str,
    ascension: int,
    temperature: float,
    total_games: int = 0,
    epsilon_mode: str = "none",
    epsilon_start: float = 0.8,
    epsilon_end: float = 0.3,
    epsilon_decay: int = 50000,
) -> Dict[str, Any]:
    """Play a single game and return transitions + result.

    This function runs in a worker process. Workers are torch-free: all
    neural-network inference is delegated to the InferenceServer running
    in the main process via InferenceClient. If the server is unavailable
    (client is None or request times out), the worker falls back to the
    heuristic StrategicPlanner.

    Epsilon-greedy: with probability epsilon, use heuristic planner instead
    of NN for strategic decisions (still records NN value/log_prob for training).
    Epsilon decays from 1.0 to 0.3 over 50K games.

    Returns a dict with:
        seed, won, floor, hp, decisions, duration_s,
        transitions: list of dicts with (obs, action_mask, action, reward,
                     done, value, log_prob, final_floor, cleared_act1/2/3)
    """
    import random as _random

    from packages.engine.game import GameRunner, GamePhase, CombatAction
    from packages.training.planner import StrategicPlanner
    from packages.training.combat_planner import CombatPlanner
    from packages.training.state_encoder_v2 import RunStateEncoder
    from packages.training.inference_server import get_client

    from packages.training.turn_solver import TurnSolverAdapter

    encoder = RunStateEncoder()
    planner = StrategicPlanner()
    combat_planner = CombatPlanner(top_k=3, lookahead_turns=1)  # Fast config for training
    turn_solver = TurnSolverAdapter(time_budget_ms=5.0, node_budget=1000)

    client = get_client()

    try:
        runner = GameRunner(seed=seed, ascension=ascension, character="Watcher", verbose=False)
    except Exception:
        return {
            "seed": seed, "won": False, "floor": 0, "hp": 0,
            "decisions": 0, "duration_s": 0.0, "transitions": [],
        }

    t0 = time.monotonic()
    step = 0
    prev_floor = 0
    prev_potential = compute_potential(runner.run_state)
    decisions = 0
    transitions: List[Dict[str, Any]] = []

    # Track combat events for event-based rewards
    was_in_combat = False
    combat_room_type = "monster"

    while not runner.game_over and step < 5000:
        try:
            actions = runner.get_available_actions()
        except Exception:
            break
        if not actions:
            break

        phase = runner.phase
        rs = runner.run_state
        current_floor = getattr(rs, "floor", 0)

        if phase == GamePhase.COMBAT:
            was_in_combat = True
            combat_room_type = getattr(runner, "current_room_type", "monster")
            # Combat: TurnSolver > CombatPlanner > heuristic
            runner.take_action(_pick_combat_action(actions, runner, combat_planner, turn_solver))
        elif len(actions) == 1:
            # Check for combat-end event rewards
            if was_in_combat and phase != GamePhase.COMBAT:
                was_in_combat = False
            runner.take_action(actions[0])
        else:
            # Strategic decision point
            decisions += 1

            # Check if combat just ended (for event rewards)
            combat_just_ended = was_in_combat and phase != GamePhase.COMBAT
            if combat_just_ended:
                was_in_combat = False

            n_actions = len(actions)

            # Map GamePhase to phase_type for state encoding
            _PHASE_MAP = {
                GamePhase.MAP_NAVIGATION: "path",
                GamePhase.COMBAT_REWARDS: "card_pick",
                GamePhase.BOSS_REWARDS: "card_pick",
                GamePhase.REST: "rest",
                GamePhase.SHOP: "shop",
                GamePhase.EVENT: "event",
                GamePhase.NEOW: "other",
                GamePhase.TREASURE: "other",
            }
            phase_type = _PHASE_MAP.get(phase, "other")

            # Encode state with phase context (always needed for obs recording)
            run_obs = encoder.encode(rs, phase_type=phase_type)

            # ACTION_DIM constant — must match StrategicNet.action_dim
            _ACTION_DIM = 256
            mask = np.zeros(_ACTION_DIM, dtype=np.bool_)
            mask[:n_actions] = True

            # --- Inference via centralized server ---
            action_idx: int = 0
            value: float = 0.0
            log_prob: float = 0.0
            logits_np: Optional[np.ndarray] = None

            if client is not None:
                resp = client.infer_strategic(run_obs, n_actions)
                if resp is not None and resp.get("ok"):
                    logits_np = resp["logits"]  # numpy array, length action_dim
                    value = float(resp["value"])

                    # Temperature-scaled sampling
                    if temperature > 0:
                        logits_scaled = logits_np / temperature
                        logits_scaled = logits_scaled - logits_scaled.max()
                        probs = np.exp(logits_scaled)
                        probs /= probs.sum()
                        action_idx = int(np.random.choice(len(probs), p=probs))
                    else:
                        action_idx = int(np.argmax(logits_np))

                    # log_prob on unscaled logits (for PPO IS ratio)
                    logits_base = logits_np - logits_np.max()
                    probs_base = np.exp(logits_base)
                    probs_base /= probs_base.sum()
                    log_prob = float(np.log(probs_base[action_idx] + 1e-8))

                    # Clamp to valid range
                    action_idx = min(action_idx, n_actions - 1)
                else:
                    # Server timed out or returned error — fall back to heuristic
                    client = None

            if client is not None and logits_np is not None:
                # Recompute probs_base for behavior policy log_prob
                logits_base = logits_np - logits_np.max()
                probs_base = np.exp(logits_base)
                probs_base /= probs_base.sum()

                # Epsilon-greedy heuristic override (only in importance_weighted mode)
                used_heuristic = False
                if epsilon_mode == "importance_weighted":
                    epsilon = max(epsilon_end, epsilon_start - total_games / max(epsilon_decay, 1))
                    if _random.random() < epsilon:
                        if phase == GamePhase.MAP_NAVIGATION:
                            heuristic_idx = planner.plan_path_choice(runner, actions)
                        elif phase == GamePhase.REST:
                            heuristic_idx = planner.plan_rest_site(runner, actions)
                        elif phase in (GamePhase.COMBAT_REWARDS, GamePhase.BOSS_REWARDS):
                            heuristic_idx = planner.plan_card_pick(runner, actions)
                        elif phase == GamePhase.SHOP:
                            heuristic_idx = planner.plan_shop_action(runner, actions)
                        elif phase == GamePhase.EVENT:
                            heuristic_idx = planner.plan_event_choice(runner, actions)
                        else:
                            heuristic_idx = action_idx
                        action_idx = min(heuristic_idx, n_actions - 1)
                        used_heuristic = True

                    # Correct behavior policy: b(a|s) = (1-eps)*pi(a|s) + eps*I(a==heuristic)
                    pi_prob = float(probs_base[action_idx])
                    if used_heuristic:
                        behavior_prob = (1.0 - epsilon) * pi_prob + epsilon
                    else:
                        behavior_prob = (1.0 - epsilon) * pi_prob
                    log_prob = float(np.log(max(behavior_prob, 1e-8)))
                # else: epsilon_mode == "none" — pure on-policy, log_prob already correct

            if client is not None:
                # --- PBRS reward ---
                # Take action first, then compute Phi(s') - gamma * Phi(s)
                runner.take_action(actions[action_idx])
                new_rs = runner.run_state
                new_potential = compute_potential(new_rs)

                # PBRS: gamma * Phi(s') - Phi(s) preserves optimal policy
                gamma = 1.0
                pbrs_reward = gamma * new_potential - prev_potential

                # Event-based rewards on top of PBRS
                event_reward = 0.0
                if combat_just_ended:
                    rt = combat_room_type.lower() if isinstance(combat_room_type, str) else "monster"
                    if rt in ("elite", "e"):
                        event_reward = EVENT_REWARDS["elite_win"]
                    elif rt in ("boss", "b"):
                        event_reward = EVENT_REWARDS["boss_win"]
                    else:
                        event_reward = EVENT_REWARDS["combat_win"]
                    # Scale by HP efficiency
                    hp_pct = new_rs.current_hp / max(new_rs.max_hp, 1)
                    event_reward *= (0.5 + 0.5 * hp_pct)

                reward = pbrs_reward + event_reward
                prev_potential = new_potential

                # Record transition as numpy-serializable dict
                transitions.append({
                    "obs": run_obs,
                    "action_mask": mask,
                    "action": action_idx,
                    "reward": reward,
                    "done": False,
                    "value": value,
                    "log_prob": log_prob,
                    "final_floor": 0.0,
                    "cleared_act1": 0.0,
                    "cleared_act2": 0.0,
                    "cleared_act3": 0.0,
                })

            else:
                # No inference server available: use heuristic planner
                if phase == GamePhase.MAP_NAVIGATION:
                    idx = planner.plan_path_choice(runner, actions)
                elif phase == GamePhase.REST:
                    idx = planner.plan_rest_site(runner, actions)
                elif phase in (GamePhase.COMBAT_REWARDS, GamePhase.BOSS_REWARDS):
                    idx = planner.plan_card_pick(runner, actions)
                elif phase == GamePhase.SHOP:
                    idx = planner.plan_shop_action(runner, actions)
                elif phase == GamePhase.EVENT:
                    idx = planner.plan_event_choice(runner, actions)
                else:
                    idx = 0
                runner.take_action(actions[min(idx, n_actions - 1)])
                prev_potential = compute_potential(runner.run_state)

        step += 1
        prev_floor = current_floor

    # Game ended
    duration = time.monotonic() - t0
    rs = runner.run_state
    won = runner.game_won
    final_floor = getattr(rs, "floor", 0)
    final_hp = getattr(rs, "current_hp", 0)

    cleared_acts = [
        final_floor >= 17,
        final_floor >= 34,
        final_floor >= 51,
    ]

    # Terminal reward on last transition (only if game truly ended)
    truly_terminal = runner.game_over or won
    if transitions:
        if truly_terminal:
            if won:
                transitions[-1]["reward"] += 1.0
            else:
                progress = final_floor / 55.0
                transitions[-1]["reward"] += -0.5 * (1 - progress)
            transitions[-1]["done"] = True
        # Truncated games (step >= 5000, error): leave done=False so GAE
        # bootstraps from value estimate instead of zero

    # Backfill aux targets
    for t in transitions:
        t["final_floor"] = final_floor / 55.0
        t["cleared_act1"] = float(cleared_acts[0])
        t["cleared_act2"] = float(cleared_acts[1])
        t["cleared_act3"] = float(cleared_acts[2])

    # Capture deck and death info for episode logging
    deck_final = []
    try:
        deck_final = [
            (c.id + "+" if getattr(c, "upgraded", False) else c.id)
            if hasattr(c, "id") else str(c)
            for c in getattr(rs, "deck", [])
        ]
    except Exception:
        pass

    death_enemy = ""
    try:
        if not won and runner.current_combat is not None:
            for e in runner.current_combat.state.enemies:
                if e.hp > 0:
                    death_enemy = getattr(e, "name", getattr(e, "id", ""))
                    break
    except Exception:
        pass

    return {
        "seed": seed,
        "won": won,
        "floor": final_floor,
        "hp": final_hp,
        "decisions": decisions,
        "duration_s": round(duration, 2),
        "transitions": transitions,
        "deck_final": deck_final,
        "death_enemy": death_enemy,
        "room_type": getattr(runner, "current_room_type", ""),
    }


# ---------------------------------------------------------------------------
# OvernightRunner — orchestrates training loop + multiprocessing
# ---------------------------------------------------------------------------

class OvernightRunner:
    """Manages overnight training with scheduling and sweep.

    Config keys:
        headless_after_min: Minutes after start to go headless (default 30)
        visual_at: HH:MM time to switch back to visual mode (default "07:30")
        sweep_configs: List of hyperparameter dicts to sweep
        run_dir: Directory for logs and checkpoints
        max_games: Maximum total games to play (default 50000)
        games_per_batch: Games per training batch (default 16)
        workers: Number of parallel workers (default 8)
        ascension: Ascension level (default 0 for initial training)
        eval_every: Games between evaluation runs (default 500)
        ppo_batch_size: PPO mini-batch size (default 256)
        temperature: Exploration temperature for strategic decisions (default 1.0)
    """

    def __init__(self, config: Dict[str, Any]):
        self.headless_after_min = config.get("headless_after_min", 30)
        self.visual_at = config.get("visual_at", "07:30")
        self.sweep_configs = config.get("sweep_configs", DEFAULT_SWEEP_CONFIGS)
        self.run_dir = Path(config.get("run_dir", "logs/overnight"))
        self.max_games = config.get("max_games", 50000)
        self.games_per_batch = config.get("games_per_batch", 16)
        self.workers = config.get("workers", 8)
        self.ascension = config.get("ascension", 0)
        self.eval_every = config.get("eval_every", 500)
        self.ppo_batch_size = config.get("ppo_batch_size", 256)
        self.temperature = config.get("temperature", 1.0)
        self.resume_path = config.get("resume_path", None)

        self.run_dir.mkdir(parents=True, exist_ok=True)
        self._start_time = time.monotonic()
        self._start_datetime = datetime.now()
        self._current_sweep_idx = 0
        self._games_per_sweep = self.max_games // max(len(self.sweep_configs), 1)

        # Episodes log file
        self._episodes_path = self.run_dir / "episodes.jsonl"

        # Graceful shutdown flag (set by signal handler)
        self._shutdown_requested = False

        # Stats tracking
        self.total_games = 0
        self.total_wins = 0
        self.recent_floors: Deque[int] = deque(maxlen=100)
        self.recent_wins: Deque[bool] = deque(maxlen=100)
        self.sweep_results: List[Dict[str, Any]] = []
        self._episode_counter = 0  # Unique ID per game for GAE episode separation

        # Current sweep config (set by _run_config for epsilon forwarding)
        self._current_sweep_config: Dict[str, Any] = {}

        # Inference server + persistent pool (created in run())
        self._server = None
        self._executor: Optional[Any] = None

    def should_be_headless(self) -> bool:
        """Check if we should be in headless mode based on schedule."""
        elapsed_min = (time.monotonic() - self._start_time) / 60.0
        if elapsed_min < self.headless_after_min:
            return False

        # Check if we've hit the visual_at time
        now = datetime.now()
        try:
            hour, minute = map(int, self.visual_at.split(":"))
            if now.hour == hour and now.minute >= minute:
                return False
            if now.hour > hour:
                return False
        except (ValueError, AttributeError):
            pass

        return True

    def get_current_sweep_config(self) -> Optional[Dict[str, Any]]:
        """Return current hyperparameter config from sweep schedule."""
        if not self.sweep_configs:
            return None
        if self._current_sweep_idx >= len(self.sweep_configs):
            return None
        return self.sweep_configs[self._current_sweep_idx]

    def _advance_sweep(self) -> bool:
        """Advance to next sweep config. Returns False if sweep is done."""
        self._current_sweep_idx += 1
        return self._current_sweep_idx < len(self.sweep_configs)

    def write_status(self, stats: Dict[str, Any]) -> None:
        """Write status.json for monitoring."""
        elapsed = time.monotonic() - self._start_time
        games_per_min = self.total_games / max(elapsed / 60.0, 0.01)
        status = {
            "timestamp": datetime.now().isoformat(),
            "elapsed_hours": round(elapsed / 3600, 2),
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "win_rate_100": round(sum(self.recent_wins) / max(len(self.recent_wins), 1) * 100, 1),
            "avg_floor_100": round(sum(self.recent_floors) / max(len(self.recent_floors), 1), 1),
            "games_per_min": round(games_per_min, 1),
            "current_sweep": self._current_sweep_idx,
            "total_sweeps": len(self.sweep_configs),
            "headless": self.should_be_headless(),
            **stats,
        }
        status_path = self.run_dir / "status.json"
        status_path.write_text(json.dumps(status, indent=2))

    def _record_game(self, won: bool, floor: int) -> None:
        """Record a game result."""
        self.total_games += 1
        if won:
            self.total_wins += 1
        self.recent_floors.append(floor)
        self.recent_wins.append(won)

    def _log_episode(self, result: Dict[str, Any]) -> None:
        """Append one episode to episodes.jsonl."""
        entry = {
            "timestamp": datetime.now().isoformat(),
            "seed": result["seed"],
            "floor": result["floor"],
            "won": result["won"],
            "hp": result["hp"],
            "decisions": result["decisions"],
            "duration_s": result["duration_s"],
            "num_transitions": len(result.get("transitions", [])),
            "deck_final": result.get("deck_final", []),
            "death_enemy": result.get("death_enemy", ""),
            "death_room": result.get("room_type", ""),
        }
        with open(self._episodes_path, "a") as f:
            f.write(json.dumps(entry) + "\n")

    def run(self) -> Dict[str, Any]:
        """Main overnight loop.

        Integrates with StrategicTrainer to train the strategic model
        while using the combat solver for combat phases.
        """
        import torch
        from .strategic_net import StrategicNet, _get_device
        from .strategic_trainer import StrategicTrainer
        from .state_encoder_v2 import RunStateEncoder
        from .self_play import SeedPool

        device = _get_device()
        encoder = RunStateEncoder()

        # Initialize model (optionally resume from checkpoint)
        if self.resume_path:
            try:
                model = StrategicNet.load(self.resume_path, device=device)
                model.train()
                logger.info("Resumed model from %s", self.resume_path)
            except Exception as e:
                logger.warning("Failed to resume from %s: %s — starting fresh", self.resume_path, e)
                model = StrategicNet(input_dim=encoder.RUN_DIM).to(device)
        else:
            model = StrategicNet(input_dim=encoder.RUN_DIM).to(device)
        logger.info(
            "Strategic model: %d parameters, device=%s",
            model.param_count(), device,
        )

        # --- Inference server setup ---
        from packages.training.inference_server import InferenceServer

        self._server = InferenceServer(
            n_workers=self.workers, max_batch_size=self.workers
        )
        self._server.sync_strategic_from_pytorch(model, version=0)
        self._server.start()
        logger.info("InferenceServer started (workers=%d)", self.workers)

        # Persistent worker pool using server's queues (spawn context for Metal safety)
        ctx = mp.get_context("spawn")
        self._executor = ctx.Pool(
            processes=self.workers,
            initializer=_worker_init,
            initargs=(self._server.request_q, self._server.response_qs, self._server.slot_q),
        )
        logger.info("Worker pool started (%d processes)", self.workers)

        # --- Signal handler for graceful shutdown ---
        def _handle_shutdown(signum, frame):
            sig_name = signal.Signals(signum).name
            logger.info("Graceful shutdown requested (%s), finishing current batch...", sig_name)
            self._shutdown_requested = True

        signal.signal(signal.SIGTERM, _handle_shutdown)
        signal.signal(signal.SIGINT, _handle_shutdown)

        seed_pool = SeedPool(max_plays=5)
        best_avg_floor = 0.0

        # Adaptive 3-phase sweep:
        # Phase 1: Each config gets equal games (~25% of total each)
        # Phase 2: Keep top 2 configs, each gets ~15% more games
        # Phase 3: All-in on best config with remaining games
        n_configs = len(self.sweep_configs)
        phase1_games_per = self.max_games // (n_configs * 3)  # ~33% of budget split equally
        phase2_games_per = self.max_games // 9                 # ~33% split between top 3
        # phase3 gets the rest

        config_scores: Dict[int, Dict[str, Any]] = {}  # idx -> {avg_floor, games, ...}

        def _run_config(sweep_idx: int, sweep_config: Dict, n_games: int) -> Dict[str, Any]:
            """Run a config for n_games, return metrics."""
            nonlocal best_avg_floor
            self._current_sweep_idx = sweep_idx
            self._current_sweep_config = sweep_config
            lr = sweep_config.get("lr", 1e-4)
            batch_size = sweep_config.get("batch_size", self.ppo_batch_size)
            temp = sweep_config.get("temperature", self.temperature)
            self.temperature = temp

            trainer = StrategicTrainer(
                model=model,
                lr=lr,
                entropy_coeff=sweep_config.get("entropy_coeff", 0.05),
                clip_epsilon=sweep_config.get("clip_epsilon", 0.2),
                batch_size=batch_size,
                lr_schedule=sweep_config.get("lr_schedule", "cosine"),
                lr_T_max=sweep_config.get("lr_T_max", 30000),
                lr_T_0=sweep_config.get("lr_T_0", 5000),
            )

            config_name = sweep_config.get("name", f"config_{sweep_idx}")
            sweep_games = 0
            sweep_start = time.monotonic()
            sweep_floors: Deque[int] = deque(maxlen=200)

            logger.info(
                "Config '%s': lr=%.1e, ent=%.3f, batch=%d, temp=%.1f",
                config_name, lr,
                sweep_config.get("entropy_coeff", 0.05),
                batch_size, temp,
            )

            while sweep_games < n_games and self.total_games < self.max_games and not self._shutdown_requested:
                batch_t0 = time.monotonic()
                batch_results = self._play_batch(model, encoder, seed_pool, trainer)
                batch_duration = time.monotonic() - batch_t0

                for result in batch_results:
                    self._record_game(result["won"], result["floor"])
                    self._log_episode(result)
                    sweep_games += 1
                    sweep_floors.append(result["floor"])

                games_per_min = len(batch_results) / max(batch_duration / 60.0, 0.01)

                # Train if enough transitions
                if len(trainer.buffer) >= trainer.batch_size:
                    metrics = trainer.train_batch()
                    # Only decay entropy when there's learning signal
                    # (avg floor above random baseline of ~5)
                    avg_floor = sum(self.recent_floors) / max(len(self.recent_floors), 1)
                    if avg_floor > 7.0:
                        trainer.decay_entropy(min_coeff=0.01, decay=0.999)
                    elif avg_floor > 5.5:
                        # Slow decay while still exploring
                        trainer.decay_entropy(min_coeff=0.02, decay=0.9999)

                    # Sync updated weights to inference server
                    if self._server is not None:
                        self._server.sync_strategic_from_pytorch(
                            model, version=trainer.train_steps
                        )

                    avg_floor = sum(self.recent_floors) / max(len(self.recent_floors), 1)
                    wr = sum(self.recent_wins) / max(len(self.recent_wins), 1)

                    logger.info(
                        "[%s] Games %d | Floor %.1f | WR %.1f%% | Loss %.4f | "
                        "Ent %.3f | LR %.1e | %.1f g/min",
                        config_name, self.total_games, avg_floor, wr * 100,
                        metrics.get("total_loss", 0),
                        metrics.get("entropy_coeff", 0),
                        metrics.get("lr", 0),
                        games_per_min,
                    )

                    if trainer.maybe_checkpoint(avg_floor):
                        best_avg_floor = avg_floor
                        logger.info("New best avg floor: %.1f", avg_floor)

                    # Adaptive ascension scaling
                    if self.total_games % 5000 < self.games_per_batch:
                        self._check_ascension_bump()

                self.write_status({
                    "sweep_config": sweep_config,
                    "sweep_phase": "adaptive",
                    "config_name": config_name,
                    "sweep_games": sweep_games,
                    "train_steps": trainer.train_steps,
                    "buffer_size": len(trainer.buffer),
                    "games_per_min": round(games_per_min, 1),
                    "entropy_coeff": trainer.entropy_coeff,
                })

            sweep_elapsed = time.monotonic() - sweep_start
            sweep_avg = sum(sweep_floors) / max(len(sweep_floors), 1)

            result_info = {
                "config": sweep_config,
                "games": sweep_games,
                "avg_floor": round(sweep_avg, 1),
                "win_rate": round(sum(self.recent_wins) / max(len(self.recent_wins), 1) * 100, 1),
                "duration_min": round(sweep_elapsed / 60, 1),
                "train_steps": trainer.train_steps,
            }
            self.sweep_results.append(result_info)
            return result_info

        # Phase 1: Explore all configs
        logger.info("=== Phase 1: Exploring %d configs (%d games each) ===",
                     n_configs, phase1_games_per)
        for idx, cfg in enumerate(self.sweep_configs):
            if self._shutdown_requested:
                break
            result = _run_config(idx, cfg, phase1_games_per)
            config_scores[idx] = result

        # Phase 2: Keep top 3 by avg_floor
        if not self._shutdown_requested and config_scores:
            sorted_configs = sorted(config_scores.items(),
                                    key=lambda x: x[1]["avg_floor"], reverse=True)
            top3 = sorted_configs[:3]
            top_names = ", ".join(
                f"{self.sweep_configs[idx].get('name', '?')} ({info['avg_floor']:.1f})"
                for idx, info in top3
            )
            logger.info("=== Phase 2: Top 3 configs: %s ===", top_names)

            for idx, _ in top3:
                if self._shutdown_requested:
                    break
                result = _run_config(idx, self.sweep_configs[idx], phase2_games_per)
                config_scores[idx] = result

        # Phase 3: All-in on best
        if not self._shutdown_requested and config_scores:
            sorted_configs = sorted(config_scores.items(),
                                    key=lambda x: x[1]["avg_floor"], reverse=True)
            best_idx = sorted_configs[0][0]
            best_cfg = self.sweep_configs[best_idx]
            remaining = self.max_games - self.total_games
            logger.info("=== Phase 3: All-in on '%s' (%.1f avg floor, %d games remaining) ===",
                         best_cfg.get("name", "?"), sorted_configs[0][1]["avg_floor"], remaining)

            if remaining > 0:
                _run_config(best_idx, best_cfg, remaining)

        # Save checkpoint and clean up
        if self._shutdown_requested:
            logger.info("Saving checkpoint before shutdown...")
            model.save(self.run_dir / "shutdown_checkpoint.pt")
            logger.info("Checkpoint saved to %s", self.run_dir / "shutdown_checkpoint.pt")
        model.save(self.run_dir / "final_strategic.pt")
        self._write_summary()

        # Cleanup inference server and worker pool
        if self._executor is not None:
            self._executor.terminate()
            self._executor.join()
            self._executor = None
        if self._server is not None:
            self._server.stop()
            self._server = None

        if self._shutdown_requested:
            logger.info("Graceful shutdown complete. %d games played.", self.total_games)

        return {
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "best_avg_floor": best_avg_floor,
            "sweep_results": self.sweep_results,
        }

    def _check_ascension_bump(self) -> None:
        """Check if we should increase ascension based on recent performance.

        Evaluated against rolling 1K-game window (or whatever recent_floors holds).
        Only increases, never decreases.
        """
        if len(self.recent_floors) < 50:
            return
        avg_floor = sum(self.recent_floors) / len(self.recent_floors)
        wr = sum(self.recent_wins) / max(len(self.recent_wins), 1)
        for min_floor, min_wr, target_asc in ASCENSION_BREAKPOINTS:
            if avg_floor >= min_floor and wr >= min_wr and self.ascension < target_asc:
                logger.info(
                    "Ascension bump: A%d -> A%d (avg_floor=%.1f, WR=%.1f%%)",
                    self.ascension, target_asc, avg_floor, wr * 100,
                )
                self.ascension = target_asc

    def _play_batch(
        self,
        model,
        encoder,
        seed_pool,
        trainer,
    ) -> List[Dict[str, Any]]:
        """Play a batch of games in parallel using the persistent mp.Pool.

        Workers are torch-free; inference is handled by the InferenceServer
        running in the main process. No weight serialization per batch.
        """
        seeds = [seed_pool.get_seed() for _ in range(self.games_per_batch)]

        # Extract epsilon params from current sweep config
        cfg = self._current_sweep_config
        eps_mode = cfg.get("epsilon_mode", "none")
        eps_start = cfg.get("epsilon_start", 0.8)
        eps_end = cfg.get("epsilon_end", 0.3)
        eps_decay = cfg.get("epsilon_decay", 50000)

        # Submit all games to the persistent pool
        async_results = [
            self._executor.apply_async(
                _play_one_game,
                (seed, self.ascension, self.temperature, self.total_games,
                 eps_mode, eps_start, eps_end, eps_decay),
            )
            for seed in seeds
        ]

        results: List[Dict[str, Any]] = []
        timed_out = False
        for ar, seed in zip(async_results, seeds):
            try:
                result = ar.get(timeout=120)
            except Exception as e:
                logger.warning("Game %s failed: %s", seed, e)
                result = {
                    "seed": seed, "won": False, "floor": 0, "hp": 0,
                    "decisions": 0, "duration_s": 0.0, "transitions": [],
                }
                timed_out = True

            # Add transitions from this game to trainer buffer
            # Each game gets a unique episode_id for per-episode GAE
            self._episode_counter += 1
            ep_id = self._episode_counter
            for t in result.get("transitions", []):
                trainer.add_transition(
                    obs=t["obs"],
                    action_mask=t["action_mask"],
                    action=t["action"],
                    reward=t["reward"],
                    done=t["done"],
                    value=t["value"],
                    log_prob=t["log_prob"],
                    episode_id=ep_id,
                )
                # Backfill aux targets
                buf_t = trainer.buffer[-1]
                buf_t.final_floor = t["final_floor"]
                buf_t.cleared_act1 = t["cleared_act1"]
                buf_t.cleared_act2 = t["cleared_act2"]
                buf_t.cleared_act3 = t["cleared_act3"]

            seed_pool.record_result(seed, {"won": result["won"], "floor": result["floor"]})
            results.append(result)

        # If any worker timed out, recreate pool to avoid hung workers
        if timed_out and self._executor is not None:
            logger.warning("Recreating worker pool after timeout")
            try:
                self._executor.terminate()
                self._executor.join(timeout=5)
            except Exception:
                pass
            # Drain and refill slot queue for clean worker init
            while not self._server.slot_q.empty():
                try:
                    self._server.slot_q.get_nowait()
                except Exception:
                    break
            for i in range(self._server.n_workers):
                self._server.slot_q.put(i)
            ctx = mp.get_context("spawn")
            self._executor = ctx.Pool(
                processes=self.workers,
                initializer=_worker_init,
                initargs=(self._server.request_q, self._server.response_qs, self._server.slot_q),
            )

        return results

    def _write_summary(self) -> None:
        """Write a summary of the overnight run."""
        summary = {
            "start": self._start_datetime.isoformat(),
            "end": datetime.now().isoformat(),
            "elapsed_hours": round((time.monotonic() - self._start_time) / 3600, 2),
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "final_win_rate": round(self.total_wins / max(self.total_games, 1) * 100, 1),
            "final_avg_floor": round(sum(self.recent_floors) / max(len(self.recent_floors), 1), 1),
            "sweep_results": self.sweep_results,
        }
        (self.run_dir / "summary.json").write_text(json.dumps(summary, indent=2))
        logger.info("Overnight run complete. Summary written to %s", self.run_dir / "summary.json")


# Import for backward compat — the canonical source is strategic_trainer.py
STRATEGIC_REWARDS = {
    "floor_cleared": 0.01,
    "normal_kill": 0.05,
    "elite_kill": 0.15,
    "boss_kill": 0.40,
    "game_win": 1.0,
    "game_loss_base": -0.3,
    "hp_efficiency_scale": 0.05,
}


def main():
    """CLI entry point for overnight training."""
    import argparse

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s | %(levelname)s | %(message)s",
        datefmt="%H:%M:%S",
    )

    parser = argparse.ArgumentParser(description="Overnight training runner")
    parser.add_argument("--workers", type=int, default=8, help="Number of parallel workers")
    parser.add_argument("--games", type=int, default=50000, help="Maximum total games")
    parser.add_argument("--batch", type=int, default=16, help="Games per batch")
    parser.add_argument("--batch-size", type=int, default=256, help="PPO mini-batch size")
    parser.add_argument("--ascension", type=int, default=0, help="Ascension level")
    parser.add_argument("--run-dir", type=str, default="logs/overnight", help="Output directory")
    parser.add_argument("--headless-after", type=int, default=30, help="Go headless after N minutes")
    parser.add_argument("--visual-at", type=str, default="07:30", help="Switch to visual at HH:MM")
    parser.add_argument("--temperature", type=float, default=1.0, help="Exploration temperature (0=greedy)")
    parser.add_argument("--resume", type=str, default=None, help="Path to checkpoint .pt to resume from")
    args = parser.parse_args()

    runner = OvernightRunner({
        "workers": args.workers,
        "max_games": args.games,
        "games_per_batch": args.batch,
        "ppo_batch_size": args.batch_size,
        "ascension": args.ascension,
        "run_dir": args.run_dir,
        "headless_after_min": args.headless_after,
        "visual_at": args.visual_at,
        "temperature": args.temperature,
        "resume_path": args.resume,
    })

    result = runner.run()
    logger.info(
        "Done: %d games, %d wins (%.1f%%), best floor %.1f",
        result["total_games"], result["total_wins"],
        result["total_wins"] / max(result["total_games"], 1) * 100,
        result["best_avg_floor"],
    )


if __name__ == "__main__":
    main()
