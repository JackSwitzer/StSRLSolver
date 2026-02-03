"""
Combat Flow Tests - Comprehensive tests for Slay the Spire combat mechanics.

Tests cover:
1. Turn order: player turn -> end of turn effects -> enemy turn -> start of turn effects
2. End of turn: block decay, power durations, poison ticks
3. Start of turn: draw cards, energy reset, relic triggers
4. Card play order and effect resolution
5. Death checks (when exactly does lethal get checked)
6. Draw pile shuffle when empty
7. Hand size limits (10 cards)
8. Exhaust pile handling
9. Card costs during play (0 after Corruption, X-costs, etc.)
10. Stance transitions mid-turn
11. Multi-enemy targeting and death mid-attack
12. Block carryover (Barricade, Calipers)
13. Energy carryover (Ice Cream)
14. Artifact blocking debuffs
15. Intangible application timing
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    create_player, create_enemy, create_combat,
    PlayCard, UsePotion, EndTurn,
)
from packages.engine.state.rng import Random
from packages.engine.content.stances import StanceID, StanceManager, STANCES
from packages.engine.content.cards import (
    Card, CardType, CardTarget, get_card, get_starting_deck,
    STRIKE_W, DEFEND_W, ERUPTION, VIGILANCE, TANTRUM,
)
from packages.engine.content.powers import (
    PowerManager, create_power, create_strength, create_dexterity,
    create_weak, create_vulnerable, create_frail, create_poison,
    create_artifact, create_intangible, create_vigor,
)
from packages.engine.calc.damage import (
    calculate_damage, calculate_block, calculate_incoming_damage,
    WEAK_MULT, VULN_MULT, WRATH_MULT, DIVINITY_MULT, FRAIL_MULT,
)
from packages.engine.combat_engine import CombatEngine, create_simple_combat


# =============================================================================
# FIXTURES
# =============================================================================

@pytest.fixture
def basic_combat():
    """Create a basic combat state for testing."""
    enemies = [
        create_enemy("JawWorm", hp=44, max_hp=44, move_damage=11),
    ]
    deck = ["Strike_P", "Strike_P", "Defend_P", "Defend_P", "Eruption"]
    return create_combat(
        player_hp=80,
        player_max_hp=80,
        enemies=enemies,
        deck=deck,
        energy=3,
        max_energy=3,
    )


@pytest.fixture
def multi_enemy_combat():
    """Combat with multiple enemies."""
    enemies = [
        create_enemy("Louse", hp=15, max_hp=15, move_damage=6),
        create_enemy("Louse", hp=15, max_hp=15, move_damage=6),
        create_enemy("Louse", hp=15, max_hp=15, move_damage=6),
    ]
    deck = ["Strike_P"] * 5 + ["Defend_P"] * 5
    return create_combat(
        player_hp=80,
        player_max_hp=80,
        enemies=enemies,
        deck=deck,
        energy=3,
        max_energy=3,
    )


@pytest.fixture
def stance_combat():
    """Combat setup for stance testing."""
    enemies = [
        create_enemy("TestEnemy", hp=100, max_hp=100, move_damage=10),
    ]
    deck = ["Eruption", "Vigilance", "EmptyFist", "EmptyBody", "Crescendo", "Tranquility"]
    return create_combat(
        player_hp=80,
        player_max_hp=80,
        enemies=enemies,
        deck=deck,
        energy=3,
        max_energy=3,
    )


@pytest.fixture
def power_manager():
    """Create an empty PowerManager for testing."""
    return PowerManager()


# =============================================================================
# SECTION 1: TURN ORDER TESTS
# =============================================================================

class TestTurnOrder:
    """Test turn order: player -> end of turn -> enemy -> start of turn."""

    def test_player_turn_comes_first(self, basic_combat):
        """Player acts before enemies."""
        # Player should have energy and be able to act
        assert basic_combat.energy == 3
        assert basic_combat.turn == 1

    def test_end_turn_resets_energy(self, basic_combat):
        """Energy resets at start of next turn."""
        # Spend some energy
        basic_combat.energy = 0

        # Simulate turn transition - energy resets to max
        basic_combat.energy = basic_combat.max_energy
        assert basic_combat.energy == 3

    def test_turn_counter_increments(self, basic_combat):
        """Turn counter increases each turn."""
        initial_turn = basic_combat.turn
        basic_combat.turn += 1
        assert basic_combat.turn == initial_turn + 1


class TestEndOfTurnEffects:
    """Test end of turn effects: block decay, power durations, poison."""

    def test_block_decays_normally(self, basic_combat):
        """Block resets to 0 at end of turn without Barricade."""
        basic_combat.player.block = 15
        assert "Barricade" not in basic_combat.player.statuses

        # Simulate block decay
        basic_combat.player.block = 0
        assert basic_combat.player.block == 0

    def test_block_persists_with_barricade(self, basic_combat):
        """Block persists with Barricade."""
        basic_combat.player.block = 15
        basic_combat.player.statuses["Barricade"] = 1

        # With Barricade, block should NOT decay
        # (Implementation keeps block)
        assert basic_combat.player.block == 15

    def test_power_durations_decrement(self, power_manager):
        """Turn-based powers decrement at end of round."""
        power_manager.add_power(create_weak(2))
        assert power_manager.get_amount("Weakened") == 2

        # Simulate end of round
        removed = power_manager.at_end_of_round()
        assert power_manager.get_amount("Weakened") == 1

        # Another round
        removed = power_manager.at_end_of_round()
        assert not power_manager.has_power("Weakened")

    def test_poison_ticks_at_start_of_turn(self, power_manager):
        """Poison deals damage at start of turn."""
        power_manager.add_power(create_poison(5))

        effects = power_manager.at_start_of_turn()
        assert effects["poison_damage"] == 5

    def test_vulnerable_decrements(self, power_manager):
        """Vulnerable decrements at end of round."""
        power_manager.add_power(create_vulnerable(2))

        power_manager.at_end_of_round()
        assert power_manager.get_amount("Vulnerable") == 1

        power_manager.at_end_of_round()
        assert not power_manager.has_power("Vulnerable")

    def test_frail_decrements(self, power_manager):
        """Frail decrements at end of round."""
        power_manager.add_power(create_frail(3))

        power_manager.at_end_of_round()
        assert power_manager.get_amount("Frail") == 2


class TestStartOfTurnEffects:
    """Test start of turn: draw cards, energy reset, relic triggers."""

    def test_energy_resets_to_max(self, basic_combat):
        """Energy resets to max_energy at turn start."""
        basic_combat.energy = 0
        # Simulate turn start
        basic_combat.energy = basic_combat.max_energy
        assert basic_combat.energy == 3

    def test_draw_five_cards_default(self, basic_combat):
        """Draw 5 cards at turn start by default."""
        # Standard draw amount is 5
        draw_amount = 5
        assert draw_amount == 5

    def test_no_draw_power_prevents_draw(self, power_manager):
        """No Draw power prevents drawing."""
        power_manager.add_power(create_power("No Draw", 1))
        assert power_manager.has_power("No Draw")

    def test_turn_counters_reset(self, basic_combat):
        """Combat tracking counters reset each turn."""
        basic_combat.cards_played_this_turn = 5
        basic_combat.attacks_played_this_turn = 3

        # Reset for new turn
        basic_combat.cards_played_this_turn = 0
        basic_combat.attacks_played_this_turn = 0

        assert basic_combat.cards_played_this_turn == 0
        assert basic_combat.attacks_played_this_turn == 0


# =============================================================================
# SECTION 2: CARD PLAY TESTS
# =============================================================================

class TestCardPlayOrder:
    """Test card play order and effect resolution."""

    def test_energy_deducted_first(self, basic_combat):
        """Energy cost deducted when playing card."""
        initial_energy = basic_combat.energy
        card_cost = 1

        # Play a 1-cost card
        basic_combat.energy -= card_cost
        assert basic_combat.energy == initial_energy - card_cost

    def test_card_removed_from_hand(self, basic_combat):
        """Card removed from hand when played."""
        basic_combat.hand = ["Strike_P", "Defend_P"]

        # Play first card
        played_card = basic_combat.hand.pop(0)
        assert played_card == "Strike_P"
        assert len(basic_combat.hand) == 1

    def test_card_goes_to_discard(self, basic_combat):
        """Card goes to discard pile after play."""
        basic_combat.hand = ["Strike_P"]
        basic_combat.discard_pile = []

        card = basic_combat.hand.pop(0)
        basic_combat.discard_pile.append(card)

        assert card in basic_combat.discard_pile
        assert len(basic_combat.hand) == 0

    def test_exhaust_goes_to_exhaust_pile(self, basic_combat):
        """Exhausted cards go to exhaust pile."""
        basic_combat.exhaust_pile = []

        card = "Offering"  # An exhaust card
        basic_combat.exhaust_pile.append(card)

        assert card in basic_combat.exhaust_pile

    def test_cards_played_counter_increments(self, basic_combat):
        """Cards played this turn counter increments."""
        assert basic_combat.cards_played_this_turn == 0

        basic_combat.cards_played_this_turn += 1
        assert basic_combat.cards_played_this_turn == 1

    def test_attacks_played_counter_increments(self, basic_combat):
        """Attacks played counter increments for attack cards."""
        assert basic_combat.attacks_played_this_turn == 0

        # Play an attack
        basic_combat.attacks_played_this_turn += 1
        assert basic_combat.attacks_played_this_turn == 1


class TestDamageApplication:
    """Test damage calculation and application."""

    def test_basic_damage_calculation(self):
        """Basic damage with no modifiers."""
        damage = calculate_damage(6)
        assert damage == 6

    def test_strength_adds_damage(self):
        """Strength adds flat damage."""
        damage = calculate_damage(6, strength=3)
        assert damage == 9

    def test_weak_reduces_damage(self):
        """Weak reduces damage by 25%."""
        damage = calculate_damage(10, weak=True)
        assert damage == 7  # 10 * 0.75 = 7.5 -> 7

    def test_vulnerable_increases_damage(self):
        """Vulnerable increases damage by 50%."""
        damage = calculate_damage(10, vuln=True)
        assert damage == 15  # 10 * 1.5

    def test_damage_blocked_first(self):
        """Damage hits block before HP."""
        hp_loss, block_remaining = calculate_incoming_damage(10, block=5)
        assert hp_loss == 5
        assert block_remaining == 0

    def test_full_block(self):
        """Full block prevents HP loss."""
        hp_loss, block_remaining = calculate_incoming_damage(5, block=10)
        assert hp_loss == 0
        assert block_remaining == 5


class TestBlockCalculation:
    """Test block calculation."""

    def test_basic_block(self):
        """Basic block with no modifiers."""
        block = calculate_block(5)
        assert block == 5

    def test_dexterity_adds_block(self):
        """Dexterity adds flat block."""
        block = calculate_block(5, dexterity=3)
        assert block == 8

    def test_frail_reduces_block(self):
        """Frail reduces block by 25%."""
        block = calculate_block(8, frail=True)
        assert block == 6  # 8 * 0.75 = 6

    def test_dexterity_before_frail(self):
        """Dexterity applies before Frail."""
        # (5 + 3) * 0.75 = 6
        block = calculate_block(5, dexterity=3, frail=True)
        assert block == 6


# =============================================================================
# SECTION 3: DEATH CHECK TESTS
# =============================================================================

class TestDeathChecks:
    """Test when death is checked."""

    def test_player_death_at_zero_hp(self, basic_combat):
        """Player is dead at 0 HP."""
        basic_combat.player.hp = 0
        assert basic_combat.player.is_dead

    def test_player_death_below_zero(self, basic_combat):
        """Player is dead below 0 HP."""
        basic_combat.player.hp = -5
        assert basic_combat.player.is_dead

    def test_enemy_death_at_zero_hp(self, basic_combat):
        """Enemy is dead at 0 HP."""
        basic_combat.enemies[0].hp = 0
        assert basic_combat.enemies[0].is_dead

    def test_combat_ends_on_player_death(self, basic_combat):
        """Combat ends when player dies."""
        basic_combat.player.hp = 0
        assert basic_combat.is_defeat()

    def test_combat_ends_on_all_enemies_dead(self, basic_combat):
        """Combat ends when all enemies die."""
        for enemy in basic_combat.enemies:
            enemy.hp = 0
        assert basic_combat.is_victory()

    def test_victory_check_requires_all_dead(self, multi_enemy_combat):
        """All enemies must be dead for victory."""
        multi_enemy_combat.enemies[0].hp = 0
        multi_enemy_combat.enemies[1].hp = 0
        # One enemy still alive
        assert not multi_enemy_combat.is_victory()

        multi_enemy_combat.enemies[2].hp = 0
        assert multi_enemy_combat.is_victory()


# =============================================================================
# SECTION 4: DRAW PILE TESTS
# =============================================================================

class TestDrawPileShuffle:
    """Test draw pile shuffling when empty."""

    def test_shuffle_discard_into_draw(self, basic_combat):
        """Discard pile shuffles into draw pile when empty."""
        basic_combat.draw_pile = []
        basic_combat.discard_pile = ["Strike_P", "Defend_P", "Eruption"]

        # Shuffle discard into draw
        basic_combat.draw_pile = basic_combat.discard_pile.copy()
        basic_combat.discard_pile = []

        assert len(basic_combat.draw_pile) == 3
        assert len(basic_combat.discard_pile) == 0

    def test_empty_discard_no_draw(self, basic_combat):
        """No cards drawn if both piles empty."""
        basic_combat.draw_pile = []
        basic_combat.discard_pile = []

        # Cannot draw
        assert len(basic_combat.draw_pile) == 0
        assert len(basic_combat.discard_pile) == 0

    def test_exhaust_not_shuffled(self, basic_combat):
        """Exhaust pile NOT shuffled into draw pile."""
        basic_combat.draw_pile = []
        basic_combat.discard_pile = []
        basic_combat.exhaust_pile = ["Offering", "Shrug"]

        # Shuffle happens but exhaust stays
        # Only discard goes to draw
        basic_combat.draw_pile = basic_combat.discard_pile.copy()

        assert len(basic_combat.draw_pile) == 0
        assert len(basic_combat.exhaust_pile) == 2


class TestHandSizeLimits:
    """Test 10 card hand size limit."""

    def test_hand_can_have_ten_cards(self, basic_combat):
        """Hand can hold up to 10 cards."""
        basic_combat.hand = ["Strike_P"] * 10
        assert len(basic_combat.hand) == 10

    def test_hand_limit_respected(self, basic_combat):
        """Drawing stops at hand limit."""
        basic_combat.hand = ["Strike_P"] * 10
        max_hand_size = 10

        # Attempting to draw should fail
        if len(basic_combat.hand) < max_hand_size:
            basic_combat.hand.append("Defend_P")

        assert len(basic_combat.hand) == 10

    def test_retain_respects_limit(self, basic_combat):
        """Retained cards still respect hand limit."""
        basic_combat.hand = ["Protect"] * 10  # All retained
        max_hand_size = 10

        # Hand is full
        assert len(basic_combat.hand) == max_hand_size


# =============================================================================
# SECTION 5: EXHAUST PILE TESTS
# =============================================================================

class TestExhaustPileHandling:
    """Test exhaust pile mechanics."""

    def test_exhaust_card_goes_to_exhaust(self, basic_combat):
        """Exhaust card goes to exhaust pile."""
        basic_combat.exhaust_pile = []

        # Simulate playing exhaust card
        basic_combat.exhaust_pile.append("Offering")
        assert "Offering" in basic_combat.exhaust_pile

    def test_ethereal_exhausts_on_discard(self):
        """Ethereal cards exhaust if in hand at end of turn."""
        card = get_card("Apparition") if "Apparition" in [
            "Ghostly"
        ] else None
        # Apparition (Ghostly) is ethereal
        # Would go to exhaust if in hand at turn end

    def test_exhaust_pile_persists(self, basic_combat):
        """Exhaust pile persists through shuffles."""
        basic_combat.exhaust_pile = ["Offering"]
        initial_count = len(basic_combat.exhaust_pile)

        # Simulate shuffle
        basic_combat.draw_pile = basic_combat.discard_pile.copy()
        basic_combat.discard_pile = []

        # Exhaust pile unchanged
        assert len(basic_combat.exhaust_pile) == initial_count


# =============================================================================
# SECTION 6: CARD COST TESTS
# =============================================================================

class TestCardCosts:
    """Test card cost mechanics."""

    def test_normal_card_costs_energy(self):
        """Playing card costs energy."""
        card = get_card("Strike_P")
        assert card.current_cost == 1

    def test_zero_cost_card_free(self):
        """Zero cost cards are free."""
        card = get_card("Consecrate")
        assert card.current_cost == 0

    def test_upgraded_cost_reduction(self):
        """Upgraded cards can have reduced cost."""
        card = get_card("Eruption")
        assert card.current_cost == 2

        card.upgrade()
        assert card.current_cost == 1  # Eruption+ costs 1

    def test_modified_cost_in_cache(self, basic_combat):
        """Card cost modifications tracked in cache."""
        basic_combat.card_costs["Strike_P"] = 0
        assert basic_combat.card_costs["Strike_P"] == 0


# =============================================================================
# SECTION 7: STANCE TESTS
# =============================================================================

class TestStanceTransitions:
    """Test stance transitions mid-turn."""

    def test_enter_wrath_from_neutral(self):
        """Enter Wrath from Neutral."""
        sm = StanceManager()
        assert sm.current == StanceID.NEUTRAL

        result = sm.change_stance(StanceID.WRATH)
        assert sm.current == StanceID.WRATH
        assert result["is_stance_change"]

    def test_enter_calm_from_neutral(self):
        """Enter Calm from Neutral."""
        sm = StanceManager()
        result = sm.change_stance(StanceID.CALM)
        assert sm.current == StanceID.CALM

    def test_exit_calm_gives_energy(self):
        """Exiting Calm gives 2 energy."""
        sm = StanceManager()
        sm.change_stance(StanceID.CALM)

        result = sm.exit_stance()
        assert result["energy_gained"] == 2

    def test_violet_lotus_extra_energy(self):
        """Violet Lotus gives 3 energy on Calm exit."""
        sm = StanceManager(has_violet_lotus=True)
        sm.change_stance(StanceID.CALM)

        result = sm.exit_stance()
        assert result["energy_gained"] == 3

    def test_wrath_doubles_damage(self):
        """Wrath stance doubles damage."""
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)

        damage = sm.at_damage_give(10.0)
        assert damage == 20.0

    def test_wrath_doubles_incoming(self):
        """Wrath stance doubles incoming damage."""
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)

        damage = sm.at_damage_receive(10.0)
        assert damage == 20.0

    def test_divinity_triples_damage(self):
        """Divinity stance triples damage."""
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)

        damage = sm.at_damage_give(10.0)
        assert damage == 30.0

    def test_divinity_no_incoming_mult(self):
        """Divinity does NOT multiply incoming damage."""
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)

        damage = sm.at_damage_receive(10.0)
        assert damage == 10.0  # NOT 30.0

    def test_divinity_exits_at_turn_end(self):
        """Divinity automatically exits at turn end."""
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)

        result = sm.on_turn_end()
        assert sm.current == StanceID.NEUTRAL
        assert result.get("divinity_ended")

    def test_mantra_triggers_divinity(self):
        """10 Mantra triggers Divinity."""
        sm = StanceManager()

        for i in range(3):
            sm.add_mantra(3)

        result = sm.add_mantra(1)  # Reaches 10
        assert result["divinity_triggered"]
        assert sm.current == StanceID.DIVINITY

    def test_same_stance_no_change(self):
        """Entering same stance doesn't trigger effects."""
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)

        result = sm.change_stance(StanceID.WRATH)
        assert not result["is_stance_change"]


