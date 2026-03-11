"""
Tests for newly implemented power handlers (POW-002B / POW-003A).

Tests cover:
- System/Shared: NoBlockPower, Entangled, Double Damage, Repair, Lock-On, Panache reset
- Boss/Enemy: Angry, Curiosity, GrowthPower, Fading, Thievery
- Defect: Storm, Static Discharge
- Watcher: BlockReturnPower, FreeAttackPower, CannotChangeStancePower
- Dispatch: modifyBlockLast, onVictory power triggers
"""

import pytest
from packages.engine.state.combat import CombatState, EntityState, EnemyCombatState, create_combat
from packages.engine.combat_engine import CombatEngine
from packages.engine.content.cards import get_card, CardType
from packages.engine.registry import (
    execute_power_triggers, PowerContext, POWER_REGISTRY
)
# Import powers module to register handlers
from packages.engine.registry import powers as _powers  # noqa: F401


# =============================================================================
# Helper to create a basic combat state
# =============================================================================

def _make_state(
    player_hp=50,
    player_max_hp=50,
    enemies=None,
    deck=None,
    gold=100,
):
    """Create a test combat state."""
    if enemies is None:
        enemies = [EnemyCombatState(hp=30, max_hp=30, id="test_enemy")]
    if deck is None:
        deck = ["Strike"]
    state = create_combat(
        player_hp=player_hp,
        player_max_hp=player_max_hp,
        enemies=enemies,
        deck=deck,
    )
    state.gold = gold
    return state


# =============================================================================
# SECTION: Registration Tests
# =============================================================================

class TestNewHandlerRegistration:
    """Verify all new handlers are properly registered."""

    def test_no_block_power_registered(self):
        assert POWER_REGISTRY.has_handler("modifyBlockLast", "NoBlockPower")
        assert POWER_REGISTRY.has_handler("atEndOfRound", "NoBlockPower")

    def test_entangled_registered(self):
        assert POWER_REGISTRY.has_handler("atEndOfTurn", "Entangled")

    def test_double_damage_registered(self):
        assert POWER_REGISTRY.has_handler("atDamageGive", "Double Damage")
        assert POWER_REGISTRY.has_handler("atEndOfRound", "Double Damage")

    def test_repair_registered(self):
        assert POWER_REGISTRY.has_handler("onVictory", "Repair")

    def test_lockon_registered(self):
        assert POWER_REGISTRY.has_handler("atEndOfRound", "Lockon")

    def test_panache_start_registered(self):
        assert POWER_REGISTRY.has_handler("atStartOfTurn", "Panache")

    def test_angry_registered(self):
        assert POWER_REGISTRY.has_handler("onAttacked", "Angry")

    def test_curiosity_registered(self):
        assert POWER_REGISTRY.has_handler("onPlayCard", "Curiosity")

    def test_growth_power_registered(self):
        assert POWER_REGISTRY.has_handler("atEndOfRound", "GrowthPower")

    def test_fading_registered(self):
        assert POWER_REGISTRY.has_handler("atEndOfTurn", "Fading")

    def test_thievery_registered(self):
        assert POWER_REGISTRY.has_handler("onAttack", "Thievery")

    def test_storm_registered(self):
        assert POWER_REGISTRY.has_handler("onUseCard", "Storm")

    def test_static_discharge_registered(self):
        assert POWER_REGISTRY.has_handler("onAttacked", "StaticDischarge")

    def test_block_return_registered(self):
        assert POWER_REGISTRY.has_handler("onAttacked", "BlockReturnPower")

    def test_free_attack_registered(self):
        assert POWER_REGISTRY.has_handler("onUseCard", "FreeAttackPower")

    def test_cannot_change_stance_registered(self):
        assert POWER_REGISTRY.has_handler("atEndOfTurn", "CannotChangeStancePower")


# =============================================================================
# SECTION: System / Shared Powers
# =============================================================================

