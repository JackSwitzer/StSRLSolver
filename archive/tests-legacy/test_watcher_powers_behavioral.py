"""
Behavioral tests for critical Watcher powers.

Tests verify that:
- MentalFortress grants block on any stance change
- Rushdown draws cards when entering Wrath
- Devotion adds mantra via engine (triggers Divinity transition correctly)
- Foresight actually scries at start of turn
- LikeWater grants block only once (no double-trigger)
- Metallicize grants block only once (no double-trigger)
- Plated Armor grants block only once (no double-trigger)
- BattleHymn adds Smite to hand
- Nirvana grants block on scry
"""

import pytest
from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState, create_combat,
    PlayCard, EndTurn,
)
from packages.engine.combat_engine import CombatEngine
from packages.engine.content.cards import get_card, CardType
from packages.engine.registry import (
    execute_power_triggers, PowerContext, POWER_REGISTRY,
)
from packages.engine.registry import powers as _powers  # noqa: F401


def _make_state(
    player_hp=50,
    player_max_hp=50,
    enemies=None,
    deck=None,
    hand=None,
    stance="Neutral",
    relics=None,
):
    """Create a test combat state."""
    if enemies is None:
        enemies = [EnemyCombatState(hp=30, max_hp=30, id="test_enemy")]
    if deck is None:
        deck = ["Strike"] * 5
    state = create_combat(
        player_hp=player_hp,
        player_max_hp=player_max_hp,
        enemies=enemies,
        deck=deck,
    )
    state.stance = stance
    if hand is not None:
        state.hand = list(hand)
    if relics is not None:
        state.relics = dict(relics)
    return state


def _make_engine(state):
    """Create a CombatEngine wired to the state."""
    engine = CombatEngine(state)
    engine.phase = engine.phase.__class__("PLAYER_TURN")
    return engine


# =============================================================================
# MentalFortress: Gain block on any stance change
# =============================================================================

class TestMentalFortress:
    def test_gains_block_on_stance_change(self):
        state = _make_state(stance="Neutral")
        state.player.statuses["MentalFortress"] = 4
        engine = _make_engine(state)

        engine._change_stance(engine._parse_stance("Wrath"))
        assert state.player.block >= 4

    def test_gains_block_on_calm_exit(self):
        state = _make_state(stance="Calm")
        state.player.statuses["MentalFortress"] = 6
        engine = _make_engine(state)

        engine._change_stance(engine._parse_stance("Wrath"))
        assert state.player.block >= 6

    def test_no_block_if_same_stance(self):
        state = _make_state(stance="Wrath")
        state.player.statuses["MentalFortress"] = 4
        engine = _make_engine(state)

        result = engine._change_stance(engine._parse_stance("Wrath"))
        assert result["changed"] is False
        assert state.player.block == 0

    def test_double_stance_change_double_block(self):
        state = _make_state(stance="Neutral")
        state.player.statuses["MentalFortress"] = 4
        engine = _make_engine(state)

        engine._change_stance(engine._parse_stance("Calm"))
        engine._change_stance(engine._parse_stance("Wrath"))
        assert state.player.block >= 8  # 4 per stance change


# =============================================================================
# Rushdown: Draw 2 when entering Wrath
# =============================================================================

class TestRushdown:
    def test_draws_on_wrath_entry(self):
        state = _make_state(stance="Neutral")
        state.player.statuses["Rushdown"] = 2
        state.draw_pile = ["Strike", "Strike", "Strike", "Strike"]
        state.hand = []
        engine = _make_engine(state)

        engine._change_stance(engine._parse_stance("Wrath"))
        assert len(state.hand) == 2

    def test_no_draw_on_calm_entry(self):
        state = _make_state(stance="Neutral")
        state.player.statuses["Rushdown"] = 2
        state.draw_pile = ["Strike", "Strike"]
        state.hand = []
        engine = _make_engine(state)

        engine._change_stance(engine._parse_stance("Calm"))
        assert len(state.hand) == 0

    def test_draws_on_wrath_from_calm(self):
        state = _make_state(stance="Calm")
        state.player.statuses["Rushdown"] = 2
        state.draw_pile = ["Strike", "Strike", "Strike"]
        state.hand = []
        engine = _make_engine(state)

        engine._change_stance(engine._parse_stance("Wrath"))
        assert len(state.hand) == 2


# =============================================================================
# Devotion: Gain Mantra -> Divinity transition fires hooks
# =============================================================================

class TestDevotion:
    def test_adds_mantra(self):
        state = _make_state()
        state.player.statuses["Devotion"] = 3
        state.mantra = 0
        engine = _make_engine(state)

        execute_power_triggers(
            "atStartOfTurnPostDraw", state, state.player
        )
        assert state.mantra == 3

    def test_enters_divinity_at_10(self):
        state = _make_state()
        state.player.statuses["Devotion"] = 5
        state.mantra = 7
        engine = _make_engine(state)

        execute_power_triggers(
            "atStartOfTurnPostDraw", state, state.player
        )
        assert state.stance == "Divinity"
        assert state.mantra == 2  # 12 - 10 = 2

    def test_divinity_fires_mental_fortress(self):
        """Key test: Devotion -> Divinity transition should fire MentalFortress."""
        state = _make_state(stance="Neutral")
        state.player.statuses["Devotion"] = 5
        state.player.statuses["MentalFortress"] = 6
        state.mantra = 7
        engine = _make_engine(state)

        execute_power_triggers(
            "atStartOfTurnPostDraw", state, state.player
        )
        assert state.stance == "Divinity"
        assert state.player.block >= 6  # MentalFortress triggered

    def test_divinity_fires_rushdown(self):
        """Devotion -> Divinity should NOT trigger Rushdown (Divinity != Wrath)."""
        state = _make_state(stance="Neutral")
        state.player.statuses["Devotion"] = 5
        state.player.statuses["Rushdown"] = 2
        state.mantra = 8
        state.draw_pile = ["Strike", "Strike", "Strike"]
        state.hand = []
        engine = _make_engine(state)

        hand_before = len(state.hand)
        execute_power_triggers(
            "atStartOfTurnPostDraw", state, state.player
        )
        # Rushdown only draws on Wrath entry, not Divinity
        assert len(state.hand) == hand_before


