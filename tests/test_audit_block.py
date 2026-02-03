"""
Block pipeline audit tests.

Verifies Python engine block calculation matches decompiled Java behavior.
See docs/audit/block-pipeline.md for full audit report.
"""
import pytest

from packages.engine.calc.damage import calculate_block


# =============================================================================
# Dexterity Tests
# =============================================================================

class TestDexterity:
    """Dexterity adds flat block before other modifiers."""

    def test_positive_dexterity(self):
        # 5 base + 3 dex = 8
        assert calculate_block(5, dexterity=3) == 8

    def test_negative_dexterity(self):
        # 5 base - 2 dex = 3
        assert calculate_block(5, dexterity=-2) == 3

    def test_negative_dexterity_floor_zero(self):
        # 3 base - 5 dex = -2 -> clamped to 0
        assert calculate_block(3, dexterity=-5) == 0

    def test_zero_dexterity(self):
        assert calculate_block(5, dexterity=0) == 5

    def test_large_dexterity(self):
        assert calculate_block(5, dexterity=10) == 15


# =============================================================================
# Frail Tests
# =============================================================================

class TestFrail:
    """Frail reduces block by 25% (multiply by 0.75), applied after Dexterity."""

    def test_frail_basic(self):
        # 8 * 0.75 = 6
        assert calculate_block(8, frail=True) == 6

    def test_frail_rounds_down(self):
        # 5 * 0.75 = 3.75 -> 3 (floor via int())
        assert calculate_block(5, frail=True) == 3

    def test_frail_with_dexterity(self):
        # (5 + 2) * 0.75 = 5.25 -> 5
        assert calculate_block(5, dexterity=2, frail=True) == 5

    def test_frail_with_negative_dex(self):
        # (8 + (-2)) * 0.75 = 4.5 -> 4
        assert calculate_block(8, dexterity=-2, frail=True) == 4

    def test_frail_dex_makes_zero(self):
        # (2 + (-3)) * 0.75 = -0.75 -> max(0, int(-0.75)) = 0
        assert calculate_block(2, dexterity=-3, frail=True) == 0

    def test_frail_10_block(self):
        # 10 * 0.75 = 7.5 -> 7
        assert calculate_block(10, frail=True) == 7

    def test_frail_not_applied(self):
        assert calculate_block(5, frail=False) == 5

    def test_frail_1_block(self):
        # 1 * 0.75 = 0.75 -> 0
        assert calculate_block(1, frail=True) == 0


# =============================================================================
# No Block Tests
# =============================================================================

class TestNoBlock:
    """No Block power sets block to 0 (applied last via modifyBlockLast)."""

    def test_no_block(self):
        assert calculate_block(10, no_block=True) == 0

    def test_no_block_with_dex(self):
        assert calculate_block(10, dexterity=5, no_block=True) == 0

    def test_no_block_with_frail(self):
        assert calculate_block(10, frail=True, no_block=True) == 0


# =============================================================================
# Edge Cases
# =============================================================================

class TestEdgeCases:
    """Edge cases for block calculation."""

    def test_zero_base_block(self):
        assert calculate_block(0) == 0

    def test_zero_base_with_dexterity(self):
        # 0 + 3 = 3
        assert calculate_block(0, dexterity=3) == 3

    def test_zero_base_with_frail(self):
        # 0 * 0.75 = 0
        assert calculate_block(0, frail=True) == 0

    def test_large_block(self):
        assert calculate_block(100, dexterity=10) == 110

    def test_large_block_frail(self):
        # 100 * 0.75 = 75
        assert calculate_block(100, frail=True) == 75

    def test_defend_base_5_no_modifiers(self):
        """Defend base case: 5 block, no modifiers."""
        assert calculate_block(5) == 5

    def test_defend_upgraded_8_no_modifiers(self):
        """Defend+ base case: 8 block, no modifiers."""
        assert calculate_block(8) == 8

    def test_defend_upgraded_frail(self):
        """Defend+ with Frail: 8 * 0.75 = 6."""
        assert calculate_block(8, frail=True) == 6

    def test_frail_rounding_matches_java(self):
        """
        Java: MathUtils.floor(float) for positive values.
        Python: int(float) for positive values.
        Both floor positive values the same way.

        Key test: 7 * 0.75 = 5.25 -> 5 in both Java and Python.
        """
        assert calculate_block(7, frail=True) == 5  # 7 * 0.75 = 5.25 -> 5

    def test_dex_order_before_frail(self):
        """
        Java: DexterityPower.modifyBlock runs in first loop,
              FrailPower.modifyBlock runs in same loop but later.
        Order: base -> +dex -> *0.75

        (8 + 2) * 0.75 = 7.5 -> 7
        NOT 8 * 0.75 + 2 = 8
        """
        assert calculate_block(8, dexterity=2, frail=True) == 7
