"""
Comprehensive Power/Buff/Debuff Tests

Tests all power mechanics including:
1. Stacking behavior (Strength adds, Vulnerable has duration)
2. Duration countdown (end of turn vs start of turn)
3. Damage modification powers (Strength, Weak, Vulnerable, etc.)
4. Block modification powers (Dexterity, Frail)
5. Energy powers (Energized, Berserk, etc.)
6. Card draw powers (Draw Down, Brutality, etc.)
7. Triggered powers (Thorns, Flame Barrier, Mental Fortress, etc.)
8. Watcher-specific powers (Foresight, Study, Establishment, etc.)
9. Stance-related powers (Rushdown, Like Water, etc.)
10. Debuff immunity interactions
11. Power removal effects
12. Negative strength/dexterity handling
13. Power caps and limits
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.content.powers import (
    Power,
    PowerType,
    DamageType,
    PowerManager,
    POWER_DATA,
    # Constants
    WEAK_MULTIPLIER,
    WEAK_MULTIPLIER_PAPER_CRANE,
    VULNERABLE_MULTIPLIER,
    VULNERABLE_MULTIPLIER_ODD_MUSHROOM,
    VULNERABLE_MULTIPLIER_PAPER_FROG,
    FRAIL_MULTIPLIER,
    FLIGHT_MULTIPLIER,
    LOCKON_MULTIPLIER,
    # Factory functions
    create_power,
    create_strength,
    create_dexterity,
    create_weak,
    create_vulnerable,
    create_frail,
    create_poison,
    create_artifact,
    create_intangible,
    create_vigor,
    create_mantra,
)

from packages.engine.content.stances import StanceManager, StanceID


# =============================================================================
# SECTION 1: POWER STACKING BEHAVIOR
# =============================================================================

class TestPowerStacking:
    """Test power stacking mechanics."""

    def test_strength_stacks_additively(self):
        """Strength stacks: 3 + 2 = 5."""
        pm = PowerManager()
        pm.add_power(create_strength(3))
        pm.add_power(create_strength(2))
        assert pm.get_strength() == 5

    def test_vulnerability_duration_stacks(self):
        """Vulnerable duration stacks: 2 turns + 1 turn = 3 turns."""
        pm = PowerManager()
        pm.add_power(create_vulnerable(2))
        pm.add_power(create_vulnerable(1))
        assert pm.get_amount("Vulnerable") == 3

    def test_weak_duration_stacks(self):
        """Weak duration stacks: 2 turns + 2 turns = 4 turns."""
        pm = PowerManager()
        pm.add_power(create_weak(2))
        pm.add_power(create_weak(2))
        assert pm.get_amount("Weakened") == 4

    def test_frail_duration_stacks(self):
        """Frail duration stacks."""
        pm = PowerManager()
        pm.add_power(create_frail(1))
        pm.add_power(create_frail(2))
        assert pm.get_amount("Frail") == 3

    def test_artifact_stacks_additively(self):
        """Artifact charges stack: 2 + 1 = 3."""
        pm = PowerManager()
        pm.add_power(create_artifact(2))
        pm.add_power(create_artifact(1))
        assert pm.get_amount("Artifact") == 3

    def test_intangible_duration_stacks(self):
        """Intangible duration stacks."""
        pm = PowerManager()
        pm.add_power(create_intangible(1))
        pm.add_power(create_intangible(2))
        assert pm.get_amount("Intangible") == 3

    def test_vigor_stacks_additively(self):
        """Vigor stacks: 5 + 3 = 8."""
        pm = PowerManager()
        pm.add_power(create_vigor(5))
        pm.add_power(create_vigor(3))
        assert pm.get_amount("Vigor") == 8

    def test_poison_stacks_additively(self):
        """Poison stacks: 3 + 5 = 8."""
        pm = PowerManager()
        pm.add_power(create_poison(3))
        pm.add_power(create_poison(5))
        assert pm.get_amount("Poison") == 8

    def test_power_cap_at_999(self):
        """Powers cap at 999 by default."""
        power = create_strength(500)
        power.stack(600)
        assert power.amount == 999

    def test_poison_cap_at_9999(self):
        """Poison has higher cap (9999)."""
        poison = create_poison(5000)
        poison.stack(5000)
        assert poison.amount == 9999

    def test_non_stacking_power(self):
        """Some powers don't stack (e.g., Barricade)."""
        pm = PowerManager()
        barricade = create_power("Barricade", 1)
        pm.add_power(barricade)
        # Adding again shouldn't change amount
        barricade2 = create_power("Barricade", 1)
        pm.add_power(barricade2)
        assert pm.get_amount("Barricade") == 1