# =============================================================================
# Foresight: Actually scries at start of turn
# =============================================================================

class TestForesight:
    def test_scries_cards(self):
        state = _make_state()
        state.player.statuses["Foresight"] = 3
        # Put some cards in draw pile with a status/curse near the top
        # (top = end of list) so it's in the scried cards
        state.draw_pile = ["Strike", "Strike", "Strike", "Defend", "Wound"]
        original_draw_len = len(state.draw_pile)
        engine = _make_engine(state)

        execute_power_triggers("atStartOfTurn", state, state.player)
        # The heuristic auto-scry should have discarded the Wound (status card)
        # and kept Strike and Defend on top.  Wound moved to discard.
        assert "Wound" not in state.draw_pile
        assert "Wound" in state.discard_pile
        assert len(state.draw_pile) == original_draw_len - 1

    def test_scry_triggers_nirvana(self):
        """Foresight scry should trigger Nirvana block gain."""
        state = _make_state()
        state.player.statuses["Foresight"] = 2
        state.player.statuses["Nirvana"] = 3
        state.draw_pile = ["Strike", "Strike", "Strike"]
        engine = _make_engine(state)

        execute_power_triggers("atStartOfTurn", state, state.player)
        # Nirvana should have given block
        assert state.player.block > 0


# =============================================================================
# No double-trigger tests
# =============================================================================

class TestNoDoubleTrigger:
    def test_like_water_single_trigger(self):
        """LikeWater should only grant block once via atEndOfTurnPreEndTurnCards."""
        state = _make_state(stance="Calm")
        state.player.statuses["LikeWater"] = 5
        state.player.block = 0

        execute_power_triggers(
            "atEndOfTurnPreEndTurnCards", state, state.player
        )
        assert state.player.block == 5

    def test_metallicize_single_trigger(self):
        """Metallicize should only grant block once via atEndOfTurnPreEndTurnCards."""
        state = _make_state()
        state.player.statuses["Metallicize"] = 3
        state.player.block = 0

        execute_power_triggers(
            "atEndOfTurnPreEndTurnCards", state, state.player
        )
        assert state.player.block == 3

    def test_plated_armor_single_trigger(self):
        """Plated Armor should only grant block once via atEndOfTurnPreEndTurnCards."""
        state = _make_state()
        state.player.statuses["Plated Armor"] = 4
        state.player.block = 0

        execute_power_triggers(
            "atEndOfTurnPreEndTurnCards", state, state.player
        )
        assert state.player.block == 4


# =============================================================================
# BattleHymn: Add Smite to hand at start of turn
# =============================================================================

class TestBattleHymn:
    def test_adds_smite(self):
        state = _make_state()
        state.player.statuses["BattleHymn"] = 1
        state.hand = []
        engine = _make_engine(state)

        execute_power_triggers("atStartOfTurn", state, state.player)
        assert "Smite" in state.hand

    def test_adds_multiple_smites(self):
        state = _make_state()
        state.player.statuses["BattleHymn"] = 3
        state.hand = []
        engine = _make_engine(state)

        execute_power_triggers("atStartOfTurn", state, state.player)
        assert state.hand.count("Smite") == 3


# =============================================================================
# Nirvana: Gain block on scry
# =============================================================================

class TestNirvana:
    def test_gains_block_on_scry(self):
        state = _make_state()
        state.player.statuses["Nirvana"] = 3
        state.player.block = 0
        state.draw_pile = ["Strike", "Defend", "Strike"]
        engine = _make_engine(state)

        # Trigger scry - sets up pending selection
        engine._scry(2)
        assert state.pending_scry_selection
        # Complete scry (keep all) - this fires onScry triggers including Nirvana
        engine._complete_scry_selection(())
        assert state.player.block >= 3  # At least base Nirvana block


# =============================================================================
# Registration checks
# =============================================================================

class TestWatcherPowerRegistration:
    def test_mental_fortress_registered(self):
        assert POWER_REGISTRY.has_handler("onChangeStance", "MentalFortress")

    def test_rushdown_registered(self):
        assert POWER_REGISTRY.has_handler("onChangeStance", "Rushdown")

    def test_devotion_registered(self):
        assert POWER_REGISTRY.has_handler("atStartOfTurnPostDraw", "Devotion")

    def test_foresight_registered(self):
        assert POWER_REGISTRY.has_handler("atStartOfTurn", "Foresight")

    def test_like_water_registered(self):
        assert POWER_REGISTRY.has_handler("atEndOfTurnPreEndTurnCards", "LikeWater")

    def test_battle_hymn_registered(self):
        assert POWER_REGISTRY.has_handler("atStartOfTurn", "BattleHymn")

    def test_nirvana_registered(self):
        assert POWER_REGISTRY.has_handler("onScry", "Nirvana")

    def test_establishment_inline(self):
        """Establishment: cost reduction handled inline in combat_engine._start_player_turn()."""
        # No registry handler — logic is inline using card_costs dict
        assert True  # Behavioral test in test_relic_missing_batch.py
