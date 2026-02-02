#!/usr/bin/env python3
"""
Complete Game Simulator for Verification

A step-by-step game simulator that tracks all RNG state.
Run alongside the actual game to verify parity.
"""

import sys
import os
import json

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, field, asdict
from enum import Enum

from core.state.rng import Random, seed_to_long, long_to_seed
from core.state.game_rng import GameRNGState, RNGStream
from core.generation.map import MapGenerator, MapGeneratorConfig, MapRoomNode, RoomType
from core.generation.rewards import generate_card_rewards, RewardState


class Phase(Enum):
    NEOW = "neow"
    MAP = "map"
    COMBAT = "combat"
    REWARDS = "rewards"
    EVENT = "event"
    SHOP = "shop"
    REST = "rest"
    TREASURE = "treasure"
    BOSS_RELIC = "boss_relic"


@dataclass
class GameState:
    """Complete game state for verification."""
    seed: str
    seed_long: int

    # Progress
    act: int = 1
    floor: int = 0
    phase: Phase = Phase.NEOW

    # Resources
    hp: int = 72
    max_hp: int = 72
    gold: int = 99

    # Collections
    deck: List[str] = field(default_factory=list)
    relics: List[str] = field(default_factory=list)
    potions: List[str] = field(default_factory=list)

    # RNG
    rng: GameRNGState = None

    # Reward state (pity timers)
    reward_state: RewardState = field(default_factory=RewardState)

    # Map
    map_nodes: List[List[MapRoomNode]] = None
    position: Tuple[int, int] = (-1, -1)  # (x, y) on map

    # Current context
    current_room_type: Optional[str] = None
    pending_card_reward: Optional[List[str]] = None

    # Neow
    neow_choice: Optional[str] = None
    starting_relic: str = "PureWater"  # Watcher's starting relic

    def __post_init__(self):
        if self.rng is None:
            self.rng = GameRNGState(self.seed)

        # Watcher starting deck
        if not self.deck:
            self.deck = [
                "Strike_P", "Strike_P", "Strike_P", "Strike_P",
                "Defend_P", "Defend_P", "Defend_P", "Defend_P",
                "Eruption", "Vigilance"
            ]

        # Starting relic
        if not self.relics:
            self.relics = [self.starting_relic]


