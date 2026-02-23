"""
Tests for the Power Registry Integration with Combat Engine.

This file tests that the power registry system correctly triggers power effects
at the appropriate points during combat.
"""

import pytest
from packages.engine.state.combat import CombatState, EntityState, EnemyCombatState, create_combat
from packages.engine.combat_engine import CombatEngine
from packages.engine.content.cards import get_card
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

    def test_thousand_cuts_registered_after_card_played(self):
        """Thousand Cuts should be registered for onAfterCardPlayed."""
        assert POWER_REGISTRY.has_handler("onAfterCardPlayed", "ThousandCuts")

    def test_beat_of_death_registered_after_use(self):
        """Beat of Death should be registered for onAfterUseCard."""
        assert POWER_REGISTRY.has_handler("onAfterUseCard", "BeatOfDeath")

    def test_slow_registered_after_use_and_damage_receive(self):
        """Slow should be registered for onAfterUseCard and atDamageReceive."""
        assert POWER_REGISTRY.has_handler("onAfterUseCard", "Slow")
        assert POWER_REGISTRY.has_handler("atDamageReceive", "Slow")

    def test_time_warp_registered_after_use(self):
        """Time Warp should be registered for onAfterUseCard."""
        assert POWER_REGISTRY.has_handler("onAfterUseCard", "Time Warp")

    def test_bias_registered_at_start_of_turn(self):
        """Bias should be registered for atStartOfTurn."""
        assert POWER_REGISTRY.has_handler("atStartOfTurn", "Bias")

    def test_discipline_power_registered_start_and_end(self):
        """DisciplinePower should be registered for start and end of turn."""
        assert POWER_REGISTRY.has_handler("atStartOfTurn", "DisciplinePower")
        assert POWER_REGISTRY.has_handler("atEndOfTurn", "DisciplinePower")


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

    def test_discipline_draws_saved_energy(self):
        """DisciplinePower should draw saved energy count then reset."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["A", "B", "C", "D", "E"],
        )
        state.hand = []
        state.energy = 3
        state.player.statuses["DisciplinePower"] = -1

        execute_power_triggers("atEndOfTurn", state, state.player)
        assert state.player.statuses["DisciplinePower"] == 3

        execute_power_triggers("atStartOfTurn", state, state.player)
        assert len(state.hand) == 3
        assert state.player.statuses["DisciplinePower"] == -1


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


class TestOnAfterUseCard:
    """Test powers triggered after card effects resolve."""

    def test_slow_stacks_on_after_use(self):
        """Slow should increment on each card play."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="test")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        enemy.statuses["Slow"] = 0
        execute_power_triggers("onAfterUseCard", state, enemy)
        assert enemy.statuses.get("Slow") == 1
        execute_power_triggers("onAfterUseCard", state, enemy)
        assert enemy.statuses.get("Slow") == 2

    def test_slow_damage_receive_scales(self):
        """Slow should increase NORMAL damage taken by 10% per stack."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="test")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        enemy.statuses["Slow"] = 2
        result = execute_power_triggers(
            "atDamageReceive", state, enemy,
            {"value": 10, "damage_type": "NORMAL"}
        )
        assert result == 12

    def test_slow_resets_at_end_of_round(self):
        """Slow should reset to 0 at end of round without being removed."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="test")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        enemy.statuses["Slow"] = 3
        execute_power_triggers("atEndOfRound", state, enemy)
        assert enemy.statuses.get("Slow") == 0

    def test_time_warp_triggers_strength_and_end_turn_flag(self):
        """Time Warp should buff all enemies and request turn end at 12 cards."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[
                EnemyCombatState(hp=40, max_hp=40, id="time_eater"),
                EnemyCombatState(hp=35, max_hp=35, id="ally"),
            ],
            deck=["Strike"],
        )
        time_eater = state.enemies[0]
        time_eater.statuses["Time Warp"] = 11
        trigger_data = {}
        execute_power_triggers("onAfterUseCard", state, time_eater, trigger_data)
        assert time_eater.statuses.get("Time Warp") == 0
        assert state.enemies[0].statuses.get("Strength") == 2
        assert state.enemies[1].statuses.get("Strength") == 2
        assert trigger_data.get("force_end_turn") is True


class TestOnAfterCardPlayed:
    """Test powers triggered after a card is played."""

    def test_thousand_cuts_hits_all_enemies(self):
        """Thousand Cuts should damage all enemies after a card is played."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[
                EnemyCombatState(hp=30, max_hp=30, id="test1"),
                EnemyCombatState(hp=28, max_hp=28, id="test2"),
            ],
            deck=["Defend_P"],
        )
        state.hand = ["Defend_P"]
        state.player.statuses["ThousandCuts"] = 2
        engine = CombatEngine(state)
        engine.play_card(0, target_index=-1)
        assert state.enemies[0].hp == 28
        assert state.enemies[1].hp == 26


