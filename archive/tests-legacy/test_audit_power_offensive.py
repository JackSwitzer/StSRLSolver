"""
Audit tests: Offensive power triggers against decompiled Java behavior.

Verifies damage calculation chain, per-power math, and edge cases
match AbstractCard.calculateCardDamage from decompiled source.
"""

import pytest
from packages.engine.calc.damage import (
    calculate_damage,
    calculate_block,
    calculate_incoming_damage,
    apply_hp_loss,
    WEAK_MULT,
    VULN_MULT,
    WRATH_MULT,
    DIVINITY_MULT,
)
from packages.engine.content.powers import (
    PowerManager,
    create_strength,
    create_vigor,
    create_power,
    create_weak,
    create_vulnerable,
)
from packages.engine.content.stances import (
    StanceManager,
    StanceID,
    STANCES,
)


# =============================================================================
# DAMAGE CHAIN ORDER (Java: AbstractCard.calculateCardDamage)
# =============================================================================

class TestDamageChainOrder:
    """Verify the full damage calculation chain matches Java ordering."""

    def test_base_damage_only(self):
        assert calculate_damage(6) == 6

    def test_strength_adds_flat(self):
        # Java: StrengthPower.atDamageGive -> damage + amount
        assert calculate_damage(6, strength=3) == 9

    def test_negative_strength(self):
        assert calculate_damage(6, strength=-2) == 4

    def test_vigor_adds_flat(self):
        # Java: VigorPower.atDamageGive -> damage + amount
        assert calculate_damage(6, vigor=5) == 11

    def test_strength_and_vigor_stack_additively(self):
        # Both are flat adds in atDamageGive, applied before multipliers
        assert calculate_damage(6, strength=3, vigor=5) == 14

    def test_pen_nib_doubles(self):
        # Java: PenNibPower.atDamageGive -> damage * 2.0 (NORMAL)
        assert calculate_damage(6, pen_nib=True) == 12

    def test_double_damage_doubles(self):
        # Java: DoubleDamagePower.atDamageGive -> damage * 2.0 (NORMAL)
        assert calculate_damage(6, double_damage=True) == 12

    def test_pen_nib_and_double_damage_stack_multiplicatively(self):
        # Both are *2.0, applied sequentially
        assert calculate_damage(6, pen_nib=True, double_damage=True) == 24

    def test_weak_reduces_25_percent(self):
        # Java: WeakPower.atDamageGive -> damage * 0.75 (NORMAL)
        assert calculate_damage(10, weak=True) == 7  # 10 * 0.75 = 7.5 -> 7

    def test_weak_with_paper_crane(self):
        # Java: WeakPower -> if !owner.isPlayer && Paper Crane -> *0.60
        assert calculate_damage(10, weak=True, weak_paper_crane=True) == 6  # 10 * 0.60

    def test_strength_then_weak(self):
        # Flat adds first, then Weak multiplier
        # (6 + 3) * 0.75 = 6.75 -> 6
        assert calculate_damage(6, strength=3, weak=True) == 6

    def test_pen_nib_then_weak(self):
        # PenNib (pri 6) before Weak (pri 99)
        # 6 * 2.0 * 0.75 = 9.0
        assert calculate_damage(6, pen_nib=True, weak=True) == 9

    def test_wrath_stance_doubles(self):
        # Java: WrathStance.atDamageGive -> damage * 2.0 (NORMAL)
        assert calculate_damage(6, stance_mult=WRATH_MULT) == 12

    def test_divinity_stance_triples(self):
        # Java: DivinityStance.atDamageGive -> damage * 3.0 (NORMAL)
        assert calculate_damage(6, stance_mult=DIVINITY_MULT) == 18

    def test_vulnerable_increases_50_percent(self):
        # Java: VulnerablePower.atDamageReceive -> damage * 1.5 (NORMAL)
        assert calculate_damage(10, vuln=True) == 15

    def test_vulnerable_paper_frog(self):
        # Java: VulnerablePower -> if !owner.isPlayer && Paper Frog -> *1.75
        assert calculate_damage(10, vuln=True, vuln_paper_frog=True) == 17  # 10 * 1.75

    def test_flight_halves(self):
        assert calculate_damage(10, flight=True) == 5

    def test_intangible_caps_at_1(self):
        # Java: IntangiblePower.atDamageFinalReceive -> if > 1, return 1
        assert calculate_damage(100, intangible=True) == 1

    def test_intangible_does_not_increase_low_damage(self):
        assert calculate_damage(0, intangible=True) == 0
        assert calculate_damage(1, intangible=True) == 1

    def test_minimum_damage_zero(self):
        assert calculate_damage(0, strength=-5) == 0