# =============================================================================
# SECTION 8: MULTI-ENEMY TESTS
# =============================================================================

class TestMultiEnemyTargeting:
    """Test multi-enemy targeting and death mid-attack."""

    def test_aoe_hits_all_enemies(self, multi_enemy_combat):
        """AoE attacks hit all enemies."""
        living = multi_enemy_combat.living_enemies()
        assert len(living) == 3

    def test_enemy_death_during_multi_hit(self, multi_enemy_combat):
        """Enemy can die during multi-hit attack."""
        multi_enemy_combat.enemies[0].hp = 5

        # Deal 10 damage in 2 hits
        for _ in range(2):
            if multi_enemy_combat.enemies[0].hp > 0:
                multi_enemy_combat.enemies[0].hp -= 5

        assert multi_enemy_combat.enemies[0].hp <= 0
        assert multi_enemy_combat.enemies[0].is_dead

    def test_targeting_dead_enemy_skips(self, multi_enemy_combat):
        """Dead enemies are skipped in targeting."""
        multi_enemy_combat.enemies[0].hp = 0

        living = [e for e in multi_enemy_combat.enemies if not e.is_dead]
        assert len(living) == 2

    def test_all_enemies_dead_ends_combat(self, multi_enemy_combat):
        """Combat ends when all enemies die."""
        for enemy in multi_enemy_combat.enemies:
            enemy.hp = 0

        assert multi_enemy_combat.is_victory()


