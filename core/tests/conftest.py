"""
Pytest fixtures for Slay the Spire RL test suite.

Provides shared fixtures for testing simulation, combat, and state modules.
"""

import sys
import os
import pytest
from typing import List


# Register custom pytest markers
def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line(
        "markers", "slow: marks tests as slow (deselect with '-m \"not slow\"')"
    )

# Ensure core module is importable
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.state.combat import (
    CombatState,
    EntityState,
    EnemyCombatState,
    PlayCard,
    EndTurn,
    create_player,
    create_enemy,
    create_combat,
)
from core.simulation import (
    SimulationConfig,
    BatchConfig,
    StateSerializer,
)


# =============================================================================
# Combat State Fixtures
# =============================================================================


@pytest.fixture
def basic_player() -> EntityState:
    """Create a basic player entity for testing."""
    return EntityState(
        hp=70,
        max_hp=80,
        block=0,
        statuses={},
    )


@pytest.fixture
def player_with_statuses() -> EntityState:
    """Create a player with common statuses applied."""
    return EntityState(
        hp=60,
        max_hp=80,
        block=10,
        statuses={
            "Strength": 2,
            "Dexterity": 1,
            "Mantra": 5,
        },
    )


@pytest.fixture
def basic_enemy() -> EnemyCombatState:
    """Create a basic enemy (Jaw Worm) for testing."""
    return EnemyCombatState(
        hp=40,
        max_hp=42,
        block=0,
        id="JawWorm",
        move_id=1,
        move_damage=11,
        move_hits=1,
        statuses={},
    )


@pytest.fixture
def enemy_with_statuses() -> EnemyCombatState:
    """Create an enemy with statuses applied."""
    return EnemyCombatState(
        hp=35,
        max_hp=42,
        block=5,
        id="JawWorm",
        move_id=2,
        move_damage=7,
        move_hits=1,
        statuses={
            "Vulnerable": 2,
            "Strength": 3,
        },
    )


@pytest.fixture
def multiple_enemies() -> List[EnemyCombatState]:
    """Create multiple enemies for multi-target testing."""
    return [
        EnemyCombatState(
            hp=20, max_hp=20, block=0, id="AcidSlime_S",
            move_id=1, move_damage=3, move_hits=1, statuses={},
        ),
        EnemyCombatState(
            hp=25, max_hp=28, block=0, id="SpikeSlime_M",
            move_id=1, move_damage=8, move_hits=1, statuses={},
        ),
    ]


@pytest.fixture
def starter_deck() -> List[str]:
    """Create a Watcher starter deck."""
    return [
        "Strike_P", "Strike_P", "Strike_P", "Strike_P",
        "Defend_P", "Defend_P", "Defend_P", "Defend_P",
        "Eruption", "Vigilance", "AscendersBane",
    ]


@pytest.fixture
def basic_combat_state(basic_player, basic_enemy, starter_deck) -> CombatState:
    """Create a basic combat state for testing."""
    return CombatState(
        player=basic_player,
        energy=3,
        max_energy=3,
        stance="Neutral",
        hand=["Strike_P", "Defend_P", "Eruption", "Vigilance", "AscendersBane"],
        draw_pile=["Strike_P", "Strike_P", "Strike_P", "Defend_P", "Defend_P", "Defend_P"],
        discard_pile=[],
        exhaust_pile=[],
        enemies=[basic_enemy],
        potions=["", "", ""],
    )


@pytest.fixture
def wrath_combat_state(player_with_statuses, basic_enemy) -> CombatState:
    """Create a combat state with player in Wrath stance."""
    return CombatState(
        player=player_with_statuses,
        energy=2,
        max_energy=3,
        stance="Wrath",
        hand=["Strike_P", "Tantrum", "Vigilance", "InnerPeace"],
        draw_pile=["Strike_P"] * 5 + ["Defend_P"] * 5,
        discard_pile=["Eruption"],
        exhaust_pile=[],
        enemies=[basic_enemy],
        potions=["Block Potion", "", ""],
    )