# =============================================================================
# FULL CHAIN COMBOS (verify multiplicative ordering)
# =============================================================================

class TestFullChainCombos:
    """Test realistic multi-modifier scenarios."""

    def test_str_wrath_vuln(self):
        # (6 + 3) * 2.0 * 1.5 = 27
        assert calculate_damage(6, strength=3, stance_mult=WRATH_MULT, vuln=True) == 27

    def test_str_vigor_pennib_wrath_vuln(self):
        # (6 + 3 + 5) * 2.0 * 2.0 * 1.5 = 84
        assert calculate_damage(
            6, strength=3, vigor=5, pen_nib=True, stance_mult=WRATH_MULT, vuln=True
        ) == 84

    def test_weak_wrath_vuln(self):
        # 10 * 0.75 * 2.0 * 1.5 = 22.5 -> 22
        assert calculate_damage(10, weak=True, stance_mult=WRATH_MULT, vuln=True) == 22

    def test_divinity_vuln(self):
        # 6 * 3.0 * 1.5 = 27
        assert calculate_damage(6, stance_mult=DIVINITY_MULT, vuln=True) == 27

    def test_everything_vs_intangible(self):
        # Huge damage, but intangible caps at 1
        assert calculate_damage(
            50, strength=10, vigor=5, pen_nib=True, double_damage=True,
            stance_mult=WRATH_MULT, vuln=True, intangible=True
        ) == 1

    def test_flight_and_vuln(self):
        # 10 * 1.5 * 0.5 = 7.5 -> 7
        assert calculate_damage(10, vuln=True, flight=True) == 7


# =============================================================================
# STANCE MECHANICS
# =============================================================================

class TestStanceMechanics:
    """Verify stance damage modifiers and transitions."""

    def test_wrath_damage_give(self):
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)
        assert sm.at_damage_give(10.0) == 20.0

    def test_wrath_damage_receive(self):
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)
        assert sm.at_damage_receive(10.0) == 20.0

    def test_divinity_damage_give(self):
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)
        assert sm.at_damage_give(10.0) == 30.0

    def test_divinity_no_damage_receive_increase(self):
        # Java DivinityStance has NO atDamageReceive override -> 1.0x
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)
        assert sm.at_damage_receive(10.0) == 10.0

    def test_calm_no_damage_modifier(self):
        sm = StanceManager()
        sm.change_stance(StanceID.CALM)
        assert sm.at_damage_give(10.0) == 10.0
        assert sm.at_damage_receive(10.0) == 10.0

    def test_neutral_no_damage_modifier(self):
        sm = StanceManager()
        assert sm.at_damage_give(10.0) == 10.0

    def test_non_normal_damage_unaffected_wrath(self):
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)
        assert sm.at_damage_give(10.0, "HP_LOSS") == 10.0
        assert sm.at_damage_give(10.0, "THORNS") == 10.0

    def test_divinity_exit_timing_matches_java(self):
        """
        Java: DivinityStance.atStartOfTurn() exits to Neutral.
        Python now matches: on_turn_start() exits Divinity.
        """
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)
        assert sm.is_in_divinity()

        # Divinity persists through end of turn
        sm.on_turn_end()
        assert sm.current == StanceID.DIVINITY

        # Exits at start of next turn (matches Java)
        result = sm.on_turn_start()
        assert sm.current == StanceID.NEUTRAL
        assert result.get("divinity_ended") is True

    def test_divinity_energy_on_enter(self):
        sm = StanceManager()
        result = sm.change_stance(StanceID.DIVINITY)
        assert result["energy_gained"] == 3

    def test_calm_energy_on_exit(self):
        sm = StanceManager()
        sm.change_stance(StanceID.CALM)
        result = sm.exit_stance()
        assert result["energy_gained"] == 2

    def test_calm_energy_violet_lotus(self):
        sm = StanceManager(has_violet_lotus=True)
        sm.change_stance(StanceID.CALM)
        result = sm.exit_stance()
        assert result["energy_gained"] == 3


