"""
Comprehensive tests for enemy AI patterns in Slay the Spire.

Tests cover:
1. First move guarantees (JawWorm always Chomp, Cultist always Incantation, etc.)
2. Anti-repeat patterns (lastMove, lastMoveBefore, lastTwoMoves tracking)
3. Ascension scaling for damage values (A2+, A17+, etc.)
4. Ascension scaling for HP ranges (A7+, A8+, A9+, etc.)
5. Phase transitions (Champ at 50%, Slime Boss split, Awakened One phase 2)
6. Conditional moves (Lagavulin wake on damage, Bronze Automaton hyper beam)
7. Turn counters and cycle patterns
8. Elite-specific patterns at A18+ (Gremlin Nob, Lagavulin, Book of Stabbing)
9. Boss-specific mechanics (Time Eater card counter, Heart beat, Shapes invincibility)
10. Multi-enemy encounters (Gremlin Gang, Slavers, Orb Walkers)

Uses pytest with deterministic RNG seeds for reproducibility.
"""

import pytest
from packages.engine.content.enemies import (
    # Base classes
    Enemy, EnemyState, MoveInfo, Intent, EnemyType,

    # Exordium enemies
    JawWorm, Cultist, AcidSlimeM, AcidSlimeL, AcidSlimeS,
    SpikeSlimeM, SpikeSlimeL, SpikeSlimeS, Louse, FungiBeast,
    LouseNormal, LouseDefensive, Looter, SlaverBlue, SlaverRed,

    # Exordium elites
    GremlinNob, Lagavulin, Sentries,

    # Exordium bosses
    SlimeBoss, TheGuardian, Hexaghost,

    # City enemies
    Chosen, Byrd, Centurion, Healer, Snecko, SnakePlant,
    Mugger, Taskmaster, ShelledParasite, SphericGuardian,
    BanditBear, BanditLeader, BanditPointy,

    # City elites
    GremlinLeader, BookOfStabbing,

    # City bosses
    Champ, TheCollector, BronzeAutomaton,

    # Beyond enemies
    Maw, Darkling, OrbWalker, Spiker, Repulsor,
    WrithingMass, Transient, Exploder, SpireGrowth, SnakeDagger,

    # Beyond elites
    GiantHead, Nemesis, Reptomancer,

    # Beyond bosses
    AwakenedOne, TimeEater, Donu, Deca,

    # Act 4 enemies
    SpireShield, SpireSpear, CorruptHeart,

    # Minions
    TorchHead, BronzeOrb, GremlinFat, GremlinThief,
    GremlinTsundere, GremlinWarrior, GremlinWizard,
)
from packages.engine.state.rng import Random


# ============================================================================
# UTILITY FIXTURES
# ============================================================================

@pytest.fixture
def rng_seed_1():
    """Fresh RNG with seed 1."""
    return Random(1)


@pytest.fixture
def rng_seed_42():
    """Fresh RNG with seed 42."""
    return Random(42)


@pytest.fixture
def rng_seed_12345():
    """Fresh RNG with seed 12345."""
    return Random(12345)


def create_enemy(enemy_class, seed=1, ascension=0, **kwargs):
    """Helper to create an enemy with a specific seed."""
    ai_rng = Random(seed)
    hp_rng = Random(seed + 1000)  # Different seed for HP
    return enemy_class(ai_rng=ai_rng, ascension=ascension, hp_rng=hp_rng, **kwargs)


# ============================================================================
# 1. FIRST MOVE GUARANTEES
# ============================================================================

class TestFirstMoveGuarantees:
    """Test that certain enemies always use specific moves on their first turn."""

    def test_jaw_worm_first_move_always_chomp(self):
        """Jaw Worm always uses Chomp on first turn."""
        for seed in [1, 42, 100, 999, 12345]:
            enemy = create_enemy(JawWorm, seed=seed)
            move = enemy.roll_move()
            assert move.move_id == JawWorm.CHOMP, f"Seed {seed}: Expected Chomp, got {move.name}"
            assert move.name == "Chomp"

    def test_jaw_worm_first_move_all_ascensions(self):
        """Jaw Worm always Chomps first regardless of ascension."""
        for asc in [0, 2, 7, 17, 20]:
            enemy = create_enemy(JawWorm, ascension=asc)
            move = enemy.roll_move()
            assert move.move_id == JawWorm.CHOMP

    def test_cultist_first_move_always_incantation(self):
        """Cultist always uses Incantation on first turn."""
        for seed in [1, 42, 100, 999, 12345]:
            enemy = create_enemy(Cultist, seed=seed)
            move = enemy.roll_move()
            assert move.move_id == Cultist.INCANTATION
            assert move.name == "Incantation"
            assert move.intent == Intent.BUFF

    def test_cultist_always_dark_strike_after_first(self):
        """Cultist always uses Dark Strike after first turn."""
        enemy = create_enemy(Cultist, seed=42)
        # First turn: Incantation
        move1 = enemy.roll_move()
        assert move1.move_id == Cultist.INCANTATION

        # All subsequent turns: Dark Strike
        for _ in range(5):
            move = enemy.roll_move()
            assert move.move_id == Cultist.DARK_STRIKE

    def test_gremlin_nob_first_move_always_bellow(self):
        """Gremlin Nob always uses Bellow on first turn."""
        for seed in [1, 42, 100, 999, 12345]:
            enemy = create_enemy(GremlinNob, seed=seed)
            move = enemy.roll_move()
            assert move.move_id == GremlinNob.BELLOW
            assert move.intent == Intent.BUFF

    def test_slime_boss_first_move_always_sticky(self):
        """Slime Boss always uses Goop Spray on first turn."""
        for seed in [1, 42, 100, 999]:
            enemy = create_enemy(SlimeBoss, seed=seed)
            move = enemy.roll_move()
            assert move.move_id == SlimeBoss.STICKY
            assert move.name == "Goop Spray"

    def test_hexaghost_first_move_always_activate(self):
        """Hexaghost always uses Activate on turn 1."""
        enemy = create_enemy(Hexaghost, seed=42)
        move = enemy.roll_move()
        assert move.move_id == Hexaghost.ACTIVATE

    def test_hexaghost_second_move_always_divider(self):
        """Hexaghost always uses Divider on turn 2."""
        enemy = create_enemy(Hexaghost, seed=42)
        enemy.roll_move()  # Turn 1: Activate
        move = enemy.roll_move()  # Turn 2: Divider
        assert move.move_id == Hexaghost.DIVIDER

    def test_snecko_first_move_always_glare(self):
        """Snecko always uses Glare on first turn."""
        for seed in [1, 42, 100]:
            enemy = create_enemy(Snecko, seed=seed)
            move = enemy.roll_move()
            assert move.move_id == Snecko.GLARE
            assert "confused" in move.effects

    def test_maw_first_move_always_roar(self):
        """The Maw always uses Roar on first turn."""
        enemy = create_enemy(Maw, seed=42)
        move = enemy.roll_move()
        assert move.move_id == Maw.ROAR

    def test_chosen_first_move_a17_always_hex(self):
        """Chosen at A17+ always uses Hex first."""
        enemy = create_enemy(Chosen, seed=42, ascension=17)
        move = enemy.roll_move()
        assert move.move_id == Chosen.HEX

    def test_chosen_first_move_below_a17_always_poke(self):
        """Chosen below A17 always uses Poke first."""
        enemy = create_enemy(Chosen, seed=42, ascension=0)
        move = enemy.roll_move()
        assert move.move_id == Chosen.POKE

    def test_awakened_one_first_move_always_slash(self):
        """Awakened One always uses Slash on first turn of phase 1."""
        for seed in [1, 42, 100]:
            enemy = create_enemy(AwakenedOne, seed=seed)
            move = enemy.roll_move()
            assert move.move_id == AwakenedOne.SLASH

    def test_red_slaver_first_move_always_stab(self):
        """Red Slaver always uses Stab on first turn."""
        for seed in [1, 42, 100]:
            enemy = create_enemy(SlaverRed, seed=seed)
            move = enemy.roll_move()
            assert move.move_id == SlaverRed.STAB

    def test_spheric_guardian_first_move_always_activate(self):
        """Spheric Guardian always uses Activate on first turn."""
        enemy = create_enemy(SphericGuardian, seed=42)
        move = enemy.roll_move()
        assert move.move_id == SphericGuardian.INITIAL_BLOCK

    def test_corrupt_heart_first_move_always_debilitate(self):
        """Corrupt Heart always uses Debilitate on first turn."""
        enemy = create_enemy(CorruptHeart, seed=42)
        move = enemy.roll_move()
        assert move.move_id == CorruptHeart.DEBILITATE


