"""
Shared pytest fixtures for Slay the Spire RL test suite.

This module provides reusable fixtures for:
- Combat state creation
- RNG with known seeds
- Mock game states
- Enemy and player configurations
"""

import pytest
import sys

# Ensure project root is in path - works for both main repo and worktrees
# Ensure project root is in path (use the correct path for worktrees)
import os
_project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, _project_root)

from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    PlayCard, UsePotion, EndTurn,
)
from packages.engine.state.rng import XorShift128, Random, GameRNG, seed_to_long


# =============================================================================
# RNG Fixtures
# =============================================================================


@pytest.fixture
def rng_seed_42():
    """RNG initialized with seed 42 for deterministic tests."""
    return Random(42)


@pytest.fixture
def rng_seed_12345():
    """RNG initialized with seed 12345 for deterministic tests."""
    return Random(12345)


@pytest.fixture
def game_rng_abc():
    """GameRNG initialized with seed 'ABC' - a common test seed."""
    return GameRNG(seed_to_long("ABC"))


@pytest.fixture
def game_rng_1abcd():
    """GameRNG initialized with seed '1ABCD' - verified game seed."""
    return GameRNG(seed_to_long("1ABCD"))


@pytest.fixture
def known_seeds():
    """Collection of known seed strings with verified properties."""
    return {
        "ABC": seed_to_long("ABC"),
        "1ABCD": seed_to_long("1ABCD"),
        "TEST123": seed_to_long("TEST123"),
        "0": seed_to_long("0"),
        "ZZZZZ": seed_to_long("ZZZZZ"),
    }


@pytest.fixture
def xorshift_deterministic():
    """XorShift128 with explicit state for reproducible sequences."""
    return XorShift128(0x12345678, 0x87654321)


# =============================================================================
# Player State Fixtures
# =============================================================================


@pytest.fixture
def player_full_hp():
    """Player with full HP (80/80)."""
    return EntityState(hp=80, max_hp=80, block=0, statuses={})


@pytest.fixture
def player_low_hp():
    """Player with low HP (20/80) - danger zone."""
    return EntityState(hp=20, max_hp=80, block=0, statuses={})


@pytest.fixture
def player_with_strength():
    """Player with +3 Strength."""
    return EntityState(hp=80, max_hp=80, block=0, statuses={"Strength": 3})


@pytest.fixture
def player_with_dexterity():
    """Player with +3 Dexterity."""
    return EntityState(hp=80, max_hp=80, block=0, statuses={"Dexterity": 3})


@pytest.fixture
def player_debuffed():
    """Player with common debuffs (Weak, Vulnerable, Frail)."""
    return EntityState(
        hp=60, max_hp=80, block=0,
        statuses={"Weak": 2, "Vulnerable": 2, "Frail": 2}
    )


@pytest.fixture
def player_with_artifact():
    """Player with Artifact stacks (blocks debuffs)."""
    return EntityState(hp=80, max_hp=80, block=0, statuses={"Artifact": 2})


# =============================================================================
# Enemy Fixtures
# =============================================================================


@pytest.fixture
def jaw_worm():
    """Standard Jaw Worm enemy."""
    return EnemyCombatState(
        hp=44, max_hp=44, block=0,
        statuses={},
        id="JawWorm",
        move_id=0,
        move_damage=11,
        move_hits=1,
        move_block=0,
        move_effects={}
    )


@pytest.fixture
def louse():
    """Standard Red Louse enemy."""
    return EnemyCombatState(
        hp=15, max_hp=15, block=0,
        statuses={},
        id="Louse",
        move_id=0,
        move_damage=6,
        move_hits=1,
        move_block=0,
        move_effects={"Weak": 1}
    )


@pytest.fixture
def gremlin_nob():
    """Gremlin Nob elite (rage-on-skill mechanic)."""
    return EnemyCombatState(
        hp=82, max_hp=82, block=0,
        statuses={},
        id="GremlinNob",
        move_id=0,
        move_damage=14,
        move_hits=1,
        move_block=0,
        move_effects={}
    )


@pytest.fixture
def vulnerable_enemy():
    """Enemy with Vulnerable status."""
    return EnemyCombatState(
        hp=50, max_hp=50, block=0,
        statuses={"Vulnerable": 2},
        id="TestEnemy",
        move_id=0,
        move_damage=10,
        move_hits=1,
        move_block=0,
        move_effects={}
    )


@pytest.fixture
def multi_hit_enemy():
    """Enemy with a multi-hit attack."""
    return EnemyCombatState(
        hp=60, max_hp=60, block=0,
        statuses={},
        id="MultiHitter",
        move_id=0,
        move_damage=6,
        move_hits=5,  # 6x5 = 30 total
        move_block=0,
        move_effects={}
    )


@pytest.fixture
def blocking_enemy():
    """Enemy that gains block."""
    return EnemyCombatState(
        hp=40, max_hp=40, block=10,
        statuses={},
        id="Blocker",
        move_id=0,
        move_damage=0,
        move_hits=0,
        move_block=15,
        move_effects={}
    )


