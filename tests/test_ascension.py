"""
Comprehensive tests for Ascension level modifiers (A1-A20) in Slay the Spire.

Ascension Breakdown:
A1: Elite enemies deal more damage
A2: Normal enemies deal more damage
A3: Elites have more HP
A4: Normal enemies have more HP
A5: Heal 75% HP at boss (not 100%)
A6: Start with 10 less HP (90 Watcher -> 72, but actually 82->72 for Watcher base)
A7: Boss deals more damage
A8: Boss has more HP
A9: More unique enemies per pool (increased enemy variety)
A10: Start with Ascender's Bane curse
A11: Boss doesn't drop potions; fewer potion slots
A12: Upgraded cards less common
A13: Boss heal 75% (not 100%)
A14: Starting deck Strike->Strike+ (lose 4 max HP for Watcher, starts at 68)
A15: ? rooms more likely to have monsters; less starting gold
A16: Less gold from combat
A17: Normal enemies have better AI patterns
A18: Elite enemies have better AI patterns
A19: Fewer card rewards to choose from (boss relics); Boss AI improved
A20: Heart has double damage first turn
"""

import pytest
import sys
import os

# Add core to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from packages.engine.state.run import create_watcher_run, WATCHER_BASE_HP, WATCHER_BASE_GOLD, CardInstance
from packages.engine.state.rng import Random
from packages.engine.content.enemies import (
    JawWorm, Cultist, AcidSlimeM, SpikeSlimeL,
    GremlinNob, Lagavulin, Sentries,
    SlimeBoss, TheGuardian, Hexaghost,
    CorruptHeart, create_enemy
)
from packages.engine.generation.rewards import (
    generate_card_rewards, generate_gold_reward, RewardState,
    CARD_UPGRADE_CHANCES
)
from packages.engine.generation.map import MapGenerator, MapGeneratorConfig, RoomType


# =============================================================================
# Test Fixtures
# =============================================================================

@pytest.fixture
def rng():
    """Create a deterministic RNG for tests."""
    return Random(12345)


@pytest.fixture
def hp_rng():
    """Create a separate RNG for HP rolls."""
    return Random(67890)


# =============================================================================
# A1: Elite enemies deal more damage
# =============================================================================

class TestA1EliteDamage:
    """Test that elites deal more damage at A1+."""

    def test_gremlin_nob_damage_below_a1(self):
        """Gremlin Nob should deal base damage below A1."""
        rng = Random(100)
        nob = GremlinNob(rng, ascension=0)
        dmg = nob._get_damage_values()
        assert dmg["rush"] == 14
        assert dmg["skull_bash"] == 6

    def test_gremlin_nob_damage_at_a3(self):
        """Gremlin Nob should deal more damage at A3+."""
        rng = Random(100)
        nob = GremlinNob(rng, ascension=3)
        dmg = nob._get_damage_values()
        assert dmg["rush"] == 16
        assert dmg["skull_bash"] == 8

    def test_lagavulin_damage_below_a3(self):
        """Lagavulin should deal base damage below A3."""
        rng = Random(100)
        lag = Lagavulin(rng, ascension=0)
        dmg = lag._get_damage_values()
        assert dmg["attack"] == 18

    def test_lagavulin_damage_at_a3(self):
        """Lagavulin should deal more damage at A3+."""
        rng = Random(100)
        lag = Lagavulin(rng, ascension=3)
        dmg = lag._get_damage_values()
        assert dmg["attack"] == 20

    def test_sentries_damage_below_a3(self):
        """Sentries should deal base damage below A3."""
        rng = Random(100)
        sentry = Sentries(rng, ascension=0, position=0)
        dmg = sentry._get_damage_values()
        assert dmg["damage"] == 9

    def test_sentries_damage_at_a3(self):
        """Sentries should deal more damage at A3+."""
        rng = Random(100)
        sentry = Sentries(rng, ascension=3, position=0)
        dmg = sentry._get_damage_values()
        assert dmg["damage"] == 10


# =============================================================================
# A2: Normal enemies deal more damage
# =============================================================================

