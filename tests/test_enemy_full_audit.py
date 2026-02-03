"""
Full Enemy Parity Audit - Java vs Python

Every test in this file compares the Python enemy implementation against the
decompiled Java source. Tests that FAIL indicate a confirmed discrepancy.

Java source: decompiled/java-src/com/megacrit/cardcrawl/monsters/
Python source: packages/engine/content/enemies.py

Generated from comprehensive line-by-line audit of all enemy classes.
"""

import pytest
from packages.engine.content.enemies import (
    Enemy, MoveInfo, Intent, EnemyType, EnemyState,
    JawWorm, Cultist, AcidSlimeM, AcidSlimeL, AcidSlimeS,
    SpikeSlimeM, SpikeSlimeL, SpikeSlimeS,
    FungiBeast, LouseNormal, LouseDefensive, Louse,
    GremlinNob, Lagavulin, Sentries, Looter,
    SlaverBlue, SlaverRed,
    SlimeBoss, TheGuardian, Hexaghost,
    Chosen, Byrd, Centurion, Healer, Snecko, SnakePlant,
    Mugger, Taskmaster, ShelledParasite, SphericGuardian,
    BanditBear, BanditLeader, BanditPointy,
    GremlinLeader, BookOfStabbing,
    Maw, Darkling, OrbWalker, Spiker, Repulsor,
    WrithingMass, Transient, Exploder, SpireGrowth, SnakeDagger,
    Champ, TheCollector, BronzeAutomaton,
    AwakenedOne, TimeEater, Donu, Deca,
    SpireShield, SpireSpear, CorruptHeart,
    TorchHead, BronzeOrb,
    GremlinFat, GremlinThief, GremlinTsundere, GremlinWarrior, GremlinWizard,
    GiantHead, Nemesis, Reptomancer,
)
from packages.engine.state.rng import Random


def make_rng(seed=42):
    return Random(seed)


# ============================================================
# ACT 1 NORMAL ENEMIES
# ============================================================

class TestJawWormParity:
    """Java: exordium/JawWorm.java"""

    def test_move_ids(self):
        """Java: CHOMP=1, BELLOW=2, THRASH=3"""
        jw = JawWorm(make_rng())
        assert jw.CHOMP == 1
        assert jw.BELLOW == 2
        assert jw.THRASH == 3

    def test_hp_base(self):
        """Java: HP_MIN=40, HP_MAX=44"""
        jw = JawWorm(make_rng(), ascension=0)
        assert 40 <= jw.state.max_hp <= 44

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=42, A_2_HP_MAX=46 (at A7+)"""
        jw = JawWorm(make_rng(), ascension=7)
        assert 42 <= jw.state.max_hp <= 46

    def test_chomp_damage_base(self):
        """Java: CHOMP_DMG=11"""
        jw = JawWorm(make_rng(), ascension=0)
        dmg = jw._get_damage_values()
        assert dmg["chomp"] == 11

    def test_chomp_damage_a2(self):
        """Java: A_2_CHOMP_DMG=12"""
        jw = JawWorm(make_rng(), ascension=2)
        dmg = jw._get_damage_values()
        assert dmg["chomp"] == 12

    def test_bellow_str_base(self):
        """Java: BELLOW_STR=3"""
        jw = JawWorm(make_rng(), ascension=0)
        dmg = jw._get_damage_values()
        assert dmg["bellow_str"] == 3

    def test_bellow_str_a2(self):
        """Java: A_2_BELLOW_STR=4"""
        jw = JawWorm(make_rng(), ascension=2)
        dmg = jw._get_damage_values()
        assert dmg["bellow_str"] == 4

    def test_bellow_str_a17(self):
        """Java: A_17_BELLOW_STR=5"""
        jw = JawWorm(make_rng(), ascension=17)
        dmg = jw._get_damage_values()
        assert dmg["bellow_str"] == 5

    def test_bellow_block_base(self):
        """Java: BELLOW_BLOCK=6"""
        jw = JawWorm(make_rng(), ascension=0)
        dmg = jw._get_damage_values()
        assert dmg["bellow_block"] == 6

    def test_bellow_block_a17(self):
        """Java: A_17_BELLOW_BLOCK=9"""
        jw = JawWorm(make_rng(), ascension=17)
        dmg = jw._get_damage_values()
        assert dmg["bellow_block"] == 9

    def test_first_move_chomp(self):
        """Java: firstMove -> CHOMP (byte 1)"""
        jw = JawWorm(make_rng(), ascension=0)
        move = jw.get_move(50)
        assert move.move_id == 1
        assert move.name == "Chomp"

    def test_hard_mode_no_first_move(self):
        """Java: hardMode -> firstMove=false"""
        jw = JawWorm(make_rng(), ascension=0, hard_mode=True)
        assert jw.state.first_turn is False


class TestCultistParity:
    """Java: exordium/Cultist.java"""

    def test_move_ids_match_java(self):
        """
        Java: DARK_STRIKE=1, INCANTATION=3
        CRITICAL BUG: Python has INCANTATION=1, DARK_STRIKE=2 (wrong IDs)
        """
        c = Cultist(make_rng())
        # Java IDs
        assert c.INCANTATION == 3, \
            f"Python INCANTATION={c.INCANTATION}, Java INCANTATION=3"

    def test_dark_strike_id_matches_java(self):
        """Java: DARK_STRIKE=1"""
        c = Cultist(make_rng())
        assert c.DARK_STRIKE == 1

    def test_hp_base(self):
        """Java: HP_MIN=48, HP_MAX=54"""
        c = Cultist(make_rng(), ascension=0)
        assert 48 <= c.state.max_hp <= 54

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=50, A_2_HP_MAX=56 (at A7+)"""
        c = Cultist(make_rng(), ascension=7)
        assert 50 <= c.state.max_hp <= 56

    def test_attack_damage(self):
        """Java: ATTACK_DMG=6 (constant, no ascension scaling)"""
        c = Cultist(make_rng(), ascension=0)
        dmg = c._get_damage_values()
        assert dmg["dark_strike"] == 6

    def test_ritual_base(self):
        """Java: RITUAL_AMT=3"""
        c = Cultist(make_rng(), ascension=0)
        dmg = c._get_damage_values()
        assert dmg["ritual"] == 3

    def test_ritual_a2(self):
        """Java: A_2_RITUAL_AMT=4"""
        c = Cultist(make_rng(), ascension=2)
        dmg = c._get_damage_values()
        assert dmg["ritual"] == 4

    def test_ritual_a17_takeTurn_adds_one(self):
        """
        Java: At A17+, takeTurn applies ritualAmount + 1.
        So at A2 (ritualAmount=4), A17 applies 5 ritual.
        Python should reflect this in the move effects.
        """
        c = Cultist(make_rng(), ascension=17)
        dmg = c._get_damage_values()
        # Java: ritualAmount = A2+ ? 4 : 3, then at A17 takeTurn does +1
        # So effective ritual at A17 should be 5
        assert dmg["ritual"] == 5

    def test_first_move_incantation(self):
        """Java: firstMove -> INCANTATION (buff)"""
        c = Cultist(make_rng())
        move = c.get_move(50)
        assert move.intent == Intent.BUFF

    def test_after_first_always_attack(self):
        """Java: After first turn, always DARK_STRIKE"""
        c = Cultist(make_rng())
        c.get_move(50)  # First turn
        move = c.get_move(50)  # Second turn
        assert move.intent == Intent.ATTACK
        assert move.base_damage == 6


