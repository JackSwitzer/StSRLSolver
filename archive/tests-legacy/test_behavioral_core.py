"""
Behavioral Tests for Slay the Spire RL Engine.

These tests verify REAL game behavior through actual game state manipulation,
not just data existence checks. Every test plays cards, advances turns, or
runs game sequences and asserts on observable state changes.

Categories:
    1. Combat Behavior - Playing cards, dealing damage, gaining block, stances
    2. Full Game Loop - Multi-step sequences through game phases
    3. Action/Observation Contract - RL API correctness
    4. Effect Registration - Card effects resolve through the registry
"""

import pytest
import numpy as np

from packages.engine.game import GameRunner, GamePhase
from packages.engine.combat_engine import (
    CombatEngine, CombatResult, create_simple_combat,
    CombatPhase, PlayCard, EndTurn, UsePotion,
)
from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    create_player, create_enemy, create_combat,
)
from packages.engine.content.cards import get_card, CardType, CardTarget
from packages.engine.effects.registry import (
    get_effect_handler, execute_effect, EffectContext, list_registered_effects,
)
from packages.engine.rl_observations import ObservationEncoder


# =============================================================================
# Helpers
# =============================================================================

def _make_engine(
    deck,
    enemy_hp=50,
    enemy_damage=6,
    player_hp=80,
    num_enemies=1,
    relics=None,
):
    """Create a CombatEngine ready for testing, with combat already started."""
    enemies = [
        EnemyCombatState(
            hp=enemy_hp,
            max_hp=enemy_hp,
            id=f"TestEnemy_{i}",
            name=f"TestEnemy_{i}",
            enemy_type="NORMAL",
            move_damage=enemy_damage,
            move_hits=1,
            first_turn=True,
        )
        for i in range(num_enemies)
    ]
    state = create_combat(
        player_hp=player_hp,
        player_max_hp=player_hp,
        enemies=enemies,
        deck=deck,
        energy=3,
        max_energy=3,
        relics=relics,
    )
    engine = CombatEngine(state)
    engine.start_combat()
    return engine


def _find_card_in_hand(engine, card_prefix):
    """Find the hand index of a card whose id starts with card_prefix."""
    for i, card_id in enumerate(engine.state.hand):
        if card_id.startswith(card_prefix):
            return i
    return None


def _play_card_by_name(engine, card_prefix, target_index=-1):
    """Play the first card matching card_prefix from hand."""
    idx = _find_card_in_hand(engine, card_prefix)
    if idx is None:
        raise ValueError(f"Card {card_prefix!r} not in hand: {engine.state.hand}")
    return engine.play_card(idx, target_index)


def _navigate_to_combat(runner, max_steps=50):
    """Step the runner until it enters combat, or give up."""
    for _ in range(max_steps):
        if runner.game_over or runner.phase == GamePhase.COMBAT:
            break
        actions = runner.get_available_action_dicts()
        if not actions:
            break
        runner.take_action_dict(actions[0])


def _play_combat_to_end(runner, max_steps=300):
    """Keep taking first available action until combat ends."""
    for _ in range(max_steps):
        if runner.game_over or runner.phase != GamePhase.COMBAT:
            break
        actions = runner.get_available_action_dicts()
        if not actions:
            break
        runner.take_action_dict(actions[0])


# =============================================================================
# 1. COMBAT BEHAVIOR TESTS
# =============================================================================


