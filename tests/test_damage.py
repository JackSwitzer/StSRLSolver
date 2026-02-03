"""
Damage Calculation Tests

Tests damage formulas against known game mechanics.
Verifies order of operations and special cases.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.calc.damage import (
    calculate_damage, calculate_block, calculate_incoming_damage,
    WEAK_MULT, VULN_MULT, WRATH_MULT, DIVINITY_MULT, FRAIL_MULT,
)


class TestBasicDamage:
    """Test basic damage calculation."""

    def test_base_damage_only(self):
        """Pure base damage with no modifiers."""
        assert calculate_damage(10) == 10
        assert calculate_damage(0) == 0
        assert calculate_damage(1) == 1

    def test_strength_adds(self):
        """Strength adds to base damage."""
        # Strike (6) + 3 Strength = 9
        assert calculate_damage(6, strength=3) == 9
        # Negative strength
        assert calculate_damage(6, strength=-2) == 4

    def test_vigor_adds(self):
        """Vigor adds to damage (one-time)."""
        assert calculate_damage(6, vigor=5) == 11
        # Vigor + Strength both add
        assert calculate_damage(6, strength=2, vigor=3) == 11

    def test_minimum_zero(self):
        """Damage cannot go below 0."""
        assert calculate_damage(5, strength=-10) == 0
        assert calculate_damage(1, strength=-5) == 0


class TestMultipliers:
    """Test damage multipliers."""

    def test_weak_reduces_damage(self):
        """Weak reduces outgoing damage by 25%."""
        # 10 * 0.75 = 7.5 -> 7
        assert calculate_damage(10, weak=True) == 7
        # 8 * 0.75 = 6
        assert calculate_damage(8, weak=True) == 6

    def test_vulnerable_increases_damage(self):
        """Vulnerable increases incoming damage by 50%."""
        # 10 * 1.5 = 15
        assert calculate_damage(10, vuln=True) == 15
        # 7 * 1.5 = 10.5 -> 10
        assert calculate_damage(7, vuln=True) == 10

    def test_pen_nib_doubles(self):
        """Pen Nib doubles damage."""
        assert calculate_damage(10, pen_nib=True) == 20
        assert calculate_damage(6, pen_nib=True) == 12

    def test_wrath_stance(self):
        """Wrath stance doubles damage."""
        assert calculate_damage(10, stance_mult=WRATH_MULT) == 20
        # With strength: (6+3) * 2 = 18
        assert calculate_damage(6, strength=3, stance_mult=WRATH_MULT) == 18

    def test_divinity_stance(self):
        """Divinity stance triples damage."""
        assert calculate_damage(10, stance_mult=DIVINITY_MULT) == 30
        # With vulnerable: 10 * 3 * 1.5 = 45
        assert calculate_damage(10, stance_mult=DIVINITY_MULT, vuln=True) == 45


class TestOrderOfOperations:
    """Test correct order of damage calculation.

    Java order:
    1. Base + flat (Strength, Vigor)
    2. Attacker multipliers (Pen Nib, Weak)
    3. Stance multiplier
    4. Defender multipliers (Vulnerable)
    5. Intangible cap
    """

    def test_strength_before_weak(self):
        """Strength adds before weak multiplies."""
        # (6 + 4) * 0.75 = 7.5 -> 7
        assert calculate_damage(6, strength=4, weak=True) == 7

    def test_strength_weak_vulnerable(self):
        """Full chain: base + str -> weak -> vuln."""
        # (6 + 2) * 0.75 * 1.5 = 9
        result = calculate_damage(6, strength=2, weak=True, vuln=True)
        assert result == 9

    def test_pen_nib_before_weak(self):
        """Pen Nib applies before Weak."""
        # (10 * 2) * 0.75 = 15
        assert calculate_damage(10, pen_nib=True, weak=True) == 15

    def test_stance_after_pen_nib(self):
        """Stance multiplier after Pen Nib."""
        # 10 * 2 (pen nib) * 2 (wrath) = 40
        result = calculate_damage(10, pen_nib=True, stance_mult=WRATH_MULT)
        assert result == 40


class TestIntangible:
    """Test Intangible mechanic."""

    def test_intangible_caps_at_one(self):
        """Intangible caps all damage at 1."""
        assert calculate_damage(100, intangible=True) == 1
        assert calculate_damage(10, intangible=True) == 1
        # Even with huge multipliers
        result = calculate_damage(
            50,
            strength=10,
            stance_mult=DIVINITY_MULT,
            vuln=True,
            intangible=True
        )
        assert result == 1

    def test_intangible_allows_one(self):
        """1 damage goes through Intangible."""
        assert calculate_damage(1, intangible=True) == 1

    def test_intangible_on_zero(self):
        """0 damage stays 0 with Intangible."""
        assert calculate_damage(0, intangible=True) == 0


class TestBlockCalculation:
    """Test block calculation."""

    def test_base_block(self):
        """Pure block with no modifiers."""
        assert calculate_block(5) == 5
        assert calculate_block(10) == 10

    def test_dexterity_adds(self):
        """Dexterity adds to block."""
        # Defend (5) + 3 Dex = 8
        assert calculate_block(5, dexterity=3) == 8
        # Negative dex
        assert calculate_block(5, dexterity=-2) == 3

    def test_frail_reduces(self):
        """Frail reduces block by 25%."""
        # 8 * 0.75 = 6
        assert calculate_block(8, frail=True) == 6
        # 10 * 0.75 = 7.5 -> 7
        assert calculate_block(10, frail=True) == 7

    def test_dexterity_before_frail(self):
        """Dexterity adds before Frail multiplies."""
        # (5 + 3) * 0.75 = 6
        assert calculate_block(5, dexterity=3, frail=True) == 6

    def test_minimum_zero(self):
        """Block cannot go below 0."""
        assert calculate_block(5, dexterity=-10) == 0


class TestSpecialCases:
    """Test edge cases and special interactions."""

    def test_rounding_floor(self):
        """Damage rounds down (floor)."""
        # 7 * 0.75 = 5.25 -> 5
        assert calculate_damage(7, weak=True) == 5
        # 7 * 1.5 = 10.5 -> 10
        assert calculate_damage(7, vuln=True) == 10

    def test_paper_frog_vulnerable(self):
        """Paper Frog increases vulnerable to 75%."""
        # 10 * 1.75 = 17.5 -> 17
        assert calculate_damage(10, vuln=True, vuln_paper_frog=True) == 17

    def test_complex_scenario(self):
        """Complex real-game scenario."""
        # Eruption (9 dmg) in Wrath, with 2 Str, enemy Vulnerable
        # (9 + 2) * 2.0 * 1.5 = 33
        result = calculate_damage(
            9,
            strength=2,
            stance_mult=WRATH_MULT,
            vuln=True
        )
        assert result == 33


class TestIncomingDamage:
    """Test incoming damage calculation."""

    def test_wrath_incoming(self):
        """Wrath doubles incoming damage."""
        # calculate_incoming_damage returns (hp_loss, block_remaining)
        hp_loss, _ = calculate_incoming_damage(
            10,
            block=0,
            is_wrath=True
        )
        assert hp_loss == 20

    def test_divinity_no_incoming_mult(self):
        """Divinity does NOT increase incoming damage (only outgoing)."""
        # Divinity should behave like neutral stance for incoming damage
        hp_loss, _ = calculate_incoming_damage(
            10,
            block=0,
            is_wrath=False
        )
        assert hp_loss == 10  # NOT 30 (10 * 3.0)

    def test_block_reduces(self):
        """Block reduces incoming damage."""
        # 10 damage - 5 block = 5 HP loss
        hp_loss, block_left = calculate_incoming_damage(10, block=5)
        assert hp_loss == 5
        assert block_left == 0

        # 10 damage - 15 block = 0 HP loss, 5 block remaining
        hp_loss, block_left = calculate_incoming_damage(10, block=15)
        assert hp_loss == 0
        assert block_left == 5


# =============================================================================
# EDGE CASE TESTS - Comprehensive coverage of Java-specific behavior
# =============================================================================


class TestRoundingBehavior:
    """Test Java-like floor rounding (always floor, never round)."""

    def test_floor_not_round_weak(self):
        """Weak always floors, never rounds."""
        # 7 * 0.75 = 5.25 -> 5 (floor, not round to 5)
        assert calculate_damage(7, weak=True) == 5
        # 9 * 0.75 = 6.75 -> 6 (floor, not round to 7)
        assert calculate_damage(9, weak=True) == 6
        # 11 * 0.75 = 8.25 -> 8
        assert calculate_damage(11, weak=True) == 8

    def test_floor_not_round_vulnerable(self):
        """Vulnerable always floors, never rounds."""
        # 7 * 1.5 = 10.5 -> 10 (floor, not round to 11)
        assert calculate_damage(7, vuln=True) == 10
        # 9 * 1.5 = 13.5 -> 13 (floor, not round to 14)
        assert calculate_damage(9, vuln=True) == 13
        # 3 * 1.5 = 4.5 -> 4
        assert calculate_damage(3, vuln=True) == 4

    def test_floor_not_round_frail(self):
        """Frail always floors, never rounds."""
        # 7 * 0.75 = 5.25 -> 5
        assert calculate_block(7, frail=True) == 5
        # 9 * 0.75 = 6.75 -> 6
        assert calculate_block(9, frail=True) == 6
        # 11 * 0.75 = 8.25 -> 8
        assert calculate_block(11, frail=True) == 8

    def test_floor_multiple_multipliers(self):
        """Floor after all multipliers, not between them."""
        # (8) * 0.75 (weak) * 1.5 (vuln) = 9.0 exactly
        assert calculate_damage(8, weak=True, vuln=True) == 9
        # (7 + 1) * 0.75 * 1.5 = 9
        assert calculate_damage(7, strength=1, weak=True, vuln=True) == 9
        # (7) * 0.75 * 1.5 = 7.875 -> 7
        assert calculate_damage(7, weak=True, vuln=True) == 7

    def test_floor_chain_with_stance(self):
        """Floor after stance multiplier applied."""
        # 7 * 2.0 (wrath) * 0.75 (weak) = 10.5 -> 10
        # Note: weak applied before stance in calculation order
        # (7) * 0.75 * 2.0 = 10.5 -> 10
        assert calculate_damage(7, weak=True, stance_mult=WRATH_MULT) == 10
        # 5 * 0.75 * 3.0 = 11.25 -> 11
        assert calculate_damage(5, weak=True, stance_mult=DIVINITY_MULT) == 11


class TestMinimumDamageZero:
    """Test that damage can never go below 0."""

    def test_negative_strength_minimum(self):
        """Large negative strength cannot make damage negative."""
        assert calculate_damage(5, strength=-10) == 0
        assert calculate_damage(1, strength=-100) == 0
        assert calculate_damage(0, strength=-5) == 0

    def test_negative_total_with_multipliers(self):
        """Negative base+flat still results in 0 after multipliers."""
        # (-5) * 0.75 = -3.75 -> 0
        assert calculate_damage(5, strength=-10, weak=True) == 0
        # (-5) * 1.5 = -7.5 -> 0
        assert calculate_damage(5, strength=-10, vuln=True) == 0

    def test_zero_base_with_negative_strength(self):
        """Zero base with negative strength."""
        assert calculate_damage(0, strength=-3) == 0

    def test_zero_with_all_multipliers(self):
        """Zero stays zero regardless of multipliers."""
        assert calculate_damage(0, weak=True, vuln=True, stance_mult=WRATH_MULT) == 0


class TestBlockMinimumZero:
    """Test that block can never go below 0."""

    def test_negative_dexterity_minimum(self):
        """Large negative dexterity cannot make block negative."""
        assert calculate_block(5, dexterity=-10) == 0
        assert calculate_block(1, dexterity=-100) == 0
        assert calculate_block(3, dexterity=-5) == 0

    def test_negative_with_frail(self):
        """Negative base+dex with frail still results in 0."""
        # (5 - 10) * 0.75 = -3.75 -> 0
        assert calculate_block(5, dexterity=-10, frail=True) == 0

    def test_zero_base_with_negative_dex(self):
        """Zero base with negative dexterity."""
        assert calculate_block(0, dexterity=-3) == 0

    def test_no_block_power(self):
        """No Block power sets block to 0."""
        assert calculate_block(10, no_block=True) == 0
        assert calculate_block(10, dexterity=5, no_block=True) == 0
        assert calculate_block(10, dexterity=5, frail=True, no_block=True) == 0


class TestAttackerMultipliers:
    """Test all attacker multipliers: Weak, Pen Nib, Paper Crane, Double Damage."""

    def test_weak_exact_075(self):
        """Weak multiplier is exactly 0.75."""
        # 100 * 0.75 = 75
        assert calculate_damage(100, weak=True) == 75
        # 4 * 0.75 = 3
        assert calculate_damage(4, weak=True) == 3

    def test_pen_nib_exact_2x(self):
        """Pen Nib exactly doubles damage."""
        assert calculate_damage(7, pen_nib=True) == 14
        assert calculate_damage(50, pen_nib=True) == 100

    def test_double_damage_2x(self):
        """Double Damage power doubles damage."""
        assert calculate_damage(7, double_damage=True) == 14
        assert calculate_damage(50, double_damage=True) == 100

    def test_pen_nib_and_double_damage_stack(self):
        """Pen Nib and Double Damage both apply (4x total)."""
        # 10 * 2 * 2 = 40
        assert calculate_damage(10, pen_nib=True, double_damage=True) == 40

    def test_paper_crane_weak(self):
        """Paper Crane makes Weak reduce by 40% instead of 25%."""
        # 100 * 0.60 = 60
        assert calculate_damage(100, weak=True, weak_paper_crane=True) == 60
        # 10 * 0.60 = 6
        assert calculate_damage(10, weak=True, weak_paper_crane=True) == 6

    def test_paper_crane_without_weak(self):
        """Paper Crane does nothing without Weak."""
        # Paper Crane flag alone shouldn't affect damage
        assert calculate_damage(10, weak_paper_crane=True) == 10

    def test_weak_pen_nib_order(self):
        """Pen Nib applies before Weak."""
        # (10 * 2) * 0.75 = 15
        assert calculate_damage(10, pen_nib=True, weak=True) == 15
        # Not: 10 * 0.75 * 2 = 15 (same result but different order)

    def test_weak_double_damage_pen_nib_order(self):
        """All attacker multipliers in order: pen_nib -> double_damage -> weak."""
        # (10 * 2 * 2) * 0.75 = 30
        assert calculate_damage(10, pen_nib=True, double_damage=True, weak=True) == 30


class TestDefenderMultipliers:
    """Test all defender multipliers: Vulnerable, Paper Frog, Odd Mushroom, Flight."""

    def test_vulnerable_exact_15(self):
        """Vulnerable multiplier is exactly 1.5."""
        assert calculate_damage(100, vuln=True) == 150
        assert calculate_damage(10, vuln=True) == 15

    def test_paper_frog_vulnerable(self):
        """Paper Frog increases vulnerable to 75%."""
        # 100 * 1.75 = 175
        assert calculate_damage(100, vuln=True, vuln_paper_frog=True) == 175
        # 10 * 1.75 = 17.5 -> 17
        assert calculate_damage(10, vuln=True, vuln_paper_frog=True) == 17

    def test_paper_frog_without_vulnerable(self):
        """Paper Frog does nothing without Vulnerable."""
        assert calculate_damage(10, vuln_paper_frog=True) == 10

    def test_flight_halves_damage(self):
        """Flight reduces damage by 50%."""
        assert calculate_damage(100, flight=True) == 50
        assert calculate_damage(10, flight=True) == 5
        # 7 * 0.5 = 3.5 -> 3
        assert calculate_damage(7, flight=True) == 3

    def test_vulnerable_flight_stack(self):
        """Vulnerable and Flight both apply."""
        # 10 * 1.5 * 0.5 = 7.5 -> 7
        assert calculate_damage(10, vuln=True, flight=True) == 7
        # 100 * 1.5 * 0.5 = 75
        assert calculate_damage(100, vuln=True, flight=True) == 75


class TestStanceMultipliers:
    """Test stance multipliers: Wrath (2x out, 2x in), Divinity (3x out, NO in), Calm (no mult)."""

    def test_wrath_outgoing_2x(self):
        """Wrath doubles outgoing damage."""
        assert calculate_damage(10, stance_mult=WRATH_MULT) == 20
        assert calculate_damage(7, stance_mult=WRATH_MULT) == 14

    def test_wrath_with_vulnerable(self):
        """Wrath + Vulnerable stack multiplicatively."""
        # 10 * 2.0 * 1.5 = 30
        assert calculate_damage(10, stance_mult=WRATH_MULT, vuln=True) == 30

    def test_divinity_outgoing_3x(self):
        """Divinity triples outgoing damage."""
        assert calculate_damage(10, stance_mult=DIVINITY_MULT) == 30
        assert calculate_damage(7, stance_mult=DIVINITY_MULT) == 21

    def test_divinity_with_vulnerable(self):
        """Divinity + Vulnerable stack multiplicatively."""
        # 10 * 3.0 * 1.5 = 45
        assert calculate_damage(10, stance_mult=DIVINITY_MULT, vuln=True) == 45

    def test_calm_no_multiplier(self):
        """Calm stance has no damage multiplier (1.0)."""
        # Calm is represented by stance_mult=1.0 (default)
        assert calculate_damage(10, stance_mult=1.0) == 10
        assert calculate_damage(10) == 10  # Default is 1.0

    def test_wrath_incoming_2x(self):
        """Wrath doubles incoming damage."""
        hp_loss, _ = calculate_incoming_damage(10, block=0, is_wrath=True)
        assert hp_loss == 20

    def test_divinity_no_incoming_multiplier(self):
        """Divinity does NOT affect incoming damage - only outgoing."""
        # There's no is_divinity parameter - incoming damage is neutral
        hp_loss, _ = calculate_incoming_damage(10, block=0)
        assert hp_loss == 10

    def test_wrath_incoming_with_vulnerable(self):
        """Wrath incoming + Vulnerable stack."""
        # 10 * 2.0 * 1.5 = 30
        hp_loss, _ = calculate_incoming_damage(10, block=0, is_wrath=True, vuln=True)
        assert hp_loss == 30


class TestIntangibleEdgeCases:
    """Test Intangible: caps damage at 1, but 0 stays 0."""

    def test_intangible_caps_at_1(self):
        """Intangible caps all damage at 1."""
        assert calculate_damage(100, intangible=True) == 1
        assert calculate_damage(1000, intangible=True) == 1
        assert calculate_damage(2, intangible=True) == 1

    def test_intangible_allows_1(self):
        """1 damage passes through Intangible unchanged."""
        assert calculate_damage(1, intangible=True) == 1

    def test_intangible_zero_stays_zero(self):
        """0 damage stays 0 with Intangible (not boosted to 1)."""
        assert calculate_damage(0, intangible=True) == 0
        # Negative strength causing 0
        assert calculate_damage(5, strength=-5, intangible=True) == 0

    def test_intangible_applied_last(self):
        """Intangible applied after all multipliers."""
        # (50 + 10) * 3.0 * 1.5 = 270 -> 1
        assert calculate_damage(50, strength=10, stance_mult=DIVINITY_MULT, vuln=True, intangible=True) == 1

    def test_intangible_incoming_damage(self):
        """Intangible caps incoming damage at 1."""
        hp_loss, _ = calculate_incoming_damage(100, block=0, intangible=True)
        assert hp_loss == 1

    def test_intangible_incoming_with_wrath(self):
        """Intangible applied after Wrath multiplier."""
        # 10 * 2 = 20 -> 1
        hp_loss, _ = calculate_incoming_damage(10, block=0, is_wrath=True, intangible=True)
        assert hp_loss == 1

    def test_intangible_hp_loss(self):
        """Intangible affects HP_LOSS damage type."""
        from packages.engine.calc.damage import apply_hp_loss
        assert apply_hp_loss(10, intangible=True) == 1
        assert apply_hp_loss(1, intangible=True) == 1
        assert apply_hp_loss(0, intangible=True) == 0


class TestToriiRelic:
    """Test Torii: damage 2-5 reduced to 1."""

    def test_torii_in_range(self):
        """Torii reduces damage 2-5 to 1."""
        hp_loss, _ = calculate_incoming_damage(2, block=0, torii=True)
        assert hp_loss == 1
        hp_loss, _ = calculate_incoming_damage(3, block=0, torii=True)
        assert hp_loss == 1
        hp_loss, _ = calculate_incoming_damage(4, block=0, torii=True)
        assert hp_loss == 1
        hp_loss, _ = calculate_incoming_damage(5, block=0, torii=True)
        assert hp_loss == 1

    def test_torii_below_range(self):
        """Torii does not affect damage below 2."""
        hp_loss, _ = calculate_incoming_damage(1, block=0, torii=True)
        assert hp_loss == 1
        hp_loss, _ = calculate_incoming_damage(0, block=0, torii=True)
        assert hp_loss == 0

    def test_torii_above_range(self):
        """Torii does not affect damage above 5."""
        hp_loss, _ = calculate_incoming_damage(6, block=0, torii=True)
        assert hp_loss == 6
        hp_loss, _ = calculate_incoming_damage(10, block=0, torii=True)
        assert hp_loss == 10
        hp_loss, _ = calculate_incoming_damage(100, block=0, torii=True)
        assert hp_loss == 100

    def test_torii_with_vulnerable(self):
        """Torii applies after Vulnerable."""
        # 3 * 1.5 = 4.5 -> 4, which is in Torii range -> 1
        hp_loss, _ = calculate_incoming_damage(3, block=0, vuln=True, torii=True)
        assert hp_loss == 1
        # 4 * 1.5 = 6, above Torii range -> 6
        hp_loss, _ = calculate_incoming_damage(4, block=0, vuln=True, torii=True)
        assert hp_loss == 6

    def test_torii_with_intangible(self):
        """Intangible caps before Torii check."""
        # 100 -> 1 (intangible), Torii doesn't apply to 1
        hp_loss, _ = calculate_incoming_damage(100, block=0, intangible=True, torii=True)
        assert hp_loss == 1


class TestTungstenRod:
    """Test Tungsten Rod: -1 from all HP loss."""

    def test_tungsten_rod_reduces_hp_loss(self):
        """Tungsten Rod reduces HP loss by 1."""
        hp_loss, _ = calculate_incoming_damage(10, block=5, tungsten_rod=True)
        # 10 - 5 = 5 HP loss -> 4 with Tungsten Rod
        assert hp_loss == 4

    def test_tungsten_rod_minimum_zero(self):
        """Tungsten Rod cannot make HP loss negative."""
        hp_loss, _ = calculate_incoming_damage(1, block=0, tungsten_rod=True)
        # 1 - 1 = 0
        assert hp_loss == 0

    def test_tungsten_rod_no_effect_on_blocked(self):
        """Tungsten Rod only affects unblocked damage."""
        hp_loss, block_left = calculate_incoming_damage(5, block=10, tungsten_rod=True)
        # Fully blocked, no HP loss
        assert hp_loss == 0
        assert block_left == 5

    def test_tungsten_rod_with_intangible(self):
        """Tungsten Rod + Intangible: 1 -> 0."""
        hp_loss, _ = calculate_incoming_damage(100, block=0, intangible=True, tungsten_rod=True)
        # 100 -> 1 (intangible) -> 0 (tungsten)
        assert hp_loss == 0

    def test_tungsten_rod_hp_loss_type(self):
        """Tungsten Rod affects HP_LOSS damage."""
        from packages.engine.calc.damage import apply_hp_loss
        assert apply_hp_loss(5, tungsten_rod=True) == 4
        assert apply_hp_loss(1, tungsten_rod=True) == 0

    def test_tungsten_rod_with_torii(self):
        """Tungsten Rod + Torii interaction."""
        # 4 damage -> 1 (torii) -> 0 (tungsten)
        hp_loss, _ = calculate_incoming_damage(4, block=0, torii=True, tungsten_rod=True)
        assert hp_loss == 0


class TestFrailDexterityInteraction:
    """Test Frail + Dexterity interaction."""

    def test_dexterity_added_before_frail(self):
        """Dexterity adds before Frail multiplies."""
        # (5 + 3) * 0.75 = 6
        assert calculate_block(5, dexterity=3, frail=True) == 6
        # (10 + 5) * 0.75 = 11.25 -> 11
        assert calculate_block(10, dexterity=5, frail=True) == 11

    def test_negative_dex_with_frail(self):
        """Negative dexterity + Frail."""
        # (10 - 3) * 0.75 = 5.25 -> 5
        assert calculate_block(10, dexterity=-3, frail=True) == 5
        # (5 - 2) * 0.75 = 2.25 -> 2
        assert calculate_block(5, dexterity=-2, frail=True) == 2

    def test_frail_with_large_positive_dex(self):
        """Large positive dex with Frail."""
        # (5 + 10) * 0.75 = 11.25 -> 11
        assert calculate_block(5, dexterity=10, frail=True) == 11
        # (3 + 20) * 0.75 = 17.25 -> 17
        assert calculate_block(3, dexterity=20, frail=True) == 17


class TestOrderOfOperationsComplete:
    """Test complete order: base + flat -> attacker mult -> stance -> defender mult -> intangible."""

    def test_full_order_of_operations(self):
        """Full chain: (base + str + vigor) * pen_nib * weak * stance * vuln * flight -> intangible."""
        # (6 + 3 + 2) = 11
        # 11 * 2 (pen_nib) = 22
        # 22 * 0.75 (weak) = 16.5
        # 16.5 * 2.0 (wrath) = 33
        # 33 * 1.5 (vuln) = 49.5
        # 49.5 * 0.5 (flight) = 24.75 -> 24
        result = calculate_damage(
            6,
            strength=3,
            vigor=2,
            pen_nib=True,
            weak=True,
            stance_mult=WRATH_MULT,
            vuln=True,
            flight=True
        )
        assert result == 24

    def test_intangible_always_last(self):
        """Intangible always applies last, capping at 1."""
        # Same calculation as above but with intangible
        result = calculate_damage(
            6,
            strength=3,
            vigor=2,
            pen_nib=True,
            weak=True,
            stance_mult=WRATH_MULT,
            vuln=True,
            flight=True,
            intangible=True
        )
        assert result == 1

    def test_attacker_mult_before_stance(self):
        """Pen Nib/Weak apply before stance multiplier."""
        # 10 * 2 (pen_nib) * 2 (wrath) = 40
        assert calculate_damage(10, pen_nib=True, stance_mult=WRATH_MULT) == 40
        # vs 10 * 2 (wrath) * 2 (pen_nib) = 40 (same result but different conceptual order)

    def test_stance_before_defender_mult(self):
        """Stance multiplier before Vulnerable."""
        # 10 * 2 (wrath) * 1.5 (vuln) = 30
        assert calculate_damage(10, stance_mult=WRATH_MULT, vuln=True) == 30

    def test_flat_adds_before_all_mults(self):
        """Strength/Vigor add before any multipliers."""
        # (10 + 5) * 0.75 (weak) = 11.25 -> 11
        assert calculate_damage(10, strength=5, weak=True) == 11
        # (10 + 3 + 2) * 0.75 = 11.25 -> 11
        assert calculate_damage(10, strength=3, vigor=2, weak=True) == 11


class TestMultiHitAttacks:
    """Test multi-hit attacks with per-hit modifiers."""

    def test_multi_hit_basic(self):
        """Simulate multi-hit with same modifiers per hit."""
        # Flurry of Blows: 4 damage x3
        damage_per_hit = calculate_damage(4, strength=2)  # 6 each
        total = damage_per_hit * 3
        assert damage_per_hit == 6
        assert total == 18

    def test_multi_hit_with_weak(self):
        """Multi-hit with Weak applies to each hit."""
        # 4 * 0.75 = 3 per hit
        damage_per_hit = calculate_damage(4, weak=True)
        assert damage_per_hit == 3
        # x3 hits
        assert damage_per_hit * 3 == 9

    def test_multi_hit_vulnerable(self):
        """Multi-hit against Vulnerable."""
        # 4 * 1.5 = 6 per hit
        damage_per_hit = calculate_damage(4, vuln=True)
        assert damage_per_hit == 6
        # x5 hits
        assert damage_per_hit * 5 == 30

    def test_multi_hit_wrath(self):
        """Multi-hit in Wrath stance."""
        # 4 * 2 = 8 per hit
        damage_per_hit = calculate_damage(4, stance_mult=WRATH_MULT)
        assert damage_per_hit == 8
        # x4 hits
        assert damage_per_hit * 4 == 32

    def test_multi_hit_intangible_per_hit(self):
        """Each hit against Intangible is capped at 1."""
        # Each of 5 hits deals 1 damage
        damage_per_hit = calculate_damage(10, intangible=True)
        assert damage_per_hit == 1
        # x5 hits = 5 total
        total = damage_per_hit * 5
        assert total == 5

    def test_akabeko_first_hit_bonus(self):
        """Akabeko: +8 damage on first attack each combat (simulated)."""
        # First hit: 6 + 8 = 14
        first_hit = calculate_damage(6, vigor=8)  # Vigor simulates Akabeko
        assert first_hit == 14
        # Subsequent hits: just 6
        subsequent_hit = calculate_damage(6)
        assert subsequent_hit == 6


class TestIncomingDamageEdgeCases:
    """Test incoming damage calculation edge cases."""

    def test_incoming_wrath_vulnerable_stack(self):
        """Wrath + Vulnerable stack on incoming damage."""
        # 10 * 2 * 1.5 = 30
        hp_loss, _ = calculate_incoming_damage(10, block=0, is_wrath=True, vuln=True)
        assert hp_loss == 30

    def test_incoming_odd_mushroom(self):
        """Odd Mushroom reduces Vulnerable to 25%."""
        # 10 * 1.25 = 12.5 -> 12
        hp_loss, _ = calculate_incoming_damage(10, block=0, vuln=True, vuln_odd_mushroom=True)
        assert hp_loss == 12

    def test_block_absorbs_modified_damage(self):
        """Block absorbs damage after multipliers."""
        # 10 * 2 (wrath) = 20, block 15 -> 5 HP loss
        hp_loss, block_left = calculate_incoming_damage(10, block=15, is_wrath=True)
        assert hp_loss == 5
        assert block_left == 0

    def test_block_overflow(self):
        """Excess block remains after absorbing damage."""
        hp_loss, block_left = calculate_incoming_damage(10, block=25)
        assert hp_loss == 0
        assert block_left == 15

    def test_zero_incoming_damage(self):
        """Zero damage doesn't consume block."""
        hp_loss, block_left = calculate_incoming_damage(0, block=10)
        assert hp_loss == 0
        assert block_left == 10


