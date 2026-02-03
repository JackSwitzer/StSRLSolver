"""
Tests for Relic Registry Integration with CombatRunner.

Verifies that the registry-based relic trigger system properly integrates
with the combat engine at all trigger points.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.registry import (
    execute_relic_triggers,
    RELIC_REGISTRY,
    RelicContext,
)
from packages.engine.state.combat import (
    CombatState,
    EntityState,
    EnemyCombatState,
    create_combat,
    create_enemy,
)
from packages.engine.content.cards import ALL_CARDS, CardType


# =============================================================================
# TEST HELPERS
# =============================================================================

def create_test_combat(
    player_hp: int = 70,
    max_hp: int = 80,
    enemies: list = None,
    relics: list = None,
    energy: int = 3,
    hand: list = None,
    deck: list = None,
) -> CombatState:
    """Create a combat state for testing."""
    if enemies is None:
        enemies = [create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)]
    if relics is None:
        relics = []
    if hand is None:
        hand = ["Strike"]
    if deck is None:
        deck = ["Strike", "Strike", "Defend", "Defend"]

    state = create_combat(
        player_hp=player_hp,
        player_max_hp=max_hp,
        enemies=enemies,
        deck=deck,
        energy=energy,
        relics=relics,
    )
    state.hand = hand
    return state


# =============================================================================
# AT_BATTLE_START Tests
# =============================================================================

class TestAtBattleStartTriggers:
    """Test atBattleStart relic triggers via registry."""

    def test_vajra_applies_strength(self):
        """Vajra should apply 1 Strength at battle start."""
        state = create_test_combat(relics=["Vajra"])
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Strength", 0) == 1

    def test_anchor_applies_block(self):
        """Anchor should apply 10 Block at battle start."""
        state = create_test_combat(relics=["Anchor"])
        execute_relic_triggers("atBattleStart", state)
        assert state.player.block == 10

    def test_bag_of_marbles_applies_vulnerable(self):
        """Bag of Marbles should apply 1 Vulnerable to all enemies."""
        state = create_test_combat(
            relics=["Bag of Marbles"],
            enemies=[
                create_enemy("E1", hp=30, max_hp=30),
                create_enemy("E2", hp=40, max_hp=40),
            ]
        )
        execute_relic_triggers("atBattleStart", state)
        for enemy in state.enemies:
            assert enemy.statuses.get("Vulnerable", 0) == 1

    def test_akabeko_applies_vigor(self):
        """Akabeko should apply 8 Vigor at battle start."""
        state = create_test_combat(relics=["Akabeko"])
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Vigor", 0) == 8

    def test_blood_vial_heals(self):
        """Blood Vial should heal 2 HP at battle start."""
        state = create_test_combat(player_hp=70, max_hp=80, relics=["Blood Vial"])
        execute_relic_triggers("atBattleStart", state)
        assert state.player.hp == 72

    def test_bronze_scales_applies_thorns(self):
        """Bronze Scales should apply 3 Thorns at battle start."""
        state = create_test_combat(relics=["Bronze Scales"])
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Thorns", 0) == 3

    def test_fossilized_helix_applies_buffer(self):
        """Fossilized Helix should apply 1 Buffer at battle start."""
        state = create_test_combat(relics=["FossilizedHelix"])
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Buffer", 0) == 1

    def test_oddly_smooth_stone_applies_dexterity(self):
        """Oddly Smooth Stone should apply 1 Dexterity at battle start."""
        state = create_test_combat(relics=["Oddly Smooth Stone"])
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Dexterity", 0) == 1

    def test_thread_and_needle_applies_plated_armor(self):
        """Thread and Needle should apply 4 Plated Armor at battle start."""
        state = create_test_combat(relics=["Thread and Needle"])
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Plated Armor", 0) == 4


# =============================================================================
# AT_TURN_START Tests
# =============================================================================

class TestAtTurnStartTriggers:
    """Test atTurnStart relic triggers via registry."""

    def test_lantern_gives_energy_turn_1(self):
        """Lantern should give +1 energy on turn 1."""
        state = create_test_combat(relics=["Lantern"], energy=3)
        state.turn = 1
        execute_relic_triggers("atTurnStart", state)
        assert state.energy == 4

    def test_lantern_no_energy_turn_2(self):
        """Lantern should not give energy on turn 2+."""
        state = create_test_combat(relics=["Lantern"], energy=3)
        state.turn = 2
        execute_relic_triggers("atTurnStart", state)
        assert state.energy == 3

    def test_horn_cleat_gives_block_turn_2(self):
        """Horn Cleat should give 14 block on turn 2."""
        state = create_test_combat(relics=["HornCleat"])
        state.turn = 2
        execute_relic_triggers("atTurnStart", state)
        assert state.player.block == 14

    def test_horn_cleat_no_block_turn_1(self):
        """Horn Cleat should not give block on turn 1."""
        state = create_test_combat(relics=["HornCleat"])
        state.turn = 1
        execute_relic_triggers("atTurnStart", state)
        assert state.player.block == 0

    def test_happy_flower_counter_increments(self):
        """Happy Flower should increment counter and give energy every 3 turns."""
        state = create_test_combat(relics=["Happy Flower"], energy=3)
        state.set_relic_counter("Happy Flower", 0)

        # Turn 1
        execute_relic_triggers("atTurnStart", state)
        assert state.get_relic_counter("Happy Flower") == 1
        assert state.energy == 3

        # Turn 2
        execute_relic_triggers("atTurnStart", state)
        assert state.get_relic_counter("Happy Flower") == 2
        assert state.energy == 3

        # Turn 3 - should trigger
        execute_relic_triggers("atTurnStart", state)
        assert state.get_relic_counter("Happy Flower") == 0
        assert state.energy == 4

    def test_art_of_war_gives_energy_no_attacks(self):
        """Art of War should give +1 energy if no attacks played last turn."""
        state = create_test_combat(relics=["Art of War"], energy=3)
        state.set_relic_counter("Art of War", 0)  # 0 = no attacks last turn
        execute_relic_triggers("atTurnStart", state)
        assert state.energy == 4

    def test_art_of_war_no_energy_with_attacks(self):
        """Art of War should not give energy if attacks were played."""
        state = create_test_combat(relics=["Art of War"], energy=3)
        state.set_relic_counter("Art of War", 1)  # 1 = attacks played
        execute_relic_triggers("atTurnStart", state)
        assert state.energy == 3


# =============================================================================
# ON_PLAYER_END_TURN Tests
# =============================================================================

class TestOnPlayerEndTurnTriggers:
    """Test onPlayerEndTurn relic triggers via registry."""

    def test_orichalcum_gives_block_when_zero(self):
        """Orichalcum should give 6 block if player has 0 block."""
        state = create_test_combat(relics=["Orichalcum"])
        state.player.block = 0
        execute_relic_triggers("onPlayerEndTurn", state)
        assert state.player.block == 6

    def test_orichalcum_no_block_when_positive(self):
        """Orichalcum should not give block if player has any block."""
        state = create_test_combat(relics=["Orichalcum"])
        state.player.block = 5
        execute_relic_triggers("onPlayerEndTurn", state)
        assert state.player.block == 5

    def test_stone_calendar_damage_turn_7(self):
        """Stone Calendar should deal 52 damage to all enemies on turn 7."""
        state = create_test_combat(
            relics=["StoneCalendar"],
            enemies=[
                create_enemy("E1", hp=100, max_hp=100),
                create_enemy("E2", hp=100, max_hp=100),
            ]
        )
        state.turn = 7
        execute_relic_triggers("onPlayerEndTurn", state)
        for enemy in state.enemies:
            assert enemy.hp == 48  # 100 - 52 = 48

    def test_stone_calendar_no_damage_other_turns(self):
        """Stone Calendar should not deal damage on turns other than 7."""
        state = create_test_combat(
            relics=["StoneCalendar"],
            enemies=[create_enemy("E1", hp=100, max_hp=100)]
        )
        state.turn = 6
        execute_relic_triggers("onPlayerEndTurn", state)
        assert state.enemies[0].hp == 100

    def test_art_of_war_tracks_attacks(self):
        """Art of War should track whether attacks were played."""
        state = create_test_combat(relics=["Art of War"])
        state.attacks_played_this_turn = 2
        execute_relic_triggers("onPlayerEndTurn", state)
        assert state.get_relic_counter("Art of War") == 1

        state.attacks_played_this_turn = 0
        execute_relic_triggers("onPlayerEndTurn", state)
        assert state.get_relic_counter("Art of War") == 0


# =============================================================================
# WAS_HP_LOST Tests
# =============================================================================

class TestWasHPLostTriggers:
    """Test wasHPLost relic triggers via registry."""

    def test_centennial_puzzle_draws_once(self):
        """Centennial Puzzle should draw 3 cards first time HP is lost."""
        state = create_test_combat(relics=["Centennial Puzzle"])
        state.draw_pile = ["Card1", "Card2", "Card3", "Card4"]
        state.hand = []
        state.set_relic_counter("Centennial Puzzle", 0)

        execute_relic_triggers("wasHPLost", state, {"hp_lost": 5})

        assert len(state.hand) == 3
        assert state.get_relic_counter("Centennial Puzzle") == 1

    def test_centennial_puzzle_only_once(self):
        """Centennial Puzzle should not draw on subsequent HP loss."""
        state = create_test_combat(relics=["Centennial Puzzle"])
        state.draw_pile = ["Card1", "Card2", "Card3"]
        state.hand = []
        state.set_relic_counter("Centennial Puzzle", 1)  # Already triggered

        execute_relic_triggers("wasHPLost", state, {"hp_lost": 5})

        assert len(state.hand) == 0  # No cards drawn

    def test_runic_cube_draws_card(self):
        """Runic Cube should draw 1 card when HP is lost."""
        state = create_test_combat(relics=["Runic Cube"])
        state.draw_pile = ["Card1", "Card2"]
        state.hand = []

        execute_relic_triggers("wasHPLost", state, {"hp_lost": 5})

        assert len(state.hand) == 1

    def test_self_forming_clay_next_turn_block(self):
        """Self-Forming Clay should give NextTurnBlock when HP is lost."""
        state = create_test_combat(relics=["Self Forming Clay"])

        execute_relic_triggers("wasHPLost", state, {"hp_lost": 5})

        assert state.player.statuses.get("NextTurnBlock", 0) == 3

    def test_red_skull_strength_when_bloodied(self):
        """Red Skull should give 3 Strength when HP drops to 50% or below."""
        state = create_test_combat(relics=["Red Skull"], player_hp=40, max_hp=80)
        state.set_relic_counter("Red Skull", 0)

        execute_relic_triggers("wasHPLost", state, {"hp_lost": 5})

        assert state.player.statuses.get("Strength", 0) == 3
        assert state.get_relic_counter("Red Skull") == 1


# =============================================================================
# ON_PLAY_CARD Tests
# =============================================================================

class TestOnPlayCardTriggers:
    """Test onPlayCard relic triggers via registry."""

    def test_shuriken_counter_attacks(self):
        """Shuriken should count attacks and give Strength every 3."""
        state = create_test_combat(relics=["Shuriken"])
        state.set_relic_counter("Shuriken", 0)

        # Get an attack card (use color suffix)
        attack_card = ALL_CARDS.get("Strike_P")  # Watcher Strike
        assert attack_card is not None, "Strike_P card not found"
        assert attack_card.card_type == CardType.ATTACK

        # Play 3 attacks
        for i in range(3):
            execute_relic_triggers("onPlayCard", state, {"card": attack_card})

        assert state.player.statuses.get("Strength", 0) == 1
        assert state.get_relic_counter("Shuriken") == 0  # Reset after triggering

    def test_kunai_counter_attacks(self):
        """Kunai should count attacks and give Dexterity every 3."""
        state = create_test_combat(relics=["Kunai"])
        state.set_relic_counter("Kunai", 0)

        attack_card = ALL_CARDS.get("Strike_P")

        for i in range(3):
            execute_relic_triggers("onPlayCard", state, {"card": attack_card})

        assert state.player.statuses.get("Dexterity", 0) == 1

    def test_nunchaku_counter_attacks(self):
        """Nunchaku should count attacks and give energy every 10."""
        state = create_test_combat(relics=["Nunchaku"], energy=3)
        state.set_relic_counter("Nunchaku", 0)

        attack_card = ALL_CARDS.get("Strike_P")

        for i in range(10):
            execute_relic_triggers("onPlayCard", state, {"card": attack_card})

        assert state.energy == 4

    def test_ornamental_fan_counter_attacks(self):
        """Ornamental Fan should count attacks and give block every 3."""
        state = create_test_combat(relics=["Ornamental Fan"])
        state.set_relic_counter("Ornamental Fan", 0)

        attack_card = ALL_CARDS.get("Strike_P")

        for i in range(3):
            execute_relic_triggers("onPlayCard", state, {"card": attack_card})

        assert state.player.block == 4

    def test_ink_bottle_counter_all_cards(self):
        """Ink Bottle should count all cards and draw every 10."""
        state = create_test_combat(relics=["InkBottle"])
        state.set_relic_counter("InkBottle", 0)
        state.draw_pile = ["Card1", "Card2"]
        state.hand = []

        attack_card = ALL_CARDS.get("Strike_P")

        for i in range(10):
            execute_relic_triggers("onPlayCard", state, {"card": attack_card})

        assert len(state.hand) == 1

    def test_bird_faced_urn_heals_on_power(self):
        """Bird-Faced Urn should heal 2 HP when playing a Power."""
        state = create_test_combat(relics=["Bird Faced Urn"], player_hp=70, max_hp=80)

        # Find a power card
        power_card = None
        for card_id, card in ALL_CARDS.items():
            if card.card_type == CardType.POWER:
                power_card = card
                break

        assert power_card is not None, "No power card found"
        execute_relic_triggers("onPlayCard", state, {"card": power_card})
        assert state.player.hp == 72


# =============================================================================
# ON_EXHAUST Tests
# =============================================================================

class TestOnExhaustTriggers:
    """Test onExhaust relic triggers via registry."""

    def test_charons_ashes_damage(self):
        """Charon's Ashes should deal 3 damage to all enemies on exhaust."""
        state = create_test_combat(
            relics=["Charons Ashes"],
            enemies=[
                create_enemy("E1", hp=30, max_hp=30),
                create_enemy("E2", hp=30, max_hp=30),
            ]
        )

        card = ALL_CARDS.get("Strike_P")
        execute_relic_triggers("onExhaust", state, {"card": card})

        for enemy in state.enemies:
            assert enemy.hp == 27


