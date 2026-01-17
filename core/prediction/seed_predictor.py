"""
Comprehensive Seed Predictor for Slay the Spire

Provides easy-to-use interface for predicting all RNG-dependent outcomes.
Uses the counter-based GameRNGState for accurate predictions.

Usage:
    from core.prediction.seed_predictor import SeedPredictor

    # Simple usage (no Neow choice specified)
    predictor = SeedPredictor("TEST123")
    print(predictor.neow)
    print(predictor.encounters[:3])
    print(predictor.card_rewards[:3])

    # With Neow choice for accurate card predictions
    predictor = SeedPredictor("TEST123", neow_choice="HUNDRED_GOLD")
    print(predictor.card_rewards[:3])

    # With path simulation
    predictor = SeedPredictor("TEST123")
    predictor.apply_neow("HUNDRED_GOLD")
    predictor.apply_combat(1)
    predictor.apply_shop()
    print(predictor.predict_next_cards())
"""

import os
import importlib.util
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple, Union

# Load modules
_core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def _load_module(name: str, filepath: str):
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

_rng_mod = _load_module("rng", os.path.join(_core_dir, "state", "rng.py"))
_rewards_mod = _load_module("rewards", os.path.join(_core_dir, "generation", "rewards.py"))
_encounters_mod = _load_module("encounters", os.path.join(_core_dir, "generation", "encounters.py"))
_relics_mod = _load_module("relics", os.path.join(_core_dir, "generation", "relics.py"))
_game_rng_mod = _load_module("game_rng", os.path.join(_core_dir, "state", "game_rng.py"))

GameRNGState = _game_rng_mod.GameRNGState
RNGStream = _game_rng_mod.RNGStream


# ============================================================================
# DATA CLASSES
# ============================================================================

@dataclass
class NeowOptions:
    """Neow's blessing options."""
    option1: str  # Category 0: Basic reward
    option2: str  # Category 1: Small reward
    option3_drawback: str  # Category 2 drawback
    option3_reward: str  # Category 2 reward
    boss_swap_relic: str = "Unknown"  # The actual boss relic from swap
    calling_bell_relics: Optional[Tuple[str, str, str]] = None  # If boss is Calling Bell

    def __str__(self):
        result = (f"1: {self.option1}\n"
                  f"2: {self.option2}\n"
                  f"3: {self.option3_drawback} -> {self.option3_reward}\n"
                  f"4: BOSS_RELIC ({self.boss_swap_relic})")
        if self.calling_bell_relics:
            c, u, r = self.calling_bell_relics
            result += f"\n   -> Calling Bell grants: {c} (C), {u} (U), {r} (R)"
        return result


@dataclass
class Encounter:
    """A single encounter."""
    floor: int
    enemy: str
    hp: Optional[int]  # None for multi-enemy
    floor_type: str  # "WEAK", "STRONG", "ELITE"

    def __str__(self):
        hp_str = f"{self.hp} HP" if self.hp else "multi-enemy"
        return f"Floor {self.floor}: {self.enemy} ({hp_str}) [{self.floor_type}]"


@dataclass
class CardReward:
    """Card reward for a floor."""
    floor: int
    cards: List[Tuple[str, str]]  # [(name, rarity), ...]

    def __str__(self):
        cards_str = ", ".join([f"{name} ({rarity[0]})" for name, rarity in self.cards])
        return f"Floor {self.floor}: {cards_str}"


@dataclass
class EventResult:
    """Result of entering a ? room."""
    floor: int
    roll: float
    room_type: str  # "EVENT", "MONSTER", "SHOP", "TREASURE", "ELITE"
    event_name: Optional[str]  # If room_type is EVENT

    def __str__(self):
        if self.event_name:
            return f"Floor {self.floor}: {self.room_type} - {self.event_name}"
        return f"Floor {self.floor}: {self.room_type}"


# ============================================================================
# CONSTANTS
# ============================================================================