# ============================================================================
# 2. ANTI-REPEAT PATTERNS
# ============================================================================

class TestAntiRepeatPatterns:
    """Test lastMove, lastTwoMoves, and lastMoveBefore tracking."""

    def test_jaw_worm_no_chomp_twice(self):
        """Jaw Worm cannot use Chomp twice in a row (except first turn)."""
        enemy = create_enemy(JawWorm, seed=42)
        # First turn: Chomp (guaranteed)
        enemy.roll_move()

        # Force a roll that would normally pick Chomp (roll < 25)
        move = enemy.get_move(10)  # Would pick Chomp, but can't repeat
        # Should use secondary RNG to pick between Bellow and Thrash
        assert move.move_id in [JawWorm.BELLOW, JawWorm.THRASH]

    def test_jaw_worm_no_thrash_three_times(self):
        """Jaw Worm cannot use Thrash three times in a row."""
        enemy = create_enemy(JawWorm, seed=42)
        enemy.roll_move()  # First turn: Chomp

        # Manually set history to have Thrash twice
        enemy.state.move_history = [JawWorm.THRASH, JawWorm.THRASH]
        enemy.state.first_turn = False

        # Roll that would normally pick Thrash (25-54 range)
        move = enemy.get_move(35)
        # Should pick Chomp or Bellow instead
        assert move.move_id in [JawWorm.CHOMP, JawWorm.BELLOW]

    def test_jaw_worm_no_bellow_twice(self):
        """Jaw Worm cannot use Bellow twice in a row."""
        enemy = create_enemy(JawWorm, seed=42)
        enemy.roll_move()  # First: Chomp

        # Set last move as Bellow
        enemy.state.move_history[-1] = JawWorm.BELLOW

        # Roll that would normally pick Bellow (55-99 range)
        move = enemy.get_move(75)
        # Should pick Chomp or Thrash instead
        assert move.move_id in [JawWorm.CHOMP, JawWorm.THRASH]

    def test_fungi_beast_no_bite_three_times(self):
        """Fungi Beast cannot use Bite three times in a row."""
        enemy = create_enemy(FungiBeast, seed=42)
        enemy.state.move_history = [FungiBeast.BITE, FungiBeast.BITE]

        # Roll that would normally pick Bite (0-59 range)
        move = enemy.get_move(30)
        assert move.move_id == FungiBeast.GROW

    def test_fungi_beast_no_grow_twice(self):
        """Fungi Beast cannot use Grow twice in a row."""
        enemy = create_enemy(FungiBeast, seed=42)
        enemy.state.move_history = [FungiBeast.GROW]

        # Roll that would normally pick Grow (60-99 range)
        move = enemy.get_move(75)
        assert move.move_id == FungiBeast.BITE

    def test_acid_slime_m_a17_lick_no_repeat(self):
        """Acid Slime (M) at A17+ cannot repeat Lick."""
        enemy = create_enemy(AcidSlimeM, seed=42, ascension=17)
        enemy.state.move_history = [AcidSlimeM.LICK]

        # Roll in Lick range (80-99)
        move = enemy.get_move(90)
        assert move.move_id != AcidSlimeM.LICK

    def test_acid_slime_m_below_a17_lick_can_repeat_once(self):
        """Acid Slime (M) below A17 can use Lick twice but not three times."""
        enemy = create_enemy(AcidSlimeM, seed=42, ascension=0)

        # One Lick in history - can still Lick
        enemy.state.move_history = [AcidSlimeM.LICK]
        move = enemy.get_move(85)
        assert move.move_id == AcidSlimeM.LICK

        # Two Licks in history - cannot Lick again
        enemy.state.move_history = [AcidSlimeM.LICK, AcidSlimeM.LICK]
        move = enemy.get_move(85)
        assert move.move_id != AcidSlimeM.LICK

    def test_spike_slime_l_a17_lick_no_repeat(self):
        """Spike Slime (L) at A17+ cannot repeat Lick."""
        enemy = create_enemy(SpikeSlimeL, seed=42, ascension=17)
        enemy.state.move_history = [SpikeSlimeL.LICK]

        # Roll in Lick range (30-99)
        move = enemy.get_move(50)
        assert move.move_id == SpikeSlimeL.FLAME_TACKLE

    def test_gremlin_nob_skull_bash_unconditional_below_a18(self):
        """Java: Below A18, roll < 33 unconditionally uses Skull Bash (no lastMove check)."""
        enemy = create_enemy(GremlinNob, seed=42)
        enemy.roll_move()  # Bellow first
        enemy.state.move_history[-1] = GremlinNob.SKULL_BASH

        # Roll that would pick Skull Bash (0-32) - Java does NOT check lastMove
        move = enemy.get_move(20)
        assert move.move_id == GremlinNob.SKULL_BASH

    def test_gremlin_nob_no_rush_three_times(self):
        """Gremlin Nob cannot use Rush three times in a row."""
        enemy = create_enemy(GremlinNob, seed=42)
        enemy.roll_move()  # Bellow first
        enemy.state.move_history = [GremlinNob.RUSH, GremlinNob.RUSH]

        # Roll that would pick Rush (33-99)
        move = enemy.get_move(50)
        assert move.move_id == GremlinNob.SKULL_BASH

    def test_book_of_stabbing_no_multi_stab_three_times(self):
        """Book of Stabbing cannot Multi-Stab three times in a row."""
        enemy = create_enemy(BookOfStabbing, seed=42)
        enemy.state.move_history = [BookOfStabbing.MULTI_STAB, BookOfStabbing.MULTI_STAB]

        # Roll that would normally pick Multi-Stab (15-99)
        move = enemy.get_move(50)
        assert move.move_id == BookOfStabbing.SINGLE_STAB

    def test_time_eater_no_reverberate_three_times(self):
        """Time Eater cannot Reverberate three times in a row."""
        enemy = create_enemy(TimeEater, seed=42)
        enemy.state.move_history = [TimeEater.REVERBERATE, TimeEater.REVERBERATE]

        # Roll that would pick Reverberate (0-44)
        move = enemy.get_move(20)
        # Should recurse to 50-99 range
        assert move.move_id in [TimeEater.HEAD_SLAM, TimeEater.RIPPLE]

    def test_awakened_one_no_slash_three_times_phase1(self):
        """Awakened One cannot Slash three times in a row in phase 1."""
        enemy = create_enemy(AwakenedOne, seed=42)
        enemy.roll_move()  # First: Slash
        enemy.state.move_history = [AwakenedOne.SLASH, AwakenedOne.SLASH]

        # Roll that would pick Slash (25-99)
        move = enemy.get_move(50)
        assert move.move_id == AwakenedOne.SOUL_STRIKE


# ============================================================================
# 3. ASCENSION DAMAGE SCALING
# ============================================================================

