"""
Game Runner - Main orchestrator for a Slay the Spire run.

This module provides the GameRunner class that manages a complete game from
seed to victory/defeat. It handles:
- Run initialization with seed and ascension
- Map navigation and room dispatch
- Room handlers (combat, event, shop, rest, treasure, boss)
- Game state tracking and decision logging
- Abstract action interface for bot/RL integration

Usage:
    runner = GameRunner(seed="TEST123", ascension=20)
    runner.run()  # Full run with random actions
    # OR
    runner.run_to_floor(5)  # Run first 5 floors
    # OR manual control:
    while not runner.game_over:
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum, auto
from typing import List, Dict, Optional, Union, Any, Tuple
import itertools
import random

from .state.run import (
    RunState,
    create_watcher_run,
    create_ironclad_run,
    create_silent_run,
    create_defect_run,
)
from .state.rng import Random, seed_to_long
from .state.combat import CombatState, EnemyCombatState, create_combat, create_enemy
from .generation.map import (
    MapRoomNode, RoomType, MapGenerator, MapGeneratorConfig,
    get_map_seed_offset, map_to_string, generate_act4_map
)
from .combat_engine import (
    CombatEngine, CombatResult, PlayCard, EndTurn, UsePotion,
    CombatPhase as CombatEnginePhase, create_simple_combat,
    create_combat_from_enemies,
)
from .handlers.combat import create_enemies_from_encounter, ENCOUNTER_TABLE
from .content.cards import get_card, CardTarget, CardType, CardColor, ALL_CARDS
from .generation.encounters import (
    generate_exordium_encounters, generate_city_encounters,
    generate_beyond_encounters, generate_ending_encounters,
)
from .handlers.shop_handler import (
    ShopHandler, ShopState, ShopAction as ShopHandlerAction,
    ShopActionType, ShopResult,
)
from .handlers.reward_handler import (
    RewardHandler, CombatRewards, BossRelicChoices,
    ClaimGoldAction, ClaimPotionAction, SkipPotionAction,
    PickCardAction, SkipCardAction, SingingBowlAction,
    ClaimRelicAction, ClaimEmeraldKeyAction, SkipEmeraldKeyAction,
    PickBossRelicAction, ProceedFromRewardsAction,
    RewardAction as RewardHandlerAction,
)
from .handlers.rooms import (
    RestHandler, TreasureHandler, ChestType, ChestReward,
    NeowHandler, NeowBlessing, NeowBlessingType, NeowDrawbackType, NeowResult,
)
from .handlers.event_handler import (
    EventHandler as NewEventHandler, EventState, EventPhase, EventChoiceResult,
)
from .registry import execute_relic_triggers


# =============================================================================
# Game Phase Enumeration
# =============================================================================

class GamePhase(Enum):
    """Current phase of the game."""
    MAP_NAVIGATION = auto()      # Choosing next room on map
    COMBAT = auto()              # In a combat encounter
    COMBAT_REWARDS = auto()      # Choosing rewards after combat
    EVENT = auto()               # Making event choice
    SHOP = auto()                # In shop menu
    REST = auto()                # At rest site (choosing action)
    TREASURE = auto()            # At treasure room
    BOSS_REWARDS = auto()        # Choosing boss relic after boss kill
    NEOW = auto()                # Neow's blessing choice (start of run)
    RUN_COMPLETE = auto()        # Game ended (win or loss)


# =============================================================================
# Action Types
# =============================================================================

@dataclass(frozen=True)
class PathAction:
    """Choose a path on the map."""
    node_index: int  # Index into available paths


@dataclass(frozen=True)
class NeowAction:
    """Choose Neow blessing."""
    choice_index: int


@dataclass(frozen=True)
class CombatAction:
    """Action during combat (delegated to combat system)."""
    action_type: str  # "play_card", "use_potion", "end_turn"
    card_idx: int = -1
    target_idx: int = -1
    potion_idx: int = -1


@dataclass(frozen=True)
class RewardAction:
    """Choose a reward after combat."""
    reward_type: str  # "card", "gold", "potion", "relic", "skip"
    choice_index: int = 0  # For card rewards, which card to pick


@dataclass(frozen=True)
class EventAction:
    """Choose an event option."""
    choice_index: int


@dataclass(frozen=True)
class ShopAction:
    """Action in shop."""
    action_type: str  # "buy_card", "buy_relic", "buy_potion", "remove_card", "leave"
    item_index: int = 0


@dataclass(frozen=True)
class RestAction:
    """Action at rest site."""
    action_type: str  # "rest", "upgrade", "dig", "lift", "recall", "ruby_key"
    card_index: int = -1  # For upgrade, which card to upgrade


@dataclass(frozen=True)
class TreasureAction:
    """Action at treasure room."""
    action_type: str  # "take_relic", "sapphire_key"


@dataclass(frozen=True)
class BossRewardAction:
    """Choose boss relic."""
    relic_index: int


GameAction = Union[
    PathAction, NeowAction, CombatAction, RewardAction,
    EventAction, ShopAction, RestAction, TreasureAction, BossRewardAction
]

# JSON action dictionary shape for agent-facing API.
ActionDict = Dict[str, Any]


# =============================================================================
# Decision Log Entry
# =============================================================================

@dataclass
class DecisionLogEntry:
    """Record of a decision made during the run."""
    floor: int
    act: int
    phase: GamePhase
    action_taken: GameAction
    available_actions: List[GameAction]
    state_snapshot: Dict[str, Any]  # Relevant state at time of decision
    result: Optional[Dict[str, Any]] = None  # Outcome of the action


@dataclass
class PendingSelectionContext:
    """State for a required follow-up selection action."""
    selection_type: str  # "card_select" | "stance_select"
    source_action_type: str
    pile: str
    min_cards: int
    max_cards: int
    candidate_indices: List[int] = field(default_factory=list)
    candidate_values: List[str] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)
    parent_action_id: str = ""


# =============================================================================
# Game Runner
# =============================================================================

class GameRunner:
    """
    Main orchestrator for a Slay the Spire run.

    Manages the full game loop from initialization to completion,
    providing an interface for both manual play and bot/RL integration.
    """

    def __init__(
        self,
        seed: Union[str, int],
        ascension: int = 20,
        character: str = "Watcher",
        skip_neow: bool = True,  # For now, skip Neow blessing
        verbose: bool = True,
    ):
        """
        Initialize a new game run.

        Args:
            seed: Seed string (e.g., "TEST123") or numeric seed
            ascension: Ascension level (0-20)
            character: Character class ("Watcher", "Ironclad", "Silent", "Defect")
            skip_neow: If True, skip Neow blessing phase
            verbose: If True, print game events
        """
        self.verbose = verbose
        self.skip_neow = skip_neow

        # Parse seed
        if isinstance(seed, str):
            self.seed_string = seed.upper()
            self.seed = seed_to_long(self.seed_string)
        else:
            self.seed = seed
            self.seed_string = str(seed)

        # Create run state
        character_key = character.strip().lower()
        if character_key == "watcher":
            factory = create_watcher_run
        elif character_key == "ironclad":
            factory = create_ironclad_run
        elif character_key == "silent":
            factory = create_silent_run
        elif character_key == "defect":
            factory = create_defect_run
        else:
            raise ValueError(f"Unknown character: {character}")

        self.run_state = factory(self.seed_string, ascension)

        # Game status flags
        self.game_over = False
        self.game_won = False
        self.game_lost = False

        # Current phase
        self.phase = GamePhase.NEOW if not skip_neow else GamePhase.MAP_NAVIGATION

        # Decision log
        self.decision_log: List[DecisionLogEntry] = []

        # Combat state (when in combat)
        self.current_combat = None
        self.pending_rewards: List[Dict] = []  # Legacy format for backwards compatibility

        # Combat rewards (new system using RewardHandler)
        self.current_rewards: Optional[CombatRewards] = None
        self.current_room_type: str = "monster"  # "monster", "elite", "boss"
        self.is_burning_elite: bool = False  # Track if current elite is burning
        self.pending_selection: Optional[PendingSelectionContext] = None

        # Current event state (when in event)
        self.current_event: Optional[Dict] = None  # Legacy format
        self.event_handler = NewEventHandler()  # New complete event system
        self.current_event_state: Optional[EventState] = None  # New event state

        # Current shop state (when in shop)
        self.current_shop: Optional[ShopState] = None

        # Neow blessing state (when at start of run)
        self.neow_blessings: Optional[List[NeowBlessing]] = None
        self.neow_pending_result: Optional[NeowResult] = None  # For blessings requiring card selection

        # Treasure room state
        self.current_chest_type: Optional[ChestType] = None
        self.current_chest_reward: Optional[ChestReward] = None

        # Boss fight tracking: after boss combat, go to COMBAT_REWARDS first
        self._boss_fight_pending_boss_rewards: bool = False

        # Encounter tables (generated per act)
        self._monster_list: List[str] = []
        self._elite_list: List[str] = []
        self._boss_name: str = ""
        self._monster_index: int = 0
        self._elite_index: int = 0

        # RNG instances for different purposes
        self._init_rng()

        # Generate encounter tables for Act 1
        self._generate_encounter_tables()

        if self.verbose:
            self._log(f"=== Game Started ===")
            self._log(f"Seed: {self.seed_string}")
            self._log(f"Ascension: {ascension}")
            self._log(f"Character: {character}")
            self._log(f"Starting HP: {self.run_state.current_hp}/{self.run_state.max_hp}")
            self._log(f"Starting Gold: {self.run_state.gold}")

    def _init_rng(self):
        """Initialize RNG streams for different game systems."""
        counters = self.run_state.rng_counters or {}
        def counter(name: str) -> int:
            return int(counters.get(name, 0))

        # AI RNG for enemy decisions
        self.ai_rng = Random(self.seed + 1000, counter("ai"))
        # Monster HP RNG
        self.hp_rng = Random(self.seed + 2000, counter("monster_hp"))
        # Event RNG
        self.event_rng = Random(self.seed + 3000, counter("event"))
        # Misc RNG
        self.misc_rng = Random(self.seed + 4000, counter("misc"))
        # Card reward RNG
        self.card_rng = Random(self.seed + 5000, counter("card"))
        # Treasure/gold RNG
        self.treasure_rng = Random(self.seed + 6000, counter("treasure"))
        # Potion drop RNG
        self.potion_rng = Random(self.seed + 7000, counter("potion"))
        # Relic RNG
        self.relic_rng = Random(self.seed + 8000, counter("relic"))
        # Merchant RNG (for shop prices and generation)
        self.merchant_rng = Random(self.seed + 9000, counter("merchant"))
        # Neow RNG (for blessing selection)
        self.neow_rng = Random(self.seed + 10000, counter("neow"))
        # Monster encounter selection RNG
        self.monster_rng = Random(self.seed + 100, counter("monster"))
        # Deck shuffle RNG
        self.shuffle_rng = Random(self.seed + 200, counter("shuffle"))
        # Random card effects RNG
        self.card_random_rng = Random(self.seed + 300, counter("card_random"))

        self._sync_rng_counters()

    def _sync_rng_counters(self) -> None:
        """Persist RNG counters into the run state."""
        self.run_state.sync_rng_counters({
            "ai": self.ai_rng.counter,
            "monster_hp": self.hp_rng.counter,
            "event": self.event_rng.counter,
            "misc": self.misc_rng.counter,
            "card": self.card_rng.counter,
            "treasure": self.treasure_rng.counter,
            "potion": self.potion_rng.counter,
            "relic": self.relic_rng.counter,
            "merchant": self.merchant_rng.counter,
            "neow": self.neow_rng.counter,
            "monster": self.monster_rng.counter,
            "shuffle": self.shuffle_rng.counter,
            "card_random": self.card_random_rng.counter,
        })

    def _apply_act_transition_rng_snaps(self) -> None:
        """Apply cardRng snapping when transitioning to a new act."""
        c = self.card_rng.counter
        if 0 < c < 250:
            self.card_rng.counter = 250
        elif 250 < c < 500:
            self.card_rng.counter = 500
        elif 500 < c < 750:
            self.card_rng.counter = 750
    def _generate_encounter_tables(self):
        """Generate encounter tables for the current act using monsterRng."""
        act = self.run_state.act
        if act == 1:
            monsters, elites, boss = generate_exordium_encounters(self.monster_rng)
        elif act == 2:
            monsters, elites, boss = generate_city_encounters(self.monster_rng)
        elif act == 3:
            monsters, elites, boss = generate_beyond_encounters(self.monster_rng)
        elif act == 4:
            monsters, elites, boss = generate_ending_encounters()
        else:
            monsters, elites, boss = ["Jaw Worm"], ["Lagavulin"], "Slime Boss"

        self._monster_list = monsters
        self._elite_list = elites
        self._boss_name = boss
        self._monster_index = 0
        self._elite_index = 0
        self._sync_rng_counters()

    def _log(self, message: str):
        """Print a message if verbose mode is enabled."""
        if self.verbose:
            print(message)

    # =========================================================================
    # Main Game Loop
    # =========================================================================

    def run(self) -> Dict[str, Any]:
        """
        Run the game to completion using random action selection.

        Returns:
            Dict with run statistics
        """
        while not self.game_over:
            actions = self.get_available_actions()
            if not actions:
                self._log("No actions available - ending run")
                break

            # Random action selection
            action = random.choice(actions)
            self.take_action(action)

        return self.get_run_statistics()

    def run_to_floor(self, target_floor: int) -> Dict[str, Any]:
        """
        Run the game until reaching a specific floor (or game ends).

        Args:
            target_floor: Floor number to stop at

        Returns:
            Dict with current run statistics
        """
        while not self.game_over and self.run_state.floor < target_floor:
            actions = self.get_available_actions()
            if not actions:
                break

            # Random action selection
            action = random.choice(actions)
            self.take_action(action)

        return self.get_run_statistics()

    # =========================================================================
    # Action Interface
    # =========================================================================

    def _phase_to_action_phase(self, phase: Optional[GamePhase] = None) -> str:
        """Map GamePhase to JSON action phase string."""
        phase = phase or self.phase
        mapping = {
            GamePhase.MAP_NAVIGATION: "map",
            GamePhase.COMBAT: "combat",
            GamePhase.COMBAT_REWARDS: "reward",
            GamePhase.BOSS_REWARDS: "reward",
            GamePhase.EVENT: "event",
            GamePhase.SHOP: "shop",
            GamePhase.REST: "rest",
            GamePhase.TREASURE: "treasure",
            GamePhase.NEOW: "neow",
            GamePhase.RUN_COMPLETE: "complete",
        }
        return mapping.get(phase, "unknown")

    def _make_action_id(self, action_type: str, params: Dict[str, Any]) -> str:
        """Deterministic action id from type + sorted params."""
        parts = [action_type]
        for key in sorted(params.keys()):
            value = params[key]
            if isinstance(value, list):
                value_str = ",".join(str(v) for v in value)
            else:
                value_str = str(value)
            parts.append(f"{key}={value_str}")
        return "|".join(parts)

    def _get_combat_potion_id(self, potion_slot: int) -> Optional[str]:
        """Get potion id in a combat slot, if valid."""
        if not self.current_combat:
            return None
        potions = self.current_combat.state.potions
        if potion_slot < 0 or potion_slot >= len(potions):
            return None
        potion_id = potions[potion_slot]
        return potion_id or None

    def _build_discovery_offer(self, potion_id: str, count: int = 3) -> List[str]:
        """Build deterministic card offers for discovery-style potions."""
        if potion_id == "AttackPotion":
            pool = [
                cid for cid, card in ALL_CARDS.items()
                if card.card_type == CardType.ATTACK and card.color != CardColor.COLORLESS
            ]
        elif potion_id == "SkillPotion":
            pool = [
                cid for cid, card in ALL_CARDS.items()
                if card.card_type == CardType.SKILL and card.color != CardColor.COLORLESS
            ]
        elif potion_id == "PowerPotion":
            pool = [
                cid for cid, card in ALL_CARDS.items()
                if card.card_type == CardType.POWER and card.color != CardColor.COLORLESS
            ]
        else:
            pool = [cid for cid, card in ALL_CARDS.items() if card.color == CardColor.COLORLESS]

        if not pool:
            return []

        # Deterministic, unique sampling via card RNG stream.
        chosen: List[str] = []
        pool_local = list(pool)
        rng = getattr(self.current_combat.state, "card_rng", None) or self.card_rng
        while pool_local and len(chosen) < count:
            idx = rng.random(len(pool_local) - 1) if rng else 0
            chosen.append(pool_local.pop(idx))
        return chosen

    def _build_pending_selection_for_potion(
        self,
        potion_slot: int,
        potion_id: str,
        parent_action_id: str,
    ) -> Optional[PendingSelectionContext]:
        """Create selection context for potions that require additional input."""
        state = self.current_combat.state if self.current_combat else None
        if state is None:
            return None

        has_sacred_bark = state.has_relic("SacredBark")

        if potion_id in ("AttackPotion", "SkillPotion", "PowerPotion", "ColorlessPotion"):
            offer = self._build_discovery_offer(potion_id, count=3)
            return PendingSelectionContext(
                selection_type="card_select",
                source_action_type="use_potion",
                pile="offer",
                min_cards=1,
                max_cards=1,
                candidate_indices=list(range(len(offer))),
                metadata={"potion_slot": potion_slot, "potion_id": potion_id, "offer_cards": offer},
                parent_action_id=parent_action_id,
            )

        if potion_id == "LiquidMemories":
            discard_count = len(state.discard_pile)
            max_cards = min(2 if has_sacred_bark else 1, discard_count)
            min_cards = 1 if discard_count > 0 else 0
            return PendingSelectionContext(
                selection_type="card_select",
                source_action_type="use_potion",
                pile="discard",
                min_cards=min_cards,
                max_cards=max_cards,
                candidate_indices=list(range(len(state.discard_pile))),
                metadata={"potion_slot": potion_slot, "potion_id": potion_id},
                parent_action_id=parent_action_id,
            )

        if potion_id in ("GamblersBrew", "ElixirPotion"):
            return PendingSelectionContext(
                selection_type="card_select",
                source_action_type="use_potion",
                pile="hand",
                min_cards=0,
                max_cards=len(state.hand),
                candidate_indices=list(range(len(state.hand))),
                metadata={"potion_slot": potion_slot, "potion_id": potion_id},
                parent_action_id=parent_action_id,
            )

        if potion_id == "StancePotion":
            return PendingSelectionContext(
                selection_type="stance_select",
                source_action_type="use_potion",
                pile="",
                min_cards=1,
                max_cards=1,
                candidate_values=["Calm", "Wrath"],
                metadata={"potion_slot": potion_slot, "potion_id": potion_id},
                parent_action_id=parent_action_id,
            )

        return None

    def _selection_context_actions(self) -> List[ActionDict]:
        """Emit explicit follow-up actions for pending selection state."""
        if not self.pending_selection:
            return []
        ctx = self.pending_selection
        actions: List[ActionDict] = []

        if ctx.selection_type == "stance_select":
            for stance in ctx.candidate_values:
                params = {"stance": stance, "parent_action_id": ctx.parent_action_id}
                actions.append({
                    "id": self._make_action_id("select_stance", params),
                    "type": "select_stance",
                    "label": f"Select stance: {stance}",
                    "params": params,
                    "phase": self._phase_to_action_phase(),
                })
            return actions

        candidates = list(ctx.candidate_indices)
        if not candidates and ctx.min_cards > 0:
            return []

        # Exhaustive only for small bounded selections; otherwise provide practical candidates.
        if ctx.max_cards <= 2:
            for size in range(ctx.min_cards, ctx.max_cards + 1):
                for combo in itertools.combinations(candidates, size):
                    params = {
                        "pile": ctx.pile,
                        "card_indices": list(combo),
                        "min_cards": ctx.min_cards,
                        "max_cards": ctx.max_cards,
                        "parent_action_id": ctx.parent_action_id,
                    }
                    actions.append({
                        "id": self._make_action_id("select_cards", params),
                        "type": "select_cards",
                        "label": f"Select {ctx.pile} cards: {list(combo)}",
                        "params": params,
                        "phase": self._phase_to_action_phase(),
                    })
            return actions

        # Hand-wide selections (Gamblers/Elixir): include no-op, singles, and full-set candidates.
        representative_sets: List[List[int]] = [[]]
        representative_sets.extend([[idx] for idx in candidates])
        if candidates:
            representative_sets.append(list(candidates))

        for selected in representative_sets:
            if len(selected) < ctx.min_cards or len(selected) > ctx.max_cards:
                continue
            params = {
                "pile": ctx.pile,
                "card_indices": selected,
                "min_cards": ctx.min_cards,
                "max_cards": ctx.max_cards,
                "parent_action_id": ctx.parent_action_id,
            }
            actions.append({
                "id": self._make_action_id("select_cards", params),
                "type": "select_cards",
                "label": f"Select {ctx.pile} cards: {selected}",
                "params": params,
                "phase": self._phase_to_action_phase(),
            })
        return actions

    def _apply_pending_selection(self, action_dict: ActionDict) -> Dict[str, Any]:
        """Execute pending selection and resolve the parent potion action."""
        if not self.pending_selection or not self.current_combat:
            return {"success": False, "error": "No pending selection"}

        ctx = self.pending_selection
        state = self.current_combat.state
        params = action_dict.get("params", {}) or {}
        action_type = action_dict.get("type")
        potion_slot = int(ctx.metadata.get("potion_slot", -1))
        potion_id = ctx.metadata.get("potion_id")

        if action_type == "select_stance":
            stance = str(params.get("stance", ""))
            if stance not in ("Calm", "Wrath"):
                return {"success": False, "error": "Invalid stance selection"}
            # Consume potion then change stance with full trigger path.
            if 0 <= potion_slot < len(state.potions):
                state.potions[potion_slot] = ""
            self.current_combat._change_stance(self.current_combat._parse_stance(stance))
            execute_relic_triggers("onUsePotion", state, {"potion": potion_id})
            self.pending_selection = None
            return {"success": True, "data": {"potion": potion_id, "selected_stance": stance}}

        if action_type != "select_cards":
            return {"success": False, "error": "Expected select_cards/select_stance action"}

        card_indices = [int(i) for i in params.get("card_indices", [])]
        if len(card_indices) < ctx.min_cards or len(card_indices) > ctx.max_cards:
            return {"success": False, "error": "Invalid number of selected cards"}

        # Consume potion before applying side effects.
        if 0 <= potion_slot < len(state.potions):
            state.potions[potion_slot] = ""

        if potion_id in ("AttackPotion", "SkillPotion", "PowerPotion", "ColorlessPotion"):
            offer_cards = list(ctx.metadata.get("offer_cards", []))
            if not card_indices:
                return {"success": False, "error": "Must select one offered card"}
            selected_idx = card_indices[0]
            if selected_idx < 0 or selected_idx >= len(offer_cards):
                return {"success": False, "error": "Invalid offered card index"}
            chosen_card = offer_cards[selected_idx]
            copies = 2 if state.has_relic("SacredBark") else 1
            added = 0
            for _ in range(copies):
                if len(state.hand) >= 10:
                    break
                state.hand.append(chosen_card)
                state.card_costs[chosen_card] = 0
                added += 1
            execute_relic_triggers("onUsePotion", state, {"potion": potion_id})
            self.pending_selection = None
            return {"success": True, "data": {"potion": potion_id, "cards_added": added, "card": chosen_card}}

        if potion_id == "LiquidMemories":
            moved: List[str] = []
            for idx in sorted(card_indices, reverse=True):
                if idx < 0 or idx >= len(state.discard_pile):
                    continue
                if len(state.hand) >= 10:
                    break
                card_id = state.discard_pile.pop(idx)
                state.hand.append(card_id)
                state.card_costs[card_id] = 0
                moved.append(card_id)
            execute_relic_triggers("onUsePotion", state, {"potion": potion_id})
            self.pending_selection = None
            return {"success": True, "data": {"potion": potion_id, "cards_moved": moved}}

        if potion_id == "GamblersBrew":
            discarded: List[str] = []
            for idx in sorted(card_indices, reverse=True):
                if idx < 0 or idx >= len(state.hand):
                    continue
                discarded.append(state.hand.pop(idx))
            state.discard_pile.extend(discarded)
            if discarded:
                self.current_combat._draw_cards(len(discarded))
            execute_relic_triggers("onUsePotion", state, {"potion": potion_id})
            self.pending_selection = None
            return {"success": True, "data": {"potion": potion_id, "cards_discarded": list(reversed(discarded))}}

        if potion_id == "ElixirPotion":
            exhausted: List[str] = []
            for idx in sorted(card_indices, reverse=True):
                if idx < 0 or idx >= len(state.hand):
                    continue
                exhausted.append(state.hand.pop(idx))
            state.exhaust_pile.extend(exhausted)
            execute_relic_triggers("onUsePotion", state, {"potion": potion_id})
            self.pending_selection = None
            return {"success": True, "data": {"potion": potion_id, "cards_exhausted": list(reversed(exhausted))}}

        self.pending_selection = None
        return {"success": False, "error": f"Unsupported selection potion: {potion_id}"}

    def _action_to_dict(self, action: GameAction) -> ActionDict:
        """Convert a GameAction dataclass into a JSON action dict."""
        phase = self._phase_to_action_phase()
        action_type = "unknown"
        params: Dict[str, Any] = {}
        label = ""

        if isinstance(action, PathAction):
            action_type = "path_choice"
            params = {"node_index": action.node_index}
            label = f"Path to node {action.node_index}"
        elif isinstance(action, NeowAction):
            action_type = "neow_choice"
            params = {"choice_index": action.choice_index}
            label = f"Neow choice {action.choice_index}"
        elif isinstance(action, CombatAction):
            action_type = action.action_type
            if action.action_type == "play_card":
                params = {"card_index": action.card_idx}
                if action.target_idx >= 0:
                    params["target_index"] = action.target_idx
                label = f"Play card {action.card_idx}"
            elif action.action_type == "use_potion":
                params = {"potion_slot": action.potion_idx}
                if action.target_idx >= 0:
                    params["target_index"] = action.target_idx
                label = f"Use potion {action.potion_idx}"
            elif action.action_type == "end_turn":
                params = {}
                label = "End turn"
        elif isinstance(action, RewardAction):
            if action.reward_type == "card":
                card_reward_idx = action.choice_index // 100
                card_idx = action.choice_index % 100
                action_type = "pick_card"
                params = {"card_reward_index": card_reward_idx, "card_index": card_idx}
                label = f"Pick card {card_idx} (reward {card_reward_idx})"
            elif action.reward_type == "skip_card":
                action_type = "skip_card"
                params = {"card_reward_index": action.choice_index}
                label = f"Skip card reward {action.choice_index}"
            elif action.reward_type == "singing_bowl":
                action_type = "singing_bowl"
                params = {"card_reward_index": action.choice_index}
                label = f"Singing Bowl (reward {action.choice_index})"
            elif action.reward_type == "gold":
                action_type = "claim_gold"
                params = {}
                label = "Claim gold"
            elif action.reward_type == "potion":
                action_type = "claim_potion"
                params = {"potion_reward_index": 0}
                label = "Claim potion"
            elif action.reward_type == "skip_potion":
                action_type = "skip_potion"
                params = {"potion_reward_index": 0}
                label = "Skip potion"
            elif action.reward_type == "relic":
                action_type = "claim_relic"
                params = {"relic_reward_index": 0}
                label = "Claim relic"
            elif action.reward_type == "emerald_key":
                action_type = "claim_emerald_key"
                params = {}
                label = "Claim emerald key"
            elif action.reward_type == "skip_emerald_key":
                action_type = "skip_emerald_key"
                params = {}
                label = "Skip emerald key"
            elif action.reward_type == "proceed":
                action_type = "proceed_from_rewards"
                params = {}
                label = "Proceed from rewards"
        elif isinstance(action, EventAction):
            action_type = "event_choice"
            params = {"choice_index": action.choice_index}
            label = f"Event choice {action.choice_index}"
        elif isinstance(action, ShopAction):
            if action.action_type == "buy_colored_card":
                action_type = "buy_card"
                params = {"item_index": action.item_index, "card_pool": "colored"}
                label = f"Buy colored card {action.item_index}"
            elif action.action_type == "buy_colorless_card":
                action_type = "buy_card"
                params = {"item_index": action.item_index, "card_pool": "colorless"}
                label = f"Buy colorless card {action.item_index}"
            elif action.action_type == "buy_relic":
                action_type = "buy_relic"
                params = {"item_index": action.item_index}
                label = f"Buy relic {action.item_index}"
            elif action.action_type == "buy_potion":
                action_type = "buy_potion"
                params = {"item_index": action.item_index}
                label = f"Buy potion {action.item_index}"
            elif action.action_type == "remove_card":
                action_type = "remove_card"
                params = {"card_index": action.item_index}
                label = f"Remove card {action.item_index}"
            elif action.action_type == "leave":
                action_type = "leave_shop"
                params = {}
                label = "Leave shop"
        elif isinstance(action, RestAction):
            if action.action_type == "rest":
                action_type = "rest"
                params = {}
                label = "Rest"
            elif action.action_type == "upgrade":
                action_type = "smith"
                params = {"card_index": action.card_index}
                label = f"Smith card {action.card_index}"
            elif action.action_type == "dig":
                action_type = "dig"
                params = {}
                label = "Dig"
            elif action.action_type == "lift":
                action_type = "lift"
                params = {}
                label = "Lift"
            elif action.action_type == "toke":
                action_type = "toke"
                params = {"card_index": action.card_index}
                label = f"Toke card {action.card_index}"
            elif action.action_type == "recall":
                action_type = "recall"
                params = {}
                label = "Recall"
            elif action.action_type == "ruby_key":
                action_type = "recall"
                params = {}
                label = "Recall ruby key"
        elif isinstance(action, TreasureAction):
            if action.action_type == "take_relic":
                action_type = "take_relic"
                params = {}
                label = "Take relic"
            elif action.action_type == "sapphire_key":
                action_type = "sapphire_key"
                params = {}
                label = "Take sapphire key"
        elif isinstance(action, BossRewardAction):
            action_type = "pick_boss_relic"
            params = {"relic_index": action.relic_index}
            label = f"Pick boss relic {action.relic_index}"

        action_id = self._make_action_id(action_type, params)
        return {
            "id": action_id,
            "type": action_type,
            "label": label,
            "params": params,
            "phase": phase,
        }

    def _dict_to_action(self, action_dict: ActionDict) -> GameAction:
        """Convert JSON action dict into a GameAction dataclass."""
        action_type = action_dict.get("type")
        params = action_dict.get("params", {}) or {}

        if action_type == "path_choice":
            return PathAction(node_index=int(params["node_index"]))
        if action_type == "neow_choice":
            return NeowAction(choice_index=int(params["choice_index"]))
        if action_type == "play_card":
            return CombatAction(
                action_type="play_card",
                card_idx=int(params["card_index"]),
                target_idx=int(params.get("target_index", -1)),
            )
        if action_type == "use_potion":
            return CombatAction(
                action_type="use_potion",
                potion_idx=int(params["potion_slot"]),
                target_idx=int(params.get("target_index", -1)),
            )
        if action_type == "end_turn":
            return CombatAction(action_type="end_turn")
        if action_type == "event_choice":
            return EventAction(choice_index=int(params["choice_index"]))
        if action_type == "pick_card":
            card_reward_index = int(params["card_reward_index"])
            card_index = int(params["card_index"])
            choice_index = card_reward_index * 100 + card_index
            return RewardAction(reward_type="card", choice_index=choice_index)
        if action_type == "skip_card":
            card_reward_index = int(params.get("card_reward_index", 0))
            return RewardAction(reward_type="skip_card", choice_index=card_reward_index)
        if action_type == "singing_bowl":
            card_reward_index = int(params.get("card_reward_index", 0))
            return RewardAction(reward_type="singing_bowl", choice_index=card_reward_index)
        if action_type == "claim_gold":
            return RewardAction(reward_type="gold", choice_index=0)
        if action_type == "claim_potion":
            return RewardAction(reward_type="potion", choice_index=0)
        if action_type == "skip_potion":
            return RewardAction(reward_type="skip_potion", choice_index=0)
        if action_type == "claim_relic":
            return RewardAction(reward_type="relic", choice_index=0)
        if action_type == "claim_emerald_key":
            return RewardAction(reward_type="emerald_key", choice_index=0)
        if action_type == "skip_emerald_key":
            return RewardAction(reward_type="skip_emerald_key", choice_index=0)
        if action_type == "proceed_from_rewards":
            return RewardAction(reward_type="proceed", choice_index=0)
        if action_type == "pick_boss_relic":
            return BossRewardAction(relic_index=int(params["relic_index"]))
        if action_type == "skip_boss_relic":
            return BossRewardAction(relic_index=-1)
        if action_type == "buy_card":
            pool = params.get("card_pool", "colored")
            action_type_value = "buy_colorless_card" if pool == "colorless" else "buy_colored_card"
            return ShopAction(action_type=action_type_value, item_index=int(params["item_index"]))
        if action_type == "buy_colored_card":
            return ShopAction(action_type="buy_colored_card", item_index=int(params["item_index"]))
        if action_type == "buy_colorless_card":
            return ShopAction(action_type="buy_colorless_card", item_index=int(params["item_index"]))
        if action_type == "buy_relic":
            return ShopAction(action_type="buy_relic", item_index=int(params["item_index"]))
        if action_type == "buy_potion":
            return ShopAction(action_type="buy_potion", item_index=int(params["item_index"]))
        if action_type == "remove_card":
            card_index = int(params["card_index"])
            return ShopAction(action_type="remove_card", item_index=card_index)
        if action_type == "leave_shop":
            return ShopAction(action_type="leave", item_index=0)
        if action_type == "leave":
            return ShopAction(action_type="leave", item_index=0)
        if action_type == "rest":
            return RestAction(action_type="rest")
        if action_type == "smith":
            return RestAction(action_type="upgrade", card_index=int(params["card_index"]))
        if action_type == "dig":
            return RestAction(action_type="dig")
        if action_type == "lift":
            return RestAction(action_type="lift")
        if action_type == "toke":
            return RestAction(action_type="toke", card_index=int(params["card_index"]))
        if action_type == "recall":
            return RestAction(action_type="ruby_key")
        if action_type == "take_relic":
            return TreasureAction(action_type="take_relic")
        if action_type == "sapphire_key":
            return TreasureAction(action_type="sapphire_key")

        raise ValueError(f"Unknown action type: {action_type}")

    def get_available_action_dicts(self) -> List[ActionDict]:
        """Get all valid actions as JSON-serializable dicts."""
        if self.pending_selection is not None:
            return self._selection_context_actions()

        actions = [self._action_to_dict(a) for a in self.get_available_actions()]

        # Enrich selection-required potion actions with explicit requirements.
        if self.phase == GamePhase.COMBAT and self.current_combat is not None:
            selection_potions = {
                "AttackPotion",
                "SkillPotion",
                "PowerPotion",
                "ColorlessPotion",
                "LiquidMemories",
                "GamblersBrew",
                "ElixirPotion",
                "StancePotion",
            }
            for action in actions:
                if action.get("type") != "use_potion":
                    continue
                params = action.get("params", {}) or {}
                potion_slot = int(params.get("potion_slot", -1))
                potion_id = self._get_combat_potion_id(potion_slot)
                if potion_id not in selection_potions:
                    continue
                if potion_id == "StancePotion":
                    action["requires"] = ["stance"]
                elif potion_id in ("LiquidMemories", "GamblersBrew", "ElixirPotion"):
                    action["requires"] = ["card_indices"]
                else:
                    action["requires"] = ["card_indices"]

        if self.phase == GamePhase.BOSS_REWARDS:
            unresolved = True
            if self.current_rewards and self.current_rewards.boss_relics:
                unresolved = not self.current_rewards.boss_relics.is_resolved
            if unresolved:
                skip_action = {
                    "type": "skip_boss_relic",
                    "label": "Skip boss relic",
                    "params": {},
                    "phase": self._phase_to_action_phase(),
                }
                skip_action["id"] = self._make_action_id(skip_action["type"], skip_action["params"])
                actions.append(skip_action)
        return actions

    def take_action_dict(self, action_dict: ActionDict) -> Dict[str, Any]:
        """Execute a JSON action dict via the dataclass adapter."""
        if self.pending_selection is not None:
            return self._apply_pending_selection(action_dict)

        # Intercept selection-required potions so caller can resolve via explicit actions.
        if self.phase == GamePhase.COMBAT and action_dict.get("type") == "use_potion":
            params = action_dict.get("params", {}) or {}
            if "potion_slot" not in params:
                return {"success": False, "error": "Missing potion_slot"}
            potion_slot = int(params.get("potion_slot", -1))
            potion_id = self._get_combat_potion_id(potion_slot)
            if potion_id is None:
                return {"success": False, "error": "Invalid or empty potion slot"}
            if (
                potion_id == "LiquidMemories"
                and self.current_combat
                and len(self.current_combat.state.discard_pile) == 0
            ):
                return {"success": False, "error": "No cards in discard pile"}

            selection_ctx = self._build_pending_selection_for_potion(
                potion_slot=potion_slot,
                potion_id=potion_id,
                parent_action_id=str(action_dict.get("id", "")),
            )
            if selection_ctx is not None:
                self.pending_selection = selection_ctx
                # If caller already provided selection params, execute immediately.
                if potion_id == "StancePotion" and "stance" in params:
                    selection_action = {
                        "type": "select_stance",
                        "params": {
                            "stance": params.get("stance"),
                            "parent_action_id": selection_ctx.parent_action_id,
                        },
                    }
                    return self._apply_pending_selection(selection_action)
                if "card_indices" in params:
                    selection_action = {
                        "type": "select_cards",
                        "params": {
                            "pile": selection_ctx.pile,
                            "card_indices": params.get("card_indices", []),
                            "min_cards": selection_ctx.min_cards,
                            "max_cards": selection_ctx.max_cards,
                            "parent_action_id": selection_ctx.parent_action_id,
                        },
                    }
                    return self._apply_pending_selection(selection_action)

                return {
                    "success": False,
                    "error": "Selection required",
                    "requires_selection": True,
                    "candidate_actions": self._selection_context_actions(),
                }

        try:
            action = self._dict_to_action(action_dict)
        except Exception as exc:  # invalid action dict
            return {"success": False, "error": str(exc)}

        try:
            success = self.take_action(action)
        except Exception as exc:
            return {"success": False, "error": str(exc)}

        if not success:
            return {"success": False, "error": "Invalid action for current state"}
        return {"success": True, "data": {}}

    def get_available_actions(self) -> List[GameAction]:
        """
        Get all valid actions for the current game state.

        Returns:
            List of valid GameAction objects
        """
        if self.game_over:
            return []

        actions = []
        if self.phase == GamePhase.NEOW:
            actions = self._get_neow_actions()
        elif self.phase == GamePhase.MAP_NAVIGATION:
            actions = self._get_path_actions()
        elif self.phase == GamePhase.COMBAT:
            actions = self._get_combat_actions()
        elif self.phase == GamePhase.COMBAT_REWARDS:
            actions = self._get_reward_actions()
        elif self.phase == GamePhase.EVENT:
            actions = self._get_event_actions()
        elif self.phase == GamePhase.SHOP:
            actions = self._get_shop_actions()
        elif self.phase == GamePhase.REST:
            actions = self._get_rest_actions()
        elif self.phase == GamePhase.TREASURE:
            actions = self._get_treasure_actions()
        elif self.phase == GamePhase.BOSS_REWARDS:
            actions = self._get_boss_reward_actions()

        # Fallback safety: prevent infinite loops with empty action lists
        if not actions:
            if self.phase == GamePhase.COMBAT_REWARDS:
                actions = [RewardAction(reward_type="proceed", choice_index=0)]
            elif self.phase == GamePhase.BOSS_REWARDS:
                actions = [BossRewardAction(relic_index=0)]
            elif self.phase == GamePhase.EVENT:
                # Force-leave the event
                self._log("No event actions available, auto-leaving event")
                self.current_event_state = None
                self.phase = GamePhase.MAP_NAVIGATION
                actions = self._get_path_actions()
            elif self.phase == GamePhase.SHOP:
                actions = [ShopAction(action_type="leave")]
            elif self.phase == GamePhase.TREASURE:
                actions = [TreasureAction(action_type="take_relic")]

        return actions

    def take_action(self, action: Union[GameAction, ActionDict]) -> bool:
        """
        Execute an action and advance the game state.

        Args:
            action: The action to take

        Returns:
            True if action was valid and executed, False otherwise
        """
        if self.game_over:
            return False

        if isinstance(action, dict):
            try:
                action = self._dict_to_action(action)
            except (KeyError, TypeError, ValueError):
                return False

        # Log the decision
        available = self.get_available_actions()
        log_entry = DecisionLogEntry(
            floor=self.run_state.floor,
            act=self.run_state.act,
            phase=self.phase,
            action_taken=action,
            available_actions=available,
            state_snapshot=self._create_state_snapshot(),
        )

        # Execute based on action type
        success = False
        result = None

        if isinstance(action, PathAction):
            success, result = self._handle_path_action(action)
        elif isinstance(action, NeowAction):
            success, result = self._handle_neow_action(action)
        elif isinstance(action, CombatAction):
            success, result = self._handle_combat_action(action)
        elif isinstance(action, RewardAction):
            success, result = self._handle_reward_action(action)
        elif isinstance(action, EventAction):
            success, result = self._handle_event_action(action)
        elif isinstance(action, ShopAction):
            success, result = self._handle_shop_action(action)
        elif isinstance(action, RestAction):
            success, result = self._handle_rest_action(action)
        elif isinstance(action, TreasureAction):
            success, result = self._handle_treasure_action(action)
        elif isinstance(action, BossRewardAction):
            success, result = self._handle_boss_reward_action(action)

        log_entry.result = result
        self.decision_log.append(log_entry)

        return success

    # =========================================================================
    # Observation Interface
    # =========================================================================

    def get_observation(self) -> Dict[str, Any]:
        """Return a JSON-serializable observation for the current state."""
        # Ensure map exists for current act
        if self.run_state.get_current_map() is None:
            self.run_state.generate_map_for_act(self.run_state.act)

        return {
            "phase": self._phase_to_action_phase(),
            "run": self._build_run_observation(),
            "map": self._build_map_observation(),
            "combat": self._build_combat_observation() if self.phase == GamePhase.COMBAT else None,
            "event": self._build_event_observation() if self.phase == GamePhase.EVENT else None,
            "reward": self._build_reward_observation()
            if self.phase in (GamePhase.COMBAT_REWARDS, GamePhase.BOSS_REWARDS)
            else None,
            "shop": self._build_shop_observation() if self.phase == GamePhase.SHOP else None,
            "rest": self._build_rest_observation() if self.phase == GamePhase.REST else None,
            "treasure": self._build_treasure_observation() if self.phase == GamePhase.TREASURE else None,
        }

    def _build_run_observation(self) -> Dict[str, Any]:
        """Serialize run-level state."""
        return {
            "seed": self.seed_string,
            "ascension": self.run_state.ascension,
            "act": self.run_state.act,
            "floor": self.run_state.floor,
            "gold": self.run_state.gold,
            "current_hp": self.run_state.current_hp,
            "max_hp": self.run_state.max_hp,
            "deck": [
                {"id": c.id, "upgraded": c.upgraded, "misc_value": c.misc_value}
                for c in self.run_state.deck
            ],
            "relics": [
                {
                    "id": r.id,
                    "counter": r.counter,
                    "triggered_this_combat": r.triggered_this_combat,
                    "triggered_this_turn": r.triggered_this_turn,
                }
                for r in self.run_state.relics
            ],
            "potions": [s.potion_id for s in self.run_state.potion_slots],
            "keys": {
                "ruby": self.run_state.has_ruby_key,
                "emerald": self.run_state.has_emerald_key,
                "sapphire": self.run_state.has_sapphire_key,
            },
            "map_position": {
                "x": self.run_state.map_position.x,
                "y": self.run_state.map_position.y,
            },
            "rng_counters": dict(self.run_state.rng_counters),
        }

    def _build_map_observation(self) -> Dict[str, Any]:
        """Serialize full current-act map visibility."""
        current_map = self.run_state.get_current_map() or []
        nodes: List[Dict[str, Any]] = []
        edges: List[Dict[str, Any]] = []
        for row in current_map:
            for node in row:
                nodes.append({
                    "x": node.x,
                    "y": node.y,
                    "room_type": node.room_type.name if node.room_type else None,
                    "has_emerald_key": node.has_emerald_key,
                })
                for edge in node.edges:
                    edges.append({
                        "src_x": edge.src_x,
                        "src_y": edge.src_y,
                        "dst_x": edge.dst_x,
                        "dst_y": edge.dst_y,
                        "is_boss": edge.is_boss,
                    })

        available_paths = [
            {"x": node.x, "y": node.y, "room_type": node.room_type.name if node.room_type else None}
            for node in self.run_state.get_available_paths()
        ]

        visited_nodes = [
            {"act": act, "x": x, "y": y} for (act, x, y) in self.run_state.visited_nodes
        ]

        return {
            "act": self.run_state.act,
            "nodes": nodes,
            "edges": edges,
            "available_paths": available_paths,
            "visited_nodes": visited_nodes,
        }

    def _build_combat_observation(self) -> Optional[Dict[str, Any]]:
        """Serialize combat state."""
        if not self.current_combat:
            return None
        state = self.current_combat.state
        return {
            "player": {
                "hp": state.player.hp,
                "max_hp": state.player.max_hp,
                "block": state.player.block,
                "statuses": dict(state.player.statuses),
            },
            "energy": state.energy,
            "max_energy": state.max_energy,
            "stance": state.stance,
            "mantra": state.mantra,
            "hand": list(state.hand),
            "draw_pile": list(state.draw_pile),
            "discard_pile": list(state.discard_pile),
            "exhaust_pile": list(state.exhaust_pile),
            "enemies": [
                {
                    "id": e.id,
                    "hp": e.hp,
                    "max_hp": e.max_hp,
                    "block": e.block,
                    "statuses": dict(e.statuses),
                    "move_id": e.move_id,
                    "move_damage": e.move_damage,
                    "move_hits": e.move_hits,
                    "move_block": e.move_block,
                    "move_effects": dict(e.move_effects),
                }
                for e in state.enemies
            ],
            "turn": state.turn,
            "cards_played_this_turn": state.cards_played_this_turn,
            "attacks_played_this_turn": state.attacks_played_this_turn,
            "skills_played_this_turn": state.skills_played_this_turn,
            "powers_played_this_turn": state.powers_played_this_turn,
            "relic_counters": dict(state.relic_counters),
            "card_costs": dict(state.card_costs),
        }

    def _build_event_observation(self) -> Optional[Dict[str, Any]]:
        """Serialize event state and choices."""
        if not self.current_event_state:
            return None
        choices = self.event_handler.get_available_choices(self.current_event_state, self.run_state)
        choice_obs = []
        for choice in choices:
            requires_selection = (
                choice.requires_upgradable_cards or
                choice.requires_removable_cards or
                choice.requires_transformable_cards
            )
            selection_type = None
            if choice.requires_upgradable_cards:
                selection_type = "upgrade"
            elif choice.requires_removable_cards:
                selection_type = "remove"
            elif choice.requires_transformable_cards:
                selection_type = "transform"
            choice_obs.append({
                "choice_index": choice.index,
                "label": choice.text,
                "requires_card_selection": requires_selection,
                "card_selection_type": selection_type,
                "card_selection_count": 1 if requires_selection else 0,
            })

        return {
            "event_id": self.current_event_state.event_id,
            "phase": self.current_event_state.phase.name,
            "attempt_count": self.current_event_state.attempt_count,
            "hp_cost_modifier": self.current_event_state.hp_cost_modifier,
            "choices": choice_obs,
            "pending_rewards": dict(self.current_event_state.pending_rewards),
        }

    def _build_reward_observation(self) -> Optional[Dict[str, Any]]:
        """Serialize reward state."""
        rewards = self.current_rewards
        if not rewards:
            return None
        card_rewards = []
        for cr in rewards.card_rewards:
            card_rewards.append({
                "cards": [
                    {
                        "id": c.id,
                        "upgraded": c.upgraded,
                        "rarity": c.rarity.name if hasattr(c, "rarity") and c.rarity else None,
                    }
                    for c in cr.cards
                ],
                "claimed_index": cr.claimed_index,
                "skipped": cr.skipped,
                "singing_bowl_used": cr.singing_bowl_used,
            })

        boss_relics = None
        if rewards.boss_relics:
            boss_relics = {
                "relics": [r.id for r in rewards.boss_relics.relics],
                "chosen_index": rewards.boss_relics.chosen_index,
            }

        return {
            "gold": {"amount": rewards.gold.amount, "claimed": rewards.gold.claimed} if rewards.gold else None,
            "potion": {
                "id": rewards.potion.potion.id,
                "claimed": rewards.potion.claimed,
                "skipped": rewards.potion.skipped,
            } if rewards.potion else None,
            "card_rewards": card_rewards,
            "relic": {
                "id": rewards.relic.relic.id,
                "claimed": rewards.relic.claimed,
            } if rewards.relic else None,
            "second_relic": {
                "id": rewards.second_relic.relic.id,
                "claimed": rewards.second_relic.claimed,
            } if rewards.second_relic else None,
            "boss_relics": boss_relics,
            "emerald_key": {
                "available": rewards.emerald_key is not None,
                "claimed": rewards.emerald_key.claimed if rewards.emerald_key else False,
            },
        }

    def _build_shop_observation(self) -> Optional[Dict[str, Any]]:
        """Serialize shop inventory."""
        if not self.current_shop:
            return None
        return {
            "colored_cards": [
                {
                    "id": c.card.id,
                    "upgraded": c.card.upgraded,
                    "price": c.price,
                    "purchased": c.purchased,
                }
                for c in self.current_shop.colored_cards
            ],
            "colorless_cards": [
                {
                    "id": c.card.id,
                    "upgraded": c.card.upgraded,
                    "price": c.price,
                    "purchased": c.purchased,
                }
                for c in self.current_shop.colorless_cards
            ],
            "relics": [
                {"id": r.relic.id, "price": r.price, "purchased": r.purchased}
                for r in self.current_shop.relics
            ],
            "potions": [
                {"id": p.potion.id, "price": p.price, "purchased": p.purchased}
                for p in self.current_shop.potions
            ],
            "purge_cost": self.current_shop.purge_cost,
            "purge_available": self.current_shop.purge_available,
        }

    def _build_rest_observation(self) -> Dict[str, Any]:
        """Serialize rest site options."""
        actions = self._get_rest_actions()
        return {
            "available_actions": [self._action_to_dict(a)["type"] for a in actions],
        }

    def _build_treasure_observation(self) -> Dict[str, Any]:
        """Serialize treasure room options."""
        actions = self._get_treasure_actions()
        return {
            "available_actions": [self._action_to_dict(a)["type"] for a in actions],
        }

    # =========================================================================
    # Action Generators
    # =========================================================================

    def _get_neow_actions(self) -> List[GameAction]:
        """Get available Neow blessing choices."""
        # Generate blessings if not already generated
        if self.neow_blessings is None:
            # Check if this is the first run (no previous score)
            # For simplicity, we'll use is_first_run based on whether run_state has a previous_score
            is_first_run = not hasattr(self.run_state, 'previous_score') or self.run_state.previous_score == 0
            previous_score = getattr(self.run_state, 'previous_score', 0)
            self.neow_blessings = NeowHandler.get_blessing_options(
                self.neow_rng,
                previous_score=previous_score,
                is_first_run=is_first_run,
            )

        # Return actions for each blessing
        return [NeowAction(i) for i in range(len(self.neow_blessings))]

    def _get_path_actions(self) -> List[GameAction]:
        """Get available path choices on the map."""
        paths = self.run_state.get_available_paths()
        return [PathAction(i) for i in range(len(paths))]

    def _get_combat_actions(self) -> List[GameAction]:
        """Get available combat actions from the CombatEngine."""
        if self.current_combat is None:
            # No active combat engine, fallback to end turn
            return [CombatAction(action_type="end_turn")]

        actions: List[GameAction] = []
        engine_actions = self.current_combat.get_legal_actions()

        for action in engine_actions:
            if isinstance(action, PlayCard):
                actions.append(CombatAction(
                    action_type="play_card",
                    card_idx=action.card_idx,
                    target_idx=action.target_idx,
                ))
            elif isinstance(action, UsePotion):
                actions.append(CombatAction(
                    action_type="use_potion",
                    potion_idx=action.potion_idx,
                    target_idx=action.target_idx,
                ))
            elif isinstance(action, EndTurn):
                actions.append(CombatAction(action_type="end_turn"))

        return actions

    def _get_reward_actions(self) -> List[GameAction]:
        """Get available reward choices using the RewardHandler system."""
        actions = []

        # If no rewards generated yet, only allow proceed
        if self.current_rewards is None:
            actions.append(RewardAction(reward_type="proceed", choice_index=0))
            return actions

        rewards = self.current_rewards

        # Gold (auto-claimed, but include explicit action for clarity)
        if rewards.gold and not rewards.gold.claimed:
            actions.append(RewardAction(reward_type="gold", choice_index=0))

        # Potion rewards
        if rewards.potion and not rewards.potion.claimed and not rewards.potion.skipped:
            if self.run_state.count_empty_potion_slots() > 0:
                actions.append(RewardAction(reward_type="potion", choice_index=0))
            actions.append(RewardAction(reward_type="skip_potion", choice_index=0))

        # Card rewards (can have multiple with Prayer Wheel)
        for i, card_reward in enumerate(rewards.card_rewards):
            if not card_reward.is_resolved:
                # Can pick any card from this reward
                for j, card in enumerate(card_reward.cards):
                    # Encode card_reward_index and card_index: i * 100 + j
                    actions.append(RewardAction(reward_type="card", choice_index=i * 100 + j))

                # Can skip this card reward
                actions.append(RewardAction(reward_type="skip_card", choice_index=i))

                # Singing Bowl option (+2 max HP instead of card)
                if self.run_state.has_relic("Singing Bowl"):
                    actions.append(RewardAction(reward_type="singing_bowl", choice_index=i))

        # Relic rewards (elite only)
        if rewards.relic and not rewards.relic.claimed:
            actions.append(RewardAction(reward_type="relic", choice_index=0))

        # Emerald key (burning elite only)
        if rewards.emerald_key and not rewards.emerald_key.claimed:
            actions.append(RewardAction(reward_type="emerald_key", choice_index=0))
            actions.append(RewardAction(reward_type="skip_emerald_key", choice_index=0))

        # Can proceed if mandatory rewards are resolved
        if self._mandatory_rewards_resolved(rewards):
            actions.append(RewardAction(reward_type="proceed", choice_index=0))

        return actions

    def _mandatory_rewards_resolved(self, rewards: CombatRewards) -> bool:
        """Check if all mandatory rewards have been resolved (allowing proceed)."""
        # Gold auto-claims, doesn't block
        # Cards can be skipped
        for card_reward in rewards.card_rewards:
            if not card_reward.is_resolved:
                return False

        # Elite relic is mandatory (can't skip)
        if rewards.relic and not rewards.relic.claimed:
            return False

        # Potion and emerald key are optional
        return True

    def _get_event_actions(self) -> List[GameAction]:
        """Get available event choices using EventHandler."""
        if self.current_event_state is None:
            # No event active, shouldn't happen
            return [EventAction(0)]  # Default leave option

        # Get available choices from the event handler
        choices = self.event_handler.get_available_choices(
            self.current_event_state,
            self.run_state
        )

        # Convert to EventAction objects
        return [EventAction(choice.index) for choice in choices]

    def _get_shop_actions(self) -> List[GameAction]:
        """Get available shop actions using ShopHandler."""
        if self.current_shop is None:
            # No shop state, can only leave
            return [ShopAction(action_type="leave")]

        actions = []

        # Always can leave
        actions.append(ShopAction(action_type="leave"))

        gold = self.run_state.gold

        # Colored cards
        for shop_card in self.current_shop.get_available_colored_cards():
            if shop_card.price <= gold:
                actions.append(ShopAction(
                    action_type="buy_colored_card",
                    item_index=shop_card.slot_index,
                ))

        # Colorless cards
        for shop_card in self.current_shop.get_available_colorless_cards():
            if shop_card.price <= gold:
                actions.append(ShopAction(
                    action_type="buy_colorless_card",
                    item_index=shop_card.slot_index,
                ))

        # Relics
        for shop_relic in self.current_shop.get_available_relics():
            if shop_relic.price <= gold:
                actions.append(ShopAction(
                    action_type="buy_relic",
                    item_index=shop_relic.slot_index,
                ))

        # Potions (only if we have empty slots)
        if self.run_state.count_empty_potion_slots() > 0:
            for shop_potion in self.current_shop.get_available_potions():
                if shop_potion.price <= gold:
                    actions.append(ShopAction(
                        action_type="buy_potion",
                        item_index=shop_potion.slot_index,
                    ))

        # Card removal (one action per removable card)
        if self.current_shop.purge_available and self.current_shop.purge_cost <= gold:
            removable = self.run_state.get_removable_cards()
            for card_idx, card in removable:
                actions.append(ShopAction(
                    action_type="remove_card",
                    item_index=card_idx,  # card_index stored in item_index
                ))

        return actions

    def _get_rest_actions(self) -> List[GameAction]:
        """Get available rest site actions."""
        actions = []

        # Rest (heal) - blocked by Coffee Dripper or full HP
        if not self.run_state.has_relic("Coffee Dripper"):
            if self.run_state.current_hp < self.run_state.max_hp:
                actions.append(RestAction(action_type="rest"))

        # Upgrade (if have upgradeable cards)
        upgradeable = self.run_state.get_upgradeable_cards()
        for idx, card in upgradeable:
            actions.append(RestAction(action_type="upgrade", card_index=idx))

        # Dig (if have Shovel relic)
        if self.run_state.has_relic("Shovel"):
            actions.append(RestAction(action_type="dig"))

        # Lift (if have Girya relic with charges)
        if self.run_state.has_relic("Girya"):
            counter = self.run_state.get_relic_counter("Girya")
            if counter < 3:
                actions.append(RestAction(action_type="lift"))

        # Toke (if have Peace Pipe and have removable cards)
        if self.run_state.has_relic("Peace Pipe"):
            removable = self.run_state.get_removable_cards()
            for idx, card in removable:
                actions.append(RestAction(action_type="toke", card_index=idx))

        # Ruby key (if don't have it and in Act 3)
        if self.run_state.act == 3 and not self.run_state.has_ruby_key:
            actions.append(RestAction(action_type="ruby_key"))

        return actions

    def _get_treasure_actions(self) -> List[GameAction]:
        """Get available treasure room actions."""
        actions = []

        # Take relic
        actions.append(TreasureAction(action_type="take_relic"))

        # Sapphire key option (if Act 3 and don't have it)
        if self.run_state.act == 3 and not self.run_state.has_sapphire_key:
            actions.append(TreasureAction(action_type="sapphire_key"))

        return actions

    def _get_boss_reward_actions(self) -> List[GameAction]:
        """Get available boss relic choices using RewardHandler."""
        # If we have boss rewards generated, use those
        if self.current_rewards and self.current_rewards.boss_relics:
            boss_relics = self.current_rewards.boss_relics
            if not boss_relics.is_resolved:
                return [BossRewardAction(i) for i in range(len(boss_relics.relics))]
            else:
                # Already picked, proceed to next act
                return []
        # Fallback: 3 boss relics
        return [BossRewardAction(i) for i in range(3)]

    # =========================================================================
    # Action Handlers
    # =========================================================================

    def _handle_path_action(self, action: PathAction) -> Tuple[bool, Dict]:
        """Handle choosing a path on the map."""
        paths = self.run_state.get_available_paths()

        if action.node_index < 0 or action.node_index >= len(paths):
            return False, {"error": "Invalid path index"}

        target_node = paths[action.node_index]

        # Move to the node
        self.run_state.move_to(target_node.x, target_node.y)
        self.run_state.advance_floor()

        self._log(f"\n--- Floor {self.run_state.floor} (Act {self.run_state.act}) ---")
        self._log(f"Entered: {target_node.room_type.name} room at ({target_node.x}, {target_node.y})")

        # Dispatch to appropriate room handler
        self._enter_room(target_node)

        return True, {"room_type": target_node.room_type.name}

    def _handle_neow_action(self, action: NeowAction) -> Tuple[bool, Dict]:
        """Handle Neow blessing choice using NeowHandler."""
        if self.neow_blessings is None or action.choice_index >= len(self.neow_blessings):
            return False, {"error": "Invalid Neow choice"}

        blessing = self.neow_blessings[action.choice_index]
        self._log(f"Neow blessing chosen: {blessing.description}")

        # Apply the blessing
        result = NeowHandler.apply_blessing(
            self.run_state,
            blessing,
            self.neow_rng,
            self.card_rng,
            self.relic_rng,
            self.potion_rng,
        )

        # Log the result
        if result.blessing_applied:
            self._log(f"  Blessing: {result.blessing_applied}")
        if result.drawback_applied:
            self._log(f"  Drawback: {result.drawback_applied}")

        # Check if blessing requires card selection - auto-select
        if result.requires_card_selection:
            sel_type = result.card_selection_type
            self._log(f"  (Auto-selecting for Neow: {sel_type})")
            if sel_type == "upgrade":
                upgradeable = self.run_state.get_upgradeable_cards()
                if upgradeable:
                    idx = upgradeable[0][0]
                    self.run_state.upgrade_card(idx)
                    self._log(f"  Auto-upgraded card at index {idx}")
            elif sel_type in ("remove", "remove_two"):
                removable = self.run_state.get_removable_cards()
                count = 2 if sel_type == "remove_two" else 1
                for _ in range(count):
                    if removable:
                        idx = removable[0][0]
                        removed = self.run_state.remove_card(idx)
                        self._log(f"  Auto-removed {removed.id if removed else 'card'}")
                        removable = self.run_state.get_removable_cards()
            elif sel_type in ("transform", "transform_two"):
                removable = self.run_state.get_removable_cards()
                count = 2 if sel_type == "transform_two" else 1
                for _ in range(count):
                    if removable:
                        idx = removable[0][0]
                        removed = self.run_state.remove_card(idx)
                        self._log(f"  Auto-transformed {removed.id if removed else 'card'}")
                        removable = self.run_state.get_removable_cards()
            elif sel_type == "choose" and result.card_choices:
                card = result.card_choices[0]
                self.run_state.add_card(card.id, getattr(card, 'upgraded', False))
                self._log(f"  Auto-chose card: {card.id}")

        # Clear Neow state and proceed to map
        self.neow_blessings = None
        self.phase = GamePhase.MAP_NAVIGATION

        self._sync_rng_counters()

        return True, {
            "choice": action.choice_index,
            "blessing_type": blessing.blessing_type.value,
            "result": result.blessing_applied,
            "drawback": result.drawback_applied,
        }

    def _handle_combat_action(self, action: CombatAction) -> Tuple[bool, Dict]:
        """Handle combat action using the CombatEngine."""
        if self.current_combat is None:
            # Fallback: auto-win if no combat engine
            self._log(f"Combat action: {action.action_type} (no engine, auto-win)")
            self._end_combat(victory=True)
            return True, {"victory": True}

        result = {}

        # Convert CombatAction to engine action and execute
        if action.action_type == "play_card":
            engine_action = PlayCard(card_idx=action.card_idx, target_idx=action.target_idx)
            result = self.current_combat.execute_action(engine_action)
            if result.get("success"):
                card_name = result.get("card", "card")
                self._log(f"Played: {card_name}")
            else:
                self._log(f"Failed to play card: {result.get('error', 'unknown')}")

        elif action.action_type == "use_potion":
            engine_action = UsePotion(potion_idx=action.potion_idx, target_idx=action.target_idx)
            result = self.current_combat.execute_action(engine_action)
            if result.get("success"):
                self._log(f"Used potion: {result.get('potion', 'potion')}")

        elif action.action_type == "end_turn":
            engine_action = EndTurn()
            self.current_combat.execute_action(engine_action)
            result = {"type": "end_turn"}
            self._log("End turn")

        # Smoke Bomb escape path: leave combat with no rewards and no defeat.
        if self.current_combat and getattr(self.current_combat.state, "escaped", False):
            self.run_state.current_hp = self.current_combat.state.player.hp
            self.current_combat = None
            self.current_rewards = None
            self.phase = GamePhase.MAP_NAVIGATION
            result["escaped"] = True
            self._log("Escaped combat with Smoke Bomb (no rewards)")
            self._sync_rng_counters()
            return True, result

        # Check if combat is over
        if self.current_combat.is_combat_over():
            if self.current_combat.is_victory():
                combat_result = self.current_combat.get_result()
                # Update player HP from combat
                self.run_state.current_hp = self.current_combat.state.player.hp
                self._end_combat(victory=True, combat_result=combat_result)
            else:
                self._end_combat(victory=False)
            return True, result

        self._sync_rng_counters()
        return True, result

    def _handle_reward_action(self, action: RewardAction) -> Tuple[bool, Dict]:
        """Handle reward choice using the RewardHandler system."""
        # Handle proceed action
        if action.reward_type == "proceed":
            self._log("Proceeding from rewards")
            if self._boss_fight_pending_boss_rewards:
                # After collecting combat rewards from boss, go to BOSS_REWARDS
                self._boss_fight_pending_boss_rewards = False
                # Generate boss relic choices (keep existing rewards for boss relics)
                if self.current_rewards and self.current_rewards.boss_relics:
                    self.phase = GamePhase.BOSS_REWARDS
                else:
                    # Re-generate boss rewards for relic selection
                    self.current_rewards = RewardHandler.generate_boss_rewards(
                        run_state=self.run_state,
                        card_rng=self.card_rng,
                        treasure_rng=self.treasure_rng,
                        potion_rng=self.potion_rng,
                        relic_rng=self.relic_rng,
                    )
                    self.phase = GamePhase.BOSS_REWARDS
                self._sync_rng_counters()
                return True, {"proceeded_to_boss_rewards": True}
            self.current_rewards = None
            self.phase = GamePhase.MAP_NAVIGATION
            self._sync_rng_counters()
            return True, {"proceeded": True}

        # If no rewards, just proceed
        if self.current_rewards is None:
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {"no_rewards": True}

        rewards = self.current_rewards
        result = {"reward_type": action.reward_type, "success": True}

        # Handle gold
        if action.reward_type == "gold":
            if rewards.gold and not rewards.gold.claimed:
                amount = rewards.gold.amount
                self.run_state.add_gold(amount)
                rewards.gold.claimed = True
                self._log(f"Gained {amount} gold (total: {self.run_state.gold})")
                result["gold_gained"] = amount

        # Handle potion
        elif action.reward_type == "potion":
            if rewards.potion and not rewards.potion.claimed:
                if self.run_state.count_empty_potion_slots() > 0:
                    self.run_state.add_potion(rewards.potion.potion.id)
                    rewards.potion.claimed = True
                    self._log(f"Gained potion: {rewards.potion.potion.name}")
                    result["potion_gained"] = rewards.potion.potion.name
                else:
                    result["success"] = False
                    result["error"] = "No empty potion slots"

        elif action.reward_type == "skip_potion":
            if rewards.potion and not rewards.potion.claimed:
                rewards.potion.skipped = True
                self._log("Skipped potion")
                result["potion_skipped"] = True

        # Handle card
        elif action.reward_type == "card":
            # Decode card_reward_index and card_index from choice_index
            card_reward_idx = action.choice_index // 100
            card_idx = action.choice_index % 100

            if card_reward_idx < len(rewards.card_rewards):
                card_reward = rewards.card_rewards[card_reward_idx]
                if not card_reward.is_resolved and card_idx < len(card_reward.cards):
                    card = card_reward.cards[card_idx]
                    self.run_state.add_card(card.id, card.upgraded)
                    card_reward.claimed_index = card_idx
                    self._log(f"Added {card.name} to deck")
                    result["card_added"] = card.name
                    result["card_rarity"] = card.rarity.name if hasattr(card, 'rarity') else "UNKNOWN"
                else:
                    result["success"] = False
                    result["error"] = "Invalid card choice"
            else:
                result["success"] = False
                result["error"] = "Invalid card reward index"

        elif action.reward_type == "skip_card":
            card_reward_idx = action.choice_index
            if card_reward_idx < len(rewards.card_rewards):
                card_reward = rewards.card_rewards[card_reward_idx]
                if not card_reward.is_resolved:
                    card_reward.skipped = True
                    self._log("Skipped card reward")
                    result["card_skipped"] = True

        elif action.reward_type == "singing_bowl":
            if self.run_state.has_relic("Singing Bowl"):
                card_reward_idx = action.choice_index
                if card_reward_idx < len(rewards.card_rewards):
                    card_reward = rewards.card_rewards[card_reward_idx]
                    if not card_reward.is_resolved:
                        self.run_state.gain_max_hp(2)
                        self.run_state.heal(2)
                        card_reward.singing_bowl_used = True
                        self._log("Singing Bowl: +2 Max HP")
                        result["max_hp_gained"] = 2

        # Handle relic
        elif action.reward_type == "relic":
            if rewards.relic and not rewards.relic.claimed:
                self.run_state.add_relic(
                    rewards.relic.relic.id,
                    misc_rng=self.misc_rng,
                    card_rng=self.card_rng,
                    relic_rng=self.relic_rng,
                    potion_rng=self.potion_rng,
                )
                rewards.relic.claimed = True
                self._log(f"Gained relic: {rewards.relic.relic.name}")
                result["relic_gained"] = rewards.relic.relic.name

        # Handle emerald key
        elif action.reward_type == "emerald_key":
            if rewards.emerald_key and not rewards.emerald_key.claimed:
                self.run_state.obtain_emerald_key()
                rewards.emerald_key.claimed = True
                self._log("Obtained Emerald Key")
                result["emerald_key"] = True

        elif action.reward_type == "skip_emerald_key":
            if rewards.emerald_key and not rewards.emerald_key.claimed:
                rewards.emerald_key.claimed = True  # Mark as resolved
                self._log("Skipped Emerald Key")
                result["emerald_key_skipped"] = True

        # Auto-proceed if all mandatory rewards resolved
        if self._mandatory_rewards_resolved(rewards):
            # Auto-claim gold if not yet claimed
            if rewards.gold and not rewards.gold.claimed:
                amount = rewards.gold.amount
                self.run_state.add_gold(amount)
                rewards.gold.claimed = True
                self._log(f"Auto-claimed {amount} gold")

        return True, result

    def _handle_event_action(self, action: EventAction) -> Tuple[bool, Dict]:
        """Handle event choice using EventHandler."""
        if self.current_event_state is None:
            self._log(f"Event choice: option {action.choice_index} (no event)")
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {"choice": action.choice_index}

        # Execute the choice, passing misc_rng if the handler supports it
        try:
            result = self.event_handler.execute_choice(
                self.current_event_state,
                action.choice_index,
                self.run_state,
                self.event_rng,
                card_idx=None,  # Would be passed for card selection events
                misc_rng=self.misc_rng,
            )
        except TypeError:
            # Fallback if event handler doesn't support misc_rng yet
            result = self.event_handler.execute_choice(
                self.current_event_state,
                action.choice_index,
                self.run_state,
                self.event_rng,
                card_idx=None,
            )

        self._sync_rng_counters()

        # Log the result
        self._log(f"Event choice: {result.choice_name}")
        if result.description:
            self._log(f"  {result.description}")

        # Log state changes
        if result.hp_change != 0:
            self._log(f"  HP: {result.hp_change:+d} ({self.run_state.current_hp}/{self.run_state.max_hp})")
        if result.max_hp_change != 0:
            self._log(f"  Max HP: {result.max_hp_change:+d}")
        if result.gold_change != 0:
            self._log(f"  Gold: {result.gold_change:+d} ({self.run_state.gold})")
        for relic in result.relics_gained:
            self._log(f"  +Relic: {relic}")
        for relic in result.relics_lost:
            self._log(f"  -Relic: {relic}")
        for card in result.cards_gained:
            self._log(f"  +Card: {card}")
        for card in result.cards_removed:
            self._log(f"  -Card: {card}")

        # Death check after event damage
        if self.run_state.current_hp <= 0:
            self.game_over = True
            self.game_lost = True
            self.game_won = False
            self.phase = GamePhase.RUN_COMPLETE
            self._log("Died from event damage - GAME OVER")
            return True, {
                "choice": action.choice_index,
                "choice_name": result.choice_name,
                "died": True,
            }

        # Handle combat trigger
        if result.combat_triggered:
            self._log(f"  Combat triggered: {result.combat_encounter}")
            # Would enter combat here with the specified encounter
            self._enter_combat(is_elite=False, is_boss=False)
            return True, {
                "choice": action.choice_index,
                "choice_name": result.choice_name,
                "combat_triggered": True,
                "combat_encounter": result.combat_encounter,
            }

        # Handle card selection requirement - auto-select first valid card
        if result.requires_card_selection:
            sel_type = result.card_selection_type
            self._log(f"  (Auto-selecting card for: {sel_type})")
            selected_idx = None

            if sel_type == "upgrade":
                upgradeable = self.run_state.get_upgradeable_cards()
                if upgradeable:
                    selected_idx = upgradeable[0][0]
                    self.run_state.upgrade_card(selected_idx)
                    self._log(f"  Auto-upgraded card at index {selected_idx}")
            elif sel_type == "remove":
                removable = self.run_state.get_removable_cards()
                if removable:
                    selected_idx = removable[0][0]
                    removed = self.run_state.remove_card(selected_idx)
                    self._log(f"  Auto-removed {removed.id if removed else 'card'}")
            elif sel_type == "transform":
                removable = self.run_state.get_removable_cards()
                if removable:
                    selected_idx = removable[0][0]
                    removed = self.run_state.remove_card(selected_idx)
                    self._log(f"  Auto-transformed {removed.id if removed else 'card'}")
            elif sel_type == "duplicate":
                if self.run_state.deck:
                    card = self.run_state.deck[0]
                    self.run_state.add_card(card.id, getattr(card, 'upgraded', False))
                    self._log(f"  Auto-duplicated {card.id}")
            elif sel_type == "choose":
                # Card choose from library/Neow - skip (take nothing)
                self._log(f"  Auto-skipped card choice")

            # Complete the event and transition
            self.current_event_state = None
            self.phase = GamePhase.MAP_NAVIGATION

        # Check if event is complete
        if result.event_complete:
            self.current_event_state = None
            self.phase = GamePhase.MAP_NAVIGATION
        else:
            # Update event phase for multi-phase events
            if result.next_phase:
                self.current_event_state.phase = result.next_phase

        return True, {
            "choice": action.choice_index,
            "choice_name": result.choice_name,
            "event_complete": result.event_complete,
            "hp_change": result.hp_change,
            "gold_change": result.gold_change,
            "cards_gained": result.cards_gained,
            "cards_removed": result.cards_removed,
            "relics_gained": result.relics_gained,
        }

    def _handle_shop_action(self, action: ShopAction) -> Tuple[bool, Dict]:
        """Handle shop action using ShopHandler."""
        if action.action_type == "leave":
            self._log("Left shop")
            self.current_shop = None
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {"left": True}

        if self.current_shop is None:
            return False, {"error": "Not in shop"}

        # Handle buying colored cards
        if action.action_type == "buy_colored_card":
            shop_card = None
            for c in self.current_shop.colored_cards:
                if c.slot_index == action.item_index and not c.purchased:
                    shop_card = c
                    break

            if shop_card is None:
                return False, {"error": "Card not found or already purchased"}
            if self.run_state.gold < shop_card.price:
                return False, {"error": "Not enough gold"}

            # Process purchase
            self.run_state.spend_gold(shop_card.price)
            self.run_state.add_card(shop_card.card.id, shop_card.card.upgraded)
            shop_card.purchased = True

            self._log(f"Purchased {shop_card.card.name} for {shop_card.price} gold")
            return True, {
                "action": "buy_colored_card",
                "item": shop_card.card.name,
                "price": shop_card.price,
                "gold_remaining": self.run_state.gold,
            }

        # Handle buying colorless cards
        if action.action_type == "buy_colorless_card":
            shop_card = None
            for c in self.current_shop.colorless_cards:
                if c.slot_index == action.item_index and not c.purchased:
                    shop_card = c
                    break

            if shop_card is None:
                return False, {"error": "Card not found or already purchased"}
            if self.run_state.gold < shop_card.price:
                return False, {"error": "Not enough gold"}

            # Process purchase
            self.run_state.spend_gold(shop_card.price)
            self.run_state.add_card(shop_card.card.id, shop_card.card.upgraded)
            shop_card.purchased = True

            self._log(f"Purchased {shop_card.card.name} for {shop_card.price} gold")
            return True, {
                "action": "buy_colorless_card",
                "item": shop_card.card.name,
                "price": shop_card.price,
                "gold_remaining": self.run_state.gold,
            }

        # Handle buying relics
        if action.action_type == "buy_relic":
            shop_relic = None
            for r in self.current_shop.relics:
                if r.slot_index == action.item_index and not r.purchased:
                    shop_relic = r
                    break

            if shop_relic is None:
                return False, {"error": "Relic not found or already purchased"}
            if self.run_state.gold < shop_relic.price:
                return False, {"error": "Not enough gold"}

            # Process purchase
            self.run_state.spend_gold(shop_relic.price)
            self.run_state.add_relic(
                shop_relic.relic.id,
                misc_rng=self.misc_rng,
                card_rng=self.card_rng,
                relic_rng=self.relic_rng,
                potion_rng=self.potion_rng,
            )
            shop_relic.purchased = True

            self._log(f"Purchased {shop_relic.relic.name} for {shop_relic.price} gold")
            return True, {
                "action": "buy_relic",
                "item": shop_relic.relic.name,
                "price": shop_relic.price,
                "gold_remaining": self.run_state.gold,
            }

        # Handle buying potions
        if action.action_type == "buy_potion":
            if self.run_state.count_empty_potion_slots() == 0:
                return False, {"error": "No empty potion slots"}

            shop_potion = None
            for p in self.current_shop.potions:
                if p.slot_index == action.item_index and not p.purchased:
                    shop_potion = p
                    break

            if shop_potion is None:
                return False, {"error": "Potion not found or already purchased"}
            if self.run_state.gold < shop_potion.price:
                return False, {"error": "Not enough gold"}

            # Process purchase
            self.run_state.spend_gold(shop_potion.price)
            self.run_state.add_potion(shop_potion.potion.id)
            shop_potion.purchased = True

            self._log(f"Purchased {shop_potion.potion.name} for {shop_potion.price} gold")
            return True, {
                "action": "buy_potion",
                "item": shop_potion.potion.name,
                "price": shop_potion.price,
                "gold_remaining": self.run_state.gold,
            }

        # Handle card removal
        if action.action_type == "remove_card":
            if not self.current_shop.purge_available:
                return False, {"error": "Card removal not available"}
            if self.run_state.gold < self.current_shop.purge_cost:
                return False, {"error": "Not enough gold"}

            card_idx = action.item_index
            if card_idx < 0 or card_idx >= len(self.run_state.deck):
                return False, {"error": "Invalid card index"}

            # Get card info before removal
            card = self.run_state.deck[card_idx]
            card_name = card.name if hasattr(card, 'name') else card.id

            # Process removal
            cost = self.current_shop.purge_cost
            self.run_state.spend_gold(cost)
            self.run_state.remove_card(card_idx)
            self.current_shop.purge_available = False

            # Increment purge count for next shop
            self.run_state.purge_count += 1

            self._log(f"Removed {card_name} for {cost} gold")
            return True, {
                "action": "remove_card",
                "item": card_name,
                "price": cost,
                "gold_remaining": self.run_state.gold,
            }

        return False, {"error": f"Unknown shop action: {action.action_type}"}

    def _handle_rest_action(self, action: RestAction) -> Tuple[bool, Dict]:
        """Handle rest site action."""
        result = {}

        if action.action_type == "rest":
            rest_result = RestHandler.rest(self.run_state)
            if rest_result.hp_healed:
                self._log(
                    f"Rested: healed {rest_result.hp_healed} HP "
                    f"({self.run_state.current_hp}/{self.run_state.max_hp})"
                )
            result = {"healed": rest_result.hp_healed}

            # Dream Catcher: generate card reward after resting
            if rest_result.dream_catcher_triggered:
                from .generation.rewards import RewardState, generate_card_rewards
                reward_state = RewardState()
                cards = generate_card_rewards(
                    self.card_rng,
                    reward_state,
                    act=self.run_state.act,
                    player_class=self.run_state.character,
                    ascension=self.run_state.ascension,
                    room_type="normal",
                    num_cards=3,
                )
                if cards:
                    self._log("Dream Catcher: choose a card reward")
                    # Store card choices and transition to a card reward selection
                    # For now, auto-skip (full implementation would need a sub-phase)
                    self._log(f"  Available: {[c.name for c in cards]}")
                    result["dream_catcher_cards"] = [c.id for c in cards]

        elif action.action_type == "upgrade":
            if action.card_index >= 0:
                rest_result = RestHandler.smith(self.run_state, action.card_index)
                if rest_result.card_upgraded:
                    self._log(f"Upgraded: {rest_result.card_upgraded}")
                    result = {"upgraded": rest_result.card_upgraded}

        elif action.action_type in ("ruby_key", "recall"):
            rest_result = RestHandler.recall(self.run_state)
            if self.run_state.has_ruby_key:
                self._log("Obtained Ruby Key (skipped rest)")
                result = {"ruby_key": True}

        elif action.action_type == "dig":
            rest_result = RestHandler.dig(
                self.run_state,
                self.relic_rng,
                misc_rng=self.misc_rng,
                card_rng=self.card_rng,
                potion_rng=self.potion_rng,
            )
            if rest_result.relic_gained:
                self._log(f"Dug with Shovel: gained {rest_result.relic_gained}")
                result = {"dug": rest_result.relic_gained}

        elif action.action_type == "lift":
            rest_result = RestHandler.lift(self.run_state)
            if rest_result.strength_gained:
                self._log("Lifted with Girya (gained Strength)")
                result = {"lifted": True}

        elif action.action_type == "toke":
            # Peace Pipe: remove a card
            if action.card_index >= 0 and action.card_index < len(self.run_state.deck):
                rest_result = RestHandler.toke(self.run_state, action.card_index)
                if rest_result.card_removed:
                    self._log(f"Toked (Peace Pipe): removed {rest_result.card_removed}")
                    result = {"toked": rest_result.card_removed}

        self.phase = GamePhase.MAP_NAVIGATION
        self._sync_rng_counters()
        return True, result

    def _handle_treasure_action(self, action: TreasureAction) -> Tuple[bool, Dict]:
        """Handle treasure room action using TreasureHandler."""
        if action.action_type == "take_relic":
            reward = TreasureHandler.open_chest(
                run_state=self.run_state,
                treasure_rng=self.treasure_rng,
                relic_rng=self.relic_rng,
                take_sapphire_key=False,
                misc_rng=self.misc_rng,
                card_rng=self.card_rng,
                potion_rng=self.potion_rng,
            )
            self._sync_rng_counters()
            self._log(f"Opened {reward.chest_type.value} chest: {reward.relic_name} ({reward.relic_tier})")
            if reward.curse_added:
                self._log(f"  Cursed Key: gained {reward.curse_added}")
            if reward.matryoshka_relics:
                for r in reward.matryoshka_relics:
                    self._log(f"  Matryoshka bonus: {r}")
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {
                "took_relic": True,
                "relic_id": reward.relic_id,
                "relic_name": reward.relic_name,
                "chest_type": reward.chest_type.value,
                "curse_added": reward.curse_added,
            }

        elif action.action_type == "sapphire_key":
            reward = TreasureHandler.open_chest(
                run_state=self.run_state,
                treasure_rng=self.treasure_rng,
                relic_rng=self.relic_rng,
                take_sapphire_key=True,
                misc_rng=self.misc_rng,
                card_rng=self.card_rng,
                potion_rng=self.potion_rng,
            )
            self._sync_rng_counters()
            self._log("Obtained Sapphire Key (skipped relic)")
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {"sapphire_key": True}

        return False, {"error": "Invalid treasure action"}

    def _handle_boss_reward_action(self, action: BossRewardAction) -> Tuple[bool, Dict]:
        """Handle boss relic choice using RewardHandler."""
        result = {"relic_index": action.relic_index}

        # If we have boss rewards generated, pick from those
        if self.current_rewards and self.current_rewards.boss_relics:
            boss_relics = self.current_rewards.boss_relics
            if not boss_relics.is_resolved and action.relic_index < 0:
                boss_relics.chosen_index = -1
                self._log("Boss relic skipped")
                result["skipped"] = True
            elif not boss_relics.is_resolved and 0 <= action.relic_index < len(boss_relics.relics):
                relic = boss_relics.relics[action.relic_index]

                # Handle boss relic pickup effects (starter relic replacement)
                RewardHandler._handle_boss_relic_pickup(self.run_state, relic)

                # Add the relic
                self.run_state.add_relic(
                    relic.id,
                    misc_rng=self.misc_rng,
                    card_rng=self.card_rng,
                    relic_rng=self.relic_rng,
                    potion_rng=self.potion_rng,
                )
                boss_relics.chosen_index = action.relic_index

                self._log(f"Boss relic chosen: {relic.name}")
                result["relic_name"] = relic.name
                result["relic_id"] = relic.id
            else:
                self._log(f"Invalid boss relic index: {action.relic_index}")
        else:
            self._log(f"Boss relic chosen: index {action.relic_index} (no rewards generated)")

        # Clear rewards
        self.current_rewards = None

        # After boss, advance to next act
        if self.run_state.act < 3:
            # Act 1 -> Act 2, Act 2 -> Act 3
            self.run_state.advance_act()
            self._apply_act_transition_rng_snaps()
            self._generate_encounter_tables()
            self._log(f"\n=== Entering Act {self.run_state.act} ===")
            self.phase = GamePhase.MAP_NAVIGATION
        elif self.run_state.act == 3:
            # Check if keys collected for Act 4
            has_all_keys = (
                self.run_state.has_ruby_key
                and self.run_state.has_emerald_key
                and self.run_state.has_sapphire_key
            )
            if has_all_keys:
                self.run_state.advance_act()
                self._apply_act_transition_rng_snaps()
                self._generate_encounter_tables()
                self._log(f"\n=== Entering Act 4 (The Ending) ===")
                self.phase = GamePhase.MAP_NAVIGATION
            else:
                # Victory without Act 4
                self.game_won = True
                self.game_over = True
                self.phase = GamePhase.RUN_COMPLETE
                self._log("\n=== VICTORY ===")
        else:
            # Act 4 boss defeated (or beyond) -> victory
            self.game_won = True
            self.game_over = True
            self.phase = GamePhase.RUN_COMPLETE
            self._log("\n=== VICTORY ===")

        return True, result

    # =========================================================================
    # Room Entry Handlers
    # =========================================================================

    def _apply_room_entry_relics(self, room_type: RoomType) -> None:
        """Apply out-of-combat relic effects that trigger on room entry."""
        # Java Maw Bank: gain 12 gold on every room entry until spent.
        maw_bank = self.run_state.get_relic("MawBank")
        if maw_bank and maw_bank.counter != -2:
            self.run_state.add_gold(12)

        # Ssserpent Head: gain 50 gold on entering ? rooms.
        if room_type == RoomType.EVENT and self.run_state.has_relic("SsserpentHead"):
            self.run_state.add_gold(50)

    def _enter_room(self, node: MapRoomNode):
        """Enter a room and set up appropriate phase."""
        room_type = node.room_type
        self._apply_room_entry_relics(room_type)

        if room_type == RoomType.MONSTER:
            self._enter_combat(is_elite=False, is_boss=False)
        elif room_type == RoomType.ELITE:
            self._enter_combat(is_elite=True, is_boss=False)
        elif room_type == RoomType.BOSS:
            self._enter_combat(is_elite=False, is_boss=True)
        elif room_type == RoomType.EVENT:
            self._enter_event()
        elif room_type == RoomType.SHOP:
            self._enter_shop()
        elif room_type == RoomType.REST:
            self._enter_rest()
        elif room_type == RoomType.TREASURE:
            self._enter_treasure()

    def _enter_combat(self, is_elite: bool = False, is_boss: bool = False):
        """Enter a combat encounter using real Enemy AI objects."""
        combat_type = "BOSS" if is_boss else ("ELITE" if is_elite else "MONSTER")
        self._log(f"Combat started ({combat_type})")
        self.phase = GamePhase.COMBAT
        self.current_room_type = "boss" if is_boss else ("elite" if is_elite else "monster")

        deck_ids = [
            (card.id + "+" if card.upgraded else card.id) if hasattr(card, 'id') else str(card)
            for card in self.run_state.deck
        ]
        relics = [r.id if hasattr(r, 'id') else str(r) for r in self.run_state.relics]
        potions = [
            "" if slot.is_empty() else (slot.potion_id or "")
            for slot in self.run_state.potion_slots
        ]
        potions.extend("" for _ in range(max(0, 3 - len(potions))))

        if is_boss:
            encounter_name = self._boss_name or "Slime Boss"
        elif is_elite:
            if self._elite_index < len(self._elite_list):
                encounter_name = self._elite_list[self._elite_index]
                self._elite_index += 1
            else:
                encounter_name = "Lagavulin"
        else:
            if self._monster_index < len(self._monster_list):
                encounter_name = self._monster_list[self._monster_index]
                self._monster_index += 1
            else:
                encounter_name = "Jaw Worm"

        floor_num = self.run_state.floor
        floor_ai_rng = Random(self.seed + floor_num)
        floor_hp_rng = Random(self.seed + floor_num)

        if encounter_name in ENCOUNTER_TABLE:
            enemies = create_enemies_from_encounter(
                encounter_name,
                ai_rng=floor_ai_rng,
                ascension=self.run_state.ascension,
                hp_rng=floor_hp_rng,
            )

            self.current_combat = create_combat_from_enemies(
                enemies=enemies,
                player_hp=self.run_state.current_hp,
                player_max_hp=self.run_state.max_hp,
                deck=deck_ids,
                relics=relics,
                potions=potions,
                ascension=self.run_state.ascension,
                bottled_cards=self.run_state.get_bottled_cards(),
            )
        else:
            # Fallback to simple combat for unknown encounters
            self._log(f"WARNING: Unknown encounter '{encounter_name}', using simple combat")
            self.current_combat = create_simple_combat(
                enemy_id=encounter_name,
                enemy_hp=50,
                enemy_damage=8,
                player_hp=self.run_state.current_hp,
                deck=deck_ids,
            )
            self.current_combat.state.potions = potions

        # Attach room metadata + RNG streams so combat/potion handlers can use proper context.
        self.current_combat.state.current_room_type = self.current_room_type
        self.current_combat.state.is_boss_combat = bool(is_boss)
        self.current_combat.state.card_rng = self.card_rng
        self.current_combat.state.card_random_rng = self.card_random_rng
        self.current_combat.state.potion_rng = self.potion_rng
        self.current_combat.state.relic_rng = self.relic_rng
        self.current_combat.state.player_class = str(self.run_state.character).upper()

        # Start combat
        self.current_combat.start_combat()

        enemy_names = [e.id for e in self.current_combat.state.enemies]
        self._log(f"Encounter: {encounter_name} -> {enemy_names}")
        self._log(f"Player: {self.run_state.current_hp}/{self.run_state.max_hp} HP, Deck: {len(deck_ids)} cards")

    def _end_combat(self, victory: bool, combat_result: Optional[CombatResult] = None):
        """End a combat encounter and generate rewards using RewardHandler."""
        # Clean up combat engine
        self.current_combat = None

        if victory:
            self.run_state.combats_won += 1

            # Log combat stats if available
            if combat_result:
                self._log(f"Combat victory! HP: {self.run_state.current_hp}/{self.run_state.max_hp}")
                self._log(f"  Turns: {combat_result.turns}, Cards played: {combat_result.cards_played}")
                self._log(f"  Damage dealt: {combat_result.damage_dealt}, Damage taken: {combat_result.damage_taken}")
            else:
                self._log(f"Combat victory! HP: {self.run_state.current_hp}/{self.run_state.max_hp}")

            # Generate rewards using RewardHandler
            if self.current_room_type == "boss":
                # Boss: go through COMBAT_REWARDS first (gold/potion/cards),
                # then transition to BOSS_REWARDS for boss relic choice
                self._boss_fight_pending_boss_rewards = True
                self.current_rewards = RewardHandler.generate_boss_rewards(
                    run_state=self.run_state,
                    card_rng=self.card_rng,
                    treasure_rng=self.treasure_rng,
                    potion_rng=self.potion_rng,
                    relic_rng=self.relic_rng,
                )
                self.phase = GamePhase.COMBAT_REWARDS
            else:
                # Monster/Elite rewards: gold, potion chance, cards, and relic (elite only)
                enemies_killed = 1  # TODO: Track actual enemy count from combat
                self.current_rewards = RewardHandler.generate_combat_rewards(
                    run_state=self.run_state,
                    room_type=self.current_room_type,
                    card_rng=self.card_rng,
                    treasure_rng=self.treasure_rng,
                    potion_rng=self.potion_rng,
                    relic_rng=self.relic_rng,
                    enemies_killed=enemies_killed,
                    is_burning_elite=self.is_burning_elite,
                )
                self.phase = GamePhase.COMBAT_REWARDS

            # Between-floor relic triggers
            self._trigger_post_combat_relics()

            # Auto-claim gold (in StS, gold is always auto-collected)
            if self.current_rewards and self.current_rewards.gold:
                RewardHandler.auto_claim_gold(self.run_state, self.current_rewards)
                self._log(f"  Gold: {self.current_rewards.gold.amount}")

            # Log rewards
            if self.current_rewards:
                if self.current_rewards.potion:
                    self._log(f"  Potion: {self.current_rewards.potion.potion.name}")
                for i, card_reward in enumerate(self.current_rewards.card_rewards):
                    card_names = [c.name for c in card_reward.cards]
                    self._log(f"  Card choices: {card_names}")
                if self.current_rewards.relic:
                    self._log(f"  Relic: {self.current_rewards.relic.relic.name}")
                if self.current_rewards.boss_relics:
                    relic_names = [r.name for r in self.current_rewards.boss_relics.relics]
                    self._log(f"  Boss relics: {relic_names}")
        else:
            self.game_lost = True
            self.game_over = True
            self.phase = GamePhase.RUN_COMPLETE
            self._log(f"Combat defeat - GAME OVER")

        self._sync_rng_counters()

    def _trigger_post_combat_relics(self):
        """Trigger between-floor relic effects after combat victory."""
        rs = self.run_state
        # Burning Blood: Heal 6 HP after combat
        if rs.has_relic("Burning Blood"):
            old_hp = rs.current_hp
            rs.heal(6)
            healed = rs.current_hp - old_hp
            if healed > 0:
                self._log(f"  Burning Blood: healed {healed} HP")

        # Black Blood (upgraded Burning Blood): Heal 12 HP after combat
        if rs.has_relic("Black Blood"):
            old_hp = rs.current_hp
            rs.heal(12)
            healed = rs.current_hp - old_hp
            if healed > 0:
                self._log(f"  Black Blood: healed {healed} HP")

        # Meat on the Bone: If HP < 50%, heal 12
        if rs.has_relic("MeatOnTheBone") or rs.has_relic("Meat on the Bone"):
            if rs.current_hp < rs.max_hp * 0.5:
                old_hp = rs.current_hp
                rs.heal(12)
                healed = rs.current_hp - old_hp
                if healed > 0:
                    self._log(f"  Meat on the Bone: healed {healed} HP")

        # Blood Vial: Heal 2 at end of combat
        if rs.has_relic("BloodVial") or rs.has_relic("Blood Vial"):
            old_hp = rs.current_hp
            rs.heal(2)
            healed = rs.current_hp - old_hp
            if healed > 0:
                self._log(f"  Blood Vial: healed {healed} HP")

    def _enter_event(self):
        """Enter an event room and select event using EventHandler."""
        # Select an event using the event handler
        self.current_event_state = self.event_handler.select_event(
            self.run_state,
            self.event_rng,
            misc_rng=self.misc_rng
        )
        self._sync_rng_counters()

        if self.current_event_state is None:
            # No event available, skip
            self._log("Event encountered: No events available")
            self.phase = GamePhase.MAP_NAVIGATION
            return

        # Get event definition for logging
        event_def = self.event_handler._get_event_definition(self.current_event_state.event_id)
        event_name = event_def.name if event_def else self.current_event_state.event_id
        self._log(f"Event: {event_name}")

        # Log available choices
        choices = self.event_handler.get_available_choices(
            self.current_event_state,
            self.run_state
        )
        for choice in choices:
            self._log(f"  [{choice.index}] {choice.text}")

        self.phase = GamePhase.EVENT

    def _enter_shop(self):
        """Enter a shop room and generate inventory."""
        self._log("Entered shop")

        # Trigger Meal Ticket healing
        if self.run_state.has_relic("MealTicket"):
            old_hp = self.run_state.current_hp
            self.run_state.heal(15)
            actual_heal = self.run_state.current_hp - old_hp
            if actual_heal > 0:
                self._log(f"Meal Ticket: healed {actual_heal} HP")

        # Generate shop inventory
        self.current_shop = ShopHandler.create_shop(
            self.run_state,
            self.merchant_rng,
            self.card_rng,
            self.potion_rng,
        )
        self._sync_rng_counters()

        # Log shop contents
        self._log(f"Shop inventory:")
        self._log(f"  Colored cards: {len(self.current_shop.colored_cards)}")
        for c in self.current_shop.colored_cards:
            sale = " [SALE]" if c.on_sale else ""
            self._log(f"    - {c.card.name} ({c.card.rarity.name}): {c.price}g{sale}")
        self._log(f"  Colorless cards: {len(self.current_shop.colorless_cards)}")
        for c in self.current_shop.colorless_cards:
            self._log(f"    - {c.card.name} ({c.card.rarity.name}): {c.price}g")
        self._log(f"  Relics: {len(self.current_shop.relics)}")
        for r in self.current_shop.relics:
            self._log(f"    - {r.relic.name} ({r.relic.tier.name}): {r.price}g")
        self._log(f"  Potions: {len(self.current_shop.potions)}")
        for p in self.current_shop.potions:
            self._log(f"    - {p.potion.name} ({p.potion.rarity.name}): {p.price}g")
        self._log(f"  Card removal: {self.current_shop.purge_cost}g")
        self._log(f"  Player gold: {self.run_state.gold}")

        self.phase = GamePhase.SHOP

    def _enter_rest(self):
        """Enter a rest site."""
        self._log("Arrived at rest site")

        # Eternal Feather: heal on entering rest site
        healed = RestHandler.on_enter_rest_site(self.run_state)
        if healed > 0:
            self._log(f"Eternal Feather: healed {healed} HP ({self.run_state.current_hp}/{self.run_state.max_hp})")

        self.phase = GamePhase.REST

    def _enter_treasure(self):
        """Enter a treasure room."""
        self._log("Found treasure chest")
        self.phase = GamePhase.TREASURE

    # =========================================================================
    # State Utilities
    # =========================================================================

    def _create_state_snapshot(self) -> Dict[str, Any]:
        """Create a snapshot of relevant game state for logging."""
        return {
            "hp": self.run_state.current_hp,
            "max_hp": self.run_state.max_hp,
            "gold": self.run_state.gold,
            "floor": self.run_state.floor,
            "act": self.run_state.act,
            "deck_size": len(self.run_state.deck),
            "relic_count": len(self.run_state.relics),
        }

    def get_run_statistics(self) -> Dict[str, Any]:
        """Get statistics for the current run."""
        return {
            "seed": self.seed_string,
            "ascension": self.run_state.ascension,
            "game_won": self.game_won,
            "game_lost": self.game_lost,
            "final_floor": self.run_state.floor,
            "final_act": self.run_state.act,
            "final_hp": self.run_state.current_hp,
            "final_max_hp": self.run_state.max_hp,
            "final_gold": self.run_state.gold,
            "deck_size": len(self.run_state.deck),
            "relic_count": len(self.run_state.relics),
            "combats_won": self.run_state.combats_won,
            "floors_climbed": self.run_state.floors_climbed,
            "decisions_made": len(self.decision_log),
        }

    def display_map(self):
        """Display the current act's map."""
        current_map = self.run_state.get_current_map()
        if current_map:
            print(f"\n=== Act {self.run_state.act} Map ===")
            print(map_to_string(current_map))
            print(f"Current position: {self.run_state.map_position}")

    def get_current_room_type(self) -> Optional[RoomType]:
        """Get the type of the current room."""
        if self.run_state.map_position.is_at_start():
            return None

        current_map = self.run_state.get_current_map()
        if current_map:
            pos = self.run_state.map_position
            return current_map[pos.y][pos.x].room_type
        return None