# =============================================================================
# SECTION 2: DURATION COUNTDOWN
# =============================================================================

class TestDurationCountdown:
    """Test turn-based power countdown mechanics."""

    def test_vulnerable_decrements_at_end_of_round(self):
        """Vulnerable decrements at end of round."""
        pm = PowerManager()
        pm.add_power(create_vulnerable(2))
        pm.at_end_of_round()
        assert pm.get_amount("Vulnerable") == 1
        pm.at_end_of_round()
        assert not pm.has_power("Vulnerable")

    def test_weak_decrements_at_end_of_round(self):
        """Weak decrements at end of round."""
        pm = PowerManager()
        pm.add_power(create_weak(3))
        pm.at_end_of_round()
        assert pm.get_amount("Weakened") == 2
        pm.at_end_of_round()
        assert pm.get_amount("Weakened") == 1
        pm.at_end_of_round()
        assert not pm.has_power("Weakened")

    def test_frail_decrements_at_end_of_round(self):
        """Frail decrements at end of round."""
        pm = PowerManager()
        pm.add_power(create_frail(2))
        pm.at_end_of_round()
        assert pm.get_amount("Frail") == 1
        pm.at_end_of_round()
        assert not pm.has_power("Frail")

    def test_intangible_decrements_at_end_of_turn(self):
        """Intangible is turn-based, decrements at end of round."""
        pm = PowerManager()
        pm.add_power(create_intangible(2))
        pm.at_end_of_round()
        assert pm.get_amount("Intangible") == 1
        pm.at_end_of_round()
        assert not pm.has_power("Intangible")

    def test_just_applied_skips_first_decrement(self):
        """Monster-applied debuffs skip first decrement (just_applied)."""
        pm = PowerManager()
        # Monster applies Weak to player - just_applied = True
        weak = create_weak(2, is_source_monster=True)
        pm.add_power(weak)
        assert weak.just_applied == True
        # First end of round: skip decrement, just_applied -> False
        pm.at_end_of_round()
        assert pm.get_amount("Weakened") == 2
        # Second end of round: actually decrement
        pm.at_end_of_round()
        assert pm.get_amount("Weakened") == 1

    def test_removed_powers_returned(self):
        """at_end_of_round returns list of removed powers."""
        pm = PowerManager()
        pm.add_power(create_weak(1))
        pm.add_power(create_vulnerable(1))
        removed = pm.at_end_of_round()
        assert "Weakened" in removed
        assert "Vulnerable" in removed

    def test_poison_at_start_of_turn(self):
        """Poison deals damage at start of turn, not end of round."""
        pm = PowerManager()
        pm.add_power(create_poison(5))
        effects = pm.at_start_of_turn()
        assert effects["poison_damage"] == 5
        # Poison amount should still be 5 (damage calc is external)
        assert pm.get_amount("Poison") == 5


# =============================================================================
# SECTION 3: DAMAGE MODIFICATION POWERS
# =============================================================================