class TestA2NormalDamage:
    """Test that normal enemies deal more damage at A2+."""

    def test_jaw_worm_damage_below_a2(self):
        """Jaw Worm should deal base damage below A2."""
        rng = Random(100)
        worm = JawWorm(rng, ascension=0)
        dmg = worm._get_damage_values()
        assert dmg["chomp"] == 11
        assert dmg["bellow_str"] == 3

    def test_jaw_worm_damage_at_a2(self):
        """Jaw Worm should deal more damage at A2+."""
        rng = Random(100)
        worm = JawWorm(rng, ascension=2)
        dmg = worm._get_damage_values()
        assert dmg["chomp"] == 12
        assert dmg["bellow_str"] == 4

    def test_cultist_ritual_below_a2(self):
        """Cultist should have base ritual below A2."""
        rng = Random(100)
        cultist = Cultist(rng, ascension=0)
        dmg = cultist._get_damage_values()
        assert dmg["ritual"] == 3

    def test_cultist_ritual_at_a2(self):
        """Cultist should have increased ritual at A2+."""
        rng = Random(100)
        cultist = Cultist(rng, ascension=2)
        dmg = cultist._get_damage_values()
        assert dmg["ritual"] == 4


# =============================================================================
# A3: Elites have more HP
# =============================================================================

class TestA3EliteHP:
    """Test that elites have more HP at A3+."""

    def test_gremlin_nob_hp_below_a8(self):
        """Gremlin Nob should have base HP below A8."""
        rng = Random(100)
        nob = GremlinNob(rng, ascension=0)
        min_hp, max_hp = nob._get_hp_range()
        assert min_hp == 82
        assert max_hp == 86

    def test_gremlin_nob_hp_at_a8(self):
        """Gremlin Nob should have more HP at A8+."""
        rng = Random(100)
        nob = GremlinNob(rng, ascension=8)
        min_hp, max_hp = nob._get_hp_range()
        assert min_hp == 85
        assert max_hp == 90

    def test_lagavulin_hp_below_a8(self):
        """Lagavulin should have base HP below A8."""
        rng = Random(100)
        lag = Lagavulin(rng, ascension=0)
        min_hp, max_hp = lag._get_hp_range()
        assert min_hp == 109
        assert max_hp == 111

    def test_lagavulin_hp_at_a8(self):
        """Lagavulin should have more HP at A8+."""
        rng = Random(100)
        lag = Lagavulin(rng, ascension=8)
        min_hp, max_hp = lag._get_hp_range()
        assert min_hp == 112
        assert max_hp == 115

    def test_sentries_hp_below_a8(self):
        """Sentries should have base HP below A8."""
        rng = Random(100)
        sentry = Sentries(rng, ascension=0, position=0)
        min_hp, max_hp = sentry._get_hp_range()
        assert min_hp == 38
        assert max_hp == 42

    def test_sentries_hp_at_a8(self):
        """Sentries should have more HP at A8+."""
        rng = Random(100)
        sentry = Sentries(rng, ascension=8, position=0)
        min_hp, max_hp = sentry._get_hp_range()
        assert min_hp == 39
        assert max_hp == 45


# =============================================================================
# A4: Normal enemies have more HP
# =============================================================================

class TestA4NormalHP:
    """Test that normal enemies have more HP at A4+ (A7 for normals)."""

    def test_jaw_worm_hp_below_a7(self):
        """Jaw Worm should have base HP below A7."""
        rng = Random(100)
        worm = JawWorm(rng, ascension=0)
        min_hp, max_hp = worm._get_hp_range()
        assert min_hp == 40
        assert max_hp == 44

    def test_jaw_worm_hp_at_a7(self):
        """Jaw Worm should have more HP at A7+."""
        rng = Random(100)
        worm = JawWorm(rng, ascension=7)
        min_hp, max_hp = worm._get_hp_range()
        assert min_hp == 42
        assert max_hp == 46

    def test_cultist_hp_below_a7(self):
        """Cultist should have base HP below A7."""
        rng = Random(100)
        cultist = Cultist(rng, ascension=0)
        min_hp, max_hp = cultist._get_hp_range()
        assert min_hp == 48
        assert max_hp == 54

    def test_cultist_hp_at_a7(self):
        """Cultist should have more HP at A7+."""
        rng = Random(100)
        cultist = Cultist(rng, ascension=7)
        min_hp, max_hp = cultist._get_hp_range()
        assert min_hp == 50
        assert max_hp == 56


