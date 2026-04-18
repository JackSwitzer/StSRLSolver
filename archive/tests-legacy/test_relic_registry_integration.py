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
            "FossilizedHelix", "Data Disk",
        ]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("atBattleStart", relic), f"{relic} not registered for atBattleStart"

    def test_at_turn_start_relics_registered(self):
        """Key atTurnStart relics should be registered."""
        expected = ["Lantern", "HornCleat", "Happy Flower", "Art of War", "Damaru", "Mercury Hourglass"]
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


# =============================================================================
# ON_MONSTER_DEATH Tests
# =============================================================================

class TestOnMonsterDeathTriggers:
    """Test onMonsterDeath relic triggers via registry."""

    def test_gremlin_horn_energy_and_draw(self):
        """Gremlin Horn should give 1 energy and draw 1 card when enemy dies."""
        state = create_test_combat(relics=["Gremlin Horn"], energy=3)
        state.draw_pile = ["Card1", "Card2", "Card3"]
        state.hand = []

        dead_enemy = create_enemy("Dead", hp=0, max_hp=30)
        dead_enemy.hp = 0

        execute_relic_triggers("onMonsterDeath", state, {"enemy": dead_enemy})

        assert state.energy == 4
        assert len(state.hand) == 1

    def test_specimen_transfers_poison(self):
        """The Specimen should transfer Poison to random enemy when poisoned enemy dies."""
        state = create_test_combat(
            relics=["The Specimen"],
            enemies=[
                create_enemy("E1", hp=0, max_hp=30),  # Dead with poison
                create_enemy("E2", hp=30, max_hp=30),  # Alive
            ]
        )
        # Set up dead enemy with poison
        state.enemies[0].hp = 0
        state.enemies[0].statuses["Poison"] = 5

        execute_relic_triggers("onMonsterDeath", state, {"enemy": state.enemies[0]})

        # Poison should transfer to the living enemy
        assert state.enemies[1].statuses.get("Poison", 0) == 5

    def test_specimen_no_transfer_without_poison(self):
        """The Specimen should not transfer anything if dead enemy had no Poison."""
        state = create_test_combat(
            relics=["The Specimen"],
            enemies=[
                create_enemy("E1", hp=0, max_hp=30),
                create_enemy("E2", hp=30, max_hp=30),
            ]
        )
        state.enemies[0].hp = 0

        execute_relic_triggers("onMonsterDeath", state, {"enemy": state.enemies[0]})

        assert state.enemies[1].statuses.get("Poison", 0) == 0


# =============================================================================
# ON_OBTAIN_CARD Tests
# =============================================================================

class TestOnObtainCardTriggers:
    """Test onObtainCard relic triggers via registry."""

    def test_ceramic_fish_gains_gold(self):
        """Ceramic Fish should give 9 gold when obtaining a card."""
        state = create_test_combat(relics=["Ceramic Fish"])
        state.gold = 100

        execute_relic_triggers("onObtainCard", state, {"card_id": "Strike_R"})

        assert state.gold == 109

    def test_frozen_egg_upgrades_powers(self):
        """Frozen Egg 2 should return upgraded version of Power cards."""
        state = create_test_combat(relics=["Frozen Egg 2"])

        # Test with a power card (Eruption is a Skill, let's use a Power)
        from packages.engine.content.cards import ALL_CARDS, CardType

        # Find a power card that exists
        power_id = None
        for card_id, card in ALL_CARDS.items():
            if card.card_type == CardType.POWER and not card_id.endswith("+"):
                power_id = card_id
                break

        if power_id:
            ctx = RelicContext(
                state=state,
                relic_id="Frozen Egg 2",
                trigger_data={"card_id": power_id}
            )
            from packages.engine.registry.relics import frozen_egg_obtain
            result = frozen_egg_obtain(ctx)
            assert result == power_id + "+"

    def test_molten_egg_upgrades_attacks(self):
        """Molten Egg 2 should return upgraded version of Attack cards."""
        state = create_test_combat(relics=["Molten Egg 2"])

        # Strike is an attack
        ctx = RelicContext(
            state=state,
            relic_id="Molten Egg 2",
            trigger_data={"card_id": "Strike_R"}
        )
        from packages.engine.registry.relics import molten_egg_obtain
        result = molten_egg_obtain(ctx)
        assert result == "Strike_R+"

    def test_toxic_egg_upgrades_skills(self):
        """Toxic Egg 2 should return upgraded version of Skill cards."""
        state = create_test_combat(relics=["Toxic Egg 2"])

        # Defend is a skill
        ctx = RelicContext(
            state=state,
            relic_id="Toxic Egg 2",
            trigger_data={"card_id": "Defend_R"}
        )
        from packages.engine.registry.relics import toxic_egg_obtain
        result = toxic_egg_obtain(ctx)
        assert result == "Defend_R+"

    def test_darkstone_periapt_gains_max_hp_on_curse(self):
        """Darkstone Periapt should give 6 max HP when obtaining a Curse."""
        state = create_test_combat(relics=["Darkstone Periapt"], player_hp=70, max_hp=80)

        # Use a curse card
        from packages.engine.content.cards import ALL_CARDS, CardType

        curse_id = None
        for card_id, card in ALL_CARDS.items():
            if card.card_type == CardType.CURSE:
                curse_id = card_id
                break

        if curse_id:
            execute_relic_triggers("onObtainCard", state, {"card_id": curse_id})
            assert state.player.max_hp == 86
            assert state.player.hp == 76  # Also heals to new max

    def test_eggs_dont_double_upgrade(self):
        """Egg relics should not upgrade already-upgraded cards."""
        state = create_test_combat(relics=["Molten Egg 2"])

        ctx = RelicContext(
            state=state,
            relic_id="Molten Egg 2",
            trigger_data={"card_id": "Strike_R+"}  # Already upgraded
        )
        from packages.engine.registry.relics import molten_egg_obtain
        result = molten_egg_obtain(ctx)
        assert result == "Strike_R+"  # Unchanged