class TestDamageModificationPowers:
    """Test powers that modify damage dealt/received."""

    def test_strength_adds_to_damage(self):
        """Strength adds flat damage."""
        pm = PowerManager()
        pm.add_power(create_strength(3))
        damage = pm.calculate_damage_dealt(6)
        assert damage == 9.0

    def test_negative_strength_reduces_damage(self):
        """Negative strength reduces damage."""
        pm = PowerManager()
        pm.add_power(create_strength(-2))
        damage = pm.calculate_damage_dealt(6)
        assert damage == 4.0

    def test_vigor_adds_to_damage(self):
        """Vigor adds flat damage (consumed after attack)."""
        pm = PowerManager()
        pm.add_power(create_vigor(5))
        damage = pm.calculate_damage_dealt(6)
        assert damage == 11.0

    def test_strength_and_vigor_combine(self):
        """Strength and Vigor both add to damage."""
        pm = PowerManager()
        pm.add_power(create_strength(3))
        pm.add_power(create_vigor(2))
        damage = pm.calculate_damage_dealt(6)
        assert damage == 11.0  # 6 + 3 + 2

    def test_weak_reduces_damage_dealt(self):
        """Weak reduces damage dealt by 25%."""
        pm = PowerManager()
        pm.add_power(create_weak(1))
        damage = pm.calculate_damage_dealt(10)
        assert damage == 7.5  # 10 * 0.75

    def test_weak_with_paper_crane(self):
        """Paper Crane makes Weak reduce damage by 40%."""
        pm = PowerManager()
        pm.has_paper_crane = True
        pm.add_power(create_weak(1))
        damage = pm.calculate_damage_dealt(10, target_is_player=True)
        assert damage == 6.0  # 10 * 0.60

    def test_vulnerable_increases_damage_received(self):
        """Vulnerable increases damage received by 50%."""
        pm = PowerManager()
        pm.add_power(create_vulnerable(1))
        damage = pm.calculate_damage_received(10.0)
        assert damage == 15  # 10 * 1.5

    def test_vulnerable_with_odd_mushroom(self):
        """Odd Mushroom reduces Vulnerable to 25% increase."""
        pm = PowerManager()
        pm.has_odd_mushroom = True
        pm.add_power(create_vulnerable(1))
        damage = pm.calculate_damage_received(10.0, is_player=True)
        assert damage == 12  # 10 * 1.25

    def test_vulnerable_with_paper_frog(self):
        """Paper Frog increases Vulnerable to 75% increase."""
        pm = PowerManager()
        pm.has_paper_frog = True
        pm.add_power(create_vulnerable(1))
        damage = pm.calculate_damage_received(10.0, is_player=False)
        assert damage == 17  # 10 * 1.75 = 17.5 -> 17

    def test_pen_nib_doubles_damage(self):
        """Pen Nib doubles damage."""
        pm = PowerManager()
        pm.add_power(create_power("Pen Nib", 1))
        damage = pm.calculate_damage_dealt(6)
        assert damage == 12.0

    def test_double_damage_doubles(self):
        """Double Damage power doubles damage."""
        pm = PowerManager()
        pm.add_power(create_power("Double Damage", 1))
        damage = pm.calculate_damage_dealt(10)
        assert damage == 20.0

    def test_intangible_caps_damage_at_one(self):
        """Intangible caps all damage received at 1."""
        pm = PowerManager()
        pm.add_power(create_intangible(1))
        damage = pm.calculate_damage_received(100.0)
        assert damage == 1

    def test_intangible_allows_one_damage(self):
        """1 damage goes through Intangible."""
        pm = PowerManager()
        pm.add_power(create_intangible(1))
        damage = pm.calculate_damage_received(1.0)
        assert damage == 1

    def test_intangible_with_vulnerable(self):
        """Intangible + Vulnerable: Vulnerable applies first, then capped."""
        pm = PowerManager()
        pm.add_power(create_intangible(1))
        pm.add_power(create_vulnerable(1))
        # 10 * 1.5 = 15, but capped to 1
        damage = pm.calculate_damage_received(10.0)
        assert damage == 1

    def test_flight_reduces_damage(self):
        """Flight (enemy power) reduces damage by 50%."""
        pm = PowerManager()
        pm.add_power(create_power("Flight", 3))
        damage = pm.calculate_damage_received(10.0)
        assert damage == 5  # 10 * 0.5


# =============================================================================
# SECTION 4: BLOCK MODIFICATION POWERS
# =============================================================================

class TestBlockModificationPowers:
    """Test powers that modify block gained."""

    def test_dexterity_adds_to_block(self):
        """Dexterity adds flat block."""
        pm = PowerManager()
        pm.add_power(create_dexterity(2))
        block = pm.calculate_block(5)
        assert block == 7

    def test_negative_dexterity_reduces_block(self):
        """Negative dexterity reduces block."""
        pm = PowerManager()
        pm.add_power(create_dexterity(-2))
        block = pm.calculate_block(5)
        assert block == 3

    def test_negative_dexterity_minimum_zero(self):
        """Block cannot go below 0."""
        pm = PowerManager()
        pm.add_power(create_dexterity(-10))
        block = pm.calculate_block(5)
        assert block == 0

    def test_frail_reduces_block(self):
        """Frail reduces block by 25%."""
        pm = PowerManager()
        pm.add_power(create_frail(1))
        block = pm.calculate_block(8)
        assert block == 6  # 8 * 0.75

    def test_dexterity_before_frail(self):
        """Dexterity adds before Frail multiplies."""
        pm = PowerManager()
        pm.add_power(create_dexterity(2))
        pm.add_power(create_frail(1))
        block = pm.calculate_block(5)
        assert block == 5  # (5 + 2) * 0.75 = 5.25 -> 5

    def test_no_block_power(self):
        """No Block power sets block to 0."""
        pm = PowerManager()
        pm.add_power(create_power("NoBlockPower", 1))
        block = pm.calculate_block(10)
        assert block == 0


# =============================================================================
# SECTION 5: ENERGY POWERS
# =============================================================================

