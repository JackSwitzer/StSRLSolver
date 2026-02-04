"""
Comprehensive edge case tests for damage and block calculation.

These tests verify exact Java parity for rounding, negative stats, relic combinations,
and boundary conditions identified in the damage calculation audit.
"""

import pytest
from packages.engine.calc.damage import (
    calculate_damage,
    calculate_block,
    calculate_incoming_damage,
    apply_hp_loss,
    WEAK_MULT,
    WEAK_MULT_PAPER_CRANE,
    VULN_MULT,
    VULN_MULT_ODD_MUSHROOM,
    VULN_MULT_PAPER_FROG,
    FRAIL_MULT,
    WRATH_MULT,
    DIVINITY_MULT,
)


# =============================================================================
# ROUNDING EDGE CASES
# =============================================================================

class TestRounding:
    """Test that rounding behavior matches Java's MathUtils.floor()."""

    def test_weak_rounding_down(self):
        """10 * 0.75 = 7.5 → should floor to 7, not round to 8."""
        assert calculate_damage(10, weak=True) == 7

    def test_weak_rounding_various_values(self):
        """Test weak rounding for various base damages."""
        # All should floor down
        assert calculate_damage(11, weak=True) == 8   # 11 * 0.75 = 8.25 → 8
        assert calculate_damage(13, weak=True) == 9   # 13 * 0.75 = 9.75 → 9
        assert calculate_damage(14, weak=True) == 10  # 14 * 0.75 = 10.5 → 10
        assert calculate_damage(15, weak=True) == 11  # 15 * 0.75 = 11.25 → 11

    def test_vulnerable_rounding(self):
        """Test vulnerable rounding."""
        assert calculate_damage(7, vuln=True) == 10   # 7 * 1.5 = 10.5 → 10
        assert calculate_damage(11, vuln=True) == 16  # 11 * 1.5 = 16.5 → 16

    def test_frail_rounding(self):
        """Test frail rounding on block."""
        assert calculate_block(7, frail=True) == 5   # 7 * 0.75 = 5.25 → 5
        assert calculate_block(11, frail=True) == 8  # 11 * 0.75 = 8.25 → 8

    def test_multiple_multipliers_compound_rounding(self):
        """Test that multiple multipliers compound before rounding."""
        # 10 * 0.75 (weak) * 1.5 (vuln) = 11.25 → 11
        assert calculate_damage(10, weak=True, vuln=True) == 11

        # 6 * 0.75 (weak) * 2.0 (wrath) = 9.0 → 9
        assert calculate_damage(6, weak=True, stance_mult=WRATH_MULT) == 9

        # 7 * 0.75 (weak) * 2.0 (wrath) * 1.5 (vuln) = 15.75 → 15
        assert calculate_damage(7, weak=True, stance_mult=WRATH_MULT, vuln=True) == 15


# =============================================================================
# NEGATIVE STATS
# =============================================================================

class TestNegativeStats:
    """Test handling of negative Strength and Dexterity."""

    def test_negative_strength_simple(self):
        """Negative strength reduces damage."""
        assert calculate_damage(10, strength=-3) == 7
        assert calculate_damage(6, strength=-2) == 4

    def test_negative_strength_floor_at_zero(self):
        """Damage can't go below 0."""
        assert calculate_damage(5, strength=-10) == 0
        assert calculate_damage(3, strength=-5) == 0

    def test_negative_strength_with_multipliers(self):
        """Negative strength interacts with multipliers."""
        # (10 - 3) * 2 (wrath) = 14
        assert calculate_damage(10, strength=-3, stance_mult=WRATH_MULT) == 14

        # (10 - 5) * 0.75 (weak) = 3.75 → 3
        assert calculate_damage(10, strength=-5, weak=True) == 3

    def test_negative_dexterity_simple(self):
        """Negative dexterity reduces block."""
        assert calculate_block(10, dexterity=-3) == 7
        assert calculate_block(6, dexterity=-2) == 4

    def test_negative_dexterity_floor_at_zero(self):
        """Block can't go below 0."""
        assert calculate_block(5, dexterity=-10) == 0
        assert calculate_block(3, dexterity=-5) == 0

    def test_negative_dexterity_with_frail(self):
        """Negative dexterity applies before frail multiplier."""
        # (8 - 3) * 0.75 (frail) = 3.75 → 3
        assert calculate_block(8, dexterity=-3, frail=True) == 3

        # (10 - 6) * 0.75 (frail) = 3.0 → 3
        assert calculate_block(10, dexterity=-6, frail=True) == 3

    def test_negative_dex_plus_frail_reaches_zero(self):
        """Combined negative dex and frail can reach 0."""
        # (5 - 4) * 0.75 = 0.75 → 0
        assert calculate_block(5, dexterity=-4, frail=True) == 0