class TestNoBlockPower:
    """Test NoBlockPower: modifyBlockLast -> 0."""

    def test_modify_block_last_returns_zero(self):
        """NoBlockPower should reduce all block to 0."""
        state = _make_state()
        state.player.statuses["NoBlockPower"] = 1
        result = execute_power_triggers(
            "modifyBlockLast", state, state.player, {"value": 15.0}
        )
        assert result == 0

    def test_no_block_power_end_round_decrement(self):
        """NoBlockPower decrements at end of round."""
        state = _make_state()
        state.player.statuses["NoBlockPower"] = 2
        execute_power_triggers("atEndOfRound", state, state.player)
        assert state.player.statuses.get("NoBlockPower") == 1

    def test_no_block_power_end_round_remove(self):
        """NoBlockPower removed when reaching 0."""
        state = _make_state()
        state.player.statuses["NoBlockPower"] = 1
        execute_power_triggers("atEndOfRound", state, state.player)
        # After decrement from 1, it becomes 0 which gets removed
        assert state.player.statuses.get("NoBlockPower", 0) == 0

    def test_no_block_via_combat_engine(self):
        """NoBlockPower should prevent block gain from cards via combat engine."""
        state = _make_state(deck=["Defend", "Strike"])
        state.player.statuses["NoBlockPower"] = 1
        engine = CombatEngine(state)
        engine.phase = engine.phase.__class__("PLAYER_TURN")
        state.energy = 3
        # Calculate block through the engine path
        block = engine._calculate_block_gained(5)
        assert block == 0


class TestEntangled:
    """Test Entangled: can't play attacks, removed at end of turn."""

    def test_entangled_removed_at_end_of_turn(self):
        """Entangled should be removed at end of turn."""
        state = _make_state()
        state.player.statuses["Entangled"] = 1
        execute_power_triggers("atEndOfTurn", state, state.player)
        assert "Entangled" not in state.player.statuses

    def test_entangled_blocks_attacks(self):
        """Entangled should prevent playing Attack cards."""
        state = _make_state(deck=["Strike_R", "Defend_R"])
        state.hand = ["Strike_R", "Defend_R"]
        state.player.statuses["Entangled"] = 1
        state.energy = 3
        engine = CombatEngine(state)
        engine.phase = engine.phase.__class__("PLAYER_TURN")
        strike = get_card("Strike_R")
        defend = get_card("Defend_R")
        assert not engine._can_play_card(strike, 0)
        assert engine._can_play_card(defend, 1)


class TestDoubleDamage:
    """Test Double Damage: doubles NORMAL damage, decrements at end of round."""

    def test_double_damage_give(self):
        """Double Damage should double NORMAL damage."""
        state = _make_state()
        state.player.statuses["Double Damage"] = 1
        result = execute_power_triggers(
            "atDamageGive", state, state.player,
            {"value": 10.0, "damage_type": "NORMAL"}
        )
        assert result == 20.0

    def test_double_damage_no_effect_on_thorns(self):
        """Double Damage should NOT affect THORNS damage."""
        state = _make_state()
        state.player.statuses["Double Damage"] = 1
        result = execute_power_triggers(
            "atDamageGive", state, state.player,
            {"value": 10.0, "damage_type": "THORNS"}
        )
        # THORNS damage is not doubled — result should be unchanged
        assert result is None or result == 10.0

    def test_double_damage_decrement_at_end_of_round(self):
        """Double Damage should decrement at end of round."""
        state = _make_state()
        state.player.statuses["Double Damage"] = 2
        execute_power_triggers("atEndOfRound", state, state.player)
        assert state.player.statuses.get("Double Damage") == 1

    def test_double_damage_remove_at_zero(self):
        """Double Damage should be removed when it reaches 0."""
        state = _make_state()
        state.player.statuses["Double Damage"] = 1
        execute_power_triggers("atEndOfRound", state, state.player)
        assert state.player.statuses.get("Double Damage", 0) == 0


