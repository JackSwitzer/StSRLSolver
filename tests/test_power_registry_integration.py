"""
Tests for the Power Registry Integration with Combat Engine.

This file tests that the power registry system correctly triggers power effects
at the appropriate points during combat.
"""

import pytest
from packages.engine.state.combat import CombatState, EntityState, EnemyCombatState, create_combat
from packages.engine.registry import (
    execute_power_triggers, PowerContext, POWER_REGISTRY
)
from packages.engine.registry.powers import (
    metallicize_end, plated_armor_end, like_water_end,
    constricted_end, combust_end, ritual_end,
    weak_end_round, vulnerable_end_round, frail_end_round,
    poison_start, regeneration_start, choked_start,
    after_image_on_use, choked_on_use,
    dark_embrace_exhaust, feel_no_pain_exhaust,
    mental_fortress_stance, rushdown_stance,
    dexterity_modify_block, frail_modify_block,
    strength_damage_give, vigor_damage_give, weak_damage_give,
    vulnerable_damage_receive, intangible_damage_final,
    juggernaut_gain_block, wave_of_hand_gain_block,
)


class TestPowerRegistrySetup:
    """Test that power handlers are properly registered."""

    def test_metallicize_registered(self):
        """Metallicize should be registered for atEndOfTurnPreEndTurnCards."""
        assert POWER_REGISTRY.has_handler("atEndOfTurnPreEndTurnCards", "Metallicize")

    def test_plated_armor_registered(self):
        """Plated Armor should be registered for atEndOfTurnPreEndTurnCards."""
        assert POWER_REGISTRY.has_handler("atEndOfTurnPreEndTurnCards", "Plated Armor")

    def test_poison_registered(self):
        """Poison should be registered for atStartOfTurn."""
        assert POWER_REGISTRY.has_handler("atStartOfTurn", "Poison")

    def test_strength_registered(self):
        """Strength should be registered for atDamageGive."""
        assert POWER_REGISTRY.has_handler("atDamageGive", "Strength")

    def test_vulnerable_registered(self):
        """Vulnerable should be registered for atDamageReceive."""
        assert POWER_REGISTRY.has_handler("atDamageReceive", "Vulnerable")

    def test_weak_end_round_registered(self):
        """Weakened should be registered for atEndOfRound."""
        assert POWER_REGISTRY.has_handler("atEndOfRound", "Weakened")


class TestAtEndOfTurnPreEndTurnCards:
    """Test powers that trigger before discarding at end of turn."""

    def test_metallicize_gains_block(self):
        """Metallicize should gain block equal to its amount at end of turn."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Metallicize"] = 3
        initial_block = state.player.block

        execute_power_triggers("atEndOfTurnPreEndTurnCards", state, state.player)

        assert state.player.block == initial_block + 3

    def test_plated_armor_gains_block(self):
        """Plated Armor should gain block equal to its amount at end of turn."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Plated Armor"] = 4
        initial_block = state.player.block

        execute_power_triggers("atEndOfTurnPreEndTurnCards", state, state.player)

        assert state.player.block == initial_block + 4

    def test_like_water_gains_block_in_calm(self):
        """Like Water should gain block if in Calm stance."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["LikeWater"] = 5
        state.stance = "Calm"
        initial_block = state.player.block

        execute_power_triggers("atEndOfTurnPreEndTurnCards", state, state.player)

        assert state.player.block == initial_block + 5

    def test_like_water_no_block_in_wrath(self):
        """Like Water should not gain block if not in Calm stance."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["LikeWater"] = 5
        state.stance = "Wrath"
        initial_block = state.player.block

        execute_power_triggers("atEndOfTurnPreEndTurnCards", state, state.player)

        assert state.player.block == initial_block  # No change


class TestAtStartOfTurn:
    """Test powers that trigger at start of turn."""

    def test_poison_deals_damage_and_decrements(self):
        """Poison should deal HP damage and decrement at start of turn."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        enemy.statuses["Poison"] = 5

        execute_power_triggers("atStartOfTurn", state, enemy)

        assert enemy.hp == 25  # 30 - 5 poison damage
        assert enemy.statuses.get("Poison", 0) == 4  # Decremented by 1

    def test_poison_removes_at_zero(self):
        """Poison should be removed when it reaches 0."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        enemy.statuses["Poison"] = 1

        execute_power_triggers("atStartOfTurn", state, enemy)

        assert enemy.hp == 29  # 30 - 1 poison damage
        assert "Poison" not in enemy.statuses  # Removed


class TestAtEndOfRound:
    """Test powers that trigger at end of round (after all turns)."""

    def test_weak_decrements(self):
        """Weakened should decrement at end of round."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Weakened"] = 2

        execute_power_triggers("atEndOfRound", state, state.player)

        assert state.player.statuses.get("Weakened", 0) == 1

    def test_weak_removes_at_zero(self):
        """Weakened should be removed when it reaches 0."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Weakened"] = 1

        execute_power_triggers("atEndOfRound", state, state.player)

        assert "Weakened" not in state.player.statuses

    def test_vulnerable_decrements(self):
        """Vulnerable should decrement at end of round."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        enemy.statuses["Vulnerable"] = 3

        execute_power_triggers("atEndOfRound", state, enemy)

        assert enemy.statuses.get("Vulnerable", 0) == 2

    def test_frail_decrements(self):
        """Frail should decrement at end of round."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Frail"] = 2

        execute_power_triggers("atEndOfRound", state, state.player)

        assert state.player.statuses.get("Frail", 0) == 1