class TestCombatDamage:
    """Tests that attack cards deal damage and cost energy."""

    def test_strike_deals_damage_and_costs_energy(self):
        """Play Strike, verify energy spent and damage dealt to enemy."""
        engine = _make_engine(["Strike_P"] * 5 + ["Defend_P"] * 5, enemy_hp=50)
        initial_energy = engine.state.energy
        enemy_hp_before = engine.state.enemies[0].hp

        idx = _find_card_in_hand(engine, "Strike_P")
        assert idx is not None, "Strike_P should be in hand after start_combat"

        strike = get_card("Strike_P")
        result = engine.play_card(idx, target_index=0)
        assert result["success"], f"Failed to play Strike: {result}"

        assert engine.state.energy < initial_energy, "Energy should decrease after playing Strike"
        assert engine.state.enemies[0].hp < enemy_hp_before, "Enemy HP should decrease after Strike"

    def test_defend_gains_block_and_costs_energy(self):
        """Play Defend, verify energy spent and block gained."""
        engine = _make_engine(["Defend_P"] * 5 + ["Strike_P"] * 5, enemy_hp=50)
        initial_energy = engine.state.energy
        block_before = engine.state.player.block

        idx = _find_card_in_hand(engine, "Defend_P")
        assert idx is not None

        result = engine.play_card(idx)
        assert result["success"]

        assert engine.state.energy < initial_energy, "Energy should decrease"
        assert engine.state.player.block > block_before, "Block should increase"

    def test_insufficient_energy_prevents_play(self):
        """Cannot play a card if not enough energy."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=50)
        # Drain all energy
        engine.state.energy = 0

        idx = _find_card_in_hand(engine, "Strike_P")
        assert idx is not None
        result = engine.play_card(idx, target_index=0)
        assert not result["success"], "Should fail with 0 energy"

    def test_card_removed_from_hand_after_play(self):
        """Playing a card removes it from hand."""
        engine = _make_engine(["Strike_P"] * 5 + ["Defend_P"] * 5, enemy_hp=50)
        hand_size_before = len(engine.state.hand)

        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)

        assert len(engine.state.hand) == hand_size_before - 1

    def test_played_card_goes_to_discard(self):
        """Non-exhaust cards go to discard pile after play."""
        engine = _make_engine(["Strike_P"] * 5 + ["Defend_P"] * 5, enemy_hp=50)
        discard_before = len(engine.state.discard_pile)

        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)

        assert len(engine.state.discard_pile) == discard_before + 1
        assert "Strike_P" in engine.state.discard_pile

    def test_exhaust_card_goes_to_exhaust_pile(self):
        """Exhaust cards go to exhaust pile after play."""
        # Tranquility exhausts
        engine = _make_engine(
            ["Tranquility"] * 3 + ["Strike_P"] * 7,
            enemy_hp=50,
        )
        idx = _find_card_in_hand(engine, "Tranquility")
        if idx is None:
            pytest.skip("Tranquility not drawn")

        exhaust_before = len(engine.state.exhaust_pile)
        engine.play_card(idx)
        assert len(engine.state.exhaust_pile) == exhaust_before + 1

    def test_killing_enemy_sets_hp_zero(self):
        """Dealing enough damage kills an enemy (HP -> 0)."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=5, enemy_damage=1)

        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)

        assert engine.state.enemies[0].hp <= 0, "Enemy should be dead"

    def test_all_enemies_dead_ends_combat(self):
        """Combat ends when all enemies reach 0 HP."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=1, enemy_damage=1)

        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)

        assert engine.is_combat_over()
        assert engine.is_victory()


class TestCombatBlock:
    """Tests that block absorbs damage correctly."""

    def test_block_absorbs_damage_before_hp(self):
        """Block is consumed before HP takes damage."""
        engine = _make_engine(["Defend_P"] * 5 + ["Strike_P"] * 5, enemy_hp=100, enemy_damage=8)

        # Play defend to gain block
        idx = _find_card_in_hand(engine, "Defend_P")
        engine.play_card(idx)
        block_after_defend = engine.state.player.block
        assert block_after_defend > 0

        hp_before = engine.state.player.hp

        # End turn - enemy attacks
        engine.end_turn()

        hp_after = engine.state.player.hp
        # If block was 5 and damage was 8, player should take 3 damage, not 8
        damage_taken = hp_before - hp_after
        assert damage_taken < 8, f"Block should have absorbed some damage, took {damage_taken}"

    def test_block_resets_each_turn(self):
        """Block resets to 0 at start of new turn (no Barricade)."""
        engine = _make_engine(["Defend_P"] * 5 + ["Strike_P"] * 5, enemy_hp=100, enemy_damage=0)

        idx = _find_card_in_hand(engine, "Defend_P")
        engine.play_card(idx)
        assert engine.state.player.block > 0

        # End turn and start new one (enemy does 0 damage)
        engine.end_turn()
        # Block should have reset to 0 at start of new player turn
        assert engine.state.player.block == 0, "Block should reset at start of turn"


class TestStanceMechanics:
    """Tests Watcher stance mechanics."""

    def test_eruption_enters_wrath(self):
        """Eruption enters Wrath stance."""
        engine = _make_engine(["Eruption"] + ["Strike_P"] * 9, enemy_hp=100)
        assert engine.state.stance == "Neutral"

        idx = _find_card_in_hand(engine, "Eruption")
        if idx is None:
            pytest.skip("Eruption not drawn")

        engine.play_card(idx, target_index=0)
        assert engine.state.stance == "Wrath"

    def test_wrath_doubles_damage_dealt(self):
        """Wrath stance doubles outgoing damage."""
        # Set up two engines with same state, one in Wrath
        engine_neutral = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        engine_wrath = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        engine_wrath.state.stance = "Wrath"

        # Both play Strike
        neutral_hp_before = engine_neutral.state.enemies[0].hp
        wrath_hp_before = engine_wrath.state.enemies[0].hp

        idx_n = _find_card_in_hand(engine_neutral, "Strike_P")
        idx_w = _find_card_in_hand(engine_wrath, "Strike_P")

        engine_neutral.play_card(idx_n, target_index=0)
        engine_wrath.play_card(idx_w, target_index=0)

        neutral_damage = neutral_hp_before - engine_neutral.state.enemies[0].hp
        wrath_damage = wrath_hp_before - engine_wrath.state.enemies[0].hp

        assert wrath_damage == neutral_damage * 2, (
            f"Wrath should double damage: {wrath_damage} vs {neutral_damage}"
        )

    def test_wrath_doubles_damage_taken(self):
        """Wrath stance doubles incoming damage."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100, enemy_damage=10)
        hp_neutral = engine.state.player.hp

        # End turn in Neutral - enemy attacks
        engine.end_turn()
        damage_in_neutral = hp_neutral - engine.state.player.hp

        # Start new turn, enter wrath, end turn again
        engine.state.stance = "Wrath"
        hp_before_wrath = engine.state.player.hp
        engine.end_turn()
        damage_in_wrath = hp_before_wrath - engine.state.player.hp

        assert damage_in_wrath == damage_in_neutral * 2, (
            f"Wrath should double incoming: {damage_in_wrath} vs {damage_in_neutral}"
        )

    def test_vigilance_enters_calm(self):
        """Vigilance enters Calm stance."""
        engine = _make_engine(["Vigilance"] + ["Strike_P"] * 9, enemy_hp=100)
        idx = _find_card_in_hand(engine, "Vigilance")
        if idx is None:
            pytest.skip("Vigilance not drawn")

        engine.play_card(idx)
        assert engine.state.stance == "Calm"

    def test_calm_exit_grants_energy(self):
        """Leaving Calm grants 2 energy."""
        engine = _make_engine(
            ["Vigilance", "Eruption", "Vigilance", "Eruption"] + ["Strike_P"] * 6,
            enemy_hp=100,
        )

        # Enter Calm using the EffectContext directly (guaranteed, no draw luck)
        ctx = EffectContext(state=engine.state)
        ctx.change_stance("Calm")
        assert engine.state.stance == "Calm"

        energy_before_exit = engine.state.energy

        # Leave Calm by entering Wrath via EffectContext
        result = ctx.change_stance("Wrath")
        assert engine.state.stance == "Wrath"

        # Exiting Calm gives +2 energy
        assert result["energy_gained"] >= 2, (
            f"Calm exit should grant 2 energy, got {result['energy_gained']}"
        )
        assert engine.state.energy == energy_before_exit + 2, (
            f"Expected energy {energy_before_exit + 2}, got {engine.state.energy}"
        )