@pytest.fixture
def multi_enemy_combat_state(basic_player, multiple_enemies) -> CombatState:
    """Create a combat state with multiple enemies."""
    return CombatState(
        player=basic_player,
        energy=3,
        max_energy=3,
        stance="Neutral",
        hand=["Strike_P", "Defend_P", "Conclude", "WheelKick", "CutThroughFate"],
        draw_pile=["Strike_P"] * 5,
        discard_pile=[],
        exhaust_pile=[],
        enemies=multiple_enemies,
        potions=["Fire Potion", "", ""],
    )


# =============================================================================
# Simulation Configuration Fixtures
# =============================================================================


@pytest.fixture
def default_sim_config() -> SimulationConfig:
    """Create default simulation configuration."""
    return SimulationConfig()


@pytest.fixture
def fast_sim_config() -> SimulationConfig:
    """Create simulation configuration optimized for fast testing."""
    return SimulationConfig(
        n_workers=2,
        batch_size=10,
        max_turns_per_combat=50,
        max_floors_per_run=20,
        default_search_budget=100,
        prefork_workers=False,
    )


@pytest.fixture
def single_worker_config() -> SimulationConfig:
    """Create simulation configuration with single worker for deterministic testing."""
    return SimulationConfig(
        n_workers=1,
        batch_size=5,
        max_turns_per_combat=50,
        prefork_workers=False,
    )


@pytest.fixture
def default_batch_config() -> BatchConfig:
    """Create default batch configuration."""
    return BatchConfig()


@pytest.fixture
def fast_batch_config() -> BatchConfig:
    """Create batch configuration for fast testing."""
    return BatchConfig(
        chunk_size=100,
        result_buffer_size=1000,
        report_interval=50,
        enable_gc=False,
        max_errors=10,
    )


# =============================================================================
# Serializer Fixtures
# =============================================================================


@pytest.fixture
def serializer() -> StateSerializer:
    """Create a default state serializer."""
    return StateSerializer()


@pytest.fixture
def fast_serializer() -> StateSerializer:
    """Create a serializer with fast batch config."""
    return StateSerializer(BatchConfig(compress=False))


# =============================================================================
# Seed Fixtures
# =============================================================================


@pytest.fixture
def test_seeds() -> List[str]:
    """Generate a list of test seeds."""
    return [f"TEST{i:04d}" for i in range(100)]


@pytest.fixture
def small_seed_batch() -> List[str]:
    """Generate a small batch of seeds for quick tests."""
    return [f"QUICK{i:02d}" for i in range(10)]


@pytest.fixture
def benchmark_seeds() -> List[str]:
    """Generate seeds for benchmark testing."""
    return [f"BENCH{i:06d}" for i in range(1000)]


# =============================================================================
# Helper Functions
# =============================================================================


@pytest.fixture
def create_random_combat_state():
    """Factory fixture to create randomized combat states."""
    import random

    def _create(
        player_hp: int = None,
        num_enemies: int = 1,
        hand_size: int = 5,
        seed: int = None,
    ) -> CombatState:
        if seed is not None:
            random.seed(seed)

        player = EntityState(
            hp=player_hp or random.randint(30, 80),
            max_hp=80,
            block=random.randint(0, 15),
            statuses={},
        )

        enemies = []
        enemy_types = ["JawWorm", "Cultist", "AcidSlime_M", "SpikeSlime_S"]
        for i in range(num_enemies):
            enemy_type = random.choice(enemy_types)
            enemies.append(EnemyCombatState(
                hp=random.randint(20, 50),
                max_hp=random.randint(40, 60),
                block=random.randint(0, 10),
                id=enemy_type,
                move_id=random.randint(0, 3),
                move_damage=random.randint(5, 15),
                move_hits=random.randint(1, 3),
                statuses={},
            ))

        cards = ["Strike_P", "Defend_P", "Eruption", "Vigilance", "Tantrum", "InnerPeace"]
        hand = random.choices(cards, k=hand_size)
        draw_pile = random.choices(cards, k=random.randint(10, 20))

        return CombatState(
            player=player,
            energy=random.randint(1, 4),
            max_energy=3,
            stance=random.choice(["Neutral", "Wrath", "Calm"]),
            hand=hand,
            draw_pile=draw_pile,
            discard_pile=[],
            exhaust_pile=[],
            enemies=enemies,
            potions=["", "", ""],
        )

    return _create