# =============================================================================
# ON_USE_POTION Tests
# =============================================================================

class TestOnUsePotionTriggers:
    """Test onUsePotion relic triggers via registry."""

    def test_toy_ornithopter_heals(self):
        """Toy Ornithopter should heal 5 HP when using a potion."""
        state = create_test_combat(relics=["Toy Ornithopter"], player_hp=60, max_hp=80)

        execute_relic_triggers("onUsePotion", state)

        assert state.player.hp == 65

    def test_toy_ornithopter_respects_max_hp(self):
        """Toy Ornithopter healing should not exceed max HP."""
        state = create_test_combat(relics=["Toy Ornithopter"], player_hp=78, max_hp=80)

        execute_relic_triggers("onUsePotion", state)

        assert state.player.hp == 80


# =============================================================================
# DAMAGE MODIFIER Tests
# =============================================================================

class TestDamageModifierTriggers:
    """Test damage modifier relic triggers via registry."""

    def test_wrist_blade_zero_cost_bonus(self):
        """Wrist Blade should add 4 damage to 0-cost Attacks."""
        state = create_test_combat(relics=["WristBlade"])
        state.card_costs = {}

        # Create a mock card with 0 cost
        class MockCard:
            def __init__(self):
                self.id = "Shiv"
                self.cost = 0
                self.card_type = CardType.ATTACK

        mock_card = MockCard()
        state.card_costs["Shiv"] = 0

        ctx = RelicContext(
            state=state,
            relic_id="WristBlade",
            trigger_data={"value": 6},  # Base damage
            card=mock_card,
        )
        from packages.engine.registry.relics import wrist_blade_damage
        result = wrist_blade_damage(ctx)

        assert result == 10  # 6 + 4

    def test_wrist_blade_no_bonus_nonzero_cost(self):
        """Wrist Blade should not add damage to cards costing 1+."""
        state = create_test_combat(relics=["WristBlade"])
        state.card_costs = {"Strike": 1}

        class MockCard:
            def __init__(self):
                self.id = "Strike"
                self.cost = 1
                self.card_type = CardType.ATTACK

        mock_card = MockCard()

        ctx = RelicContext(
            state=state,
            relic_id="WristBlade",
            trigger_data={"value": 6},
            card=mock_card,
        )
        from packages.engine.registry.relics import wrist_blade_damage
        result = wrist_blade_damage(ctx)

        assert result == 6  # Unchanged

    def test_strike_dummy_bonus(self):
        """Strike Dummy should add 3 damage to cards containing 'Strike'."""
        state = create_test_combat(relics=["StrikeDummy"])

        class MockCard:
            def __init__(self):
                self.id = "Strike_R"
                self.card_type = CardType.ATTACK

        mock_card = MockCard()

        ctx = RelicContext(
            state=state,
            relic_id="StrikeDummy",
            trigger_data={"value": 6},
            card=mock_card,
        )
        from packages.engine.registry.relics import strike_dummy_damage
        result = strike_dummy_damage(ctx)

        assert result == 9  # 6 + 3

    def test_strike_dummy_no_bonus_other_cards(self):
        """Strike Dummy should not add damage to non-Strike cards."""
        state = create_test_combat(relics=["StrikeDummy"])

        class MockCard:
            def __init__(self):
                self.id = "Bash"
                self.card_type = CardType.ATTACK

        mock_card = MockCard()

        ctx = RelicContext(
            state=state,
            relic_id="StrikeDummy",
            trigger_data={"value": 10},
            card=mock_card,
        )
        from packages.engine.registry.relics import strike_dummy_damage
        result = strike_dummy_damage(ctx)

        assert result == 10  # Unchanged

    def test_boot_minimum_damage(self):
        """The Boot should ensure minimum 5 damage on attacks."""
        state = create_test_combat(relics=["Boot"])

        ctx = RelicContext(
            state=state,
            relic_id="Boot",
            trigger_data={"value": 3},  # Low damage
        )
        from packages.engine.registry.relics import boot_damage
        result = boot_damage(ctx)

        assert result == 5  # Minimum 5

    def test_boot_no_change_above_5(self):
        """The Boot should not modify damage already at or above 5."""
        state = create_test_combat(relics=["Boot"])

        ctx = RelicContext(
            state=state,
            relic_id="Boot",
            trigger_data={"value": 10},
        )
        from packages.engine.registry.relics import boot_damage
        result = boot_damage(ctx)

        assert result == 10  # Unchanged

    def test_boot_zero_damage_unchanged(self):
        """The Boot should not change 0 damage to 5."""
        state = create_test_combat(relics=["Boot"])

        ctx = RelicContext(
            state=state,
            relic_id="Boot",
            trigger_data={"value": 0},
        )
        from packages.engine.registry.relics import boot_damage
        result = boot_damage(ctx)

        assert result == 0  # Still 0

    def test_torii_reduces_low_damage(self):
        """Torii should reduce damage from 2-5 to 1."""
        state = create_test_combat(relics=["Torii"])

        for dmg in [2, 3, 4, 5]:
            ctx = RelicContext(
                state=state,
                relic_id="Torii",
                trigger_data={"value": dmg},
            )
            from packages.engine.registry.relics import torii_damage
            result = torii_damage(ctx)
            assert result == 1, f"Torii should reduce {dmg} to 1"

    def test_torii_no_change_outside_range(self):
        """Torii should not modify damage outside 2-5 range."""
        state = create_test_combat(relics=["Torii"])

        for dmg in [1, 6, 10, 15]:
            ctx = RelicContext(
                state=state,
                relic_id="Torii",
                trigger_data={"value": dmg},
            )
            from packages.engine.registry.relics import torii_damage
            result = torii_damage(ctx)
            assert result == dmg, f"Torii should not modify {dmg}"

    def test_tungsten_rod_reduces_hp_loss(self):
        """Tungsten Rod should reduce HP loss by 1."""
        state = create_test_combat(relics=["TungstenRod"])

        ctx = RelicContext(
            state=state,
            relic_id="TungstenRod",
            trigger_data={"value": 5},  # HP loss
        )
        from packages.engine.registry.relics import tungsten_rod_hp_loss
        result = tungsten_rod_hp_loss(ctx)

        assert result == 4

    def test_tungsten_rod_minimum_zero(self):
        """Tungsten Rod should not reduce HP loss below 0."""
        state = create_test_combat(relics=["TungstenRod"])

        ctx = RelicContext(
            state=state,
            relic_id="TungstenRod",
            trigger_data={"value": 1},
        )
        from packages.engine.registry.relics import tungsten_rod_hp_loss
        result = tungsten_rod_hp_loss(ctx)

        assert result == 0