# =============================================================================
# Deck Fixtures
# =============================================================================


@pytest.fixture
def watcher_starter_deck():
    """Watcher starting deck card IDs."""
    return [
        "Strike_P", "Strike_P", "Strike_P", "Strike_P",
        "Defend_P", "Defend_P", "Defend_P", "Defend_P",
        "Eruption", "Vigilance"
    ]


@pytest.fixture
def ironclad_starter_deck():
    """Ironclad starting deck card IDs."""
    return [
        "Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R",
        "Defend_R", "Defend_R", "Defend_R", "Defend_R",
        "Bash"
    ]


@pytest.fixture
def minimal_deck():
    """Minimal deck for fast tests (5 cards)."""
    return ["Strike_P", "Strike_P", "Defend_P", "Defend_P", "Eruption"]


@pytest.fixture
def all_attacks_deck():
    """Deck with only attack cards."""
    return ["Strike_P"] * 10


@pytest.fixture
def all_skills_deck():
    """Deck with only skill cards."""
    return ["Defend_P"] * 10


# =============================================================================
# Combat State Fixtures
# =============================================================================


def create_combat_state(
    player_hp: int = 80,
    player_max_hp: int = 80,
    enemies: list = None,
    deck: list = None,
    hand: list = None,
    energy: int = 3,
    max_energy: int = 3,
    stance: str = "Neutral",
    potions: list = None,
    relics: list = None,
) -> CombatState:
    """
    Factory function to create combat states with sensible defaults.

    Args:
        player_hp: Current player HP
        player_max_hp: Maximum player HP
        enemies: List of EnemyCombatState objects
        deck: List of card IDs for draw pile
        hand: List of card IDs for starting hand (overrides deck draw)
        energy: Starting energy
        max_energy: Maximum energy per turn
        stance: Starting stance (Neutral, Calm, Wrath, Divinity)
        potions: List of potion IDs
        relics: List of relic IDs

    Returns:
        Configured CombatState
    """
    if enemies is None:
        enemies = [EnemyCombatState(
            hp=44, max_hp=44, block=0,
            statuses={},
            id="JawWorm",
            move_id=0,
            move_damage=11,
            move_hits=1,
            move_block=0,
            move_effects={}
        )]

    if deck is None:
        deck = [
            "Strike_P", "Strike_P", "Strike_P", "Strike_P",
            "Defend_P", "Defend_P", "Defend_P", "Defend_P",
            "Eruption", "Vigilance"
        ]

    # Create player state
    player = EntityState(hp=player_hp, max_hp=player_max_hp, block=0, statuses={})

    # Set up card piles
    if hand is not None:
        hand_cards = hand
        draw_pile = [c for c in deck if c not in hand]
    else:
        hand_cards = deck[:5] if len(deck) >= 5 else deck.copy()
        draw_pile = deck[5:] if len(deck) > 5 else []

    return CombatState(
        player=player,
        energy=energy,
        max_energy=max_energy,
        stance=stance,
        hand=hand_cards,
        draw_pile=draw_pile,
        discard_pile=[],
        exhaust_pile=[],
        enemies=enemies,
        potions=potions or [],
        relics=relics or [],
        turn=1,
        cards_played_this_turn=0,
        attacks_played_this_turn=0,
        skills_played_this_turn=0,
        powers_played_this_turn=0,
        relic_counters={},
        card_costs={},
    )


@pytest.fixture
def basic_combat(jaw_worm, watcher_starter_deck):
    """Basic combat: single Jaw Worm, starter deck, full resources."""
    return create_combat_state(
        player_hp=80,
        player_max_hp=80,
        enemies=[jaw_worm],
        deck=watcher_starter_deck,
        energy=3,
    )


@pytest.fixture
def multi_enemy_combat():
    """Combat with three Louse enemies."""
    enemies = [
        EnemyCombatState(
            hp=15, max_hp=15, block=0,
            statuses={},
            id="Louse",
            move_id=0,
            move_damage=6,
            move_hits=1,
            move_block=0,
            move_effects={}
        )
        for _ in range(3)
    ]
    return create_combat_state(enemies=enemies)


@pytest.fixture
def stance_combat():
    """Combat setup for testing stance mechanics."""
    stance_deck = [
        "Eruption", "Vigilance",  # Basic stance cards
        "EmptyFist", "EmptyBody",  # Neutral stance cards
        "Crescendo", "Tranquility",  # Stance exit cards
        "InnerPeace", "FearNoEvil",  # Stance-based draws
    ]
    return create_combat_state(
        deck=stance_deck,
        hand=["Eruption", "Vigilance", "EmptyFist", "Crescendo", "Tranquility"],
    )


@pytest.fixture
def wrath_combat():
    """Combat starting in Wrath stance."""
    return create_combat_state(stance="Wrath")


@pytest.fixture
def calm_combat():
    """Combat starting in Calm stance."""
    return create_combat_state(stance="Calm")


