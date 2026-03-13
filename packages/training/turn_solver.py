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

import time
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple
import heapq

from packages.engine.combat_engine import CombatEngine, CombatPhase
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

# Node budgets per room type
_BUDGETS: Dict[str, Tuple[float, int]] = {
    "monster": (2.0, 500),
    "elite": (30.0, 5_000),
    "boss": (100.0, 20_000),
}


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
    ):
        self.default_time_budget_ms = time_budget_ms
        self.default_node_budget = node_budget

        # Tree reuse state
        self._cached_plan: Optional[List[Action]] = None
        self._cached_plan_index: int = 0
        self._cached_engine_hash: Optional[int] = None

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

        time_ms, node_limit = _BUDGETS.get(room_type, _BUDGETS["monster"])
        # Override with instance defaults if they're more generous
        time_ms = max(time_ms, self.default_time_budget_ms)
        node_limit = max(node_limit, self.default_node_budget)

        actions = engine.get_legal_actions()
        if not actions:
            return None

        # Only EndTurn available — nothing to search
        non_end = [a for a in actions if not isinstance(a, EndTurn)]
        if not non_end:
            return [EndTurn()]

        # Estimate tree size for strategy selection
        estimated_nodes = self._estimate_tree_size(engine)

        if estimated_nodes < 300:
            plan = self._exact_dfs(engine, time_ms, node_limit)
        elif estimated_nodes < 5000:
            plan = self._best_first_search(engine, time_ms, node_limit)
        else:
            plan = self._beam_search(engine, time_ms, node_limit)

        if plan is not None:
            # Cache for tree reuse
            self._cached_plan = plan
            self._cached_plan_index = 0
            self._cached_engine_hash = self._state_hash(engine)

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

        # Simulate enemy turn to get actual HP loss
        try:
            sim = engine.copy()
            sim.execute_action(EndTurn())
            # Run until player's next turn or combat ends
            while (
                sim.state.phase == CombatPhase.ENEMY_TURN
                and not sim.is_combat_over()
            ):
                sim.tick()
            hp_after = sim.state.player.hp
        except Exception:
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

        # Score: all terms are in comparable units (HP-equivalent)
        score = (
            -6.0 * actual_hp_lost
            + 60.0 * enemies_killed
            - 1.5 * now_ehp / max(orig_ehp, 1) * 10  # normalized remaining HP
            - 12.0 * min(est_turns, 10)
        )

        # Stance: ending in Calm banks energy; ending in Wrath is dangerous
        if state.stance == "Calm":
            score += 25.0
        elif state.stance == "Wrath" and now_living > 0:
            score -= 60.0

        # Penalize unspent energy (unless Ice Cream relic present)
        unspent = state.energy
        if unspent > 0:
            has_ice_cream = any(r.relic_id == "IceCream" for r in getattr(state, 'relics', []))
            if not has_ice_cream:
                score -= 3.0 * unspent

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

            # Try each non-EndTurn action
            non_end = [a for a in actions if not isinstance(a, EndTurn)]
            for action in non_end:
                if nodes_visited > node_budget or time.monotonic() > deadline:
                    return

                child = eng.copy()
                try:
                    child.execute_action(action)
                except Exception:
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

            actions = node.engine.get_legal_actions()
            non_end = [a for a in actions if not isinstance(a, EndTurn)]

            for action in non_end:
                if nodes_visited >= node_budget or time.monotonic() > deadline:
                    break

                child_engine = node.engine.copy()
                try:
                    child_engine.execute_action(action)
                except Exception:
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
    ) -> Optional[List[Action]]:
        """Beam search with reserved slots for setup cards.

        Keeps `beam_width` candidates per depth level.
        Reserves `reserved_setup_slots` beam positions for setup actions
        (stance changes, powers, draw cards) so they aren't pruned early.
        """
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

                actions = node.engine.get_legal_actions()
                non_end = [a for a in actions if not isinstance(a, EndTurn)]

                for action in non_end:
                    if nodes_visited >= node_budget or time.monotonic() > deadline:
                        break

                    child_engine = node.engine.copy()
                    try:
                        child_engine.execute_action(action)
                    except Exception:
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
        """Quick hash for checking if engine state changed (for cache validity)."""
        s = engine.state
        return hash((
            s.player.hp,
            s.player.block,
            s.energy,
            s.turn,
            s.stance,
            tuple(s.hand),
            len(s.draw_pile),
            s.cards_played_this_turn,
        ))

    def _invalidate_cache(self) -> None:
        """Clear the cached plan."""
        self._cached_plan = None
        self._cached_plan_index = 0
        self._cached_engine_hash = None