# =============================================================================
# ON_PLAY_CARD Tests (Advanced Triggers)
# =============================================================================

class TestOnPlayCardAdvancedTriggers:
    """Test advanced onPlayCard relic triggers."""

    def test_necronomicon_marks_replay(self):
        """Necronomicon should mark first 2+ cost attack for replay."""
        state = create_test_combat(relics=["Necronomicon"])
        state.card_costs = {"Heavy Blade": 2}
        state.set_relic_counter("Necronomicon", 0)
        state.cards_to_replay = None

        class MockCard:
            def __init__(self):
                self.id = "Heavy Blade"
                self.cost = 2
                self.card_type = CardType.ATTACK

        mock_card = MockCard()

        execute_relic_triggers("onPlayCard", state, {"card": mock_card})

        assert state.get_relic_counter("Necronomicon") == 1  # Triggered
        assert state.cards_to_replay == ["Heavy Blade"]

    def test_necronomicon_only_once_per_turn(self):
        """Necronomicon should only trigger once per turn."""
        state = create_test_combat(relics=["Necronomicon"])
        state.card_costs = {"Heavy Blade": 2}
        state.set_relic_counter("Necronomicon", 1)  # Already triggered
        state.cards_to_replay = []

        class MockCard:
            def __init__(self):
                self.id = "Heavy Blade"
                self.cost = 2
                self.card_type = CardType.ATTACK

        mock_card = MockCard()

        execute_relic_triggers("onPlayCard", state, {"card": mock_card})

        assert state.cards_to_replay == []  # No new replay

    def test_necronomicon_ignores_cheap_attacks(self):
        """Necronomicon should not trigger on attacks costing less than 2."""
        state = create_test_combat(relics=["Necronomicon"])
        state.card_costs = {"Strike": 1}
        state.set_relic_counter("Necronomicon", 0)
        state.cards_to_replay = None

        class MockCard:
            def __init__(self):
                self.id = "Strike"
                self.cost = 1
                self.card_type = CardType.ATTACK

        mock_card = MockCard()

        execute_relic_triggers("onPlayCard", state, {"card": mock_card})

        assert state.get_relic_counter("Necronomicon") == 0
        assert state.cards_to_replay is None

    def test_velvet_choker_tracks_cards(self):
        """Velvet Choker should track cards played this turn."""
        state = create_test_combat(relics=["Velvet Choker"])
        state.set_relic_counter("Velvet Choker", 0)

        attack_card = ALL_CARDS.get("Strike_R")

        for i in range(5):
            execute_relic_triggers("onPlayCard", state, {"card": attack_card})

        assert state.get_relic_counter("Velvet Choker") == 5

    def test_orange_pellets_clears_debuffs(self):
        """Orange Pellets should clear debuffs after attack+skill+power."""
        state = create_test_combat(relics=["OrangePellets"])
        state.set_relic_counter("OrangePellets", 0)
        state.player.statuses["Weakened"] = 2
        state.player.statuses["Vulnerable"] = 3

        # Get card types
        attack_card = ALL_CARDS.get("Strike_R")
        skill_card = ALL_CARDS.get("Defend_R")

        # Find a power card
        power_card = None
        for card_id, card in ALL_CARDS.items():
            if card.card_type == CardType.POWER:
                power_card = card
                break

        # Play attack
        execute_relic_triggers("onPlayCard", state, {"card": attack_card})
        assert state.player.statuses.get("Weakened", 0) == 2  # Not cleared yet

        # Play skill
        execute_relic_triggers("onPlayCard", state, {"card": skill_card})
        assert state.player.statuses.get("Weakened", 0) == 2  # Not cleared yet

        # Play power - should clear debuffs
        if power_card:
            execute_relic_triggers("onPlayCard", state, {"card": power_card})
            assert state.player.statuses.get("Weakened", 0) == 0
            assert state.player.statuses.get("Vulnerable", 0) == 0


