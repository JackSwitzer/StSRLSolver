"""
Audit tests: Turn-based power triggers against decompiled Java behavior.

Tests verify that the Python engine's POWER_DATA registry and PowerManager
correctly model the Java power hooks for turn-start/end triggers.
"""

import pytest
from packages.engine.content.powers import (
    POWER_DATA,
    PowerManager,
    create_power,
    create_poison,
    PowerType,
)


# =============================================================================
# POWER_DATA Registry Tests
# =============================================================================


class TestPowerDataRegistry:
    """Verify POWER_DATA has correct entries for all audited powers."""

    def test_poison_in_registry(self):
        assert "Poison" in POWER_DATA
        data = POWER_DATA["Poison"]
        assert data["type"] == PowerType.DEBUFF
        assert data["is_turn_based"] is True
        assert "at_start_of_turn" in data["mechanics"]

    def test_poison_max_amount(self):
        assert POWER_DATA["Poison"]["max_amount"] == 9999

    def test_dark_embrace_in_registry(self):
        assert "Dark Embrace" in POWER_DATA
        data = POWER_DATA["Dark Embrace"]
        assert data["type"] == PowerType.BUFF
        assert "on_exhaust" in data["mechanics"]

    def test_evolve_in_registry(self):
        assert "Evolve" in POWER_DATA
        data = POWER_DATA["Evolve"]
        assert data["type"] == PowerType.BUFF
        assert "on_card_draw" in data["mechanics"]

    def test_rupture_in_registry(self):
        assert "Rupture" in POWER_DATA
        data = POWER_DATA["Rupture"]
        assert data["type"] == PowerType.BUFF
        assert "was_hp_lost" in data["mechanics"]

    # --- Missing powers that should be added ---

    def test_regen_in_registry(self):
        assert "Regeneration" in POWER_DATA
        data = POWER_DATA["Regeneration"]
        assert data["type"] == PowerType.BUFF
        # Java RegenPower does NOT decrement -- it just heals
        assert data.get("is_turn_based", False) is False

    def test_combust_in_registry(self):
        assert "Combust" in POWER_DATA
        data = POWER_DATA["Combust"]
        assert data["type"] == PowerType.BUFF
        assert "at_end_of_turn" in data["mechanics"]

    def test_brutality_in_registry(self):
        assert "Brutality" in POWER_DATA
        data = POWER_DATA["Brutality"]
        assert data["type"] == PowerType.BUFF
        assert "at_start_of_turn_post_draw" in data["mechanics"]

    def test_feel_no_pain_in_registry(self):
        assert "Feel No Pain" in POWER_DATA
        data = POWER_DATA["Feel No Pain"]
        assert data["type"] == PowerType.BUFF
        assert "on_exhaust" in data["mechanics"]

    def test_fire_breathing_in_registry(self):
        assert "Fire Breathing" in POWER_DATA
        data = POWER_DATA["Fire Breathing"]
        assert data["type"] == PowerType.BUFF
        assert "on_card_draw" in data["mechanics"]

    def test_thousand_cuts_in_registry(self):
        assert "Thousand Cuts" in POWER_DATA
        data = POWER_DATA["Thousand Cuts"]
        assert data["type"] == PowerType.BUFF
        # Java uses onAfterCardPlayed, NOT onUseCard
        assert "on_after_card_played" in data["mechanics"]


# =============================================================================
# PowerManager Turn Hook Tests
# =============================================================================