class TestEnergyPowers:
    """Test energy-related powers."""

    def test_energized_data_exists(self):
        """Energized power exists in registry."""
        assert "Energized" in POWER_DATA
        data = POWER_DATA["Energized"]
        assert data["type"] == PowerType.BUFF
        assert "on_energy_recharge" in data["mechanics"]

    def test_berserk_data_exists(self):
        """DevaForm (Berserk) power exists."""
        assert "DevaForm" in POWER_DATA
        data = POWER_DATA["DevaForm"]
        assert data["type"] == PowerType.BUFF

    def test_energized_power_creation(self):
        """Can create Energized power."""
        power = create_power("Energized", 2)
        assert power.id == "Energized"
        assert power.amount == 2
        assert power.power_type == PowerType.BUFF


# =============================================================================
# SECTION 6: CARD DRAW POWERS
# =============================================================================

class TestCardDrawPowers:
    """Test card draw related powers."""

    def test_draw_reduction_data(self):
        """Draw Reduction (Draw Down) power data."""
        assert "Draw Reduction" in POWER_DATA
        data = POWER_DATA["Draw Reduction"]
        assert data["type"] == PowerType.DEBUFF
        assert data["is_turn_based"] == True

    def test_draw_power_data(self):
        """Draw (Draw Up) power data."""
        assert "Draw" in POWER_DATA
        data = POWER_DATA["Draw"]
        assert data["type"] == PowerType.BUFF
        assert data["can_go_negative"] == True

    def test_evolve_power_data(self):
        """Evolve power triggers on Status card draw."""
        assert "Evolve" in POWER_DATA
        data = POWER_DATA["Evolve"]
        assert "on_card_draw" in data["mechanics"]

    def test_dark_embrace_power_data(self):
        """Dark Embrace triggers on exhaust."""
        assert "Dark Embrace" in POWER_DATA
        data = POWER_DATA["Dark Embrace"]
        assert "on_exhaust" in data["mechanics"]


# =============================================================================
# SECTION 7: TRIGGERED POWERS
# =============================================================================

class TestTriggeredPowers:
    """Test powers that trigger on specific events."""

    def test_thorns_data(self):
        """Thorns retaliates when attacked."""
        assert "Thorns" in POWER_DATA
        data = POWER_DATA["Thorns"]
        assert "on_attacked" in data["mechanics"]
        assert data["type"] == PowerType.BUFF

    def test_flame_barrier_data(self):
        """Flame Barrier retaliates and removes at start of turn."""
        assert "Flame Barrier" in POWER_DATA
        data = POWER_DATA["Flame Barrier"]
        assert "on_attacked" in data["mechanics"]
        assert "at_start_of_turn" in data["mechanics"]

    def test_flame_barrier_removal_at_start_of_turn(self):
        """Flame Barrier is removed at start of turn."""
        pm = PowerManager()
        pm.add_power(create_power("Flame Barrier", 4))
        assert pm.has_power("Flame Barrier")
        effects = pm.at_start_of_turn()
        assert "Flame Barrier" in effects["removed_powers"]
        assert not pm.has_power("Flame Barrier")

    def test_metallicize_data(self):
        """Metallicize gains block at end of turn."""
        assert "Metallicize" in POWER_DATA
        data = POWER_DATA["Metallicize"]
        assert "at_end_of_turn_pre_cards" in data["mechanics"]

    def test_plated_armor_data(self):
        """Plated Armor gains block and loses stacks when hit."""
        assert "Plated Armor" in POWER_DATA
        data = POWER_DATA["Plated Armor"]
        assert "at_end_of_turn_pre_cards" in data["mechanics"]
        assert "was_hp_lost" in data["mechanics"]

    def test_after_image_data(self):
        """After Image gains block on card play."""
        assert "After Image" in POWER_DATA
        data = POWER_DATA["After Image"]
        assert "on_use_card" in data["mechanics"]

    def test_juggernaut_data(self):
        """Juggernaut deals damage when gaining block."""
        assert "Juggernaut" in POWER_DATA
        data = POWER_DATA["Juggernaut"]
        assert "on_gained_block" in data["mechanics"]

    def test_rupture_data(self):
        """Rupture gains Strength from self-damage."""
        assert "Rupture" in POWER_DATA
        data = POWER_DATA["Rupture"]
        assert "was_hp_lost" in data["mechanics"]

    def test_buffer_data(self):
        """Buffer prevents next HP loss."""
        assert "Buffer" in POWER_DATA
        data = POWER_DATA["Buffer"]
        assert "on_attacked_to_change_damage" in data["mechanics"]


# =============================================================================
# SECTION 8: WATCHER-SPECIFIC POWERS
# =============================================================================