class TestHPLossEdgeCases:
    """Test HP loss that can't go below 0."""

    def test_hp_loss_basic(self):
        """Basic HP loss calculation."""
        from packages.engine.calc.damage import apply_hp_loss
        assert apply_hp_loss(10) == 10
        assert apply_hp_loss(0) == 0

    def test_hp_loss_intangible(self):
        """Intangible caps HP loss at 1."""
        from packages.engine.calc.damage import apply_hp_loss
        assert apply_hp_loss(100, intangible=True) == 1
        assert apply_hp_loss(5, intangible=True) == 1
        assert apply_hp_loss(1, intangible=True) == 1

    def test_hp_loss_tungsten_rod(self):
        """Tungsten Rod reduces HP loss by 1."""
        from packages.engine.calc.damage import apply_hp_loss
        assert apply_hp_loss(10, tungsten_rod=True) == 9
        assert apply_hp_loss(1, tungsten_rod=True) == 0

    def test_hp_loss_intangible_tungsten_combined(self):
        """Intangible + Tungsten Rod: 1 - 1 = 0."""
        from packages.engine.calc.damage import apply_hp_loss
        # 100 -> 1 (intangible) -> 0 (tungsten)
        assert apply_hp_loss(100, intangible=True, tungsten_rod=True) == 0

    def test_hp_loss_zero_unaffected(self):
        """Zero HP loss unaffected by relics."""
        from packages.engine.calc.damage import apply_hp_loss
        assert apply_hp_loss(0, intangible=True) == 0
        assert apply_hp_loss(0, tungsten_rod=True) == 0
        assert apply_hp_loss(0, intangible=True, tungsten_rod=True) == 0