# =============================================================================
# POWER MANAGER DAMAGE CALCULATION
# =============================================================================

class TestPowerManagerDamage:
    """Test PowerManager.calculate_damage_dealt matches expected behavior."""

    def test_strength_in_power_manager(self):
        pm = PowerManager()
        pm.add_power(create_strength(3))
        assert pm.calculate_damage_dealt(6) == 9.0

    def test_vigor_in_power_manager(self):
        pm = PowerManager()
        pm.add_power(create_power("Vigor", 5))
        assert pm.calculate_damage_dealt(6) == 11.0

    def test_pen_nib_in_power_manager(self):
        pm = PowerManager()
        pm.add_power(create_power("Pen Nib", 1))
        assert pm.calculate_damage_dealt(6) == 12.0

    def test_double_damage_in_power_manager(self):
        pm = PowerManager()
        pm.add_power(create_power("Double Damage", 1))
        assert pm.calculate_damage_dealt(6) == 12.0

    def test_weak_in_power_manager(self):
        pm = PowerManager()
        pm.add_power(create_weak(1))
        # 10 * 0.75 = 7.5
        assert pm.calculate_damage_dealt(10) == 7.5

    def test_strength_vigor_weak_combined(self):
        pm = PowerManager()
        pm.add_power(create_strength(3))
        pm.add_power(create_power("Vigor", 5))
        pm.add_power(create_weak(1))
        # (6 + 3 + 5) * 0.75 = 10.5
        assert pm.calculate_damage_dealt(6) == 10.5


# =============================================================================
# POWER MANAGER DEFENDER CALCULATION
# =============================================================================

class TestPowerManagerDefender:
    """Test PowerManager.calculate_damage_received."""

    def test_vulnerable(self):
        pm = PowerManager()
        pm.add_power(create_vulnerable(1))
        assert pm.calculate_damage_received(10.0) == 15

    def test_vulnerable_odd_mushroom(self):
        pm = PowerManager()
        pm.has_odd_mushroom = True
        pm.add_power(create_vulnerable(1))
        assert pm.calculate_damage_received(10.0, is_player=True) == 12  # 10 * 1.25

    def test_vulnerable_paper_frog(self):
        pm = PowerManager()
        pm.has_paper_frog = True
        pm.add_power(create_vulnerable(1))
        assert pm.calculate_damage_received(10.0, is_player=False) == 17  # 10 * 1.75

    def test_intangible_caps(self):
        pm = PowerManager()
        pm.add_power(create_power("Intangible", 1))
        assert pm.calculate_damage_received(100.0) == 1

    def test_intangible_low_damage(self):
        pm = PowerManager()
        pm.add_power(create_power("Intangible", 1))
        assert pm.calculate_damage_received(1.0) == 1
        assert pm.calculate_damage_received(0.0) == 0


# =============================================================================
# INCOMING DAMAGE (Wrath receiving)
# =============================================================================

class TestIncomingDamage:
    """Test calculate_incoming_damage for Wrath receiving 2x."""

    def test_wrath_doubles_incoming(self):
        hp_loss, blk = calculate_incoming_damage(10, 0, is_wrath=True)
        assert hp_loss == 20

    def test_wrath_incoming_with_block(self):
        hp_loss, blk = calculate_incoming_damage(10, 5, is_wrath=True)
        # 10 * 2 = 20, 20 - 5 = 15
        assert hp_loss == 15
        assert blk == 0

    def test_wrath_vuln_incoming(self):
        hp_loss, blk = calculate_incoming_damage(10, 0, is_wrath=True, vuln=True)
        # 10 * 2.0 * 1.5 = 30
        assert hp_loss == 30

    def test_divinity_does_not_increase_incoming(self):
        # Divinity only increases outgoing, not incoming
        # calculate_incoming_damage has no divinity param -- correct
        hp_loss, blk = calculate_incoming_damage(10, 0)
        assert hp_loss == 10