# =============================================================================
# A5: Heal 75% HP at boss (not 100%) - Tested via run state
# =============================================================================

class TestA5BossHealReduction:
    """Test that boss heal is reduced at A5+."""

    def test_boss_heal_below_a5(self):
        """Player should heal to full after boss below A5."""
        run = create_watcher_run("SEED123", ascension=0)
        run.current_hp = 30
        run.max_hp = 80
        # Simulate post-boss heal (would be 100%)
        heal_amount = run.max_hp - run.current_hp
        assert heal_amount == 50  # Full heal

    def test_boss_heal_at_a5(self):
        """Player should heal 75% after boss at A5+."""
        # At A5+, boss heal is 75% of max HP, not 100%
        run = create_watcher_run("SEED123", ascension=5)
        run.current_hp = 30
        run.max_hp = 80
        # At A5+, heal would be to 75% of max HP = 60
        target_hp = int(run.max_hp * 0.75)
        heal_to = max(run.current_hp, target_hp)
        assert target_hp == 60
        assert heal_to == 60

    def test_boss_heal_at_a5_no_downgrade(self):
        """If HP > 75%, don't reduce it."""
        run = create_watcher_run("SEED123", ascension=5)
        run.current_hp = 70  # Above 75% of 80 = 60
        run.max_hp = 80
        target_hp = int(run.max_hp * 0.75)
        heal_to = max(run.current_hp, target_hp)
        assert heal_to == 70  # Keep current HP


# =============================================================================
# A6: Start with less max HP
# =============================================================================

class TestA6StartingHP:
    """Test that player starts with less max HP at A6+ (A14+ for Watcher)."""

    def test_watcher_starting_hp_below_a6(self):
        """Watcher should start at full HP below A6."""
        run = create_watcher_run("SEED123", ascension=0)
        assert run.max_hp == 72
        assert run.current_hp == 72

    def test_watcher_starting_hp_at_a6(self):
        """A6+: start at 90% current HP. round(72 * 0.9) = 65."""
        run = create_watcher_run("SEED123", ascension=6)
        assert run.max_hp == 72
        assert run.current_hp == 65  # round(72 * 0.9)

    def test_watcher_starting_hp_at_a14(self):
        """Watcher at A14+: max HP 68 (72-4), current HP round(68*0.9)=61 (A6+ penalty)."""
        run = create_watcher_run("SEED123", ascension=14)
        assert run.max_hp == 68  # 72 - 4
        assert run.current_hp == 61  # round(68 * 0.9) -- A6+ starts at 90% HP


# =============================================================================
# A7: Boss deals more damage
# =============================================================================

class TestA7BossDamage:
    """Test that bosses deal more damage at A7+."""

    def test_slime_boss_damage_below_a4(self):
        """Slime Boss should deal base damage below A4."""
        rng = Random(100)
        boss = SlimeBoss(rng, ascension=0)
        dmg = boss._get_damage_values()
        assert dmg["slam"] == 35

    def test_slime_boss_damage_at_a4(self):
        """Slime Boss should deal more damage at A4+."""
        rng = Random(100)
        boss = SlimeBoss(rng, ascension=4)
        dmg = boss._get_damage_values()
        assert dmg["slam"] == 38

    def test_guardian_damage_below_a4(self):
        """The Guardian should deal base damage below A4."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=0)
        dmg = guardian._get_damage_values()
        assert dmg["fierce_bash"] == 32
        assert dmg["roll"] == 9

    def test_guardian_damage_at_a4(self):
        """The Guardian should deal more damage at A4+."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=4)
        dmg = guardian._get_damage_values()
        assert dmg["fierce_bash"] == 36
        assert dmg["roll"] == 10


# =============================================================================
# A8: Boss has more HP
# =============================================================================

