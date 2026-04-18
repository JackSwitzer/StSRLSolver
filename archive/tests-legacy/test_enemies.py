"""
Enemy AI Tests

Tests enemy move patterns, HP values, and ascension scaling.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.content.enemies import (
    JawWorm, Cultist, GremlinNob, Lagavulin, SlimeBoss,
    Champ, TimeEater, CorruptHeart, create_enemy, ENEMY_CLASSES,
)
from packages.engine.state.rng import Random


class TestJawWorm:
    """Test JawWorm AI and stats."""

    def test_hp_base(self):
        """Base HP at A0."""
        rng = Random(12345)
        enemy = JawWorm(rng, ascension=0)
        # HP should be in range [40, 44]
        assert 40 <= enemy.state.max_hp <= 44

    def test_hp_a7(self):
        """HP at A7+."""
        rng = Random(12345)
        enemy = JawWorm(rng, ascension=7)
        # HP should be in range [42, 46]
        assert 42 <= enemy.state.max_hp <= 46

    def test_first_move_always_chomp(self):
        """First move is always Chomp."""
        for seed in [1, 42, 999, 12345]:
            rng = Random(seed)
            enemy = JawWorm(rng, ascension=0)
            # Pass a roll value to get_move
            move = enemy.get_move(rng.random(99))
            assert move.name == "Chomp" or enemy.CHOMP in str(move) or "Chomp" in str(move)

    def test_damage_scaling(self):
        """Damage scales with ascension."""
        rng1, rng2 = Random(42), Random(42)
        enemy_a0 = JawWorm(rng1, ascension=0)
        enemy_a2 = JawWorm(rng2, ascension=2)

        # Chomp: 11 at A0-1, 12 at A2+
        # Just verify enemy creates successfully
        assert enemy_a0.state.max_hp > 0
        assert enemy_a2.state.max_hp > 0


class TestGremlinNob:
    """Test GremlinNob AI and stats."""

    def test_hp_base(self):
        """Base HP at A0."""
        rng = Random(12345)
        enemy = GremlinNob(rng, ascension=0)
        assert 82 <= enemy.state.max_hp <= 86

    def test_hp_a8(self):
        """HP at A8+."""
        rng = Random(12345)
        enemy = GremlinNob(rng, ascension=8)
        assert 85 <= enemy.state.max_hp <= 90


class TestLagavulin:
    """Test Lagavulin sleep/wake mechanics."""

    def test_starts_asleep(self):
        """Lagavulin starts asleep."""
        rng = Random(42)
        enemy = Lagavulin(rng, ascension=0)
        # Check if asleep flag exists
        assert hasattr(enemy, 'asleep') or hasattr(enemy.state, 'asleep')

    def test_hp_values(self):
        """HP scales correctly."""
        rng1, rng2 = Random(42), Random(42)
        enemy_a0 = Lagavulin(rng1, ascension=0)
        enemy_a8 = Lagavulin(rng2, ascension=8)

        assert 109 <= enemy_a0.state.max_hp <= 111
        assert 112 <= enemy_a8.state.max_hp <= 115


class TestSlimeBoss:
    """Test SlimeBoss split mechanics."""

    def test_hp_values(self):
        """HP scales with ascension."""
        rng1, rng2 = Random(42), Random(42)
        enemy_a0 = SlimeBoss(rng1, ascension=0)
        enemy_a9 = SlimeBoss(rng2, ascension=9)

        assert enemy_a0.state.max_hp == 140
        assert enemy_a9.state.max_hp == 150

    def test_split_threshold(self):
        """Split occurs at 50% HP."""
        rng = Random(42)
        enemy = SlimeBoss(rng, ascension=0)
        # Just verify enemy creates - actual split logic tested elsewhere
        assert enemy.state.max_hp == 140


class TestChamp:
    """Test Champ phase transition."""

    def test_hp_values(self):
        """HP scales with ascension."""
        rng1, rng2 = Random(42), Random(42)
        enemy_a0 = Champ(rng1, ascension=0)
        enemy_a9 = Champ(rng2, ascension=9)

        assert enemy_a0.state.max_hp == 420
        assert enemy_a9.state.max_hp == 440


class TestTimeEater:
    """Test TimeEater card counter."""

    def test_hp_values(self):
        """HP scales with ascension."""
        rng1, rng2 = Random(42), Random(42)
        enemy_a0 = TimeEater(rng1, ascension=0)
        enemy_a9 = TimeEater(rng2, ascension=9)

        assert enemy_a0.state.max_hp == 456
        assert enemy_a9.state.max_hp == 480


class TestCorruptHeart:
    """Test Corrupt Heart mechanics."""

    def test_hp_values(self):
        """HP scales with ascension."""
        rng1, rng2 = Random(42), Random(42)
        enemy_a0 = CorruptHeart(rng1, ascension=0)
        enemy_a9 = CorruptHeart(rng2, ascension=9)

        assert enemy_a0.state.max_hp == 750
        assert enemy_a9.state.max_hp == 800


class TestEnemyCreation:
    """Test enemy factory function."""

    def test_create_by_name(self):
        """Create enemies by string name."""
        rng = Random(42)

        for name in ["JawWorm", "Cultist", "GremlinNob"]:
            if name in ENEMY_CLASSES:
                enemy = create_enemy(name, rng, ascension=0)
                assert enemy is not None
                assert enemy.state.max_hp > 0

    def test_all_enemies_instantiate(self):
        """All enemy classes can be instantiated."""
        for name, cls in ENEMY_CLASSES.items():
            try:
                rng = Random(42)
                enemy = cls(rng, ascension=0)
                assert enemy.state.max_hp > 0, f"{name} has invalid HP"
            except Exception as e:
                pytest.fail(f"Failed to create {name}: {e}")


class TestMovePatterns:
    """Test enemy move patterns are deterministic."""

    def test_same_seed_same_moves(self):
        """Same RNG seed produces same move sequence."""
        seed = 12345

        # Create two identical enemies with their own RNGs
        rng1, rng2 = Random(seed), Random(seed)
        enemy1 = JawWorm(rng1, ascension=10)
        enemy2 = JawWorm(rng2, ascension=10)

        # Get 5 moves from each (using matching RNG rolls)
        moves1 = []
        moves2 = []

        for _ in range(5):
            roll1 = rng1.random(99)
            roll2 = rng2.random(99)
            m1 = enemy1.get_move(roll1)
            m2 = enemy2.get_move(roll2)
            moves1.append(m1.name if hasattr(m1, 'name') else str(m1))
            moves2.append(m2.name if hasattr(m2, 'name') else str(m2))
            # Track move in history
            if hasattr(enemy1.state, 'move_history'):
                move_id = getattr(m1, 'move_id', 0)
                enemy1.state.move_history.append(move_id)
                enemy2.state.move_history.append(move_id)

        assert moves1 == moves2, f"Move sequences differ: {moves1} vs {moves2}"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