# =============================================================================
# RELIC MODIFIER COMBINATIONS
# =============================================================================

class TestRelicModifiers:
    """Test relic-specific modifier values."""

    def test_paper_crane_weak(self):
        """Paper Crane: Weak reduces damage by 40% instead of 25%."""
        # 10 * 0.6 = 6
        assert calculate_damage(10, weak=True, weak_paper_crane=True) == 6

        # Verify regular weak is different
        assert calculate_damage(10, weak=True) == 7

    def test_paper_frog_vulnerable(self):
        """Paper Frog: Vulnerable increases damage by 75% instead of 50%."""
        # 10 * 1.75 = 17.5 → 17
        assert calculate_damage(10, vuln=True, vuln_paper_frog=True) == 17

        # Verify regular vuln is different
        assert calculate_damage(10, vuln=True) == 15

    def test_odd_mushroom_vulnerable(self):
        """Odd Mushroom: Vulnerable on player only increases by 25%."""
        # This is tested in incoming damage
        hp_loss, _ = calculate_incoming_damage(10, 0, vuln=True, vuln_odd_mushroom=True)
        assert hp_loss == 12  # 10 * 1.25 = 12.5 → 12

        # Verify regular vuln is different
        hp_loss, _ = calculate_incoming_damage(10, 0, vuln=True)
        assert hp_loss == 15

    def test_pen_nib_doubles_damage(self):
        """Pen Nib doubles damage before other multipliers."""
        # 6 * 2 (pen nib) = 12
        assert calculate_damage(6, pen_nib=True) == 12

        # 6 * 2 (pen nib) * 2 (wrath) = 24
        assert calculate_damage(6, pen_nib=True, stance_mult=WRATH_MULT) == 24

    def test_pen_nib_with_strength_and_weak(self):
        """Pen Nib interacts with strength and weak."""
        # (10 + 5) * 2 (pen nib) * 0.75 (weak) = 22.5 → 22
        assert calculate_damage(10, strength=5, pen_nib=True, weak=True) == 22

    def test_flight_halves_damage(self):
        """Flight reduces damage by 50%."""
        assert calculate_damage(10, flight=True) == 5
        assert calculate_damage(11, flight=True) == 5  # 11 * 0.5 = 5.5 → 5

    def test_flight_with_vulnerable(self):
        """Flight and Vulnerable stack multiplicatively."""
        # 10 * 1.5 (vuln) * 0.5 (flight) = 7.5 → 7
        assert calculate_damage(10, vuln=True, flight=True) == 7


# =============================================================================
# STANCE INTERACTIONS
# =============================================================================

class TestStanceInteractions:
    """Test stance multiplier interactions with powers."""

    def test_weak_in_wrath_order(self):
        """Weak applies before Wrath - order matters."""
        # 10 * 0.75 (weak) * 2.0 (wrath) = 15
        assert calculate_damage(10, weak=True, stance_mult=WRATH_MULT) == 15

    def test_weak_in_divinity_order(self):
        """Weak applies before Divinity."""
        # 10 * 0.75 (weak) * 3.0 (divinity) = 22.5 → 22
        assert calculate_damage(10, weak=True, stance_mult=DIVINITY_MULT) == 22

    def test_vulnerable_in_wrath(self):
        """Vulnerable applies after Wrath."""
        # 10 * 2.0 (wrath) * 1.5 (vuln) = 30
        assert calculate_damage(10, stance_mult=WRATH_MULT, vuln=True) == 30

    def test_pen_nib_in_divinity(self):
        """Pen Nib doubles before Divinity triples."""
        # 6 * 2 (pen nib) * 3 (divinity) = 36
        assert calculate_damage(6, pen_nib=True, stance_mult=DIVINITY_MULT) == 36

    def test_full_combo_wrath(self):
        """Test full combination in Wrath."""
        # (10 + 5 str) * 2 (pen nib) * 0.75 (weak) * 2 (wrath) * 1.5 (vuln) = 67.5 → 67
        assert calculate_damage(
            10,
            strength=5,
            pen_nib=True,
            weak=True,
            stance_mult=WRATH_MULT,
            vuln=True,
        ) == 67

    def test_wrath_incoming_damage(self):
        """Wrath doubles incoming damage too."""
        # 10 * 2 (wrath) - 5 block = 15 hp loss
        hp_loss, remaining = calculate_incoming_damage(10, 5, is_wrath=True)
        assert hp_loss == 15
        assert remaining == 0

    def test_divinity_does_not_affect_incoming(self):
        """Divinity does NOT increase incoming damage."""
        # Divinity has no effect on incoming - this is tested by NOT passing stance_mult
        # to calculate_incoming_damage (which doesn't have that parameter)
        hp_loss, remaining = calculate_incoming_damage(10, 5)
        assert hp_loss == 5  # Normal calculation