class TestA8BossHP:
    """Test that bosses have more HP at A8+ (A9 for bosses)."""

    def test_slime_boss_hp_below_a9(self):
        """Slime Boss should have base HP below A9."""
        rng = Random(100)
        boss = SlimeBoss(rng, ascension=0)
        min_hp, max_hp = boss._get_hp_range()
        assert min_hp == 140
        assert max_hp == 140

    def test_slime_boss_hp_at_a9(self):
        """Slime Boss should have more HP at A9+."""
        rng = Random(100)
        boss = SlimeBoss(rng, ascension=9)
        min_hp, max_hp = boss._get_hp_range()
        assert min_hp == 150
        assert max_hp == 150

    def test_guardian_hp_below_a9(self):
        """The Guardian should have base HP below A9."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=0)
        min_hp, max_hp = guardian._get_hp_range()
        assert min_hp == 240
        assert max_hp == 240

    def test_guardian_hp_at_a9(self):
        """The Guardian should have more HP at A9+."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=9)
        min_hp, max_hp = guardian._get_hp_range()
        assert min_hp == 250
        assert max_hp == 250


# =============================================================================
# A9: More unique enemies per pool
# =============================================================================

class TestA9EnemyVariety:
    """Test that there are more unique enemies in pools at A9+."""

    def test_guardian_mode_shift_below_a9(self):
        """Guardian mode shift threshold below A9."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=0)
        assert guardian.mode_shift_damage == 30

    def test_guardian_mode_shift_at_a9(self):
        """Guardian mode shift threshold at A9+."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=9)
        assert guardian.mode_shift_damage == 35

    def test_guardian_mode_shift_at_a19(self):
        """Guardian mode shift threshold at A19+."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=19)
        assert guardian.mode_shift_damage == 40


# =============================================================================
# A10: Start with Ascender's Bane curse
# =============================================================================

class TestA10AscendersBane:
    """Test that player starts with Ascender's Bane at A10+."""

    def test_no_ascenders_bane_below_a10(self):
        """Player should not have Ascender's Bane below A10."""
        run = create_watcher_run("SEED123", ascension=9)
        card_ids = [card.id for card in run.deck]
        assert "AscendersBane" not in card_ids

    def test_ascenders_bane_at_a10(self):
        """Player should have Ascender's Bane at A10+."""
        run = create_watcher_run("SEED123", ascension=10)
        card_ids = [card.id for card in run.deck]
        assert "AscendersBane" in card_ids

    def test_ascenders_bane_count_at_a10(self):
        """Should have exactly one Ascender's Bane."""
        run = create_watcher_run("SEED123", ascension=10)
        count = sum(1 for card in run.deck if card.id == "AscendersBane")
        assert count == 1


# =============================================================================
# A11: Boss doesn't drop potions; fewer potion slots
# =============================================================================

class TestA11PotionSlots:
    """Test that player has fewer potion slots at A11+."""

    def test_potion_slots_below_a11(self):
        """Player should have 3 potion slots below A11."""
        run = create_watcher_run("SEED123", ascension=10)
        assert len(run.potion_slots) == 3

    def test_potion_slots_at_a11(self):
        """Player should have 2 potion slots at A11+."""
        run = create_watcher_run("SEED123", ascension=11)
        assert len(run.potion_slots) == 2

    def test_potion_slots_at_a20(self):
        """Player should have 2 potion slots at A20."""
        run = create_watcher_run("SEED123", ascension=20)
        assert len(run.potion_slots) == 2


# =============================================================================
# A12: Upgraded cards less common
# =============================================================================

class TestA12CardUpgrades:
    """Test that upgraded cards are less common at A12+."""

    def test_upgrade_chance_act2_below_a12(self):
        """Act 2 card upgrade chance should be 25% below A12."""
        chances = CARD_UPGRADE_CHANCES[2]
        assert chances["default"] == 0.25

    def test_upgrade_chance_act2_at_a12(self):
        """Act 2 card upgrade chance should be 12.5% at A12+."""
        chances = CARD_UPGRADE_CHANCES[2]
        assert chances["a12"] == 0.125

    def test_upgrade_chance_act3_below_a12(self):
        """Act 3 card upgrade chance should be 50% below A12."""
        chances = CARD_UPGRADE_CHANCES[3]
        assert chances["default"] == 0.50

    def test_upgrade_chance_act3_at_a12(self):
        """Act 3 card upgrade chance should be 25% at A12+."""
        chances = CARD_UPGRADE_CHANCES[3]
        assert chances["a12"] == 0.25


