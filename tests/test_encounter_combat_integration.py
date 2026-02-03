"""
Comprehensive tests for encounter-to-combat pipeline.

Covers:
1. Encounter table -> Enemy instantiation correctness
2. Enemy AI delegation through CombatEngine
3. Multi-enemy encounter composition
4. Per-floor RNG seeding determinism
5. Headless mode correctness
6. Previously untested enemy AI patterns
7. GameRunner combat integration
"""

import pytest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from packages.engine.state.rng import Random
from packages.engine.content.enemies import (
    Enemy, MoveInfo, Intent, EnemyType, ENEMY_CLASSES, create_enemy,
    JawWorm, Cultist, Louse, LouseNormal, LouseDefensive, FungiBeast,
    AcidSlimeM, SpikeSlimeM, AcidSlimeL, SpikeSlimeL, AcidSlimeS, SpikeSlimeS,
    Looter, Mugger, SlaverBlue, SlaverRed, Sentries,
    GremlinNob, Lagavulin, SlimeBoss, TheGuardian, Hexaghost,
    Chosen, Byrd, Centurion, Healer, Snecko, SnakePlant,
    ShelledParasite, SphericGuardian,
    GremlinLeader, BookOfStabbing, Taskmaster,
    Champ, TheCollector, BronzeAutomaton,
    Maw, Darkling, OrbWalker, Spiker, Repulsor, WrithingMass, Transient,
    Exploder, SpireGrowth, SnakeDagger,
    GiantHead, Nemesis, Reptomancer,
    AwakenedOne, TimeEater, Donu, Deca,
    SpireShield, SpireSpear, CorruptHeart,
    TorchHead, BronzeOrb,
    GremlinFat, GremlinThief, GremlinTsundere, GremlinWarrior, GremlinWizard,
    BanditBear, BanditLeader, BanditPointy,
)
from packages.engine.handlers.combat import (
    ENCOUNTER_TABLE, create_enemies_from_encounter,
)
from packages.engine.combat_engine import (
    CombatEngine, create_combat_from_enemies, create_simple_combat,
)
from packages.engine.game import GameRunner, GamePhase, run_headless, RunResult


# =============================================================================
# Encounter Table Tests
# =============================================================================