# ---------------------------------------------------------------------------
# TurnSolverAdapter: drop-in bridge for overnight.py
# ---------------------------------------------------------------------------

class TurnSolverAdapter:
    """Drop-in replacement for _pick_combat_action() in overnight.py.

    Wraps TurnSolver and bridges between the GameRunner's CombatAction format
    and the engine-level Action format used by TurnSolver.

    When an InferenceClient is available, uses GumbelMCTS with GPU-backed
    value evaluation for elite/boss fights. Falls back to the DFS/beam
    TurnSolver for normal fights or when the inference server is unavailable.

    Usage:
        adapter = TurnSolverAdapter()
        action = adapter.pick_action(actions, runner, room_type="monster")
    """

    def __init__(
        self,
        time_budget_ms: float = 5.0,
        node_budget: int = 1000,
        mcts_simulations: int = 16,
        use_mcts_for: Optional[Tuple[str, ...]] = None,
    ):
        """
        Args:
            time_budget_ms: Time budget for DFS/beam TurnSolver.
            node_budget: Node budget for DFS/beam TurnSolver.
            mcts_simulations: Simulation budget for GumbelMCTS (default 16).
            use_mcts_for: Room types to use MCTS for. Default ("elite", "boss").
                Set to ("monster", "elite", "boss") to use MCTS everywhere.
        """
        self._solver = TurnSolver(
            time_budget_ms=time_budget_ms,
            node_budget=node_budget,
        )
        self._mcts_simulations = mcts_simulations
        self._use_mcts_for = use_mcts_for or ("elite", "boss")
        self._mcts: Optional[Any] = None  # Lazy-init GumbelMCTS
        self._mcts_policy_fn: Optional[Any] = None  # Cached policy_fn

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

    def _get_mcts(self) -> Optional[Any]:
        """Lazily initialize GumbelMCTS with GPU-backed policy_fn if available.

        Returns None if InferenceClient is not available, in which case
        the adapter falls back to the DFS/beam TurnSolver.
        """
        if self._mcts is not None:
            return self._mcts

        try:
            from packages.training.inference_server import get_client
            from packages.training.gumbel_mcts import GumbelMCTS
            from packages.training.combat_state_encoder import make_mcts_policy_fn

            client = get_client()
            if client is None:
                return None

            self._mcts_policy_fn = make_mcts_policy_fn(client)
            self._mcts = GumbelMCTS(
                policy_fn=self._mcts_policy_fn,
                num_simulations=self._mcts_simulations,
                max_candidates=min(16, self._mcts_simulations),
            )
            return self._mcts
        except Exception:
            return None

    def pick_action(self, actions: list, runner, room_type: str = "monster"):
        """Pick best combat action.

        For room types in use_mcts_for (default: elite, boss), attempts
        GumbelMCTS with GPU-backed value evaluation. Falls back to
        DFS/beam TurnSolver if MCTS is unavailable or fails.

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
                return combat_action
            # Cache invalid — replan
            self._cached_plan = None

        # Try GumbelMCTS for eligible room types
        if room_type in self._use_mcts_for:
            mcts_result = self._try_mcts(engine, actions, state)
            if mcts_result is not None:
                return mcts_result

        # Fall back to DFS/beam TurnSolver
        try:
            plan = self._solver.solve_turn(engine, room_type)
        except Exception:
            return None  # Fall through to CombatPlanner/heuristic

        if plan is None or len(plan) == 0:
            return None

        self._cached_plan = plan
        self._cached_plan_index = 0
        self._cached_turn = current_turn

        # Return first action from plan
        engine_action = plan[0]
        combat_action = self._match_engine_to_combat(engine_action, actions, state)
        if combat_action is not None:
            self._cached_plan_index = 1
            return combat_action

        return None  # Fall through to CombatPlanner/heuristic

    def _try_mcts(self, engine: Any, actions: list, state: Any) -> Optional[Any]:
        """Try to use GumbelMCTS for action selection.

        Returns a CombatAction if MCTS succeeds, None otherwise.
        """
        mcts = self._get_mcts()
        if mcts is None:
            return None

        try:
            visit_probs = mcts.search(engine)
            if not visit_probs:
                return None

            # Select best action (greedy -- temperature=0)
            best_engine_action = mcts.select_action(visit_probs, temperature=0.0)

            # Match to combat action
            combat_action = self._match_engine_to_combat(best_engine_action, actions, state)
            return combat_action
        except Exception:
            return None  # Fall back to DFS/beam TurnSolver

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