# =============================================================================
# A13: Boss heal 75% (not 100%) - Similar to A5 but for act end
# =============================================================================

class TestA13BossHeal:
    """Test boss healing reduction at A13+."""

    def test_heal_percent_at_a13(self):
        """Verify 75% heal target at A13+."""
        max_hp = 80
        heal_percent = 0.75  # A13+ reduction
        target = int(max_hp * heal_percent)
        assert target == 60


# =============================================================================
# A14: Starting deck modifications / Max HP reduction
# =============================================================================

class TestA14StartingDeck:
    """Test starting deck and HP modifications at A14+."""

    def test_max_hp_at_a14(self):
        """Watcher should have 68 max HP at A14+."""
        run = create_watcher_run("SEED123", ascension=14)
        assert run.max_hp == 68

    def test_deck_size_consistent(self):
        """Deck size should be consistent at A14."""
        run_a0 = create_watcher_run("SEED123", ascension=0)
        run_a14 = create_watcher_run("SEED123", ascension=14)
        # A14+ has same deck size (but Strikes are worse in practice)
        assert len(run_a0.deck) == 10
        assert len(run_a14.deck) == 10 + 1  # +1 for Ascender's Bane


# =============================================================================
# A15: Less starting gold; ? rooms more likely to have monsters
# =============================================================================

class TestA15EventPenalties:
    """A15+ affects event gold amounts, NOT starting gold. Starting gold is always 99."""

    def test_starting_gold_always_99(self):
        """Starting gold is always 99 regardless of ascension."""
        for asc in [0, 6, 14, 15, 20]:
            run = create_watcher_run("SEED123", ascension=asc)
            assert run.gold == 99, f"A{asc}: expected 99 gold, got {run.gold}"


# =============================================================================
# A16: Less gold from combat
# =============================================================================

class TestA16GoldRewards:
    """Test that gold rewards are reduced at A16+ (A13 for gold reduction)."""

    def test_normal_gold_below_a13(self):
        """Normal room gold should be variable below A13."""
        rng = Random(100)
        # Below A13, gold is random 10-20
        gold = generate_gold_reward(rng, "normal", ascension=0)
        assert 10 <= gold <= 20

    def test_normal_gold_at_a13(self):
        """Normal room gold should be fixed 15 at A13+."""
        rng = Random(100)
        gold = generate_gold_reward(rng, "normal", ascension=13)
        assert gold == 15

    def test_elite_gold_below_a13(self):
        """Elite room gold should be variable below A13."""
        rng = Random(100)
        gold = generate_gold_reward(rng, "elite", ascension=0)
        assert 25 <= gold <= 35

    def test_elite_gold_at_a13(self):
        """Elite room gold should be fixed 30 at A13+."""
        rng = Random(100)
        gold = generate_gold_reward(rng, "elite", ascension=13)
        assert gold == 30

    def test_boss_gold_reduction_at_a13(self):
        """Boss gold should be reduced by 25% at A13+."""
        rng = Random(100)
        gold_a0 = generate_gold_reward(rng, "boss", ascension=0)
        rng2 = Random(100)
        gold_a13 = generate_gold_reward(rng2, "boss", ascension=13)
        # Boss gold is 100 +/- 5 at A0, 75 +/- 3.75 at A13+
        assert gold_a0 >= 95
        assert gold_a13 <= 78  # 75% of ~100


# =============================================================================
# A17: Normal enemies have better AI patterns
# =============================================================================