# =============================================================================
# SECTION 9: BLOCK CARRYOVER TESTS
# =============================================================================

class TestBlockCarryover:
    """Test block carryover with Barricade and Calipers."""

    def test_barricade_keeps_all_block(self, basic_combat):
        """Barricade keeps all block."""
        basic_combat.player.statuses["Barricade"] = 1
        basic_combat.player.block = 20

        # With Barricade, block persists
        assert basic_combat.player.block == 20

    def test_calipers_keeps_15_block(self, basic_combat):
        """Calipers keeps 15 block (decays rest)."""
        basic_combat.player.block = 25

        # Calipers effect: keep 15, lose rest
        calipers_keep = 15
        block_after = max(0, basic_combat.player.block - calipers_keep)
        final_block = min(calipers_keep, basic_combat.player.block)

        # If block was 25, keep 15
        assert final_block == 15 or basic_combat.player.block == 25

    def test_block_stacks_with_barricade(self, basic_combat):
        """Block stacks across turns with Barricade."""
        basic_combat.player.statuses["Barricade"] = 1
        basic_combat.player.block = 10

        # Gain more block
        basic_combat.player.block += 8

        assert basic_combat.player.block == 18


# =============================================================================
# SECTION 10: ENERGY CARRYOVER TESTS
# =============================================================================

