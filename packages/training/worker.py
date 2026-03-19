"""Game worker: plays one game in a subprocess for parallel training.

Contains:
- _worker_init(): per-process setup for InferenceClient
- _play_one_game(): full game loop with transitions for PPO training
- _pick_combat_action(): simplified combat action selection (TurnSolver only)
"""

from __future__ import annotations

import json
import logging
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

import numpy as np

from .reward_config import (
    EVENT_REWARDS,
    FLOOR_MILESTONES,
    REWARD_WEIGHTS,
    UPGRADE_REWARDS,
    compute_potential,
)
from .training_config import MODEL_ACTION_DIM

logger = logging.getLogger(__name__)

# ACTION_DIM constant — sourced from training_config, must match StrategicNet.action_dim
_ACTION_DIM = MODEL_ACTION_DIM

# ---------------------------------------------------------------------------
# Strategic search — value-head per-option evaluation
# ---------------------------------------------------------------------------

def _strategic_search(actions, runner, encoder, client, phase_type, n_actions):
    """Evaluate each option via value head, return best + search policy."""
    if client is None or n_actions <= 1:
        return None
    option_values = []
    for i in range(n_actions):
        obs = encoder.encode(runner.run_state, phase_type=phase_type,
            boss_name=getattr(runner, "_boss_name", ""),
            room_type=getattr(runner, "current_room_type", ""),
            actions=actions, runner=runner)
        resp = client.infer_strategic(obs, 1)
        if resp and resp.get("ok"):
            option_values.append(float(resp["value"]))
        else:
            return None
    vals = np.array(option_values)
    vals = vals - vals.max()
    search_policy = np.exp(vals * 2.0)
    search_policy /= search_policy.sum()
    best_idx = int(np.argmax(option_values))
    return best_idx, search_policy


# ---------------------------------------------------------------------------
# Worker initializer — called once per worker process by mp.Pool
# ---------------------------------------------------------------------------

# Worker name — set per-process in _worker_init
_worker_name = "Unknown"

# Worker names — mapped by slot_id for dashboard display
_WORKER_NAMES = [
    "Vengeance", "Fury", "Zen", "Vigilante", "Serenity", "Tempest",
    "Oracle", "Ascendant", "Sentinel", "Harmony", "Specter", "Eclipse",
]


def _worker_init(request_q, response_qs, slot_q):
    """Called once per worker process to set up InferenceClient.

    Pops a unique slot_id from slot_q so each worker knows which
    response queue to listen on. If slot acquisition fails, the worker
    runs without an InferenceClient (falls back to first legal action).
    """
    global _worker_name
    from packages.training.inference_server import InferenceClient
    try:
        slot_id = slot_q.get(timeout=10)
    except Exception:
        logger.warning("Worker failed to acquire slot from slot_q — running without inference")
        _worker_name = "NoSlot"
        return
    InferenceClient.setup_worker(request_q, response_qs[slot_id], slot_id)
    _worker_name = _WORKER_NAMES[slot_id % len(_WORKER_NAMES)]


# ---------------------------------------------------------------------------
# Combat action selection — TurnSolver only, no heuristic fallback
# ---------------------------------------------------------------------------

def _pick_combat_action(actions, runner, turn_solver_adapter=None):
    """Pick a combat action. TurnSolver first, then prefer card plays over EndTurn.

    Safety net: if solver returns EndTurn but playable cards exist, prefer a card.
    This prevents the 5.9% zero-card-played bug where the solver ends turn immediately.
    """
    if len(actions) <= 1:
        return actions[0]

    combat = runner.current_combat
    if combat is None:
        return actions[0]

    room_type = getattr(runner, "current_room_type", "monster")

    # TurnSolver: works for all fight types
    if turn_solver_adapter is not None:
        try:
            result = turn_solver_adapter.pick_action(actions, runner, room_type)
            if result is not None:
                return result  # Trust the solver's decision (including EndTurn)
        except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
            logger.warning("_pick_combat_action: TurnSolver failed: %s", e)

    # Fallback (solver returned None): prefer playing a card over ending turn
    for a in actions:
        if hasattr(a, 'action_type') and getattr(a, 'action_type', '') == 'play_card':
            return a
    return actions[0]


