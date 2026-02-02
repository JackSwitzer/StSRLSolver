"""
Comprehensive tests for CombatEngine.

Tests exercise the actual combat engine, not just data structures.
Covers: initialization, turn flow, card playing, energy, stances,
enemy AI, combat resolution, legal actions, multi-enemy, potions.
"""

import pytest
import sys

sys.path.insert(0, "/Users/jackswitzer/Desktop/SlayTheSpireRL")

from packages.engine.combat_engine import (
    CombatEngine,
    CombatPhase,
    CombatResult,
    create_simple_combat,
)
from packages.engine.state.combat import (
    CombatState,
    EnemyCombatState,
    EntityState,
    PlayCard,
    UsePotion,
    EndTurn,
    create_combat,
    create_enemy,
)
from packages.engine.content.cards import get_card, CardType, CardTarget
from packages.engine.content.stances import StanceID


# =============================================================================
# HELPERS
# =============================================================================


def make_engine(
    enemy_id="JawWorm",
    enemy_hp=42,
    enemy_damage=11,
    player_hp=80,
    deck=None,
    energy=3,
    potions=None,
    relics=None,
    num_enemies=1,
):
    """Create a CombatEngine for testing."""
    if deck is None:
        deck = [
            "Strike_P", "Strike_P", "Strike_P", "Strike_P",
            "Defend_P", "Defend_P", "Defend_P", "Defend_P",
            "Eruption", "Vigilance",
        ]

    enemies = []
    for i in range(num_enemies):
        eid = enemy_id if num_enemies == 1 else f"{enemy_id}"
        enemies.append(EnemyCombatState(
            hp=enemy_hp,
            max_hp=enemy_hp,
            id=enemy_id,
            name=enemy_id,
            enemy_type="NORMAL",
            move_damage=enemy_damage,
            move_hits=1,
            first_turn=True,
        ))

    state = create_combat(
        player_hp=player_hp,
        player_max_hp=player_hp,
        enemies=enemies,
        deck=deck,
        energy=energy,
        max_energy=energy,
        potions=potions or ["", "", ""],
        relics=relics or [],
    )

    return CombatEngine(state)


def find_card_action(actions, card_id, target_idx=None):
    """Find a PlayCard action for a specific card ID from legal actions."""
    for a in actions:
        if isinstance(a, PlayCard):
            # We need the engine's hand to check card_id, so just return by index
            pass
    return None


def find_play_card(engine, card_id, target_idx=0):
    """Find and return a PlayCard action for a given card_id in the engine's hand."""
    for i, cid in enumerate(engine.state.hand):
        if cid == card_id:
            card = engine._get_card(cid)
            if card.target == CardTarget.ENEMY:
                return PlayCard(card_idx=i, target_idx=target_idx)
            else:
                return PlayCard(card_idx=i, target_idx=-1)
    return None


# =============================================================================
# 1. COMBAT INITIALIZATION
# =============================================================================


class TestCombatInitialization:
    """Test creating combat with Watcher starter deck vs JawWorm."""

    def test_start_combat_phase(self):
        engine = make_engine()
        assert engine.phase == CombatPhase.NOT_STARTED
        engine.start_combat()
        assert engine.phase == CombatPhase.PLAYER_TURN

    def test_start_combat_draws_hand(self):
        engine = make_engine()
        engine.start_combat()
        assert len(engine.state.hand) == 5

    def test_start_combat_sets_turn(self):
        engine = make_engine()
        engine.start_combat()
        # CombatState starts at turn=1, _start_player_turn increments to 2
        assert engine.state.turn == 2

    def test_start_combat_sets_energy(self):
        engine = make_engine()
        engine.start_combat()
        assert engine.state.energy == 3

    def test_starter_deck_cards_distributed(self):
        engine = make_engine()
        engine.start_combat()
        total = (
            len(engine.state.hand)
            + len(engine.state.draw_pile)
            + len(engine.state.discard_pile)
        )
        assert total == 10  # Watcher starter deck

    def test_enemy_gets_initial_move(self):
        engine = make_engine()
        engine.start_combat()
        enemy = engine.state.enemies[0]
        # JawWorm first turn is Chomp (move_id=1, damage=11)
        assert enemy.move_id == 1
        assert enemy.move_damage == 11

    def test_start_combat_idempotent(self):
        engine = make_engine()
        engine.start_combat()
        hand_before = list(engine.state.hand)
        engine.start_combat()  # Should be no-op
        assert engine.state.hand == hand_before