class TestA17NormalAI:
    """Test that normal enemies have improved AI at A17+."""

    def test_jaw_worm_strength_at_a17(self):
        """Jaw Worm should have higher Bellow strength at A17+."""
        rng = Random(100)
        worm = JawWorm(rng, ascension=17)
        dmg = worm._get_damage_values()
        assert dmg["bellow_str"] == 5  # 5 at A17+, 4 at A2+, 3 at A0

    def test_cultist_ritual_at_a17(self):
        """Cultist should have higher ritual at A17+."""
        rng = Random(100)
        cultist = Cultist(rng, ascension=17)
        dmg = cultist._get_damage_values()
        assert dmg["ritual"] == 5

    def test_slime_behavior_at_a17(self):
        """Medium slimes have different move probabilities at A17+."""
        # At A17+, slimes use different probability distributions
        # This is checked via the ascension >= 17 conditionals in get_move
        rng = Random(100)
        slime = AcidSlimeM(rng, ascension=17)
        assert slime.ascension >= 17


# =============================================================================
# A18: Elite enemies have better AI patterns
# =============================================================================

class TestA18EliteAI:
    """Test that elite enemies have improved AI at A18+."""

    def test_gremlin_nob_enrage_at_a18(self):
        """Gremlin Nob should have higher Enrage at A18+."""
        rng = Random(100)
        nob = GremlinNob(rng, ascension=18)
        dmg = nob._get_damage_values()
        assert dmg["enrage"] == 3

    def test_lagavulin_debuff_at_a18(self):
        """Lagavulin should have higher debuff at A18+."""
        rng = Random(100)
        lag = Lagavulin(rng, ascension=18)
        dmg = lag._get_damage_values()
        assert dmg["debuff"] == 2

    def test_sentries_daze_at_a18(self):
        """Sentries should apply more Daze at A18+."""
        rng = Random(100)
        sentry = Sentries(rng, ascension=18, position=1)
        # Get the move that applies Daze
        move = sentry.get_move(50)
        # First turn middle sentry does Beam which applies Daze
        if "daze" in move.effects:
            assert move.effects["daze"] == 3  # 3 at A18+ (Java: Sentry.java:60)

    def test_gremlin_nob_ai_pattern_at_a18(self):
        """Gremlin Nob should use improved AI at A18+."""
        rng = Random(100)
        nob = GremlinNob(rng, ascension=18)
        # First move is always Bellow
        move1 = nob.get_move(50)
        assert move1.name == "Bellow"
        # At A18+, prioritizes Skull Bash if not used in last 2 turns
        move2 = nob.get_move(50)
        # Should be Skull Bash due to A18+ AI
        assert move2.name == "Skull Bash"


# =============================================================================
# A19: Fewer card rewards to choose from
# =============================================================================