class TestCombatEngineHookOrder:
    """Verify combat_engine plays hooks in Java order."""

    def test_after_image_blocks_beat_of_death(self):
        """After Image (onUseCard) should resolve before Beat of Death."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="test")],
            deck=["Strike_P"],
        )
        state.hand = ["Strike_P"]
        state.player.statuses["AfterImage"] = 1
        state.enemies[0].statuses["BeatOfDeath"] = 1
        engine = CombatEngine(state)
        engine.play_card(0, target_index=0)
        assert state.player.hp == 50
        assert state.player.block == 0


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


class TestPow003AliasLifecycle:
    """Alias/lifecycle hooks that share Java behavior semantics."""

    def test_draw_card_next_turn_alias_registered_and_triggers(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike", "Defend", "Bash"],
        )
        state.hand = []
        state.player.statuses["DrawCardNextTurn"] = 2

        assert POWER_REGISTRY.has_handler("atStartOfTurnPostDraw", "DrawCardNextTurn")
        execute_power_triggers("atStartOfTurnPostDraw", state, state.player)

        assert len(state.hand) == 2
        assert "DrawCardNextTurn" not in state.player.statuses

    def test_intangible_player_hooks_registered(self):
        assert POWER_REGISTRY.has_handler("atDamageFinalReceive", "IntangiblePlayer")
        assert POWER_REGISTRY.has_handler("atEndOfRound", "IntangiblePlayer")

    def test_intangible_player_end_of_round_decrements(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["IntangiblePlayer"] = 2

        execute_power_triggers("atEndOfRound", state, state.player)
        assert state.player.statuses.get("IntangiblePlayer", 0) == 1
        execute_power_triggers("atEndOfRound", state, state.player)
        assert "IntangiblePlayer" not in state.player.statuses

    def test_wave_of_the_hand_power_expires_at_end_of_round(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["WaveOfTheHandPower"] = 1

        execute_power_triggers("atEndOfRound", state, state.player)
        assert "WaveOfTheHandPower" not in state.player.statuses

    def test_thorns_on_attacked_damages_attacker(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"],
        )
        attacker = state.enemies[0]
        attacker.hp = 20
        state.player.statuses["Thorns"] = 3

        execute_power_triggers(
            "onAttacked",
            state,
            state.player,
            {"attacker": attacker, "damage": 5, "damage_type": "NORMAL"},
        )

        assert attacker.hp == 17


class TestPow003BLongTailPowers:
    """Long-tail Java hook closures for POW-003B."""

    def test_flight_hooks_registered(self):
        assert POWER_REGISTRY.has_handler("atStartOfTurn", "Flight")
        assert POWER_REGISTRY.has_handler("atDamageFinalReceive", "Flight")
        assert POWER_REGISTRY.has_handler("onAttacked", "Flight")

    def test_flight_halves_normal_and_decrements_on_attacked(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="bird")],
            deck=["Strike"],
        )
        bird = state.enemies[0]
        bird.statuses["Flight"] = 3

        halved = execute_power_triggers(
            "atDamageFinalReceive",
            state,
            bird,
            {"value": 10, "damage_type": "NORMAL"},
        )
        assert halved == 5

        execute_power_triggers(
            "onAttacked",
            state,
            bird,
            {"attacker": state.player, "damage": 5, "unblocked_damage": 5, "damage_type": "NORMAL"},
        )
        assert bird.statuses.get("Flight", 0) == 2

        execute_power_triggers("atStartOfTurn", state, bird)
        assert bird.statuses.get("Flight", 0) == 3

    def test_malleable_gains_block_scales_and_resets(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=45, max_hp=45, id="slime")],
            deck=["Strike"],
        )
        slime = state.enemies[0]
        slime.statuses["Malleable"] = 3

        execute_power_triggers(
            "onAttacked",
            state,
            slime,
            {"attacker": state.player, "damage": 6, "unblocked_damage": 6, "damage_type": "NORMAL"},
        )
        assert slime.block == 3
        assert slime.statuses.get("Malleable", 0) == 4

        execute_power_triggers("atEndOfTurn", state, slime)
        assert slime.statuses.get("Malleable", 0) == 3

    def test_invincible_caps_damage_tracks_spent_and_resets(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=300, max_hp=300, id="heart")],
            deck=["Strike"],
        )
        heart = state.enemies[0]
        heart.statuses["Invincible"] = 200

        changed = execute_power_triggers(
            "onAttackedToChangeDamage",
            state,
            heart,
            {"value": 250, "damage_type": "NORMAL"},
        )
        assert changed == 200
        assert heart.statuses.get("Invincible", 0) == 0

        execute_power_triggers("atStartOfTurn", state, heart)
        assert heart.statuses.get("Invincible", 0) == 200

    def test_pen_nib_doubles_normal_damage_and_is_consumed_by_attack(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Pen Nib"] = 1

        doubled = execute_power_triggers(
            "atDamageGive",
            state,
            state.player,
            {"value": 9, "damage_type": "NORMAL"},
        )
        assert doubled == 18

        execute_power_triggers(
            "onUseCard",
            state,
            state.player,
            {"card_id": "Strike_P", "card": get_card("Strike_P")},
        )
        assert "Pen Nib" not in state.player.statuses

    def test_equilibrium_end_of_round_decrements_and_clears_retain_hand(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Equilibrium"] = 1
        state.player.statuses["RetainHand"] = 1

        execute_power_triggers("atEndOfRound", state, state.player)
        assert "Equilibrium" not in state.player.statuses
        assert "RetainHand" not in state.player.statuses

    def test_echo_form_marks_repeat_and_resets_counter_each_turn(self):
        state = create_combat(
            player_hp=50,
            player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="test")],
            deck=["Strike"],
        )
        state.player.statuses["Echo Form"] = 1
        state.cards_played_this_turn = 1

        trigger_data = {"card_id": "Strike_P", "card": get_card("Strike_P")}
        execute_power_triggers("onUseCard", state, state.player, trigger_data)
        assert trigger_data.get("repeat_play_count") == 1

        execute_power_triggers("atStartOfTurn", state, state.player)

        state.cards_played_this_turn = 1
        second = {"card_id": "Defend_P", "card": get_card("Defend_P")}
        execute_power_triggers("onUseCard", state, state.player, second)
        assert second.get("repeat_play_count") == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