class TestAscensionDamageScaling:
    """Test that damage values scale correctly with ascension."""

    def test_jaw_worm_chomp_damage_a0(self):
        """Jaw Worm Chomp deals 11 damage at A0."""
        enemy = create_enemy(JawWorm, ascension=0)
        move = enemy.roll_move()
        assert move.base_damage == 11

    def test_jaw_worm_chomp_damage_a2(self):
        """Jaw Worm Chomp deals 12 damage at A2+."""
        enemy = create_enemy(JawWorm, ascension=2)
        move = enemy.roll_move()
        assert move.base_damage == 12

    def test_jaw_worm_bellow_strength_scaling(self):
        """Jaw Worm Bellow gives 3/4/5 Strength at A0/A2/A17."""
        for asc, expected_str in [(0, 3), (2, 4), (17, 5)]:
            enemy = create_enemy(JawWorm, ascension=asc)
            enemy.roll_move()  # Chomp first
            enemy.state.move_history[-1] = JawWorm.BELLOW
            dmg = enemy._get_damage_values()
            assert dmg["bellow_str"] == expected_str, f"A{asc}: Expected {expected_str}, got {dmg['bellow_str']}"

    def test_cultist_ritual_scaling(self):
        """Cultist Ritual gives 3/4/5 at A0/A2/A17."""
        for asc, expected in [(0, 3), (2, 4), (17, 5)]:
            enemy = create_enemy(Cultist, ascension=asc)
            dmg = enemy._get_damage_values()
            assert dmg["ritual"] == expected

    def test_acid_slime_m_damage_a2(self):
        """Acid Slime (M) deals 8/12 damage at A2+ (vs 7/10)."""
        enemy_a0 = create_enemy(AcidSlimeM, ascension=0)
        enemy_a2 = create_enemy(AcidSlimeM, ascension=2)

        dmg_a0 = enemy_a0._get_damage_values()
        dmg_a2 = enemy_a2._get_damage_values()

        assert dmg_a0["spit"] == 7
        assert dmg_a2["spit"] == 8
        assert dmg_a0["tackle"] == 10
        assert dmg_a2["tackle"] == 12

    def test_gremlin_nob_damage_a3(self):
        """Gremlin Nob deals 16/8 damage at A3+ (vs 14/6)."""
        enemy_a0 = create_enemy(GremlinNob, ascension=0)
        enemy_a3 = create_enemy(GremlinNob, ascension=3)

        dmg_a0 = enemy_a0._get_damage_values()
        dmg_a3 = enemy_a3._get_damage_values()

        assert dmg_a0["rush"] == 14
        assert dmg_a3["rush"] == 16
        assert dmg_a0["skull_bash"] == 6
        assert dmg_a3["skull_bash"] == 8

    def test_gremlin_nob_enrage_a18(self):
        """Gremlin Nob Enrage gives 3 Strength at A18+ (vs 2)."""
        enemy_a0 = create_enemy(GremlinNob, ascension=0)
        enemy_a18 = create_enemy(GremlinNob, ascension=18)

        assert enemy_a0._get_damage_values()["enrage"] == 2
        assert enemy_a18._get_damage_values()["enrage"] == 3

    def test_lagavulin_damage_a3(self):
        """Lagavulin deals 20 damage at A3+ (vs 18)."""
        enemy_a0 = create_enemy(Lagavulin, ascension=0)
        enemy_a3 = create_enemy(Lagavulin, ascension=3)

        assert enemy_a0._get_damage_values()["attack"] == 18
        assert enemy_a3._get_damage_values()["attack"] == 20

    def test_lagavulin_debuff_a18(self):
        """Lagavulin Siphon Soul removes 2 Str/Dex at A18+ (vs 1)."""
        enemy_a0 = create_enemy(Lagavulin, ascension=0)
        enemy_a18 = create_enemy(Lagavulin, ascension=18)

        assert enemy_a0._get_damage_values()["debuff"] == 1
        assert enemy_a18._get_damage_values()["debuff"] == 2

    def test_slime_boss_damage_a4(self):
        """Slime Boss Slam deals 38 damage at A4+ (vs 35)."""
        enemy_a0 = create_enemy(SlimeBoss, ascension=0)
        enemy_a4 = create_enemy(SlimeBoss, ascension=4)

        assert enemy_a0._get_damage_values()["slam"] == 35
        assert enemy_a4._get_damage_values()["slam"] == 38

    def test_slime_boss_slimed_a19(self):
        """Slime Boss applies 5 Slimed at A19+ (vs 3)."""
        enemy_a0 = create_enemy(SlimeBoss, ascension=0)
        enemy_a19 = create_enemy(SlimeBoss, ascension=19)

        move_a0 = enemy_a0.roll_move()
        move_a19 = enemy_a19.roll_move()

        assert move_a0.effects["slimed"] == 3
        assert move_a19.effects["slimed"] == 5

    def test_champ_damage_scaling(self):
        """Champ damage scales at A4 and A9."""
        enemy_a0 = create_enemy(Champ, ascension=0)
        enemy_a4 = create_enemy(Champ, ascension=4)
        enemy_a9 = create_enemy(Champ, ascension=9)

        assert enemy_a0._get_damage_values()["slash"] == 16
        assert enemy_a4._get_damage_values()["slash"] == 18
        assert enemy_a9._get_damage_values()["slash"] == 18  # Same as A4

    def test_time_eater_damage_a4(self):
        """Time Eater deals 32/8 damage at A4+ (vs 26/7)."""
        enemy_a0 = create_enemy(TimeEater, ascension=0)
        enemy_a4 = create_enemy(TimeEater, ascension=4)

        dmg_a0 = enemy_a0._get_damage_values()
        dmg_a4 = enemy_a4._get_damage_values()

        assert dmg_a0["head_slam"] == 26
        assert dmg_a4["head_slam"] == 32
        assert dmg_a0["reverberate"] == 7
        assert dmg_a4["reverberate"] == 8

    def test_corrupt_heart_damage_a4(self):
        """Corrupt Heart deals 45 Echo and 15 Blood Shots at A4+."""
        enemy_a0 = create_enemy(CorruptHeart, ascension=0)
        enemy_a4 = create_enemy(CorruptHeart, ascension=4)

        dmg_a0 = enemy_a0._get_damage_values()
        dmg_a4 = enemy_a4._get_damage_values()

        assert dmg_a0["echo"] == 40
        assert dmg_a4["echo"] == 45
        assert dmg_a0["blood_count"] == 12
        assert dmg_a4["blood_count"] == 15


# ============================================================================
# 4. ASCENSION HP SCALING
# ============================================================================

