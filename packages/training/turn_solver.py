"""
Turn Solver: Adaptive hybrid search for optimal card play sequences.

Finds the best action sequence for the current combat turn by searching
over ALL action types (cards, potions, scry selections) using the engine's
own copy() + execute_action() for guaranteed correctness.

Adaptive strategy selection:
- Small turns (<300 estimated nodes): Exact DFS with alpha-beta pruning
- Medium turns (300-5000): Best-first search with pruning
- Large turns (5000+): Beam search with reserved setup slots

Lexicographic scoring (not weighted sums):
- Priority 1: Death = -inf, Lethal = +inf
- Priority 2: Minimize expected HP loss
- Priority 3: Enemy HP destroyed
- Priority 4: Setup value (powers, stance, mantra)
"""

from __future__ import annotations

import logging
import time
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple
import heapq

logger = logging.getLogger(__name__)

from packages.engine.combat_engine import CombatEngine, CombatPhase
from packages.engine.content.cards import ALL_CARDS, CardType, resolve_card_id
from packages.engine.state.combat import (
    Action,
    EndTurn,
    PlayCard,
    SelectScryDiscard,
    UsePotion,
)


# ---------------------------------------------------------------------------
# Scoring constants — minimal, simulation-grounded
# ---------------------------------------------------------------------------

_SCORE_DEATH = -1_000_000.0
_SCORE_LETHAL = 1_000_000.0

# Cards that do setup (stance, draw, powers) — get reserved beam slots
_SETUP_CARD_PREFIXES = frozenset({
    "Eruption", "Vigilance", "Crescendo", "Tranquility",
    "InnerPeace", "EmptyBody", "EmptyFist", "EmptyMind",
    "Prostrate", "Worship", "Pray", "Devotion",
    "Tantrum", "MentalFortress", "TalkToTheHand",
    "ThirdEye", "CutThroughFate", "Scrawl",
    "Rushdown", "StudyCard", "Foresight",
    "Miracle", "Vault",
})

# Stance-entering cards: card_id -> stance (used for action priority)
_STANCE_CARDS: Dict[str, str] = {
    "Eruption": "Wrath", "Crescendo": "Wrath", "Tantrum": "Wrath",
    "Vigilance": "Calm", "ClearTheMind": "Calm",  # Tranquility alias
    "EmptyBody": "Neutral", "EmptyFist": "Neutral", "EmptyMind": "Neutral",
    "InnerPeace": "Calm",
}

# Early cutoff: skip nodes this far below the current best
_EARLY_CUTOFF_GAP = 200.0

# Solver budgets are passed per-call from worker.py (which reads from training_config).
# No hardcoded budgets here — solver uses self.default_time_budget_ms and self.default_node_budget.

# LRU cache size for turn-level plan caching
_PLAN_CACHE_SIZE = 512


# ---------------------------------------------------------------------------
# Search node
# ---------------------------------------------------------------------------

@dataclass
class SearchNode:
    """A node in the turn search tree."""
    engine: CombatEngine
    actions: List[Action]  # Actions taken to reach this node
    score: float = 0.0

    def __lt__(self, other: SearchNode) -> bool:
        """For heap ordering — higher score = better (negate for min-heap)."""
        return self.score > other.score


# ---------------------------------------------------------------------------
# TurnSolver
# ---------------------------------------------------------------------------