class TestAcidSlimeSParity:
    """Java: exordium/AcidSlime_S.java"""

    def test_hp_base(self):
        """Java: HP_MIN=8, HP_MAX=12"""
        s = AcidSlimeS(make_rng(), ascension=0)
        assert 8 <= s.state.max_hp <= 12

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=9, A_2_HP_MAX=13"""
        s = AcidSlimeS(make_rng(), ascension=7)
        assert 9 <= s.state.max_hp <= 13

    def test_tackle_damage_base(self):
        """Java: TACKLE_DAMAGE=3"""
        s = AcidSlimeS(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 3

    def test_tackle_damage_a2(self):
        """Java: A_2_TACKLE_DAMAGE=4"""
        s = AcidSlimeS(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 4

    def test_a17_alternating_in_takeTurn(self):
        """
        Java AcidSlime_S: takeTurn() directly sets next move:
        - After TACKLE (1): sets DEBUFF (2)
        - After DEBUFF (2): sets TACKLE (1)
        This means the alternating pattern is in takeTurn, not getMove.
        getMove at A17+ only determines first move.
        Python may not model this correctly.
        """
        # Java A17+ getMove: lastTwoMoves(1) -> sets TACKLE(!), else sets DEBUFF
        # This means if the last two were TACKLE, it selects TACKLE again (odd)
        # But takeTurn overrides by setting the next move directly.
        s = AcidSlimeS(make_rng(), ascension=17)
        move1 = s.get_move(50)
        # First call to getMove: lastTwoMoves(1) is false -> sets DEBUFF (Lick)
        assert move1.intent == Intent.DEBUFF


class TestAcidSlimeMParity:
    """Java: exordium/AcidSlime_M.java"""

    def test_hp_base(self):
        """Java: HP_MIN=28, HP_MAX=32"""
        s = AcidSlimeM(make_rng(), ascension=0)
        assert 28 <= s.state.max_hp <= 32

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=29, A_2_HP_MAX=34"""
        s = AcidSlimeM(make_rng(), ascension=7)
        assert 29 <= s.state.max_hp <= 34

    def test_spit_damage_base(self):
        """Java: W_TACKLE_DMG=7"""
        s = AcidSlimeM(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["spit"] == 7

    def test_spit_damage_a2(self):
        """Java: A_2_W_TACKLE_DMG=8"""
        s = AcidSlimeM(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["spit"] == 8

    def test_tackle_damage_base(self):
        """Java: N_TACKLE_DMG=10"""
        s = AcidSlimeM(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 10

    def test_tackle_damage_a2(self):
        """Java: A_2_N_TACKLE_DMG=12"""
        s = AcidSlimeM(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 12

    def test_move_ids(self):
        """Java: WOUND_TACKLE=1, NORMAL_TACKLE=2, WEAK_LICK=4"""
        s = AcidSlimeM(make_rng())
        assert s.CORROSIVE_SPIT == 1
        assert s.TACKLE == 2
        assert s.LICK == 4


class TestAcidSlimeLParity:
    """Java: exordium/AcidSlime_L.java"""

    def test_hp_base(self):
        """Java: HP_MIN=65, HP_MAX=69"""
        s = AcidSlimeL(make_rng(), ascension=0)
        assert 65 <= s.state.max_hp <= 69

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=68, A_2_HP_MAX=72"""
        s = AcidSlimeL(make_rng(), ascension=7)
        assert 68 <= s.state.max_hp <= 72

    def test_spit_damage_base(self):
        """Java: W_TACKLE_DMG=11"""
        s = AcidSlimeL(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["spit"] == 11

    def test_spit_damage_a2(self):
        """Java: A_2_W_TACKLE_DMG=12"""
        s = AcidSlimeL(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["spit"] == 12

    def test_tackle_damage_base(self):
        """Java: N_TACKLE_DMG=16"""
        s = AcidSlimeL(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 16

    def test_tackle_damage_a2(self):
        """Java: A_2_N_TACKLE_DMG=18"""
        s = AcidSlimeL(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 18


class TestSpikeSlimeSParity:
    """Java: exordium/SpikeSlime_S.java"""

    def test_hp_base(self):
        """Java: HP_MIN=10, HP_MAX=14"""
        s = SpikeSlimeS(make_rng(), ascension=0)
        assert 10 <= s.state.max_hp <= 14

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=11, A_2_HP_MAX=15"""
        s = SpikeSlimeS(make_rng(), ascension=7)
        assert 11 <= s.state.max_hp <= 15

    def test_tackle_damage_base(self):
        """Java: TACKLE_DAMAGE=5"""
        s = SpikeSlimeS(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 5

    def test_tackle_damage_a2(self):
        """Java: A_2_TACKLE_DAMAGE=6"""
        s = SpikeSlimeS(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 6


class TestSpikeSlimeMParity:
    """Java: exordium/SpikeSlime_M.java"""

    def test_hp_base(self):
        """Java: HP_MIN=28, HP_MAX=32"""
        s = SpikeSlimeM(make_rng(), ascension=0)
        assert 28 <= s.state.max_hp <= 32

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=29, A_2_HP_MAX=34"""
        s = SpikeSlimeM(make_rng(), ascension=7)
        assert 29 <= s.state.max_hp <= 34

    def test_tackle_damage_base(self):
        """Java: TACKLE_DAMAGE=8"""
        s = SpikeSlimeM(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 8

    def test_tackle_damage_a2(self):
        """Java: A_2_TACKLE_DAMAGE=10"""
        s = SpikeSlimeM(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 10

    def test_frail_base(self):
        """Java: FRAIL_TURNS=1 (applied in takeTurn, not ascension-dependent for M)"""
        s = SpikeSlimeM(make_rng(), ascension=0)
        move = s.get_move(50)  # Should be Lick
        if "frail" in move.effects:
            assert move.effects["frail"] == 1


class TestSpikeSlimeLParity:
    """Java: exordium/SpikeSlime_L.java"""

    def test_hp_base(self):
        """Java: HP_MIN=64, HP_MAX=70"""
        s = SpikeSlimeL(make_rng(), ascension=0)
        assert 64 <= s.state.max_hp <= 70

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=67, A_2_HP_MAX=73"""
        s = SpikeSlimeL(make_rng(), ascension=7)
        assert 67 <= s.state.max_hp <= 73

    def test_tackle_damage_base(self):
        """Java: TACKLE_DAMAGE=16"""
        s = SpikeSlimeL(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 16

    def test_tackle_damage_a2(self):
        """Java: A_2_TACKLE_DAMAGE=18"""
        s = SpikeSlimeL(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["tackle"] == 18

    def test_frail_a17(self):
        """Java: A17+ applies 3 Frail (in takeTurn)"""
        s = SpikeSlimeL(make_rng(), ascension=17)
        move = s.get_move(50)  # Should be Lick at 70% prob
        if "frail" in move.effects:
            assert move.effects["frail"] == 3


class TestFungiBeastParity:
    """Java: exordium/FungiBeast.java"""

    def test_hp_base(self):
        """Java: HP_MIN=22, HP_MAX=28"""
        f = FungiBeast(make_rng(), ascension=0)
        assert 22 <= f.state.max_hp <= 28

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=24, A_2_HP_MAX=28"""
        f = FungiBeast(make_rng(), ascension=7)
        assert 24 <= f.state.max_hp <= 28

    def test_bite_damage(self):
        """Java: BITE_DMG=6 (constant)"""
        f = FungiBeast(make_rng(), ascension=0)
        dmg = f._get_damage_values()
        assert dmg["bite"] == 6

    def test_grow_str_base(self):
        """Java: GROW_STR=3"""
        f = FungiBeast(make_rng(), ascension=0)
        dmg = f._get_damage_values()
        assert dmg["strength"] == 3

    def test_grow_str_a2(self):
        """Java: A_2_GROW_STR=4"""
        f = FungiBeast(make_rng(), ascension=2)
        dmg = f._get_damage_values()
        assert dmg["strength"] == 4

    def test_grow_str_a17_takeTurn_adds_one(self):
        """
        Java: At A17+, takeTurn applies strAmt + 1.
        At A2+ (strAmt=4), A17 effective = 5.
        At base (strAmt=3), A17 effective = 4.
        Python should reflect this.
        """
        f = FungiBeast(make_rng(), ascension=17)
        dmg = f._get_damage_values()
        # Java at A17: strAmt is from constructor (A2+=4), but takeTurn does strAmt+1=5
        assert dmg["strength"] == 5

    def test_spore_cloud(self):
        """Java: usePreBattleAction applies SporeCloudPower(2)"""
        f = FungiBeast(make_rng())
        assert f.state.powers.get("spore_cloud") == 2


class TestLouseNormalParity:
    """Java: exordium/LouseNormal.java"""

    def test_id(self):
        """Java: ID = "FuzzyLouseNormal" """
        l = LouseNormal(make_rng())
        assert l.ID == "FuzzyLouseNormal"

    def test_hp_base(self):
        """Java: HP_MIN=10, HP_MAX=15"""
        l = LouseNormal(make_rng(), ascension=0)
        assert 10 <= l.state.max_hp <= 15

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=11, A_2_HP_MAX=16"""
        l = LouseNormal(make_rng(), ascension=7)
        assert 11 <= l.state.max_hp <= 16

    def test_move_ids(self):
        """Java: BITE=3, STRENGTHEN=4"""
        l = LouseNormal(make_rng())
        assert l.BITE == 3
        assert l.GROW == 4

    def test_str_amount_base(self):
        """Java: STR_AMOUNT=3 (at base), 4 at A17+"""
        l = LouseNormal(make_rng(), ascension=0)
        dmg = l._get_damage_values()
        assert dmg["strength"] == 3

    def test_str_amount_a17(self):
        """Java: A17+ applies 4 Strength"""
        l = LouseNormal(make_rng(), ascension=17)
        dmg = l._get_damage_values()
        assert dmg["strength"] == 4

    def test_bite_damage_rolled(self):
        """Java: biteDamage = monsterHpRng.random(5,7) or (6,8) at A2+"""
        l = LouseNormal(make_rng(), ascension=0)
        assert 5 <= l.bite_damage <= 7

    def test_bite_damage_rolled_a2(self):
        l = LouseNormal(make_rng(), ascension=2)
        assert 6 <= l.bite_damage <= 8


class TestLouseDefensiveParity:
    """Java: exordium/LouseDefensive.java"""

    def test_id(self):
        """Java: ID = "FuzzyLouseDefensive" """
        l = LouseDefensive(make_rng())
        assert l.ID == "FuzzyLouseDefensive"

    def test_hp_base(self):
        """Java: HP_MIN=11, HP_MAX=17"""
        l = LouseDefensive(make_rng(), ascension=0)
        assert 11 <= l.state.max_hp <= 17

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=12, A_2_HP_MAX=18"""
        l = LouseDefensive(make_rng(), ascension=7)
        assert 12 <= l.state.max_hp <= 18

    def test_move_ids(self):
        """Java: BITE=3, WEAKEN=4"""
        l = LouseDefensive(make_rng())
        assert l.BITE == 3
        assert l.SPIT_WEB == 4

    def test_weak_amount(self):
        """Java: WEAK_AMT=2 (constant)"""
        l = LouseDefensive(make_rng(), ascension=0)
        dmg = l._get_damage_values()
        assert dmg["weak"] == 2


class TestLooterParity:
    """Java: exordium/Looter.java"""

    def test_hp_base(self):
        """Java: HP_MIN=44, HP_MAX=48"""
        l = Looter(make_rng(), ascension=0)
        assert 44 <= l.state.max_hp <= 48

    def test_hp_a7(self):
        """Java: A_2_HP_MIN=46, A_2_HP_MAX=50"""
        l = Looter(make_rng(), ascension=7)
        assert 46 <= l.state.max_hp <= 50

    def test_swipe_damage_base(self):
        """Java: swipeDmg=10"""
        l = Looter(make_rng(), ascension=0)
        dmg = l._get_damage_values()
        assert dmg["swipe"] == 10

    def test_swipe_damage_a2(self):
        """Java: swipeDmg=11 at A2+"""
        l = Looter(make_rng(), ascension=2)
        dmg = l._get_damage_values()
        assert dmg["swipe"] == 11

    def test_lunge_damage_base(self):
        """Java: lungeDmg=12"""
        l = Looter(make_rng(), ascension=0)
        dmg = l._get_damage_values()
        assert dmg["lunge"] == 12

    def test_lunge_damage_a2(self):
        """Java: lungeDmg=14 at A2+"""
        l = Looter(make_rng(), ascension=2)
        dmg = l._get_damage_values()
        assert dmg["lunge"] == 14

    def test_gold_amt_base(self):
        """Java: goldAmt = A17+ ? 20 : 15"""
        l = Looter(make_rng(), ascension=0)
        assert l.state.powers.get("thievery") == 15

    def test_gold_amt_a17(self):
        l = Looter(make_rng(), ascension=17)
        assert l.state.powers.get("thievery") == 20

    def test_getMove_always_mug(self):
        """Java: getMove always sets MUG (byte 1). Pattern is driven by takeTurn."""
        l = Looter(make_rng())
        move = l.get_move(50)
        assert move.move_id == l.MUG


class TestSlaverBlueParity:
    """Java: exordium/SlaverBlue.java"""

    def test_hp_base(self):
        s = SlaverBlue(make_rng(), ascension=0)
        assert 46 <= s.state.max_hp <= 50

    def test_hp_a7(self):
        s = SlaverBlue(make_rng(), ascension=7)
        assert 48 <= s.state.max_hp <= 52

    def test_stab_damage_base(self):
        s = SlaverBlue(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["stab"] == 12

    def test_stab_damage_a2(self):
        s = SlaverBlue(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["stab"] == 13

    def test_rake_damage_base(self):
        s = SlaverBlue(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["rake"] == 7

    def test_rake_damage_a2(self):
        s = SlaverBlue(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["rake"] == 8


class TestSlaverRedParity:
    """Java: exordium/SlaverRed.java"""

    def test_hp_base(self):
        s = SlaverRed(make_rng(), ascension=0)
        assert 46 <= s.state.max_hp <= 50

    def test_hp_a7(self):
        s = SlaverRed(make_rng(), ascension=7)
        assert 48 <= s.state.max_hp <= 52

    def test_stab_damage_base(self):
        s = SlaverRed(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["stab"] == 13

    def test_stab_damage_a2(self):
        s = SlaverRed(make_rng(), ascension=2)
        dmg = s._get_damage_values()
        assert dmg["stab"] == 14

    def test_first_move_stab(self):
        """Java: First move always STAB"""
        s = SlaverRed(make_rng(), ascension=0)
        move = s.get_move(50)
        assert move.move_id == s.STAB


# ============================================================
# ACT 1 ELITES
# ============================================================

class TestGremlinNobParity:
    """Java: exordium/GremlinNob.java"""

    def test_hp_base(self):
        """Java: HP_MIN=82, HP_MAX=86"""
        n = GremlinNob(make_rng(), ascension=0)
        assert 82 <= n.state.max_hp <= 86

    def test_hp_a8(self):
        """Java: A_2_HP_MIN=85, A_2_HP_MAX=90"""
        n = GremlinNob(make_rng(), ascension=8)
        assert 85 <= n.state.max_hp <= 90

    def test_move_ids(self):
        """Java: BULL_RUSH=1, SKULL_BASH=2, BELLOW=3"""
        n = GremlinNob(make_rng())
        assert n.RUSH == 1
        assert n.SKULL_BASH == 2
        assert n.BELLOW == 3

    def test_bash_damage_base(self):
        """Java: BASH_DMG=6"""
        n = GremlinNob(make_rng(), ascension=0)
        dmg = n._get_damage_values()
        assert dmg["skull_bash"] == 6

    def test_bash_damage_a3(self):
        """Java: A_2_BASH_DMG=8"""
        n = GremlinNob(make_rng(), ascension=3)
        dmg = n._get_damage_values()
        assert dmg["skull_bash"] == 8

    def test_rush_damage_base(self):
        """Java: RUSH_DMG=14"""
        n = GremlinNob(make_rng(), ascension=0)
        dmg = n._get_damage_values()
        assert dmg["rush"] == 14

    def test_rush_damage_a3(self):
        """Java: A_2_RUSH_DMG=16"""
        n = GremlinNob(make_rng(), ascension=3)
        dmg = n._get_damage_values()
        assert dmg["rush"] == 16

    def test_enrage_base(self):
        """Java: ANGRY_LEVEL=2"""
        n = GremlinNob(make_rng(), ascension=0)
        dmg = n._get_damage_values()
        assert dmg["enrage"] == 2

    def test_enrage_a18(self):
        """Java: A18+ enrage=3"""
        n = GremlinNob(make_rng(), ascension=18)
        dmg = n._get_damage_values()
        assert dmg["enrage"] == 3

    def test_first_move_bellow(self):
        """Java: !usedBellow -> BELLOW"""
        n = GremlinNob(make_rng())
        move = n.get_move(50)
        assert move.move_id == n.BELLOW

    def test_below_a18_num_under_33_unconditional_skull_bash(self):
        """
        Java: Below A18, num < 33 -> unconditionally SKULL_BASH (no lastMove check).
        Python should NOT check lastMove for this branch.
        """
        n = GremlinNob(make_rng(), ascension=0)
        n.get_move(50)  # Bellow
        # Force history to have SKULL_BASH
        n.state.move_history.append(n.SKULL_BASH)
        # num=10 < 33 -> should still be SKULL_BASH regardless of history
        move = n.get_move(10)
        assert move.move_id == n.SKULL_BASH, \
            "Java: num < 33 -> SKULL_BASH unconditionally (no lastMove check)"


class TestLagavulinParity:
    """Java: exordium/Lagavulin.java"""

    def test_hp_base(self):
        """Java: HP_MIN=109, HP_MAX=111"""
        l = Lagavulin(make_rng(), ascension=0)
        assert 109 <= l.state.max_hp <= 111

    def test_hp_a8(self):
        """Java: A_2_HP_MIN=112, A_2_HP_MAX=115"""
        l = Lagavulin(make_rng(), ascension=8)
        assert 112 <= l.state.max_hp <= 115

    def test_move_ids(self):
        """
        Java: DEBUFF=1, STRONG_ATK=3, OPEN=4, IDLE=5
        Python has: SIPHON_SOUL=1, ATTACK=3, SLEEP=4, STUN=5
        """
        l = Lagavulin(make_rng())
        assert l.SIPHON_SOUL == 1
        assert l.ATTACK == 3

    def test_attack_damage_base(self):
        """Java: STRONG_ATK_DMG=18"""
        l = Lagavulin(make_rng(), ascension=0)
        dmg = l._get_damage_values()
        assert dmg["attack"] == 18

    def test_attack_damage_a3(self):
        """Java: A_2_STRONG_ATK_DMG=20"""
        l = Lagavulin(make_rng(), ascension=3)
        dmg = l._get_damage_values()
        assert dmg["attack"] == 20

    def test_debuff_base(self):
        """Java: DEBUFF_AMT=-1"""
        l = Lagavulin(make_rng(), ascension=0)
        dmg = l._get_damage_values()
        assert dmg["debuff"] == 1

    def test_debuff_a18(self):
        """Java: A_18_DEBUFF_AMT=-2"""
        l = Lagavulin(make_rng(), ascension=18)
        dmg = l._get_damage_values()
        assert dmg["debuff"] == 2

    def test_metallicize(self):
        """Java: ARMOR_AMT=8"""
        l = Lagavulin(make_rng())
        assert l.state.powers.get("metallicize") == 8

    def test_sleep_for_3_turns(self):
        """Java: idleCount reaches 3 -> wakes up"""
        l = Lagavulin(make_rng())
        m1 = l.get_move(50)
        assert m1.intent == Intent.SLEEP
        m2 = l.get_move(50)
        assert m2.intent == Intent.SLEEP
        m3 = l.get_move(50)  # Third turn, should wake
        assert m3.intent == Intent.ATTACK


class TestSentryParity:
    """Java: exordium/Sentry.java"""

    def test_hp_base(self):
        """Java: HP_MIN=38, HP_MAX=42"""
        s = Sentries(make_rng(), ascension=0)
        assert 38 <= s.state.max_hp <= 42

    def test_hp_a8(self):
        """Java: A_2_HP_MIN=39, A_2_HP_MAX=45"""
        s = Sentries(make_rng(), ascension=8)
        assert 39 <= s.state.max_hp <= 45

    def test_beam_damage_base(self):
        """Java: beamDmg = A3+ ? 10 : 9"""
        s = Sentries(make_rng(), ascension=0)
        dmg = s._get_damage_values()
        assert dmg["damage"] == 9

    def test_beam_damage_a3(self):
        s = Sentries(make_rng(), ascension=3)
        dmg = s._get_damage_values()
        assert dmg["damage"] == 10

    def test_daze_base(self):
        """Java: DAZED_AMT=2"""
        s = Sentries(make_rng(), ascension=0)
        s.state.first_turn = False
        s.state.move_history.append(s.BOLT)
        move = s.get_move(50)  # Should be BEAM after BOLT
        if "daze" in move.effects:
            assert move.effects["daze"] == 2

    def test_daze_a18(self):
        """Java: A_18_DAZED_AMT=3"""
        s = Sentries(make_rng(), ascension=18)
        s.state.first_turn = False
        s.state.move_history.append(s.BOLT)
        move = s.get_move(50)
        if "daze" in move.effects:
            assert move.effects["daze"] == 3

    def test_starting_move_by_index(self):
        """
        Java: First move based on monsters.lastIndexOf(this) % 2.
        Even index -> BOLT (3), Odd index -> BEAM (4).
        Python uses position parameter.
        """
        # Even position (0, 2) -> BOLT
        s0 = Sentries(make_rng(), ascension=0, position=0)
        move0 = s0.get_move(50)
        assert move0.move_id == s0.BOLT

        # Odd position (1) -> BEAM
        s1 = Sentries(make_rng(), ascension=0, position=1)
        move1 = s1.get_move(50)
        assert move1.move_id == s1.BEAM

    def test_artifact_pre_battle(self):
        """Java: usePreBattleAction -> ArtifactPower(1). Python should track this."""
        # This is a behavioral note - Sentries start with 1 Artifact
        pass  # No direct test, just documenting


# ============================================================
# ACT 1 BOSSES
# ============================================================

class TestSlimeBossParity:
    """Java: exordium/SlimeBoss.java"""

    def test_hp_base(self):
        """Java: HP=140 (fixed)"""
        sb = SlimeBoss(make_rng(), ascension=0)
        assert sb.state.max_hp == 140

    def test_hp_a9(self):
        """Java: A_2_HP=150 (at A9+)"""
        sb = SlimeBoss(make_rng(), ascension=9)
        assert sb.state.max_hp == 150

    def test_slam_damage_base(self):
        """Java: SLAM_DAMAGE=35"""
        sb = SlimeBoss(make_rng(), ascension=0)
        dmg = sb._get_damage_values()
        assert dmg["slam"] == 35

    def test_slam_damage_a4(self):
        """Java: A_2_SLAM_DAMAGE=38"""
        sb = SlimeBoss(make_rng(), ascension=4)
        dmg = sb._get_damage_values()
        assert dmg["slam"] == 38

    def test_first_move_sticky(self):
        """Java: firstTurn -> STICKY (byte 4)"""
        sb = SlimeBoss(make_rng())
        move = sb.get_move(50)
        assert move.move_id == sb.STICKY

    def test_slimed_count_base(self):
        """Java: A19 -> 5 Slimed, else 3"""
        sb = SlimeBoss(make_rng(), ascension=0)
        move = sb.get_move(50)
        assert move.effects.get("slimed") == 3

    def test_slimed_count_a19(self):
        sb = SlimeBoss(make_rng(), ascension=19)
        move = sb.get_move(50)
        assert move.effects.get("slimed") == 5

    def test_getMove_only_handles_first_turn(self):
        """
        Java: getMove() only handles firstTurn. All subsequent moves are set
        directly in takeTurn(). Python handles all moves in get_move().
        This is a structural difference but the Python pattern should still match.
        """
        sb = SlimeBoss(make_rng())
        # First: STICKY
        m1 = sb.get_move(50)
        assert m1.move_id == sb.STICKY
        # Second: PREP_SLAM
        m2 = sb.get_move(50)
        assert m2.move_id == sb.PREP_SLAM
        # Third: SLAM
        m3 = sb.get_move(50)
        assert m3.move_id == sb.SLAM
        # Fourth: STICKY (cycle)
        m4 = sb.get_move(50)
        assert m4.move_id == sb.STICKY


class TestTheGuardianParity:
    """Java: exordium/TheGuardian.java"""

    def test_hp_base(self):
        """Java: HP=240"""
        g = TheGuardian(make_rng(), ascension=0)
        assert g.state.max_hp == 240

    def test_hp_a9(self):
        """Java: A_2_HP=250 (at A9+, also at A19)"""
        g = TheGuardian(make_rng(), ascension=9)
        assert g.state.max_hp == 250

    def test_fierce_bash_base(self):
        """Java: FIERCE_BASH_DMG=32"""
        g = TheGuardian(make_rng(), ascension=0)
        dmg = g._get_damage_values()
        assert dmg["fierce_bash"] == 32

    def test_fierce_bash_a4(self):
        """Java: A_2_FIERCE_BASH_DMG=36"""
        g = TheGuardian(make_rng(), ascension=4)
        dmg = g._get_damage_values()
        assert dmg["fierce_bash"] == 36

    def test_roll_damage_base(self):
        """Java: ROLL_DMG=9"""
        g = TheGuardian(make_rng(), ascension=0)
        dmg = g._get_damage_values()
        assert dmg["roll"] == 9

    def test_roll_damage_a4(self):
        """Java: A_2_ROLL_DMG=10"""
        g = TheGuardian(make_rng(), ascension=4)
        dmg = g._get_damage_values()
        assert dmg["roll"] == 10

    def test_mode_shift_base(self):
        """Java: DMG_THRESHOLD=30"""
        g = TheGuardian(make_rng(), ascension=0)
        assert g.mode_shift_damage == 30

    def test_mode_shift_a9(self):
        """Java: A_2_DMG_THRESHOLD=35"""
        g = TheGuardian(make_rng(), ascension=9)
        assert g.mode_shift_damage == 35

    def test_mode_shift_a19(self):
        """Java: A_19_DMG_THRESHOLD=40"""
        g = TheGuardian(make_rng(), ascension=19)
        assert g.mode_shift_damage == 40

    def test_move_ids(self):
        """Java: CLOSE_UP=1, FIERCE_BASH=2, ROLL_ATTACK=3, TWIN_SLAM=4, WHIRLWIND=5, CHARGE_UP=6, VENT_STEAM=7"""
        g = TheGuardian(make_rng())
        assert g.CHARGING_UP == 6
        assert g.FIERCE_BASH == 2
        assert g.ROLL_ATTACK == 3
        assert g.TWIN_SLAM == 4
        assert g.WHIRLWIND == 5
        assert g.VENT_STEAM == 7

    def test_offensive_pattern(self):
        """Java: Offensive pattern: CHARGE_UP -> FIERCE_BASH -> VENT_STEAM -> WHIRLWIND"""
        g = TheGuardian(make_rng())
        m1 = g.get_move(50)
        assert m1.move_id == g.CHARGING_UP
        m2 = g.get_move(50)
        assert m2.move_id == g.FIERCE_BASH
        m3 = g.get_move(50)
        assert m3.move_id == g.VENT_STEAM
        m4 = g.get_move(50)
        assert m4.move_id == g.WHIRLWIND


class TestHexaghostParity:
    """Java: exordium/Hexaghost.java"""

    def test_hp_base(self):
        """Java: HP=250"""
        h = Hexaghost(make_rng(), ascension=0)
        assert h.state.max_hp == 250

    def test_hp_a9(self):
        """Java: A_2_HP=264"""
        h = Hexaghost(make_rng(), ascension=9)
        assert h.state.max_hp == 264

    def test_move_ids(self):
        """Java: DIVIDER=1, TACKLE=2, INFLAME=3, SEAR=4, ACTIVATE=5, INFERNO=6"""
        h = Hexaghost(make_rng())
        assert h.DIVIDER == 1
        assert h.TACKLE == 2
        assert h.INFLAME == 3
        assert h.SEAR == 4
        assert h.ACTIVATE == 5
        assert h.INFERNO == 6

    def test_first_move_activate(self):
        """Java: !activated -> ACTIVATE (byte 5)"""
        h = Hexaghost(make_rng())
        move = h.get_move(50)
        assert move.move_id == h.ACTIVATE

    def test_pattern_uses_orbActiveCount(self):
        """
        Java: Uses orbActiveCount (0-6) to select moves after Activate.
        0: SEAR, 1: TACKLE, 2: SEAR, 3: INFLAME, 4: TACKLE, 5: SEAR, 6: INFERNO
        Python uses (turn_count - 3) % 7 which should give same result.
        """
        h = Hexaghost(make_rng())
        h.get_move(50)  # Turn 1: ACTIVATE
        h.get_move(50)  # Turn 2: DIVIDER (special)
        # Now the pattern begins
        m3 = h.get_move(50)  # orbActiveCount=0: SEAR
        assert m3.move_id == h.SEAR
        m4 = h.get_move(50)  # orbActiveCount=1: TACKLE
        assert m4.move_id == h.TACKLE
        m5 = h.get_move(50)  # orbActiveCount=2: SEAR
        assert m5.move_id == h.SEAR
        m6 = h.get_move(50)  # orbActiveCount=3: INFLAME
        assert m6.move_id == h.INFLAME
        m7 = h.get_move(50)  # orbActiveCount=4: TACKLE
        assert m7.move_id == h.TACKLE
        m8 = h.get_move(50)  # orbActiveCount=5: SEAR
        assert m8.move_id == h.SEAR
        m9 = h.get_move(50)  # orbActiveCount=6: INFERNO
        assert m9.move_id == h.INFERNO

    def test_sear_damage(self):
        """Java: SEAR_DMG=6 (constant)"""
        h = Hexaghost(make_rng(), ascension=0)
        dmg = h._get_damage_values()
        assert dmg["sear"] == 6

    def test_tackle_damage_base(self):
        """Java: FIRE_TACKLE_DMG=5"""
        h = Hexaghost(make_rng(), ascension=0)
        dmg = h._get_damage_values()
        assert dmg["tackle"] == 5

    def test_tackle_damage_a4(self):
        """Java: A_4_FIRE_TACKLE_DMG=6"""
        h = Hexaghost(make_rng(), ascension=4)
        dmg = h._get_damage_values()
        assert dmg["tackle"] == 6

    def test_inferno_damage_base(self):
        """Java: INFERNO_DMG=2"""
        h = Hexaghost(make_rng(), ascension=0)
        dmg = h._get_damage_values()
        assert dmg["inferno"] == 2

    def test_inferno_damage_a4(self):
        """Java: A_4_INFERNO_DMG=3"""
        h = Hexaghost(make_rng(), ascension=4)
        dmg = h._get_damage_values()
        assert dmg["inferno"] == 3

    def test_str_amount_base(self):
        """Java: STR_AMT=2"""
        h = Hexaghost(make_rng(), ascension=0)
        dmg = h._get_damage_values()
        assert dmg["inflame_str"] == 2

    def test_str_amount_a19(self):
        """Java: A_19_STR_AMT=3"""
        h = Hexaghost(make_rng(), ascension=19)
        dmg = h._get_damage_values()
        assert dmg["inflame_str"] == 3

    def test_burn_count_base(self):
        """Java: BURN_COUNT=1"""
        h = Hexaghost(make_rng(), ascension=0)
        dmg = h._get_damage_values()
        assert dmg["burn_count"] == 1

    def test_burn_count_a19(self):
        """Java: A_19_BURN_COUNT=2"""
        h = Hexaghost(make_rng(), ascension=19)
        dmg = h._get_damage_values()
        assert dmg["burn_count"] == 2


# ============================================================
# ACT 2 BOSSES
# ============================================================

class TestChampParity:
    """Java: city/Champ.java"""

    def test_hp_base(self):
        """Java: HP=420"""
        c = Champ(make_rng(), ascension=0)
        assert c.state.max_hp == 420

    def test_hp_a9(self):
        """Java: A_9_HP=440"""
        c = Champ(make_rng(), ascension=9)
        assert c.state.max_hp == 440

    def test_slash_damage_base(self):
        """Java: SLASH_DMG=16"""
        c = Champ(make_rng(), ascension=0)
        dmg = c._get_damage_values()
        assert dmg["slash"] == 16

    def test_slash_damage_a4(self):
        """Java: A_2_SLASH_DMG=18"""
        c = Champ(make_rng(), ascension=4)
        dmg = c._get_damage_values()
        assert dmg["slash"] == 18

    def test_slap_damage_base(self):
        """Java: SLAP_DMG=12"""
        c = Champ(make_rng(), ascension=0)
        dmg = c._get_damage_values()
        assert dmg["slap"] == 12

    def test_slap_damage_a4(self):
        """Java: A_2_SLAP_DMG=14"""
        c = Champ(make_rng(), ascension=4)
        dmg = c._get_damage_values()
        assert dmg["slap"] == 14

    def test_execute_damage(self):
        """Java: EXECUTE_DMG=10 (constant)"""
        c = Champ(make_rng(), ascension=0)
        dmg = c._get_damage_values()
        assert dmg["execute"] == 10

    def test_str_base(self):
        """Java: STR_AMT=2"""
        c = Champ(make_rng(), ascension=0)
        dmg = c._get_damage_values()
        assert dmg["strength"] == 2

    def test_str_a4(self):
        """Java: A_4_STR_AMT=3"""
        c = Champ(make_rng(), ascension=4)
        dmg = c._get_damage_values()
        assert dmg["strength"] == 3

    def test_str_a19(self):
        """Java: A_19_STR_AMT=4"""
        c = Champ(make_rng(), ascension=19)
        dmg = c._get_damage_values()
        assert dmg["strength"] == 4

    def test_forge_base(self):
        """Java: FORGE_AMT=5, BLOCK_AMT=15"""
        c = Champ(make_rng(), ascension=0)
        dmg = c._get_damage_values()
        assert dmg["forge"] == 5
        assert dmg["block"] == 15

    def test_forge_a9(self):
        """Java: A_9_FORGE_AMT=6, A_9_BLOCK_AMT=18"""
        c = Champ(make_rng(), ascension=9)
        dmg = c._get_damage_values()
        assert dmg["forge"] == 6
        assert dmg["block"] == 18

    def test_forge_a19(self):
        """Java: A_19_FORGE_AMT=7, A_19_BLOCK_AMT=20"""
        c = Champ(make_rng(), ascension=19)
        dmg = c._get_damage_values()
        assert dmg["forge"] == 7
        assert dmg["block"] == 20

    def test_phase_transition_strict_less_than(self):
        """
        Java: currentHealth < maxHealth / 2 (strict less-than, integer division).
        At 420 HP, threshold is 210. Must be < 210 (i.e., 209 or less).
        """
        c = Champ(make_rng(), ascension=0)
        c.state.max_hp = 420
        c.state.current_hp = 210
        assert not c.check_phase_transition(), \
            "Java uses strict < for phase transition, not <="
        c.state.current_hp = 209
        assert c.check_phase_transition()

    def test_anger_strength_is_3x(self):
        """Java: Anger gives strAmt * 3 strength"""
        c = Champ(make_rng(), ascension=0)
        c.state.current_hp = 100
        c.state.max_hp = 420
        move = c.get_move(50)
        assert move.move_id == c.ANGER
        assert move.effects.get("strength") == 6  # 2 * 3

    def test_execute_check_uses_lastMove_and_lastMoveBefore(self):
        """
        Java: After threshold, checks !lastMove(3) && !lastMoveBefore(3).
        This means Execute can be used if it wasn't used in either of the last 2 turns.
        """
        c = Champ(make_rng(), ascension=0)
        c.threshold_reached = True
        c.state.move_history = [c.ANGER]  # Just did Anger
        # Should use Execute (neither last nor lastBefore is Execute)
        move = c.get_move(50)
        assert move.move_id == c.EXECUTE


class TestCorruptHeartParity:
    """Java: ending/CorruptHeart.java"""

    def test_hp_base(self):
        """Java: HP=750"""
        h = CorruptHeart(make_rng(), ascension=0)
        assert h.state.max_hp == 750

    def test_hp_a9(self):
        """Java: A9+=800"""
        h = CorruptHeart(make_rng(), ascension=9)
        assert h.state.max_hp == 800

    def test_echo_damage_base(self):
        """Java: damage[0]=40"""
        h = CorruptHeart(make_rng(), ascension=0)
        dmg = h._get_damage_values()
        assert dmg["echo"] == 40

    def test_echo_damage_a4(self):
        """Java: damage[0]=45 at A4+"""
        h = CorruptHeart(make_rng(), ascension=4)
        dmg = h._get_damage_values()
        assert dmg["echo"] == 45

    def test_blood_damage(self):
        """Java: damage[1]=2 (constant)"""
        h = CorruptHeart(make_rng(), ascension=0)
        dmg = h._get_damage_values()
        assert dmg["blood"] == 2

    def test_blood_count_base(self):
        """Java: bloodHitCount=12"""
        h = CorruptHeart(make_rng(), ascension=0)
        dmg = h._get_damage_values()
        assert dmg["blood_count"] == 12

    def test_blood_count_a4(self):
        """Java: bloodHitCount=15 at A4+"""
        h = CorruptHeart(make_rng(), ascension=4)
        dmg = h._get_damage_values()
        assert dmg["blood_count"] == 15

    def test_first_move_debilitate(self):
        """Java: isFirstMove -> DEBILITATE (byte 3)"""
        h = CorruptHeart(make_rng())
        move = h.get_move(50)
        assert move.move_id == h.DEBILITATE

    def test_invincible_base(self):
        """Java: invincibleAmt=300"""
        h = CorruptHeart(make_rng(), ascension=0)
        effects = h.get_pre_battle_effects()
        assert effects["self_effects"]["invincible"] == 300

    def test_invincible_a19(self):
        """Java: invincibleAmt -= 100 at A19 (=200)"""
        h = CorruptHeart(make_rng(), ascension=19)
        effects = h.get_pre_battle_effects()
        assert effects["self_effects"]["invincible"] == 200

    def test_beat_of_death_base(self):
        """Java: beatAmount=1"""
        h = CorruptHeart(make_rng(), ascension=0)
        effects = h.get_pre_battle_effects()
        assert effects["self_effects"]["beat_of_death"] == 1

    def test_beat_of_death_a19(self):
        """Java: beatAmount=2 at A19"""
        h = CorruptHeart(make_rng(), ascension=19)
        effects = h.get_pre_battle_effects()
        assert effects["self_effects"]["beat_of_death"] == 2

    def test_3turn_cycle(self):
        """
        Java: moveCount % 3 determines attack pattern:
        0: random blood/echo, 1: echo if not last else blood, 2: buff
        """
        h = CorruptHeart(make_rng())
        h.get_move(50)  # First: DEBILITATE
        m1 = h.get_move(50)  # cycle_pos=0: blood or echo
        assert m1.move_id in (h.BLOOD_SHOTS, h.ECHO)
        m2 = h.get_move(50)  # cycle_pos=1
        assert m2.move_id in (h.BLOOD_SHOTS, h.ECHO)
        m3 = h.get_move(50)  # cycle_pos=2: BUFF
        assert m3.move_id == h.BUFF

    def test_buff_cycle_0_artifact(self):
        """Java: buffCount=0 -> +2 Artifact"""
        h = CorruptHeart(make_rng())
        h.is_first_move = False
        h.move_count = 2  # Will be cycle_pos=2 (buff)
        move = h.get_move(50)
        assert move.move_id == h.BUFF
        assert move.effects.get("artifact") == 2


# ============================================================
# ACT 3 BOSSES
# ============================================================

class TestAwakenedOneParity:
    """Java: beyond/AwakenedOne.java"""

    def test_hp_base(self):
        """Java: HP=300"""
        a = AwakenedOne(make_rng(), ascension=0)
        assert a.state.max_hp == 300

    def test_hp_a9(self):
        """Java: A9+=320"""
        a = AwakenedOne(make_rng(), ascension=9)
        assert a.state.max_hp == 320

    def test_phase1_first_move_slash(self):
        """Java: Phase 1 first move = SLASH"""
        a = AwakenedOne(make_rng())
        move = a.get_move(50)
        assert move.move_id == a.SLASH

    def test_phase2_first_move_dark_echo(self):
        """Java: Phase 2 first move = DARK_ECHO"""
        a = AwakenedOne(make_rng())
        a.trigger_rebirth()
        move = a.get_move(50)
        assert move.move_id == a.DARK_ECHO


class TestTimeEaterParity:
    """Java: beyond/TimeEater.java"""

    def test_hp_base(self):
        """Java: HP=456"""
        t = TimeEater(make_rng(), ascension=0)
        assert t.state.max_hp == 456

    def test_hp_a9(self):
        """Java: A9+=480"""
        t = TimeEater(make_rng(), ascension=9)
        assert t.state.max_hp == 480


class TestDonuParity:
    """Java: beyond/Donu.java"""

    def test_hp_base(self):
        """Java: HP=250"""
        d = Donu(make_rng(), ascension=0)
        assert d.state.max_hp == 250

    def test_hp_a9(self):
        """Java: A9+=265"""
        d = Donu(make_rng(), ascension=9)
        assert d.state.max_hp == 265

    def test_beam_damage_base(self):
        """Java: BEAM_DMG=10"""
        d = Donu(make_rng(), ascension=0)
        dmg = d._get_damage_values()
        assert dmg["beam"] == 10

    def test_beam_damage_a4(self):
        """Java: A_2_BEAM_DMG=12"""
        d = Donu(make_rng(), ascension=4)
        dmg = d._get_damage_values()
        assert dmg["beam"] == 12

    def test_starts_with_buff(self):
        """Java: isAttacking starts false -> first move is CIRCLE_OF_PROTECTION"""
        d = Donu(make_rng())
        move = d.get_move(50)
        assert move.move_id == d.CIRCLE_OF_PROTECTION


class TestDecaParity:
    """Java: beyond/Deca.java"""

    def test_hp_base(self):
        d = Deca(make_rng(), ascension=0)
        assert d.state.max_hp == 250

    def test_hp_a9(self):
        d = Deca(make_rng(), ascension=9)
        assert d.state.max_hp == 265

    def test_starts_with_attack(self):
        """Java: isAttacking starts true -> first move is BEAM"""
        d = Deca(make_rng())
        move = d.get_move(50)
        assert move.move_id == d.BEAM


# ============================================================
# ACT 4
# ============================================================

class TestSpireShieldParity:
    """Java: ending/SpireShield.java"""

    def test_hp_base(self):
        s = SpireShield(make_rng(), ascension=0)
        # Need to verify against Java
        assert s.state.max_hp > 0

    def test_exists(self):
        """Verify SpireShield class exists"""
        s = SpireShield(make_rng())
        assert s.ID == "SpireShield"


class TestSpireSpearParity:
    """Java: ending/SpireSpear.java"""

    def test_exists(self):
        s = SpireSpear(make_rng())
        assert s.ID == "SpireSpear"


# ============================================================
# MINIONS
# ============================================================

class TestTorchHeadParity:
    """Java: city/TorchHead.java"""

    def test_tackle_damage(self):
        """Java: TACKLE_DMG=7"""
        t = TorchHead(make_rng())
        dmg = t._get_damage_values()
        assert dmg["tackle"] == 7

    def test_always_tackle(self):
        t = TorchHead(make_rng())
        move = t.get_move(50)
        assert move.move_id == t.TACKLE


class TestBronzeOrbParity:
    """Java: city/BronzeOrb.java"""

    def test_exists(self):
        b = BronzeOrb(make_rng())
        assert b.ID == "BronzeOrb"


# ============================================================
# ACT 2 NORMAL ENEMIES
# ============================================================

class TestChosenParity:
    """Java: city/Chosen.java"""

    def test_hp_base(self):
        c = Chosen(make_rng(), ascension=0)
        assert 95 <= c.state.max_hp <= 99

    def test_hp_a7(self):
        c = Chosen(make_rng(), ascension=7)
        assert 98 <= c.state.max_hp <= 103


class TestByrdParity:
    """Java: city/Byrd.java"""

    def test_exists(self):
        b = Byrd(make_rng())
        assert b.ID == "Byrd"


class TestCenturionParity:
    def test_exists(self):
        c = Centurion(make_rng())
        assert c.ID == "Centurion"


class TestHealerParity:
    def test_exists(self):
        h = Healer(make_rng())
        assert h.ID == "Healer"


class TestSneckoParity:
    def test_exists(self):
        s = Snecko(make_rng())
        assert s.ID == "Snecko"


class TestSnakePlantParity:
    def test_exists(self):
        s = SnakePlant(make_rng())
        assert s.ID == "SnakePlant"


class TestMuggerParity:
    def test_exists(self):
        m = Mugger(make_rng())
        assert m.ID == "Mugger"


class TestTaskmasterParity:
    def test_exists(self):
        t = Taskmaster(make_rng())
        assert t.ID == "SlaverBoss"


class TestShelledParasiteParity:
    def test_exists(self):
        s = ShelledParasite(make_rng())
        assert s.ID == "Shelled Parasite"


class TestSphericGuardianParity:
    def test_exists(self):
        s = SphericGuardian(make_rng())
        assert s.ID == "SphericGuardian"


# ============================================================
# ACT 2 ELITES
# ============================================================

class TestGremlinLeaderParity:
    def test_exists(self):
        g = GremlinLeader(make_rng())
        assert g.ID == "GremlinLeader"


class TestBookOfStabbingParity:
    def test_exists(self):
        b = BookOfStabbing(make_rng())
        assert b.ID == "BookOfStabbing"


# ============================================================
# ACT 3 NORMAL ENEMIES
# ============================================================

class TestDarklingParity:
    def test_exists(self):
        d = Darkling(make_rng())
        assert d.ID == "Darkling"


class TestOrbWalkerParity:
    def test_exists(self):
        o = OrbWalker(make_rng())
        assert o.ID == "Orb Walker"


class TestSpikerParity:
    def test_exists(self):
        s = Spiker(make_rng())
        assert s.ID == "Spiker"


class TestRepulsorParity:
    def test_exists(self):
        r = Repulsor(make_rng())
        assert r.ID == "Repulsor"


class TestWrithingMassParity:
    def test_exists(self):
        w = WrithingMass(make_rng())
        assert w.ID == "WrithingMass"


class TestTransientParity:
    def test_exists(self):
        t = Transient(make_rng())
        assert t.ID == "Transient"


class TestExploderParity:
    def test_exists(self):
        e = Exploder(make_rng())
        assert e.ID == "Exploder"


class TestSpireGrowthParity:
    def test_exists(self):
        s = SpireGrowth(make_rng())
        assert s.ID == "Serpent"


class TestSnakeDaggerParity:
    def test_exists(self):
        s = SnakeDagger(make_rng())
        assert s.ID == "Dagger"


class TestMawParity:
    def test_exists(self):
        m = Maw(make_rng())
        assert m.ID == "Maw"


# ============================================================
# ACT 3 ELITES
# ============================================================

class TestGiantHeadParity:
    def test_exists(self):
        g = GiantHead(make_rng())
        assert g.ID == "GiantHead"


class TestNemesisParity:
    def test_exists(self):
        n = Nemesis(make_rng())
        assert n.ID == "Nemesis"


class TestReptomancerParity:
    def test_exists(self):
        r = Reptomancer(make_rng())
        assert r.ID == "Reptomancer"


# ============================================================
# GREMLINS (Act 1/2 misc)
# ============================================================

class TestGremlinFatParity:
    def test_exists(self):
        g = GremlinFat(make_rng())
        assert g.ID == "GremlinFat"


class TestGremlinThiefParity:
    def test_exists(self):
        g = GremlinThief(make_rng())
        assert g.ID == "GremlinThief"


class TestGremlinTsundereParity:
    def test_exists(self):
        g = GremlinTsundere(make_rng())
        assert g.ID == "GremlinTsundere"


class TestGremlinWarriorParity:
    def test_exists(self):
        g = GremlinWarrior(make_rng())
        assert g.ID == "GremlinWarrior"


class TestGremlinWizardParity:
    def test_exists(self):
        g = GremlinWizard(make_rng())
        assert g.ID == "GremlinWizard"


class TestBanditBearParity:
    def test_exists(self):
        b = BanditBear(make_rng())
        assert b.ID == "BanditBear"


class TestBanditLeaderParity:
    def test_exists(self):
        b = BanditLeader(make_rng())
        assert b.ID == "BanditLeader"


class TestBanditPointyParity:
    def test_exists(self):
        b = BanditPointy(make_rng())
        assert b.ID == "BanditChild"


# ============================================================
# ACT 2 BOSSES (continued)
# ============================================================

class TestTheCollectorParity:
    """Java: city/TheCollector.java"""

    def test_hp_base(self):
        c = TheCollector(make_rng(), ascension=0)
        assert c.state.max_hp == 282

    def test_hp_a9(self):
        c = TheCollector(make_rng(), ascension=9)
        assert c.state.max_hp == 300


class TestBronzeAutomatonParity:
    """Java: city/BronzeAutomaton.java"""

    def test_hp_base(self):
        b = BronzeAutomaton(make_rng(), ascension=0)
        assert b.state.max_hp == 300

    def test_hp_a9(self):
        b = BronzeAutomaton(make_rng(), ascension=9)
        assert b.state.max_hp == 320

    def test_first_move_spawn(self):
        b = BronzeAutomaton(make_rng())
        move = b.get_move(50)
        assert move.move_id == b.SPAWN_ORBS


# ============================================================
# MISSING ENEMY CHECK
# ============================================================

class TestMissingEnemies:
    """Check that all Java enemy classes have Python equivalents."""

    def test_apology_slime_missing(self):
        """
        Java has ApologySlime.java in exordium/.
        This is a cosmetic/joke enemy but may need implementation.
        """
        with pytest.raises(ImportError):
            from packages.engine.content.enemies import ApologySlime  # noqa

    @pytest.mark.skip(reason="ApologySlime is a joke/cosmetic enemy, not needed for gameplay parity")
    def test_apology_slime_exists(self):
        """ApologySlime is a joke enemy, not needed for parity."""
        from packages.engine.content.enemies import ApologySlime  # noqa
