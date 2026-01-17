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
from .generation.map import (
    MapRoomNode, RoomType, MapGenerator, MapGeneratorConfig,
    get_map_seed_offset, map_to_string, generate_act4_map
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
        self.pending_rewards: List[Dict] = []

        # Current event state (when in event)
        self.current_event: Optional[Dict] = None

        # Current shop state (when in shop)
        self.current_shop: Optional[Dict] = None

        # RNG instances for different purposes
        self._init_rng()

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

        if self.phase == GamePhase.NEOW:
            return self._get_neow_actions()
        elif self.phase == GamePhase.MAP_NAVIGATION:
            return self._get_path_actions()
        elif self.phase == GamePhase.COMBAT:
            return self._get_combat_actions()
        elif self.phase == GamePhase.COMBAT_REWARDS:
            return self._get_reward_actions()
        elif self.phase == GamePhase.EVENT:
            return self._get_event_actions()
        elif self.phase == GamePhase.SHOP:
            return self._get_shop_actions()
        elif self.phase == GamePhase.REST:
            return self._get_rest_actions()
        elif self.phase == GamePhase.TREASURE:
            return self._get_treasure_actions()
        elif self.phase == GamePhase.BOSS_REWARDS:
            return self._get_boss_reward_actions()
        else:
            return []

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
        # Simplified: 4 blessing options
        return [NeowAction(i) for i in range(4)]

    def _get_path_actions(self) -> List[GameAction]:
        """Get available path choices on the map."""
        paths = self.run_state.get_available_paths()
        return [PathAction(i) for i in range(len(paths))]

    def _get_combat_actions(self) -> List[GameAction]:
        """Get available combat actions (stub)."""
        # Stub: just end turn for now
        return [CombatAction(action_type="end_turn")]

    def _get_reward_actions(self) -> List[GameAction]:
        """Get available reward choices."""
        actions = []

        # Always can skip
        actions.append(RewardAction(reward_type="skip"))

        # Add actions for each pending reward
        for i, reward in enumerate(self.pending_rewards):
            if reward.get("type") == "card":
                # Card reward - can pick one of the offered cards
                cards = reward.get("cards", [])
                for j in range(len(cards)):
                    actions.append(RewardAction(reward_type="card", choice_index=j))
            elif reward.get("type") == "gold":
                actions.append(RewardAction(reward_type="gold", choice_index=i))
            elif reward.get("type") == "potion":
                actions.append(RewardAction(reward_type="potion", choice_index=i))
            elif reward.get("type") == "relic":
                actions.append(RewardAction(reward_type="relic", choice_index=i))

        return actions

    def _get_event_actions(self) -> List[GameAction]:
        """Get available event choices (stub)."""
        # Stub: 3 options
        return [EventAction(i) for i in range(3)]

    def _get_shop_actions(self) -> List[GameAction]:
        """Get available shop actions (stub)."""
        # Stub: just leave
        return [ShopAction(action_type="leave")]

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
        """Get available boss relic choices."""
        # Typically 3 boss relics to choose from
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
        """Handle Neow blessing choice (stub)."""
        self._log(f"Neow blessing chosen: option {action.choice_index}")
        self.phase = GamePhase.MAP_NAVIGATION
        return True, {"choice": action.choice_index}

    def _handle_combat_action(self, action: CombatAction) -> Tuple[bool, Dict]:
        """Handle combat action (stub - auto-win for now)."""
        self._log(f"Combat action: {action.action_type}")

        # Stub: combat ends after one action
        self._end_combat(victory=True)
        return True, {"victory": True}

    def _handle_reward_action(self, action: RewardAction) -> Tuple[bool, Dict]:
        """Handle reward choice."""
        if action.reward_type == "skip":
            self._log("Skipped rewards")
            self.pending_rewards.clear()
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {"skipped": True}

        # Handle specific reward types
        if action.reward_type == "gold" and self.pending_rewards:
            for reward in self.pending_rewards:
                if reward.get("type") == "gold":
                    amount = reward.get("amount", 0)
                    self.run_state.add_gold(amount)
                    self._log(f"Gained {amount} gold (total: {self.run_state.gold})")
                    self.pending_rewards.remove(reward)
                    break

        elif action.reward_type == "card":
            # Stub: would pick from card rewards
            self._log(f"Card reward chosen: index {action.choice_index}")
            self.pending_rewards = [r for r in self.pending_rewards if r.get("type") != "card"]

        # If no more rewards, return to map
        if not self.pending_rewards:
            self.phase = GamePhase.MAP_NAVIGATION

        return True, {"reward_type": action.reward_type}

    def _handle_event_action(self, action: EventAction) -> Tuple[bool, Dict]:
        """Handle event choice (stub)."""
        self._log(f"Event choice: option {action.choice_index}")
        self.phase = GamePhase.MAP_NAVIGATION
        return True, {"choice": action.choice_index}

    def _handle_shop_action(self, action: ShopAction) -> Tuple[bool, Dict]:
        """Handle shop action (stub)."""
        if action.action_type == "leave":
            self._log("Left shop")
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {"left": True}

        return False, {"error": "Shop not implemented"}

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
        """Handle treasure room action."""
        if action.action_type == "take_relic":
            self._log("Took relic from chest (stub)")
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {"took_relic": True}

        elif action.action_type == "sapphire_key":
            self.run_state.obtain_sapphire_key()
            self._log("Obtained Sapphire Key (skipped relic)")
            self.phase = GamePhase.MAP_NAVIGATION
            return True, {"sapphire_key": True}

        return False, {"error": "Invalid treasure action"}

    def _handle_boss_reward_action(self, action: BossRewardAction) -> Tuple[bool, Dict]:
        """Handle boss relic choice."""
        self._log(f"Boss relic chosen: index {action.relic_index}")

        # After boss, check if we advance to next act
        if self.run_state.act < 4:
            self.run_state.advance_act()
            self._log(f"\n=== Entering Act {self.run_state.act} ===")
            self.phase = GamePhase.MAP_NAVIGATION
        else:
            # Won the game
            self.game_won = True
            self.game_over = True
            self.phase = GamePhase.RUN_COMPLETE
            self._log("\n=== VICTORY ===")

        return True, {"relic_index": action.relic_index}

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
        """Enter a combat encounter."""
        combat_type = "BOSS" if is_boss else ("ELITE" if is_elite else "MONSTER")
        self._log(f"Combat started ({combat_type})")
        self.phase = GamePhase.COMBAT

        # Stub: would initialize actual combat here
        # For now, we'll auto-resolve in the combat action handler

    def _end_combat(self, victory: bool):
        """End a combat encounter."""
        if victory:
            self.run_state.combats_won += 1
            self._log(f"Combat victory! HP: {self.run_state.current_hp}/{self.run_state.max_hp}")

            # Set up rewards
            self.pending_rewards = [
                {"type": "gold", "amount": 25 + self.misc_rng.random(10)},
                {"type": "card", "cards": ["Strike", "Defend", "Eruption"]},  # Stub
            ]
            self.phase = GamePhase.COMBAT_REWARDS
        else:
            self.game_lost = True
            self.game_over = True
            self.phase = GamePhase.RUN_COMPLETE
            self._log(f"Combat defeat - GAME OVER")

    def _enter_event(self):
        """Enter an event room."""
        self._log("Event encountered (stub)")
        self.phase = GamePhase.EVENT

    def _enter_shop(self):
        """Enter a shop room."""
        self._log("Entered shop (stub)")
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