class TurnSolver:
    """Find optimal card play sequence for the current turn.

    Hybrid approach:
    - Small turns (<300 estimated nodes): Exact DFS with alpha-beta pruning
    - Medium turns (300-5000): Best-first search with pruning
    - Large turns (5000+): Beam search with reserved setup slots

    Uses CombatEngine.copy() + execute_action() for simulation (guaranteed correct).
    Searches over ALL actions (cards + potions + scry), not just cards.
    """

    def __init__(
        self,
        time_budget_ms: float = 5.0,
        node_budget: int = 1000,
        neural_eval=None,  # Optional: (engine) -> float, for neural leaf blending
    ):
        self.default_time_budget_ms = time_budget_ms
        self.default_node_budget = node_budget
        self._neural_eval = neural_eval

        # Tree reuse state
        self._cached_plan: Optional[List[Action]] = None
        self._cached_plan_index: int = 0
        self._cached_engine_hash: Optional[int] = None

        # State-based plan cache: hash(board_state) -> (plan, score)
        # Reuses plans when the same hand+energy+enemy state recurs
        from collections import OrderedDict
        self._plan_cache: OrderedDict[int, Tuple[List[Action], float]] = OrderedDict()

    # -----------------------------------------------------------------------
    # Public API
    # -----------------------------------------------------------------------

    def solve_turn(
        self,
        engine: CombatEngine,
        room_type: str = "monster",
    ) -> Optional[List[Action]]:
        """Find best action sequence for this turn.

        Returns list of actions to execute (may include potions, cards, end_turn).
        Returns None if search fails or there are no meaningful choices.
        """
        if engine.phase != CombatPhase.PLAYER_TURN or engine.state.combat_over:
            return None

        # Budget comes from worker (via training_config.SOLVER_BUDGETS, scaled by enemy HP)
        time_ms = self.default_time_budget_ms
        node_limit = self.default_node_budget

        actions = engine.get_legal_actions()
        if not actions:
            return None

        # Only EndTurn available — nothing to search
        non_end = [a for a in actions if not isinstance(a, EndTurn)]
        if not non_end:
            return [EndTurn()]

        # Check plan cache — reuse if we've seen this exact board state
        state_key = self._state_hash(engine)
        if state_key in self._plan_cache:
            cached_plan, _ = self._plan_cache[state_key]
            # Move to end (LRU)
            self._plan_cache.move_to_end(state_key)
            self._cached_plan = cached_plan
            self._cached_plan_index = 0
            self._cached_engine_hash = state_key
            return cached_plan

        # Estimate tree size for strategy selection
        estimated_nodes = self._estimate_tree_size(engine)

        if estimated_nodes < 300:
            plan = self._exact_dfs(engine, time_ms, node_limit)
        elif estimated_nodes < 5000:
            plan = self._best_first_search(engine, time_ms, node_limit)
        else:
            plan = self._beam_search(engine, time_ms, node_limit, room_type=room_type)

        if plan is not None:
            # Cache for tree reuse
            self._cached_plan = plan
            self._cached_plan_index = 0
            self._cached_engine_hash = state_key

            # Store in plan cache (LRU eviction)
            self._plan_cache[state_key] = (plan, 0.0)
            if len(self._plan_cache) > _PLAN_CACHE_SIZE:
                self._plan_cache.popitem(last=False)

        return plan

    def get_next_action(
        self,
        engine: CombatEngine,
        room_type: str = "monster",
    ) -> Optional[Action]:
        """Get the next single action to take.

        Uses cached plan if available and still valid. Replans on divergence.
        """
        if engine.phase != CombatPhase.PLAYER_TURN or engine.state.combat_over:
            return None

        # Try cached plan
        if self._cached_plan is not None and self._cached_plan_index < len(self._cached_plan):
            next_action = self._cached_plan[self._cached_plan_index]

            # Validate the action is still legal
            legal = engine.get_legal_actions()
            if self._action_in_list(next_action, legal):
                self._cached_plan_index += 1
                return next_action

            # Divergence — invalidate cache and replan
            self._invalidate_cache()

        # No cache or cache invalid — solve from scratch
        plan = self.solve_turn(engine, room_type)
        if plan and len(plan) > 0:
            self._cached_plan_index = 1
            return plan[0]

        return None

    # -----------------------------------------------------------------------
    # Tree size estimation
    # -----------------------------------------------------------------------

    def _estimate_tree_size(self, engine: CombatEngine) -> int:
        """Estimate total search tree nodes for strategy selection.

        Uses branching_factor ^ estimated_depth as a rough upper bound.
        """
        actions = engine.get_legal_actions()
        non_end = [a for a in actions if not isinstance(a, EndTurn)]
        branching = len(non_end)

        if branching == 0:
            return 1

        # Estimate depth: roughly energy / average_cost, capped
        energy = engine.state.energy
        hand_size = len(engine.state.hand)
        # Most cards cost 1, so depth ~ min(energy, hand_size)
        estimated_depth = min(energy, hand_size, 8)

        # Branching decreases as cards are played (hand shrinks)
        # Use geometric mean approximation
        total = 0
        b = branching
        for _ in range(estimated_depth):
            total += b
            b = max(1, b - 1)  # Hand shrinks by ~1 per play

        return max(total, 1)

    # -----------------------------------------------------------------------
    # Scoring
    # -----------------------------------------------------------------------

    def _score_terminal(
        self,
        engine: CombatEngine,
        original: CombatEngine,
    ) -> float:
        """Score a terminal turn state by SIMULATING the enemy turn.

        No heuristic weights — we copy the engine, execute EndTurn, and
        measure actual HP lost after enemy attacks. This is ground truth,
        not estimation.

        Scoring:
        1. Dead (now or after enemy turn) -> -1M
        2. All enemies dead (lethal) -> +1M + hp_remaining
        3. Otherwise: linear combination of simulated outcomes
        """
        state = engine.state
        player = state.player

        if player.hp <= 0:
            return _SCORE_DEATH

        living = [e for e in state.enemies if e.hp > 0 and not e.is_escaping]

        if not living:
            return _SCORE_LETHAL + player.hp * 200

        # Simulate enemy turn to get actual HP loss.
        # CombatEngine.end_turn() is monolithic: runs full enemy turn
        # synchronously and returns with phase == PLAYER_TURN.
        # Just read sim.state.player.hp after the call.
        try:
            sim = engine.copy()
            sim.execute_action(EndTurn())
            hp_after = sim.state.player.hp
        except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
            logger.warning("TurnSolver: enemy turn simulation failed: %s", e)
            # Fallback: estimate from intents
            hp_after = player.hp - self._estimate_incoming(state)

        actual_hp_lost = max(0, player.hp - hp_after)

        if hp_after <= 0:
            return _SCORE_DEATH + player.hp  # Dead after enemy turn

        # Enemies killed this turn
        orig_living = sum(1 for e in original.state.enemies if e.hp > 0 and not e.is_escaping)
        now_living = sum(1 for e in living)
        enemies_killed = orig_living - now_living

        # Enemy HP destroyed
        orig_ehp = sum(e.hp for e in original.state.enemies if e.hp > 0 and not e.is_escaping)
        now_ehp = sum(e.hp for e in living)

        # Rough turns-to-kill estimate
        dmg_this_turn = orig_ehp - now_ehp
        est_turns = now_ehp / max(dmg_this_turn, 1) if dmg_this_turn > 0 else 10

        # Score: all weights from training_config.SOLVER_SCORING
        from .training_config import SOLVER_SCORING as W
        score = (
            W["hp_lost_weight"] * actual_hp_lost
            + W["enemy_kill_bonus"] * enemies_killed
            + W["remaining_hp_weight"] * now_ehp / max(orig_ehp, 1)
            + W["turns_to_kill_weight"] * min(est_turns, 10)
        )

        # Stance: Wrath doubles damage dealt AND taken.
        # Penalty scales with incoming damage, not a flat number.
        incoming = TurnSolver._estimate_incoming(state)
        if state.stance == "Calm":
            score += W["calm_bonus"]
        elif state.stance == "Wrath" and now_living > 0:
            score -= min(incoming * W["wrath_incoming_scale"], W["wrath_cap"])

        # Penalize unspent energy with playable cards (unless Ice Cream relic)
        unspent = state.energy
        if unspent > 0:
            has_ice_cream = any(
                (getattr(r, 'relic_id', None) or getattr(r, 'id', None) or str(r)) == "IceCream"
                for r in getattr(state, 'relics', [])
            )
            if not has_ice_cream:
                costs = state.card_costs
                playable = sum(1 for card_id in state.hand
                               if costs.get(card_id, 1) <= unspent)
                if playable > 0:
                    score += W["unspent_energy_weight"] * unspent + W["unspent_playable_weight"] * playable
                else:
                    score += W["unspent_idle_weight"] * unspent

        # Neural leaf evaluation: blend with heuristic for robustness
        if self._neural_eval is not None:
            try:
                neural_score = self._neural_eval(engine)
                # Blend: 70% neural, 30% heuristic
                # Scale neural output (0-1 probability) to heuristic score range
                score = 0.7 * neural_score * 100.0 + 0.3 * score
            except Exception:
                pass  # Fall back to pure heuristic

        return score

    @staticmethod
    def _estimate_incoming(state) -> int:
        """Fallback: estimate incoming damage from enemy intents."""
        total = 0
        for e in state.enemies:
            if e.hp > 0 and not e.is_escaping and e.move_damage > 0:
                raw = e.move_damage * e.move_hits
                if state.stance == "Wrath":
                    raw *= 2
                total += raw
        return max(0, total - state.player.block)

    # -----------------------------------------------------------------------
    # Strategy 1: Exact DFS (small turns, <300 nodes)
    # -----------------------------------------------------------------------

    def _exact_dfs(
        self,
        engine: CombatEngine,
        time_budget_ms: float,
        node_budget: int,
    ) -> Optional[List[Action]]:
        """Exact DFS with alpha-beta style pruning for small turn trees."""
        deadline = time.monotonic() + time_budget_ms / 1000.0
        original = engine

        best_score = _SCORE_DEATH - 1
        best_plan: Optional[List[Action]] = None
        nodes_visited = 0

        def dfs(eng: CombatEngine, path: List[Action], alpha: float) -> None:
            nonlocal best_score, best_plan, nodes_visited

            nodes_visited += 1
            if nodes_visited > node_budget:
                return
            if time.monotonic() > deadline:
                return

            actions = eng.get_legal_actions()
            if not actions:
                return

            # Score EndTurn as a terminal
            end_turn_score = self._score_terminal(eng, original)
            if end_turn_score > best_score:
                best_score = end_turn_score
                best_plan = path + [EndTurn()]

            # Alpha-beta: if we already found lethal, skip
            if best_score >= _SCORE_LETHAL:
                return

            # Try each non-EndTurn action, sorted by heuristic priority
            sorted_non_end = self._sorted_actions(actions, eng)
            for action in sorted_non_end:
                if nodes_visited > node_budget or time.monotonic() > deadline:
                    return

                child = eng.copy()
                try:
                    child.execute_action(action)
                except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                    logger.warning("TurnSolver: DFS execute_action failed: %s", e)
                    continue

                # If combat is over after this action, score it
                if child.state.combat_over:
                    score = self._score_terminal(child, original)
                    if score > best_score:
                        best_score = score
                        best_plan = path + [action]
                    continue

                # If still player's turn, recurse
                if child.phase == CombatPhase.PLAYER_TURN:
                    dfs(child, path + [action], best_score)
                else:
                    # Turn ended (shouldn't happen for non-EndTurn, but handle it)
                    score = self._score_terminal(child, original)
                    if score > best_score:
                        best_score = score
                        best_plan = path + [action]

        dfs(engine, [], _SCORE_DEATH - 1)
        return best_plan

    # -----------------------------------------------------------------------
    # Strategy 2: Best-first search (medium turns, 300-5000 nodes)
    # -----------------------------------------------------------------------

    def _best_first_search(
        self,
        engine: CombatEngine,
        time_budget_ms: float,
        node_budget: int,
    ) -> Optional[List[Action]]:
        """Best-first search with priority queue for medium turn trees."""
        deadline = time.monotonic() + time_budget_ms / 1000.0
        original = engine
        nodes_visited = 0

        best_score = _SCORE_DEATH - 1
        best_plan: Optional[List[Action]] = None

        # Priority queue: (negative_score, counter, SearchNode)
        # Using counter for stable heap ordering
        counter = 0
        heap: list = []

        # Seed with root
        root_score = self._score_terminal(engine, original)
        heapq.heappush(heap, (-root_score, counter, SearchNode(engine, [], root_score)))
        counter += 1

        while heap and nodes_visited < node_budget:
            if time.monotonic() > deadline:
                break

            _, _, node = heapq.heappop(heap)
            nodes_visited += 1

            # Score EndTurn from this state
            end_score = self._score_terminal(node.engine, original)
            if end_score > best_score:
                best_score = end_score
                best_plan = node.actions + [EndTurn()]

            if best_score >= _SCORE_LETHAL:
                break

            # Early cutoff: skip nodes far worse than current best
            if best_score > _SCORE_DEATH and end_score < best_score - _EARLY_CUTOFF_GAP:
                continue

            actions = node.engine.get_legal_actions()
            sorted_non_end = self._sorted_actions(actions, node.engine)

            for action in sorted_non_end:
                if nodes_visited >= node_budget or time.monotonic() > deadline:
                    break

                child_engine = node.engine.copy()
                try:
                    child_engine.execute_action(action)
                except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                    logger.warning("TurnSolver: best-first execute_action failed: %s", e)
                    continue

                nodes_visited += 1
                child_actions = node.actions + [action]

                if child_engine.state.combat_over:
                    score = self._score_terminal(child_engine, original)
                    if score > best_score:
                        best_score = score
                        best_plan = child_actions
                    continue

                if child_engine.phase == CombatPhase.PLAYER_TURN:
                    score = self._score_terminal(child_engine, original)
                    heapq.heappush(
                        heap,
                        (-score, counter, SearchNode(child_engine, child_actions, score)),
                    )
                    counter += 1
                else:
                    score = self._score_terminal(child_engine, original)
                    if score > best_score:
                        best_score = score
                        best_plan = child_actions

        return best_plan

    # -----------------------------------------------------------------------
    # Strategy 3: Beam search (large turns, 5000+ nodes)
    # -----------------------------------------------------------------------

    def _beam_search(
        self,
        engine: CombatEngine,
        time_budget_ms: float,
        node_budget: int,
        beam_width: int = 20,
        reserved_setup_slots: int = 4,
        room_type: str = "monster",
    ) -> Optional[List[Action]]:
        """Beam search with reserved slots for setup cards.

        Keeps `beam_width` candidates per depth level.
        Reserves `reserved_setup_slots` beam positions for setup actions
        (stance changes, powers, draw cards) so they aren't pruned early.

        Beam width scales with fight importance:
        - boss: 30, elite: 25, monster: 20
        """
        # Dynamic beam width based on room type
        if room_type == "boss":
            beam_width = 30
        elif room_type == "elite":
            beam_width = 25
        deadline = time.monotonic() + time_budget_ms / 1000.0
        original = engine
        nodes_visited = 0

        best_score = _SCORE_DEATH - 1
        best_plan: Optional[List[Action]] = None

        # Current beam: list of SearchNode
        beam: List[SearchNode] = [SearchNode(engine, [], 0.0)]

        max_depth = min(engine.state.energy + 2, len(engine.state.hand) + 2, 12)

        for _depth in range(max_depth):
            if time.monotonic() > deadline or nodes_visited >= node_budget:
                break

            candidates: List[SearchNode] = []
            setup_candidates: List[SearchNode] = []

            for node in beam:
                if time.monotonic() > deadline or nodes_visited >= node_budget:
                    break

                # Score EndTurn at this depth
                end_score = self._score_terminal(node.engine, original)
                if end_score > best_score:
                    best_score = end_score
                    best_plan = node.actions + [EndTurn()]

                if best_score >= _SCORE_LETHAL:
                    return best_plan

                # Early cutoff: skip nodes far worse than current best
                if best_score > _SCORE_DEATH and end_score < best_score - _EARLY_CUTOFF_GAP:
                    continue

                actions = node.engine.get_legal_actions()
                sorted_non_end = self._sorted_actions(actions, node.engine)

                for action in sorted_non_end:
                    if nodes_visited >= node_budget or time.monotonic() > deadline:
                        break

                    child_engine = node.engine.copy()
                    try:
                        child_engine.execute_action(action)
                    except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                        logger.warning("TurnSolver: beam search execute_action failed: %s", e)
                        continue

                    nodes_visited += 1
                    child_actions = node.actions + [action]

                    if child_engine.state.combat_over:
                        score = self._score_terminal(child_engine, original)
                        if score > best_score:
                            best_score = score
                            best_plan = child_actions
                        continue

                    if child_engine.phase != CombatPhase.PLAYER_TURN:
                        score = self._score_terminal(child_engine, original)
                        if score > best_score:
                            best_score = score
                            best_plan = child_actions
                        continue

                    score = self._score_terminal(child_engine, original)
                    child_node = SearchNode(child_engine, child_actions, score)

                    # Classify: setup vs normal
                    if self._is_setup_action(action, node.engine):
                        setup_candidates.append(child_node)
                    else:
                        candidates.append(child_node)

            if not candidates and not setup_candidates:
                break

            # Sort by score descending
            candidates.sort(key=lambda n: n.score, reverse=True)
            setup_candidates.sort(key=lambda n: n.score, reverse=True)

            # Reserve slots for setup actions
            reserved = setup_candidates[:reserved_setup_slots]
            remaining_width = beam_width - len(reserved)
            normal = candidates[:remaining_width]

            beam = reserved + normal
            if not beam:
                break

        return best_plan

    # -----------------------------------------------------------------------
    # Action priority for move ordering
    # -----------------------------------------------------------------------

    @staticmethod
    def _action_priority(action: Action, engine: CombatEngine) -> float:
        """Return a sort key for action ordering (lower = explore first).

        Priority tiers (lower is better):
        - 0-99:   Lethal candidates (high damage when enemies are low HP)
        - 100-199: Stance-entering cards that enable lethal or setup
        - 200-299: Attack cards (sorted by damage/cost efficiency)
        - 300-399: Zero-cost cards
        - 400-499: Setup cards (powers, draw, block)
        - 500-599: Other playable cards
        - 600-699: Potions / scry
        - 900:     EndTurn (always last)
        """
        state = engine.state

        if isinstance(action, EndTurn):
            return 900.0

        if isinstance(action, UsePotion):
            return 600.0  # Potions are worth trying but after cards

        if isinstance(action, SelectScryDiscard):
            return 650.0

        if not isinstance(action, PlayCard):
            return 800.0

        hand = state.hand
        if action.card_idx < 0 or action.card_idx >= len(hand):
            return 800.0

        card_id = hand[action.card_idx]
        base_id = card_id.rstrip("+")
        upgraded = card_id.endswith("+")

        # Resolve aliases and look up card data
        resolved = resolve_card_id(base_id)
        card_def = ALL_CARDS.get(resolved)
        if card_def is None:
            return 500.0  # Unknown card, middle priority

        # Compute effective damage
        dmg = card_def.damage  # -1 if not an attack
        if upgraded:
            dmg = card_def.base_damage + card_def.upgrade_damage if card_def.base_damage >= 0 else -1
        cost = state.card_costs.get(card_id, card_def.current_cost if not upgraded else (card_def.upgrade_cost if card_def.upgrade_cost is not None else card_def.cost))

        # Wrath doubles damage
        in_wrath = state.stance == "Wrath"
        eff_dmg = dmg
        if dmg > 0 and in_wrath:
            eff_dmg = dmg * 2

        # Enemy HP info
        living = [e for e in state.enemies if e.hp > 0 and not e.is_escaping]
        min_ehp = min((e.hp + e.block for e in living), default=999)
        total_ehp = sum(e.hp for e in living)

        # --- Tier 0: Lethal candidates (attacks that can kill an enemy) ---
        if card_def.card_type == CardType.ATTACK and eff_dmg > 0:
            # Check if this card can kill any enemy
            if action.target_idx >= 0:
                # Single target
                target = next(
                    (e for i, e in enumerate(state.enemies)
                     if i == action.target_idx and e.hp > 0),
                    None,
                )
                if target is not None:
                    effective_hp = target.hp + target.block
                    if eff_dmg >= effective_hp:
                        # Can kill! Lower priority = explored first
                        return 10.0 - min(eff_dmg, 50)
            else:
                # AoE or self-target attack
                if eff_dmg >= min_ehp:
                    return 20.0 - min(eff_dmg, 50)

            # High damage relative to enemy HP (>50% of min enemy effective HP)
            if eff_dmg > min_ehp * 0.5:
                return 50.0 + (1.0 - eff_dmg / max(min_ehp, 1)) * 50

        # --- Tier 1: Stance cards that enable lethal ---
        if base_id in _STANCE_CARDS or resolved in _STANCE_CARDS:
            stance_target = _STANCE_CARDS.get(base_id) or _STANCE_CARDS.get(resolved, "")
            if stance_target == "Wrath" and not in_wrath:
                # Entering Wrath doubles damage — high priority when enemies are low
                if total_ehp < 50:
                    return 100.0  # Very high priority for lethal setup
                return 150.0
            elif stance_target == "Calm":
                # Calm banks energy — good setup
                return 170.0
            return 180.0

        # --- Tier 2: Attack cards by damage efficiency ---
        if card_def.card_type == CardType.ATTACK and eff_dmg > 0:
            efficiency = eff_dmg / max(cost, 0.5)  # damage per energy
            return 200.0 + max(0, 50 - efficiency * 10)

        # --- Tier 3: Zero-cost cards (free value) ---
        if cost == 0:
            return 300.0

        # --- Tier 4: Setup cards (powers, block, draw) ---
        if base_id in _SETUP_CARD_PREFIXES or resolved in _SETUP_CARD_PREFIXES:
            return 400.0

        if card_def.card_type == CardType.POWER:
            return 410.0

        if card_def.card_type == CardType.SKILL:
            if card_def.base_block > 0:
                return 450.0  # Block cards
            return 460.0

        return 500.0

    def _sorted_actions(
        self, actions: List[Action], engine: CombatEngine
    ) -> List[Action]:
        """Sort non-EndTurn actions by heuristic priority (best first)."""
        non_end = [a for a in actions if not isinstance(a, EndTurn)]
        non_end.sort(key=lambda a: self._action_priority(a, engine))
        return non_end

    # -----------------------------------------------------------------------
    # Helpers
    # -----------------------------------------------------------------------

    def _is_setup_action(self, action: Action, engine: CombatEngine) -> bool:
        """Check if an action is a 'setup' action deserving reserved beam slots."""
        if isinstance(action, PlayCard):
            hand = engine.state.hand
            if 0 <= action.card_idx < len(hand):
                card_id = hand[action.card_idx]
                # Strip upgrade suffix for matching
                base_id = card_id.rstrip("+")
                return base_id in _SETUP_CARD_PREFIXES
        if isinstance(action, UsePotion):
            return True  # Potions are always worth considering
        return False

    def _action_in_list(self, action: Action, legal_actions: List[Action]) -> bool:
        """Check if an action is in the legal actions list (structural equality)."""
        for la in legal_actions:
            if type(action) is type(la):
                if isinstance(action, PlayCard) and isinstance(la, PlayCard):
                    if action.card_idx == la.card_idx and action.target_idx == la.target_idx:
                        return True
                elif isinstance(action, EndTurn):
                    return True
                elif isinstance(action, UsePotion) and isinstance(la, UsePotion):
                    if action.potion_idx == la.potion_idx and action.target_idx == la.target_idx:
                        return True
                elif isinstance(action, SelectScryDiscard) and isinstance(la, SelectScryDiscard):
                    if action.discard_indices == la.discard_indices:
                        return True
        return False

    def _state_hash(self, engine: CombatEngine) -> int:
        """Quick hash for checking if engine state changed (for cache validity).

        Includes enemy state so cache hits are safe across different fights.
        """
        s = engine.state
        enemy_key = tuple(
            (e.id, e.hp, e.block, getattr(e, 'intent', None)) for e in s.enemies if e.hp > 0
        ) if hasattr(s, 'enemies') else ()
        power_key = tuple(sorted(
            (p.id, p.amount) for p in getattr(s.player, 'powers', [])
        )) if hasattr(s.player, 'powers') else ()
        return hash((
            s.player.hp,
            s.player.block,
            s.energy,
            s.turn,
            s.stance,
            tuple(s.hand),
            len(s.draw_pile),
            s.cards_played_this_turn,
            enemy_key,
            power_key,
        ))

    def _invalidate_cache(self) -> None:
        """Clear the cached plan."""
        self._cached_plan = None
        self._cached_plan_index = 0
        self._cached_engine_hash = None


