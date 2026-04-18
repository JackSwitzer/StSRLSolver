"""
Looter & Mugger Flee Pattern Tests.

Verifies the multi-turn MUG → MUG → (SMOKE_BOMB|LUNGE) → SMOKE_BOMB → ESCAPE
pattern from Java decompiled source.

Java sources:
  decompiled/java-src/com/megacrit/cardcrawl/monsters/exordium/Looter.java
  decompiled/java-src/com/megacrit/cardcrawl/monsters/city/Mugger.java
"""

import pytest
from packages.engine.content.enemies import (
    Looter, Mugger, MoveInfo, Intent,
)
from packages.engine.state.rng import Random


def make_rng(seed=42):
    return Random(seed)


def roll_n_moves(enemy, n):
    """Roll n moves, returning list of MoveInfo objects."""
    moves = []
    for _ in range(n):
        move = enemy.roll_move()
        moves.append(move)
    return moves


# ============================================================
# Looter AI Pattern
# ============================================================


class TestLooterFleePattern:
    """
    Java: exordium/Looter.java
    Move IDs: MUG=1, SMOKE_BOMB=2, ESCAPE=3, LUNGE=4
    """

    def test_move_ids_match_java(self):
        assert Looter.MUG == 1
        assert Looter.SMOKE_BOMB == 2
        assert Looter.ESCAPE == 3
        assert Looter.LUNGE == 4

    def test_initial_move_is_mug(self):
        """Java getMove() always returns MUG for initial move."""
        e = Looter(make_rng(), 0, make_rng())
        move = e.roll_move()
        assert move.move_id == Looter.MUG
        assert move.intent == Intent.ATTACK

    def test_second_move_is_mug(self):
        """After first MUG (slashCount < 2), set next to MUG."""
        e = Looter(make_rng(), 0, make_rng())
        moves = roll_n_moves(e, 2)
        assert moves[0].move_id == Looter.MUG
        assert moves[1].move_id == Looter.MUG

    def test_third_move_is_smoke_bomb_or_lunge(self):
        """After 2nd MUG (slashCount==2): 50% SMOKE_BOMB / 50% LUNGE."""
        e = Looter(make_rng(), 0, make_rng())
        moves = roll_n_moves(e, 3)
        assert moves[0].move_id == Looter.MUG
        assert moves[1].move_id == Looter.MUG
        assert moves[2].move_id in (Looter.SMOKE_BOMB, Looter.LUNGE)

    def test_after_lunge_is_smoke_bomb(self):
        """After LUNGE: force SMOKE_BOMB next turn."""
        # Try many seeds to find one where the 3rd move is LUNGE
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 4)
            if moves[2].move_id == Looter.LUNGE:
                assert moves[3].move_id == Looter.SMOKE_BOMB, \
                    f"After LUNGE, expected SMOKE_BOMB but got {moves[3].move_id}"
                return
        pytest.fail("Could not find a seed producing LUNGE on turn 3")

    def test_after_smoke_bomb_is_escape(self):
        """After SMOKE_BOMB: force ESCAPE next turn."""
        # Try seeds to find one where 3rd move is SMOKE_BOMB
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 4)
            if moves[2].move_id == Looter.SMOKE_BOMB:
                assert moves[3].move_id == Looter.ESCAPE, \
                    f"After SMOKE_BOMB, expected ESCAPE but got {moves[3].move_id}"
                assert moves[3].intent == Intent.ESCAPE
                return
        pytest.fail("Could not find a seed producing SMOKE_BOMB on turn 3")

    def test_full_sequence_with_lunge(self):
        """Full sequence: MUG -> MUG -> LUNGE -> SMOKE_BOMB -> ESCAPE."""
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 5)
            if moves[2].move_id == Looter.LUNGE:
                assert [m.move_id for m in moves] == [
                    Looter.MUG, Looter.MUG, Looter.LUNGE,
                    Looter.SMOKE_BOMB, Looter.ESCAPE
                ]
                return
        pytest.fail("Could not find a seed producing LUNGE path")

    def test_full_sequence_without_lunge(self):
        """Shorter sequence: MUG -> MUG -> SMOKE_BOMB -> ESCAPE."""
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 4)
            if moves[2].move_id == Looter.SMOKE_BOMB:
                assert [m.move_id for m in moves] == [
                    Looter.MUG, Looter.MUG, Looter.SMOKE_BOMB,
                    Looter.ESCAPE
                ]
                return
        pytest.fail("Could not find a seed producing SMOKE_BOMB path")

    def test_escape_repeats(self):
        """After ESCAPE, enemy keeps returning ESCAPE."""
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 6)
            if moves[2].move_id == Looter.SMOKE_BOMB:
                # MUG, MUG, SMOKE_BOMB, ESCAPE, ESCAPE, ESCAPE
                assert moves[3].move_id == Looter.ESCAPE
                assert moves[4].move_id == Looter.ESCAPE
                assert moves[5].move_id == Looter.ESCAPE
                return
        pytest.fail("Could not find a suitable seed")

    def test_smoke_bomb_has_block(self):
        """SMOKE_BOMB should grant 6 block (Java: escapeDef = 6)."""
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Looter.SMOKE_BOMB:
                assert moves[2].block == 6
                return
        pytest.fail("Could not find a seed producing SMOKE_BOMB on turn 3")

    def test_escape_has_escape_effect(self):
        """ESCAPE move should have escape effect to trigger enemy fleeing."""
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 4)
            if moves[2].move_id == Looter.SMOKE_BOMB:
                assert moves[3].effects.get("escape") is True
                return
        pytest.fail("Could not find a suitable seed")

    def test_lunge_damage_values(self):
        """LUNGE should use lunge damage (12 base, 14 at A2+)."""
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Looter.LUNGE:
                assert moves[2].base_damage == 12  # A0
                return

        pytest.fail("Could not find a seed producing LUNGE on turn 3")

    def test_lunge_damage_a2(self):
        """At A2+, lunge damage should be 14."""
        for seed in range(200):
            e = Looter(make_rng(seed), 2, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Looter.LUNGE:
                assert moves[2].base_damage == 14
                return
        pytest.fail("Could not find a seed producing LUNGE on turn 3")

    def test_mug_damage_values(self):
        """MUG should use swipe damage (10 base, 11 at A2+)."""
        e = Looter(make_rng(), 0, make_rng())
        move = e.roll_move()
        assert move.base_damage == 10

        e2 = Looter(make_rng(), 2, make_rng())
        move2 = e2.roll_move()
        assert move2.base_damage == 11

    def test_hp_range_a0(self):
        """HP range: 44-48 at A0."""
        e = Looter(make_rng(), 0, make_rng())
        assert 44 <= e.state.current_hp <= 48

    def test_hp_range_a7(self):
        """HP range: 46-50 at A7+."""
        e = Looter(make_rng(), 7, make_rng())
        assert 46 <= e.state.current_hp <= 50

    def test_thievery_power_a0(self):
        """Thievery 15 gold at A0."""
        e = Looter(make_rng(), 0, make_rng())
        assert e.state.powers.get("thievery") == 15

    def test_thievery_power_a17(self):
        """Thievery 20 gold at A17+."""
        e = Looter(make_rng(), 17, make_rng())
        assert e.state.powers.get("thievery") == 20

    def test_both_paths_reachable(self):
        """Both SMOKE_BOMB and LUNGE paths should be reachable across seeds."""
        saw_smoke = False
        saw_lunge = False
        for seed in range(200):
            e = Looter(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Looter.SMOKE_BOMB:
                saw_smoke = True
            elif moves[2].move_id == Looter.LUNGE:
                saw_lunge = True
            if saw_smoke and saw_lunge:
                break
        assert saw_smoke, "Never saw SMOKE_BOMB path in 200 seeds"
        assert saw_lunge, "Never saw LUNGE path in 200 seeds"


# ============================================================
# Mugger AI Pattern
# ============================================================


class TestMuggerFleePattern:
    """
    Java: city/Mugger.java
    Move IDs: MUG=1, SMOKE_BOMB=2, ESCAPE=3, BIGSWIPE=4
    """

    def test_move_ids_match_java(self):
        assert Mugger.MUG == 1
        assert Mugger.SMOKE_BOMB == 2
        assert Mugger.ESCAPE == 3
        assert Mugger.BIGSWIPE == 4

    def test_initial_move_is_mug(self):
        e = Mugger(make_rng(), 0, make_rng())
        move = e.roll_move()
        assert move.move_id == Mugger.MUG
        assert move.intent == Intent.ATTACK

    def test_second_move_is_mug(self):
        e = Mugger(make_rng(), 0, make_rng())
        moves = roll_n_moves(e, 2)
        assert moves[0].move_id == Mugger.MUG
        assert moves[1].move_id == Mugger.MUG

    def test_third_move_is_smoke_bomb_or_bigswipe(self):
        e = Mugger(make_rng(), 0, make_rng())
        moves = roll_n_moves(e, 3)
        assert moves[2].move_id in (Mugger.SMOKE_BOMB, Mugger.BIGSWIPE)

    def test_after_bigswipe_is_smoke_bomb(self):
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 4)
            if moves[2].move_id == Mugger.BIGSWIPE:
                assert moves[3].move_id == Mugger.SMOKE_BOMB
                return
        pytest.fail("Could not find a seed producing BIGSWIPE on turn 3")

    def test_after_smoke_bomb_is_escape(self):
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 4)
            if moves[2].move_id == Mugger.SMOKE_BOMB:
                assert moves[3].move_id == Mugger.ESCAPE
                assert moves[3].intent == Intent.ESCAPE
                return
        pytest.fail("Could not find a seed producing SMOKE_BOMB on turn 3")

    def test_full_sequence_with_bigswipe(self):
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 5)
            if moves[2].move_id == Mugger.BIGSWIPE:
                assert [m.move_id for m in moves] == [
                    Mugger.MUG, Mugger.MUG, Mugger.BIGSWIPE,
                    Mugger.SMOKE_BOMB, Mugger.ESCAPE
                ]
                return
        pytest.fail("Could not find a seed producing BIGSWIPE path")

    def test_full_sequence_without_bigswipe(self):
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 4)
            if moves[2].move_id == Mugger.SMOKE_BOMB:
                assert [m.move_id for m in moves] == [
                    Mugger.MUG, Mugger.MUG, Mugger.SMOKE_BOMB,
                    Mugger.ESCAPE
                ]
                return
        pytest.fail("Could not find a seed producing SMOKE_BOMB path")

    def test_escape_repeats(self):
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 6)
            if moves[2].move_id == Mugger.SMOKE_BOMB:
                assert moves[3].move_id == Mugger.ESCAPE
                assert moves[4].move_id == Mugger.ESCAPE
                assert moves[5].move_id == Mugger.ESCAPE
                return
        pytest.fail("Could not find a suitable seed")

    def test_smoke_bomb_block_a0(self):
        """SMOKE_BOMB: 11 block at A0."""
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Mugger.SMOKE_BOMB:
                assert moves[2].block == 11
                return
        pytest.fail("Could not find a seed producing SMOKE_BOMB")

    def test_smoke_bomb_block_a17(self):
        """SMOKE_BOMB: 17 block at A17+ (Java: escapeDef + 6)."""
        for seed in range(200):
            e = Mugger(make_rng(seed), 17, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Mugger.SMOKE_BOMB:
                assert moves[2].block == 17
                return
        pytest.fail("Could not find a seed producing SMOKE_BOMB")

    def test_bigswipe_damage_a0(self):
        """BIGSWIPE: 16 damage at A0."""
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Mugger.BIGSWIPE:
                assert moves[2].base_damage == 16
                return
        pytest.fail("Could not find a seed producing BIGSWIPE")

    def test_bigswipe_damage_a2(self):
        """BIGSWIPE: 18 damage at A2+."""
        for seed in range(200):
            e = Mugger(make_rng(seed), 2, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Mugger.BIGSWIPE:
                assert moves[2].base_damage == 18
                return
        pytest.fail("Could not find a seed producing BIGSWIPE")

    def test_mug_damage_values(self):
        e = Mugger(make_rng(), 0, make_rng())
        move = e.roll_move()
        assert move.base_damage == 10

        e2 = Mugger(make_rng(), 2, make_rng())
        move2 = e2.roll_move()
        assert move2.base_damage == 11

    def test_hp_range_a0(self):
        e = Mugger(make_rng(), 0, make_rng())
        assert 48 <= e.state.current_hp <= 52

    def test_hp_range_a7(self):
        e = Mugger(make_rng(), 7, make_rng())
        assert 50 <= e.state.current_hp <= 54

    def test_thievery_power_a0(self):
        e = Mugger(make_rng(), 0, make_rng())
        assert e.state.powers.get("thievery") == 15

    def test_thievery_power_a17(self):
        e = Mugger(make_rng(), 17, make_rng())
        assert e.state.powers.get("thievery") == 20

    def test_both_paths_reachable(self):
        saw_smoke = False
        saw_bigswipe = False
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 3)
            if moves[2].move_id == Mugger.SMOKE_BOMB:
                saw_smoke = True
            elif moves[2].move_id == Mugger.BIGSWIPE:
                saw_bigswipe = True
            if saw_smoke and saw_bigswipe:
                break
        assert saw_smoke, "Never saw SMOKE_BOMB path in 200 seeds"
        assert saw_bigswipe, "Never saw BIGSWIPE path in 200 seeds"

    def test_escape_has_escape_effect(self):
        for seed in range(200):
            e = Mugger(make_rng(seed), 0, make_rng(seed))
            moves = roll_n_moves(e, 4)
            if moves[2].move_id == Mugger.SMOKE_BOMB:
                assert moves[3].effects.get("escape") is True
                return
        pytest.fail("Could not find a suitable seed")


# ============================================================
# Combat Integration: Escape Ends Combat
# ============================================================


class TestEscapeCombatIntegration:
    """Verify that when all enemies escape, combat ends as a victory."""

    def _make_combat_state(self, enemy_id, ascension=0, seed=42):
        """Create a minimal combat state with one Looter/Mugger."""
        from packages.engine.state.combat import (
            CombatState, EntityState, EnemyCombatState,
        )
        from packages.engine.combat_engine import CombatEngine
        from packages.engine.content.enemies import create_enemy

        rng = Random(seed)
        enemy_obj = create_enemy(enemy_id, rng, ascension=ascension, hp_rng=Random(seed))

        player = EntityState(hp=50, max_hp=50, block=99)
        enemy_state = EnemyCombatState(
            hp=enemy_obj.state.current_hp,
            max_hp=enemy_obj.state.max_hp,
            block=0,
            statuses=dict(enemy_obj.state.powers),
            id=enemy_obj.ID,
            name=enemy_obj.NAME,
            enemy_type=str(enemy_obj.TYPE.value),
        )

        state = CombatState(
            player=player,
            energy=3,
            max_energy=3,
            hand=[],
            draw_pile=[],
            discard_pile=[],
            enemies=[enemy_state],
            relics={},
        )

        # Provide ai_rng so the engine delegates to real Enemy objects
        engine = CombatEngine(state, ai_rng=Random(seed))
        engine.enemy_objects = [enemy_obj]
        return engine

    def test_looter_escape_ends_combat(self):
        """When Looter escapes, combat should end (player wins)."""
        # Give player 99 block so they take no damage
        engine = self._make_combat_state("Looter", seed=42)
        engine.start_combat()

        # Run turns until combat ends (max 10 turns to avoid infinite loop)
        turns = 0
        while not engine.is_combat_over() and turns < 10:
            engine.end_turn()  # Player does nothing, ends turn
            turns += 1

        assert engine.is_combat_over(), f"Combat should be over after {turns} turns"
        # The Looter escaped, so the enemy list should show the enemy as escaped
        looter = engine.state.enemies[0]
        assert looter.is_escaping, "Looter should be marked as escaping"
        assert looter.hp > 0, "Looter should still have HP (escaped, not dead)"

    def test_mugger_escape_ends_combat(self):
        """When Mugger escapes, combat should end."""
        engine = self._make_combat_state("Mugger", seed=42)
        engine.start_combat()

        turns = 0
        while not engine.is_combat_over() and turns < 10:
            engine.end_turn()
            turns += 1

        assert engine.is_combat_over(), f"Combat should be over after {turns} turns"
        mugger = engine.state.enemies[0]
        assert mugger.is_escaping, "Mugger should be marked as escaping"
        assert mugger.hp > 0, "Mugger should still have HP (escaped, not dead)"