# ---------------------------------------------------------------------------
# Worker function — runs in subprocess via mp.Pool
# ---------------------------------------------------------------------------

def _play_one_game(
    seed: str,
    ascension: int,
    temperature: float,
    total_games: int = 0,
    turn_solver_ms: float = 50.0,
    strategic_search: bool = False,
) -> Dict[str, Any]:
    """Play a single game and return transitions + result.

    This function runs in a worker process. Workers are torch-free: all
    neural-network inference is delegated to the InferenceServer running
    in the main process via InferenceClient. If the server is unavailable
    (client is None or request times out), the worker falls back to first
    legal action for strategic decisions.

    Pure on-policy: model makes decision, log_prob from unscaled policy.
    No epsilon-greedy, no heuristic override.

    Returns a dict with:
        seed, won, floor, hp, decisions, duration_s,
        transitions: list of dicts with (obs, action_mask, action, reward,
                     done, value, log_prob, final_floor, cleared_act1/2/3)
    """
    from packages.engine.game import GameRunner, GamePhase, CombatAction
    from packages.training.state_encoders import RunStateEncoder
    from packages.training.inference_server import get_client
    from packages.training.turn_solver import TurnSolverAdapter

    encoder = RunStateEncoder()
    # Scale node budget proportionally with time budget (100 nodes per ms)
    _node_budget = max(1000, int(turn_solver_ms * 100))
    turn_solver = TurnSolverAdapter(
        time_budget_ms=turn_solver_ms,
        node_budget=_node_budget,
        # Multi-turn solver for boss/elite: 3 turns ahead, 4 candidate plans, 5s budget
        multi_turn_depth=3,
        multi_turn_k=4,
        multi_turn_budget_ms=5000.0,
    )

    client = get_client()

    # Worker status file for live dashboard grid
    import os
    _worker_id = os.getpid()
    _wname = globals().get("_worker_name", f"W{_worker_id}")
    _status_dir = Path("logs/active/workers")
    _status_dir.mkdir(parents=True, exist_ok=True)
    _status_file = _status_dir / f"{_wname}.json"
    _last_status_floor = -1

    def _write_worker_status(floor, phase_str, hp, max_hp, seed_str, in_combat=False, enemy=""):
        nonlocal _last_status_floor
        if floor == _last_status_floor and not in_combat:
            return  # Only update on floor change or combat events
        _last_status_floor = floor
        try:
            _status_file.write_text(json.dumps({
                "name": _wname, "id": _worker_id, "seed": seed_str, "floor": floor,
                "phase": phase_str, "hp": hp, "max_hp": max_hp,
                "enemy": enemy, "ts": round(time.monotonic(), 1),
            }))
        except Exception:
            pass

    try:
        runner = GameRunner(seed=seed, ascension=ascension, character="Watcher", verbose=False)
    except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
        logger.warning("_play_one_game: GameRunner init failed for seed=%s: %s", seed, e)
        return {
            "seed": seed, "won": False, "floor": 0, "hp": 0,
            "decisions": 0, "duration_s": 0.0, "transitions": [],
        }

    t0 = time.monotonic()
    step = 0
    prev_floor = 0
    reached_milestones: set = set()
    prev_potential = compute_potential(runner.run_state)
    decisions = 0
    transitions: List[Dict[str, Any]] = []

    # Track combat events for event-based rewards
    was_in_combat = False
    combat_room_type = "monster"
    prev_deck_size = len(getattr(runner.run_state, "deck", []))

    # Per-combat stats tracking
    combats: List[Dict[str, Any]] = []
    combat_start_hp = 0
    combat_cards_played = 0
    combat_turns = 0
    combat_start_time = 0.0
    combat_potions_used = 0
    event_reward_potion_use = 0.0
    combat_stance_changes = 0
    prev_stance = "Neutral"
    # Per-turn card tracking
    turn_cards: List[str] = []  # Cards played this turn
    turns_log: List[Dict[str, Any]] = []  # Per-turn log for current combat
    # Solver telemetry per combat
    combat_solver_ms = 0.0  # Total solver time this combat
    combat_solver_calls = 0  # Number of solver calls this combat

    # Event and path tracking for episode logging
    events_visited: List[Dict[str, Any]] = []  # {floor, event_id}
    path_choices_log: List[Dict[str, Any]] = []  # {floor, chosen, options}

    # Helper: record combat summary when transitioning out of combat.
    # Captures enemy names from combat state before it's cleared.
    def _finish_combat_summary():
        nonlocal was_in_combat
        if not was_in_combat:
            return
        was_in_combat = False
        # Include final turn's cards if any
        if turn_cards:
            turns_log.append({"turn": combat_turns + 1, "cards": turn_cards[:]})
        # Capture encounter name from combat enemies before state is cleared
        _enc_name = ""
        _combat_ref = getattr(runner, "current_combat", None)
        if _combat_ref is not None:
            try:
                _enemy_ids = [getattr(e, "id", getattr(e, "name", "?"))
                              for e in _combat_ref.state.enemies]
                _enc_name = ", ".join(_enemy_ids)
            except Exception:
                pass
        # Use runner.run_state directly (not captured `rs` which may be stale).
        # On death the engine now syncs HP to 0, but clamp with max(0, _) anyway.
        _post_hp = max(0, getattr(runner.run_state, "current_hp", 0))
        combats.append({
            "floor": current_floor,
            "room_type": combat_room_type,
            "encounter_name": _enc_name,
            "hp_lost": max(0, combat_start_hp - _post_hp),
            "cards_played": combat_cards_played,
            "turns": combat_turns,
            "potions_used": combat_potions_used,
            "stance_changes": combat_stance_changes,
            "turns_detail": turns_log[:],
            "duration_ms": round((time.monotonic() - combat_start_time) * 1000),
            "solver_ms": round(combat_solver_ms),
            "solver_calls": combat_solver_calls,
        })

    while not runner.game_over and step < 5000:
        try:
            actions = runner.get_available_actions()
        except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
            logger.warning("_play_one_game: get_available_actions failed at floor=%d: %s", prev_floor, e)
            break
        if not actions:
            break

        phase = runner.phase
        rs = runner.run_state
        current_floor = getattr(rs, "floor", 0)

        # Update worker status on floor change
        if current_floor != _last_status_floor:
            _write_worker_status(
                current_floor, phase.name if hasattr(phase, 'name') else str(phase),
                getattr(rs, "current_hp", 0), getattr(rs, "max_hp", 80), seed,
            )

        if phase == GamePhase.COMBAT:
            if not was_in_combat:
                # Combat just started — reset solver cache, record baseline
                turn_solver.reset()
                combat_start_hp = getattr(rs, "current_hp", 0)
                combat_cards_played = 0
                combat_turns = 0
                combat_start_time = time.monotonic()
                combat_potions_used = 0
                event_reward_potion_use = 0.0
                combat_stance_changes = 0
                prev_stance = getattr(getattr(rs, "combat", None), "stance", "Neutral") if hasattr(rs, "combat") else "Neutral"
                turn_cards = []
                turns_log = []
                combat_solver_ms = 0.0
                combat_solver_calls = 0
            was_in_combat = True
            combat_room_type = getattr(runner, "current_room_type", "monster")
            # Track card plays, potion uses, stance changes
            _solve_t0 = time.monotonic()
            action = _pick_combat_action(actions, runner, turn_solver)
            combat_solver_ms += (time.monotonic() - _solve_t0) * 1000
            combat_solver_calls += 1
            if hasattr(action, "action_type"):
                atype = getattr(action, "action_type", "")
                if atype == "play_card":
                    combat_cards_played += 1
                    # Look up actual card name from hand using card_idx
                    _cidx = getattr(action, "card_idx", -1)
                    _combat = getattr(runner, "current_combat", None)
                    _hand = getattr(getattr(_combat, "state", None), "hand", None) if _combat else None
                    card_id = _hand[_cidx] if _hand and 0 <= _cidx < len(_hand) else "?"
                    turn_cards.append(card_id)
                elif atype == "use_potion":
                    combat_potions_used += 1
                    # Reward potion use during elite/boss fights
                    _rt = combat_room_type.lower() if isinstance(combat_room_type, str) else "monster"
                    if _rt in ("elite", "e"):
                        event_reward_potion_use += REWARD_WEIGHTS["potion_use_elite"]
                    elif _rt in ("boss", "b"):
                        event_reward_potion_use += REWARD_WEIGHTS["potion_use_boss"]
                    turn_cards.append(f"potion:{getattr(action, 'potion_idx', '?')}")
                elif atype == "end_turn":
                    combat_turns += 1
                    # Log hand state, energy, player/enemy state at turn end
                    _combat_ref = getattr(runner, "current_combat", None)
                    _turn_info = {"turn": combat_turns, "cards": turn_cards[:]}
                    if _combat_ref:
                        _st = _combat_ref.state
                        _hand_ids = list(_st.hand) if hasattr(_st, "hand") else []
                        _turn_info["hand_at_end"] = _hand_ids[:10]
                        _turn_info["energy_left"] = getattr(_st, "energy", 0)
                        # Player state for replay
                        _turn_info["player_hp"] = getattr(_st, "hp", 0)
                        _turn_info["player_block"] = getattr(_st, "block", 0)
                        _turn_info["stance"] = getattr(_st, "stance", "Neutral")
                        # Enemy state for replay
                        _enemies = getattr(_st, "enemies", [])
                        _turn_info["enemies"] = [
                            {"name": getattr(e, "name", "?"), "hp": getattr(e, "hp", 0),
                             "max_hp": getattr(e, "max_hp", 0), "block": getattr(e, "block", 0),
                             "intent": getattr(e, "intent", {}).get("type", "?") if isinstance(getattr(e, "intent", None), dict) else str(getattr(e, "intent", "?"))}
                            for e in _enemies if getattr(e, "hp", 0) > 0
                        ]
                        # Count playable cards (cost <= energy)
                        _costs = getattr(_st, "card_costs", {})
                        _energy = getattr(_st, "energy", 0)
                        _playable = sum(1 for c in _hand_ids if _costs.get(c, 1) <= _energy)
                        _turn_info["playable_unplayed"] = _playable
                    turns_log.append(_turn_info)
                    turn_cards.clear()
            runner.take_action(action)
            # Detect stance changes after action
            combat_state = getattr(runner, "current_combat", None)
            if combat_state is not None:
                cur_stance = getattr(combat_state.state, "stance", "Neutral")
                if cur_stance != prev_stance:
                    combat_stance_changes += 1
                    prev_stance = cur_stance
            # Detect phase change FROM combat after action execution.
            phase_after = runner.phase
            if was_in_combat and phase_after != GamePhase.COMBAT:
                _finish_combat_summary()
        elif len(actions) == 1:
            # Check for combat-end transition
            if was_in_combat and phase != GamePhase.COMBAT:
                _finish_combat_summary()
            runner.take_action(actions[0])
        else:
            # Strategic decision point
            decisions += 1

            # Check if combat just ended (for event rewards)
            combat_just_ended = was_in_combat and phase != GamePhase.COMBAT
            if combat_just_ended:
                _finish_combat_summary()

            # Track event encounters
            if phase == GamePhase.EVENT:
                _evt_state = getattr(runner, "current_event_state", None)
                _evt_id = getattr(_evt_state, "event_id", None) if _evt_state else None
                if _evt_id:
                    events_visited.append({"floor": current_floor, "event_id": _evt_id})

            # Track path choices (what options were available and what was chosen)
            if phase == GamePhase.MAP_NAVIGATION:
                try:
                    _avail_paths = runner.run_state.get_available_paths()
                    _path_options = [
                        {"x": n.x, "y": n.y,
                         "room_type": n.room_type.name if n.room_type else None}
                        for n in _avail_paths
                    ]
                    if _path_options:
                        path_choices_log.append({
                            "floor": current_floor,
                            "options": _path_options,
                        })
                except Exception:
                    pass

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

            # Encode state with phase context + boss/room info + available actions
            _boss = getattr(runner, "_boss_name", "")
            _room = getattr(runner, "current_room_type", "")
            run_obs = encoder.encode(
                rs, phase_type=phase_type, boss_name=_boss, room_type=_room,
                actions=actions, runner=runner,
            )

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

                    # log_prob from UNSCALED policy (matches trainer's forward pass).
                    logits_base = logits_np - logits_np.max()
                    probs_base = np.exp(logits_base)
                    probs_base /= probs_base.sum()
                    log_prob = float(np.log(probs_base[action_idx] + 1e-8))

                    # Clamp to valid range
                    action_idx = min(action_idx, n_actions - 1)

                    # Strategic search: value-head per-option evaluation
                    if strategic_search and n_actions > 1 and client is not None:
                        search_result = _strategic_search(actions, runner, encoder, client, phase_type, n_actions)
                        if search_result is not None:
                            best_idx, search_policy = search_result
                            # Blend 70% search / 30% model policy
                            model_probs = np.exp(logits_np[:n_actions] - logits_np[:n_actions].max())
                            model_probs /= model_probs.sum()
                            blended = 0.7 * search_policy + 0.3 * model_probs
                            blended /= blended.sum()
                            action_idx = int(np.random.choice(n_actions, p=blended))
                else:
                    logits_np = None

            if logits_np is not None:
                # --- PBRS reward ---
                # Take action first, then compute Phi(s') - gamma * Phi(s)
                runner.take_action(actions[action_idx])

                # Record which path was chosen (after action so index is valid)
                if phase == GamePhase.MAP_NAVIGATION and path_choices_log:
                    _last_pc = path_choices_log[-1]
                    if _last_pc.get("floor") == current_floor and "chosen" not in _last_pc:
                        _last_pc["chosen"] = action_idx

                new_rs = runner.run_state
                new_potential = compute_potential(new_rs)

                # PBRS: gamma * Phi(s') - Phi(s) preserves optimal policy
                gamma = 0.99
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

                    # Penalize HP lost in combat (reduced: was dominating rewards)
                    hp_lost = max(0, combat_start_hp - getattr(new_rs, "current_hp", 0))
                    event_reward += REWARD_WEIGHTS["damage_per_hp"] * hp_lost

                    # Penalize wasteful potion use (used potion but still lost lots of HP)
                    if combat_potions_used > 0 and hp_pct < 0.5:
                        event_reward += REWARD_WEIGHTS["potion_waste_penalty"] * combat_potions_used

                    # Reward potion use in elite/boss fights that were won
                    if combat_potions_used > 0 and rt in ("elite", "e", "boss", "b"):
                        event_reward += REWARD_WEIGHTS["potion_kill_same_fight"]
                        event_reward += event_reward_potion_use

                    # Penalize hoarding potions in tough fights when low HP
                    if hp_pct < 0.3 and combat_potions_used == 0 and rt in ("elite", "e", "boss", "b"):
                        _potions = getattr(new_rs, "potions", [])
                        _has_potions = any(p is not None for p in _potions) if _potions else False
                        if _has_potions:
                            event_reward += REWARD_WEIGHTS.get("potion_hoard_penalty", -0.30)

                # Card removal reward (deck thinning is critical for Watcher)
                new_deck_size = len(getattr(new_rs, "deck", []))
                if new_deck_size < prev_deck_size:
                    event_reward += REWARD_WEIGHTS["shop_remove"] * (prev_deck_size - new_deck_size)

                # Upgrade detection (deck size unchanged, card upgraded)
                elif new_deck_size == prev_deck_size and UPGRADE_REWARDS:
                    # Check if an upgrade just happened (rest site or smith event)
                    if phase_type == "rest":
                        new_deck = list(getattr(new_rs, "deck", []))
                        for card in new_deck:
                            cid = getattr(card, "id", str(card))
                            if getattr(card, "upgraded", False) and cid in UPGRADE_REWARDS:
                                event_reward += UPGRADE_REWARDS[cid]
                                break

                prev_deck_size = new_deck_size

                # Floor milestone rewards (one-time per game)
                new_floor = getattr(new_rs, "floor", 0)
                for milestone_floor, milestone_reward in FLOOR_MILESTONES.items():
                    if new_floor >= milestone_floor and milestone_floor not in reached_milestones:
                        # F16 HP bonus: reward arriving at boss floor healthy
                        if milestone_floor == 16:
                            _cur_hp = getattr(new_rs, "current_hp", 0)
                            milestone_reward = (
                                REWARD_WEIGHTS.get("f16_hp_bonus_base", 0.50)
                                + REWARD_WEIGHTS.get("f16_hp_bonus_per_hp", 0.02) * _cur_hp
                            )
                        event_reward += milestone_reward
                        reached_milestones.add(milestone_floor)

                reward = pbrs_reward + event_reward
                prev_potential = new_potential

                # Record transition as numpy-serializable dict
                transitions.append({
                    "obs": run_obs,
                    "action_mask": mask,
                    "action": action_idx,
                    "reward": reward,
                    "pbrs": pbrs_reward,
                    "event_reward": event_reward,
                    "done": False,
                    "value": value,
                    "log_prob": log_prob,
                    "final_floor": 0.0,
                    "cleared_act1": 0.0,
                    "cleared_act2": 0.0,
                    "cleared_act3": 0.0,
                })

            else:
                # No inference server available: take first legal action
                runner.take_action(actions[0])
                # Record path choice for fallback path
                if phase == GamePhase.MAP_NAVIGATION and path_choices_log:
                    _last_pc = path_choices_log[-1]
                    if _last_pc.get("floor") == current_floor and "chosen" not in _last_pc:
                        _last_pc["chosen"] = 0
                prev_potential = compute_potential(runner.run_state)

        step += 1
        prev_floor = current_floor

    # Record final combat if we died in combat (loop exits before phase changes)
    if was_in_combat:
        _finish_combat_summary()

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
                transitions[-1]["reward"] += REWARD_WEIGHTS.get("win_reward", 10.0)
            else:
                progress = final_floor / 55.0
                death_scale = REWARD_WEIGHTS.get("death_penalty_scale", -1.0)
                transitions[-1]["reward"] += death_scale * (1 - progress)
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
        death_enemies = getattr(runner, "last_death_enemies", [])
        if death_enemies:
            death_enemy = ", ".join(death_enemies)
    except Exception:
        pass

    # Capture relics for episode logging
    relics_final = []
    try:
        relics_final = [
            getattr(r, "id", str(r)) for r in getattr(rs, "relics", [])
        ]
    except Exception:
        pass

    # Clear worker status file (game done)
    try:
        _status_file.unlink(missing_ok=True)
    except Exception:
        pass

    # Aggregate solver stats from combats
    total_solver_ms = sum(c.get("solver_ms", 0) for c in combats)
    total_solver_calls = sum(c.get("solver_calls", 0) for c in combats)

    return {
        "seed": seed,
        "won": won,
        "floor": final_floor,
        "hp": final_hp,
        "max_hp": getattr(rs, "max_hp", 0),
        "decisions": decisions,
        "duration_s": round(duration, 2),
        "solver_ms": round(total_solver_ms),
        "solver_calls": total_solver_calls,
        "transitions": transitions,
        "deck_final": deck_final,
        "relics_final": relics_final,
        "death_enemy": death_enemy,
        "room_type": getattr(runner, "current_room_type", ""),
        "combats": combats,
        "events": events_visited,
        "path_choices": path_choices_log,
    }