class GameSimulator:
    """
    Step-by-step game simulator with full RNG tracking.

    Usage:
        sim = GameSimulator("33J85JVCVSPJY")
        sim.choose_neow("BOSS_SWAP")
        sim.generate_map()
        sim.enter_room(0, 0)  # Enter first floor node
        sim.complete_combat()  # See card reward
        sim.pick_card(0)  # Pick first card
        sim.enter_room(1, 1)  # Next room
        ...
    """

    def __init__(self, seed: str, ascension: int = 20):
        self.seed = seed.upper()
        self.ascension = ascension

        self.state = GameState(
            seed=self.seed,
            seed_long=seed_to_long(self.seed),
        )

        self.history = []
        self.log_enabled = True

    def log(self, msg: str):
        if self.log_enabled:
            print(msg)
        self.history.append(msg)

    # =========================================================================
    # NEOW
    # =========================================================================

    def choose_neow(self, choice: str) -> Dict[str, Any]:
        """
        Apply Neow blessing.

        Common choices:
        - BOSS_SWAP: Trade starting relic for random boss relic
        - HUNDRED_GOLD: +100 gold
        - RANDOM_COMMON_RELIC: Get a random common relic
        - THREE_CARDS: Choose 1 of 3 cards
        - UPGRADE_CARD: Upgrade a card
        - REMOVE_CARD: Remove a card
        - etc.
        """
        self.log(f"\n{'='*60}")
        self.log(f"NEOW CHOICE: {choice}")
        self.log(f"{'='*60}")

        self.state.neow_choice = choice
        self.state.rng.apply_neow_choice(choice)

        result = {"choice": choice}

        if choice == "BOSS_SWAP":
            # Would need boss relic pool + relicRng
            self.state.relics = ["BossRelicPlaceholder"]
            result["relic"] = "BossRelicPlaceholder"
            self.log(f"  Starting relic traded for boss relic")

        elif choice == "HUNDRED_GOLD":
            self.state.gold += 100
            result["gold"] = self.state.gold
            self.log(f"  Gold: {self.state.gold}")

        elif choice == "RANDOM_COMMON_RELIC":
            self.state.relics.append("CommonRelicPlaceholder")
            result["relic"] = "CommonRelicPlaceholder"

        self.log(f"  cardRng counter: {self.state.rng.get_counter(RNGStream.CARD)}")

        self.state.phase = Phase.MAP
        return result

    # =========================================================================
    # MAP
    # =========================================================================

    def generate_map(self) -> List[Dict]:
        """Generate and display the map."""
        self.log(f"\n{'='*60}")
        self.log(f"GENERATING MAP (Act {self.state.act})")
        self.log(f"{'='*60}")

        map_seed = self.state.seed_long + self.state.act
        config = MapGeneratorConfig(ascension_level=self.ascension)
        rng = Random(map_seed)
        generator = MapGenerator(rng, config)

        self.state.map_nodes = generator.generate()

        # Print map
        self._print_map()

        return self._map_to_dict()

    def _print_map(self):
        """Print ASCII map."""
        if not self.state.map_nodes:
            return

        self.log("\nMap (floor 1 at bottom, floor 15 at top):")
        self.log("-" * 40)

        # Print from top to bottom (highest floor first)
        for y in range(14, -1, -1):
            row = self.state.map_nodes[y]
            line = f"F{y+1:2d} |"
            for x in range(7):
                node = row[x]
                if node.room_type:
                    symbol = node.room_type.value
                    # Mark current position
                    if self.state.position == (x, y):
                        symbol = f"[{symbol}]"
                    else:
                        symbol = f" {symbol} "
                else:
                    symbol = "   "
                line += symbol
            self.log(line)

        self.log("-" * 40)
        self.log("     " + "".join(f" {x} " for x in range(7)))
        self.log("\nLegend: M=Monster, E=Elite, R=Rest, $=Shop, ?=Event, T=Treasure")

    def _map_to_dict(self) -> List[Dict]:
        """Convert map to serializable format."""
        result = []
        for row in self.state.map_nodes:
            row_data = []
            for node in row:
                if node.room_type:
                    row_data.append({
                        "x": node.x,
                        "y": node.y,
                        "type": node.room_type.value,
                        "edges": [(e.dst_x, e.dst_y) for e in node.edges]
                    })
            result.append(row_data)
        return result

    def get_available_paths(self) -> List[Tuple[int, int]]:
        """Get available next nodes to visit."""
        if not self.state.map_nodes:
            return []

        x, y = self.state.position

        if y == -1:
            # At start - can visit any floor 0 node
            return [(node.x, 0) for node in self.state.map_nodes[0] if node.room_type]

        current_node = self.state.map_nodes[y][x]
        return [(e.dst_x, e.dst_y) for e in current_node.edges]

    # =========================================================================
    # ROOM ENTRY
    # =========================================================================

    def enter_room(self, x: int, y: int) -> Dict[str, Any]:
        """
        Enter a room at position (x, y).

        Returns room info and predictions.
        """
        if not self.state.map_nodes:
            self.generate_map()

        # Validate move
        available = self.get_available_paths()
        if (x, y) not in available:
            self.log(f"ERROR: Cannot move to ({x}, {y}). Available: {available}")
            return {"error": f"Invalid move. Available: {available}"}

        self.state.position = (x, y)
        self.state.floor = y + 1
        node = self.state.map_nodes[y][x]
        room_type = node.room_type.value

        self.log(f"\n{'='*60}")
        self.log(f"FLOOR {self.state.floor}: {node.room_type.name} at ({x}, {y})")
        self.log(f"{'='*60}")

        self.state.current_room_type = room_type

        result = {
            "floor": self.state.floor,
            "x": x,
            "y": y,
            "room_type": room_type,
            "room_name": node.room_type.name,
        }

        # Handle room type
        if room_type in ["M", "E", "B"]:
            self.state.phase = Phase.COMBAT
            result["phase"] = "combat"
            self.log(f"  Entering combat...")
            self.log(f"  (Call complete_combat() when combat ends to see card reward)")

        elif room_type == "$":
            self.state.phase = Phase.SHOP
            result.update(self._enter_shop())

        elif room_type == "?":
            self.state.phase = Phase.EVENT
            result["phase"] = "event"
            self.state.rng.apply_event()
            self.log(f"  Event room (uses miscRng)")

        elif room_type == "R":
            self.state.phase = Phase.REST
            result["phase"] = "rest"
            self.log(f"  Rest site options: rest, upgrade")

        elif room_type == "T":
            self.state.phase = Phase.TREASURE
            result.update(self._enter_treasure())

        result["cardRng"] = self.state.rng.get_counter(RNGStream.CARD)
        self.log(f"  cardRng counter: {result['cardRng']}")

        # Show available next paths
        next_paths = self.get_available_paths()
        if next_paths:
            self.log(f"  Next available: {next_paths}")

        return result

    # =========================================================================
    # COMBAT
    # =========================================================================

    def complete_combat(self) -> Dict[str, Any]:
        """
        Complete combat and generate card reward.

        Call this when combat ends in the real game to see predicted card reward.
        """
        if self.state.phase != Phase.COMBAT:
            return {"error": "Not in combat"}

        room_type = self.state.current_room_type
        is_elite = room_type == "E"

        self.log(f"\n--- COMBAT COMPLETE ---")
        self.log(f"  Room type: {'Elite' if is_elite else 'Monster/Boss'}")

        # Generate card reward
        counter_before = self.state.rng.get_counter(RNGStream.CARD)
        card_rng = self.state.rng.get_rng(RNGStream.CARD)

        cards = generate_card_rewards(
            rng=card_rng,
            reward_state=self.state.reward_state,
            act=self.state.act,
            player_class="WATCHER",
            ascension=self.ascension,
            room_type="elite" if is_elite else "normal",
            num_cards=3,
        )

        counter_after = card_rng.counter
        self.state.rng.set_counter(RNGStream.CARD, counter_after)

        card_names = [c.name for c in cards]
        self.state.pending_card_reward = card_names

        self.log(f"\n  CARD REWARD PREDICTION:")
        self.log(f"  cardRng: {counter_before} -> {counter_after}")
        for i, name in enumerate(card_names):
            self.log(f"    [{i}] {name}")

        self.state.phase = Phase.REWARDS

        return {
            "cards": card_names,
            "cardRng_before": counter_before,
            "cardRng_after": counter_after,
            "is_elite": is_elite,
        }

    def pick_card(self, index: int) -> Dict[str, Any]:
        """Pick a card from the reward."""
        if not self.state.pending_card_reward:
            return {"error": "No pending card reward"}

        if index < 0 or index >= len(self.state.pending_card_reward):
            return {"error": f"Invalid index. Choose 0-{len(self.state.pending_card_reward)-1}"}

        card = self.state.pending_card_reward[index]
        self.state.deck.append(card)
        self.state.pending_card_reward = None
        self.state.phase = Phase.MAP

        self.log(f"  Picked: {card}")
        self.log(f"  Deck size: {len(self.state.deck)}")

        return {"picked": card, "deck_size": len(self.state.deck)}

    def skip_card_reward(self) -> Dict[str, Any]:
        """Skip the card reward."""
        self.state.pending_card_reward = None
        self.state.phase = Phase.MAP
        self.log(f"  Skipped card reward")
        return {"skipped": True}

    # =========================================================================
    # SHOP
    # =========================================================================

    def _enter_shop(self) -> Dict[str, Any]:
        """Enter shop and consume RNG."""
        counter_before = self.state.rng.get_counter(RNGStream.CARD)
        self.state.rng.apply_shop()
        counter_after = self.state.rng.get_counter(RNGStream.CARD)

        self.log(f"  Shop entered")
        self.log(f"  cardRng: {counter_before} -> {counter_after} (+{counter_after - counter_before})")

        return {
            "phase": "shop",
            "cardRng_consumed": counter_after - counter_before,
        }

    def leave_shop(self) -> Dict[str, Any]:
        """Leave the shop."""
        self.state.phase = Phase.MAP
        self.log(f"  Left shop")
        return {"left": True}

    # =========================================================================
    # TREASURE
    # =========================================================================

    def _enter_treasure(self) -> Dict[str, Any]:
        """Enter treasure room."""
        self.state.rng.apply_treasure()
        self.log(f"  Treasure room (uses treasureRng)")
        return {"phase": "treasure"}

    # =========================================================================
    # REST
    # =========================================================================

    def rest(self) -> Dict[str, Any]:
        """Rest at campfire (heal 30% HP)."""
        heal = int(self.state.max_hp * 0.3)
        self.state.hp = min(self.state.hp + heal, self.state.max_hp)
        self.state.phase = Phase.MAP
        self.log(f"  Rested: +{heal} HP (now {self.state.hp}/{self.state.max_hp})")
        return {"healed": heal, "hp": self.state.hp}

    def upgrade_card(self, index: int) -> Dict[str, Any]:
        """Upgrade a card at campfire."""
        if index < 0 or index >= len(self.state.deck):
            return {"error": f"Invalid card index"}

        card = self.state.deck[index]
        # Simple upgrade tracking - append "+"
        if not card.endswith("+"):
            self.state.deck[index] = card + "+"
            self.log(f"  Upgraded: {card} -> {card}+")

        self.state.phase = Phase.MAP
        return {"upgraded": card}

    # =========================================================================
    # ACT TRANSITION
    # =========================================================================

    def complete_act(self) -> Dict[str, Any]:
        """Complete current act and transition to next."""
        self.log(f"\n{'='*60}")
        self.log(f"ACT {self.state.act} COMPLETE")
        self.log(f"{'='*60}")

        self.state.act += 1
        self.state.floor = 0
        self.state.position = (-1, -1)

        # RNG snapping
        self.state.rng.transition_to_next_act()

        self.log(f"  Entering Act {self.state.act}")
        self.log(f"  cardRng snapped to: {self.state.rng.get_counter(RNGStream.CARD)}")

        # Generate new map
        self.generate_map()

        return {
            "act": self.state.act,
            "cardRng": self.state.rng.get_counter(RNGStream.CARD),
        }

    # =========================================================================
    # STATE
    # =========================================================================

    def get_state(self) -> Dict[str, Any]:
        """Get current game state for comparison."""
        return {
            "seed": self.state.seed,
            "act": self.state.act,
            "floor": self.state.floor,
            "hp": self.state.hp,
            "max_hp": self.state.max_hp,
            "gold": self.state.gold,
            "deck": self.state.deck.copy(),
            "deck_size": len(self.state.deck),
            "relics": self.state.relics.copy(),
            "potions": self.state.potions.copy(),
            "position": self.state.position,
            "phase": self.state.phase.value,
            "neow_choice": self.state.neow_choice,
            "rng": {
                "card": self.state.rng.get_counter(RNGStream.CARD),
                "monster": self.state.rng.get_counter(RNGStream.MONSTER),
                "event": self.state.rng.get_counter(RNGStream.EVENT),
                "relic": self.state.rng.get_counter(RNGStream.RELIC),
                "treasure": self.state.rng.get_counter(RNGStream.TREASURE),
                "potion": self.state.rng.get_counter(RNGStream.POTION),
                "merchant": self.state.rng.get_counter(RNGStream.MERCHANT),
            }
        }

    def print_state(self):
        """Print current state summary."""
        s = self.get_state()
        self.log(f"\n--- CURRENT STATE ---")
        self.log(f"Act {s['act']}, Floor {s['floor']}")
        self.log(f"HP: {s['hp']}/{s['max_hp']}, Gold: {s['gold']}")
        self.log(f"Deck: {s['deck_size']} cards")
        self.log(f"Relics: {s['relics']}")
        self.log(f"cardRng: {s['rng']['card']}")