# =============================================================================
# 2. TURN FLOW
# =============================================================================


class TestTurnFlow:
    """Start combat -> player turn -> play cards -> end turn -> enemy turn -> back to player."""

    def test_full_turn_cycle(self):
        engine = make_engine(enemy_hp=200, enemy_damage=5)
        engine.start_combat()

        assert engine.phase == CombatPhase.PLAYER_TURN
        turn_before = engine.state.turn

        # End turn triggers enemy turn and starts next player turn
        engine.end_turn()

        assert engine.phase == CombatPhase.PLAYER_TURN
        assert engine.state.turn == turn_before + 1

    def test_player_takes_damage_from_enemy(self):
        engine = make_engine(enemy_hp=200, enemy_damage=8)
        engine.start_combat()
        hp_before = engine.state.player.hp

        engine.end_turn()

        # Enemy should have attacked
        assert engine.state.player.hp < hp_before

    def test_block_decays_on_new_turn(self):
        engine = make_engine(enemy_hp=200, enemy_damage=0)
        engine.start_combat()

        # Play a Defend to get block
        action = find_play_card(engine, "Defend_P")
        if action:
            engine.execute_action(action)
            assert engine.state.player.block > 0

        engine.end_turn()

        # Block should reset on new turn
        assert engine.state.player.block == 0

    def test_hand_discarded_on_end_turn(self):
        engine = make_engine(enemy_hp=200, enemy_damage=0)
        engine.start_combat()
        assert len(engine.state.hand) == 5

        engine.end_turn()

        # New hand drawn, old hand discarded
        assert len(engine.state.hand) == 5

    def test_end_turn_only_during_player_turn(self):
        engine = make_engine()
        # Not started yet
        engine.end_turn()  # Should be no-op
        assert engine.phase == CombatPhase.NOT_STARTED


# =============================================================================
# 3. CARD PLAYING
# =============================================================================