class TestRepair:
    """Test Repair: heal on victory."""

    def test_repair_heals_on_victory(self):
        """Repair should heal the player on victory."""
        state = _make_state(player_hp=30, player_max_hp=50)
        state.player.statuses["Repair"] = 7
        execute_power_triggers("onVictory", state, state.player)
        assert state.player.hp == 37

    def test_repair_capped_at_max_hp(self):
        """Repair healing should not exceed max HP."""
        state = _make_state(player_hp=48, player_max_hp=50)
        state.player.statuses["Repair"] = 10
        execute_power_triggers("onVictory", state, state.player)
        assert state.player.hp == 50

    def test_repair_no_heal_if_dead(self):
        """Repair should not heal if player HP is 0."""
        state = _make_state(player_hp=0, player_max_hp=50)
        state.player.statuses["Repair"] = 7
        execute_power_triggers("onVictory", state, state.player)
        assert state.player.hp == 0

    def test_repair_via_combat_engine(self):
        """Repair should trigger via combat engine onVictory dispatch."""
        state = _make_state(player_hp=30, player_max_hp=50, deck=["Strike"])
        state.player.statuses["Repair"] = 5
        state.enemies[0].hp = 0  # Enemy already dead
        engine = CombatEngine(state)
        engine._end_combat(player_won=True)
        assert state.player.hp == 35


class TestLockOn:
    """Test Lock-On: decrement at end of round."""

    def test_lockon_decrements(self):
        """Lock-On should decrement at end of round."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Lockon"] = 3
        execute_power_triggers("atEndOfRound", state, enemy)
        assert enemy.statuses.get("Lockon") == 2

    def test_lockon_removed_at_zero(self):
        """Lock-On should be removed when reaching 0."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Lockon"] = 1
        execute_power_triggers("atEndOfRound", state, enemy)
        assert enemy.statuses.get("Lockon", 0) == 0


class TestPanacheReset:
    """Test Panache: counter resets at start of turn."""

    def test_panache_counter_resets(self):
        """Panache counter should reset to 0 at start of turn."""
        state = _make_state()
        state.player.statuses["Panache"] = 10
        state.player.statuses["PanacheCounter"] = 3
        execute_power_triggers("atStartOfTurn", state, state.player)
        assert state.player.statuses.get("PanacheCounter") == 0


# =============================================================================
# SECTION: Boss / Enemy Powers
# =============================================================================

class TestAngry:
    """Test Angry: gain Strength when attacked."""

    def test_angry_gains_strength_on_attack(self):
        """Angry enemy should gain Strength when hit."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Angry"] = 2
        execute_power_triggers(
            "onAttacked", state, enemy,
            {"attacker": state.player, "damage": 10, "unblocked_damage": 8,
             "damage_type": "NORMAL"}
        )
        assert enemy.statuses.get("Strength") == 2

    def test_angry_stacks_strength(self):
        """Angry should stack Strength on multiple hits."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Angry"] = 1
        enemy.statuses["Strength"] = 3
        execute_power_triggers(
            "onAttacked", state, enemy,
            {"attacker": state.player, "damage": 5, "unblocked_damage": 5,
             "damage_type": "NORMAL"}
        )
        assert enemy.statuses.get("Strength") == 4

    def test_angry_no_trigger_on_hp_loss(self):
        """Angry should not trigger on HP_LOSS damage type."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Angry"] = 2
        execute_power_triggers(
            "onAttacked", state, enemy,
            {"attacker": state.player, "damage": 10, "unblocked_damage": 8,
             "damage_type": "HP_LOSS"}
        )
        assert enemy.statuses.get("Strength", 0) == 0

    def test_angry_no_trigger_on_zero_damage(self):
        """Angry should not trigger when all damage is blocked."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Angry"] = 2
        execute_power_triggers(
            "onAttacked", state, enemy,
            {"attacker": state.player, "damage": 10, "unblocked_damage": 0,
             "damage_type": "NORMAL"}
        )
        assert enemy.statuses.get("Strength", 0) == 0


