"""
Damage Pipeline Audit Tests

Verifies Python damage calculations match decompiled Java source.
Each test documents the Java behavior and checks Python matches.
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
    FLIGHT_MULT,
)


# =========================================================================
# Constants verification
# =========================================================================


class TestConstants:
    """Verify multiplier constants match decompiled Java."""

    def test_weak_mult(self):
        assert WEAK_MULT == 0.75  # WeakPower.atDamageGive: damage * 0.75f

    def test_weak_paper_crane(self):
        assert WEAK_MULT_PAPER_CRANE == 0.60  # WeakPower.atDamageGive: damage * 0.6f

    def test_vuln_mult(self):
        assert VULN_MULT == 1.50  # VulnerablePower.atDamageReceive: damage * 1.5f

    def test_vuln_odd_mushroom(self):
        assert VULN_MULT_ODD_MUSHROOM == 1.25  # VulnerablePower: damage * 1.25f

    def test_vuln_paper_frog(self):
        assert VULN_MULT_PAPER_FROG == 1.75  # VulnerablePower: damage * 1.75f

    def test_frail_mult(self):
        assert FRAIL_MULT == 0.75

    def test_wrath_mult(self):
        assert WRATH_MULT == 2.0  # WrathStance.atDamageGive: damage * 2.0f

    def test_divinity_mult(self):
        assert DIVINITY_MULT == 3.0  # DivinityStance.atDamageGive: damage * 3.0f

    def test_flight_mult(self):
        assert FLIGHT_MULT == 0.50  # FlightPower.atDamageFinalReceive: damage / 2.0f


# =========================================================================
# Outgoing Damage: calculate_damage()
# =========================================================================


class TestOutgoingDamage:
    """
    Java chain (DamageInfo.applyPowers, player attacking):
    1. atDamageGive: Strength(+flat), Vigor(+flat), Weak(*0.75), PenNib(*2.0)
    2. stance.atDamageGive: Wrath(*2.0), Divinity(*3.0)
    3. atDamageReceive: Vulnerable(*1.5)
    4. atDamageFinalReceive: Intangible(cap 1), Flight(/2.0)
    5. MathUtils.floor, min 0
    """

    def test_base_damage(self):
        assert calculate_damage(6) == 6

    def test_strength_positive(self):
        # Java: StrengthPower.atDamageGive adds this.amount
        assert calculate_damage(6, strength=3) == 9

    def test_strength_negative(self):
        assert calculate_damage(6, strength=-2) == 4

    def test_strength_reduces_to_zero(self):
        assert calculate_damage(6, strength=-10) == 0

    def test_vigor(self):
        # Java: VigorPower.atDamageGive adds this.amount (same hook as Strength)
        assert calculate_damage(6, vigor=5) == 11

    def test_strength_plus_vigor(self):
        assert calculate_damage(6, strength=3, vigor=5) == 14

    def test_weak(self):
        # Java: WeakPower.atDamageGive: damage * 0.75f
        assert calculate_damage(10, weak=True) == 7  # 10 * 0.75 = 7.5 -> 7

    def test_weak_paper_crane(self):
        # Java: WeakPower.atDamageGive: damage * 0.6f (enemy has Weak, player has Paper Crane)
        assert calculate_damage(10, weak=True, weak_paper_crane=True) == 6  # 10 * 0.6

    def test_pen_nib(self):
        # Java: PenNibPower.atDamageGive: damage * 2.0f
        assert calculate_damage(6, pen_nib=True) == 12

    def test_wrath_stance(self):
        # Java: WrathStance.atDamageGive: damage * 2.0f
        assert calculate_damage(6, stance_mult=WRATH_MULT) == 12

    def test_divinity_stance(self):
        # Java: DivinityStance.atDamageGive: damage * 3.0f
        assert calculate_damage(6, stance_mult=DIVINITY_MULT) == 18

    def test_vulnerable(self):
        # Java: VulnerablePower.atDamageReceive: damage * 1.5f
        assert calculate_damage(6, vuln=True) == 9

    def test_vulnerable_paper_frog(self):
        # Java: VulnerablePower.atDamageReceive: damage * 1.75f
        assert calculate_damage(10, vuln=True, vuln_paper_frog=True) == 17  # 10 * 1.75

    def test_flight(self):
        # Java: FlightPower.atDamageFinalReceive: damage / 2.0f
        assert calculate_damage(10, flight=True) == 5

    def test_intangible_caps_at_1(self):
        # Java: IntangiblePower.atDamageFinalReceive: if damage > 1 -> 1
        assert calculate_damage(100, intangible=True) == 1

    def test_intangible_1_stays_1(self):
        assert calculate_damage(1, intangible=True) == 1

    def test_intangible_0_stays_0(self):
        assert calculate_damage(0, intangible=True) == 0

    def test_wrath_plus_vuln(self):
        # Java: (6 * 2.0) * 1.5 = 18.0 -> 18
        assert calculate_damage(6, stance_mult=WRATH_MULT, vuln=True) == 18

    def test_str_wrath_vuln(self):
        # Java: (6+3) * 2.0 * 1.5 = 27.0 -> 27
        assert calculate_damage(6, strength=3, stance_mult=WRATH_MULT, vuln=True) == 27

    def test_weak_wrath(self):
        # Java: 10 * 0.75 * 2.0 = 15.0 -> 15
        assert calculate_damage(10, weak=True, stance_mult=WRATH_MULT) == 15

    def test_pen_nib_wrath_vuln(self):
        # Java: 6 * 2.0(pen) * 2.0(wrath) * 1.5(vuln) = 36.0 -> 36
        assert calculate_damage(6, pen_nib=True, stance_mult=WRATH_MULT, vuln=True) == 36

    def test_flight_plus_vuln(self):
        # Java: 10 * 1.5(vuln) * 0.5(flight) = 7.5 -> 7
        assert calculate_damage(10, vuln=True, flight=True) == 7

    def test_intangible_overrides_everything(self):
        # Even with huge multipliers, intangible caps at 1
        assert calculate_damage(50, strength=10, stance_mult=WRATH_MULT, vuln=True, intangible=True) == 1


class TestOutgoingDamageRounding:
    """
    Java floors once at the end. Python must match.
    These tests catch rounding errors from intermediate truncation.
    """

    def test_weak_rounds_down(self):
        # 7 * 0.75 = 5.25 -> 5
        assert calculate_damage(7, weak=True) == 5

    def test_str_weak_vuln_single_chain(self):
        """
        CRITICAL: This tests the single-chain computation.
        Java: (6+3) * 0.75 * 1.5 = 10.125 -> floor -> 10
        Python calculate_damage with vuln=True should also give 10.
        """
        result = calculate_damage(6, strength=3, weak=True, vuln=True)
        assert result == 10, f"Expected 10 (Java single-chain), got {result}"

    def test_weak_vuln_odd_value(self):
        # 9 * 0.75 * 1.5 = 10.125 -> 10
        assert calculate_damage(9, weak=True, vuln=True) == 10

    def test_flight_rounds_down(self):
        # 7 * 0.5 = 3.5 -> 3
        assert calculate_damage(7, flight=True) == 3

    def test_weak_flight(self):
        # 10 * 0.75 * 0.5 = 3.75 -> 3
        assert calculate_damage(10, weak=True, flight=True) == 3

    def test_str_weak_wrath_vuln(self):
        # Java: (4+2) * 0.75 * 2.0 * 1.5 = 13.5 -> 13
        assert calculate_damage(4, strength=2, weak=True, stance_mult=WRATH_MULT, vuln=True) == 13


# =========================================================================
# Block: calculate_block()
# =========================================================================


class TestBlock:
    """
    Java chain (AbstractCard.applyPowersToBlock):
    1. Base block
    2. + Dexterity
    3. * Frail (0.75)
    4. Floor, min 0
    """

    def test_base_block(self):
        assert calculate_block(5) == 5

    def test_dexterity_positive(self):
        assert calculate_block(5, dexterity=2) == 7

    def test_dexterity_negative(self):
        assert calculate_block(5, dexterity=-2) == 3

    def test_dexterity_clamps_zero(self):
        assert calculate_block(5, dexterity=-10) == 0

    def test_frail(self):
        # 8 * 0.75 = 6
        assert calculate_block(8, frail=True) == 6

    def test_dex_plus_frail(self):
        # (5+2) * 0.75 = 5.25 -> 5
        assert calculate_block(5, dexterity=2, frail=True) == 5

    def test_frail_rounds_down(self):
        # 5 * 0.75 = 3.75 -> 3
        assert calculate_block(5, frail=True) == 3

    def test_no_block_power(self):
        assert calculate_block(10, no_block=True) == 0


# =========================================================================
# Incoming Damage: calculate_incoming_damage()
# =========================================================================


class TestIncomingDamage:
    """
    Java chain (AbstractPlayer.damage):
    1. applyPowers computes info.output (Wrath receive, Vuln, Intangible already applied)
    2. IntangiblePlayer redundant check
    3. decrementBlock
    4. onAttacked: Torii (AFTER block)
    5. onLoseHpLast: Tungsten Rod (AFTER Torii)
    6. HP subtraction
    """

    def test_basic_partial_block(self):
        hp_loss, blk = calculate_incoming_damage(10, 5)
        assert hp_loss == 5
        assert blk == 0

    def test_fully_blocked(self):
        hp_loss, blk = calculate_incoming_damage(5, 10)
        assert hp_loss == 0
        assert blk == 5

    def test_no_block(self):
        hp_loss, blk = calculate_incoming_damage(10, 0)
        assert hp_loss == 10

    def test_wrath_doubles_incoming(self):
        # Java: Wrath atDamageReceive * 2.0
        hp_loss, blk = calculate_incoming_damage(10, 0, is_wrath=True)
        assert hp_loss == 20

    def test_wrath_with_block(self):
        hp_loss, blk = calculate_incoming_damage(10, 5, is_wrath=True)
        assert hp_loss == 15  # 20 - 5
        assert blk == 0

    def test_vuln_incoming(self):
        hp_loss, blk = calculate_incoming_damage(10, 0, vuln=True)
        assert hp_loss == 15

    def test_vuln_odd_mushroom(self):
        hp_loss, blk = calculate_incoming_damage(10, 0, vuln=True, vuln_odd_mushroom=True)
        assert hp_loss == 12  # 10 * 1.25 = 12.5 -> 12

    def test_intangible_incoming(self):
        hp_loss, blk = calculate_incoming_damage(100, 0, intangible=True)
        assert hp_loss == 1

    def test_intangible_with_block(self):
        # Intangible caps to 1 before block
        hp_loss, blk = calculate_incoming_damage(100, 5, intangible=True)
        assert hp_loss == 0  # 1 - 1 blocked (if block >= 1)
        assert blk == 4

    def test_tungsten_rod(self):
        hp_loss, blk = calculate_incoming_damage(10, 5, tungsten_rod=True)
        assert hp_loss == 4  # (10 - 5) - 1

    def test_tungsten_rod_no_overkill(self):
        # Tungsten Rod can reduce to 0 but not below
        hp_loss, blk = calculate_incoming_damage(6, 5, tungsten_rod=True)
        assert hp_loss == 0  # (6 - 5) - 1 = 0

    def test_tungsten_rod_fully_blocked(self):
        hp_loss, blk = calculate_incoming_damage(5, 10, tungsten_rod=True)
        assert hp_loss == 0  # No HP loss, Tungsten Rod doesn't apply

    # --- Torii tests ---
    # NOTE: In Java, Torii applies AFTER block removal (onAttacked hook).
    # The current Python applies Torii BEFORE block, which is a known bug.
    # These tests document the CORRECT Java behavior.

    def test_torii_no_block(self):
        """4 damage, 0 block, Torii -> 1 HP loss (Java: onAttacked, 4 > 1 && 4 <= 5)"""
        hp_loss, blk = calculate_incoming_damage(4, 0, torii=True)
        assert hp_loss == 1

    def test_torii_above_threshold(self):
        """10 damage, 0 block, Torii -> 10 HP loss (above 5 threshold)"""
        hp_loss, blk = calculate_incoming_damage(10, 0, torii=True)
        assert hp_loss == 10

    def test_torii_at_1(self):
        """1 damage, 0 block, Torii -> 1 HP loss (not > 1, Torii doesn't trigger)"""
        hp_loss, blk = calculate_incoming_damage(1, 0, torii=True)
        assert hp_loss == 1

    def test_torii_at_threshold_boundary(self):
        """5 damage, 0 block, Torii -> 1 (exactly at upper bound)"""
        hp_loss, blk = calculate_incoming_damage(5, 0, torii=True)
        assert hp_loss == 1

    def test_torii_at_2(self):
        """2 damage, 0 block, Torii -> 1"""
        hp_loss, blk = calculate_incoming_damage(2, 0, torii=True)
        assert hp_loss == 1

    def test_torii_with_partial_block_java_behavior(self):
        """
        Java behavior: 4 damage, 3 block, Torii
        1. decrementBlock: 4 - 3 = 1 unblocked
        2. onAttacked(Torii): 1 is NOT > 1, so Torii doesn't trigger
        3. HP loss = 1

        Python (buggy): Torii triggers on 4 (2<=4<=5) -> 1, then 1 - 1 block = 0
        """
        hp_loss, blk = calculate_incoming_damage(4, 3, torii=True)
        assert hp_loss == 1, f"Java: 1 HP loss (Torii after block), Python gives {hp_loss}"

    def test_torii_block_makes_it_trigger(self):
        """
        Java behavior: 8 damage, 5 block, Torii
        1. decrementBlock: 8 - 5 = 3 unblocked
        2. onAttacked(Torii): 3 > 1 && 3 <= 5 -> reduce to 1
        3. HP loss = 1

        Python (buggy): Torii doesn't trigger on 8 (> 5), so hp_loss = 8 - 5 = 3
        """
        hp_loss, blk = calculate_incoming_damage(8, 5, torii=True)
        assert hp_loss == 1, f"Java: 1 HP loss (Torii after block), Python gives {hp_loss}"

    # --- Torii + Tungsten Rod ---

    def test_torii_plus_tungsten_no_block(self):
        """4 damage, 0 block, Torii + Tungsten Rod -> 0 HP (Torii: 1, Rod: -1)"""
        hp_loss, blk = calculate_incoming_damage(4, 0, torii=True, tungsten_rod=True)
        assert hp_loss == 0

    # --- Intangible + Tungsten Rod ---

    def test_intangible_plus_tungsten(self):
        """100 damage, 0 block, Intangible + Tungsten Rod -> 0 (1 - 1)"""
        hp_loss, blk = calculate_incoming_damage(100, 0, intangible=True, tungsten_rod=True)
        assert hp_loss == 0

    # --- Wrath + Vuln combo ---

    def test_wrath_vuln_incoming(self):
        """10 damage in Wrath while Vuln: 10 * 2.0 * 1.5 = 30"""
        hp_loss, blk = calculate_incoming_damage(10, 0, is_wrath=True, vuln=True)
        assert hp_loss == 30


# =========================================================================
# HP Loss (poison, self-damage, etc.)
# =========================================================================


class TestHPLoss:
    """
    HP_LOSS type bypasses block.
    Torii does NOT apply (explicitly excluded in Java).
    Intangible DOES apply.
    Tungsten Rod DOES apply (onLoseHpLast).
    """

    def test_basic_hp_loss(self):
        assert apply_hp_loss(5) == 5

    def test_intangible_caps_hp_loss(self):
        assert apply_hp_loss(10, intangible=True) == 1

    def test_tungsten_rod_reduces_hp_loss(self):
        assert apply_hp_loss(5, tungsten_rod=True) == 4

    def test_intangible_plus_tungsten_rod(self):
        # Intangible: 10 -> 1, Tungsten Rod: 1 -> 0
        assert apply_hp_loss(10, intangible=True, tungsten_rod=True) == 0

    def test_tungsten_rod_at_1(self):
        assert apply_hp_loss(1, tungsten_rod=True) == 0

    def test_tungsten_rod_at_0(self):
        assert apply_hp_loss(0, tungsten_rod=True) == 0

    def test_intangible_at_1(self):
        assert apply_hp_loss(1, intangible=True) == 1

    def test_intangible_at_0(self):
        assert apply_hp_loss(0, intangible=True) == 0


# =========================================================================
# Multi-hit Damage
# =========================================================================


class TestMultiHitDamage:
    """
    Multi-hit cards (e.g., Flurry of Blows, Ragnarok) deal damage per hit.
    Each hit is a separate damage() call in Java.
    Intangible caps EACH hit to 1, so 5 hits = 5 damage vs Intangible.
    """

    def test_multi_hit_vs_intangible(self):
        """5 hits of 10 damage vs Intangible = 5 total (1 per hit)"""
        total = sum(calculate_damage(10, intangible=True) for _ in range(5))
        assert total == 5

    def test_multi_hit_vs_flight(self):
        """3 hits of 10 vs Flight = 15 (5 per hit), and Flight loses 3 stacks"""
        per_hit = calculate_damage(10, flight=True)
        assert per_hit == 5
        assert per_hit * 3 == 15

    def test_multi_hit_torii(self):
        """3 hits of 4 damage, Torii: each hit reduced to 1 -> 3 total"""
        total = 0
        for _ in range(3):
            hp, _ = calculate_incoming_damage(4, 0, torii=True)
            total += hp
        assert total == 3

    def test_multi_hit_tungsten_rod(self):
        """3 hits of 4 damage, Tungsten Rod: each hit -1 -> 3*3 = 9 total"""
        total = 0
        for _ in range(3):
            hp, _ = calculate_incoming_damage(4, 0, tungsten_rod=True)
            total += hp
        assert total == 9


# =========================================================================
# Edge cases
# =========================================================================


class TestEdgeCases:
    def test_zero_base_damage(self):
        assert calculate_damage(0) == 0

    def test_zero_base_with_strength(self):
        assert calculate_damage(0, strength=5) == 5

    def test_negative_base_clamps(self):
        # Negative strength can make it negative, but min 0
        assert calculate_damage(0, strength=-5) == 0

    def test_zero_block(self):
        assert calculate_block(0) == 0

    def test_zero_block_with_dex(self):
        assert calculate_block(0, dexterity=3) == 3

    def test_double_damage_power(self):
        assert calculate_damage(6, double_damage=True) == 12

    def test_pen_nib_plus_double_damage(self):
        # Both *2.0 -> 6 * 2 * 2 = 24
        assert calculate_damage(6, pen_nib=True, double_damage=True) == 24

    def test_all_modifiers_combined(self):
        """Stress test with many modifiers at once."""
        result = calculate_damage(
            base=6,
            strength=3,
            vigor=2,
            weak=True,
            pen_nib=True,
            stance_mult=WRATH_MULT,
            vuln=True,
        )
        # (6+3+2) = 11, *2(pen) = 22, *0.75(weak) = 16.5, *2(wrath) = 33.0, *1.5(vuln) = 49.5 -> 49
        assert result == 49