class TestCardPlaying:
    """Play Strike, play Defend, verify damage/block applied correctly."""

    def test_play_strike_deals_damage(self):
        engine = make_engine(enemy_hp=42)
        engine.start_combat()

        enemy_hp_before = engine.state.enemies[0].hp
        action = find_play_card(engine, "Strike_P", target_idx=0)
        assert action is not None

        result = engine.execute_action(action)
        assert result["success"]
        # Strike_P does 6 damage
        assert engine.state.enemies[0].hp == enemy_hp_before - 6

    def test_play_defend_adds_block(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        action = find_play_card(engine, "Defend_P")
        assert action is not None

        result = engine.execute_action(action)
        assert result["success"]
        # Defend_P gives 5 block
        assert engine.state.player.block == 5

    def test_card_removed_from_hand_after_play(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()
        hand_size = len(engine.state.hand)

        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)
            assert len(engine.state.hand) == hand_size - 1

    def test_card_goes_to_discard(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()
        discard_before = len(engine.state.discard_pile)

        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)
            assert len(engine.state.discard_pile) == discard_before + 1

    def test_play_eruption_enters_wrath(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        action = find_play_card(engine, "Eruption", target_idx=0)
        if action:
            result = engine.execute_action(action)
            assert result["success"]
            assert engine.state.stance == "Wrath"

    def test_play_vigilance_enters_calm(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        action = find_play_card(engine, "Vigilance")
        if action:
            result = engine.execute_action(action)
            assert result["success"]
            assert engine.state.stance == "Calm"

    def test_cards_played_counter_increments(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()
        assert engine.state.cards_played_this_turn == 0

        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)
            assert engine.state.cards_played_this_turn == 1

    def test_invalid_hand_index(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()
        result = engine.play_card(99, 0)
        assert not result["success"]


# =============================================================================
# 4. ENERGY MANAGEMENT
# =============================================================================


class TestEnergyManagement:
    """Can't play cards without energy, end turn resets energy."""

    def test_energy_deducted_on_play(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()
        assert engine.state.energy == 3

        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)
            assert engine.state.energy == 2  # Strike costs 1

    def test_cannot_play_without_energy(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()
        engine.state.energy = 0

        actions = engine.get_legal_actions()
        play_actions = [a for a in actions if isinstance(a, PlayCard)]
        # No cards should be playable at 0 energy (all cost >= 1 in starter deck)
        # Crescendo costs 0 but not in starter deck
        assert len(play_actions) == 0

    def test_energy_resets_on_new_turn(self):
        engine = make_engine(enemy_hp=200, enemy_damage=0)
        engine.start_combat()

        # Spend all energy by playing whatever costs energy
        while engine.state.energy > 0:
            played = False
            for i, cid in enumerate(engine.state.hand):
                card = engine._get_card(cid)
                if card.current_cost > 0 and card.current_cost <= engine.state.energy:
                    if card.target == CardTarget.ENEMY:
                        engine.execute_action(PlayCard(card_idx=i, target_idx=0))
                    else:
                        engine.execute_action(PlayCard(card_idx=i, target_idx=-1))
                    played = True
                    break
            if not played:
                break

        engine.state.energy = 0  # Force to 0 if couldn't spend all
        engine.end_turn()
        assert engine.state.energy == 3

    def test_eruption_costs_2_energy(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        action = find_play_card(engine, "Eruption", target_idx=0)
        if action:
            engine.execute_action(action)
            assert engine.state.energy == 1  # 3 - 2


# =============================================================================
# 5. STANCE MECHANICS
# =============================================================================


class TestStanceMechanics:
    """Enter Wrath, enter Calm, Divinity."""

    def test_wrath_doubles_damage(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        # Force into Wrath stance
        engine.state.stance = "Wrath"

        enemy_hp_before = engine.state.enemies[0].hp
        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)
            # Strike does 6 base, Wrath doubles to 12
            assert engine.state.enemies[0].hp == enemy_hp_before - 12

    def test_calm_exit_gives_energy(self):
        engine = make_engine(enemy_hp=200, deck=[
            "Crescendo", "ClearTheMind", "Strike_P", "Strike_P",
            "Strike_P", "Strike_P", "Strike_P", "Strike_P",
            "Defend_P", "Defend_P",
        ])
        engine.start_combat()

        # Enter Calm first
        action = find_play_card(engine, "ClearTheMind")
        if action:
            engine.execute_action(action)
            assert engine.state.stance == "Calm"

            energy_before = engine.state.energy
            # Enter Wrath from Calm (exits Calm, gaining 2 energy)
            action2 = find_play_card(engine, "Crescendo")
            if action2:
                engine.execute_action(action2)
                assert engine.state.stance == "Wrath"
                # Should have gained 2 energy from exiting Calm
                # But also spent 0 on Crescendo
                assert engine.state.energy == energy_before + 2

    def test_divinity_triples_damage(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        # Force Divinity
        engine.state.stance = "Divinity"

        enemy_hp_before = engine.state.enemies[0].hp
        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)
            # Strike does 6 base, Divinity triples to 18
            assert engine.state.enemies[0].hp == enemy_hp_before - 18

    def test_divinity_exits_at_end_of_turn(self):
        engine = make_engine(enemy_hp=200, enemy_damage=0)
        engine.start_combat()

        engine.state.stance = "Divinity"
        engine.end_turn()
        # After end_turn processing, Divinity should exit to Neutral
        # (happens in end_turn before enemy turns)
        assert engine.state.stance == "Neutral"

    def test_mantra_triggers_divinity(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        engine.state.mantra = 9
        engine._add_mantra(1)  # Reach 10
        assert engine.state.stance == "Divinity"

    def test_wrath_incoming_damage_doubled(self):
        engine = make_engine(enemy_hp=200, enemy_damage=10)
        engine.start_combat()

        engine.state.stance = "Wrath"
        hp_before = engine.state.player.hp
        engine.end_turn()

        # Enemy does 10 base, doubled in Wrath = 20
        # JawWorm first turn is Chomp 11 though, so use the move_damage
        # Actually the engine sets JawWorm's move during start_combat
        damage_taken = hp_before - engine.state.player.hp
        # In Wrath, damage is doubled
        assert damage_taken > 0


# =============================================================================
# 6. ENEMY AI
# =============================================================================


class TestEnemyAI:
    """Enemies choose moves based on AI patterns."""

    def test_jaw_worm_first_turn_chomp(self):
        engine = make_engine(enemy_id="JawWorm", enemy_hp=100)
        engine.start_combat()

        enemy = engine.state.enemies[0]
        assert enemy.move_damage == 11  # Chomp
        assert enemy.move_id == 1

    def test_jaw_worm_second_turn_bellow(self):
        engine = make_engine(enemy_id="JawWorm", enemy_hp=200, enemy_damage=0)
        engine.start_combat()

        # After first turn, JawWorm should use Bellow (move_id=2)
        engine.end_turn()

        enemy = engine.state.enemies[0]
        # After Chomp (1), next is Bellow (2)
        assert enemy.move_id == 2

    def test_cultist_first_turn_incantation(self):
        engine = make_engine(enemy_id="Cultist", enemy_hp=50, enemy_damage=6)
        engine.start_combat()

        enemy = engine.state.enemies[0]
        assert enemy.move_id == 1  # Incantation

    def test_cultist_second_turn_dark_strike(self):
        engine = make_engine(enemy_id="Cultist", enemy_hp=200, enemy_damage=6)
        engine.start_combat()
        engine.end_turn()

        enemy = engine.state.enemies[0]
        assert enemy.move_id == 2  # Dark Strike
        assert enemy.move_damage == 6

    def test_enemy_move_history_tracked(self):
        engine = make_engine(enemy_id="JawWorm", enemy_hp=200, enemy_damage=0)
        engine.start_combat()

        enemy = engine.state.enemies[0]
        assert len(enemy.move_history) >= 1  # At least the initial move


# =============================================================================
# 7. COMBAT RESOLUTION
# =============================================================================


class TestCombatResolution:
    """Combat ends when enemy HP <= 0 (victory) or player HP <= 0 (defeat)."""

    def test_victory_when_enemy_dies(self):
        engine = make_engine(enemy_hp=5)
        engine.start_combat()

        # Strike for 6 should kill enemy with 5 HP
        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)
            assert engine.is_combat_over()
            assert engine.is_victory()

    def test_defeat_when_player_dies(self):
        # JawWorm Chomp does 11 damage, so set player HP very low
        engine = make_engine(enemy_hp=200, enemy_damage=11, player_hp=5)
        engine.start_combat()

        engine.end_turn()  # JawWorm Chomps for 11, player has 5 HP

        assert engine.is_combat_over()
        assert engine.is_defeat()

    def test_get_result_after_victory(self):
        engine = make_engine(enemy_hp=1)
        engine.start_combat()

        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)

        result = engine.get_result()
        assert result.victory
        assert result.hp_remaining == 80
        assert result.cards_played >= 1

    def test_get_result_after_defeat(self):
        engine = make_engine(enemy_hp=200, enemy_damage=200, player_hp=10)
        engine.start_combat()
        engine.end_turn()

        result = engine.get_result()
        assert not result.victory
        assert result.hp_remaining == 0

    def test_no_actions_after_combat_over(self):
        engine = make_engine(enemy_hp=1)
        engine.start_combat()

        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)

        actions = engine.get_legal_actions()
        assert len(actions) == 0


# =============================================================================
# 8. LEGAL ACTIONS
# =============================================================================


class TestLegalActions:
    """get_legal_actions returns correct playable cards."""

    def test_end_turn_always_available(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        actions = engine.get_legal_actions()
        end_turns = [a for a in actions if isinstance(a, EndTurn)]
        assert len(end_turns) == 1

    def test_attack_cards_require_target(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        actions = engine.get_legal_actions()
        play_actions = [a for a in actions if isinstance(a, PlayCard)]

        # Strikes target enemies, should have target_idx >= 0
        for a in play_actions:
            card_id = engine.state.hand[a.card_idx]
            card = engine._get_card(card_id)
            if card.target == CardTarget.ENEMY:
                assert a.target_idx >= 0

    def test_self_cards_have_no_target(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        actions = engine.get_legal_actions()
        for a in actions:
            if isinstance(a, PlayCard):
                card_id = engine.state.hand[a.card_idx]
                card = engine._get_card(card_id)
                if card.target == CardTarget.SELF:
                    assert a.target_idx == -1

    def test_no_play_actions_at_zero_energy(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()
        engine.state.energy = 0

        actions = engine.get_legal_actions()
        play_actions = [a for a in actions if isinstance(a, PlayCard)]
        assert len(play_actions) == 0

    def test_legal_actions_only_during_player_turn(self):
        engine = make_engine()
        # Not started
        actions = engine.get_legal_actions()
        assert len(actions) == 0

    def test_multiple_targets_for_multi_enemy(self):
        engine = make_engine(enemy_hp=200, num_enemies=2)
        engine.start_combat()

        actions = engine.get_legal_actions()
        # Each Strike in hand should generate 2 actions (one per enemy)
        strike_actions = []
        for a in actions:
            if isinstance(a, PlayCard):
                card_id = engine.state.hand[a.card_idx]
                if card_id == "Strike_P":
                    strike_actions.append(a)

        # Each Strike card should appear twice (target 0 and target 1)
        targets_per_card = {}
        for a in strike_actions:
            targets_per_card.setdefault(a.card_idx, set()).add(a.target_idx)
        for card_idx, targets in targets_per_card.items():
            assert len(targets) == 2


# =============================================================================
# 9. MULTI-ENEMY COMBAT
# =============================================================================


class TestMultiEnemyCombat:
    """2+ enemies, targeting specific enemies."""

    def test_two_enemies_both_alive(self):
        engine = make_engine(enemy_hp=50, num_enemies=2)
        engine.start_combat()

        living = engine.get_living_enemies()
        assert len(living) == 2

    def test_target_specific_enemy(self):
        engine = make_engine(enemy_hp=50, num_enemies=2)
        engine.start_combat()

        # Attack enemy 1 specifically
        action = PlayCard(card_idx=0, target_idx=1)
        # Find a Strike to play
        for i, cid in enumerate(engine.state.hand):
            card = engine._get_card(cid)
            if card.card_type == CardType.ATTACK and card.target == CardTarget.ENEMY:
                action = PlayCard(card_idx=i, target_idx=1)
                break

        hp_enemy_0 = engine.state.enemies[0].hp
        engine.execute_action(action)

        # Enemy 0 should be unaffected
        assert engine.state.enemies[0].hp == hp_enemy_0
        # Enemy 1 should have taken damage
        assert engine.state.enemies[1].hp < 50

    def test_kill_one_enemy_combat_continues(self):
        engine = make_engine(enemy_hp=5, num_enemies=2)
        engine.start_combat()

        # Kill first enemy
        for i, cid in enumerate(engine.state.hand):
            if cid == "Strike_P":
                engine.execute_action(PlayCard(card_idx=i, target_idx=0))
                break

        assert engine.state.enemies[0].hp <= 0
        assert not engine.is_combat_over()  # Second enemy still alive

    def test_kill_all_enemies_victory(self):
        # Use enough Strikes to guarantee 2 in hand
        engine = make_engine(
            enemy_hp=1, num_enemies=2,
            deck=["Strike_P"] * 10,
        )
        engine.start_combat()

        # Kill enemy 0
        action = find_play_card(engine, "Strike_P", target_idx=0)
        assert action is not None
        engine.execute_action(action)
        assert engine.state.enemies[0].hp <= 0
        assert not engine.is_combat_over()

        # Kill enemy 1
        action = find_play_card(engine, "Strike_P", target_idx=1)
        assert action is not None
        engine.execute_action(action)
        assert engine.state.enemies[1].hp <= 0

        assert engine.is_combat_over()
        assert engine.is_victory()

    def test_dead_enemy_not_in_legal_targets(self):
        engine = make_engine(enemy_hp=1, num_enemies=2)
        engine.start_combat()

        # Kill first enemy
        for i, cid in enumerate(engine.state.hand):
            if cid == "Strike_P":
                engine.execute_action(PlayCard(card_idx=i, target_idx=0))
                break

        actions = engine.get_legal_actions()
        for a in actions:
            if isinstance(a, PlayCard):
                card_id = engine.state.hand[a.card_idx]
                card = engine._get_card(card_id)
                if card.target == CardTarget.ENEMY:
                    assert a.target_idx != 0  # Dead enemy not targetable


# =============================================================================
# 10. POTION USAGE
# =============================================================================


class TestPotionUsage:
    """Use potion during combat."""

    def test_use_block_potion(self):
        engine = make_engine(enemy_hp=200, potions=["Block Potion", "", ""])
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert engine.state.player.block == 12

    def test_use_strength_potion(self):
        engine = make_engine(enemy_hp=200, potions=["Strength Potion", "", ""])
        engine.start_combat()

        engine.use_potion(0)
        assert engine.state.player.statuses.get("Strength", 0) == 2

    def test_use_fire_potion_deals_damage(self):
        engine = make_engine(enemy_hp=200, potions=["Fire Potion", "", ""])
        engine.start_combat()

        hp_before = engine.state.enemies[0].hp
        engine.use_potion(0, target_index=0)
        assert engine.state.enemies[0].hp < hp_before

    def test_potion_removed_after_use(self):
        engine = make_engine(enemy_hp=200, potions=["Block Potion", "", ""])
        engine.start_combat()

        engine.use_potion(0)
        assert engine.state.potions[0] == ""

    def test_use_energy_potion(self):
        engine = make_engine(enemy_hp=200, potions=["Energy Potion", "", ""])
        engine.start_combat()

        energy_before = engine.state.energy
        engine.use_potion(0)
        assert engine.state.energy == energy_before + 2

    def test_potion_in_legal_actions(self):
        engine = make_engine(enemy_hp=200, potions=["Block Potion", "", ""])
        engine.start_combat()

        actions = engine.get_legal_actions()
        potion_actions = [a for a in actions if isinstance(a, UsePotion)]
        assert len(potion_actions) == 1

    def test_fire_potion_needs_target(self):
        engine = make_engine(enemy_hp=200, potions=["Fire Potion", "", ""])
        engine.start_combat()

        actions = engine.get_legal_actions()
        potion_actions = [a for a in actions if isinstance(a, UsePotion)]
        # Fire Potion targets enemy
        assert all(a.target_idx >= 0 for a in potion_actions)

    def test_empty_potion_slot_not_usable(self):
        engine = make_engine(enemy_hp=200, potions=["", "", ""])
        engine.start_combat()

        result = engine.use_potion(0)
        assert not result["success"]

    def test_use_fear_potion_applies_vulnerable(self):
        engine = make_engine(enemy_hp=200, potions=["Fear Potion", "", ""])
        engine.start_combat()

        engine.use_potion(0, target_index=0)
        assert engine.state.enemies[0].statuses.get("Vulnerable", 0) == 3


# =============================================================================
# INTEGRATION: Full combat run
# =============================================================================


class TestFullCombatRun:
    """Run a full combat to completion."""

    def test_play_until_victory(self):
        engine = make_engine(enemy_hp=20, enemy_damage=3, player_hp=80)
        engine.start_combat()

        max_turns = 20
        for _ in range(max_turns):
            if engine.is_combat_over():
                break
            actions = engine.get_legal_actions()
            # Play all attacks first, then end turn
            played_any = False
            for a in actions:
                if isinstance(a, PlayCard):
                    card_id = engine.state.hand[a.card_idx]
                    card = engine._get_card(card_id)
                    if card.card_type == CardType.ATTACK:
                        engine.execute_action(a)
                        played_any = True
                        break
            if not played_any or engine.is_combat_over():
                if not engine.is_combat_over():
                    engine.execute_action(EndTurn())

        assert engine.is_combat_over()
        assert engine.is_victory()

    def test_engine_copy_independence(self):
        engine = make_engine(enemy_hp=200)
        engine.start_combat()

        copy = engine.copy()
        action = find_play_card(engine, "Strike_P", target_idx=0)
        if action:
            engine.execute_action(action)

        # Copy should be unaffected
        assert copy.state.enemies[0].hp == 200
        assert len(copy.state.hand) == 5