class TestConvenienceFunctions:
    """Test convenience functions for common scenarios."""

    def test_wrath_damage_function(self):
        """Test wrath_damage convenience function."""
        from packages.engine.calc.damage import wrath_damage
        assert wrath_damage(10) == 20
        assert wrath_damage(10, strength=3) == 26  # (10+3)*2
        assert wrath_damage(10, vuln=True) == 30  # 10*2*1.5

    def test_divinity_damage_function(self):
        """Test divinity_damage convenience function."""
        from packages.engine.calc.damage import divinity_damage
        assert divinity_damage(10) == 30
        assert divinity_damage(10, strength=3) == 39  # (10+3)*3
        assert divinity_damage(10, vuln=True) == 45  # 10*3*1.5


class TestRealGameScenarios:
    """Test real in-game scenarios for accuracy."""

    def test_strike_base(self):
        """Strike deals 6 damage."""
        assert calculate_damage(6) == 6

    def test_strike_with_strength(self):
        """Strike + 3 Strength = 9."""
        assert calculate_damage(6, strength=3) == 9

    def test_eruption_wrath(self):
        """Eruption (9) in Wrath = 18."""
        assert calculate_damage(9, stance_mult=WRATH_MULT) == 18

    def test_eruption_wrath_str_vuln(self):
        """Eruption + 2 Str in Wrath vs Vuln = (9+2)*2*1.5 = 33."""
        assert calculate_damage(9, strength=2, stance_mult=WRATH_MULT, vuln=True) == 33

    def test_ragnarok_wrath(self):
        """Ragnarok (5x5) in Wrath = 10 per hit."""
        damage_per_hit = calculate_damage(5, stance_mult=WRATH_MULT)
        assert damage_per_hit == 10
        assert damage_per_hit * 5 == 50

    def test_conclude_divinity(self):
        """Conclude (12) in Divinity = 36."""
        assert calculate_damage(12, stance_mult=DIVINITY_MULT) == 36

    def test_defend_base(self):
        """Defend gives 5 block."""
        assert calculate_block(5) == 5

    def test_defend_with_dex(self):
        """Defend + 3 Dex = 8."""
        assert calculate_block(5, dexterity=3) == 8

    def test_defend_frail(self):
        """Defend with Frail = 3 (5*0.75=3.75->3)."""
        assert calculate_block(5, frail=True) == 3

    def test_vigilance_dex_frail(self):
        """Vigilance (8) + 2 Dex with Frail = (8+2)*0.75 = 7."""
        assert calculate_block(8, dexterity=2, frail=True) == 7

    def test_nemesis_intangible(self):
        """Nemesis attack vs Intangible = 1."""
        assert calculate_damage(45, intangible=True) == 1

    def test_bronze_automaton_vs_intangible(self):
        """Automaton's HYPER BEAM vs Intangible = 1."""
        assert calculate_damage(50, intangible=True) == 1

    def test_heart_multi_hit_intangible(self):
        """Heart's 15x2 vs Intangible = 2 total (1 per hit)."""
        damage_per_hit = calculate_damage(15, intangible=True)
        assert damage_per_hit == 1
        assert damage_per_hit * 2 == 2