class TestAscensionHPScaling:
    """Test that HP ranges scale correctly with ascension."""

    def test_jaw_worm_hp_a7(self):
        """Jaw Worm has 42-46 HP at A7+ (vs 40-44)."""
        enemy_a0 = create_enemy(JawWorm, ascension=0)
        enemy_a7 = create_enemy(JawWorm, ascension=7)

        assert enemy_a0._get_hp_range() == (40, 44)
        assert enemy_a7._get_hp_range() == (42, 46)

    def test_cultist_hp_a7(self):
        """Cultist has 50-56 HP at A7+ (vs 48-54)."""
        enemy_a0 = create_enemy(Cultist, ascension=0)
        enemy_a7 = create_enemy(Cultist, ascension=7)

        assert enemy_a0._get_hp_range() == (48, 54)
        assert enemy_a7._get_hp_range() == (50, 56)

    def test_gremlin_nob_hp_a8(self):
        """Gremlin Nob has 85-90 HP at A8+ (vs 82-86)."""
        enemy_a0 = create_enemy(GremlinNob, ascension=0)
        enemy_a8 = create_enemy(GremlinNob, ascension=8)

        assert enemy_a0._get_hp_range() == (82, 86)
        assert enemy_a8._get_hp_range() == (85, 90)

    def test_lagavulin_hp_a8(self):
        """Lagavulin has 112-115 HP at A8+ (vs 109-111)."""
        enemy_a0 = create_enemy(Lagavulin, ascension=0)
        enemy_a8 = create_enemy(Lagavulin, ascension=8)

        assert enemy_a0._get_hp_range() == (109, 111)
        assert enemy_a8._get_hp_range() == (112, 115)

    def test_slime_boss_hp_a9(self):
        """Slime Boss has 150 HP at A9+ (vs 140)."""
        enemy_a0 = create_enemy(SlimeBoss, ascension=0)
        enemy_a9 = create_enemy(SlimeBoss, ascension=9)

        assert enemy_a0._get_hp_range() == (140, 140)
        assert enemy_a9._get_hp_range() == (150, 150)

    def test_hexaghost_hp_a9(self):
        """Hexaghost has 264 HP at A9+ (vs 250)."""
        enemy_a0 = create_enemy(Hexaghost, ascension=0)
        enemy_a9 = create_enemy(Hexaghost, ascension=9)

        assert enemy_a0._get_hp_range() == (250, 250)
        assert enemy_a9._get_hp_range() == (264, 264)

    def test_guardian_hp_a9(self):
        """Guardian has 250 HP at A9+ (vs 240)."""
        enemy_a0 = create_enemy(TheGuardian, ascension=0)
        enemy_a9 = create_enemy(TheGuardian, ascension=9)

        assert enemy_a0._get_hp_range() == (240, 240)
        assert enemy_a9._get_hp_range() == (250, 250)

    def test_champ_hp_a9(self):
        """Champ has 440 HP at A9+ (vs 420)."""
        enemy_a0 = create_enemy(Champ, ascension=0)
        enemy_a9 = create_enemy(Champ, ascension=9)

        assert enemy_a0._get_hp_range() == (420, 420)
        assert enemy_a9._get_hp_range() == (440, 440)

    def test_collector_hp_a9(self):
        """Collector has 300 HP at A9+ (vs 282)."""
        enemy_a0 = create_enemy(TheCollector, ascension=0)
        enemy_a9 = create_enemy(TheCollector, ascension=9)

        assert enemy_a0._get_hp_range() == (282, 282)
        assert enemy_a9._get_hp_range() == (300, 300)

    def test_automaton_hp_a9(self):
        """Bronze Automaton has 320 HP at A9+ (vs 300)."""
        enemy_a0 = create_enemy(BronzeAutomaton, ascension=0)
        enemy_a9 = create_enemy(BronzeAutomaton, ascension=9)

        assert enemy_a0._get_hp_range() == (300, 300)
        assert enemy_a9._get_hp_range() == (320, 320)

    def test_awakened_one_hp_a9(self):
        """Awakened One has 320 HP at A9+ (vs 300)."""
        enemy_a0 = create_enemy(AwakenedOne, ascension=0)
        enemy_a9 = create_enemy(AwakenedOne, ascension=9)

        assert enemy_a0._get_hp_range() == (300, 300)
        assert enemy_a9._get_hp_range() == (320, 320)

    def test_time_eater_hp_a9(self):
        """Time Eater has 480 HP at A9+ (vs 456)."""
        enemy_a0 = create_enemy(TimeEater, ascension=0)
        enemy_a9 = create_enemy(TimeEater, ascension=9)

        assert enemy_a0._get_hp_range() == (456, 456)
        assert enemy_a9._get_hp_range() == (480, 480)

    def test_donu_deca_hp_a9(self):
        """Donu/Deca have 265 HP at A9+ (vs 250)."""
        donu_a0 = create_enemy(Donu, ascension=0)
        donu_a9 = create_enemy(Donu, ascension=9)
        deca_a0 = create_enemy(Deca, ascension=0)
        deca_a9 = create_enemy(Deca, ascension=9)

        assert donu_a0._get_hp_range() == (250, 250)
        assert donu_a9._get_hp_range() == (265, 265)
        assert deca_a0._get_hp_range() == (250, 250)
        assert deca_a9._get_hp_range() == (265, 265)

    def test_corrupt_heart_hp_a9(self):
        """Corrupt Heart has 800 HP at A9+ (vs 750)."""
        enemy_a0 = create_enemy(CorruptHeart, ascension=0)
        enemy_a9 = create_enemy(CorruptHeart, ascension=9)

        assert enemy_a0._get_hp_range() == (750, 750)
        assert enemy_a9._get_hp_range() == (800, 800)


# ============================================================================
# 5. PHASE TRANSITIONS
# ============================================================================