class TestEncounterTable:
    """Verify ENCOUNTER_TABLE maps every encounter name to correct enemies."""

    @pytest.mark.parametrize("name", sorted(ENCOUNTER_TABLE.keys()))
    def test_encounter_creates_enemies(self, name):
        """Every encounter in the table should produce at least one Enemy."""
        ai_rng = Random(12345)
        hp_rng = Random(12345)
        enemies = create_enemies_from_encounter(name, ai_rng, ascension=0, hp_rng=hp_rng)
        assert len(enemies) >= 1, f"{name} produced no enemies"
        for e in enemies:
            assert isinstance(e, Enemy), f"{name} produced non-Enemy: {type(e)}"
            assert e.state.current_hp > 0, f"{name} enemy {e.ID} has 0 HP"
            assert e.state.max_hp > 0

    def test_unknown_encounter_raises(self):
        """Unknown encounter names should raise ValueError."""
        with pytest.raises(ValueError, match="Unknown encounter"):
            create_enemies_from_encounter("Nonexistent Monster", Random(0))

    def test_2_louse_creates_two(self):
        enemies = create_enemies_from_encounter("2 Louse", Random(42), 0, Random(42))
        assert len(enemies) == 2
        for e in enemies:
            assert e.ID == "Louse"

    def test_3_sentries_creates_three_with_positions(self):
        enemies = create_enemies_from_encounter("3 Sentries", Random(42), 0, Random(42))
        assert len(enemies) == 3
        assert all(e.ID == "Sentry" for e in enemies)
        # Left and Right start BOLT, Middle starts BEAM
        assert enemies[0]._starting_move == enemies[0].BOLT
        assert enemies[1]._starting_move == enemies[1].BEAM
        assert enemies[2]._starting_move == enemies[2].BOLT

    def test_gremlin_gang_creates_four(self):
        enemies = create_enemies_from_encounter("Gremlin Gang", Random(42), 0, Random(42))
        assert len(enemies) == 4

    def test_exordium_thugs_composition(self):
        enemies = create_enemies_from_encounter("Exordium Thugs", Random(42), 0, Random(42))
        assert len(enemies) == 2
        ids = {e.ID for e in enemies}
        assert "SlaverBlue" in ids
        assert "SlaverRed" in ids

    def test_centurion_and_healer_composition(self):
        enemies = create_enemies_from_encounter("Centurion and Healer", Random(42), 0, Random(42))
        assert len(enemies) == 2
        ids = [e.ID for e in enemies]
        assert "Centurion" in ids
        assert "Healer" in ids

    def test_donu_and_deca_composition(self):
        enemies = create_enemies_from_encounter("Donu and Deca", Random(42), 0, Random(42))
        assert len(enemies) == 2
        ids = {e.ID for e in enemies}
        assert "Donu" in ids
        assert "Deca" in ids

    def test_slavers_composition(self):
        enemies = create_enemies_from_encounter("Slavers", Random(42), 0, Random(42))
        assert len(enemies) == 3
        ids = {e.ID for e in enemies}
        assert "SlaverBlue" in ids
        assert "SlaverRed" in ids
        assert "SlaverBoss" in ids or "Taskmaster" in ids

    def test_reptomancer_composition(self):
        enemies = create_enemies_from_encounter("Reptomancer", Random(42), 0, Random(42))
        assert len(enemies) == 3
        assert enemies[0].ID == "Reptomancer"
        assert all(e.ID == "Dagger" for e in enemies[1:])

    def test_3_shapes_creates_three(self):
        enemies = create_enemies_from_encounter("3 Shapes", Random(42), 0, Random(42))
        assert len(enemies) == 3
        shape_ids = {"Exploder", "Repulsor", "Spiker"}
        for e in enemies:
            assert e.ID in shape_ids

    def test_4_shapes_creates_four(self):
        enemies = create_enemies_from_encounter("4 Shapes", Random(42), 0, Random(42))
        assert len(enemies) == 4

    def test_jaw_worm_horde_creates_three(self):
        enemies = create_enemies_from_encounter("Jaw Worm Horde", Random(42), 0, Random(42))
        assert len(enemies) == 3
        assert all(e.ID == "JawWorm" for e in enemies)

    def test_spire_shield_and_spear(self):
        enemies = create_enemies_from_encounter("Spire Shield and Spire Spear", Random(42), 0, Random(42))
        assert len(enemies) == 2
        ids = {e.ID for e in enemies}
        assert "SpireShield" in ids
        assert "SpireSpear" in ids

    def test_3_darklings(self):
        enemies = create_enemies_from_encounter("3 Darklings", Random(42), 0, Random(42))
        assert len(enemies) == 3
        assert all(e.ID == "Darkling" for e in enemies)

    def test_3_byrds(self):
        enemies = create_enemies_from_encounter("3 Byrds", Random(42), 0, Random(42))
        assert len(enemies) == 3
        assert all(e.ID == "Byrd" for e in enemies)

    def test_3_cultists(self):
        enemies = create_enemies_from_encounter("3 Cultists", Random(42), 0, Random(42))
        assert len(enemies) == 3
        assert all(e.ID == "Cultist" for e in enemies)

    def test_gremlin_leader_composition(self):
        enemies = create_enemies_from_encounter("Gremlin Leader", Random(42), 0, Random(42))
        assert len(enemies) == 3
        assert enemies[0].ID == "GremlinLeader"
        # Other 2 are random gremlins
        gremlin_ids = {"GremlinFat", "GremlinThief", "GremlinTsundere", "GremlinWarrior", "GremlinWizard"}
        for e in enemies[1:]:
            assert e.ID in gremlin_ids


# =============================================================================
# Enemy AI Delegation Tests
# =============================================================================