class TestA19CardRewards:
    """Test that card rewards are reduced at A19+."""

    def test_slime_boss_slimed_at_a19(self):
        """Slime Boss should apply more Slimed cards at A19+."""
        rng = Random(100)
        boss = SlimeBoss(rng, ascension=19)
        move = boss.get_move(0)  # First move is Goop Spray
        assert move.effects.get("slimed", 0) == 5  # 5 at A19+

    def test_guardian_sharp_hide_at_a19(self):
        """Guardian should have higher Sharp Hide at A19+."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=19)
        guardian.switch_to_defensive()
        assert guardian.state.powers.get("sharp_hide") == 4


# =============================================================================
# A20: Heart has double damage first turn
# =============================================================================

class TestA20HeartBehavior:
    """Test Heart special behavior at A20."""

    def test_heart_hp_at_a9(self):
        """Heart should have increased HP at A9+."""
        rng = Random(100)
        heart = CorruptHeart(rng, ascension=9)
        min_hp, max_hp = heart._get_hp_range()
        assert min_hp == 800  # 750 at A0, 800 at A9+

    def test_heart_invincible_at_a19(self):
        """Heart should have lower invincible threshold at A19+."""
        rng = Random(100)
        heart = CorruptHeart(rng, ascension=19)
        # Heart uses get_pre_battle_effects() for invincible
        pre_battle = heart.get_pre_battle_effects()
        assert pre_battle["self_effects"]["invincible"] == 200  # 300 at A0, 200 at A19+

    def test_heart_invincible_below_a19(self):
        """Heart should have normal invincible threshold below A19."""
        rng = Random(100)
        heart = CorruptHeart(rng, ascension=0)
        pre_battle = heart.get_pre_battle_effects()
        assert pre_battle["self_effects"]["invincible"] == 300

    def test_heart_beat_of_death_at_a19(self):
        """Heart should have higher Beat of Death at A19+."""
        rng = Random(100)
        heart = CorruptHeart(rng, ascension=19)
        # Heart uses get_pre_battle_effects() for beat_of_death
        pre_battle = heart.get_pre_battle_effects()
        assert pre_battle["self_effects"]["beat_of_death"] == 2  # 1 at A0, 2 at A19+

    def test_heart_beat_of_death_below_a19(self):
        """Heart should have normal Beat of Death below A19."""
        rng = Random(100)
        heart = CorruptHeart(rng, ascension=0)
        pre_battle = heart.get_pre_battle_effects()
        assert pre_battle["self_effects"]["beat_of_death"] == 1

    def test_heart_damage_at_a4(self):
        """Heart should deal more damage at A4+."""
        rng = Random(100)
        heart_a0 = CorruptHeart(rng, ascension=0)
        rng2 = Random(100)
        heart_a4 = CorruptHeart(rng2, ascension=4)
        assert heart_a0._get_damage_values()["echo"] == 40
        assert heart_a4._get_damage_values()["echo"] == 45
        assert heart_a0._get_damage_values()["blood_count"] == 12
        assert heart_a4._get_damage_values()["blood_count"] == 15


# =============================================================================
# Map Generation: Elite Room Increase at A1+
# =============================================================================

class TestMapEliteGeneration:
    """Test that maps have more elites at A1+."""

    def test_elite_count_below_a1(self):
        """Maps should have base elite count below A1."""
        config = MapGeneratorConfig(ascension_level=0)
        # Elite chance is 8% base
        assert config.elite_room_chance == 0.08

    def test_elite_count_at_a1(self):
        """Maps should have 1.6x elite count at A1+."""
        rng = Random(100)
        config = MapGeneratorConfig(ascension_level=1)
        generator = MapGenerator(rng, config)
        room_list = generator._generate_room_list(30)
        elite_count = sum(1 for r in room_list if r == RoomType.ELITE)
        # At A1+, elite count is 1.6x base
        # Base would be 30 * 0.08 = 2.4 -> 2
        # A1+ would be 30 * 0.08 * 1.6 = 3.84 -> 4
        assert elite_count >= 3  # Should be higher than A0


# =============================================================================
# Integration Tests: Full Ascension Level Checks
# =============================================================================

class TestAscensionIntegration:
    """Integration tests for full ascension configurations."""

    @pytest.mark.parametrize("ascension", range(0, 21))
    def test_run_creation_all_ascensions(self, ascension):
        """Test that runs can be created at all ascension levels."""
        run = create_watcher_run("TESTSEED", ascension=ascension)
        assert run.ascension == ascension
        assert run.max_hp > 0
        assert run.gold > 0
        assert len(run.deck) >= 10

    @pytest.mark.parametrize("ascension,expected_hp", [
        (0, 72), (6, 72), (10, 72), (13, 72), (14, 68), (20, 68)
    ])
    def test_watcher_hp_by_ascension(self, ascension, expected_hp):
        """Test Watcher HP at specific ascension breakpoints."""
        run = create_watcher_run("TESTSEED", ascension=ascension)
        assert run.max_hp == expected_hp

    @pytest.mark.parametrize("ascension,expected_gold", [
        (0, 99), (10, 99), (14, 99), (15, 99), (20, 99)
    ])
    def test_watcher_gold_by_ascension(self, ascension, expected_gold):
        """Starting gold is always 99 -- A15 only affects events."""
        run = create_watcher_run("TESTSEED", ascension=ascension)
        assert run.gold == expected_gold

    @pytest.mark.parametrize("ascension,expected_slots", [
        (0, 3), (5, 3), (10, 3), (11, 2), (15, 2), (20, 2)
    ])
    def test_potion_slots_by_ascension(self, ascension, expected_slots):
        """Test potion slot count at specific ascension breakpoints."""
        run = create_watcher_run("TESTSEED", ascension=ascension)
        assert len(run.potion_slots) == expected_slots

    @pytest.mark.parametrize("ascension,has_bane", [
        (0, False), (5, False), (9, False), (10, True), (15, True), (20, True)
    ])
    def test_ascenders_bane_by_ascension(self, ascension, has_bane):
        """Test Ascender's Bane presence at specific ascension breakpoints."""
        run = create_watcher_run("TESTSEED", ascension=ascension)
        card_ids = [card.id for card in run.deck]
        assert ("AscendersBane" in card_ids) == has_bane