# =============================================================================
# Example Usage
# =============================================================================

def main():
    """Run the GameRunner demo."""
    print("=== Slay the Spire Game Runner Demo ===\n")

    # Create a new game
    runner = GameRunner(seed="TEST123", ascension=20, verbose=True)

    # Display initial map
    runner.display_map()

    # Run first 5 floors with random actions
    print("\n--- Running 5 floors with random actions ---\n")
    stats = runner.run_to_floor(5)

    # Show final statistics
    print("\n=== Run Statistics ===")
    for key, value in stats.items():
        print(f"  {key}: {value}")

    # Display map after 5 floors
    runner.display_map()

    print("\n--- Manual control example ---")

    # Create another game for manual control demo
    runner2 = GameRunner(seed="MANUAL", ascension=20, verbose=True)

    # Get available actions
    actions = runner2.get_available_actions()
    print(f"\nAvailable actions: {len(actions)}")
    for i, action in enumerate(actions[:5]):  # Show first 5
        print(f"  {i}: {action}")

    # Take first action manually
    if actions:
        print(f"\nTaking first action: {actions[0]}")
        runner2.take_action(actions[0])


# =============================================================================
# Headless RL Mode
# =============================================================================

@dataclass
class RunResult:
    """Result of a headless game run."""
    seed: int
    ascension: int
    victory: bool
    floor_reached: int
    hp_remaining: int
    gold: int
    deck_size: int
    relics_count: int
    combats_won: int
    stats: Dict[str, Any] = field(default_factory=dict)