NEOW_CATEGORY_0 = [
    "THREE_CARDS", "ONE_RANDOM_RARE_CARD", "REMOVE_CARD",
    "UPGRADE_CARD", "TRANSFORM_CARD", "RANDOM_COLORLESS",
]

NEOW_CATEGORY_1 = [
    "THREE_SMALL_POTIONS", "RANDOM_COMMON_RELIC", "TEN_PERCENT_HP_BONUS",
    "THREE_ENEMY_KILL", "HUNDRED_GOLD",
]

NEOW_DRAWBACKS = [
    "TEN_PERCENT_HP_LOSS", "NO_GOLD", "CURSE", "PERCENT_DAMAGE",
]

NEOW_CATEGORY_2 = [
    "RANDOM_COLORLESS_2", "REMOVE_TWO", "ONE_RARE_RELIC",
    "THREE_RARE_CARDS", "TWO_FIFTY_GOLD", "TRANSFORM_TWO_CARDS",
    "TWENTY_PERCENT_HP_BONUS",
]

ACT1_EVENTS = [
    "Big Fish", "The Cleric", "Dead Adventurer", "Golden Idol",
    "Golden Wing", "World of Goop", "Liars Game", "Living Wall",
    "Mushrooms", "Scrap Ooze", "Shining Light",
]

ACT1_SHRINES = [
    "Match and Keep!", "Golden Shrine", "Transmorgrifier",
    "Purifier", "Upgrade Shrine", "Wheel of Change",
]

# cardRng consumption by Neow choice
# Maps (drawback, reward) or simple option to cardRng consumption
NEOW_CARDRNG_CONSUMPTION = {
    # Simple options (Category 0 and 1)
    "UPGRADE_CARD": 0,
    "HUNDRED_GOLD": 0,
    "TEN_PERCENT_HP_BONUS": 0,
    "RANDOM_COMMON_RELIC": 0,
    "THREE_ENEMY_KILL": 0,
    "THREE_CARDS": 0,  # Uses NeowEvent.rng
    "ONE_RANDOM_RARE_CARD": 0,  # Uses NeowEvent.rng
    "TRANSFORM_CARD": 0,  # Uses NeowEvent.rng
    "REMOVE_CARD": 0,
    "THREE_SMALL_POTIONS": 0,

    # Category 2 rewards
    "REMOVE_TWO": 0,
    "ONE_RARE_RELIC": 0,
    "THREE_RARE_CARDS": 0,  # Uses NeowEvent.rng
    "TWO_FIFTY_GOLD": 0,
    "TRANSFORM_TWO_CARDS": 0,
    "TWENTY_PERCENT_HP_BONUS": 0,

    # Colorless options consume cardRng
    "RANDOM_COLORLESS": 3,
    "RANDOM_COLORLESS_2": 3,

    # Drawbacks
    "TEN_PERCENT_HP_LOSS": 0,
    "NO_GOLD": 0,
    "PERCENT_DAMAGE": 0,
    "CURSE": 1,  # Curse selection consumes 1 cardRng call

    # Boss swap
    "BOSS_SWAP": 0,
    "BOSS_SWAP_CALLING_BELL": 9,  # Calling Bell consumes 9 cardRng calls
}


# ============================================================================
# SEED PREDICTOR
# ============================================================================