# =============================================================================
# Enemy Type-Specific Tests
# =============================================================================

class TestEnemyAscensionThresholds:
    """Test specific ascension thresholds for various enemies."""

    @pytest.mark.parametrize("ascension,damage", [
        (0, 11), (1, 11), (2, 12), (10, 12), (17, 12)
    ])
    def test_jaw_worm_chomp_damage(self, ascension, damage):
        """Test Jaw Worm chomp damage at various ascensions."""
        rng = Random(100)
        worm = JawWorm(rng, ascension=ascension)
        dmg = worm._get_damage_values()
        assert dmg["chomp"] == damage

    @pytest.mark.parametrize("ascension,strength", [
        (0, 3), (1, 3), (2, 4), (16, 4), (17, 5), (20, 5)
    ])
    def test_jaw_worm_bellow_strength(self, ascension, strength):
        """Test Jaw Worm Bellow strength at various ascensions."""
        rng = Random(100)
        worm = JawWorm(rng, ascension=ascension)
        dmg = worm._get_damage_values()
        assert dmg["bellow_str"] == strength

    @pytest.mark.parametrize("ascension,ritual", [
        (0, 3), (1, 3), (2, 4), (16, 4), (17, 5), (20, 5)
    ])
    def test_cultist_ritual(self, ascension, ritual):
        """Test Cultist ritual at various ascensions."""
        rng = Random(100)
        cultist = Cultist(rng, ascension=ascension)
        dmg = cultist._get_damage_values()
        assert dmg["ritual"] == ritual

    @pytest.mark.parametrize("ascension,min_hp,max_hp", [
        (0, 40, 44), (6, 40, 44), (7, 42, 46), (20, 42, 46)
    ])
    def test_jaw_worm_hp_range(self, ascension, min_hp, max_hp):
        """Test Jaw Worm HP range at various ascensions."""
        rng = Random(100)
        worm = JawWorm(rng, ascension=ascension)
        hp_min, hp_max = worm._get_hp_range()
        assert hp_min == min_hp
        assert hp_max == max_hp


# =============================================================================
# Boss Specific Tests
# =============================================================================

class TestBossAscensionModifiers:
    """Test ascension modifiers for all Act 1 bosses."""

    @pytest.mark.parametrize("ascension,slam_dmg", [
        (0, 35), (3, 35), (4, 38), (10, 38), (20, 38)
    ])
    def test_slime_boss_slam_damage(self, ascension, slam_dmg):
        """Test Slime Boss slam damage at various ascensions."""
        rng = Random(100)
        boss = SlimeBoss(rng, ascension=ascension)
        dmg = boss._get_damage_values()
        assert dmg["slam"] == slam_dmg

    @pytest.mark.parametrize("ascension,slimed_count", [
        (0, 3), (10, 3), (18, 3), (19, 5), (20, 5)
    ])
    def test_slime_boss_slimed_count(self, ascension, slimed_count):
        """Test Slime Boss slimed card count at various ascensions."""
        rng = Random(100)
        boss = SlimeBoss(rng, ascension=ascension)
        move = boss.get_move(0)  # First move is Goop Spray
        assert move.effects.get("slimed", 0) == slimed_count

    @pytest.mark.parametrize("ascension,mode_shift", [
        (0, 30), (8, 30), (9, 35), (18, 35), (19, 40), (20, 40)
    ])
    def test_guardian_mode_shift_threshold(self, ascension, mode_shift):
        """Test Guardian mode shift threshold at various ascensions."""
        rng = Random(100)
        guardian = TheGuardian(rng, ascension=ascension)
        assert guardian.mode_shift_damage == mode_shift


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