class TestEnemyAIDelegation:
    """Verify CombatEngine delegates to real Enemy AI when enemy_objects is set."""

    def _make_engine_with_real_enemy(self, enemy_cls, **kwargs):
        """Create a CombatEngine with a real Enemy object."""
        ai_rng = Random(99)
        hp_rng = Random(99)
        enemy = enemy_cls(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng, **kwargs)
        engine = create_combat_from_enemies(
            enemies=[enemy],
            player_hp=80,
            player_max_hp=80,
            deck=["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"],
        )
        return engine, enemy

    def test_jawworm_delegates(self):
        """JawWorm's first move through delegation should be Chomp."""
        engine, enemy = self._make_engine_with_real_enemy(JawWorm)
        engine.start_combat()
        # After start_combat, the initial move should have been rolled
        e = engine.state.enemies[0]
        # JawWorm first move is always Chomp (move_id 1)
        assert e.move_history[-1] == 1

    def test_cultist_delegates(self):
        """Cultist's first move should be Incantation through delegation."""
        engine, enemy = self._make_engine_with_real_enemy(Cultist)
        engine.start_combat()
        e = engine.state.enemies[0]
        assert e.move_history[-1] == 1  # Incantation

    def test_sentries_position_affects_first_move(self):
        """Sentries at different positions should start with different moves."""
        ai1, ai2, ai3 = Random(10), Random(10), Random(10)
        hp1, hp2, hp3 = Random(10), Random(10), Random(10)
        s_left = Sentries(ai1, 0, hp1, position=0)
        s_mid = Sentries(ai2, 0, hp2, position=1)
        s_right = Sentries(ai3, 0, hp3, position=2)
        enemies = [s_left, s_mid, s_right]
        engine = create_combat_from_enemies(
            enemies=enemies,
            player_hp=80, player_max_hp=80,
            deck=["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"],
        )
        engine.start_combat()
        # Left (pos 0) and Right (pos 2) start with BOLT (1)
        # Middle (pos 1) starts with BEAM (2)
        assert engine.state.enemies[0].move_history[-1] == 1  # BOLT
        assert engine.state.enemies[1].move_history[-1] == 2  # BEAM
        assert engine.state.enemies[2].move_history[-1] == 1  # BOLT

    def test_legacy_fallback_still_works(self):
        """create_simple_combat should use inline AI (no enemy_objects)."""
        engine = create_simple_combat("JawWorm", 42, 11, 80)
        assert engine.enemy_objects == []
        engine.start_combat()
        e = engine.state.enemies[0]
        # Inline JawWorm AI: first move is Chomp (move_id 1)
        assert e.move_history[-1] == 1

    def test_multi_enemy_delegation(self):
        """Multiple enemies should each get their own AI delegation."""
        ai = Random(42)
        hp = Random(42)
        enemies = [
            JawWorm(ai_rng=Random(42), ascension=0, hp_rng=Random(42)),
            Cultist(ai_rng=Random(43), ascension=0, hp_rng=Random(43)),
        ]
        engine = create_combat_from_enemies(
            enemies=enemies,
            player_hp=80, player_max_hp=80,
            deck=["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"],
        )
        engine.start_combat()
        # JawWorm: Chomp (1), Cultist: Incantation (1)
        assert engine.state.enemies[0].move_history[-1] == 1
        assert engine.state.enemies[1].move_history[-1] == 1


# =============================================================================
# Per-Floor RNG Determinism
# =============================================================================

class TestPerFloorRNG:
    """Verify per-floor RNG produces deterministic, distinct results."""

    def test_same_seed_same_floor_same_enemies(self):
        """Same seed + floor should always create identical enemies."""
        for floor in [1, 5, 10, 25, 50]:
            seed = 12345
            enemies1 = create_enemies_from_encounter(
                "Jaw Worm", Random(seed + floor), 0, Random(seed + floor))
            enemies2 = create_enemies_from_encounter(
                "Jaw Worm", Random(seed + floor), 0, Random(seed + floor))
            assert enemies1[0].state.current_hp == enemies2[0].state.current_hp
            assert enemies1[0].state.max_hp == enemies2[0].state.max_hp

    def test_different_floors_different_hp(self):
        """Different floors should generally produce different HP rolls."""
        seed = 12345
        hps = set()
        for floor in range(1, 20):
            enemies = create_enemies_from_encounter(
                "Jaw Worm", Random(seed + floor), 0, Random(seed + floor))
            hps.add(enemies[0].state.current_hp)
        # With 19 floors and HP range 40-44, we should get some variation
        assert len(hps) > 1

    def test_encounter_rng_isolation(self):
        """Creating one encounter should not affect another's RNG state."""
        ai1 = Random(100)
        hp1 = Random(100)
        ai2 = Random(100)
        hp2 = Random(100)

        # Create one encounter
        e1 = create_enemies_from_encounter("Jaw Worm", ai1, 0, hp1)
        # Create same encounter with fresh RNG
        e2 = create_enemies_from_encounter("Jaw Worm", ai2, 0, hp2)

        assert e1[0].state.current_hp == e2[0].state.current_hp