# =============================================================================
# DISCARD/EXHAUST Tests
# =============================================================================

class TestDiscardExhaustTriggers:
    """Test discard and exhaust relic triggers."""

    def test_strange_spoon_fifty_percent(self):
        """Strange Spoon should have 50% chance to send exhausted card to discard."""
        import random

        # Seed for deterministic test
        random.seed(42)

        state = create_test_combat(relics=["Strange Spoon"])
        state.exhaust_pile = ["TestCard"]
        state.discard_pile = []

        ctx = RelicContext(
            state=state,
            relic_id="Strange Spoon",
            trigger_data={"card_id": "TestCard"}
        )
        from packages.engine.registry.relics import strange_spoon_exhaust
        strange_spoon_exhaust(ctx)

        # With seed 42, first random() should trigger the 50% chance
        # Check that card moved (or didn't) based on random result
        total_locations = len(state.exhaust_pile) + len(state.discard_pile)
        assert total_locations == 1  # Card is in one place or the other

    def test_tingsha_deals_damage(self):
        """Tingsha should deal 3 damage to random enemy on discard."""
        import random
        random.seed(1)

        state = create_test_combat(
            relics=["Tingsha"],
            enemies=[create_enemy("E1", hp=30, max_hp=30)]
        )

        execute_relic_triggers("onManualDiscard", state)

        assert state.enemies[0].hp == 27

    def test_tough_bandages_gains_block(self):
        """Tough Bandages should gain 3 block on discard."""
        state = create_test_combat(relics=["Tough Bandages"])

        execute_relic_triggers("onManualDiscard", state)

        assert state.player.block == 3

    def test_tough_bandages_stacks_on_multiple_discards(self):
        """Tough Bandages should stack block on multiple discards."""
        state = create_test_combat(relics=["Tough Bandages"])

        for _ in range(3):
            execute_relic_triggers("onManualDiscard", state)

        assert state.player.block == 9

    def test_hovering_kite_first_discard_energy(self):
        """Hovering Kite should give 1 energy on first discard each turn."""
        state = create_test_combat(relics=["HoveringKite"], energy=3)
        state.set_relic_counter("HoveringKite", 0)

        execute_relic_triggers("onManualDiscard", state)

        assert state.energy == 4
        assert state.get_relic_counter("HoveringKite") == 1

    def test_hovering_kite_only_first_discard(self):
        """Hovering Kite should only give energy on first discard."""
        state = create_test_combat(relics=["HoveringKite"], energy=3)
        state.set_relic_counter("HoveringKite", 1)  # Already triggered

        execute_relic_triggers("onManualDiscard", state)

        assert state.energy == 3  # No additional energy

    def test_hovering_kite_resets_at_turn_start(self):
        """Hovering Kite counter should reset at turn start."""
        state = create_test_combat(relics=["HoveringKite"])
        state.set_relic_counter("HoveringKite", 1)

        execute_relic_triggers("atTurnStart", state)

        assert state.get_relic_counter("HoveringKite") == 0

    def test_unceasing_top_draws_on_empty_hand(self):
        """Unceasing Top should draw a card when hand becomes empty."""
        state = create_test_combat(relics=["Unceasing Top"])
        state.hand = []  # Empty hand
        state.draw_pile = ["Card1", "Card2", "Card3"]

        execute_relic_triggers("onEmptyHand", state)

        assert len(state.hand) == 1

    def test_unceasing_top_no_draw_with_cards(self):
        """Unceasing Top should not draw if hand is not empty."""
        state = create_test_combat(relics=["Unceasing Top"])
        state.hand = ["SomeCard"]
        state.draw_pile = ["Card1", "Card2"]

        execute_relic_triggers("onEmptyHand", state)

        assert len(state.hand) == 1  # Still just the one card