class TestWatcherPowers:
    """Test Watcher-specific powers."""

    def test_foresight_data(self):
        """Foresight (Wireheading) scries at start of turn."""
        assert "WireheadingPower" in POWER_DATA
        data = POWER_DATA["WireheadingPower"]
        assert data["name"] == "Foresight"
        assert "at_start_of_turn" in data["mechanics"]

    def test_study_data(self):
        """Study shuffles Insight into draw pile."""
        assert "Study" in POWER_DATA
        data = POWER_DATA["Study"]
        assert "at_end_of_turn" in data["mechanics"]

    def test_establishment_data(self):
        """Establishment reduces cost of retained cards."""
        assert "EstablishmentPower" in POWER_DATA
        data = POWER_DATA["EstablishmentPower"]
        assert data["name"] == "Establishment"
        assert "at_end_of_turn" in data["mechanics"]

    def test_devotion_data(self):
        """Devotion gains Mantra each turn."""
        assert "DevotionPower" in POWER_DATA
        data = POWER_DATA["DevotionPower"]
        assert data["name"] == "Devotion"
        assert "at_start_of_turn_post_draw" in data["mechanics"]

    def test_nirvana_data(self):
        """Nirvana gains block when scrying."""
        assert "Nirvana" in POWER_DATA
        data = POWER_DATA["Nirvana"]
        assert "on_scry" in data["mechanics"]

    def test_battle_hymn_data(self):
        """Battle Hymn adds Smites each turn."""
        assert "BattleHymn" in POWER_DATA
        data = POWER_DATA["BattleHymn"]
        assert "at_start_of_turn" in data["mechanics"]

    def test_master_reality_data(self):
        """Master Reality upgrades created cards."""
        assert "MasterRealityPower" in POWER_DATA
        data = POWER_DATA["MasterRealityPower"]
        assert data["stacks"] == False

    def test_mantra_creation(self):
        """Can create Mantra power."""
        power = create_mantra(5)
        assert power.id == "Mantra"
        assert power.amount == 5

    def test_vigor_creation(self):
        """Can create Vigor power."""
        power = create_vigor(8)
        assert power.id == "Vigor"
        assert power.amount == 8


# =============================================================================
# SECTION 9: STANCE-RELATED POWERS
# =============================================================================

class TestStanceRelatedPowers:
    """Test powers that interact with stances."""

    def test_mental_fortress_data(self):
        """Mental Fortress (Controlled) gains block on stance change."""
        assert "Controlled" in POWER_DATA
        data = POWER_DATA["Controlled"]
        assert data["name"] == "Mental Fortress"
        assert "on_change_stance" in data["mechanics"]

    def test_rushdown_data(self):
        """Rushdown (Adaptation) draws when entering Wrath."""
        assert "Adaptation" in POWER_DATA
        data = POWER_DATA["Adaptation"]
        assert data["name"] == "Rushdown"
        assert "on_change_stance" in data["mechanics"]

    def test_like_water_data(self):
        """Like Water gains block at end of turn if in Calm."""
        assert "LikeWaterPower" in POWER_DATA
        data = POWER_DATA["LikeWaterPower"]
        assert data["name"] == "Like Water"
        assert "at_end_of_turn_pre_cards" in data["mechanics"]

    def test_wrath_next_turn_data(self):
        """Wrath Next Turn enters Wrath at start of turn."""
        assert "WrathNextTurnPower" in POWER_DATA
        data = POWER_DATA["WrathNextTurnPower"]
        assert data["stacks"] == False
        assert "at_start_of_turn" in data["mechanics"]

    def test_wrath_next_turn_removal(self):
        """Wrath Next Turn is removed after triggering."""
        pm = PowerManager()
        pm.add_power(create_power("WrathNextTurnPower", 1))
        assert pm.has_power("WrathNextTurnPower")
        effects = pm.at_start_of_turn()
        assert effects["stance_change"] == "Wrath"
        assert not pm.has_power("WrathNextTurnPower")

    def test_cannot_change_stance_data(self):
        """Cannot Change Stance power."""
        assert "CannotChangeStancePower" in POWER_DATA
        data = POWER_DATA["CannotChangeStancePower"]
        assert data["type"] == PowerType.DEBUFF
        assert data["stacks"] == False


# =============================================================================
# SECTION 10: DEBUFF IMMUNITY (ARTIFACT)
# =============================================================================