class TestPowerManagerTurnHooks:
    """Test PowerManager at_start_of_turn and at_end_of_round behavior."""

    def test_poison_start_of_turn_returns_damage(self):
        """Poison should return HP_LOSS equal to amount at start of turn."""
        pm = PowerManager()
        pm.add_power(create_poison(5))
        effects = pm.at_start_of_turn()
        assert effects["poison_damage"] == 5

    def test_poison_does_not_auto_decrement_in_manager(self):
        """PowerManager.at_start_of_turn should report poison damage
        but the actual decrement is handled by combat_engine."""
        pm = PowerManager()
        pm.add_power(create_poison(5))
        pm.at_start_of_turn()
        # Poison should still be 5 in the manager (engine handles decrement)
        assert pm.get_amount("Poison") == 5

    def test_at_end_of_round_decrements_turn_based(self):
        """Turn-based debuffs decrement at end of round."""
        pm = PowerManager()
        pm.add_power(create_power("Weakened", 2))
        removed = pm.at_end_of_round()
        assert pm.get_amount("Weakened") == 1
        assert removed == []

    def test_at_end_of_round_removes_at_zero(self):
        """Turn-based debuffs are removed when reaching 0."""
        pm = PowerManager()
        pm.add_power(create_power("Weakened", 1))
        removed = pm.at_end_of_round()
        assert "Weakened" in removed
        assert not pm.has_power("Weakened")

    def test_just_applied_skips_first_decrement(self):
        """Monster-sourced debuffs skip first decrement."""
        pm = PowerManager()
        pm.add_power(create_power("Weakened", 2, is_source_monster=True))
        # First round: skip decrement
        removed = pm.at_end_of_round()
        assert pm.get_amount("Weakened") == 2
        # Second round: decrement
        removed = pm.at_end_of_round()
        assert pm.get_amount("Weakened") == 1

    def test_flame_barrier_removed_at_start_of_turn(self):
        """Flame Barrier is removed at start of turn."""
        pm = PowerManager()
        pm.add_power(create_power("Flame Barrier", 4))
        effects = pm.at_start_of_turn()
        assert "Flame Barrier" in effects["removed_powers"]
        assert not pm.has_power("Flame Barrier")

    def test_wrath_next_turn_stance_change(self):
        """WrathNextTurnPower triggers stance change at start of turn."""
        pm = PowerManager()
        pm.add_power(create_power("WrathNextTurnPower", 1))
        effects = pm.at_start_of_turn()
        assert effects["stance_change"] == "Wrath"
        assert not pm.has_power("WrathNextTurnPower")


# =============================================================================
# Java Hook Accuracy Tests
# =============================================================================


class TestJavaHookAccuracy:
    """Verify POWER_DATA mechanics match exact Java hook names."""

    def test_poison_uses_at_start_of_turn(self):
        """Java PoisonPower overrides atStartOfTurn(), not atEndOfTurn."""
        data = POWER_DATA["Poison"]
        assert "at_start_of_turn" in data["mechanics"]
        assert "at_end_of_turn" not in data["mechanics"]

    def test_dark_embrace_uses_on_exhaust(self):
        """Java DarkEmbracePower overrides onExhaust(card)."""
        data = POWER_DATA["Dark Embrace"]
        assert "on_exhaust" in data["mechanics"]

    def test_evolve_uses_on_card_draw(self):
        """Java EvolvePower overrides onCardDraw(card) for STATUS type."""
        data = POWER_DATA["Evolve"]
        assert "on_card_draw" in data["mechanics"]
        assert "STATUS" in data["mechanics"]["on_card_draw"]

    def test_rupture_uses_was_hp_lost(self):
        """Java RupturePower overrides wasHPLost(info, damage).
        Triggers only when damage source is self (info.owner == this.owner)."""
        data = POWER_DATA["Rupture"]
        assert "was_hp_lost" in data["mechanics"]

    def test_demon_form_uses_at_start_of_turn_post_draw(self):
        """Java DemonFormPower overrides atStartOfTurnPostDraw()."""
        data = POWER_DATA["Demon Form"]
        assert "at_start_of_turn_post_draw" in data["mechanics"]

    def test_constricted_uses_at_end_of_turn(self):
        """Java ConstrictedPower overrides atEndOfTurn(isPlayer)."""
        data = POWER_DATA["Constricted"]
        assert "at_end_of_turn" in data["mechanics"]

    def test_metallicize_uses_pre_end_turn_cards(self):
        """Java MetallicizePower overrides atEndOfTurnPreEndTurnCards."""
        data = POWER_DATA["Metallicize"]
        assert "at_end_of_turn_pre_cards" in data["mechanics"]

    def test_plated_armor_uses_pre_end_turn_cards(self):
        """Java PlatedArmorPower overrides atEndOfTurnPreEndTurnCards."""
        data = POWER_DATA["Plated Armor"]
        assert "at_end_of_turn_pre_cards" in data["mechanics"]


# =============================================================================
# Regen Bug Test
# =============================================================================


class TestRegenBug:
    """Java RegenPower does NOT decrement itself. The Python engine incorrectly
    decrements Regen by 1 each turn and removes it at 0."""

    def test_regen_should_not_decrement(self):
        """RegenPower.atEndOfTurn() in Java only heals -- no decrement.
        The power persists until combat ends or is explicitly removed.

        FIXED: combat_engine.py no longer decrements Regen stacks."""
        # Verify Regen power exists as expected
        pm = PowerManager()
        pm.add_power(create_power("Regen", 5))
        # Simulate end of round -- Regen should not decrement
        pm.at_end_of_round()
        assert pm.get_amount("Regen") == 5