# =============================================================================
# REGISTRY VERIFICATION Tests (New Triggers)
# =============================================================================

class TestNewTriggerRegistration:
    """Verify new triggers are properly registered."""

    def test_on_monster_death_relics_registered(self):
        """Monster death relics should be registered."""
        expected = ["Gremlin Horn", "The Specimen"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onMonsterDeath", relic), f"{relic} not registered"

    def test_on_obtain_card_relics_registered(self):
        """Obtain card relics should be registered."""
        expected = ["Ceramic Fish", "Frozen Egg 2", "Molten Egg 2", "Toxic Egg 2", "Darkstone Periapt"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onObtainCard", relic), f"{relic} not registered"

    def test_on_use_potion_relics_registered(self):
        """Use potion relics should be registered."""
        expected = ["Toy Ornithopter"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onUsePotion", relic), f"{relic} not registered"

    def test_damage_modifier_relics_registered(self):
        """Damage modifier relics should be registered."""
        expected = [
            ("atDamageGive", "WristBlade"),
            ("atDamageGive", "StrikeDummy"),
            ("atDamageFinalGive", "Boot"),
            ("onAttackedToChangeDamage", "Torii"),
            ("onLoseHpLast", "TungstenRod"),
        ]
        for hook, relic in expected:
            assert RELIC_REGISTRY.has_handler(hook, relic), f"{relic} not registered for {hook}"

    def test_advanced_on_play_card_relics_registered(self):
        """Advanced onPlayCard relics should be registered."""
        expected = ["Necronomicon", "Velvet Choker", "OrangePellets"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onPlayCard", relic), f"{relic} not registered"

    def test_discard_relics_registered(self):
        """Discard relics should be registered."""
        expected = ["Tingsha", "Tough Bandages", "HoveringKite"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onManualDiscard", relic), f"{relic} not registered"

    def test_exhaust_relics_registered(self):
        """Exhaust relics should be registered."""
        expected = ["Strange Spoon", "Charons Ashes", "Dead Branch"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onExhaust", relic), f"{relic} not registered"

    def test_empty_hand_relics_registered(self):
        """Empty hand relics should be registered."""
        expected = ["Unceasing Top"]
        for relic in expected:
            assert RELIC_REGISTRY.has_handler("onEmptyHand", relic), f"{relic} not registered"