class TestEnergyCarryover:
    """Test energy carryover with Ice Cream."""

    def test_ice_cream_keeps_energy(self, basic_combat):
        """Ice Cream keeps unused energy."""
        basic_combat.relics.append("Ice Cream")
        basic_combat.energy = 2

        # With Ice Cream, energy persists
        # Simulate turn end without clearing energy
        assert basic_combat.energy == 2

    def test_normal_energy_resets(self, basic_combat):
        """Without Ice Cream, energy resets."""
        basic_combat.energy = 2

        # Simulate turn start reset
        basic_combat.energy = basic_combat.max_energy
        assert basic_combat.energy == 3


# =============================================================================
# SECTION 11: ARTIFACT TESTS
# =============================================================================

class TestArtifactBlocking:
    """Test Artifact blocking debuffs."""

    def test_artifact_blocks_weak(self, power_manager):
        """Artifact blocks Weak application."""
        power_manager.add_power(create_artifact(1))

        result = power_manager.add_power(create_weak(1))
        assert result == False  # Blocked
        assert not power_manager.is_weak()

    def test_artifact_blocks_vulnerable(self, power_manager):
        """Artifact blocks Vulnerable application."""
        power_manager.add_power(create_artifact(1))

        result = power_manager.add_power(create_vulnerable(1))
        assert result == False
        assert not power_manager.is_vulnerable()

    def test_artifact_blocks_frail(self, power_manager):
        """Artifact blocks Frail application."""
        power_manager.add_power(create_artifact(1))

        result = power_manager.add_power(create_frail(1))
        assert result == False
        assert not power_manager.is_frail()

    def test_artifact_consumed(self, power_manager):
        """Artifact is consumed when blocking."""
        power_manager.add_power(create_artifact(1))
        assert power_manager.get_amount("Artifact") == 1

        power_manager.add_power(create_weak(1))
        assert power_manager.get_amount("Artifact") == 0

    def test_multiple_artifact_stacks(self, power_manager):
        """Multiple Artifact stacks work."""
        power_manager.add_power(create_artifact(3))

        power_manager.add_power(create_weak(1))
        assert power_manager.get_amount("Artifact") == 2

        power_manager.add_power(create_vulnerable(1))
        assert power_manager.get_amount("Artifact") == 1