# =============================================================================
# ON_VICTORY Tests
# =============================================================================

class TestOnVictoryTriggers:
    """Test onVictory relic triggers via registry."""

    def test_burning_blood_heals(self):
        """Burning Blood should heal 6 HP on victory."""
        state = create_test_combat(relics=["Burning Blood"], player_hp=50, max_hp=80)
        execute_relic_triggers("onVictory", state)
        assert state.player.hp == 56

    def test_black_blood_heals_more(self):
        """Black Blood should heal 12 HP on victory."""
        state = create_test_combat(relics=["Black Blood"], player_hp=50, max_hp=80)
        execute_relic_triggers("onVictory", state)
        assert state.player.hp == 62

    def test_meat_on_bone_heals_when_low(self):
        """Meat on the Bone should heal 12 HP if at 50% or less."""
        state = create_test_combat(relics=["Meat on the Bone"], player_hp=40, max_hp=80)
        execute_relic_triggers("onVictory", state)
        assert state.player.hp == 52

    def test_meat_on_bone_no_heal_when_high(self):
        """Meat on the Bone should not heal if above 50%."""
        state = create_test_combat(relics=["Meat on the Bone"], player_hp=50, max_hp=80)
        execute_relic_triggers("onVictory", state)
        assert state.player.hp == 50