# =============================================================================
# ACCURACY (Shiv base damage modification)
# =============================================================================

class TestAccuracy:
    """
    Java AccuracyPower modifies Shiv baseDamage directly:
      - Unupgraded: baseDamage = 4 + amount
      - Upgraded: baseDamage = 6 + amount

    It does NOT use atDamageGive. It modifies the card's baseDamage.
    """

    def test_accuracy_shiv_unupgraded(self):
        # Shiv base = 4, with Accuracy(4) -> base = 8
        # Then normal damage calc applies
        accuracy_amount = 4
        shiv_base = 4 + accuracy_amount
        assert calculate_damage(shiv_base) == 8

    def test_accuracy_shiv_upgraded(self):
        # Shiv+ base = 6, with Accuracy(4) -> base = 10
        accuracy_amount = 4
        shiv_base = 6 + accuracy_amount
        assert calculate_damage(shiv_base) == 10

    def test_accuracy_stacks_with_strength(self):
        # Accuracy modifies baseDamage, Strength added on top
        accuracy_amount = 4
        shiv_base = 4 + accuracy_amount  # = 8
        assert calculate_damage(shiv_base, strength=3) == 11


# =============================================================================
# ENVENOM (on-hit poison application)
# =============================================================================

class TestEnvenom:
    """
    Java EnvenomPower.onAttack:
      - damageAmount > 0 && target != owner && type == NORMAL
      - Apply amount Poison to target
    """

    def test_envenom_data_registered(self):
        from packages.engine.content.powers import POWER_DATA
        assert "Envenom" in POWER_DATA
        data = POWER_DATA["Envenom"]
        assert "on_attack" in data["mechanics"]
        assert "Poison" in data["mechanics"]["on_attack"]

    def test_envenom_requires_unblocked_normal_damage(self):
        """Envenom only triggers on NORMAL damage > 0."""
        from packages.engine.content.powers import POWER_DATA
        mechanic = POWER_DATA["Envenom"]["mechanics"]["on_attack"]
        assert "damage > 0" in mechanic
        assert "NORMAL" in mechanic


# =============================================================================
# EDGE CASES
# =============================================================================

class TestEdgeCases:
    """Edge cases from Java behavior."""

    def test_zero_base_with_strength(self):
        # 0 + 3 = 3
        assert calculate_damage(0, strength=3) == 3

    def test_negative_total_floors_to_zero(self):
        # 1 + (-5) = -4 -> 0
        assert calculate_damage(1, strength=-5) == 0

    def test_weak_on_zero_damage(self):
        # 0 * 0.75 = 0
        assert calculate_damage(0, weak=True) == 0

    def test_pen_nib_on_zero_damage(self):
        # 0 * 2 = 0
        assert calculate_damage(0, pen_nib=True) == 0

    def test_intangible_on_zero(self):
        assert calculate_damage(0, intangible=True) == 0

    def test_rounding_weak_str(self):
        # (6 + 3) * 0.75 = 6.75 -> floor -> 6
        assert calculate_damage(6, strength=3, weak=True) == 6

    def test_rounding_vuln(self):
        # 7 * 1.5 = 10.5 -> floor -> 10
        assert calculate_damage(7, vuln=True) == 10

    def test_hp_loss_ignores_block(self):
        assert apply_hp_loss(5) == 5

    def test_hp_loss_intangible(self):
        assert apply_hp_loss(10, intangible=True) == 1

    def test_hp_loss_tungsten_rod(self):
        assert apply_hp_loss(5, tungsten_rod=True) == 4

    def test_hp_loss_intangible_plus_tungsten(self):
        # Intangible caps at 1, then Tungsten -1 = 0
        assert apply_hp_loss(10, intangible=True, tungsten_rod=True) == 0