class TestCuriosity:
    """Test Curiosity: gain Strength when player plays Power card."""

    def test_curiosity_gains_strength_on_power(self):
        """Curiosity enemy should gain Strength when player plays a Power."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Curiosity"] = 1
        mock_card = type("Card", (), {"card_type": CardType.POWER, "id": "TestPower"})()
        execute_power_triggers(
            "onPlayCard", state, enemy,
            {"card": mock_card, "card_id": "TestPower"}
        )
        assert enemy.statuses.get("Strength") == 1

    def test_curiosity_no_trigger_on_attack(self):
        """Curiosity should not trigger on Attack cards."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Curiosity"] = 1
        mock_card = type("Card", (), {"card_type": CardType.ATTACK, "id": "Strike"})()
        execute_power_triggers(
            "onPlayCard", state, enemy,
            {"card": mock_card, "card_id": "Strike"}
        )
        assert enemy.statuses.get("Strength", 0) == 0

    def test_curiosity_stacks_multiple(self):
        """Curiosity amount controls Strength gain."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Curiosity"] = 3
        mock_card = type("Card", (), {"card_type": CardType.POWER, "id": "TestPower"})()
        execute_power_triggers(
            "onPlayCard", state, enemy,
            {"card": mock_card, "card_id": "TestPower"}
        )
        assert enemy.statuses.get("Strength") == 3


class TestGrowthPower:
    """Test GrowthPower: gain Strength at end of round with skip-first."""

    def test_growth_skips_first_round(self):
        """GrowthPower should skip the first end of round."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["GrowthPower"] = 3
        execute_power_triggers("atEndOfRound", state, enemy)
        assert enemy.statuses.get("Strength", 0) == 0

    def test_growth_gains_strength_second_round(self):
        """GrowthPower should gain Strength on second end of round."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["GrowthPower"] = 3
        # First round: skip
        execute_power_triggers("atEndOfRound", state, enemy)
        # Second round: gain
        execute_power_triggers("atEndOfRound", state, enemy)
        assert enemy.statuses.get("Strength") == 3

    def test_growth_accumulates(self):
        """GrowthPower should accumulate Strength each round after first."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["GrowthPower"] = 2
        # Round 1: skip
        execute_power_triggers("atEndOfRound", state, enemy)
        # Round 2: +2
        execute_power_triggers("atEndOfRound", state, enemy)
        assert enemy.statuses.get("Strength") == 2
        # Round 3: +2
        execute_power_triggers("atEndOfRound", state, enemy)
        assert enemy.statuses.get("Strength") == 4


class TestFading:
    """Test Fading: decrement, die at 1."""

    def test_fading_decrements(self):
        """Fading should decrement each turn."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Fading"] = 3
        execute_power_triggers("atEndOfTurn", state, enemy)
        assert enemy.statuses.get("Fading") == 2

    def test_fading_kills_at_one(self):
        """Fading should kill the creature when it reaches 1."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Fading"] = 1
        execute_power_triggers("atEndOfTurn", state, enemy)
        assert enemy.hp == 0

    def test_fading_not_on_player(self):
        """Fading should not affect the player (enemy-only power)."""
        state = _make_state()
        state.player.statuses["Fading"] = 1
        initial_hp = state.player.hp
        execute_power_triggers("atEndOfTurn", state, state.player)
        assert state.player.hp == initial_hp


class TestThievery:
    """Test Thievery: steal gold on unblocked attack."""

    def test_thievery_steals_gold(self):
        """Thievery should steal gold on unblocked damage."""
        state = _make_state(gold=100)
        enemy = state.enemies[0]
        enemy.statuses["Thievery"] = 15
        execute_power_triggers(
            "onAttack", state, enemy,
            {"target": state.player, "damage": 10, "unblocked_damage": 5,
             "damage_type": "NORMAL"}
        )
        assert state.gold == 85

    def test_thievery_caps_at_available_gold(self):
        """Thievery should not steal more gold than player has."""
        state = _make_state(gold=5)
        enemy = state.enemies[0]
        enemy.statuses["Thievery"] = 15
        execute_power_triggers(
            "onAttack", state, enemy,
            {"target": state.player, "damage": 10, "unblocked_damage": 5,
             "damage_type": "NORMAL"}
        )
        assert state.gold == 0

    def test_thievery_no_steal_on_blocked(self):
        """Thievery should not steal gold when damage is fully blocked."""
        state = _make_state(gold=100)
        enemy = state.enemies[0]
        enemy.statuses["Thievery"] = 15
        execute_power_triggers(
            "onAttack", state, enemy,
            {"target": state.player, "damage": 10, "unblocked_damage": 0,
             "damage_type": "NORMAL"}
        )
        assert state.gold == 100