class TestPhaseTransitions:
    """Test phase transition mechanics for bosses."""

    def test_champ_phase_transition_at_50_percent(self):
        """Champ transitions to phase 2 at <50% HP."""
        enemy = create_enemy(Champ, ascension=0)
        # A0 Champ has 420 HP
        enemy.state.current_hp = 210  # Exactly 50% - no transition
        assert not enemy.check_phase_transition()

        enemy.state.current_hp = 209  # Just below 50%
        assert enemy.check_phase_transition()
        # threshold_reached is set inside get_move, not check_phase_transition
        # The method just checks, it doesn't mutate
        # Trigger the actual transition by rolling a move
        move = enemy.roll_move()
        assert enemy.threshold_reached
        assert move.move_id == Champ.ANGER

    def test_champ_anger_on_transition(self):
        """Champ uses Anger move when transitioning to phase 2."""
        enemy = create_enemy(Champ, ascension=0)
        enemy.state.current_hp = 200  # Below 50%

        move = enemy.roll_move()
        assert move.move_id == Champ.ANGER
        assert "remove_debuffs" in move.effects
        assert move.effects["strength"] == 6  # 2 * 3 at A0

    def test_champ_execute_after_anger(self):
        """Champ uses Execute after Anger in phase 2."""
        enemy = create_enemy(Champ, ascension=0)
        enemy.state.current_hp = 200
        enemy.roll_move()  # Anger

        move = enemy.roll_move()
        assert move.move_id == Champ.EXECUTE

    def test_awakened_one_rebirth_triggers(self):
        """Awakened One should rebirth check triggers correctly."""
        enemy = create_enemy(AwakenedOne, ascension=0)
        enemy.state.current_hp = 0

        assert enemy.should_rebirth()

    def test_awakened_one_phase_2_first_move(self):
        """Awakened One uses Dark Echo first in phase 2."""
        enemy = create_enemy(AwakenedOne, ascension=0)
        enemy.trigger_rebirth()  # Move to phase 2

        move = enemy.roll_move()
        assert move.move_id == AwakenedOne.DARK_ECHO
        assert move.base_damage == 40

    def test_awakened_one_phase_2_pattern(self):
        """Awakened One alternates between Sludge and Tackle in phase 2."""
        enemy = create_enemy(AwakenedOne, seed=42, ascension=0)
        enemy.trigger_rebirth()

        enemy.roll_move()  # Dark Echo first

        # After Dark Echo, should use Sludge or Tackle
        move = enemy.roll_move()
        assert move.move_id in [AwakenedOne.SLUDGE, AwakenedOne.TACKLE]

    def test_slime_boss_split_condition(self):
        """Slime Boss should split at 50% HP."""
        enemy = create_enemy(SlimeBoss, ascension=0)
        # A0 has 140 HP
        enemy.state.current_hp = 70  # Exactly 50%
        assert enemy.should_split()

        enemy.state.current_hp = 71
        assert not enemy.should_split()

    def test_acid_slime_l_split_check(self):
        """Large Acid Slime split check works correctly."""
        enemy = create_enemy(AcidSlimeL, ascension=0)
        enemy.state.current_hp = enemy.state.max_hp  # Full HP

        # Taking damage but above 50%
        assert not enemy.check_split(enemy.state.max_hp - 1)

        # Drop to exactly 50%
        assert enemy.check_split(enemy.state.max_hp // 2)

        # Already triggered - shouldn't trigger again
        assert not enemy.check_split(enemy.state.max_hp // 4)

    def test_guardian_mode_shift(self):
        """Guardian switches to defensive mode after taking threshold damage."""
        enemy = create_enemy(TheGuardian, ascension=0)
        # A0 threshold is 30 damage
        assert enemy.offensive_mode

        enemy.take_damage(25)
        assert enemy.offensive_mode  # Not enough yet

        enemy.take_damage(10)  # Total 35 > 30
        assert not enemy.offensive_mode
        assert "sharp_hide" in enemy.state.powers

    def test_guardian_mode_shift_a9_threshold(self):
        """Guardian A9+ has 35 damage threshold (vs 30)."""
        enemy_a0 = create_enemy(TheGuardian, ascension=0)
        enemy_a9 = create_enemy(TheGuardian, ascension=9)

        assert enemy_a0.mode_shift_damage == 30
        assert enemy_a9.mode_shift_damage == 35

    def test_guardian_mode_shift_a19_threshold(self):
        """Guardian A19+ has 40 damage threshold."""
        enemy = create_enemy(TheGuardian, ascension=19)
        assert enemy.mode_shift_damage == 40


# ============================================================================
# 6. CONDITIONAL MOVES
# ============================================================================

class TestConditionalMoves:
    """Test conditional move selection based on game state."""

    def test_lagavulin_sleep_for_3_turns(self):
        """Lagavulin sleeps for 3 turns before waking."""
        enemy = create_enemy(Lagavulin, seed=42)

        # Turns 1-2: Sleep
        move1 = enemy.roll_move()
        assert move1.move_id == Lagavulin.SLEEP

        move2 = enemy.roll_move()
        assert move2.move_id == Lagavulin.SLEEP

        # Turn 3: Wakes up and attacks
        move3 = enemy.roll_move()
        assert move3.move_id == Lagavulin.ATTACK
        assert not enemy.asleep

    def test_lagavulin_wake_on_damage(self):
        """Lagavulin wakes immediately when attacked while sleeping."""
        enemy = create_enemy(Lagavulin, seed=42)
        assert enemy.asleep

        enemy.wake_up()

        assert not enemy.asleep
        assert enemy.state.next_move.move_id == Lagavulin.STUN

    def test_lagavulin_attack_pattern_after_wake(self):
        """Lagavulin uses Attack, Attack, Siphon cycle after waking."""
        enemy = create_enemy(Lagavulin, seed=42)
        # Wake up naturally
        for _ in range(3):
            enemy.roll_move()

        # Now awake - check pattern
        # First after wake: Attack (from wake turn)
        # Then: Attack, Attack, Siphon
        move1 = enemy.roll_move()
        assert move1.move_id == Lagavulin.ATTACK

        move2 = enemy.roll_move()
        assert move2.move_id == Lagavulin.SIPHON_SOUL

    def test_automaton_hyper_beam_cycle(self):
        """Bronze Automaton uses Hyper Beam every 5th turn."""
        enemy = create_enemy(BronzeAutomaton, seed=42)

        # Turn 1: Spawn Orbs
        move1 = enemy.roll_move()
        assert move1.move_id == BronzeAutomaton.SPAWN_ORBS

        # Simulate 4 more turns
        enemy.num_turns = 4
        move = enemy.roll_move()
        assert move.move_id == BronzeAutomaton.HYPER_BEAM

    def test_automaton_stunned_after_beam_below_a19(self):
        """Bronze Automaton is stunned after Hyper Beam below A19."""
        enemy = create_enemy(BronzeAutomaton, seed=42, ascension=0)
        enemy.roll_move()  # Spawn
        enemy.num_turns = 4
        enemy.roll_move()  # Hyper Beam
        enemy.state.move_history[-1] = BronzeAutomaton.HYPER_BEAM

        move = enemy.roll_move()
        assert move.move_id == BronzeAutomaton.STUNNED

    def test_automaton_no_stun_at_a19(self):
        """Bronze Automaton uses Boost instead of Stun at A19+."""
        enemy = create_enemy(BronzeAutomaton, seed=42, ascension=19)
        enemy.roll_move()  # Spawn
        enemy.num_turns = 4
        enemy.roll_move()  # Hyper Beam
        enemy.state.move_history[-1] = BronzeAutomaton.HYPER_BEAM

        move = enemy.roll_move()
        assert move.move_id == BronzeAutomaton.BOOST

    def test_time_eater_haste_at_half_hp(self):
        """Time Eater uses Haste when HP drops below 50%."""
        enemy = create_enemy(TimeEater, ascension=0)
        # A0 has 456 HP
        enemy.state.current_hp = 227  # Just below 50%

        move = enemy.roll_move()
        assert move.move_id == TimeEater.HASTE
        assert "heal_to_half" in move.effects

    def test_time_eater_haste_only_once(self):
        """Time Eater only uses Haste once per fight."""
        enemy = create_enemy(TimeEater, ascension=0)
        enemy.state.current_hp = 200

        move1 = enemy.roll_move()
        assert move1.move_id == TimeEater.HASTE

        # Try again below 50%
        move2 = enemy.roll_move()
        assert move2.move_id != TimeEater.HASTE

    def test_centurion_fury_when_alone(self):
        """Centurion uses Fury instead of Defend when alone."""
        enemy = create_enemy(Centurion, seed=42)

        # Roll that would normally trigger Defend (65-99)
        move_with_ally = enemy.get_move(80, allies_alive=2)
        move_alone = enemy.get_move(80, allies_alive=1)

        # With ally: should try Defend
        # Alone: should use Fury
        assert move_alone.move_id == Centurion.FURY

    def test_healer_heal_when_needed(self):
        """Healer/Mystic heals when total missing HP exceeds threshold."""
        enemy = create_enemy(Healer, seed=42)

        # A0 threshold is 15
        move_no_heal = enemy.get_move(10, total_missing_hp=10)
        move_heal = enemy.get_move(10, total_missing_hp=20)

        assert move_heal.move_id == Healer.HEAL
        assert move_no_heal.move_id != Healer.HEAL


# ============================================================================
# 7. TURN COUNTERS AND CYCLE PATTERNS
# ============================================================================

class TestTurnCountersAndCycles:
    """Test turn-based patterns and counters."""

    def test_hexaghost_7_turn_cycle(self):
        """Hexaghost follows a 7-turn cycle after turn 2."""
        enemy = create_enemy(Hexaghost, seed=42)

        # Turn 1: Activate, Turn 2: Divider
        enemy.roll_move()
        enemy.roll_move()

        # Turns 3-9 should follow: Sear, Tackle, Sear, Inflame, Tackle, Sear, Inferno
        expected_cycle = [
            Hexaghost.SEAR, Hexaghost.TACKLE, Hexaghost.SEAR,
            Hexaghost.INFLAME, Hexaghost.TACKLE, Hexaghost.SEAR,
            Hexaghost.INFERNO
        ]

        for expected_id in expected_cycle:
            move = enemy.roll_move()
            assert move.move_id == expected_id

    def test_slime_boss_cycle(self):
        """Slime Boss follows Sticky -> Prep -> Slam -> Sticky cycle."""
        enemy = create_enemy(SlimeBoss, seed=42)

        expected = [SlimeBoss.STICKY, SlimeBoss.PREP_SLAM, SlimeBoss.SLAM, SlimeBoss.STICKY]

        for expected_id in expected:
            move = enemy.roll_move()
            assert move.move_id == expected_id

    def test_guardian_offensive_cycle(self):
        """Guardian offensive mode follows Charge -> Bash -> Steam -> Whirlwind."""
        enemy = create_enemy(TheGuardian, seed=42)

        expected = [
            TheGuardian.CHARGING_UP, TheGuardian.FIERCE_BASH,
            TheGuardian.VENT_STEAM, TheGuardian.WHIRLWIND
        ]

        for expected_id in expected:
            move = enemy.roll_move()
            assert move.move_id == expected_id

    def test_guardian_defensive_cycle(self):
        """Guardian defensive mode alternates Roll -> Twin Slam."""
        enemy = create_enemy(TheGuardian, seed=42)
        enemy.switch_to_defensive()

        move1 = enemy.roll_move()
        assert move1.move_id == TheGuardian.ROLL_ATTACK

        move2 = enemy.roll_move()
        assert move2.move_id == TheGuardian.TWIN_SLAM

        move3 = enemy.roll_move()
        assert move3.move_id == TheGuardian.ROLL_ATTACK

    def test_donu_alternating_pattern(self):
        """Donu alternates Circle -> Beam -> Circle -> Beam."""
        enemy = create_enemy(Donu, seed=42)

        move1 = enemy.roll_move()
        assert move1.move_id == Donu.CIRCLE_OF_PROTECTION

        move2 = enemy.roll_move()
        assert move2.move_id == Donu.BEAM

        move3 = enemy.roll_move()
        assert move3.move_id == Donu.CIRCLE_OF_PROTECTION

    def test_deca_alternating_pattern(self):
        """Deca alternates Beam -> Square -> Beam -> Square."""
        enemy = create_enemy(Deca, seed=42)

        move1 = enemy.roll_move()
        assert move1.move_id == Deca.BEAM

        move2 = enemy.roll_move()
        assert move2.move_id == Deca.SQUARE_OF_PROTECTION

        move3 = enemy.roll_move()
        assert move3.move_id == Deca.BEAM

    def test_sentry_alternating_pattern(self):
        """Sentry alternates between Bolt and Beam."""
        enemy = create_enemy(Sentries, seed=42, position=0)  # Left sentry

        move1 = enemy.roll_move()
        assert move1.move_id == Sentries.BOLT  # Left starts with Bolt

        move2 = enemy.roll_move()
        assert move2.move_id == Sentries.BEAM

        move3 = enemy.roll_move()
        assert move3.move_id == Sentries.BOLT

    def test_sentry_middle_starts_beam(self):
        """Middle sentry starts with Beam instead of Bolt."""
        enemy = create_enemy(Sentries, seed=42, position=1)  # Middle sentry

        move = enemy.roll_move()
        assert move.move_id == Sentries.BEAM

    def test_transient_escalating_damage(self):
        """Transient damage increases by 10 each turn."""
        enemy = create_enemy(Transient, seed=42, ascension=0)

        # A0: starts at 30, +10 each turn
        move1 = enemy.roll_move()
        assert move1.base_damage == 30

        move2 = enemy.roll_move()
        assert move2.base_damage == 40

        move3 = enemy.roll_move()
        assert move3.base_damage == 50

    def test_gremlin_wizard_charge_cycle(self):
        """Gremlin Wizard charges for 2 turns then fires Ultimate."""
        enemy = create_enemy(GremlinWizard, seed=42, ascension=0)

        # Start at charge 1, charges to 2
        move1 = enemy.roll_move()
        assert move1.move_id == GremlinWizard.CHARGE

        # Charges to 3 (limit)
        move2 = enemy.roll_move()
        assert move2.move_id == GremlinWizard.CHARGE

        # Fire Ultimate
        move3 = enemy.roll_move()
        assert move3.move_id == GremlinWizard.DOPE_MAGIC

        # Below A17: resets to charging
        move4 = enemy.roll_move()
        assert move4.move_id == GremlinWizard.CHARGE

    def test_gremlin_wizard_keeps_firing_a17(self):
        """Gremlin Wizard at A17+ keeps firing Ultimate after first."""
        enemy = create_enemy(GremlinWizard, seed=42, ascension=17)

        # Charge up
        enemy.roll_move()  # Charge
        enemy.roll_move()  # Charge
        enemy.roll_move()  # Ultimate

        # Should keep firing at A17+
        move = enemy.roll_move()
        assert move.move_id == GremlinWizard.DOPE_MAGIC

    def test_corrupt_heart_3_turn_cycle(self):
        """Corrupt Heart follows a 3-turn cycle after Debilitate."""
        enemy = create_enemy(CorruptHeart, seed=42)

        enemy.roll_move()  # Debilitate (first)

        # Turn 0: Blood Shots or Echo (50/50)
        move1 = enemy.roll_move()
        assert move1.move_id in [CorruptHeart.BLOOD_SHOTS, CorruptHeart.ECHO]

        # Turn 1: The other attack
        move2 = enemy.roll_move()
        if move1.move_id == CorruptHeart.ECHO:
            assert move2.move_id == CorruptHeart.BLOOD_SHOTS
        # Could also be the same if not Echo last time

        # Turn 2: Buff
        # (need to advance to position 2)
        enemy.move_count = 2
        move3 = enemy.roll_move()
        assert move3.move_id == CorruptHeart.BUFF


# ============================================================================
# 8. ELITE-SPECIFIC PATTERNS AT A18+
# ============================================================================

class TestEliteA18Patterns:
    """Test elite-specific patterns that change at A18+."""

    def test_gremlin_nob_a18_skull_bash_priority(self):
        """Gremlin Nob at A18+ prioritizes Skull Bash if not used recently."""
        enemy = create_enemy(GremlinNob, seed=42, ascension=18)
        enemy.roll_move()  # Bellow first

        # A18+ should use Skull Bash if not in last 2 moves
        move = enemy.roll_move()
        assert move.move_id == GremlinNob.SKULL_BASH

    def test_sentries_a18_double_daze(self):
        """Sentries at A18+ apply 3 Daze instead of 2 (Java: Sentry.java:60)."""
        enemy_a0 = create_enemy(Sentries, seed=42, ascension=0, position=1)
        enemy_a18 = create_enemy(Sentries, seed=42, ascension=18, position=1)

        move_a0 = enemy_a0.roll_move()  # Beam
        move_a18 = enemy_a18.roll_move()  # Beam

        assert move_a0.effects.get("daze", 0) == 2
        assert move_a18.effects.get("daze", 0) == 3

    def test_book_of_stabbing_a18_stab_count_on_single(self):
        """Book of Stabbing at A18+ increments stab count even on Single Stab."""
        enemy = create_enemy(BookOfStabbing, seed=42, ascension=18)
        initial_count = enemy.stab_count

        # Force Single Stab (roll < 15 and last wasn't Single)
        move = enemy.get_move(5)

        if move.move_id == BookOfStabbing.SINGLE_STAB:
            # At A18+, stab count still increases
            assert enemy.stab_count == initial_count + 1

    def test_nemesis_a18_burn_count(self):
        """Nemesis at A18+ applies 5 Burns instead of 3."""
        enemy_a0 = create_enemy(Nemesis, seed=42, ascension=0)
        enemy_a18 = create_enemy(Nemesis, seed=42, ascension=18)

        dmg_a0 = enemy_a0._get_damage_values()
        dmg_a18 = enemy_a18._get_damage_values()

        assert dmg_a0["burn"] == 3
        assert dmg_a18["burn"] == 5

    def test_giant_head_a18_starts_at_4(self):
        """Giant Head at A18+ starts countdown at 4 instead of 5."""
        enemy_a0 = create_enemy(GiantHead, seed=42, ascension=0)
        enemy_a18 = create_enemy(GiantHead, seed=42, ascension=18)

        assert enemy_a0.count == 5
        assert enemy_a18.count == 4

    def test_reptomancer_a18_double_daggers(self):
        """Reptomancer at A18+ summons 2 daggers per Summon."""
        enemy_a0 = create_enemy(Reptomancer, seed=42, ascension=0)
        enemy_a18 = create_enemy(Reptomancer, seed=42, ascension=18)

        assert enemy_a0.daggers_per_spawn == 1
        assert enemy_a18.daggers_per_spawn == 2


# ============================================================================
# 9. BOSS-SPECIFIC MECHANICS
# ============================================================================

class TestBossSpecificMechanics:
    """Test unique boss mechanics."""

    def test_heart_invincible_power(self):
        """Corrupt Heart has Invincible 300 (A19: 200) at battle start."""
        enemy_a0 = create_enemy(CorruptHeart, ascension=0)
        enemy_a19 = create_enemy(CorruptHeart, ascension=19)

        effects_a0 = enemy_a0.get_pre_battle_effects()
        effects_a19 = enemy_a19.get_pre_battle_effects()

        assert effects_a0["self_effects"]["invincible"] == 300
        assert effects_a19["self_effects"]["invincible"] == 200

    def test_heart_beat_of_death(self):
        """Corrupt Heart has Beat of Death 1 (A19: 2)."""
        enemy_a0 = create_enemy(CorruptHeart, ascension=0)
        enemy_a19 = create_enemy(CorruptHeart, ascension=19)

        effects_a0 = enemy_a0.get_pre_battle_effects()
        effects_a19 = enemy_a19.get_pre_battle_effects()

        assert effects_a0["self_effects"]["beat_of_death"] == 1
        assert effects_a19["self_effects"]["beat_of_death"] == 2

    def test_heart_buff_cycle(self):
        """Corrupt Heart's buff move cycles through different buffs."""
        enemy = create_enemy(CorruptHeart, seed=42)

        # First move is always Debilitate, skip it
        enemy.roll_move()

        # Collect buff moves by properly cycling through the 3-turn cycle
        buff_effects = []
        for i in range(15):  # 5 full cycles (each has 1 buff on turn 2)
            enemy.is_first_move = False
            move = enemy.roll_move()
            if move.move_id == CorruptHeart.BUFF:
                buff_effects.append(move.effects.copy())
                if len(buff_effects) >= 3:
                    break

        # First buff (buff_count=0) should have artifact
        assert "artifact" in buff_effects[0]
        # Second buff (buff_count=1) should have beat_of_death
        assert "beat_of_death" in buff_effects[1]

    def test_awakened_one_curiosity(self):
        """Awakened One has Curiosity power (gain strength on player Power)."""
        enemy_a0 = create_enemy(AwakenedOne, ascension=0)
        enemy_a19 = create_enemy(AwakenedOne, ascension=19)

        effects_a0 = enemy_a0.get_pre_battle_effects()
        effects_a19 = enemy_a19.get_pre_battle_effects()

        assert effects_a0["self_effects"]["curiosity"] == 1
        assert effects_a19["self_effects"]["curiosity"] == 2

    def test_awakened_one_regenerate(self):
        """Awakened One has Regenerate 10 (A19: 15) HP/turn."""
        enemy_a0 = create_enemy(AwakenedOne, ascension=0)
        enemy_a19 = create_enemy(AwakenedOne, ascension=19)

        effects_a0 = enemy_a0.get_pre_battle_effects()
        effects_a19 = enemy_a19.get_pre_battle_effects()

        assert effects_a0["self_effects"]["regenerate"] == 10
        assert effects_a19["self_effects"]["regenerate"] == 15

    def test_donu_deca_artifact(self):
        """Donu and Deca have Artifact 2 (A19: 3)."""
        donu_a0 = create_enemy(Donu, ascension=0)
        donu_a19 = create_enemy(Donu, ascension=19)
        deca_a0 = create_enemy(Deca, ascension=0)
        deca_a19 = create_enemy(Deca, ascension=19)

        assert donu_a0.get_pre_battle_effects()["self_effects"]["artifact"] == 2
        assert donu_a19.get_pre_battle_effects()["self_effects"]["artifact"] == 3
        assert deca_a0.get_pre_battle_effects()["self_effects"]["artifact"] == 2
        assert deca_a19.get_pre_battle_effects()["self_effects"]["artifact"] == 3

    def test_deca_a19_plated_armor(self):
        """Deca at A19+ gives Plated Armor with Square of Protection."""
        enemy = create_enemy(Deca, ascension=19)
        enemy.roll_move()  # Beam

        move = enemy.roll_move()  # Square
        assert "plated_armor_all_monsters" in move.effects
        assert move.effects["plated_armor_all_monsters"] == 3

    def test_automaton_artifact(self):
        """Bronze Automaton has 3 Artifact at battle start."""
        enemy = create_enemy(BronzeAutomaton)
        effects = enemy.get_pre_battle_effects()

        assert effects["self_effects"]["artifact"] == 3

    def test_collector_mega_debuff_timing(self):
        """Collector uses Mega Debuff after turn 3."""
        enemy = create_enemy(TheCollector, seed=42)

        # Turn 1: Spawn
        enemy.roll_move()
        assert enemy.turns_taken == 1

        # Turns 2-3: Normal moves
        enemy.roll_move()
        enemy.roll_move()

        # Turn 4: Should use Mega Debuff
        move = enemy.roll_move()
        assert move.move_id == TheCollector.MEGA_DEBUFF

    def test_giant_head_slow_power(self):
        """Giant Head applies Slow power at battle start."""
        enemy = create_enemy(GiantHead, seed=42)

        effects = enemy.use_pre_battle_action()
        assert "apply_slow" in effects

    def test_nemesis_intangible(self):
        """Nemesis gains Intangible at end of each turn."""
        enemy = create_enemy(Nemesis, seed=42)

        # Initially no intangible
        assert enemy.state.powers.get("intangible", 0) == 0

        # End of turn effect
        effect = enemy.end_of_turn_effect()
        assert effect.get("gain_intangible") == 1
        assert enemy.state.powers["intangible"] == 1


# ============================================================================
# 10. MULTI-ENEMY ENCOUNTERS
# ============================================================================

class TestMultiEnemyEncounters:
    """Test multi-enemy encounter mechanics."""

    def test_gremlin_leader_ai_based_on_minion_count(self):
        """Gremlin Leader AI changes based on number of alive gremlins."""
        enemy = create_enemy(GremlinLeader, seed=42, num_gremlins_alive=2)

        # With 2 gremlins: prefers Encourage
        move_2 = enemy.get_move(30)  # Roll in Encourage range
        assert move_2.move_id in [GremlinLeader.ENCOURAGE, GremlinLeader.STAB]

        # Update to 0 gremlins
        enemy.update_gremlin_count(0)
        enemy.state.move_history = []  # Reset history

        # With 0 gremlins: prefers Rally
        move_0 = enemy.get_move(30)
        assert move_0.move_id in [GremlinLeader.RALLY, GremlinLeader.STAB]

    def test_gremlin_escape_trigger(self):
        """Gremlins set escape_next when triggered."""
        fat = create_enemy(GremlinFat, seed=42)
        thief = create_enemy(GremlinThief, seed=42)
        shield = create_enemy(GremlinTsundere, seed=42)
        warrior = create_enemy(GremlinWarrior, seed=42)
        wizard = create_enemy(GremlinWizard, seed=42)

        # Trigger escape on all
        for gremlin in [fat, thief, shield, warrior, wizard]:
            gremlin.trigger_escape()
            move = gremlin.roll_move()
            assert move.intent == Intent.ESCAPE

    def test_shield_gremlin_protect_vs_bash(self):
        """Shield Gremlin uses Protect with allies, Bash when alone."""
        enemy = create_enemy(GremlinTsundere, seed=42)

        move_with_allies = enemy.get_move(50, allies_alive=3)
        assert move_with_allies.move_id == GremlinTsundere.PROTECT

        move_alone = enemy.get_move(50, allies_alive=1)
        assert move_alone.move_id == GremlinTsundere.BASH

    def test_darkling_position_affects_chomp(self):
        """Only even-indexed Darklings can use Chomp."""
        darkling_0 = create_enemy(Darkling, seed=42, position=0)
        darkling_1 = create_enemy(Darkling, seed=42, position=1)

        # Set up so both would try Chomp
        darkling_0.state.first_turn = False
        darkling_1.state.first_turn = False

        # Position 0 (even) can Chomp
        move_0 = darkling_0.get_move(20)
        # Position 1 (odd) cannot, will recurse
        move_1 = darkling_1.get_move(20)

        assert move_0.move_id == Darkling.CHOMP
        assert move_1.move_id != Darkling.CHOMP

    def test_darkling_reincarnate_when_half_dead(self):
        """Darkling uses Reincarnate when in half-dead state."""
        enemy = create_enemy(Darkling, seed=42, position=0)
        enemy.half_dead = True

        move = enemy.roll_move()
        assert move.move_id == Darkling.REINCARNATE

    def test_reptomancer_pre_battle_daggers(self):
        """Reptomancer spawns with 2 daggers at start."""
        enemy = create_enemy(Reptomancer, seed=42)
        effects = enemy.use_pre_battle_action()

        assert "spawn_daggers" in effects
        assert len(effects["spawn_daggers"]) == 2
        assert enemy.dagger_slots[0] is True
        assert enemy.dagger_slots[1] is True

    def test_reptomancer_spawn_limit(self):
        """Reptomancer respects 4 monster limit."""
        enemy = create_enemy(Reptomancer, seed=42)

        assert enemy.can_spawn(1)  # Just Reptomancer
        assert enemy.can_spawn(3)  # Reptomancer + 2 daggers
        assert not enemy.can_spawn(4)  # Would exceed limit

    def test_spire_shield_spear_surrounded(self):
        """Spire Shield applies Surrounded to player."""
        enemy = create_enemy(SpireShield, seed=42)
        effects = enemy.get_pre_battle_effects()

        assert "surrounded" in effects["player_effects"]

    def test_spire_shield_spear_artifact_a18(self):
        """Spire Shield/Spear have 2 Artifact at A18+ (vs 1)."""
        shield_a0 = create_enemy(SpireShield, ascension=0)
        shield_a18 = create_enemy(SpireShield, ascension=18)
        spear_a0 = create_enemy(SpireSpear, ascension=0)
        spear_a18 = create_enemy(SpireSpear, ascension=18)

        assert shield_a0.get_pre_battle_effects()["self_effects"]["artifact"] == 1
        assert shield_a18.get_pre_battle_effects()["self_effects"]["artifact"] == 2
        assert spear_a0.get_pre_battle_effects()["self_effects"]["artifact"] == 1
        assert spear_a18.get_pre_battle_effects()["self_effects"]["artifact"] == 2


# ============================================================================
# ADDITIONAL RNG DETERMINISM TESTS
# ============================================================================

class TestRNGDeterminism:
    """Verify that enemy AI is fully deterministic with same seed."""

    def test_jaw_worm_sequence_deterministic(self):
        """Same seed produces same Jaw Worm move sequence."""
        moves1 = []
        moves2 = []

        for _ in range(10):
            enemy1 = create_enemy(JawWorm, seed=12345)
            enemy2 = create_enemy(JawWorm, seed=12345)

            for _ in range(5):
                moves1.append(enemy1.roll_move().move_id)
            for _ in range(5):
                moves2.append(enemy2.roll_move().move_id)

        assert moves1 == moves2

    def test_slime_ai_deterministic_a17(self):
        """Acid Slime M produces deterministic moves at A17."""
        seed = 54321

        sequences = []
        for _ in range(3):
            enemy = create_enemy(AcidSlimeM, seed=seed, ascension=17)
            seq = [enemy.roll_move().move_id for _ in range(10)]
            sequences.append(seq)

        assert all(s == sequences[0] for s in sequences)

    def test_byrd_first_turn_deterministic(self):
        """Byrd first turn (37.5% Caw) is deterministic."""
        results = []
        for seed in [1, 2, 3, 4, 5]:
            enemy = create_enemy(Byrd, seed=seed)
            move = enemy.roll_move()
            results.append((seed, move.move_id))

        # Same seeds should give same results on re-run
        results2 = []
        for seed in [1, 2, 3, 4, 5]:
            enemy = create_enemy(Byrd, seed=seed)
            move = enemy.roll_move()
            results2.append((seed, move.move_id))

        assert results == results2

    def test_writhing_mass_deterministic(self):
        """Writhing Mass with reactive behavior is deterministic."""
        seed = 99999

        enemy1 = create_enemy(WrithingMass, seed=seed)
        enemy2 = create_enemy(WrithingMass, seed=seed)

        moves1 = [enemy1.roll_move().move_id for _ in range(10)]
        moves2 = [enemy2.roll_move().move_id for _ in range(10)]

        assert moves1 == moves2


# ============================================================================
# EDGE CASES AND SPECIAL SCENARIOS
# ============================================================================

class TestEdgeCases:
    """Test edge cases and special scenarios."""

    def test_spike_slime_s_only_one_move(self):
        """Spike Slime (S) only has Tackle."""
        enemy = create_enemy(SpikeSlimeS, seed=42)

        for _ in range(10):
            move = enemy.roll_move()
            assert move.move_id == SpikeSlimeS.TACKLE

    def test_torch_head_only_one_move(self):
        """Torch Head only has Tackle."""
        enemy = create_enemy(TorchHead, seed=42)

        for _ in range(10):
            move = enemy.roll_move()
            assert move.move_id == TorchHead.TACKLE

    def test_bandit_pointy_only_one_move(self):
        """Bandit Pointy only has Stab."""
        enemy = create_enemy(BanditPointy, seed=42)

        for _ in range(10):
            move = enemy.roll_move()
            assert move.move_id == BanditPointy.ATTACK

    def test_snake_dagger_two_move_cycle(self):
        """Snake Dagger: Stab first, then Explode."""
        enemy = create_enemy(SnakeDagger, seed=42)

        move1 = enemy.roll_move()
        assert move1.move_id == SnakeDagger.WOUND

        move2 = enemy.roll_move()
        assert move2.move_id == SnakeDagger.EXPLODE

    def test_exploder_attack_then_explode(self):
        """Exploder attacks for 2 turns then explodes."""
        enemy = create_enemy(Exploder, seed=42)

        move1 = enemy.roll_move()
        assert move1.move_id == Exploder.ATTACK

        move2 = enemy.roll_move()
        assert move2.move_id == Exploder.ATTACK

        move3 = enemy.roll_move()
        assert move3.move_id == Exploder.EXPLODE

    def test_spiker_max_thorns_buffs(self):
        """Spiker stops buffing after 5 thorns uses."""
        enemy = create_enemy(Spiker, seed=42)

        # Force 6 thorns buffs
        for i in range(6):
            enemy.thorns_count = i
            if i < 5:
                enemy.state.move_history = []  # Reset so it can buff

        enemy.thorns_count = 6
        move = enemy.roll_move()
        assert move.move_id == Spiker.ATTACK

    def test_acid_slime_s_a17_special_pattern(self):
        """Acid Slime (S) at A17+ has special alternating pattern."""
        enemy = create_enemy(AcidSlimeS, seed=42, ascension=17)

        # A17+: Checks lastTwoMoves(TACKLE), forces TACKLE if true (bug in source?)
        # Otherwise uses LICK
        move1 = enemy.roll_move()
        # First move depends on history - no history means LICK
        assert move1.move_id == AcidSlimeS.LICK

        # After LICK, should... actually the logic is weird in source
        # This tests the implementation matches the source behavior

    def test_louse_curl_up_power(self):
        """Louse starts with Curl Up power."""
        for is_red in [True, False]:
            enemy = create_enemy(Louse, seed=42, is_red=is_red)
            assert "curl_up" in enemy.state.powers
            assert enemy.state.powers["curl_up"] > 0

    def test_louse_bite_damage_rolled_at_start(self):
        """Louse bite damage is rolled at combat start."""
        enemy_a0 = create_enemy(Louse, seed=42, ascension=0)
        enemy_a2 = create_enemy(Louse, seed=42, ascension=2)

        # A0: 5-7, A2+: 6-8
        assert 5 <= enemy_a0.bite_damage <= 7
        assert 6 <= enemy_a2.bite_damage <= 8

    def test_fungi_beast_spore_cloud(self):
        """Fungi Beast has Spore Cloud power."""
        enemy = create_enemy(FungiBeast, seed=42)
        assert "spore_cloud" in enemy.state.powers
        assert enemy.state.powers["spore_cloud"] == 2

    def test_shelled_parasite_starts_with_armor(self):
        """Shelled Parasite starts with 14 Plated Armor and 14 Block."""
        enemy = create_enemy(ShelledParasite, seed=42)

        assert enemy.state.powers.get("plated_armor") == 14
        assert enemy.state.block == 14

    def test_spheric_guardian_starts_with_powers(self):
        """Spheric Guardian starts with Barricade, 3 Artifact, 40 Block."""
        enemy = create_enemy(SphericGuardian, seed=42)

        assert enemy.state.powers.get("barricade") == 1
        assert enemy.state.powers.get("artifact") == 3
        assert enemy.state.block == 40

    def test_mad_gremlin_angry_power(self):
        """Mad Gremlin starts with Angry power."""
        enemy_a0 = create_enemy(GremlinWarrior, seed=42, ascension=0)
        enemy_a17 = create_enemy(GremlinWarrior, seed=42, ascension=17)

        assert enemy_a0.state.powers["angry"] == 1
        assert enemy_a17.state.powers["angry"] == 2


# ============================================================================
# MOVE INFO VALIDATION
# ============================================================================

class TestMoveInfoValidation:
    """Validate MoveInfo objects have correct attributes."""

    def test_attack_move_has_damage(self):
        """Attack moves have positive base_damage."""
        enemy = create_enemy(JawWorm, seed=42)
        move = enemy.roll_move()  # Chomp

        assert move.intent == Intent.ATTACK
        assert move.base_damage > 0

    def test_buff_move_has_effects(self):
        """Buff moves have effects dictionary."""
        enemy = create_enemy(Cultist, seed=42)
        move = enemy.roll_move()  # Incantation

        assert move.intent == Intent.BUFF
        assert "ritual" in move.effects

    def test_multi_attack_has_hits(self):
        """Multi-hit attacks have hits > 1."""
        enemy = create_enemy(Hexaghost, seed=42)
        enemy.roll_move()  # Activate
        move = enemy.roll_move()  # Divider (6 hits)

        assert move.is_multi
        assert move.hits == 6

    def test_defend_move_has_block(self):
        """Defend moves have block > 0."""
        enemy = create_enemy(TheGuardian, seed=42)
        move = enemy.roll_move()  # Charging Up

        assert move.intent == Intent.DEFEND
        assert move.block == 9

    def test_attack_defend_has_both(self):
        """Attack+Defend moves have both damage and block."""
        enemy = create_enemy(JawWorm, seed=42)
        enemy.roll_move()  # Chomp

        # Force Thrash
        enemy.state.move_history[-1] = JawWorm.BELLOW
        move = enemy.get_move(35)  # Roll in Thrash range

        if move.move_id == JawWorm.THRASH:
            assert move.intent == Intent.ATTACK_DEFEND
            assert move.base_damage > 0
            assert move.block > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
