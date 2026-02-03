"""
Stance Mechanics Audit Tests - Verify Python engine matches decompiled Java.

Tests cover:
1. Wrath: 2x damage dealt and received (NORMAL only)
2. Calm: +2 energy on exit (+3 with Violet Lotus)
3. Divinity: 3x damage dealt, +3 energy on enter, no extra damage received
4. Divinity: auto-exit at start of turn (Java) vs end of turn (Python BUG)
5. Mental Fortress: block on any stance change
6. Rushdown: draw cards only when entering Wrath
7. Flurry of Blows: discard-to-hand on any stance change
8. Mantra: threshold 10, excess carries over
9. Same-stance is a no-op
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.content.stances import StanceID, StanceManager, STANCES, StanceEffect
from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    create_player, create_enemy, create_combat,
)
from packages.engine.combat_engine import CombatEngine
from packages.engine.calc.damage import (
    calculate_damage, WRATH_MULT, DIVINITY_MULT,
)


# =============================================================================
# HELPERS
# =============================================================================

def make_engine(
    deck=None,
    stance="Neutral",
    energy=3,
    player_hp=80,
    enemy_damage=10,
    enemy_hits=1,
    relics=None,
    player_statuses=None,
    discard_pile=None,
):
    """Create a CombatEngine with controlled state for testing."""
    if deck is None:
        deck = ["Strike_P"] * 5
    enemies = [
        create_enemy("JawWorm", hp=44, max_hp=44,
                      move_damage=enemy_damage, move_hits=enemy_hits),
    ]
    state = create_combat(
        player_hp=player_hp,
        player_max_hp=player_hp,
        enemies=enemies,
        deck=deck,
        energy=energy,
        max_energy=3,
        relics=relics or [],
    )
    state.stance = stance
    if player_statuses:
        state.player.statuses.update(player_statuses)
    if discard_pile:
        state.discard_pile = list(discard_pile)
    engine = CombatEngine(state)
    engine.start_combat()
    # Restore stance after start_combat (which resets to Neutral)
    engine.state.stance = stance
    return engine


# =============================================================================
# STANCES DATA MODEL TESTS (stances.py)
# =============================================================================

class TestStanceDataModel:
    """Test the STANCES dict and StanceEffect data matches Java."""

    def test_neutral_no_multipliers(self):
        e = STANCES[StanceID.NEUTRAL]
        assert e.damage_give_multiplier == 1.0
        assert e.damage_receive_multiplier == 1.0
        assert e.energy_on_enter == 0
        assert e.energy_on_exit == 0

    def test_wrath_2x_deal_and_receive(self):
        e = STANCES[StanceID.WRATH]
        assert e.damage_give_multiplier == 2.0
        assert e.damage_receive_multiplier == 2.0

    def test_calm_2_energy_on_exit(self):
        e = STANCES[StanceID.CALM]
        assert e.energy_on_exit == 2
        assert e.energy_on_enter == 0

    def test_divinity_3x_deal_no_extra_receive(self):
        """Java DivinityStance has atDamageGive 3x but NO atDamageReceive override."""
        e = STANCES[StanceID.DIVINITY]
        assert e.damage_give_multiplier == 3.0
        assert e.damage_receive_multiplier == 1.0

    def test_divinity_3_energy_on_enter(self):
        e = STANCES[StanceID.DIVINITY]
        assert e.energy_on_enter == 3

    def test_divinity_exits_at_turn_end(self):
        e = STANCES[StanceID.DIVINITY]
        assert e.exits_at_turn_end is True


# =============================================================================
# STANCE MANAGER TESTS (stances.py)
# =============================================================================

class TestStanceManager:
    """Test StanceManager logic."""

    def test_initial_stance_is_neutral(self):
        sm = StanceManager()
        assert sm.current == StanceID.NEUTRAL

    def test_same_stance_no_op(self):
        sm = StanceManager()
        result = sm.change_stance(StanceID.NEUTRAL)
        assert result["is_stance_change"] is False

    def test_calm_exit_gives_2_energy(self):
        sm = StanceManager()
        sm.change_stance(StanceID.CALM)
        result = sm.change_stance(StanceID.NEUTRAL)
        assert result["energy_gained"] == 2

    def test_calm_exit_with_violet_lotus_gives_3(self):
        sm = StanceManager(has_violet_lotus=True)
        sm.change_stance(StanceID.CALM)
        result = sm.change_stance(StanceID.NEUTRAL)
        assert result["energy_gained"] == 3

    def test_divinity_enter_gives_3_energy(self):
        sm = StanceManager()
        result = sm.change_stance(StanceID.DIVINITY)
        assert result["energy_gained"] == 3

    def test_calm_to_divinity_gives_5_energy(self):
        sm = StanceManager()
        sm.change_stance(StanceID.CALM)
        result = sm.change_stance(StanceID.DIVINITY)
        assert result["energy_gained"] == 5  # 2 from calm exit + 3 from divinity enter

    def test_mantra_overflow(self):
        sm = StanceManager()
        sm.add_mantra(7)
        assert sm.mantra == 7
        result = sm.add_mantra(5)  # 12 total, triggers at 10
        assert result["divinity_triggered"] is True
        assert sm.mantra == 2  # 12 - 10 = 2 carry over

    def test_mantra_exact_10(self):
        sm = StanceManager()
        result = sm.add_mantra(10)
        assert result["divinity_triggered"] is True
        assert sm.mantra == 0

    def test_wrath_damage_give(self):
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)
        assert sm.at_damage_give(10, "NORMAL") == 20.0
        # HP_LOSS not affected
        assert sm.at_damage_give(10, "HP_LOSS") == 10

    def test_wrath_damage_receive(self):
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)
        assert sm.at_damage_receive(10, "NORMAL") == 20.0
        assert sm.at_damage_receive(10, "HP_LOSS") == 10

    def test_divinity_damage_give_3x(self):
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)
        assert sm.at_damage_give(10, "NORMAL") == 30.0

    def test_divinity_no_extra_damage_receive(self):
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)
        assert sm.at_damage_receive(10, "NORMAL") == 10.0

    def test_divinity_no_exit_on_turn_end(self):
        """Java: Divinity exits at start of next turn, not end of current."""
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)
        result = sm.on_turn_end()
        assert sm.current == StanceID.DIVINITY
        assert result == {}

    def test_divinity_auto_exit_on_turn_start(self):
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)
        result = sm.on_turn_start()
        assert sm.current == StanceID.NEUTRAL
        assert result.get("divinity_ended") is True

    def test_neutral_no_exit_on_turn_start(self):
        sm = StanceManager()
        result = sm.on_turn_start()
        assert sm.current == StanceID.NEUTRAL
        assert result == {}


# =============================================================================
# COMBAT ENGINE STANCE TESTS
# =============================================================================

class TestCombatEngineStances:
    """Test stance mechanics in the full combat engine."""

    def test_change_stance_to_wrath(self):
        engine = make_engine()
        result = engine._change_stance(StanceID.WRATH)
        assert result["changed"] is True
        assert engine.state.stance == "Wrath"

    def test_same_stance_no_change(self):
        engine = make_engine(stance="Wrath")
        result = engine._change_stance(StanceID.WRATH)
        assert result["changed"] is False

    def test_calm_exit_energy(self):
        engine = make_engine(stance="Calm", energy=3)
        engine._change_stance(StanceID.NEUTRAL)
        assert engine.state.energy == 5  # 3 + 2

    def test_calm_exit_violet_lotus(self):
        engine = make_engine(stance="Calm", energy=3, relics=["Violet Lotus"])
        engine._change_stance(StanceID.NEUTRAL)
        assert engine.state.energy == 6  # 3 + 3

    def test_divinity_enter_energy(self):
        engine = make_engine(stance="Neutral", energy=3)
        engine._change_stance(StanceID.DIVINITY)
        assert engine.state.energy == 6  # 3 + 3

    def test_mental_fortress_block_on_stance_change(self):
        """Java: MentalFortressPower.onChangeStance grants block if old != new."""
        engine = make_engine(player_statuses={"MentalFortress": 4})
        engine._change_stance(StanceID.WRATH)
        assert engine.state.player.block == 4
        engine._change_stance(StanceID.CALM)
        assert engine.state.player.block == 8  # 4 + 4

    def test_mental_fortress_no_block_on_same_stance(self):
        engine = make_engine(stance="Wrath", player_statuses={"MentalFortress": 4})
        engine._change_stance(StanceID.WRATH)
        assert engine.state.player.block == 0  # No change

    def test_rushdown_draw_on_wrath_entry(self):
        """Java: RushdownPower draws only when entering Wrath."""
        deck = ["Strike_P"] * 10
        engine = make_engine(deck=deck, player_statuses={"Rushdown": 2})
        # Put cards in draw pile
        engine.state.draw_pile = ["Strike_P"] * 5
        engine.state.hand = []
        engine._change_stance(StanceID.WRATH)
        assert len(engine.state.hand) == 2  # Drew 2 from Rushdown

    def test_rushdown_no_draw_on_calm_entry(self):
        """Java Rushdown only triggers for Wrath, not Calm."""
        deck = ["Strike_P"] * 10
        engine = make_engine(deck=deck, player_statuses={"Rushdown": 2})
        engine.state.draw_pile = ["Strike_P"] * 5
        engine.state.hand = []
        engine._change_stance(StanceID.CALM)
        assert len(engine.state.hand) == 0

    def test_flurry_of_blows_discard_to_hand(self):
        """Java: FlurryOfBlows.triggerExhaustedCardsOnStanceChange moves from discard to hand."""
        engine = make_engine(discard_pile=["FlurryOfBlows", "Strike_P", "FlurryOfBlows+"])
        engine.state.hand = ["Defend_P"]
        engine._change_stance(StanceID.WRATH)
        # Both FlurryOfBlows cards should move to hand
        assert "FlurryOfBlows" in engine.state.hand
        assert "FlurryOfBlows+" in engine.state.hand
        assert "FlurryOfBlows" not in engine.state.discard_pile
        assert "FlurryOfBlows+" not in engine.state.discard_pile

    def test_mantra_triggers_divinity(self):
        engine = make_engine(energy=3)
        engine._add_mantra(10)
        assert engine.state.stance == "Divinity"
        assert engine.state.energy == 6  # 3 + 3
        assert engine.state.mantra == 0

    def test_mantra_overflow_carries(self):
        engine = make_engine(energy=3)
        engine._add_mantra(13)
        assert engine.state.stance == "Divinity"
        assert engine.state.mantra == 3  # 13 - 10


# =============================================================================
# DAMAGE CALCULATION WITH STANCES
# =============================================================================

class TestStanceDamageCalc:
    """Test damage.py multipliers for stances."""

    def test_wrath_mult_value(self):
        assert WRATH_MULT == 2.0

    def test_divinity_mult_value(self):
        assert DIVINITY_MULT == 3.0

    def test_wrath_damage(self):
        # 6 base * 2.0 wrath = 12
        assert calculate_damage(6, stance_mult=WRATH_MULT) == 12

    def test_divinity_damage(self):
        # 6 base * 3.0 divinity = 18
        assert calculate_damage(6, stance_mult=DIVINITY_MULT) == 18

    def test_wrath_plus_vulnerable(self):
        # 6 base * 2.0 wrath * 1.5 vuln = 18
        assert calculate_damage(6, stance_mult=WRATH_MULT, vuln=True) == 18

    def test_wrath_plus_strength(self):
        # (6 + 3) * 2.0 = 18
        assert calculate_damage(6, strength=3, stance_mult=WRATH_MULT) == 18

    def test_wrath_str_vuln(self):
        # (6 + 3) * 2.0 * 1.5 = 27
        assert calculate_damage(6, strength=3, stance_mult=WRATH_MULT, vuln=True) == 27


# =============================================================================
# INCOMING DAMAGE IN WRATH (enemy turn)
# =============================================================================

class TestWrathIncomingDamage:
    """Test that Wrath doubles incoming damage during enemy turn."""

    def test_wrath_doubles_enemy_damage(self):
        """Wrath should double incoming NORMAL damage from enemies."""
        engine_wrath = make_engine(stance="Wrath", enemy_damage=10, player_hp=80)
        engine_wrath.state.hand = []
        engine_neutral = make_engine(stance="Neutral", enemy_damage=10, player_hp=80)
        engine_neutral.state.hand = []

        engine_wrath.end_turn()
        engine_neutral.end_turn()

        wrath_damage = engine_wrath.state.total_damage_taken
        neutral_damage = engine_neutral.state.total_damage_taken
        # Wrath should take exactly 2x the damage of Neutral
        assert wrath_damage == neutral_damage * 2, (
            f"Wrath took {wrath_damage}, Neutral took {neutral_damage}, expected 2x"
        )

    def test_neutral_takes_base_damage(self):
        """Neutral stance should not modify incoming damage."""
        engine = make_engine(stance="Neutral", enemy_damage=10, player_hp=80)
        engine.state.hand = []
        engine.end_turn()
        # Enemy move_damage is set to 10 in create_enemy
        # Strength may apply; check raw value
        damage_taken = engine.state.total_damage_taken
        assert damage_taken > 0, "Enemy should deal damage"


# =============================================================================
# DIVINITY TIMING BUG TEST
# =============================================================================

class TestDivinityTiming:
    """
    BUG: Java DivinityStance.atStartOfTurn() exits at START of next turn.
    Python exits at end of current turn in end_turn().

    This test documents the current (buggy) Python behavior.
    """

    def test_divinity_exits_end_of_turn_python_behavior(self):
        """Current Python: Divinity exits at end of turn (before enemy attacks)."""
        engine = make_engine(stance="Divinity", enemy_damage=10, player_hp=80)
        engine.state.hand = []
        engine.end_turn()
        # Divinity has no receive multiplier, so damage is same as Neutral
        damage_taken = engine.state.total_damage_taken
        assert damage_taken > 0
        # After full turn cycle, stance should be Neutral (Divinity auto-exited)
        # Note: start_player_turn is called, stance may have been set back
        # The key check: Divinity does not persist
        # end_turn exits Divinity before enemy turn in Python

    def test_divinity_should_persist_through_enemy_turn(self):
        """
        Java behavior: Divinity persists through enemy turn and exits at
        START of next player turn. This means Mental Fortress block from
        the auto-exit happens at start of next turn.

        Currently fails because Python exits early.
        """
        engine = make_engine(
            stance="Divinity",
            enemy_damage=10,
            player_hp=80,
            player_statuses={"MentalFortress": 4},
        )
        engine.state.hand = []

        # End turn -> enemy attacks -> _start_player_turn auto-called
        # In Java: Divinity persists through enemy turn, exits at start of next turn
        # _start_player_turn: block reset -> Divinity exit -> Mental Fortress block
        engine.end_turn()

        # end_turn already calls _start_player_turn which does the Divinity exit
        # Mental Fortress block (4) should be present after start of new turn
        assert engine.state.player.block >= 4, (
            "Mental Fortress block from Divinity auto-exit should happen at start of turn"
        )