class SeedPredictor:
    """
    Comprehensive seed prediction for Slay the Spire.

    Uses GameRNGState for accurate counter-based predictions.
    Supports path simulation for predictions after shops/events.
    """

    def __init__(self, seed_str: str, player_class: str = "WATCHER",
                 neow_choice: Optional[str] = None,
                 neow_drawback: Optional[str] = None,
                 boss_relic: Optional[str] = None,
                 card_rng_floor_offset: int = 0):
        """
        Initialize seed predictor.

        Args:
            seed_str: The seed string (e.g., "TEST123")
            player_class: Player class (default "WATCHER")
            neow_choice: The Neow option chosen (e.g., "HUNDRED_GOLD", "BOSS_SWAP")
            neow_drawback: The drawback if choosing option 3 (e.g., "CURSE")
            boss_relic: The boss relic received (for Calling Bell detection)
            card_rng_floor_offset: DEPRECATED - use neow_choice instead.
                Kept for backward compatibility.
        """
        self.seed_str = seed_str.upper()
        self.seed = _rng_mod.seed_to_long(self.seed_str)
        self.player_class = player_class

        # Initialize RNG state
        self._rng_state = GameRNGState(self.seed_str)

        # Apply Neow choice if provided
        if neow_choice:
            self._apply_neow_choice(neow_choice, neow_drawback, boss_relic)
        elif card_rng_floor_offset > 0:
            # Backward compatibility: offset = floors to skip
            # Each floor consumes ~9 cardRng calls
            self._rng_state.set_counter(RNGStream.CARD, card_rng_floor_offset * 9)

        # Cached results
        self._neow: Optional[NeowOptions] = None
        self._encounters: Optional[List[Encounter]] = None
        self._elite_encounters: Optional[List[str]] = None
        self._card_rewards: Optional[List[CardReward]] = None
        self._event_results: Dict[int, EventResult] = {}
        self._boss_relic_pool: Optional[List[str]] = None

    def _apply_neow_choice(self, choice: str, drawback: str = None, boss_relic: str = None):
        """Apply cardRng consumption for Neow choice."""
        consumption = 0

        # Check for Calling Bell specifically
        if choice == "BOSS_SWAP" and boss_relic == "Calling Bell":
            consumption = NEOW_CARDRNG_CONSUMPTION["BOSS_SWAP_CALLING_BELL"]
        elif choice in NEOW_CARDRNG_CONSUMPTION:
            consumption = NEOW_CARDRNG_CONSUMPTION[choice]

        # Add drawback consumption
        if drawback and drawback in NEOW_CARDRNG_CONSUMPTION:
            consumption += NEOW_CARDRNG_CONSUMPTION[drawback]

        self._rng_state.advance(RNGStream.CARD, consumption)

    # ========================================================================
    # PATH SIMULATION API
    # ========================================================================

    def apply_neow(self, choice: str, drawback: str = None, boss_relic: str = None):
        """Apply Neow choice to RNG state."""
        self._apply_neow_choice(choice, drawback, boss_relic)
        self._card_rewards = None  # Invalidate cache

    def apply_combat(self, floor: int = None):
        """Apply combat reward RNG consumption (~9 cardRng calls)."""
        if floor:
            self._rng_state.enter_floor(floor)
        self._rng_state.apply_combat("monster")
        self._card_rewards = None

    def apply_elite(self, floor: int = None):
        """Apply elite combat reward RNG consumption."""
        if floor:
            self._rng_state.enter_floor(floor)
        self._rng_state.apply_combat("elite")
        self._card_rewards = None

    def apply_shop(self):
        """Apply shop RNG consumption (~12 cardRng calls)."""
        self._rng_state.apply_shop()
        self._card_rewards = None

    def apply_event(self, event_name: str = None):
        """Apply event RNG consumption (usually 0 for cardRng)."""
        self._rng_state.apply_event(event_name)
        self._card_rewards = None

    def apply_treasure(self):
        """Apply treasure room RNG consumption (0 cardRng)."""
        self._rng_state.apply_treasure()

    def enter_floor(self, floor: int):
        """Enter a new floor."""
        self._rng_state.enter_floor(floor)

    def predict_next_cards(self) -> List[Tuple[str, str]]:
        """Predict the next card reward at current RNG state."""
        cards = _game_rng_mod.predict_card_reward(self._rng_state, self.player_class)
        return cards

    def get_card_counter(self) -> int:
        """Get current cardRng counter."""
        return self._rng_state.get_counter(RNGStream.CARD)

    def set_card_counter(self, value: int):
        """Set cardRng counter directly (for testing)."""
        self._rng_state.set_counter(RNGStream.CARD, value)
        self._card_rewards = None

    # ========================================================================
    # PROPERTIES (Cached Predictions)
    # ========================================================================

    @property
    def neow(self) -> NeowOptions:
        """Get Neow's blessing options."""
        if self._neow is None:
            self._neow = self._predict_neow()
        return self._neow

    @property
    def encounters(self) -> List[Encounter]:
        """Get all normal encounters for Act 1."""
        if self._encounters is None:
            self._generate_encounters()
        return self._encounters

    @property
    def elite_encounters(self) -> List[str]:
        """Get all elite encounters for Act 1."""
        if self._elite_encounters is None:
            self._generate_encounters()
        return self._elite_encounters

    def card_reward(self, floor: int) -> CardReward:
        """Get card reward for a specific floor."""
        if self._card_rewards is None:
            self._generate_card_rewards(max_floor=15)

        if floor <= len(self._card_rewards):
            return self._card_rewards[floor - 1]
        else:
            self._generate_card_rewards(max_floor=floor)
            return self._card_rewards[floor - 1]

    @property
    def card_rewards(self) -> List[CardReward]:
        """Get first 15 card rewards."""
        if self._card_rewards is None:
            self._generate_card_rewards(max_floor=15)
        return self._card_rewards

    def event_at_floor(self, floor: int, event_rng_counter: int = 0) -> EventResult:
        """Predict event result at a specific floor."""
        key = (floor, event_rng_counter)
        if key not in self._event_results:
            self._event_results[key] = self._predict_event(floor, event_rng_counter)
        return self._event_results[key]

    @property
    def boss_relic_pool(self) -> List[str]:
        """Get the shuffled boss relic pool for this seed."""
        if self._boss_relic_pool is None:
            self._boss_relic_pool = _relics_mod.predict_boss_relic_pool(
                self.seed, self.player_class
            )
        return self._boss_relic_pool

    @property
    def rng_state(self) -> GameRNGState:
        """Get the underlying RNG state for advanced usage."""
        return self._rng_state

    # ========================================================================
    # INTERNAL METHODS
    # ========================================================================

    def _predict_neow(self) -> NeowOptions:
        """Generate Neow options."""
        rng = _rng_mod.Random(self.seed)

        cat0_idx = rng.random_int_range(0, len(NEOW_CATEGORY_0) - 1)
        cat1_idx = rng.random_int_range(0, len(NEOW_CATEGORY_1) - 1)

        drawback_idx = rng.random_int_range(0, len(NEOW_DRAWBACKS) - 1)
        drawback = NEOW_DRAWBACKS[drawback_idx]

        # Filter category 2 based on drawback
        cat2_opts = NEOW_CATEGORY_2.copy()
        if drawback == "CURSE":
            cat2_opts.remove("REMOVE_TWO")
        if drawback == "NO_GOLD":
            cat2_opts.remove("TWO_FIFTY_GOLD")
        if drawback == "TEN_PERCENT_HP_LOSS":
            cat2_opts.remove("TWENTY_PERCENT_HP_BONUS")

        cat2_idx = rng.random_int_range(0, len(cat2_opts) - 1)

        # Predict boss swap relic
        boss_swap = _relics_mod.predict_neow_boss_swap(self.seed, self.player_class)

        # If Calling Bell, also predict the 3 relics it grants
        calling_bell_relics = None
        if boss_swap == "Calling Bell":
            calling_bell_relics = _relics_mod.predict_calling_bell_relics(
                self.seed, self.player_class
            )

        return NeowOptions(
            option1=NEOW_CATEGORY_0[cat0_idx],
            option2=NEOW_CATEGORY_1[cat1_idx],
            option3_drawback=drawback,
            option3_reward=cat2_opts[cat2_idx],
            boss_swap_relic=boss_swap,
            calling_bell_relics=calling_bell_relics,
        )

    def _generate_encounters(self):
        """Generate all Act 1 encounters."""
        monster_rng = _rng_mod.Random(self.seed)
        normal_list, elite_list = _encounters_mod.generate_exordium_encounters(monster_rng)

        self._encounters = []
        for i, enemy in enumerate(normal_list):
            floor = i + 1
            floor_type = "WEAK" if floor <= 3 else "STRONG"

            hp_rng = _rng_mod.Random(self.seed + floor)
            hp = _encounters_mod.get_enemy_hp(enemy, hp_rng)
            hp = hp if hp > 0 else None

            self._encounters.append(Encounter(
                floor=floor, enemy=enemy, hp=hp, floor_type=floor_type
            ))

        self._elite_encounters = elite_list

    def _generate_card_rewards(self, max_floor: int = 15):
        """Generate card rewards using current RNG state."""
        # Clone state to not affect the main state
        temp_state = self._rng_state.clone()

        self._card_rewards = []
        for floor in range(1, max_floor + 1):
            temp_state.enter_floor(floor)

            # Generate cards at current counter
            cards = _game_rng_mod.predict_card_reward(temp_state, self.player_class)
            card_info = [(c[0], c[1]) for c in cards]
            self._card_rewards.append(CardReward(floor=floor, cards=card_info))

            # Advance counter for next floor (simulate taking the reward)
            # Each card reward consumes ~9 cardRng calls
            temp_state.advance(RNGStream.CARD, 9)

    def _predict_event(self, floor: int, event_rng_counter: int) -> EventResult:
        """Predict event room outcome."""
        rng = _rng_mod.Random(self.seed)
        for _ in range(event_rng_counter):
            rng.random_float()

        # Base probabilities
        elite_chance = 0.10 if floor >= 6 else 0
        monster_chance = 0.10
        shop_chance = 0.03
        treasure_chance = 0.02

        roll = rng.random_float()

        # Determine room type
        cumulative = 0
        room_type = "EVENT"

        if roll < (cumulative := cumulative + elite_chance):
            room_type = "ELITE"
        elif roll < (cumulative := cumulative + monster_chance):
            room_type = "MONSTER"
        elif roll < (cumulative := cumulative + shop_chance):
            room_type = "SHOP"
        elif roll < (cumulative := cumulative + treasure_chance):
            room_type = "TREASURE"

        event_name = None
        if room_type == "EVENT":
            shrine_roll = rng.random_float()
            if shrine_roll < 0.25:
                shrine_idx = rng.random_int_range(0, len(ACT1_SHRINES) - 1)
                event_name = ACT1_SHRINES[shrine_idx]
            else:
                available = [e for e in ACT1_EVENTS
                           if e not in ["Dead Adventurer", "Mushrooms"] or floor > 6]
                event_idx = rng.random_int_range(0, len(available) - 1)
                event_name = available[event_idx]

        return EventResult(floor=floor, roll=roll, room_type=room_type, event_name=event_name)

    def summary(self) -> str:
        """Get a full summary of the seed."""
        lines = [
            f"{'='*60}",
            f"SEED: {self.seed_str} ({self.seed})",
            f"cardRng counter: {self.get_card_counter()}",
            f"{'='*60}",
            "",
            "=== NEOW BLESSINGS ===",
            str(self.neow),
            "",
            "=== ENCOUNTERS (Floors 1-5) ===",
        ]

        for enc in self.encounters[:5]:
            lines.append(f"  {enc}")

        lines.extend([
            "",
            "=== CARD REWARDS (Floors 1-5) ===",
        ])

        for cr in self.card_rewards[:5]:
            lines.append(f"  {cr}")

        lines.extend([
            "",
            "=== FIRST ? ROOM (Floor 3) ===",
            f"  {self.event_at_floor(3)}",
        ])

        return "\n".join(lines)

    def __str__(self):
        return self.summary()


# ============================================================================
# CLI
# ============================================================================

def main():
    import sys
    seed = sys.argv[1] if len(sys.argv) > 1 else "TEST123"
    neow = sys.argv[2] if len(sys.argv) > 2 else None

    predictor = SeedPredictor(seed, neow_choice=neow)
    print(predictor.summary())


if __name__ == "__main__":
    main()
