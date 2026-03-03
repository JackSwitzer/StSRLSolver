"""
Strategic Planner + StS Agent for Slay the Spire.

Provides:
1. StrategicPlanner - Run-level decision maker for non-combat phases
   (path selection, card picks, rest sites, shop decisions).
2. StSAgent - Complete agent combining CombatMCTS + StrategicPlanner
   to play full games through the GameRunner API.
"""

from __future__ import annotations

from typing import Any, Callable, Dict, List, Optional

from .mcts import CombatMCTS


# =============================================================================
# StrategicPlanner
# =============================================================================

class StrategicPlanner:
    """
    Run-level planner for non-combat decisions.

    Uses heuristic evaluation of the run state to make path/card/shop/rest
    decisions. Can optionally accept a combat_predictor function that
    estimates expected HP loss from a combat.
    """

    def __init__(
        self,
        combat_predictor: Optional[Callable] = None,
        lookahead_depth: int = 3,
    ):
        """
        Args:
            combat_predictor: Optional fn(runner, encounter_type) -> float
                              that estimates expected HP loss from a combat.
            lookahead_depth: How many floors ahead to consider for pathing.
        """
        self.combat_predictor = combat_predictor
        self.lookahead_depth = lookahead_depth

    # -----------------------------------------------------------------
    # Path selection
    # -----------------------------------------------------------------

    def plan_path_choice(self, runner: Any, available_paths: List[Any]) -> int:
        """
        Choose best path node index based on estimated outcomes.

        Scores each reachable node by room-type desirability given
        current run state (HP, deck quality, relics, potions).

        Args:
            runner: GameRunner instance.
            available_paths: List of path descriptions (MapRoomNode or dicts).

        Returns:
            Index into available_paths for the best choice.
        """
        if not available_paths:
            return 0

        rs = runner.run_state
        hp_pct = rs.current_hp / max(rs.max_hp, 1)

        best_idx = 0
        best_score = float('-inf')

        for i, path in enumerate(available_paths):
            room_type = self._extract_room_type(path)
            score = self._score_room(room_type, hp_pct, rs)
            if score > best_score:
                best_score = score
                best_idx = i

        return best_idx

    def _extract_room_type(self, path: Any) -> str:
        """Extract room type string from a path node."""
        if isinstance(path, dict):
            return path.get("room_type", path.get("type", "unknown"))
        if hasattr(path, "room_type"):
            rt = path.room_type
            return rt.value if hasattr(rt, "value") else str(rt)
        return "unknown"

    def _score_room(self, room_type: str, hp_pct: float, run_state: Any) -> float:
        """Score a room type given current run state."""
        rt = room_type.lower()

        if rt in ("rest", "rest_site"):
            # Prefer rest when low HP
            return 3.0 if hp_pct < 0.5 else 1.0

        if rt in ("monster", "combat"):
            return 2.0 if hp_pct > 0.6 else 0.5

        if rt == "elite":
            # Elites give relics but are dangerous
            return 2.5 if hp_pct > 0.7 else -1.0

        if rt == "event":
            return 1.5

        if rt in ("shop", "merchant"):
            gold = getattr(run_state, "gold", 0)
            return 2.0 if gold > 100 else 0.8

        if rt in ("treasure", "chest"):
            return 2.0

        return 0.0

    # -----------------------------------------------------------------
    # Card pick
    # -----------------------------------------------------------------

    def plan_card_pick(self, runner: Any, card_options: List[Any]) -> int:
        """
        Choose best card (or skip) from reward options.

        Returns index into card_options for best pick.
        Index == len(card_options) means skip.
        """
        if not card_options:
            return 0  # skip

        rs = runner.run_state
        deck_size = len(getattr(rs, "deck", []))

        best_idx = len(card_options)  # default: skip
        best_score = 0.0  # skip baseline

        for i, card in enumerate(card_options):
            score = self._score_card(card, deck_size, rs)
            if score > best_score:
                best_score = score
                best_idx = i

        return best_idx

    def _score_card(self, card: Any, deck_size: int, run_state: Any) -> float:
        """Heuristic card quality score."""
        card_id = card if isinstance(card, str) else getattr(card, "id", str(card))
        card_lower = card_id.lower()

        # High-value Watcher cards
        tier1 = {"rushdown", "tantrum", "ragnarok", "mentalfortress", "talktothehand"}
        tier2 = {"innerpeace", "cutthroughfate", "wheelkick", "conclude",
                 "wallop", "emptyfist", "crescendo+", "blasphemy"}

        score = 0.5  # baseline "ok" card

        for t in tier1:
            if t in card_lower:
                score = 3.0
                break
        else:
            for t in tier2:
                if t in card_lower:
                    score = 2.0
                    break

        # Penalize adding cards to bloated decks
        if deck_size > 25:
            score -= 0.5
        if deck_size > 35:
            score -= 0.5

        # Upgraded cards are better
        if card_id.endswith("+"):
            score += 0.3

        return score

    # -----------------------------------------------------------------
    # Rest site
    # -----------------------------------------------------------------

    def plan_rest_site(self, runner: Any, options: List[Any]) -> int:
        """
        Choose rest site action.

        Returns index into options for the best action.
        """
        if not options:
            return 0

        rs = runner.run_state
        hp_pct = rs.current_hp / max(rs.max_hp, 1)

        best_idx = 0
        best_score = float('-inf')

        for i, opt in enumerate(options):
            action_type = self._extract_rest_action_type(opt)
            score = self._score_rest_action(action_type, hp_pct, rs)
            if score > best_score:
                best_score = score
                best_idx = i

        return best_idx

    def _extract_rest_action_type(self, opt: Any) -> str:
        if isinstance(opt, dict):
            return opt.get("action_type", opt.get("type", "rest"))
        if hasattr(opt, "action_type"):
            return opt.action_type
        return str(opt)

    def _score_rest_action(self, action_type: str, hp_pct: float, run_state: Any) -> float:
        at = action_type.lower()

        if at == "rest":
            # Heal ~30% max HP. More valuable when low.
            if hp_pct < 0.35:
                return 5.0
            if hp_pct < 0.6:
                return 3.0
            return 0.5

        if at == "upgrade":
            # Upgrade is high-value when we're healthy
            if hp_pct > 0.6:
                return 4.0
            return 1.5

        if at == "dig":
            return 2.0 if hp_pct > 0.5 else 0.5

        if at == "lift":
            return 1.5 if hp_pct > 0.5 else 0.3

        if at == "toke":
            # Card removal is strong for deck thinning
            deck_size = len(getattr(run_state, "deck", []))
            if deck_size > 20:
                return 2.5 if hp_pct > 0.5 else 1.0
            return 1.5

        return 0.0

    # -----------------------------------------------------------------
    # Shop decisions
    # -----------------------------------------------------------------

    def plan_shop_action(self, runner: Any, options: List[Any]) -> int:
        """Choose best shop action. Prefers card removal > key cards > skip."""
        if not options:
            return 0

        rs = runner.run_state
        gold = getattr(rs, "gold", 0)
        deck_size = len(getattr(rs, "deck", []))

        best_idx = 0
        best_score = 0.0  # skip baseline

        for i, opt in enumerate(options):
            opt_str = str(opt).lower()

            # Card removal is very strong for deck thinning
            if "remove" in opt_str:
                score = 5.0 if deck_size > 15 else 2.0
            # Buying a key card
            elif any(t in opt_str for t in ("rushdown", "tantrum", "ragnarok", "mentalfortress")):
                score = 4.0 if gold > 150 else 2.0
            # Skip / leave shop
            elif "skip" in opt_str or "leave" in opt_str:
                score = 0.1
            else:
                score = 0.5  # generic buy

            if score > best_score:
                best_score = score
                best_idx = i

        return best_idx

    # -----------------------------------------------------------------
    # Event decisions
    # -----------------------------------------------------------------

    def plan_event_choice(self, runner: Any, options: List[Any]) -> int:
        """Choose best event option. Heuristic: take free stuff, avoid curses/HP loss."""
        if not options:
            return 0

        rs = runner.run_state
        hp_pct = getattr(rs, "current_hp", 0) / max(getattr(rs, "max_hp", 72), 1)

        best_idx = 0
        best_score = 0.0

        for i, opt in enumerate(options):
            opt_str = str(opt).lower()

            # Free relic/gold/upgrade = good
            if any(t in opt_str for t in ("relic", "gold", "upgrade", "heal", "max_hp")):
                score = 3.0
            # Card removal = good
            elif "remove" in opt_str:
                score = 2.5
            # Curse/damage = bad
            elif any(t in opt_str for t in ("curse", "lose_hp", "damage")):
                score = -1.0 if hp_pct < 0.5 else 0.5
            # Leave/skip = safe default
            elif "leave" in opt_str or "skip" in opt_str:
                score = 1.0
            else:
                score = 1.0  # neutral

            if score > best_score:
                best_score = score
                best_idx = i

        return best_idx

    # -----------------------------------------------------------------
    # State evaluation
    # -----------------------------------------------------------------

    def evaluate_state(self, runner: Any) -> float:
        """
        Estimate P(win) from current run state.

        Returns a value in [0, 1].
        """
        rs = runner.run_state
        hp_pct = rs.current_hp / max(rs.max_hp, 1)
        floor = getattr(rs, "floor", 1)
        act = getattr(rs, "act", 1)

        # Base survival estimate from HP
        value = hp_pct * 0.5

        # Progress bonus: further = harder but closer to winning
        progress = min(floor / 55.0, 1.0)
        value += progress * 0.2

        # Deck quality proxy (more cards != always better)
        deck_size = len(getattr(rs, "deck", []))
        if 12 <= deck_size <= 25:
            value += 0.1  # Good range
        elif deck_size > 35:
            value -= 0.1  # Bloated

        # Relic count bonus
        relic_count = len(getattr(rs, "relics", []))
        value += min(relic_count * 0.02, 0.2)

        return max(0.0, min(1.0, value))