# =============================================================================
# SECTION: Defect Powers
# =============================================================================

class TestStorm:
    """Test Storm: channel Lightning when playing Power cards."""

    def test_storm_channels_lightning(self):
        """Storm should channel Lightning orb(s) when a Power is played."""
        state = _make_state()
        state.player.statuses["Storm"] = 1
        # Initialize orb slots
        state.player.statuses["OrbSlots"] = 3
        mock_card = type("Card", (), {"card_type": CardType.POWER, "id": "TestPower"})()
        execute_power_triggers(
            "onUseCard", state, state.player,
            {"card": mock_card, "card_id": "TestPower"}
        )
        # Check that Lightning orb was channeled via orb manager
        from packages.engine.effects.orbs import get_orb_manager
        manager = get_orb_manager(state)
        assert len(manager.orbs) >= 1
        assert manager.orbs[0].orb_type.value == "Lightning"

    def test_storm_no_trigger_on_attack(self):
        """Storm should not trigger on Attack cards."""
        state = _make_state()
        state.player.statuses["Storm"] = 1
        state.player.statuses["OrbSlots"] = 3
        mock_card = type("Card", (), {"card_type": CardType.ATTACK, "id": "Strike"})()
        execute_power_triggers(
            "onUseCard", state, state.player,
            {"card": mock_card, "card_id": "Strike"}
        )
        from packages.engine.effects.orbs import get_orb_manager
        manager = get_orb_manager(state)
        assert len(manager.orbs) == 0


class TestStaticDischarge:
    """Test Static Discharge: channel Lightning when attacked."""

    def test_static_discharge_channels_on_attack(self):
        """Static Discharge should channel Lightning when hit."""
        state = _make_state()
        state.player.statuses["StaticDischarge"] = 1
        state.player.statuses["OrbSlots"] = 3
        enemy = state.enemies[0]
        execute_power_triggers(
            "onAttacked", state, state.player,
            {"attacker": enemy, "damage": 10, "unblocked_damage": 5,
             "damage_type": "NORMAL"}
        )
        from packages.engine.effects.orbs import get_orb_manager
        manager = get_orb_manager(state)
        assert len(manager.orbs) >= 1
        assert manager.orbs[0].orb_type.value == "Lightning"

    def test_static_discharge_no_trigger_on_thorns(self):
        """Static Discharge should not trigger on THORNS damage."""
        state = _make_state()
        state.player.statuses["StaticDischarge"] = 1
        state.player.statuses["OrbSlots"] = 3
        enemy = state.enemies[0]
        execute_power_triggers(
            "onAttacked", state, state.player,
            {"attacker": enemy, "damage": 10, "unblocked_damage": 5,
             "damage_type": "THORNS"}
        )
        from packages.engine.effects.orbs import get_orb_manager
        manager = get_orb_manager(state)
        assert len(manager.orbs) == 0

    def test_static_discharge_no_trigger_self_damage(self):
        """Static Discharge should not trigger when attacker is self."""
        state = _make_state()
        state.player.statuses["StaticDischarge"] = 1
        state.player.statuses["OrbSlots"] = 3
        execute_power_triggers(
            "onAttacked", state, state.player,
            {"attacker": state.player, "damage": 10, "unblocked_damage": 5,
             "damage_type": "NORMAL"}
        )
        from packages.engine.effects.orbs import get_orb_manager
        manager = get_orb_manager(state)
        assert len(manager.orbs) == 0

    def test_static_discharge_amount_controls_channels(self):
        """Static Discharge amount controls number of Lightning orbs channeled."""
        state = _make_state()
        state.player.statuses["StaticDischarge"] = 2
        state.player.statuses["OrbSlots"] = 3
        enemy = state.enemies[0]
        execute_power_triggers(
            "onAttacked", state, state.player,
            {"attacker": enemy, "damage": 10, "unblocked_damage": 5,
             "damage_type": "NORMAL"}
        )
        from packages.engine.effects.orbs import get_orb_manager
        manager = get_orb_manager(state)
        assert len(manager.orbs) >= 2