class TestExtremeValues:
    """Test extreme/boundary values."""

    def test_very_large_damage(self):
        """Very large damage values."""
        assert calculate_damage(10000) == 10000
        assert calculate_damage(10000, strength=1000) == 11000

    def test_very_large_strength(self):
        """Very large strength values."""
        assert calculate_damage(1, strength=1000) == 1001

    def test_very_negative_strength(self):
        """Very negative strength clamped to 0."""
        assert calculate_damage(100, strength=-1000) == 0

    def test_very_large_block(self):
        """Very large block values."""
        assert calculate_block(10000) == 10000
        assert calculate_block(10000, dexterity=1000) == 11000

    def test_chain_of_all_multipliers(self):
        """All multipliers chained."""
        # (10 + 5) * 2 * 2 * 0.75 * 3.0 * 1.75 * 0.5 = 118.125 -> 118
        result = calculate_damage(
            10,
            strength=5,
            pen_nib=True,
            double_damage=True,
            weak=True,
            stance_mult=DIVINITY_MULT,
            vuln=True,
            vuln_paper_frog=True,
            flight=True
        )
        # (10+5) = 15
        # 15 * 2 = 30 (pen_nib)
        # 30 * 2 = 60 (double_damage)
        # 60 * 0.75 = 45 (weak)
        # 45 * 3.0 = 135 (divinity)
        # 135 * 1.75 = 236.25 (paper_frog_vuln)
        # 236.25 * 0.5 = 118.125 -> 118
        assert result == 118