def run_headless(
    seed: int,
    ascension: int = 20,
    decision_fn=None,
    max_actions: int = 10000,
) -> RunResult:
    """
    Run a complete game headlessly with a decision function.

    Args:
        seed: Game seed (numeric)
        ascension: Ascension level
        decision_fn: Callable(run_state, actions) -> action.
                     If None, picks the first available action.
        max_actions: Safety limit to prevent infinite loops

    Returns:
        RunResult with game outcome
    """
    if decision_fn is None:
        decision_fn = lambda state, actions: actions[0]

    runner = GameRunner(seed=seed, ascension=ascension, verbose=False)
    actions_taken = 0

    while not runner.game_over and actions_taken < max_actions:
        actions = runner.get_available_actions()
        if not actions:
            break
        action = decision_fn(runner.run_state, actions)
        runner.take_action(action)
        actions_taken += 1

    stats = runner.get_run_statistics()
    return RunResult(
        seed=seed,
        ascension=ascension,
        victory=stats.get("game_won", False),
        floor_reached=runner.run_state.floor,
        hp_remaining=runner.run_state.current_hp,
        gold=runner.run_state.gold,
        deck_size=len(runner.run_state.deck),
        relics_count=len(runner.run_state.relics),
        combats_won=runner.run_state.combats_won,
        stats=stats,
    )


def run_parallel(
    seeds: List[int],
    ascension: int = 20,
    decision_fn=None,
    max_workers: int = 4,
) -> List[RunResult]:
    """
    Run multiple games in parallel using ProcessPoolExecutor.

    Args:
        seeds: List of game seeds
        ascension: Ascension level for all games
        decision_fn: Decision function (must be picklable for multiprocessing,
                     so use module-level functions, not lambdas).
                     If None, picks first available action.
        max_workers: Number of parallel workers

    Returns:
        List of RunResult, one per seed
    """
    from concurrent.futures import ProcessPoolExecutor, as_completed

    def _run_one(seed):
        return run_headless(seed, ascension, decision_fn)

    results = []
    with ProcessPoolExecutor(max_workers=max_workers) as executor:
        futures = {executor.submit(_run_one, s): s for s in seeds}
        for future in as_completed(futures):
            results.append(future.result())

    # Sort by seed to match input order
    seed_order = {s: i for i, s in enumerate(seeds)}
    results.sort(key=lambda r: seed_order.get(r.seed, 0))
    return results


if __name__ == "__main__":
    main()