# =============================================================================
# StSAgent - Complete agent combining combat + strategic planning
# =============================================================================

class StSAgent:
    """
    Complete agent combining combat MCTS + strategic planner.

    Delegates to CombatMCTS during combat and StrategicPlanner
    for all out-of-combat decisions. Interfaces with GameRunner
    through get_available_actions/take_action.
    """

    def __init__(
        self,
        combat_sims: int = 128,
        strategic_depth: int = 3,
        policy_fn: Optional[Callable] = None,
        combat_predictor: Optional[Callable] = None,
        temperature: float = 0.0,
    ):
        """
        Args:
            combat_sims: Number of MCTS simulations per combat move.
            strategic_depth: Lookahead depth for strategic planner.
            policy_fn: Optional neural network policy for CombatMCTS.
            combat_predictor: Optional combat outcome predictor for planner.
            temperature: Action selection temperature (0 = greedy).
        """
        self.combat_mcts = CombatMCTS(
            policy_fn=policy_fn,
            num_simulations=combat_sims,
        )
        self.planner = StrategicPlanner(
            combat_predictor=combat_predictor,
            lookahead_depth=strategic_depth,
        )
        self.temperature = temperature

    def get_action(self, runner: Any) -> Any:
        """
        Get best action for current game state.

        Args:
            runner: GameRunner instance.

        Returns:
            A GameAction to pass to runner.take_action().
        """
        # Import here to avoid circular dependency
        from packages.engine.game import GamePhase

        if runner.phase == GamePhase.COMBAT:
            return self._combat_action(runner)
        else:
            return self._strategic_action(runner)

    def _combat_action(self, runner: Any) -> Any:
        """Use CombatMCTS to pick a combat action."""
        engine = runner.current_combat
        if engine is None:
            actions = runner.get_available_actions()
            return actions[0] if actions else None

        action_probs = self.combat_mcts.search(engine)
        if not action_probs:
            actions = runner.get_available_actions()
            return actions[0] if actions else None

        # Map engine actions back to GameRunner CombatAction format
        best_engine_action = self.combat_mcts.select_action(
            action_probs, temperature=self.temperature
        )
        return self._engine_action_to_game_action(best_engine_action)

    def _strategic_action(self, runner: Any) -> Any:
        """Use StrategicPlanner for non-combat decisions."""
        from packages.engine.game import GamePhase

        actions = runner.get_available_actions()
        if not actions:
            return None

        if len(actions) == 1:
            return actions[0]

        phase = runner.phase

        if phase == GamePhase.MAP_NAVIGATION:
            idx = self.planner.plan_path_choice(runner, actions)
            return actions[min(idx, len(actions) - 1)]

        if phase == GamePhase.REST:
            idx = self.planner.plan_rest_site(runner, actions)
            return actions[min(idx, len(actions) - 1)]

        if phase == GamePhase.COMBAT_REWARDS or phase == GamePhase.BOSS_REWARDS:
            idx = self.planner.plan_card_pick(runner, actions)
            return actions[min(idx, len(actions) - 1)]

        if phase == GamePhase.SHOP:
            idx = self.planner.plan_shop_action(runner, actions)
            return actions[min(idx, len(actions) - 1)]

        if phase == GamePhase.EVENT:
            idx = self.planner.plan_event_choice(runner, actions)
            return actions[min(idx, len(actions) - 1)]

        # Default: pick first available action (TREASURE, NEOW, etc.)
        return actions[0]

    def _engine_action_to_game_action(self, engine_action: Any) -> Any:
        """Convert CombatEngine Action to GameRunner CombatAction."""
        from packages.engine.game import CombatAction
        from packages.engine.state.combat import PlayCard, UsePotion, EndTurn

        if isinstance(engine_action, PlayCard):
            return CombatAction(
                action_type="play_card",
                card_idx=engine_action.card_idx,
                target_idx=engine_action.target_idx,
            )
        elif isinstance(engine_action, UsePotion):
            return CombatAction(
                action_type="use_potion",
                potion_idx=engine_action.potion_idx,
                target_idx=engine_action.target_idx,
            )
        elif isinstance(engine_action, EndTurn):
            return CombatAction(action_type="end_turn")
        else:
            return CombatAction(action_type="end_turn")