# ---------------------------------------------------------------------------
# MultiTurnSolver: multi-turn lookahead for boss/elite fights
# ---------------------------------------------------------------------------

class MultiTurnSolver:
    """Multi-turn lookahead search for boss and elite fights.

    Instead of optimizing individual card plays, this solver:
    1. Uses TurnSolver to find top-K turn plans at each depth
    2. Simulates full turns (play cards → end turn → enemy response)
    3. Recurses for depth D turns ahead
    4. Returns the first-turn plan that leads to the best D-turn outcome

    With K=3 plans and D=3 depth, explores 27 leaf states — feasible in <5s.
    """

    def __init__(
        self,
        inner_solver: Optional[TurnSolver] = None,
        max_depth: int = 3,
        top_k: int = 3,
        time_budget_ms: float = 5000.0,
    ):
        self._solver = inner_solver or TurnSolver(time_budget_ms=100, node_budget=5000)
        self.max_depth = max_depth
        self.top_k = top_k
        self.time_budget_ms = time_budget_ms

    def solve(self, engine: CombatEngine) -> Optional[List[Action]]:
        """Find the best turn plan considering multi-turn outcomes.

        Returns the list of actions for the CURRENT turn only.
        """
        if engine.phase != CombatPhase.PLAYER_TURN or engine.state.combat_over:
            return None

        deadline = time.monotonic() + self.time_budget_ms / 1000.0
        best_score = _SCORE_DEATH - 1
        best_plan: Optional[List[Action]] = None

        # Get top-K turn plans for current state
        candidates = self._get_turn_candidates(engine, deadline)
        if not candidates:
            return None

        for plan, _ in candidates:
            if time.monotonic() > deadline:
                break
            # Simulate this turn plan and evaluate recursively
            score = self._evaluate_plan(engine, plan, depth=1, deadline=deadline)
            if score > best_score:
                best_score = score
                best_plan = plan

        return best_plan

    def _get_turn_candidates(
        self, engine: CombatEngine, deadline: float
    ) -> List[Tuple[List[Action], float]]:
        """Get top-K diverse turn plans from the inner TurnSolver.

        Runs the solver multiple times with different search parameters
        to find diverse candidate plans.
        """
        candidates: List[Tuple[List[Action], float]] = []
        seen_scores: set = set()

        # Run solver with decreasing budgets to find diverse plans
        for budget_ms, budget_nodes in [(100, 5000), (50, 2000), (30, 1000)]:
            if time.monotonic() > deadline or len(candidates) >= self.top_k:
                break

            solver = TurnSolver(time_budget_ms=budget_ms, node_budget=budget_nodes)
            plan = solver.solve_turn(engine, room_type="boss")
            if plan:
                # Score this plan
                score = self._score_plan(engine, plan)
                # Deduplicate by score (close enough = same plan)
                score_key = round(score, 1)
                if score_key not in seen_scores:
                    seen_scores.add(score_key)
                    candidates.append((plan, score))

        # Also try: aggressive (Wrath-first), defensive (block-first), end-turn-early
        for variant_plan in self._generate_variant_plans(engine, deadline):
            if len(candidates) >= self.top_k:
                break
            score = self._score_plan(engine, variant_plan)
            score_key = round(score, 1)
            if score_key not in seen_scores:
                seen_scores.add(score_key)
                candidates.append((variant_plan, score))

        candidates.sort(key=lambda x: -x[1])
        return candidates[:self.top_k]

    def _generate_variant_plans(
        self, engine: CombatEngine, deadline: float
    ) -> List[List[Action]]:
        """Generate variant plans: aggressive, defensive, conservative."""
        variants: List[List[Action]] = []
        state = engine.state

        # Variant 1: End turn immediately (sometimes correct for stance management)
        variants.append([EndTurn()])

        # Variant 2: Play only block cards then end turn
        block_plan: List[Action] = []
        sim = engine.copy()
        for _ in range(10):  # max 10 cards
            if time.monotonic() > deadline:
                break
            actions = sim.get_legal_actions()
            block_actions = [
                a for a in actions
                if isinstance(a, PlayCard) and self._is_block_action(a, sim)
            ]
            if not block_actions:
                break
            action = block_actions[0]
            block_plan.append(action)
            sim.execute_action(action)
        if block_plan:
            block_plan.append(EndTurn())
            variants.append(block_plan)

        return variants

    def _is_block_action(self, action: Action, engine: CombatEngine) -> bool:
        """Check if a PlayCard action generates block."""
        if not isinstance(action, PlayCard):
            return False
        hand = engine.state.hand
        if 0 <= action.card_idx < len(hand):
            card_id = hand[action.card_idx]
            base_id = card_id.rstrip("+")
            card_def = ALL_CARDS.get(base_id)
            if card_def and card_def.base_block > 0:
                return True
        return False

    def _score_plan(self, engine: CombatEngine, plan: List[Action]) -> float:
        """Score a turn plan by simulating it and enemy response."""
        sim = engine.copy()
        for action in plan:
            try:
                sim.execute_action(action)
            except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                logger.warning("TurnSolver: _score_plan execute_action failed: %s", e)
                return _SCORE_DEATH
            if sim.state.combat_over:
                break

        return self._solver._score_terminal(sim, engine)

    def _evaluate_plan(
        self,
        engine: CombatEngine,
        plan: List[Action],
        depth: int,
        deadline: float,
    ) -> float:
        """Recursively evaluate a turn plan to the given depth.

        Simulates: execute plan → enemy turn → next player turn → recurse.
        """
        if time.monotonic() > deadline:
            return self._score_plan(engine, plan)

        # Execute the plan
        sim = engine.copy()
        for action in plan:
            try:
                sim.execute_action(action)
            except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                logger.warning("TurnSolver: _evaluate_plan execute_action failed: %s", e)
                return _SCORE_DEATH
            if sim.state.combat_over:
                break

        # If combat is over, return terminal value
        if sim.state.combat_over or sim.is_combat_over():
            if sim.is_victory():
                hp = sim.state.player.hp
                return _SCORE_LETHAL + hp * 200
            return _SCORE_DEATH

        # Simulate enemy turn (the plan should end with EndTurn)
        if sim.phase == CombatPhase.PLAYER_TURN:
            # Plan didn't include EndTurn — add it
            try:
                sim.execute_action(EndTurn())
            except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                logger.warning("TurnSolver: _evaluate_plan EndTurn failed: %s", e)

        # end_turn() is monolithic — enemy turn already executed. No tick loop needed.

        if sim.state.combat_over or sim.is_combat_over():
            if sim.is_victory():
                return _SCORE_LETHAL + sim.state.player.hp * 200
            return _SCORE_DEATH + sim.state.player.hp

        if sim.state.player.hp <= 0:
            return _SCORE_DEATH

        # At max depth, evaluate with single-turn score
        if depth >= self.max_depth:
            return self._solver._score_terminal(sim, engine)

        # Recurse: find best plan for next turn
        next_candidates = self._get_turn_candidates(sim, deadline)
        if not next_candidates:
            return self._solver._score_terminal(sim, engine)

        best_next = _SCORE_DEATH - 1
        for next_plan, _ in next_candidates:
            if time.monotonic() > deadline:
                break
            score = self._evaluate_plan(sim, next_plan, depth + 1, deadline)
            if score > best_next:
                best_next = score

        return best_next