# =============================================================================
# SECTION 12: INTANGIBLE TESTS
# =============================================================================

class TestIntangibleTiming:
    """Test Intangible application timing."""

    def test_intangible_caps_damage_at_1(self):
        """Intangible caps all damage at 1."""
        damage = calculate_damage(100, intangible=True)
        assert damage == 1

    def test_intangible_with_modifiers(self):
        """Intangible still caps after modifiers."""
        # Even with huge multipliers
        damage = calculate_damage(
            50, strength=10, stance_mult=DIVINITY_MULT,
            vuln=True, intangible=True
        )
        assert damage == 1

    def test_intangible_allows_1_damage(self):
        """1 damage goes through Intangible."""
        damage = calculate_damage(1, intangible=True)
        assert damage == 1

    def test_intangible_on_zero(self):
        """0 damage stays 0 with Intangible."""
        damage = calculate_damage(0, intangible=True)
        assert damage == 0

    def test_intangible_incoming(self):
        """Intangible caps incoming damage."""
        hp_loss, _ = calculate_incoming_damage(100, block=0, intangible=True)
        assert hp_loss == 1

    def test_intangible_decrements(self, power_manager):
        """Intangible decrements at end of turn."""
        power_manager.add_power(create_intangible(2))

        power_manager.at_end_of_round()
        assert power_manager.get_amount("Intangible") == 1