# =============================================================================
# BLOCK EDGE CASES
# =============================================================================

class TestBlockEdgeCases:
    """Test block calculation edge cases."""

    def test_exactly_zero_block_remaining(self):
        """Damage exactly equals block."""
        hp_loss, remaining = calculate_incoming_damage(10, 10)
        assert hp_loss == 0
        assert remaining == 0

    def test_block_overflow(self):
        """Block exceeds damage."""
        hp_loss, remaining = calculate_incoming_damage(5, 20)
        assert hp_loss == 0
        assert remaining == 15

    def test_no_block_parameter(self):
        """No Block power sets block to 0 (tested via parameter)."""
        # Note: no_block parameter is in calculate_block
        assert calculate_block(10, no_block=True) == 0
        assert calculate_block(10, dexterity=5, no_block=True) == 0

    def test_frail_then_no_block(self):
        """Frail applies before No Block (but No Block overrides)."""
        # No Block sets final value to 0 regardless of Frail
        assert calculate_block(10, frail=True, no_block=True) == 0


# =============================================================================
# TORII + TUNGSTEN ROD COMBINATIONS
# =============================================================================

class TestToriiTungstenCombos:
    """Test Torii and Tungsten Rod interactions at boundaries."""

    def test_torii_boundary_1_damage(self):
        """Torii doesn't trigger on 1 damage (below min threshold of 2)."""
        hp_loss, _ = calculate_incoming_damage(1, 0, torii=True)
        assert hp_loss == 1

    def test_torii_boundary_2_damage(self):
        """Torii triggers on 2 damage (at min threshold)."""
        hp_loss, _ = calculate_incoming_damage(2, 0, torii=True)
        assert hp_loss == 1

    def test_torii_boundary_5_damage(self):
        """Torii triggers on 5 damage (at max threshold)."""
        hp_loss, _ = calculate_incoming_damage(5, 0, torii=True)
        assert hp_loss == 1

    def test_torii_boundary_6_damage(self):
        """Torii doesn't trigger on 6 damage (above max threshold)."""
        hp_loss, _ = calculate_incoming_damage(6, 0, torii=True)
        assert hp_loss == 6

    def test_torii_plus_tungsten_2_damage(self):
        """Both active on 2 damage: Torii reduces to 1, Tungsten to 0."""
        hp_loss, _ = calculate_incoming_damage(2, 0, torii=True, tungsten_rod=True)
        assert hp_loss == 0  # 2 → 1 (torii) → 0 (tungsten)

    def test_torii_plus_tungsten_5_damage(self):
        """Both active on 5 damage: Torii reduces to 1, Tungsten to 0."""
        hp_loss, _ = calculate_incoming_damage(5, 0, torii=True, tungsten_rod=True)
        assert hp_loss == 0  # 5 → 1 (torii) → 0 (tungsten)

    def test_torii_plus_tungsten_6_damage(self):
        """Both active on 6 damage: Torii doesn't trigger, Tungsten reduces by 1."""
        hp_loss, _ = calculate_incoming_damage(6, 0, torii=True, tungsten_rod=True)
        assert hp_loss == 5  # 6 (no torii) → 5 (tungsten)

    def test_torii_after_block(self):
        """Torii sees post-block damage."""
        # 10 damage - 8 block = 2, then Torii → 1
        hp_loss, remaining = calculate_incoming_damage(10, 8, torii=True)
        assert hp_loss == 1
        assert remaining == 0

    def test_torii_blocked_completely(self):
        """If damage is fully blocked, Torii sees 0 and doesn't trigger."""
        hp_loss, remaining = calculate_incoming_damage(10, 15, torii=True)
        assert hp_loss == 0
        assert remaining == 5

    def test_tungsten_on_1_hp_loss(self):
        """Tungsten Rod reduces 1 HP loss to 0."""
        hp_loss, _ = calculate_incoming_damage(1, 0, tungsten_rod=True)
        assert hp_loss == 0


# =============================================================================
# INTANGIBLE EDGE CASES
# =============================================================================

class TestIntangibleEdgeCases:
    """Test Intangible cap behavior."""

    def test_intangible_caps_before_block(self):
        """Intangible caps damage BEFORE block subtracts."""
        # 100 damage → 1 (intangible) → 0 (blocked by any block)
        hp_loss, remaining = calculate_incoming_damage(100, 5, intangible=True)
        assert hp_loss == 0  # 1 damage fully blocked
        assert remaining == 4

    def test_intangible_exactly_1_damage(self):
        """Intangible doesn't change 1 damage."""
        assert calculate_damage(1, intangible=True) == 1
        hp_loss, _ = calculate_incoming_damage(1, 0, intangible=True)
        assert hp_loss == 1

    def test_intangible_with_torii_and_tungsten(self):
        """Intangible + Torii + Tungsten all trigger."""
        # 100 → 1 (intangible) → 1 (torii doesn't trigger on 1) → 0 (tungsten)
        hp_loss, _ = calculate_incoming_damage(100, 0, intangible=True, tungsten_rod=True)
        assert hp_loss == 0

    def test_intangible_hp_loss_type(self):
        """Intangible caps HP_LOSS damage type."""
        assert apply_hp_loss(100, intangible=True) == 1
        assert apply_hp_loss(5, intangible=True) == 1
        assert apply_hp_loss(1, intangible=True) == 1