# =============================================================================
# ON_SHUFFLE Tests
# =============================================================================

class TestOnShuffleTriggers:
    """Test onShuffle relic triggers via registry."""

    def test_sundial_counter_shuffles(self):
        """Sundial should count shuffles and give energy every 3."""
        state = create_test_combat(relics=["Sundial"], energy=3)
        state.set_relic_counter("Sundial", 0)

        # Trigger 3 shuffles
        for i in range(3):
            execute_relic_triggers("onShuffle", state)

        assert state.energy == 5  # +2 energy
        assert state.get_relic_counter("Sundial") == 0


# =============================================================================
# Multiple Relics Tests
# =============================================================================

class TestMultipleRelics:
    """Test that multiple relics trigger correctly together."""

    def test_multiple_battle_start_relics(self):
        """Multiple atBattleStart relics should all trigger."""
        state = create_test_combat(relics=["Vajra", "Anchor", "Bronze Scales"])
        execute_relic_triggers("atBattleStart", state)

        assert state.player.statuses.get("Strength", 0) == 1
        assert state.player.block == 10
        assert state.player.statuses.get("Thorns", 0) == 3

    def test_multiple_turn_start_relics(self):
        """Multiple atTurnStart relics should all trigger."""
        state = create_test_combat(relics=["Lantern", "HornCleat"], energy=3)
        state.turn = 1
        execute_relic_triggers("atTurnStart", state)
        assert state.energy == 4  # Lantern only on turn 1
        assert state.player.block == 0  # HornCleat only on turn 2

        state.turn = 2
        execute_relic_triggers("atTurnStart", state)
        assert state.player.block == 14  # HornCleat triggers

    def test_multiple_hp_lost_relics(self):
        """Multiple wasHPLost relics should all trigger."""
        state = create_test_combat(relics=["Runic Cube", "Self Forming Clay"])
        state.draw_pile = ["Card1", "Card2"]
        state.hand = []

        execute_relic_triggers("wasHPLost", state, {"hp_lost": 5})

        assert len(state.hand) == 1  # Runic Cube
        assert state.player.statuses.get("NextTurnBlock", 0) == 3  # Self Forming Clay