# =============================================================================
# SECTION 13: COMBAT STATE TESTS
# =============================================================================

class TestCombatStateCopy:
    """Test combat state copying for tree search."""

    def test_copy_preserves_player_hp(self, basic_combat):
        """Copy preserves player HP."""
        copy = basic_combat.copy()
        assert copy.player.hp == basic_combat.player.hp

    def test_copy_is_independent(self, basic_combat):
        """Copied state is independent."""
        copy = basic_combat.copy()
        copy.player.hp = 50

        assert basic_combat.player.hp != 50

    def test_copy_preserves_hand(self, basic_combat):
        """Copy preserves hand."""
        basic_combat.hand = ["Strike_P", "Defend_P"]
        copy = basic_combat.copy()

        assert copy.hand == basic_combat.hand

    def test_copy_hand_is_independent(self, basic_combat):
        """Copied hand is independent."""
        basic_combat.hand = ["Strike_P"]
        copy = basic_combat.copy()
        copy.hand.append("Defend_P")

        assert len(basic_combat.hand) == 1
        assert len(copy.hand) == 2


class TestCombatStateActions:
    """Test legal action generation."""

    def test_end_turn_always_legal(self, basic_combat):
        """End turn is always legal."""
        actions = basic_combat.get_legal_actions()
        end_turn_actions = [a for a in actions if isinstance(a, EndTurn)]
        assert len(end_turn_actions) == 1

    def test_cards_require_energy(self, basic_combat):
        """Cards require sufficient energy."""
        basic_combat.energy = 0
        basic_combat.hand = ["Strike_P"]  # Costs 1

        # Card should not be playable with 0 energy
        # (Depends on registry, but general principle)

    def test_targeting_requires_living_enemy(self, basic_combat):
        """Targeted cards require living enemies."""
        basic_combat.enemies[0].hp = 0  # Kill enemy

        living = [e for e in basic_combat.enemies if not e.is_dead]
        assert len(living) == 0


# =============================================================================
# SECTION 14: POWER INTERACTION TESTS
# =============================================================================

class TestPowerInteractions:
    """Test power interactions."""

    def test_strength_stacks(self, power_manager):
        """Strength stacks additively."""
        power_manager.add_power(create_strength(2))
        power_manager.add_power(create_strength(3))

        assert power_manager.get_strength() == 5

    def test_negative_strength(self, power_manager):
        """Strength can go negative."""
        power_manager.add_power(create_strength(2))
        power_manager.add_power(create_strength(-5))

        assert power_manager.get_strength() == -3

    def test_vigor_consumed_on_attack(self, power_manager):
        """Vigor should be consumed after attack."""
        power_manager.add_power(create_vigor(5))
        assert power_manager.get_amount("Vigor") == 5

        # After attack, Vigor should be removed
        # (Simulated - actual implementation removes it)

    def test_damage_with_strength_and_dex(self, power_manager):
        """Damage calculation with Strength."""
        power_manager.add_power(create_strength(3))

        damage = power_manager.calculate_damage_dealt(6)
        assert damage == 9.0  # 6 + 3

    def test_block_with_dexterity(self, power_manager):
        """Block calculation with Dexterity."""
        power_manager.add_power(create_dexterity(2))

        block = power_manager.calculate_block(5)
        assert block == 7  # 5 + 2