class TestDebuffImmunity:
    """Test Artifact and debuff blocking."""

    def test_artifact_blocks_debuff(self):
        """Artifact blocks debuff application."""
        pm = PowerManager()
        pm.add_power(create_artifact(2))
        result = pm.add_power(create_weak(1))
        assert result == False  # Blocked
        assert pm.get_amount("Artifact") == 1
        assert not pm.is_weak()

    def test_artifact_consumed_on_block(self):
        """Artifact charge is consumed when blocking."""
        pm = PowerManager()
        pm.add_power(create_artifact(1))
        pm.add_power(create_weak(1))
        assert pm.get_amount("Artifact") == 0 or not pm.has_power("Artifact")

    def test_artifact_removed_at_zero(self):
        """Artifact power removed when charges depleted."""
        pm = PowerManager()
        pm.add_power(create_artifact(1))
        pm.add_power(create_vulnerable(1))
        assert not pm.has_power("Artifact")

    def test_artifact_blocks_multiple_debuffs(self):
        """Multiple Artifact charges block multiple debuffs."""
        pm = PowerManager()
        pm.add_power(create_artifact(3))
        pm.add_power(create_weak(1))
        pm.add_power(create_vulnerable(1))
        pm.add_power(create_frail(1))
        assert not pm.is_weak()
        assert not pm.is_vulnerable()
        assert not pm.is_frail()
        assert pm.get_amount("Artifact") == 0 or not pm.has_power("Artifact")

    def test_artifact_allows_buffs(self):
        """Artifact does not block buff application."""
        pm = PowerManager()
        pm.add_power(create_artifact(1))
        pm.add_power(create_strength(2))
        assert pm.get_strength() == 2
        assert pm.get_amount("Artifact") == 1  # Not consumed

    def test_artifact_blocks_specific_debuffs(self):
        """Artifact blocks various debuff types."""
        pm = PowerManager()
        pm.add_power(create_artifact(1))
        pm.add_power(create_poison(5))  # Poison is a debuff
        assert not pm.has_power("Poison")


# =============================================================================
# SECTION 11: POWER REMOVAL EFFECTS
# =============================================================================

class TestPowerRemoval:
    """Test power removal mechanics."""

    def test_remove_power_by_id(self):
        """Can remove power by ID."""
        pm = PowerManager()
        pm.add_power(create_strength(5))
        removed = pm.remove_power("Strength")
        assert removed is not None
        assert removed.amount == 5
        assert not pm.has_power("Strength")

    def test_remove_nonexistent_power(self):
        """Removing nonexistent power returns None."""
        pm = PowerManager()
        removed = pm.remove_power("Strength")
        assert removed is None

    def test_reduce_power_partial(self):
        """Can partially reduce power amount."""
        pm = PowerManager()
        pm.add_power(create_strength(5))
        removed = pm.reduce_power("Strength", 2)
        assert removed == False
        assert pm.get_strength() == 3

    def test_reduce_power_to_zero(self):
        """Reducing power to 0 removes it (for non-negative-capable)."""
        pm = PowerManager()
        pm.add_power(create_weak(2))
        removed = pm.reduce_power("Weakened", 2)
        assert removed == True
        assert not pm.has_power("Weakened")

    def test_reduce_negative_capable_below_zero(self):
        """Powers that can go negative don't get removed at 0."""
        pm = PowerManager()
        pm.add_power(create_strength(2))
        removed = pm.reduce_power("Strength", 5)
        # Strength can go negative, so not removed
        assert removed == False
        assert pm.get_strength() == -3


# =============================================================================
# SECTION 12: NEGATIVE STRENGTH/DEXTERITY
# =============================================================================

class TestNegativePowers:
    """Test negative power amounts."""

    def test_negative_strength_creation(self):
        """Can create negative Strength directly."""
        power = create_strength(-3)
        assert power.amount == -3
        assert power.can_go_negative == True

    def test_negative_dexterity_creation(self):
        """Can create negative Dexterity directly."""
        power = create_dexterity(-2)
        assert power.amount == -2
        assert power.can_go_negative == True

    def test_strength_to_negative(self):
        """Strength can be reduced to negative."""
        pm = PowerManager()
        pm.add_power(create_strength(2))
        pm.add_power(create_strength(-5))
        assert pm.get_strength() == -3

    def test_dexterity_to_negative(self):
        """Dexterity can be reduced to negative."""
        pm = PowerManager()
        pm.add_power(create_dexterity(1))
        pm.add_power(create_dexterity(-4))
        assert pm.get_dexterity() == -3

    def test_negative_strength_damage_calc(self):
        """Negative Strength reduces damage."""
        pm = PowerManager()
        pm.add_power(create_strength(-3))
        damage = pm.calculate_damage_dealt(10)
        assert damage == 7.0  # 10 + (-3)

    def test_negative_dexterity_block_calc(self):
        """Negative Dexterity reduces block."""
        pm = PowerManager()
        pm.add_power(create_dexterity(-3))
        block = pm.calculate_block(8)
        assert block == 5  # 8 + (-3)

    def test_focus_can_go_negative(self):
        """Focus can go negative."""
        focus = create_power("Focus", -2)
        assert focus.can_go_negative == True
        assert focus.amount == -2