class TestFlightInteractions:
    """Test Flight power interactions."""

    def test_flight_basic(self):
        """Flight halves damage."""
        assert calculate_damage(10, flight=True) == 5
        assert calculate_damage(100, flight=True) == 50

    def test_flight_with_vulnerable(self):
        """Flight + Vulnerable."""
        # 10 * 1.5 * 0.5 = 7.5 -> 7
        assert calculate_damage(10, vuln=True, flight=True) == 7

    def test_flight_with_wrath(self):
        """Flight vs Wrath attacker."""
        # 10 * 2 * 0.5 = 10
        assert calculate_damage(10, stance_mult=WRATH_MULT, flight=True) == 10

    def test_flight_rounding(self):
        """Flight rounding edge cases."""
        # 7 * 0.5 = 3.5 -> 3
        assert calculate_damage(7, flight=True) == 3
        # 9 * 0.5 = 4.5 -> 4
        assert calculate_damage(9, flight=True) == 4
        # 11 * 0.5 = 5.5 -> 5
        assert calculate_damage(11, flight=True) == 5


class TestZeroDamageEdgeCases:
    """Test zero damage edge cases."""

    def test_zero_base_no_modifiers(self):
        """Zero base damage."""
        assert calculate_damage(0) == 0

    def test_zero_base_with_strength(self):
        """Zero base + Strength."""
        assert calculate_damage(0, strength=5) == 5

    def test_zero_with_multipliers(self):
        """Zero damage unaffected by multipliers."""
        assert calculate_damage(0, weak=True) == 0
        assert calculate_damage(0, vuln=True) == 0
        assert calculate_damage(0, stance_mult=WRATH_MULT) == 0

    def test_zero_after_negative_strength(self):
        """Zero after negative strength still 0."""
        assert calculate_damage(5, strength=-5) == 0

    def test_zero_block_no_modifiers(self):
        """Zero block stays zero."""
        assert calculate_block(0) == 0

    def test_zero_block_with_dex(self):
        """Zero block + Dexterity."""
        assert calculate_block(0, dexterity=5) == 5


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