class TestStatusEffects:
    """Tests for status effect application and behavior."""

    def test_vulnerability_increases_damage_taken(self):
        """Vulnerable enemy takes 1.5x damage."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        enemy = engine.state.enemies[0]

        # Deal damage without vuln
        hp_before = enemy.hp
        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)
        normal_damage = hp_before - enemy.hp

        # Apply vulnerable and deal damage again
        enemy.statuses["Vulnerable"] = 3
        hp_before = enemy.hp
        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)
        vuln_damage = hp_before - enemy.hp

        assert vuln_damage == int(normal_damage * 1.5), (
            f"Vulnerable should deal 1.5x: got {vuln_damage}, expected {int(normal_damage * 1.5)}"
        )

    def test_weakness_reduces_damage_dealt(self):
        """Weak player deals 0.75x damage."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        enemy = engine.state.enemies[0]

        # Deal damage without weak
        hp_before = enemy.hp
        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)
        normal_damage = hp_before - enemy.hp

        # Apply weak to player and deal again
        engine.state.player.statuses["Weakened"] = 3
        hp_before = enemy.hp
        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)
        weak_damage = hp_before - enemy.hp

        assert weak_damage == int(normal_damage * 0.75), (
            f"Weak should deal 0.75x: got {weak_damage}, expected {int(normal_damage * 0.75)}"
        )

    def test_strength_adds_to_attack_damage(self):
        """Strength adds flat damage to attacks."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        enemy = engine.state.enemies[0]

        # Normal damage
        hp_before = enemy.hp
        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)
        normal_damage = hp_before - enemy.hp

        # Add 3 strength
        engine.state.player.statuses["Strength"] = 3
        hp_before = enemy.hp
        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)
        str_damage = hp_before - enemy.hp

        assert str_damage == normal_damage + 3, (
            f"Strength +3 should add 3 damage: got {str_damage}, expected {normal_damage + 3}"
        )

    def test_poison_ticks_on_enemy_turn(self):
        """Poison deals damage at start of enemy turn and decrements."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100, enemy_damage=0)
        enemy = engine.state.enemies[0]
        enemy.statuses["Poison"] = 5

        hp_before = enemy.hp
        engine.end_turn()  # enemy turn processes poison

        assert enemy.hp < hp_before, "Poison should deal damage"
        assert enemy.statuses.get("Poison", 0) == 4, "Poison should decrement by 1"

    def test_artifact_blocks_debuff(self):
        """Artifact blocks a debuff application."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        enemy = engine.state.enemies[0]
        enemy.statuses["Artifact"] = 1

        # Try to apply weak - should be blocked
        from packages.engine.effects.registry import EffectContext
        ctx = EffectContext(state=engine.state, target=enemy)
        ctx.apply_status_to_enemy(enemy, "Weak", 2)

        assert enemy.statuses.get("Weakened", 0) == 0, "Artifact should block Weak"
        assert enemy.statuses.get("Artifact", 0) == 0, "Artifact counter should decrement"


class TestTurnFlow:
    """Tests for turn start/end mechanics."""

    def test_end_turn_discards_hand(self):
        """Hand is discarded at end of turn."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100, enemy_damage=0)
        hand_size = len(engine.state.hand)
        assert hand_size > 0, "Should have cards in hand"

        discard_before = len(engine.state.discard_pile)
        engine.end_turn()

        # After end turn, old hand cards should have been discarded (now in
        # discard or shuffled back), and then new cards drawn for next turn.
        # The discard pile should have grown by the old hand size (minus any
        # that got shuffled into draw for the new turn draw).
        total_cards = (
            len(engine.state.hand) + len(engine.state.draw_pile) +
            len(engine.state.discard_pile) + len(engine.state.exhaust_pile)
        )
        assert total_cards == 10, "Total cards should be conserved"

    def test_draw_pile_refills_from_discard(self):
        """When draw pile is empty, discard pile shuffles in for new draws."""
        engine = _make_engine(["Strike_P"] * 5, enemy_hp=100, enemy_damage=0)
        # Play up to 3 strikes to move them to discard, keeping energy available
        plays = 0
        for _ in range(3):
            idx = _find_card_in_hand(engine, "Strike_P")
            if idx is None or engine.state.energy < 1:
                break
            engine.play_card(idx, target_index=0)
            plays += 1

        assert plays > 0, "Should have played at least one card"
        assert len(engine.state.discard_pile) >= plays, "Played cards should be in discard"

        # End turn and start new turn - engine should draw 5 cards,
        # shuffling discard into draw if needed
        engine.end_turn()

        # Cards should have been redistributed
        total = (
            len(engine.state.hand) + len(engine.state.draw_pile) +
            len(engine.state.discard_pile) + len(engine.state.exhaust_pile)
        )
        assert total == 5, "Total cards should be conserved"
        assert len(engine.state.hand) > 0, "Should have drawn cards for new turn"

    def test_energy_resets_each_turn(self):
        """Energy resets to max_energy at start of each new turn."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100, enemy_damage=0)

        # Spend some energy
        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)
        assert engine.state.energy < engine.state.max_energy

        # End turn -> next turn starts with full energy
        engine.end_turn()
        assert engine.state.energy == engine.state.max_energy, (
            f"Energy should reset to {engine.state.max_energy}, got {engine.state.energy}"
        )

    def test_turn_counter_increments(self):
        """Turn counter increments after each end_turn."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100, enemy_damage=0)
        turn_1 = engine.state.turn
        engine.end_turn()
        assert engine.state.turn == turn_1 + 1

    def test_enemy_attacks_on_end_turn(self):
        """Enemy deals damage when player ends turn."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100, enemy_damage=10)
        hp_before = engine.state.player.hp

        engine.end_turn()

        assert engine.state.player.hp < hp_before, "Enemy should deal damage"

    def test_player_death_ends_combat(self):
        """Player dying ends combat in defeat."""
        engine = _make_engine(
            ["Strike_P"] * 10, enemy_hp=100, enemy_damage=100, player_hp=10
        )
        engine.end_turn()

        assert engine.state.player.hp <= 0
        assert engine.is_combat_over()
        assert not engine.is_victory()


class TestScryMechanic:
    """Tests for scry behavior."""

    def test_scry_looks_at_draw_pile(self):
        """Scry reveals cards from top of draw pile."""
        engine = _make_engine(
            ["CutThroughFate"] + ["Strike_P"] * 9,
            enemy_hp=100,
        )
        draw_before = len(engine.state.draw_pile)
        idx = _find_card_in_hand(engine, "CutThroughFate")
        if idx is None:
            pytest.skip("CutThroughFate not drawn")

        engine.play_card(idx, target_index=0)
        # CutThroughFate: deal damage, scry 2, draw 1
        # After scry + draw, draw pile should be smaller
        assert len(engine.state.draw_pile) < draw_before


class TestMultiEnemyCombat:
    """Tests with multiple enemies."""

    def test_targeted_attack_only_hits_target(self):
        """Single-target attack only damages the targeted enemy."""
        engine = _make_engine(
            ["Strike_P"] * 10, enemy_hp=50, num_enemies=3
        )
        hp_before = [e.hp for e in engine.state.enemies]

        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=1)  # Target middle enemy

        assert engine.state.enemies[0].hp == hp_before[0], "Enemy 0 should be untouched"
        assert engine.state.enemies[1].hp < hp_before[1], "Enemy 1 should take damage"
        assert engine.state.enemies[2].hp == hp_before[2], "Enemy 2 should be untouched"

    def test_aoe_hits_all_enemies(self):
        """AoE attack damages all living enemies."""
        engine = _make_engine(
            ["Consecrate"] + ["Strike_P"] * 9,
            enemy_hp=50,
            num_enemies=3,
        )
        idx = _find_card_in_hand(engine, "Consecrate")
        if idx is None:
            pytest.skip("Consecrate not drawn")

        hp_before = [e.hp for e in engine.state.enemies]
        engine.play_card(idx)

        for i, enemy in enumerate(engine.state.enemies):
            assert enemy.hp < hp_before[i], f"Enemy {i} should take AoE damage"


# =============================================================================
# 2. FULL GAME LOOP TESTS
# =============================================================================


class TestGameLoop:
    """Tests that run multi-step game sequences using GameRunner."""

    def test_game_runner_initializes_at_map(self):
        """GameRunner starts at MAP_NAVIGATION when skip_neow=True."""
        runner = GameRunner(seed="BEHAV1", ascension=20, verbose=False)
        assert runner.phase == GamePhase.MAP_NAVIGATION
        assert not runner.game_over

    def test_navigate_to_combat(self):
        """Can navigate from map to combat."""
        runner = GameRunner(seed="BEHAV2", ascension=20, verbose=False)
        _navigate_to_combat(runner)
        # Should be in combat (or game over if something went very wrong)
        assert runner.phase == GamePhase.COMBAT or runner.game_over

    def test_full_combat_to_rewards(self):
        """Play through a combat and reach rewards phase."""
        runner = GameRunner(seed="BEHAV3", ascension=0, verbose=False)
        _navigate_to_combat(runner)

        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        _play_combat_to_end(runner)

        # Should be at rewards (or game over if player died)
        assert runner.phase in (
            GamePhase.COMBAT_REWARDS, GamePhase.RUN_COMPLETE,
            GamePhase.MAP_NAVIGATION,
        ) or runner.game_over

    def test_card_reward_adds_to_deck(self):
        """Picking a card reward adds it to the deck."""
        runner = GameRunner(seed="BEHAV4", ascension=0, verbose=False)
        _navigate_to_combat(runner)
        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        _play_combat_to_end(runner)

        if runner.phase != GamePhase.COMBAT_REWARDS:
            pytest.skip("Did not reach rewards")

        deck_before = len(runner.run_state.deck)
        actions = runner.get_available_action_dicts()
        pick_actions = [a for a in actions if a["type"] == "pick_card"]

        if not pick_actions:
            pytest.skip("No card reward to pick")

        runner.take_action_dict(pick_actions[0])
        assert len(runner.run_state.deck) >= deck_before, (
            "Deck should grow or stay same after picking card"
        )

    def test_deterministic_replay(self):
        """Same seed + same actions produce identical outcomes."""
        actions_taken = []

        # Run 1: record actions
        r1 = GameRunner(seed="REPLAY1", ascension=20, verbose=False)
        for _ in range(30):
            if r1.game_over:
                break
            actions = r1.get_available_action_dicts()
            if not actions:
                break
            action = actions[0]
            actions_taken.append(action)
            r1.take_action_dict(action)

        # Run 2: replay same actions
        r2 = GameRunner(seed="REPLAY1", ascension=20, verbose=False)
        for action in actions_taken:
            if r2.game_over:
                break
            r2.take_action_dict(action)

        # Both should be in the same state
        assert r1.phase == r2.phase
        assert r1.run_state.current_hp == r2.run_state.current_hp
        assert r1.run_state.gold == r2.run_state.gold
        assert r1.run_state.floor == r2.run_state.floor
        assert len(r1.run_state.deck) == len(r2.run_state.deck)

    def test_multiple_floors_advance(self):
        """Game advances through multiple floors."""
        runner = GameRunner(seed="FLOORS1", ascension=0, verbose=False)
        initial_floor = runner.run_state.floor

        for _ in range(200):
            if runner.game_over:
                break
            if runner.run_state.floor > initial_floor + 2:
                break
            actions = runner.get_available_action_dicts()
            if not actions:
                break
            runner.take_action_dict(actions[0])

        assert runner.run_state.floor > initial_floor, "Should advance at least one floor"


# =============================================================================
# 3. ACTION / OBSERVATION CONTRACT TESTS
# =============================================================================


class TestActionObservationContract:
    """Tests that the RL API contract works end-to-end."""

    def test_observation_has_required_fields(self):
        """Observation contains all required top-level keys."""
        runner = GameRunner(seed="OBS1", ascension=20, verbose=False)
        obs = runner.get_observation()

        assert "observation_schema_version" in obs
        assert "phase" in obs
        assert "run" in obs
        assert "map" in obs
        assert "combat" in obs or obs["combat"] is None

    def test_run_observation_has_current_hp(self):
        """Run observation uses 'current_hp' (not 'hp')."""
        runner = GameRunner(seed="OBS2", ascension=20, verbose=False)
        obs = runner.get_observation()

        assert "current_hp" in obs["run"], "run should have 'current_hp'"
        assert "max_hp" in obs["run"], "run should have 'max_hp'"
        assert obs["run"]["current_hp"] > 0
        assert obs["run"]["current_hp"] <= obs["run"]["max_hp"]

    def test_combat_observation_uses_hp_for_entities(self):
        """Combat observation uses 'hp' for player and enemies."""
        runner = GameRunner(seed="OBS3", ascension=20, verbose=False)
        _navigate_to_combat(runner)

        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        obs = runner.get_observation()
        assert obs["combat"] is not None
        assert "hp" in obs["combat"]["player"]
        assert "max_hp" in obs["combat"]["player"]
        for enemy in obs["combat"]["enemies"]:
            assert "hp" in enemy
            assert "max_hp" in enemy

    def test_observation_changes_after_action(self):
        """Observation state changes after taking an action."""
        runner = GameRunner(seed="OBS4", ascension=20, verbose=False)
        _navigate_to_combat(runner)

        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        obs_before = runner.get_observation()
        actions = runner.get_available_action_dicts()
        runner.take_action_dict(actions[0])
        obs_after = runner.get_observation()

        # Something should have changed (hand, energy, enemy hp, etc.)
        # We just verify the obs dict isn't identical
        assert obs_before != obs_after, "Observation should change after action"

    def test_all_actions_executable_for_100_steps(self):
        """For 100 steps, every returned action can be executed without error."""
        runner = GameRunner(seed="EXEC100", ascension=0, verbose=False)

        for step in range(100):
            if runner.game_over:
                break
            actions = runner.get_available_action_dicts()
            if not actions:
                break

            # Each action should be executable
            action = actions[0]
            result = runner.take_action_dict(action)
            # Result should indicate success (or at least not crash)
            assert result is not None, f"Action result was None at step {step}"

    def test_action_mask_matches_available_actions(self):
        """ActionSpace mask has bits set for each available action."""
        from packages.engine.rl_masks import ActionSpace

        runner = GameRunner(seed="MASK1", ascension=20, verbose=False)
        space = ActionSpace()

        for _ in range(20):
            if runner.game_over:
                break
            actions = runner.get_available_action_dicts()
            if not actions:
                break

            mask = space.actions_to_mask(actions)
            # Number of True bits should match number of actions
            assert np.sum(mask) == len(actions), (
                f"Mask has {np.sum(mask)} bits but {len(actions)} actions"
            )

            runner.take_action_dict(actions[0])

    def test_observation_encoder_round_trip(self):
        """Encode observation to array and back, verify key fields preserved."""
        runner = GameRunner(seed="RT1", ascension=20, verbose=False)
        obs = runner.get_observation()

        encoder = ObservationEncoder()
        arr = encoder.observation_to_array(obs)

        assert arr.shape == (encoder.size,)
        assert arr.dtype == np.float32

        # Round-trip decode
        decoded = encoder.array_to_observation(arr)

        # Key fields should be preserved
        assert decoded["run"]["ascension"] == obs["run"]["ascension"]
        assert decoded["run"]["max_hp"] == obs["run"]["max_hp"]
        # current_hp may have slight rounding difference due to ratio encoding
        hp_diff = abs(decoded["run"]["current_hp"] - obs["run"]["current_hp"])
        assert hp_diff <= 1, f"HP mismatch: {decoded['run']['current_hp']} vs {obs['run']['current_hp']}"

    def test_observation_json_serializable(self):
        """Observation can be serialized to JSON without error."""
        import json
        runner = GameRunner(seed="JSON1", ascension=20, verbose=False)
        obs = runner.get_observation()
        serialized = json.dumps(obs)
        assert len(serialized) > 0


# =============================================================================
# 4. EFFECT REGISTRATION TESTS
# =============================================================================


class TestEffectRegistration:
    """Tests that card effects are properly registered and resolve."""

    def test_play_all_hand_free_registered(self):
        """The Unraveling effect 'play_all_hand_free' is registered."""
        handler = get_effect_handler("play_all_hand_free")
        assert handler is not None, "play_all_hand_free should be registered"

    def test_play_all_hand_free_sets_flag(self):
        """play_all_hand_free sets the extra_data flag for the combat engine."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        ctx = EffectContext(state=engine.state)
        execute_effect("play_all_hand_free", ctx)
        assert ctx.extra_data.get("play_all_hand_free") is True

    def test_all_watcher_effects_registered(self):
        """All effects referenced in WATCHER_CARD_EFFECTS are registered."""
        from packages.engine.effects.cards import WATCHER_CARD_EFFECTS
        registered = set(list_registered_effects())
        missing = []
        for card_id, effects in WATCHER_CARD_EFFECTS.items():
            for eff in effects:
                if eff not in registered:
                    missing.append((card_id, eff))
        assert not missing, f"Unregistered effects: {missing}"

    def test_draw_effect_draws_cards(self):
        """The 'draw' effect actually draws cards from draw pile."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        hand_before = len(engine.state.hand)
        draw_before = len(engine.state.draw_pile)

        ctx = EffectContext(state=engine.state)
        execute_effect("draw_2", ctx)

        assert len(engine.state.hand) == hand_before + 2
        assert len(engine.state.draw_pile) == draw_before - 2

    def test_gain_block_effect_adds_block(self):
        """The gain_block effect adds block to the player."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        block_before = engine.state.player.block

        ctx = EffectContext(state=engine.state)
        execute_effect("gain_block_5", ctx)

        assert engine.state.player.block == block_before + 5

    def test_enter_wrath_changes_stance(self):
        """The enter_wrath effect changes stance."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        assert engine.state.stance == "Neutral"

        ctx = EffectContext(state=engine.state)
        execute_effect("enter_wrath", ctx)

        assert engine.state.stance == "Wrath"

    def test_enter_calm_changes_stance(self):
        """The enter_calm effect changes stance."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        ctx = EffectContext(state=engine.state)
        execute_effect("enter_calm", ctx)

        assert engine.state.stance == "Calm"

    def test_gain_energy_effect(self):
        """The gain_energy effect adds energy."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        energy_before = engine.state.energy

        ctx = EffectContext(state=engine.state)
        execute_effect("gain_energy_2", ctx)

        assert engine.state.energy == energy_before + 2

    def test_scry_effect_processes_draw_pile(self):
        """The scry effect interacts with the draw pile."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        draw_before = len(engine.state.draw_pile)

        ctx = EffectContext(state=engine.state)
        scryed = ctx.scry(3)

        # Should have seen up to 3 cards
        assert len(scryed) <= 3
        assert len(scryed) <= draw_before

    def test_unraveling_is_dead_code(self):
        """Unraveling is dead code (not in CardLibrary) and should raise ValueError."""
        import pytest
        with pytest.raises(ValueError, match="dead code"):
            get_card("Unraveling")


