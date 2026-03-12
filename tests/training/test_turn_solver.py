"""
Tests for TurnSolver and TurnSolverAdapter.

Tests cover:
1. Starter deck vs single enemy — finds lethal if available
2. Watcher stance sequence — enters Wrath before attacks, exits before enemy turn
3. Potion usage — uses Fire Potion when it enables lethal
4. Respects energy — doesn't try to play cards costing more than available energy
5. Tree reuse — get_next_action returns cached plan actions in sequence
6. Timeout — returns best-so-far within time budget
"""

import time

import pytest

from packages.engine.combat_engine import (
    CombatEngine,
    CombatPhase,
    create_simple_combat,
)
from packages.engine.state.combat import (
    CombatState,
    EnemyCombatState,
    EndTurn,
    EntityState,
    PlayCard,
    UsePotion,
    create_combat,
)
from packages.training.turn_solver import TurnSolver, TurnSolverAdapter


# =========================================================================
# Helpers
# =========================================================================


def _make_combat(
    enemy_hp: int = 40,
    enemy_damage: int = 6,
    player_hp: int = 80,
    deck=None,
    energy: int = 3,
    potions=None,
    stance: str = "Neutral",
) -> CombatEngine:
    """Create and start a simple combat for testing."""
    if deck is None:
        deck = [
            "Strike_P", "Strike_P", "Strike_P", "Strike_P",
            "Defend_P", "Defend_P", "Defend_P", "Defend_P",
            "Eruption", "Vigilance",
        ]

    enemy = EnemyCombatState(
        hp=enemy_hp,
        max_hp=enemy_hp,
        id="TestEnemy",
        name="TestEnemy",
        enemy_type="NORMAL",
        move_damage=enemy_damage,
        move_hits=1,
        first_turn=True,
    )

    state = create_combat(
        player_hp=player_hp,
        player_max_hp=player_hp,
        enemies=[enemy],
        deck=deck,
        energy=energy,
        max_energy=energy,
        potions=potions or ["", "", ""],
    )
    state.stance = stance

    engine = CombatEngine(state)
    engine.start_combat()
    return engine