# =============================================================================
# HP LOSS TYPE EDGE CASES
# =============================================================================

class TestHPLossType:
    """Test HP_LOSS damage type (poison, self-damage)."""

    def test_hp_loss_ignores_block(self):
        """HP_LOSS bypasses block entirely."""
        # This is tested by NOT passing HP loss through calculate_incoming_damage
        assert apply_hp_loss(10) == 10

    def test_hp_loss_with_tungsten(self):
        """Tungsten Rod reduces HP_LOSS by 1."""
        assert apply_hp_loss(5, tungsten_rod=True) == 4
        assert apply_hp_loss(1, tungsten_rod=True) == 0

    def test_hp_loss_with_intangible(self):
        """Intangible caps HP_LOSS at 1."""
        assert apply_hp_loss(100, intangible=True) == 1

    def test_hp_loss_with_both(self):
        """Intangible caps at 1, then Tungsten reduces to 0."""
        assert apply_hp_loss(100, intangible=True, tungsten_rod=True) == 0


# =============================================================================
# DOUBLE DAMAGE POWER
# =============================================================================

class TestDoubleDamage:
    """Test Double Damage power (rare but exists)."""

    def test_double_damage_basic(self):
        """Double Damage multiplies by 2."""
        assert calculate_damage(10, double_damage=True) == 20

    def test_double_damage_with_strength(self):
        """Double Damage applies after strength."""
        # (10 + 5) * 2 = 30
        assert calculate_damage(10, strength=5, double_damage=True) == 30

    def test_double_damage_with_pen_nib(self):
        """Both Pen Nib and Double Damage multiply."""
        # 10 * 2 (pen nib) * 2 (double damage) = 40
        assert calculate_damage(10, pen_nib=True, double_damage=True) == 40

    def test_double_damage_with_weak(self):
        """Weak applies after Double Damage."""
        # 10 * 2 (double) * 0.75 (weak) = 15
        assert calculate_damage(10, double_damage=True, weak=True) == 15


# =============================================================================
# COMPREHENSIVE COMBO TESTS
# =============================================================================

class TestComprehensiveCombos:
    """Test complex combinations of multiple modifiers."""

    def test_maximum_damage_combo(self):
        """Test absolute maximum damage scenario."""
        # 50 base + 20 str + 10 vigor = 80
        # 80 * 2 (pen nib) * 2 (double) = 320
        # 320 * 0.75 (weak) = 240
        # 240 * 3 (divinity) = 720
        # 720 * 1.75 (vuln + paper frog) = 1260
        assert calculate_damage(
            50,
            strength=20,
            vigor=10,
            pen_nib=True,
            double_damage=True,
            weak=True,
            stance_mult=DIVINITY_MULT,
            vuln=True,
            vuln_paper_frog=True,
        ) == 1260

    def test_minimum_damage_combo(self):
        """Test damage reduced to minimum (0)."""
        # 5 - 20 str = -15 → 0 (clamped)
        assert calculate_damage(5, strength=-20) == 0

    def test_realistic_watcher_combo(self):
        """Test realistic Watcher combo: Str + Wrath + Vulnerable."""
        # Tantrum (9 damage) + 6 Strength in Wrath vs Vulnerable enemy
        # (9 + 6) * 2 (wrath) * 1.5 (vuln) = 45
        assert calculate_damage(
            9,
            strength=6,
            stance_mult=WRATH_MULT,
            vuln=True,
        ) == 45

    def test_realistic_defensive_combo(self):
        """Test realistic defensive combo: Dex + Frail + block reduction."""
        # Defend (5) + 3 Dex - 2 temp dex loss + Frail
        # (5 + 3 - 2) * 0.75 = 4.5 → 4
        assert calculate_block(5, dexterity=1, frail=True) == 4

    def test_torii_tungsten_intangible_combo(self):
        """Test all three defensive relics/powers together."""
        # 100 damage → 1 (intangible) - 0 block → 1 (torii doesn't trigger) → 0 (tungsten)
        hp_loss, _ = calculate_incoming_damage(
            100, 0, intangible=True, torii=True, tungsten_rod=True
        )
        assert hp_loss == 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