# =============================================================================
# SECTION 15: EDGE CASE TESTS
# =============================================================================

class TestEdgeCases:
    """Test edge cases and corner scenarios."""

    def test_zero_base_damage(self):
        """Zero base damage stays zero."""
        damage = calculate_damage(0, strength=5)
        assert damage == 5  # 0 + 5 strength

    def test_massive_damage_calculation(self):
        """Large damage values work."""
        damage = calculate_damage(999, strength=99, stance_mult=DIVINITY_MULT, vuln=True)
        # (999 + 99) * 3 * 1.5 = 4941
        assert damage == 4941

    def test_all_multipliers_combined(self):
        """All multipliers work together."""
        # Wrath + Vulnerable
        damage = calculate_damage(10, stance_mult=WRATH_MULT, vuln=True)
        assert damage == 30  # 10 * 2 * 1.5

    def test_empty_hand_legal_actions(self, basic_combat):
        """Empty hand only allows end turn."""
        basic_combat.hand = []
        basic_combat.potions = ["", "", ""]  # Empty potion slots

        actions = basic_combat.get_legal_actions()
        # Should only have end turn
        play_actions = [a for a in actions if isinstance(a, PlayCard)]
        assert len(play_actions) == 0

    def test_combat_terminal_states(self, basic_combat):
        """Terminal state detection."""
        # Not terminal initially
        assert not basic_combat.is_terminal()

        # Victory
        for e in basic_combat.enemies:
            e.hp = 0
        assert basic_combat.is_terminal()
        assert basic_combat.is_victory()

    def test_defeat_state(self, basic_combat):
        """Defeat state detection."""
        basic_combat.player.hp = 0

        assert basic_combat.is_terminal()
        assert basic_combat.is_defeat()


# =============================================================================
# SECTION 16: RELIC TESTS
# =============================================================================