@pytest.fixture
def low_energy_combat():
    """Combat with only 1 energy (for testing energy constraints)."""
    return create_combat_state(energy=1, max_energy=3)


@pytest.fixture
def high_energy_combat():
    """Combat with 5 energy (for testing high-cost cards)."""
    return create_combat_state(energy=5, max_energy=5)


@pytest.fixture
def late_game_combat():
    """Late game combat: thin deck, good relics, more HP."""
    thin_deck = [
        "Ragnarok", "Omniscience", "Devotion",
        "Crescendo", "Tantrum", "Conclude",
    ]
    relics = ["VioletLotus", "MentalFortress", "DamselDagger"]
    return create_combat_state(
        player_hp=70,
        player_max_hp=75,
        deck=thin_deck,
        energy=4,
        max_energy=4,
        relics=relics,
    )


@pytest.fixture
def potion_combat():
    """Combat with potions available."""
    return create_combat_state(
        potions=["Fire Potion", "Block Potion", "Speed Potion"],
    )


# =============================================================================
# Game State Fixtures (for integration tests)
# =============================================================================


@pytest.fixture
def mock_game_state():
    """Mock game state for testing game-level logic."""
    return {
        "seed": "TEST123",
        "ascension": 0,
        "character": "WATCHER",
        "act": 1,
        "floor": 1,
        "gold": 99,
        "hp": 80,
        "max_hp": 80,
        "potions": [],
        "relics": ["PureWater"],
        "deck": [
            "Strike_P", "Strike_P", "Strike_P", "Strike_P",
            "Defend_P", "Defend_P", "Defend_P", "Defend_P",
            "Eruption", "Vigilance"
        ],
    }


@pytest.fixture
def ascension_20_state():
    """Game state at Ascension 20."""
    return {
        "seed": "A20TEST",
        "ascension": 20,
        "character": "WATCHER",
        "act": 1,
        "floor": 1,
        "gold": 99,
        "hp": 72,  # A20 starts with lower HP
        "max_hp": 72,
        "potions": [],
        "relics": ["PureWater"],
        "deck": [
            "Strike_P", "Strike_P", "Strike_P", "Strike_P",
            "Defend_P", "Defend_P", "Defend_P", "Defend_P",
            "Eruption", "Vigilance", "Ascender's Bane"  # A10+ adds curse
        ],
    }


# =============================================================================
# Reward State Fixtures
# =============================================================================


@pytest.fixture
def fresh_reward_state():
    """Fresh reward state (no rewards collected yet)."""
    from packages.engine.generation.rewards import RewardState
    return RewardState()


@pytest.fixture
def mid_act_reward_state():
    """Reward state after some Act 1 rewards (simulated)."""
    from packages.engine.generation.rewards import RewardState, CardBlizzardState, PotionBlizzardState
    state = RewardState()
    # Simulate some card rewards lowering the blizzard offset
    state.card_blizzard.offset = 0
    # Mark some relics as seen
    state.seen_relics = {"Vajra", "Lantern", "Orichalcum"}
    return state


# =============================================================================
# Pytest Configuration
# =============================================================================


def pytest_configure(config):
    """Configure custom markers."""
    config.addinivalue_line("markers", "slow: marks test as slow (deselect with -m 'not slow')")
    config.addinivalue_line("markers", "integration: marks test as integration test")
    config.addinivalue_line("markers", "unit: marks test as unit test")
    config.addinivalue_line("markers", "rng: marks test as RNG-related")
    config.addinivalue_line("markers", "combat: marks test as combat-related")
    config.addinivalue_line("markers", "cards: marks test as card-related")
    config.addinivalue_line("markers", "parity: marks test as parity verification")


# =============================================================================
# Test Utilities (available to all tests)
# =============================================================================


def assert_damage_in_range(actual: int, expected: int, tolerance: int = 1):
    """Assert damage is within expected range (for float rounding)."""
    assert expected - tolerance <= actual <= expected + tolerance, \
        f"Damage {actual} not in range [{expected - tolerance}, {expected + tolerance}]"


def count_card_type(hand: list, card_type: str, card_registry: dict = None) -> int:
    """Count cards of a specific type in hand."""
    # Simple heuristic if no registry
    if card_registry is None:
        attacks = {"Strike_P", "Strike_R", "Eruption", "Tantrum", "Ragnarok", "Conclude"}
        skills = {"Defend_P", "Defend_R", "Vigilance", "InnerPeace", "EmptyBody"}
        powers = {"Devotion", "Fasting", "DevaForm", "MentalFortress"}

        if card_type.lower() == "attack":
            return sum(1 for c in hand if c in attacks)
        elif card_type.lower() == "skill":
            return sum(1 for c in hand if c in skills)
        elif card_type.lower() == "power":
            return sum(1 for c in hand if c in powers)
    return 0


# Export utility functions for use in tests
@pytest.fixture
def damage_assertion():
    """Provides damage assertion utility."""
    return assert_damage_in_range


@pytest.fixture
def card_counter():
    """Provides card counting utility."""
    return count_card_type