# ---------------------------------------------------------------------------
# TurnSolverAdapter: drop-in bridge for training_runner.py
# ---------------------------------------------------------------------------

class TurnSolverAdapter:
    """Drop-in replacement for _pick_combat_action() in training_runner.py.

    Wraps TurnSolver and bridges between the GameRunner's CombatAction format
    and the engine-level Action format used by TurnSolver.

    Uses multi-turn solver for boss/elite fights (plans 3+ turns ahead),
    single-turn DFS/beam solver for normal fights.

    Usage:
        adapter = TurnSolverAdapter()
        action = adapter.pick_action(actions, runner, room_type="monster")
    """

    def __init__(
        self,
        time_budget_ms: float = 5.0,
        node_budget: int = 1000,
        multi_turn_depth: int = 5,
        multi_turn_k: int = 4,
        multi_turn_budget_ms: float = 30000.0,
        solver_budgets: dict = None,  # {room_type: (base_ms, base_nodes, cap_ms)}
        combat_net=None,  # Optional CombatNet for neural leaf evaluation
    ):
        # Build neural eval closure if CombatNet provided
        self._combat_net = combat_net
        self._combat_encoder = None
        neural_eval = None
        if combat_net is not None:
            from packages.training.state_encoders import CombatStateEncoder
            self._combat_encoder = CombatStateEncoder()
            _cn = combat_net
            _ce = self._combat_encoder

            def neural_eval(engine):
                obs = _ce.encode(engine)
                return _cn.predict(obs)

        self._solver = TurnSolver(
            time_budget_ms=time_budget_ms,
            node_budget=node_budget,
            neural_eval=neural_eval,
        )
        # Multi-turn solver for boss/elite fights
        self._multi_turn = MultiTurnSolver(
            inner_solver=TurnSolver(time_budget_ms=500, node_budget=50000, neural_eval=neural_eval),
            max_depth=multi_turn_depth,
            top_k=multi_turn_k,
            time_budget_ms=multi_turn_budget_ms,
        )
        # Per-room-type budgets: {room_type: (base_ms, base_nodes, cap_ms)}
        self._solver_budgets = solver_budgets or {}

        self._cached_plan: Optional[List[Action]] = None
        self._cached_plan_index: int = 0
        self._cached_turn: int = -1
        self._cached_combat_id: int = -1

    def reset(self):
        """Invalidate cached plan. Call at the start of each new combat."""
        self._cached_plan = None
        self._cached_plan_index = 0
        self._cached_turn = -1
        self._cached_combat_id = -1

    def pick_action(self, actions: list, runner, room_type: str = "monster"):
        """Pick best combat action using DFS/beam TurnSolver.

        For boss/elite fights, uses multi-turn solver (plans 3+ turns ahead).
        Falls back to single-turn DFS/beam solver for normal fights.

        Args:
            actions: List of CombatAction objects (from GameRunner.get_available_actions)
            runner: GameRunner instance (has .current_combat)
            room_type: "monster", "elite", or "boss"

        Returns:
            A CombatAction from the actions list. Falls back to first action on failure.
        """
        if len(actions) <= 1:
            return actions[0] if actions else None

        engine = getattr(runner, "current_combat", None)
        if engine is None:
            return None  # Let caller fall through to CombatPlanner/heuristic

        # Invalidate cache when combat identity changes (new fight)
        combat_id = id(engine)
        if combat_id != self._cached_combat_id:
            self._cached_plan = None
            self._cached_plan_index = 0
            self._cached_turn = -1
            self._cached_combat_id = combat_id

        state = engine.state
        current_turn = state.turn

        # Check if cached plan is still valid for this turn
        if (
            self._cached_plan is not None
            and self._cached_turn == current_turn
            and self._cached_plan_index < len(self._cached_plan)
        ):
            engine_action = self._cached_plan[self._cached_plan_index]
            combat_action = self._match_engine_to_combat(engine_action, actions, state)
            if combat_action is not None:
                self._cached_plan_index += 1
                self._warn_end_turn_with_playable(combat_action, actions, engine, room_type)
                return combat_action
            # Cache invalid — replan
            self._cached_plan = None

        # Apply room-type-specific budgets from training_config.SOLVER_BUDGETS
        _rt_key = room_type.lower() if room_type else "monster"
        if self._solver_budgets and _rt_key in self._solver_budgets:
            base_ms, base_nodes, cap_ms = self._solver_budgets[_rt_key]
            # Scale by total enemy HP
            total_hp = sum(getattr(e, 'hp', 0) for e in getattr(engine.state, 'enemies', []) if getattr(e, 'hp', 0) > 0)
            from .training_config import SOLVER_HP_SCALE_DIVISOR
            scale = max(1.0, total_hp / SOLVER_HP_SCALE_DIVISOR)
            budget_ms = min(base_ms * scale, cap_ms)
            budget_nodes = int(base_nodes * scale)
            self._solver.default_time_budget_ms = budget_ms
            self._solver.default_node_budget = budget_nodes

        # For boss/elite: try multi-turn solver first (plans 3+ turns ahead)
        if room_type in ("boss", "b", "elite", "e"):
            try:
                plan = self._multi_turn.solve(engine)
            except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                logger.warning("TurnSolver: multi-turn solve failed: %s", e)
                plan = None
            if plan and len(plan) > 0:
                self._cached_plan = plan
                self._cached_plan_index = 0
                self._cached_turn = current_turn
                engine_action = plan[0]
                combat_action = self._match_engine_to_combat(engine_action, actions, state)
                if combat_action is not None:
                    self._cached_plan_index = 1
                    self._warn_end_turn_with_playable(combat_action, actions, engine, room_type)
                    return combat_action

        # DFS/beam TurnSolver (fast single-turn heuristic)
        try:
            plan = self._solver.solve_turn(engine, room_type)
        except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
            logger.warning("TurnSolver: single-turn solve failed: %s", e)
            plan = None

        if plan and len(plan) > 0:
            self._cached_plan = plan
            self._cached_plan_index = 0
            self._cached_turn = current_turn

            engine_action = plan[0]
            combat_action = self._match_engine_to_combat(engine_action, actions, state)
            if combat_action is not None:
                self._cached_plan_index = 1
                self._warn_end_turn_with_playable(combat_action, actions, engine, room_type)
                return combat_action

        return None  # Fall through to first legal action

    def _warn_end_turn_with_playable(self, combat_action, actions, engine, room_type):
        """Log a warning if the solver chose EndTurn while playable cards exist with energy."""
        if getattr(combat_action, "action_type", None) == "end_turn" and engine.state.energy > 0:
            playable = [a for a in actions if getattr(a, "action_type", None) == "play_card"]
            if playable:
                logger.warning(
                    "TurnSolver chose EndTurn with %d playable cards (energy=%d, room=%s)",
                    len(playable), engine.state.energy, room_type,
                )

    def _match_engine_to_combat(self, engine_action: Action, combat_actions: list, state) -> Any:
        """Match an engine-level Action to a CombatAction in the available list.

        The CombatAction objects have an .action_type string attribute.
        """
        if isinstance(engine_action, EndTurn):
            for ca in combat_actions:
                if getattr(ca, "action_type", None) == "end_turn":
                    return ca

        elif isinstance(engine_action, PlayCard):
            for ca in combat_actions:
                if getattr(ca, "action_type", None) == "play_card":
                    if (
                        getattr(ca, "card_idx", -99) == engine_action.card_idx
                        and getattr(ca, "target_idx", -99) == engine_action.target_idx
                    ):
                        return ca

        elif isinstance(engine_action, UsePotion):
            for ca in combat_actions:
                if getattr(ca, "action_type", None) == "use_potion":
                    if (
                        getattr(ca, "potion_idx", -99) == engine_action.potion_idx
                        and getattr(ca, "target_idx", -99) == engine_action.target_idx
                    ):
                        return ca

        elif isinstance(engine_action, SelectScryDiscard):
            for ca in combat_actions:
                if getattr(ca, "action_type", None) == "select_scry_discard":
                    if getattr(ca, "scry_discard_indices", None) == engine_action.discard_indices:
                        return ca

        return None