# =============================================================================
# SECTION: Watcher Powers
# =============================================================================

class TestBlockReturnPower:
    """Test BlockReturnPower (Talk to the Hand mark)."""

    def test_block_return_gains_block(self):
        """BlockReturnPower should give player block when marked enemy is hit."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["BlockReturnPower"] = 3
        execute_power_triggers(
            "onAttacked", state, enemy,
            {"attacker": state.player, "damage": 10, "unblocked_damage": 5,
             "damage_type": "NORMAL"}
        )
        assert state.player.block == 3

    def test_block_return_no_effect_on_zero_damage(self):
        """BlockReturnPower should not grant block when no damage goes through."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["BlockReturnPower"] = 3
        execute_power_triggers(
            "onAttacked", state, enemy,
            {"attacker": state.player, "damage": 10, "unblocked_damage": 0,
             "damage_type": "NORMAL"}
        )
        assert state.player.block == 0


class TestFreeAttackPower:
    """Test FreeAttackPower: decrement on Attack play."""

    def test_free_attack_decrements_on_attack(self):
        """FreeAttackPower should decrement when an Attack is played."""
        state = _make_state()
        state.player.statuses["FreeAttackPower"] = 2
        mock_card = type("Card", (), {"card_type": CardType.ATTACK, "id": "Strike"})()
        execute_power_triggers(
            "onUseCard", state, state.player,
            {"card": mock_card, "card_id": "Strike"}
        )
        assert state.player.statuses.get("FreeAttackPower") == 1

    def test_free_attack_removed_at_one(self):
        """FreeAttackPower should be removed when it reaches 0."""
        state = _make_state()
        state.player.statuses["FreeAttackPower"] = 1
        mock_card = type("Card", (), {"card_type": CardType.ATTACK, "id": "Strike"})()
        execute_power_triggers(
            "onUseCard", state, state.player,
            {"card": mock_card, "card_id": "Strike"}
        )
        assert "FreeAttackPower" not in state.player.statuses

    def test_free_attack_no_trigger_on_skill(self):
        """FreeAttackPower should not decrement on Skill cards."""
        state = _make_state()
        state.player.statuses["FreeAttackPower"] = 2
        mock_card = type("Card", (), {"card_type": CardType.SKILL, "id": "Defend"})()
        execute_power_triggers(
            "onUseCard", state, state.player,
            {"card": mock_card, "card_id": "Defend"}
        )
        assert state.player.statuses.get("FreeAttackPower") == 2


class TestCannotChangeStancePower:
    """Test CannotChangeStancePower: removed at end of turn."""

    def test_cannot_change_stance_removed_at_end(self):
        """CannotChangeStancePower should be removed at end of turn."""
        state = _make_state()
        state.player.statuses["CannotChangeStancePower"] = 1
        execute_power_triggers("atEndOfTurn", state, state.player)
        assert "CannotChangeStancePower" not in state.player.statuses


# =============================================================================
# SECTION: Dispatch Integration Tests
# =============================================================================

class TestModifyBlockLastDispatch:
    """Test that modifyBlockLast is dispatched in the block calculation chain."""

    def test_block_calc_chain_with_no_block_power(self):
        """modifyBlockLast should be applied after modifyBlock in chain."""
        state = _make_state(deck=["Defend"])
        state.player.statuses["NoBlockPower"] = 1
        state.player.statuses["Dexterity"] = 5
        engine = CombatEngine(state)
        engine.phase = engine.phase.__class__("PLAYER_TURN")
        # Even with +5 Dex, NoBlockPower should zero out the result
        block = engine._calculate_block_gained(5)
        assert block == 0

    def test_block_calc_chain_without_no_block(self):
        """Without NoBlockPower, block calculation should work normally."""
        state = _make_state(deck=["Defend"])
        state.player.statuses["Dexterity"] = 3
        engine = CombatEngine(state)
        engine.phase = engine.phase.__class__("PLAYER_TURN")
        block = engine._calculate_block_gained(5)
        assert block == 8  # 5 base + 3 dex