# =============================================================================
# INTERACTIVE CLI
# =============================================================================

def interactive_session(seed: str = "33J85JVCVSPJY"):
    """Run an interactive session."""
    print(f"\n{'='*60}")
    print(f"STS GAME SIMULATOR - Interactive Mode")
    print(f"{'='*60}")
    print(f"Seed: {seed}")
    print(f"\nCommands:")
    print(f"  neow <choice>   - Choose Neow (BOSS_SWAP, HUNDRED_GOLD, etc)")
    print(f"  map             - Show map")
    print(f"  go <x> <y>      - Enter room at position")
    print(f"  combat          - Complete combat (show card reward)")
    print(f"  pick <n>        - Pick card n from reward")
    print(f"  skip            - Skip card reward")
    print(f"  rest            - Rest at campfire")
    print(f"  upgrade <n>     - Upgrade card n")
    print(f"  leave           - Leave shop")
    print(f"  state           - Show current state")
    print(f"  next            - Complete act")
    print(f"  quit            - Exit")
    print()

    sim = GameSimulator(seed)

    while True:
        try:
            cmd = input("\n> ").strip().lower().split()
            if not cmd:
                continue

            if cmd[0] == "quit":
                break
            elif cmd[0] == "neow":
                choice = cmd[1].upper() if len(cmd) > 1 else "BOSS_SWAP"
                sim.choose_neow(choice)
            elif cmd[0] == "map":
                sim.generate_map()
            elif cmd[0] == "go":
                x, y = int(cmd[1]), int(cmd[2])
                sim.enter_room(x, y)
            elif cmd[0] == "combat":
                sim.complete_combat()
            elif cmd[0] == "pick":
                sim.pick_card(int(cmd[1]))
            elif cmd[0] == "skip":
                sim.skip_card_reward()
            elif cmd[0] == "rest":
                sim.rest()
            elif cmd[0] == "upgrade":
                sim.upgrade_card(int(cmd[1]))
            elif cmd[0] == "leave":
                sim.leave_shop()
            elif cmd[0] == "state":
                sim.print_state()
            elif cmd[0] == "next":
                sim.complete_act()
            elif cmd[0] == "paths":
                print(f"Available paths: {sim.get_available_paths()}")
            else:
                print(f"Unknown command: {cmd[0]}")

        except (KeyboardInterrupt, EOFError):
            break
        except Exception as e:
            print(f"Error: {e}")


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Game simulator")
    parser.add_argument("seed", nargs="?", default="33J85JVCVSPJY")
    parser.add_argument("-i", "--interactive", action="store_true", help="Interactive mode")

    args = parser.parse_args()

    if args.interactive:
        interactive_session(args.seed)
    else:
        # Quick demo
        sim = GameSimulator(args.seed)
        sim.choose_neow("BOSS_SWAP")
        sim.generate_map()

        print("\n" + "="*60)
        print("Simulator ready. Use interactive mode (-i) to step through.")
        print("Or import GameSimulator in Python:")
        print("  from core.comparison.game_simulator import GameSimulator")
        print(f"  sim = GameSimulator('{args.seed}')")
        print("  sim.choose_neow('BOSS_SWAP')")
        print("  sim.enter_room(x, y)")
        print("  sim.complete_combat()")