# =============================================================================
# SECTION 13: POWER CAPS AND LIMITS
# =============================================================================

class TestPowerCapsAndLimits:
    """Test power amount caps and limits."""

    def test_default_max_999(self):
        """Default max amount is 999 (enforced during stacking)."""
        power = create_strength(500)
        power.stack(600)  # Would be 1100, but capped at 999
        assert power.amount == 999

    def test_default_min_negative_999(self):
        """Default min amount is -999 (enforced during stacking)."""
        power = create_strength(-500)
        power.stack(-600)  # Would be -1100, but floored at -999
        assert power.amount == -999

    def test_stacking_respects_cap(self):
        """Stacking respects max cap."""
        power = create_strength(500)
        power.stack(600)
        assert power.amount == 999

    def test_poison_higher_cap(self):
        """Poison has 9999 cap (enforced during stacking)."""
        power = create_poison(5000)
        power.stack(5000)  # Would be 10000, but capped at 9999
        assert power.amount == 9999

    def test_zero_amount_removal(self):
        """Powers at 0 that can't go negative are removed."""
        pm = PowerManager()
        pm.add_power(create_weak(1))
        pm.reduce_power("Weakened", 1)
        assert not pm.has_power("Weakened")

    def test_zero_strength_not_removed(self):
        """Strength at 0 is removed (stacks to 0)."""
        pm = PowerManager()
        pm.add_power(create_strength(2))
        pm.add_power(create_strength(-2))
        # When stacking brings it to 0, it should be removed
        assert pm.get_strength() == 0


# =============================================================================
# SECTION 14: BOSS/ENEMY POWER DATA
# =============================================================================

class TestBossEnemyPowers:
    """Test boss and enemy-specific powers."""

    def test_beat_of_death_data(self):
        """Beat of Death (Heart) damages player per card."""
        assert "BeatOfDeath" in POWER_DATA
        data = POWER_DATA["BeatOfDeath"]
        assert "on_after_use_card" in data["mechanics"]

    def test_curiosity_data(self):
        """Curiosity (Awakened One) gains Strength on Power play."""
        assert "Curiosity" in POWER_DATA
        data = POWER_DATA["Curiosity"]
        assert "on_use_card" in data["mechanics"]

    def test_time_warp_data(self):
        """Time Warp (Time Eater) counts cards."""
        assert "Time Warp" in POWER_DATA
        data = POWER_DATA["Time Warp"]
        assert "on_after_use_card" in data["mechanics"]

    def test_invincible_data(self):
        """Invincible (Champ, Heart) caps damage per turn."""
        assert "Invincible" in POWER_DATA
        data = POWER_DATA["Invincible"]
        assert data["priority"] == 99  # High priority

    def test_angry_data(self):
        """Angry (Gremlin Nob) gains Strength when hit."""
        assert "Angry" in POWER_DATA
        data = POWER_DATA["Angry"]
        assert "on_attacked" in data["mechanics"]

    def test_ritual_data(self):
        """Ritual (Cultist) gains Strength each round."""
        assert "GrowthPower" in POWER_DATA
        data = POWER_DATA["GrowthPower"]
        assert data["name"] == "Ritual"
        assert "at_end_of_round" in data["mechanics"]

    def test_split_data(self):
        """Split (Slimes) doesn't stack."""
        assert "Split" in POWER_DATA
        data = POWER_DATA["Split"]
        assert data["stacks"] == False


# =============================================================================
# SECTION 15: POWER TYPE VERIFICATION
# =============================================================================

class TestPowerTypes:
    """Test power type classification."""

    def test_debuff_types(self):
        """Common debuffs are classified correctly."""
        debuffs = ["Weakened", "Vulnerable", "Frail", "Poison", "Slow", "Constricted"]
        for debuff_id in debuffs:
            if debuff_id in POWER_DATA:
                assert POWER_DATA[debuff_id]["type"] == PowerType.DEBUFF, f"{debuff_id} should be DEBUFF"

    def test_buff_types(self):
        """Common buffs are classified correctly."""
        buffs = ["Strength", "Dexterity", "Artifact", "Intangible", "Thorns", "Metallicize"]
        for buff_id in buffs:
            if buff_id in POWER_DATA:
                assert POWER_DATA[buff_id]["type"] == PowerType.BUFF, f"{buff_id} should be BUFF"

    def test_strength_dynamic_type(self):
        """Strength notes it changes to DEBUFF if negative."""
        data = POWER_DATA["Strength"]
        assert "Can go negative" in data["notes"]