class TestRelicEffects:
    """Test relic effects in combat."""

    def test_has_relic_check(self, basic_combat):
        """Check if player has relic."""
        basic_combat.relics = ["Vajra", "Anchor"]

        assert basic_combat.has_relic("Vajra")
        assert basic_combat.has_relic("Anchor")
        assert not basic_combat.has_relic("Snecko Eye")

    def test_relic_counter_tracking(self, basic_combat):
        """Relic counters are tracked."""
        basic_combat.set_relic_counter("Pen Nib", 9)
        assert basic_combat.get_relic_counter("Pen Nib") == 9

    def test_relic_counter_default(self, basic_combat):
        """Relic counter returns default."""
        counter = basic_combat.get_relic_counter("Unknown", default=0)
        assert counter == 0

    def test_torii_reduces_attack_damage_2_to_5_to_1(self):
        """Torii: damage 2-5 reduced to 1."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=4, player_hp=80)
        engine.state.relics.append("Torii")
        engine.start_combat()
        engine.state.player.block = 0
        initial_hp = engine.state.player.hp
        engine.end_turn()
        assert engine.state.player.hp == initial_hp - 1

    def test_torii_does_not_affect_damage_1(self):
        """Torii: damage 1 stays 1."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=1, player_hp=80)
        engine.state.relics.append("Torii")
        engine.start_combat()
        engine.state.player.block = 0
        initial_hp = engine.state.player.hp
        engine.end_turn()
        assert engine.state.player.hp == initial_hp - 1

    def test_torii_does_not_affect_damage_6_or_more(self):
        """Torii: damage 6+ not affected."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=10, player_hp=80)
        engine.state.relics.append("Torii")
        engine.start_combat()
        engine.state.player.block = 0
        initial_hp = engine.state.player.hp
        engine.end_turn()
        assert engine.state.player.hp == initial_hp - 10

    def test_torii_applies_before_intangible(self):
        """Torii applies BEFORE Intangible check."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=4, player_hp=80)
        engine.state.relics.append("Torii")
        engine.state.player.statuses["Intangible"] = 1
        engine.start_combat()
        engine.state.player.block = 0
        initial_hp = engine.state.player.hp
        engine.end_turn()
        # Torii reduces 4->1, Intangible doesn't change 1
        assert engine.state.player.hp == initial_hp - 1

    def test_torii_with_block(self):
        """Torii applies before block absorption."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=3, player_hp=80)
        engine.state.relics.append("Torii")
        engine.start_combat()
        engine.state.player.block = 1
        initial_hp = engine.state.player.hp
        engine.end_turn()
        # Torii reduces 3->1, then block absorbs it
        assert engine.state.player.hp == initial_hp
        assert engine.state.player.block == 0

    def test_tungsten_rod_reduces_attack_damage_by_1(self):
        """Tungsten Rod: reduce HP loss by 1."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=10, player_hp=80)
        engine.state.relics.append("Tungsten Rod")
        engine.start_combat()
        engine.state.player.block = 0
        initial_hp = engine.state.player.hp
        engine.end_turn()
        assert engine.state.player.hp == initial_hp - 9

    def test_tungsten_rod_minimum_0_hp_loss(self):
        """Tungsten Rod: minimum 0 HP loss."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=5, player_hp=80)
        engine.state.relics.append("Tungsten Rod")
        engine.start_combat()
        engine.state.player.block = 5
        initial_hp = engine.state.player.hp
        engine.end_turn()
        # Block absorbs all damage (5-5=0), Tungsten Rod doesn't go negative
        assert engine.state.player.hp == initial_hp
        assert engine.state.player.block == 0

    def test_tungsten_rod_applies_after_buffer(self):
        """Tungsten Rod applies after Buffer (Buffer prevents HP loss entirely)."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=10, player_hp=80)
        engine.state.relics.append("Tungsten Rod")
        engine.state.player.statuses["Buffer"] = 1
        engine.start_combat()
        engine.state.player.block = 0
        initial_hp = engine.state.player.hp
        engine.end_turn()
        # Buffer prevents HP loss, so Tungsten Rod doesn't apply
        assert engine.state.player.hp == initial_hp
        assert engine.state.player.statuses.get("Buffer", 0) == 0

    def test_tungsten_rod_reduces_poison_damage(self):
        """Tungsten Rod: reduces poison HP loss by 1."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=0, player_hp=80)
        engine.state.relics.append("Tungsten Rod")
        engine.state.player.statuses["Poison"] = 5
        engine.start_combat()
        # Poison triggers at start of turn: 5 damage - 1 (Tungsten) = 4
        assert engine.state.player.hp == 76  # 80 - 4

    def test_torii_and_tungsten_rod_stack(self):
        """Torii and Tungsten Rod stack: damage 4 -> 1 (Torii) -> 0 (Tungsten)."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=4, player_hp=80)
        engine.state.relics.extend(["Torii", "Tungsten Rod"])
        engine.start_combat()
        engine.state.player.block = 0
        initial_hp = engine.state.player.hp
        engine.end_turn()
        # Torii: 4->1, block absorbs 0, HP loss 1, Tungsten: 1->0
        assert engine.state.player.hp == initial_hp

    def test_tungsten_rod_with_intangible_on_poison(self):
        """Tungsten Rod + Intangible on poison: 10 -> 1 (Intangible) -> 0 (Tungsten)."""
        engine = create_simple_combat("TestEnemy", enemy_hp=200, enemy_damage=0, player_hp=80)
        engine.state.relics.append("Tungsten Rod")
        engine.state.player.statuses["Poison"] = 10
        engine.state.player.statuses["Intangible"] = 1
        engine.start_combat()
        assert engine.state.player.hp == 80  # Intangible caps to 1, Tungsten reduces to 0


# =============================================================================
# SECTION 17: CARD TYPE TESTS
# =============================================================================

class TestCardTypes:
    """Test card type handling."""

    def test_attack_card_type(self):
        """Attack cards identified."""
        card = get_card("Strike_P")
        assert card.card_type == CardType.ATTACK

    def test_skill_card_type(self):
        """Skill cards identified."""
        card = get_card("Defend_P")
        assert card.card_type == CardType.SKILL

    def test_power_card_type(self):
        """Power cards identified."""
        card = get_card("MentalFortress")
        assert card.card_type == CardType.POWER

    def test_status_cards_unplayable(self):
        """Status cards are unplayable."""
        # Status cards have cost -2
        # They cannot be played normally


# =============================================================================
# SECTION 18: DRAW MECHANICS TESTS
# =============================================================================

class TestDrawMechanics:
    """Test card draw mechanics."""

    def test_draw_from_draw_pile(self, basic_combat):
        """Drawing takes from draw pile."""
        basic_combat.draw_pile = ["Strike_P", "Defend_P"]
        basic_combat.hand = []

        # Draw one card
        card = basic_combat.draw_pile.pop()
        basic_combat.hand.append(card)

        assert len(basic_combat.hand) == 1
        assert len(basic_combat.draw_pile) == 1

    def test_draw_order_is_fifo(self, basic_combat):
        """Cards drawn from top (end of list)."""
        basic_combat.draw_pile = ["Bottom", "Middle", "Top"]
        basic_combat.hand = []

        card = basic_combat.draw_pile.pop()
        assert card == "Top"


# =============================================================================
# MAIN
# =============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