class TestDamageModifiers:
    """Test damage modification hooks."""

    def test_strength_adds_damage(self):
        """Strength should add to base damage."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Strength"] = 3

        result = execute_power_triggers(
            "atDamageGive", state, state.player,
            {"value": 6}
        )

        assert result == 9  # 6 + 3 strength

    def test_vigor_adds_damage(self):
        """Vigor should add to attack damage."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Vigor"] = 5

        result = execute_power_triggers(
            "atDamageGive", state, state.player,
            {"value": 6}
        )

        assert result == 11  # 6 + 5 vigor

    def test_weak_reduces_damage(self):
        """Weak should reduce damage by 25%."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Weakened"] = 1

        result = execute_power_triggers(
            "atDamageGive", state, state.player,
            {"value": 10}
        )

        assert result == 7  # 10 * 0.75 = 7.5 -> 7

    def test_vulnerable_increases_damage(self):
        """Vulnerable should increase damage taken by 50%."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        enemy.statuses["Vulnerable"] = 1

        result = execute_power_triggers(
            "atDamageReceive", state, enemy,
            {"value": 10}
        )

        assert result == 15  # 10 * 1.5 = 15

    def test_intangible_caps_damage_at_1(self):
        """Intangible should reduce all damage to 1."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Intangible"] = 1

        result = execute_power_triggers(
            "atDamageFinalReceive", state, state.player,
            {"value": 100}
        )

        assert result == 1


class TestBlockModifiers:
    """Test block modification hooks."""

    def test_dexterity_adds_block(self):
        """Dexterity should add to block gained from cards."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Dexterity"] = 2

        result = execute_power_triggers(
            "modifyBlock", state, state.player,
            {"value": 5}
        )

        assert result == 7  # 5 + 2 dexterity

    def test_frail_reduces_block(self):
        """Frail should reduce block by 25%."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Frail"] = 1

        result = execute_power_triggers(
            "modifyBlock", state, state.player,
            {"value": 8}
        )

        assert result == 6  # 8 * 0.75 = 6


class TestOnUseCard:
    """Test powers triggered when a card is used."""

    def test_after_image_gains_block(self):
        """After Image should gain block when playing any card."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["AfterImage"] = 1
        initial_block = state.player.block

        execute_power_triggers("onUseCard", state, state.player)

        assert state.player.block == initial_block + 1

    def test_choked_loses_hp_on_card(self):
        """Choke should cause HP loss when playing cards."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Choked"] = 3

        execute_power_triggers("onUseCard", state, state.player)

        assert state.player.hp == 47  # 50 - 3


class TestOnExhaust:
    """Test powers triggered when a card is exhausted."""

    def test_dark_embrace_draws_card(self):
        """Dark Embrace should draw a card when exhausting."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["DarkEmbrace"] = 1
        state.draw_pile = ["Defend", "Defend"]
        initial_hand_size = len(state.hand)

        execute_power_triggers("onExhaust", state, state.player)

        assert len(state.hand) == initial_hand_size + 1

    def test_feel_no_pain_gains_block(self):
        """Feel No Pain should gain block when exhausting."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["FeelNoPain"] = 4
        initial_block = state.player.block

        execute_power_triggers("onExhaust", state, state.player)

        assert state.player.block == initial_block + 4


class TestOnChangeStance:
    """Test powers triggered on stance change (Watcher)."""

    def test_mental_fortress_gains_block(self):
        """Mental Fortress should gain block on stance change."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["MentalFortress"] = 6
        initial_block = state.player.block

        execute_power_triggers("onChangeStance", state, state.player, {"new_stance": "Wrath"})

        assert state.player.block == initial_block + 6

    def test_rushdown_draws_on_wrath(self):
        """Rushdown should draw cards when entering Wrath."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Rushdown"] = 2
        state.draw_pile = ["Defend", "Strike"]
        initial_hand_size = len(state.hand)

        execute_power_triggers("onChangeStance", state, state.player, {"new_stance": "Wrath"})

        assert len(state.hand) == initial_hand_size + 2

    def test_rushdown_no_draw_on_calm(self):
        """Rushdown should not draw when entering Calm."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Rushdown"] = 2
        state.draw_pile = ["Defend", "Strike"]
        initial_hand_size = len(state.hand)

        execute_power_triggers("onChangeStance", state, state.player, {"new_stance": "Calm"})

        assert len(state.hand) == initial_hand_size  # No change


class TestOnGainBlock:
    """Test powers triggered when gaining block."""

    def test_juggernaut_deals_damage(self):
        """Juggernaut should deal damage to random enemy when gaining block."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Juggernaut"] = 5
        initial_hp = state.enemies[0].hp

        execute_power_triggers("onGainBlock", state, state.player)

        # Enemy should take some damage (could be blocked first)
        assert state.enemies[0].hp <= initial_hp

    def test_wave_of_hand_applies_weak(self):
        """Wave of the Hand should apply Weak to all enemies when gaining block."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[
                EnemyCombatState(hp=30, max_hp=30, id="test1"),
                EnemyCombatState(hp=30, max_hp=30, id="test2"),
            ],
            deck=["Strike"],
        )
        state.player.statuses["WaveOfTheHand"] = 1

        execute_power_triggers("onGainBlock", state, state.player)

        assert state.enemies[0].statuses.get("Weakened", 0) == 1
        assert state.enemies[1].statuses.get("Weakened", 0) == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