# =============================================================================
# Registry Verification Tests
# =============================================================================

class TestRegistryVerification:
    """Verify the registry has all expected relics registered."""

    def test_at_battle_start_relics_registered(self):
        """Key atBattleStart relics should be registered."""
        expected = [
            "Vajra", "Anchor", "Akabeko", "Bag of Marbles", "Blood Vial",
            "Bronze Scales", "Thread and Needle", "Oddly Smooth Stone",
            "FossilizedHelix", "Data Disk", "Damaru",
        ]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("atBattleStart", relic), f"{relic} not registered for atBattleStart"

    def test_at_turn_start_relics_registered(self):
        """Key atTurnStart relics should be registered."""
        expected = ["Lantern", "HornCleat", "Happy Flower", "Art of War"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("atTurnStart", relic), f"{relic} not registered for atTurnStart"

    def test_on_player_end_turn_relics_registered(self):
        """Key onPlayerEndTurn relics should be registered."""
        expected = ["Orichalcum", "StoneCalendar", "Art of War"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onPlayerEndTurn", relic), f"{relic} not registered for onPlayerEndTurn"

    def test_was_hp_lost_relics_registered(self):
        """Key wasHPLost relics should be registered."""
        expected = ["Centennial Puzzle", "Red Skull", "Runic Cube", "Self Forming Clay"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("wasHPLost", relic), f"{relic} not registered for wasHPLost"

    def test_on_play_card_relics_registered(self):
        """Key onPlayCard relics should be registered."""
        expected = ["Shuriken", "Kunai", "Nunchaku", "Ornamental Fan", "InkBottle"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onPlayCard", relic), f"{relic} not registered for onPlayCard"

    def test_on_exhaust_relics_registered(self):
        """Key onExhaust relics should be registered."""
        expected = ["Charons Ashes", "Dead Branch"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onExhaust", relic), f"{relic} not registered for onExhaust"

    def test_on_victory_relics_registered(self):
        """Key onVictory relics should be registered."""
        expected = ["Burning Blood", "Black Blood", "Meat on the Bone"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onVictory", relic), f"{relic} not registered for onVictory"