# =============================================================================
# Previously Untested Enemy AI Patterns
# =============================================================================

class TestUntestedEnemyAI:
    """Test AI patterns for enemies that had zero AI tests."""

    def _roll_moves(self, enemy, n=5):
        """Roll n moves for an enemy, return list of move_ids."""
        moves = []
        for _ in range(n):
            move = enemy.roll_move()
            moves.append(move.move_id)
        return moves

    def test_slaver_blue_has_moves(self):
        e = SlaverBlue(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5
        assert all(isinstance(m, int) for m in moves)

    def test_orb_walker_has_moves(self):
        e = OrbWalker(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_repulsor_has_moves(self):
        e = Repulsor(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_spire_growth_has_moves(self):
        e = SpireGrowth(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_snake_plant_has_moves(self):
        e = SnakePlant(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_bandit_bear_has_moves(self):
        e = BanditBear(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_bandit_leader_has_moves(self):
        e = BanditLeader(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_taskmaster_has_moves(self):
        e = Taskmaster(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 3)
        assert len(moves) == 3

    def test_bronze_orb_has_moves(self):
        e = BronzeOrb(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_looter_returns_move(self):
        """Looter should return a valid move (even if stub)."""
        e = Looter(Random(42), 0, Random(42))
        move = e.roll_move()
        assert isinstance(move, MoveInfo)
        assert move.move_id >= 0

    def test_mugger_returns_move(self):
        """Mugger should return a valid move (even if stub)."""
        e = Mugger(Random(42), 0, Random(42))
        move = e.roll_move()
        assert isinstance(move, MoveInfo)
        assert move.move_id >= 0

    def test_louse_normal_standalone(self):
        """LouseNormal (standalone red louse class) should have valid AI."""
        e = LouseNormal(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_louse_defensive_standalone(self):
        """LouseDefensive (standalone green louse class) should have valid AI."""
        e = LouseDefensive(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_spike_slime_m_has_moves(self):
        e = SpikeSlimeM(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5

    def test_acid_slime_l_has_moves(self):
        e = AcidSlimeL(Random(42), 0, Random(42))
        moves = self._roll_moves(e, 5)
        assert len(moves) == 5


# =============================================================================
# Ascension HP Scaling for Encounters
# =============================================================================

class TestEncounterAscensionScaling:
    """Verify ascension affects enemy HP when created through encounter table."""

    @pytest.mark.parametrize("encounter,expected_higher_asc", [
        ("Jaw Worm", True),
        ("Cultist", True),
        ("Gremlin Nob", True),
        ("Lagavulin", True),
    ])
    def test_ascension_increases_hp(self, encounter, expected_higher_asc):
        """Higher ascension should generally produce higher HP."""
        seed = 99999
        e_a0 = create_enemies_from_encounter(encounter, Random(seed), 0, Random(seed))
        e_a20 = create_enemies_from_encounter(encounter, Random(seed), 20, Random(seed))
        if expected_higher_asc:
            assert e_a20[0].state.max_hp >= e_a0[0].state.max_hp


# =============================================================================
# Headless Mode Tests
# =============================================================================

class TestHeadlessMode:
    """Test run_headless() correctness."""

    def test_returns_run_result(self):
        result = run_headless(seed=42, ascension=0, max_actions=100)
        assert isinstance(result, RunResult)
        assert result.seed == 42
        assert result.ascension == 0

    def test_floor_advanced(self):
        result = run_headless(seed=42, ascension=0, max_actions=500)
        assert result.floor_reached >= 1

    def test_max_actions_limits_run(self):
        result = run_headless(seed=42, ascension=0, max_actions=5)
        # With only 5 actions, shouldn't get far
        assert result.floor_reached <= 5

    def test_deterministic(self):
        r1 = run_headless(seed=777, ascension=20, max_actions=200)
        r2 = run_headless(seed=777, ascension=20, max_actions=200)
        assert r1.floor_reached == r2.floor_reached
        assert r1.hp_remaining == r2.hp_remaining
        assert r1.combats_won == r2.combats_won
        assert r1.deck_size == r2.deck_size
        assert r1.victory == r2.victory

    def test_different_seeds_diverge(self):
        r1 = run_headless(seed=100, ascension=0, max_actions=300)
        r2 = run_headless(seed=200, ascension=0, max_actions=300)
        # With 300 actions, different seeds should produce different results
        # (not guaranteed but extremely likely)
        different = (r1.floor_reached != r2.floor_reached or
                     r1.hp_remaining != r2.hp_remaining or
                     r1.deck_size != r2.deck_size)
        assert different, "Different seeds produced identical results"

    def test_custom_decision_fn(self):
        """Custom decision function should be called."""
        call_count = [0]
        def count_fn(state, actions):
            call_count[0] += 1
            return actions[0]
        run_headless(seed=42, ascension=0, decision_fn=count_fn, max_actions=50)
        assert call_count[0] > 0

    def test_no_shared_state_between_runs(self):
        """Two sequential runs should not affect each other."""
        r1a = run_headless(seed=42, ascension=0, max_actions=100)
        run_headless(seed=999, ascension=20, max_actions=100)  # different run
        r1b = run_headless(seed=42, ascension=0, max_actions=100)
        assert r1a.floor_reached == r1b.floor_reached
        assert r1a.hp_remaining == r1b.hp_remaining


# =============================================================================
# GameRunner Combat Integration
# =============================================================================

class TestGameRunnerCombat:
    """Test that GameRunner properly enters and exits combat with real enemies."""

    def test_enters_combat_phase(self):
        runner = GameRunner(seed=42, ascension=0, verbose=False)
        # Navigate to first combat
        max_actions = 100
        entered_combat = False
        for _ in range(max_actions):
            if runner.game_over:
                break
            if runner.phase == GamePhase.COMBAT:
                entered_combat = True
                break
            actions = runner.get_available_actions()
            if actions:
                runner.take_action(actions[0])
        assert entered_combat, "Never entered combat phase"

    def test_combat_has_real_enemies(self):
        runner = GameRunner(seed=42, ascension=0, verbose=False)
        for _ in range(100):
            if runner.game_over:
                break
            if runner.phase == GamePhase.COMBAT:
                break
            actions = runner.get_available_actions()
            if actions:
                runner.take_action(actions[0])

        if runner.phase == GamePhase.COMBAT:
            assert runner.current_combat is not None
            assert len(runner.current_combat.state.enemies) >= 1
            # With real enemies, enemy_objects should be populated
            assert len(runner.current_combat.enemy_objects) >= 1

    def test_combat_can_complete(self):
        """Should be able to complete at least one combat."""
        runner = GameRunner(seed=42, ascension=0, verbose=False)
        for _ in range(500):
            if runner.game_over:
                break
            actions = runner.get_available_actions()
            if not actions:
                break
            runner.take_action(actions[0])
        assert runner.run_state.combats_won >= 1 or runner.game_over


# =============================================================================
# Enemy HP Rolling Consistency
# =============================================================================

class TestEnemyHPRolling:
    """Verify enemy HP is within expected ranges."""

    @pytest.mark.parametrize("cls,a0_range,a7_range", [
        (JawWorm, (40, 44), (42, 46)),
        (Cultist, (48, 54), (50, 56)),
        (FungiBeast, (22, 28), (24, 30)),
    ])
    def test_hp_ranges(self, cls, a0_range, a7_range):
        """Enemy HP should be within documented ranges."""
        for _ in range(20):
            e = cls(Random(_), 0, Random(_))
            assert a0_range[0] <= e.state.current_hp <= a0_range[1], \
                f"{cls.ID} A0 HP {e.state.current_hp} not in {a0_range}"
        for _ in range(20):
            e = cls(Random(_ + 100), 7, Random(_ + 100))
            assert a7_range[0] <= e.state.current_hp <= a7_range[1], \
                f"{cls.ID} A7 HP {e.state.current_hp} not in {a7_range}"

    def test_louse_bite_damage_rolled(self):
        """Louse bite damage should be rolled from HP RNG."""
        damages = set()
        for i in range(50):
            e = Louse(Random(i), 0, Random(i))
            damages.add(e.bite_damage)
        # Should see variation in 5-7 range
        assert len(damages) >= 2
        assert min(damages) >= 5
        assert max(damages) <= 7

    def test_louse_bite_damage_a2(self):
        """A2+ Louse bite damage should be 6-8."""
        damages = set()
        for i in range(50):
            e = Louse(Random(i), 2, Random(i))
            damages.add(e.bite_damage)
        assert min(damages) >= 6
        assert max(damages) <= 8

    def test_sentries_hp_a0(self):
        for i in range(20):
            e = Sentries(Random(i), 0, Random(i), position=0)
            assert 38 <= e.state.current_hp <= 42

    def test_sentries_hp_a8(self):
        for i in range(20):
            e = Sentries(Random(i), 8, Random(i), position=0)
            assert 39 <= e.state.current_hp <= 45


# =============================================================================
# All Enemies Instantiate Without Error
# =============================================================================

class TestAllEnemiesInstantiate:
    """Every enemy class in ENEMY_CLASSES should instantiate cleanly."""

    @pytest.mark.parametrize("enemy_id", sorted(ENEMY_CLASSES.keys()))
    def test_instantiate(self, enemy_id):
        """Every registered enemy should instantiate without error."""
        cls = ENEMY_CLASSES[enemy_id]
        ai = Random(42)
        hp = Random(42)
        try:
            e = cls(ai_rng=ai, ascension=0, hp_rng=hp)
        except TypeError:
            # Some enemies need extra kwargs; try with common ones
            try:
                e = cls(ai_rng=ai, ascension=0, hp_rng=hp, position=0)
            except TypeError:
                e = cls(ai_rng=ai, ascension=0, hp_rng=hp, is_red=True)
        assert e.state.current_hp > 0
        assert e.ID == enemy_id or e.ID in ENEMY_CLASSES

    @pytest.mark.parametrize("enemy_id", sorted(ENEMY_CLASSES.keys()))
    def test_roll_move(self, enemy_id):
        """Every registered enemy should be able to roll a move."""
        cls = ENEMY_CLASSES[enemy_id]
        ai = Random(42)
        hp = Random(42)
        try:
            e = cls(ai_rng=ai, ascension=0, hp_rng=hp)
        except TypeError:
            try:
                e = cls(ai_rng=ai, ascension=0, hp_rng=hp, position=0)
            except TypeError:
                e = cls(ai_rng=ai, ascension=0, hp_rng=hp, is_red=True)
        move = e.roll_move()
        assert isinstance(move, MoveInfo)
        assert move.move_id >= 0 or move.move_id == -1  # -1 is valid for some (e.g., split)
        assert isinstance(move.intent, Intent)


# =============================================================================
# Java Parity: RNG Call Order
# =============================================================================

class TestRNGCallOrder:
    """Verify RNG consumption order matches Java decompiled source."""

    def test_jawworm_first_move_consumes_one_airng(self):
        """JawWorm.rollMove() should consume exactly 1 aiRng call on first turn."""
        ai = Random(42)
        e = JawWorm(ai_rng=ai, ascension=0, hp_rng=Random(42))
        initial_counter = ai.counter
        e.roll_move()
        # roll_move calls ai_rng.random(99) once
        assert ai.counter == initial_counter + 1

    def test_cultist_first_move_consumes_one_airng(self):
        """Cultist first move is deterministic but still consumes aiRng."""
        ai = Random(42)
        e = Cultist(ai_rng=ai, ascension=0, hp_rng=Random(42))
        initial_counter = ai.counter
        e.roll_move()
        assert ai.counter == initial_counter + 1

    def test_louse_hp_rng_calls(self):
        """Louse constructor should consume hp_rng for: HP, bite_damage, curl_up."""
        hp = Random(42)
        initial = hp.counter
        e = Louse(ai_rng=Random(42), ascension=0, hp_rng=hp)
        # HP roll (1) + bite damage (1) + curl up (1) = 3
        assert hp.counter == initial + 3