class TestOnVictoryPowerDispatch:
    """Test that onVictory is dispatched for powers in combat engine."""

    def test_victory_triggers_repair(self):
        """_end_combat(player_won=True) should trigger Repair power."""
        state = _make_state(player_hp=30, player_max_hp=50, deck=["Strike"])
        state.player.statuses["Repair"] = 10
        state.enemies[0].hp = 0
        engine = CombatEngine(state)
        engine._end_combat(player_won=True)
        assert state.player.hp == 40

    def test_defeat_does_not_trigger_repair(self):
        """_end_combat(player_won=False) should NOT trigger Repair power."""
        state = _make_state(player_hp=30, player_max_hp=50, deck=["Strike"])
        state.player.statuses["Repair"] = 10
        engine = CombatEngine(state)
        engine._end_combat(player_won=False)
        assert state.player.hp == 30


# =============================================================================
# SECTION: Edge Cases
# =============================================================================

class TestEdgeCases:
    """Test edge cases for new power handlers."""

    def test_double_damage_stacks_with_strength(self):
        """Double Damage and Strength should stack correctly.

        Handler return values are chained through trigger_data["value"],
        matching Java's power iteration.
        Priority: Double Damage (6) runs before Strength (100).
        Double Damage: 10 * 2 = 20 (chained into trigger_data).
        Strength: 20 + 3 = 23 (reads chained value).
        """
        state = _make_state()
        state.player.statuses["Strength"] = 3
        state.player.statuses["Double Damage"] = 1
        result = execute_power_triggers(
            "atDamageGive", state, state.player,
            {"value": 10.0, "damage_type": "NORMAL"}
        )
        # Chained: DoubleDamage(10*2=20) -> Strength(20+3=23)
        assert result == 23.0

    def test_angry_and_thorns_interaction(self):
        """Angry should not trigger from Thorns damage type."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Angry"] = 2
        execute_power_triggers(
            "onAttacked", state, enemy,
            {"attacker": state.player, "damage": 5, "unblocked_damage": 5,
             "damage_type": "THORNS"}
        )
        assert enemy.statuses.get("Strength", 0) == 0

    def test_fading_countdown_sequence(self):
        """Fading should count down 3, 2, 1, die."""
        state = _make_state()
        enemy = state.enemies[0]
        enemy.statuses["Fading"] = 3

        execute_power_triggers("atEndOfTurn", state, enemy)
        assert enemy.statuses.get("Fading") == 2
        assert enemy.hp > 0

        execute_power_triggers("atEndOfTurn", state, enemy)
        assert enemy.statuses.get("Fading") == 1
        assert enemy.hp > 0

        execute_power_triggers("atEndOfTurn", state, enemy)
        assert enemy.hp == 0

    def test_growth_power_multiple_enemies(self):
        """GrowthPower should work independently per enemy."""
        state = _make_state(enemies=[
            EnemyCombatState(hp=30, max_hp=30, id="e1"),
            EnemyCombatState(hp=30, max_hp=30, id="e2"),
        ])
        state.enemies[0].statuses["GrowthPower"] = 2
        state.enemies[1].statuses["GrowthPower"] = 5

        # Round 1: both skip
        for e in state.enemies:
            execute_power_triggers("atEndOfRound", state, e)
        assert state.enemies[0].statuses.get("Strength", 0) == 0
        assert state.enemies[1].statuses.get("Strength", 0) == 0

        # Round 2: both gain
        for e in state.enemies:
            execute_power_triggers("atEndOfRound", state, e)
        assert state.enemies[0].statuses.get("Strength") == 2
        assert state.enemies[1].statuses.get("Strength") == 5

    def test_repair_multiple_stacks(self):
        """Repair with high stacks should still be capped at max HP."""
        state = _make_state(player_hp=1, player_max_hp=50)
        state.player.statuses["Repair"] = 100
        execute_power_triggers("onVictory", state, state.player)
        assert state.player.hp == 50