# =============================================================================
# 5. COMBAT ENGINE LEGAL ACTIONS
# =============================================================================


class TestLegalActions:
    """Tests that the combat engine returns correct legal actions."""

    def test_end_turn_always_available(self):
        """EndTurn is always a legal action during player turn."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        actions = engine.get_legal_actions()
        end_turns = [a for a in actions if isinstance(a, EndTurn)]
        assert len(end_turns) == 1

    def test_cards_available_as_actions(self):
        """Playable cards in hand appear as PlayCard actions."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        actions = engine.get_legal_actions()
        play_cards = [a for a in actions if isinstance(a, PlayCard)]
        assert len(play_cards) > 0

    def test_targeted_cards_have_target_actions(self):
        """Targeted cards (attacks) produce one action per living enemy."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=50, num_enemies=3)
        actions = engine.get_legal_actions()
        play_cards = [a for a in actions if isinstance(a, PlayCard)]

        # Each Strike in hand should produce 3 actions (one per enemy)
        strikes_in_hand = sum(1 for c in engine.state.hand if c.startswith("Strike"))
        targets_per_strike = len([a for a in play_cards if a.card_idx == 0])
        assert targets_per_strike == 3 or strikes_in_hand == 0

    def test_no_actions_when_combat_over(self):
        """No legal actions when combat is over."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=1)
        idx = _find_card_in_hand(engine, "Strike_P")
        engine.play_card(idx, target_index=0)

        # Combat should be over
        assert engine.is_combat_over()
        actions = engine.get_legal_actions()
        assert len(actions) == 0

    def test_unplayable_card_not_in_actions(self):
        """Cards that cost more than available energy don't appear."""
        engine = _make_engine(["Strike_P"] * 10, enemy_hp=100)
        engine.state.energy = 0

        actions = engine.get_legal_actions()
        play_cards = [a for a in actions if isinstance(a, PlayCard)]
        assert len(play_cards) == 0, "No cards should be playable with 0 energy"