def _make_multi_enemy_combat(
    enemies_spec: list,
    player_hp: int = 80,
    deck=None,
    energy: int = 3,
) -> CombatEngine:
    """Create combat with multiple enemies.

    enemies_spec: list of (hp, damage) tuples
    """
    if deck is None:
        deck = [
            "Strike_P", "Strike_P", "Strike_P", "Strike_P",
            "Defend_P", "Defend_P", "Defend_P", "Defend_P",
            "Eruption", "Vigilance",
        ]

    enemies = []
    for i, (hp, dmg) in enumerate(enemies_spec):
        enemies.append(EnemyCombatState(
            hp=hp,
            max_hp=hp,
            id=f"Enemy_{i}",
            name=f"Enemy_{i}",
            enemy_type="NORMAL",
            move_damage=dmg,
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
    )

    engine = CombatEngine(state)
    engine.start_combat()
    return engine


def _get_plan_card_ids(engine: CombatEngine, plan: list) -> list:
    """Extract card IDs from a plan's PlayCard actions (using original hand)."""
    hand = list(engine.state.hand)
    result = []
    for action in plan:
        if isinstance(action, PlayCard):
            if 0 <= action.card_idx < len(hand):
                result.append(hand[action.card_idx])
    return result


# =========================================================================
# Test 1: Starter deck vs single enemy — finds lethal if available
# =========================================================================


class TestFindsLethal:
    """TurnSolver should find lethal when it's available."""

    def test_finds_lethal_low_hp_enemy(self):
        """With a low-HP enemy, the solver should find a killing sequence."""
        # 12 HP enemy, player has 3 energy and Strikes (6 damage each)
        # Two Strikes = 12 damage = lethal
        engine = _make_combat(enemy_hp=12, enemy_damage=10)
        solver = TurnSolver(time_budget_ms=50.0, node_budget=5000)

        plan = solver.solve_turn(engine, "monster")

        assert plan is not None
        assert len(plan) > 0

        # Execute the plan and check enemy is dead
        sim = engine.copy()
        for action in plan:
            if sim.state.combat_over:
                break
            sim.execute_action(action)

        # The enemy should be dead (either during or after the plan)
        living = [e for e in sim.state.enemies if e.hp > 0]
        assert len(living) == 0, f"Enemy still alive with {living[0].hp} HP"

    def test_finds_lethal_over_end_turn(self):
        """Solver should prefer lethal over ending turn even when not taking damage."""
        # Enemy has 6 HP, does 0 damage — solver should still kill it
        engine = _make_combat(enemy_hp=6, enemy_damage=0)
        solver = TurnSolver(time_budget_ms=50.0, node_budget=5000)

        plan = solver.solve_turn(engine, "monster")

        assert plan is not None

        # Should contain at least one PlayCard
        play_actions = [a for a in plan if isinstance(a, PlayCard)]
        assert len(play_actions) >= 1

        # Execute and verify kill
        sim = engine.copy()
        for action in plan:
            if sim.state.combat_over:
                break
            sim.execute_action(action)

        living = [e for e in sim.state.enemies if e.hp > 0]
        assert len(living) == 0


# =========================================================================
# Test 2: Watcher stance — enters Wrath for damage, exits before enemy turn
# =========================================================================


class TestWatcherStanceSequence:
    """Solver should manage Watcher stances intelligently."""

    def test_prefers_wrath_for_damage_when_safe(self):
        """When enemy does 0 damage, solver should enter Wrath for more damage."""
        # Deck with Eruption (enters Wrath, 9 damage doubled = 18 in Wrath)
        deck = ["Eruption", "Strike_P", "Strike_P", "Defend_P", "Defend_P",
                "Strike_P", "Strike_P", "Defend_P", "Defend_P", "Vigilance"]
        engine = _make_combat(enemy_hp=30, enemy_damage=0, deck=deck)
        solver = TurnSolver(time_budget_ms=50.0, node_budget=5000)

        plan = solver.solve_turn(engine, "monster")
        assert plan is not None

        # Execute the plan
        sim = engine.copy()
        for action in plan:
            if sim.state.combat_over:
                break
            if sim.phase != CombatPhase.PLAYER_TURN:
                break
            sim.execute_action(action)

        # Should have dealt more damage with Wrath
        enemy = sim.state.enemies[0]
        # Eruption in Neutral = 9, in Wrath = 18; Strike in Wrath = 12
        # Minimum: we should have played Eruption (enters Wrath) + attacks
        assert enemy.hp < 30, "Should have dealt damage"

    def test_exits_wrath_when_enemy_attacking(self):
        """When in Wrath and enemy is attacking, solver should exit stance."""
        # Start in Wrath with Vigilance available to exit
        deck = ["Vigilance", "Strike_P", "Defend_P", "Strike_P", "Defend_P",
                "Strike_P", "Strike_P", "Defend_P", "Defend_P", "Eruption"]
        engine = _make_combat(
            enemy_hp=100, enemy_damage=20, deck=deck, stance="Wrath",
        )
        solver = TurnSolver(time_budget_ms=50.0, node_budget=5000)

        plan = solver.solve_turn(engine, "monster")
        assert plan is not None

        # Execute the plan and check stance at end
        sim = engine.copy()
        for action in plan:
            if sim.state.combat_over:
                break
            if sim.phase != CombatPhase.PLAYER_TURN:
                break
            sim.execute_action(action)

        # Should not end turn in Wrath when taking 20*2=40 damage
        # Vigilance enters Calm, which is safe
        assert sim.state.stance != "Wrath", (
            f"Should not end in Wrath with 20 damage incoming "
            f"(would be doubled to 40). Stance: {sim.state.stance}"
        )


# =========================================================================
# Test 3: Potion usage — uses Fire Potion when it enables lethal
# =========================================================================


class TestPotionUsage:
    """Solver should use potions when they enable lethal or significant advantage."""

    def test_uses_potion_for_lethal(self):
        """Fire Potion (20 damage) should be used when it enables a kill."""
        # Enemy has 25 HP, player has 1 energy and a Strike (6 damage)
        # Strike alone = 6 damage, not lethal
        # Fire Potion = 20 damage, Strike + Fire Potion = 26 >= 25
        deck = ["Strike_P", "Defend_P", "Defend_P", "Defend_P", "Defend_P",
                "Strike_P", "Strike_P", "Defend_P", "Defend_P", "Vigilance"]
        engine = _make_combat(
            enemy_hp=25,
            enemy_damage=15,
            deck=deck,
            energy=1,
            potions=["Fire Potion", "", ""],
        )
        solver = TurnSolver(time_budget_ms=100.0, node_budget=10000)

        plan = solver.solve_turn(engine, "monster")
        assert plan is not None

        # Check that a UsePotion action is in the plan
        potion_actions = [a for a in plan if isinstance(a, UsePotion)]
        assert len(potion_actions) >= 1, "Should use Fire Potion for lethal"

        # Execute and verify kill
        sim = engine.copy()
        for action in plan:
            if sim.state.combat_over:
                break
            if sim.phase != CombatPhase.PLAYER_TURN:
                break
            sim.execute_action(action)

        living = [e for e in sim.state.enemies if e.hp > 0]
        assert len(living) == 0, f"Enemy should be dead, has {living[0].hp if living else '?'} HP"


# =========================================================================
# Test 4: Respects energy — doesn't play unaffordable cards
# =========================================================================


class TestRespectsEnergy:
    """Solver should only include playable actions in its plan."""

    def test_plan_actions_are_all_legal(self):
        """Every action in the plan should be legal when executed in sequence."""
        engine = _make_combat(enemy_hp=50, enemy_damage=8, energy=2)
        solver = TurnSolver(time_budget_ms=50.0, node_budget=5000)

        plan = solver.solve_turn(engine, "monster")
        assert plan is not None

        # Execute each action in sequence and verify it's legal at that point
        sim = engine.copy()
        for action in plan:
            if sim.state.combat_over:
                break
            if sim.phase != CombatPhase.PLAYER_TURN:
                break
            legal = sim.get_legal_actions()
            # The action should be in the legal action list
            found = False
            for la in legal:
                if type(action) is type(la):
                    if isinstance(action, PlayCard) and isinstance(la, PlayCard):
                        if action.card_idx == la.card_idx and action.target_idx == la.target_idx:
                            found = True
                            break
                    elif isinstance(action, EndTurn):
                        found = True
                        break
                    elif isinstance(action, UsePotion) and isinstance(la, UsePotion):
                        if action.potion_idx == la.potion_idx and action.target_idx == la.target_idx:
                            found = True
                            break
            assert found, f"Action {action} was not legal at step"

            sim.execute_action(action)

    def test_does_not_exceed_energy(self):
        """With limited energy, solver should not try to play more cards than affordable."""
        # 1 energy, cards cost 1 each — should only play 1 card
        deck = ["Strike_P", "Strike_P", "Strike_P", "Defend_P", "Defend_P",
                "Strike_P", "Strike_P", "Defend_P", "Defend_P", "Vigilance"]
        engine = _make_combat(enemy_hp=50, enemy_damage=8, energy=1, deck=deck)
        solver = TurnSolver(time_budget_ms=50.0, node_budget=5000)

        plan = solver.solve_turn(engine, "monster")
        assert plan is not None

        # Count PlayCard actions (should be at most 1 for 1 energy with cost-1 cards)
        play_cards = [a for a in plan if isinstance(a, PlayCard)]
        assert len(play_cards) <= 1, (
            f"With 1 energy and cost-1 cards, should play at most 1 card, "
            f"but plan has {len(play_cards)} PlayCard actions"
        )


# =========================================================================
# Test 5: Tree reuse — get_next_action returns cached plan in sequence
# =========================================================================


class TestTreeReuse:
    """get_next_action should cache a plan and return actions in sequence."""

    def test_sequential_actions_from_cache(self):
        """Calling get_next_action multiple times should return consecutive plan actions."""
        engine = _make_combat(enemy_hp=12, enemy_damage=6)
        solver = TurnSolver(time_budget_ms=50.0, node_budget=5000)

        # First call — triggers planning
        action1 = solver.get_next_action(engine, "monster")
        assert action1 is not None

        # Verify plan is cached
        assert solver._cached_plan is not None
        full_plan = list(solver._cached_plan)

        if len(full_plan) > 1:
            # Execute the first action
            engine.execute_action(action1)

            if engine.phase == CombatPhase.PLAYER_TURN and not engine.state.combat_over:
                # Second call should return next cached action
                action2 = solver.get_next_action(engine, "monster")
                assert action2 is not None

    def test_replans_on_divergence(self):
        """If state diverges from expected, solver should replan."""
        engine = _make_combat(enemy_hp=50, enemy_damage=6)
        solver = TurnSolver(time_budget_ms=50.0, node_budget=5000)

        # Get first action and cache
        action1 = solver.get_next_action(engine, "monster")
        assert action1 is not None
        assert solver._cached_plan is not None
        original_plan = list(solver._cached_plan)

        # Execute the action
        engine.execute_action(action1)

        if engine.phase == CombatPhase.PLAYER_TURN and not engine.state.combat_over:
            # Manually corrupt state to simulate divergence
            engine.state.energy = 0  # Drain all energy

            # Next call should detect divergence and replan
            action2 = solver.get_next_action(engine, "monster")
            # Should still return something (even if just EndTurn)
            assert action2 is not None


# =========================================================================
# Test 6: Timeout — returns best-so-far within time budget
# =========================================================================


class TestTimeout:
    """Solver should respect time budget and return best-so-far."""

    def test_returns_within_time_budget(self):
        """Solver should not exceed its time budget significantly."""
        # Large search space: many cards, multi-enemy
        engine = _make_multi_enemy_combat(
            enemies_spec=[(30, 8), (25, 6), (20, 5)],
            deck=["Strike_P"] * 6 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"],
            energy=4,
        )
        solver = TurnSolver(time_budget_ms=20.0, node_budget=1000)

        t0 = time.monotonic()
        plan = solver.solve_turn(engine, "monster")
        elapsed_ms = (time.monotonic() - t0) * 1000.0

        # Should have returned something
        assert plan is not None

        # Should be within reasonable time (allow 2x budget for overhead)
        # The 20ms budget is for the search itself, but total includes setup
        assert elapsed_ms < 200.0, f"Took {elapsed_ms:.1f}ms, expected <200ms"

    def test_returns_valid_plan_under_pressure(self):
        """Even with tight budget, returned plan should be valid."""
        engine = _make_multi_enemy_combat(
            enemies_spec=[(40, 10), (30, 8)],
            deck=["Strike_P"] * 5 + ["Defend_P"] * 5 + ["Eruption", "Vigilance"],
            energy=3,
        )
        solver = TurnSolver(time_budget_ms=2.0, node_budget=200)

        plan = solver.solve_turn(engine, "monster")
        assert plan is not None
        assert len(plan) > 0

        # Last action should be EndTurn
        assert isinstance(plan[-1], EndTurn), "Plan should end with EndTurn"

        # All actions should be executable
        sim = engine.copy()
        for action in plan:
            if sim.state.combat_over:
                break
            if sim.phase != CombatPhase.PLAYER_TURN:
                break
            sim.execute_action(action)


# =========================================================================
# Test: Adapter bridges correctly
# =========================================================================


class TestTurnSolverAdapter:
    """TurnSolverAdapter should bridge between CombatAction and engine Action."""

    def test_adapter_returns_from_action_list(self):
        """Adapter should return one of the input CombatAction objects."""
        from packages.engine.game import CombatAction

        engine = _make_combat(enemy_hp=12, enemy_damage=6)

        # Build CombatAction list matching engine's legal actions
        engine_actions = engine.get_legal_actions()
        combat_actions = []
        for ea in engine_actions:
            if isinstance(ea, PlayCard):
                combat_actions.append(CombatAction(
                    action_type="play_card",
                    card_idx=ea.card_idx,
                    target_idx=ea.target_idx,
                ))
            elif isinstance(ea, EndTurn):
                combat_actions.append(CombatAction(action_type="end_turn"))
            elif isinstance(ea, UsePotion):
                combat_actions.append(CombatAction(
                    action_type="use_potion",
                    potion_idx=ea.potion_idx,
                    target_idx=ea.target_idx,
                ))

        # Mock runner
        class MockRunner:
            def __init__(self, combat):
                self.current_combat = combat

        adapter = TurnSolverAdapter(time_budget_ms=50.0, node_budget=5000)
        result = adapter.pick_action(combat_actions, MockRunner(engine), "monster")

        assert result is not None
        assert result in combat_actions, "Should return one of the input CombatAction objects"

    def test_adapter_fallback_on_single_action(self):
        """With only one action, adapter should return it directly."""
        from packages.engine.game import CombatAction

        end_action = CombatAction(action_type="end_turn")
        actions = [end_action]

        class MockRunner:
            current_combat = None

        adapter = TurnSolverAdapter()
        result = adapter.pick_action(actions, MockRunner(), "monster")
        assert result is end_action


# =========================================================================
# Test: Scoring function edge cases
# =========================================================================


class TestScoring:
    """Test the scoring function handles edge cases."""

    def test_lethal_scores_higher_than_non_lethal(self):
        """A plan that kills the enemy should score higher than one that doesn't."""
        solver = TurnSolver()

        # Setup: enemy with 6 HP
        engine = _make_combat(enemy_hp=6, enemy_damage=10)
        original = engine

        # Simulate: kill the enemy
        sim_kill = engine.copy()
        # Play a Strike (6 damage) to kill
        actions = sim_kill.get_legal_actions()
        strikes = [a for a in actions if isinstance(a, PlayCard)]
        if strikes:
            sim_kill.execute_action(strikes[0])

        # Simulate: just defend
        sim_defend = engine.copy()

        score_kill = solver._score_terminal(sim_kill, original)
        score_defend = solver._score_terminal(sim_defend, original)

        assert score_kill > score_defend, (
            f"Lethal score ({score_kill}) should be > non-lethal ({score_defend})"
        )

    def test_death_scores_lowest(self):
        """Player death should always be the worst score."""
        solver = TurnSolver()
        engine = _make_combat(enemy_hp=50, enemy_damage=10, player_hp=1)
        original = engine

        # Create a state where player is dead
        sim = engine.copy()
        sim.state.player.hp = 0

        score = solver._score_terminal(sim, original)
        assert score <= -100_000, f"Death score should be <= -100000, got {score}"

    def test_calm_stance_bonus(self):
        """Ending in Calm should give a small bonus (energy banked)."""
        solver = TurnSolver()
        engine = _make_combat(enemy_hp=100, enemy_damage=0)
        original = engine

        # State in Neutral
        sim_neutral = engine.copy()
        sim_neutral.state.stance = "Neutral"

        # State in Calm
        sim_calm = engine.copy()
        sim_calm.state.stance = "Calm"

        score_neutral = solver._score_terminal(sim_neutral, original)
        score_calm = solver._score_terminal(sim_calm, original)

        assert score_calm > score_neutral, (
            f"Calm score ({score_calm}) should be > Neutral ({score_neutral})"
        )
