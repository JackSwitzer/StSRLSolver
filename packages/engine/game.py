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
import random

from .state.run import RunState, create_watcher_run
from .state.rng import Random, seed_to_long
from .state.combat import CombatState, EnemyCombatState, create_combat, create_enemy
from .generation.map import (
    MapRoomNode, RoomType, MapGenerator, MapGeneratorConfig,
    get_map_seed_offset, map_to_string, generate_act4_map
)
from .combat_engine import (
    CombatEngine, CombatResult, PlayCard, EndTurn, UsePotion,
    CombatPhase as CombatEnginePhase, create_simple_combat,
)
from .content.cards import get_card, CardTarget, CardType
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
    action_type: str  # "take_relic", "sapphire_key", "leave"


@dataclass(frozen=True)
class BossRewardAction:
    """Choose boss relic."""
    relic_index: int


GameAction = Union[
    PathAction, NeowAction, CombatAction, RewardAction,
    EventAction, ShopAction, RestAction, TreasureAction, BossRewardAction
]


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
            character: Character class (only "Watcher" supported currently)
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
        self.run_state = create_watcher_run(self.seed_string, ascension)

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
        # AI RNG for enemy decisions
        self.ai_rng = Random(self.seed + 1000)
        # Monster HP RNG
        self.hp_rng = Random(self.seed + 2000)
        # Event RNG
        self.event_rng = Random(self.seed + 3000)
        # Misc RNG
        self.misc_rng = Random(self.seed + 4000)
        # Card reward RNG
        self.card_rng = Random(self.seed + 5000)
        # Treasure/gold RNG
        self.treasure_rng = Random(self.seed + 6000)
        # Potion drop RNG
        self.potion_rng = Random(self.seed + 7000)
        # Relic RNG
        self.relic_rng = Random(self.seed + 8000)
        # Merchant RNG (for shop prices and generation)
        self.merchant_rng = Random(self.seed + 9000)
        # Neow RNG (for blessing selection)
        self.neow_rng = Random(self.seed + 10000)
        # Monster encounter selection RNG
        self.monster_rng = Random(self.seed + 100)
        # Deck shuffle RNG
        self.shuffle_rng = Random(self.seed + 200)
        # Random card effects RNG
        self.card_random_rng = Random(self.seed + 300)

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

    def take_action(self, action: GameAction) -> bool:
        """
        Execute an action and advance the game state.

        Args:
            action: The action to take

        Returns:
            True if action was valid and executed, False otherwise
        """
        if self.game_over:
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

        # Rest (heal)
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
                return True, {"proceeded_to_boss_rewards": True}
            self.current_rewards = None
            self.phase = GamePhase.MAP_NAVIGATION
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
                self.run_state.add_relic(rewards.relic.relic.id)
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
            self.run_state.lose_gold(shop_card.price)
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
            self.run_state.lose_gold(shop_card.price)
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
            self.run_state.lose_gold(shop_relic.price)
            self.run_state.add_relic(shop_relic.relic.id)
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
            self.run_state.lose_gold(shop_potion.price)
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
            self.run_state.lose_gold(cost)
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
            # Heal 30% of max HP
            heal_amount = int(self.run_state.max_hp * 0.30)
            if self.run_state.has_relic("RegalPillow"):
                heal_amount += 15
            old_hp = self.run_state.current_hp
            self.run_state.heal(heal_amount)
            actual_heal = self.run_state.current_hp - old_hp
            self._log(f"Rested: healed {actual_heal} HP ({self.run_state.current_hp}/{self.run_state.max_hp})")
            result = {"healed": actual_heal}

        elif action.action_type == "upgrade":
            if action.card_index >= 0:
                card = self.run_state.deck[action.card_index]
                self.run_state.upgrade_card(action.card_index)
                self._log(f"Upgraded: {card}")
                result = {"upgraded": str(card)}

        elif action.action_type == "ruby_key":
            self.run_state.obtain_ruby_key()
            self._log("Obtained Ruby Key (skipped rest)")
            result = {"ruby_key": True}

        elif action.action_type == "dig":
            self._log("Dug with Shovel (would get relic)")
            result = {"dug": True}

        elif action.action_type == "lift":
            self.run_state.increment_relic_counter("Girya")
            self._log("Lifted with Girya (gained Strength)")
            result = {"lifted": True}

        self.phase = GamePhase.MAP_NAVIGATION
        return True, result

    def _handle_treasure_action(self, action: TreasureAction) -> Tuple[bool, Dict]:
        """Handle treasure room action using TreasureHandler."""
        if action.action_type == "take_relic":
            reward = TreasureHandler.open_chest(
                run_state=self.run_state,
                treasure_rng=self.treasure_rng,
                relic_rng=self.relic_rng,
                take_sapphire_key=False,
            )
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
            )
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
            if not boss_relics.is_resolved and 0 <= action.relic_index < len(boss_relics.relics):
                relic = boss_relics.relics[action.relic_index]

                # Handle boss relic pickup effects (starter relic replacement)
                RewardHandler._handle_boss_relic_pickup(self.run_state, relic)

                # Add the relic
                self.run_state.add_relic(relic.id)
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

    def _enter_room(self, node: MapRoomNode):
        """Enter a room and set up appropriate phase."""
        room_type = node.room_type

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
        """Enter a combat encounter using CombatEngine."""
        combat_type = "BOSS" if is_boss else ("ELITE" if is_elite else "MONSTER")
        self._log(f"Combat started ({combat_type})")
        self.phase = GamePhase.COMBAT
        self.current_room_type = "boss" if is_boss else ("elite" if is_elite else "monster")

        # Get deck as card IDs
        deck_ids = []
        for card in self.run_state.deck:
            if hasattr(card, 'id'):
                card_id = card.id
                if hasattr(card, 'upgraded') and card.upgraded:
                    card_id += "+"
            else:
                card_id = str(card)
            deck_ids.append(card_id)

        # Get relics list
        relics = [r.id if hasattr(r, 'id') else str(r) for r in self.run_state.relics]

        # Get potions
        potions = []
        for slot in self.run_state.potion_slots:
            if slot.is_empty():
                potions.append("")
            else:
                potions.append(slot.potion_id if slot.potion_id else "")
        # Ensure we have 3 potion slots
        while len(potions) < 3:
            potions.append("")

        # Select encounter from pre-generated tables
        if is_boss:
            enemy_id = self._boss_name or "SlimeBoss"
            base_hp = 250
            base_damage = 20
        elif is_elite:
            if self._elite_list and self._elite_index < len(self._elite_list):
                enemy_id = self._elite_list[self._elite_index]
                self._elite_index += 1
            else:
                enemy_id = "Lagavulin"
            # Set base stats by elite name
            elite_stats = {
                "Gremlin Nob": (84, 14), "Lagavulin": (110, 18), "3 Sentries": (120, 9),
                "Gremlin Leader": (140, 12), "Slavers": (130, 13), "Book of Stabbing": (160, 6),
                "Giant Head": (500, 13), "Nemesis": (185, 7), "Reptomancer": (180, 10),
                "Spire Shield and Spire Spear": (270, 15),
            }
            base_hp, base_damage = elite_stats.get(enemy_id, (90, 15))
        else:
            if self._monster_list and self._monster_index < len(self._monster_list):
                enemy_id = self._monster_list[self._monster_index]
                self._monster_index += 1
            else:
                enemy_id = "Jaw Worm"
            # Set base stats by monster name
            monster_stats = {
                "Jaw Worm": (42, 11), "Cultist": (50, 6), "2 Louse": (30, 6),
                "Small Slimes": (25, 5), "Blue Slaver": (48, 12), "Red Slaver": (48, 12),
                "Looter": (46, 10), "Gremlin Gang": (50, 5), "Large Slime": (65, 12),
                "Lots of Slimes": (12, 3), "Exordium Thugs": (48, 10),
                "Exordium Wildlife": (55, 10), "3 Louse": (30, 6), "2 Fungi Beasts": (24, 6),
            }
            base_hp, base_damage = monster_stats.get(enemy_id, (42, 11))

        # Ascension HP scaling
        if self.run_state.ascension >= 7:
            base_hp = int(base_hp * 1.04)
        if self.run_state.ascension >= 8:
            base_damage = int(base_damage * 1.08) if is_elite else base_damage

        # Create combat engine
        self.current_combat = create_simple_combat(
            enemy_id=enemy_id,
            enemy_hp=base_hp,
            enemy_damage=base_damage,
            player_hp=self.run_state.current_hp,
            deck=deck_ids,
        )

        # Set potions
        self.current_combat.state.potions = potions

        # Set relic counters
        for relic in relics:
            if "VioletLotus" in relic or "Violet Lotus" in relic:
                self.current_combat.state.relic_counters["_violet_lotus"] = 1
            if "Barricade" in relic:
                self.current_combat.state.relic_counters["_barricade"] = 1
            if "Runic Pyramid" in relic:
                self.current_combat.state.relic_counters["_runic_pyramid"] = 1

        # Start combat
        self.current_combat.start_combat()

        self._log(f"Enemy: {enemy_id} ({base_hp} HP)")
        self._log(f"Player: {self.run_state.current_hp} HP, Deck: {len(deck_ids)} cards")

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
            self.event_rng
        )

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
        self.current_shop = ShopHandler.create_shop(self.run_state, self.merchant_rng)

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


if __name__ == "__main__":
    main()