# =============================================================================
# SECTION 16: CONVENIENCE METHOD TESTS
# =============================================================================

class TestPowerManagerConvenience:
    """Test PowerManager convenience methods."""

    def test_is_weak(self):
        """is_weak() convenience method."""
        pm = PowerManager()
        assert not pm.is_weak()
        pm.add_power(create_weak(1))
        assert pm.is_weak()

    def test_is_vulnerable(self):
        """is_vulnerable() convenience method."""
        pm = PowerManager()
        assert not pm.is_vulnerable()
        pm.add_power(create_vulnerable(1))
        assert pm.is_vulnerable()

    def test_is_frail(self):
        """is_frail() convenience method."""
        pm = PowerManager()
        assert not pm.is_frail()
        pm.add_power(create_frail(1))
        assert pm.is_frail()

    def test_is_intangible(self):
        """is_intangible() convenience method."""
        pm = PowerManager()
        assert not pm.is_intangible()
        pm.add_power(create_intangible(1))
        assert pm.is_intangible()

    def test_has_barricade(self):
        """has_barricade() checks both Barricade and Blur."""
        pm = PowerManager()
        assert not pm.has_barricade()
        pm.add_power(create_power("Blur", 1))
        assert pm.has_barricade()

        pm2 = PowerManager()
        pm2.add_power(create_power("Barricade", 1))
        assert pm2.has_barricade()

    def test_has_artifact(self):
        """has_artifact() convenience method."""
        pm = PowerManager()
        assert not pm.has_artifact()
        pm.add_power(create_artifact(1))
        assert pm.has_artifact()


# =============================================================================
# SECTION 17: COMPLEX INTERACTION TESTS
# =============================================================================

class TestComplexInteractions:
    """Test complex power interactions."""

    def test_multiple_damage_modifiers(self):
        """Multiple damage modifiers apply correctly."""
        pm = PowerManager()
        pm.add_power(create_strength(3))
        pm.add_power(create_vigor(2))
        pm.add_power(create_weak(1))
        # (6 + 3 + 2) * 0.75 = 8.25 -> 8
        damage = pm.calculate_damage_dealt(6)
        assert damage == 8.25

    def test_multiple_block_modifiers(self):
        """Multiple block modifiers apply correctly."""
        pm = PowerManager()
        pm.add_power(create_dexterity(3))
        pm.add_power(create_frail(1))
        # (5 + 3) * 0.75 = 6
        block = pm.calculate_block(5)
        assert block == 6

    def test_vulnerable_intangible_interaction(self):
        """Vulnerable then Intangible: damage capped after vuln."""
        pm = PowerManager()
        pm.add_power(create_vulnerable(1))
        pm.add_power(create_intangible(1))
        # 10 * 1.5 = 15, but capped to 1
        damage = pm.calculate_damage_received(10.0)
        assert damage == 1

    def test_full_damage_pipeline(self):
        """Full attacker + defender damage calculation."""
        attacker = PowerManager()
        attacker.add_power(create_strength(2))
        attacker.add_power(create_weak(1))

        defender = PowerManager()
        defender.add_power(create_vulnerable(1))

        # Attacker: (6 + 2) * 0.75 = 6
        outgoing = attacker.calculate_damage_dealt(6)
        assert outgoing == 6.0

        # Defender: 6 * 1.5 = 9
        final = defender.calculate_damage_received(outgoing)
        assert final == 9


# =============================================================================
# SECTION 18: POWER DATA COMPLETENESS
# =============================================================================

class TestPowerDataCompleteness:
    """Test that power data is complete and well-formed."""

    def test_all_powers_have_name(self):
        """All powers have a name field."""
        for power_id, data in POWER_DATA.items():
            assert "name" in data, f"{power_id} missing name"

    def test_all_powers_have_type(self):
        """All powers have a type field."""
        for power_id, data in POWER_DATA.items():
            assert "type" in data, f"{power_id} missing type"
            assert data["type"] in [PowerType.BUFF, PowerType.DEBUFF]

    def test_all_powers_have_mechanics(self):
        """All powers have mechanics documentation."""
        for power_id, data in POWER_DATA.items():
            assert "mechanics" in data, f"{power_id} missing mechanics"

    def test_power_count(self):
        """Verify we have a substantial number of powers."""
        assert len(POWER_DATA) >= 60, "Expected at least 60 powers in registry"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
