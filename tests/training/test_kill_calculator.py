"""
Unit tests for Kill Calculator.

These tests run without needing the game - they use simulated combat states.
"""

import pytest
from packages.training.kill_calculator import KillCalculator, can_kill_this_turn, get_kill_line
from packages.training.line_evaluator import SimulatedPlayer, SimulatedEnemy


class TestBasicKills:
    """Test basic kill detection."""

    def setup_method(self):
        self.calc = KillCalculator()

    def test_simple_kill_with_strike(self):
        """Low HP enemy, simple strike kills."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=5, max_hp=20, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [{"id": "Strike_P"}]  # 6 damage

        assert self.calc.can_kill_all(hand, enemies, player)

    def test_cannot_kill_high_hp(self):
        """High HP enemy, not enough damage."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=100, max_hp=100, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [{"id": "Strike_P"}]

        assert not self.calc.can_kill_all(hand, enemies, player)

    def test_kill_through_block(self):
        """Enemy has block, need enough damage to punch through."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=10, max_hp=20, block=10,  # 20 total to kill
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        # Two strikes = 12 damage, not enough
        hand = [{"id": "Strike_P"}, {"id": "Strike_P"}]

        assert not self.calc.can_kill_all(hand, enemies, player)

    def test_no_enemies_is_lethal(self):
        """No enemies = already won."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = []
        hand = [{"id": "Strike_P"}]

        assert self.calc.can_kill_all(hand, enemies, player)


class TestWrathMechanics:
    """Test Watcher stance mechanics."""

    def setup_method(self):
        self.calc = KillCalculator()

    def test_wrath_doubles_damage(self):
        """Wrath should double damage output."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Calm")
        enemies = [SimulatedEnemy(
            id=0, hp=30, max_hp=30, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        # Eruption+ (9 damage) + Tantrum (3*3=9 damage)
        # In wrath: 9*2 + 9*2 = 36 damage
        hand = [{"id": "Eruption+"}, {"id": "Tantrum"}]

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert stats.can_kill_all
        assert stats.requires_wrath

    def test_calm_gives_energy(self):
        """Exiting Calm gives +2 energy."""
        player = SimulatedPlayer(hp=50, block=0, energy=2, stance="Calm")
        enemies = [SimulatedEnemy(
            id=0, hp=15, max_hp=20, block=0,
            intent_damage=5, intent_hits=1, is_attacking=True
        )]
        # Eruption+ costs 1, we have 2, but get +2 from Calm = 4 total
        # This should be enough to play Eruption+ (18 damage in wrath)
        hand = [{"id": "Eruption+"}]

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert stats.can_kill_all

    def test_safe_wrath_detection(self):
        """Detect when we can enter AND exit wrath same turn."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=20, max_hp=20, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [
            {"id": "Eruption+"},   # Enter wrath
            {"id": "EmptyFist+"},  # Exit wrath
        ]

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert stats.safe_wrath_available

    def test_wrath_not_safe_no_exit(self):
        """No stance exit card = unsafe wrath."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=20, max_hp=20, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [{"id": "Eruption+"}]  # Enter only, no exit

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert not stats.safe_wrath_available


class TestMultiEnemy:
    """Test multi-enemy scenarios."""

    def setup_method(self):
        self.calc = KillCalculator()

    def test_aoe_kills_multiple(self):
        """AOE card kills multiple enemies."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [
            SimulatedEnemy(id=0, hp=20, max_hp=20, block=0,
                           intent_damage=5, intent_hits=1, is_attacking=True),
            SimulatedEnemy(id=1, hp=15, max_hp=15, block=0,
                           intent_damage=5, intent_hits=1, is_attacking=True),
        ]
        # Ragnarok: 5*5 = 25 damage to EACH enemy
        hand = [{"id": "Ragnarok"}]

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert stats.can_kill_all
        assert stats.enemies_killed == 2

    def test_priority_targeting(self):
        """Correctly identifies most dangerous enemy."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [
            SimulatedEnemy(id=0, hp=50, max_hp=50, block=0,
                           intent_damage=5, intent_hits=1, is_attacking=True),  # Low threat
            SimulatedEnemy(id=1, hp=30, max_hp=30, block=0,
                           intent_damage=20, intent_hits=1, is_attacking=True),  # High threat
        ]
        hand = [{"id": "Strike_P"}]

        stats = self.calc.get_kill_stats(hand, enemies, player)
        # Enemy 1 should be priority (higher damage)
        assert stats.priority_target_id == 1

    def test_kill_priority_only(self):
        """Can kill priority target but not all enemies."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Calm")
        enemies = [
            SimulatedEnemy(id=0, hp=100, max_hp=100, block=0,
                           intent_damage=5, intent_hits=1, is_attacking=True),
            SimulatedEnemy(id=1, hp=25, max_hp=25, block=0,
                           intent_damage=25, intent_hits=1, is_attacking=True),  # Priority
        ]
        # Eruption+ (18 wrath) + Tantrum (18 wrath) = 36 damage
        hand = [{"id": "Eruption+"}, {"id": "Tantrum"}]

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert not stats.can_kill_all
        assert stats.can_kill_priority


class TestSequenceOptimization:
    """Test optimal sequence finding."""

    def setup_method(self):
        self.calc = KillCalculator()

    def test_minimum_cards_to_kill(self):
        """Finds shortest sequence."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=10, max_hp=20, block=0,
            intent_damage=5, intent_hits=1, is_attacking=True
        )]
        # Could use 2 Strikes (12 dmg) or 1 BowlingBash+ (10 dmg)
        hand = [
            {"id": "Strike_P"},
            {"id": "Strike_P"},
            {"id": "BowlingBash+"},  # 10 damage
        ]

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert stats.can_kill_all
        # Should prefer single card kill
        assert len(stats.fastest_kill_sequence) == 1

    def test_tracks_hp_cost(self):
        """Tracks damage we'll take after playing."""
        player = SimulatedPlayer(hp=50, block=0, energy=1, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=5, max_hp=20, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [{"id": "Strike_P"}]

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert stats.can_kill_all
        # No block, enemy intends 10, but they're dead so 0 damage
        assert stats.hp_cost == 0

    def test_non_lethal_takes_damage(self):
        """Non-lethal line results in taking damage."""
        player = SimulatedPlayer(hp=50, block=0, energy=1, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=50, max_hp=50, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [{"id": "Strike_P"}]  # Not enough to kill

        stats = self.calc.get_kill_stats(hand, enemies, player)
        assert not stats.can_kill_all
        # Can't kill, would need to check line simulator directly for damage


class TestConvenienceFunctions:
    """Test convenience functions."""

    def test_can_kill_this_turn(self):
        """Quick kill check function."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=5, max_hp=20, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [{"id": "Strike_P"}]

        assert can_kill_this_turn(hand, enemies, player)

    def test_get_kill_line(self):
        """Get kill sequence or None."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=5, max_hp=20, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [{"id": "Strike_P"}]

        line = get_kill_line(hand, enemies, player)
        assert line is not None
        assert len(line) == 1
        assert line[0].card_id == "Strike_P"

    def test_get_kill_line_returns_none(self):
        """Returns None when kill impossible."""
        player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Neutral")
        enemies = [SimulatedEnemy(
            id=0, hp=100, max_hp=100, block=0,
            intent_damage=10, intent_hits=1, is_attacking=True
        )]
        hand = [{"id": "Strike_P"}]

        line = get_kill_line(hand, enemies, player)
        assert line is None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
