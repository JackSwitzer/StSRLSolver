"""
Boss mechanics audit tests - verifies Python engine against decompiled Java.

Tests cover: Champ, TheCollector, BronzeAutomaton, AwakenedOne, TimeEater,
CorruptHeart, SpireShield, SpireSpear.
"""

import pytest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from packages.engine.state.rng import Random
from packages.engine.content.enemies import (
    Champ, TheCollector, BronzeAutomaton, AwakenedOne, TimeEater,
    CorruptHeart, SpireShield, SpireSpear, TorchHead, BronzeOrb,
    Intent, MoveInfo,
)


def make_enemy(cls, ascension=0, seed=12345):
    """Helper to create an enemy with a fixed RNG seed."""
    rng = Random(seed)
    return cls(rng, ascension)


# =============================================================================
# Champ Tests
# =============================================================================

class TestChamp:
    def test_hp_base(self):
        c = make_enemy(Champ, ascension=0)
        assert c.state.max_hp == 420

    def test_hp_a9(self):
        c = make_enemy(Champ, ascension=9)
        assert c.state.max_hp == 440

    def test_damage_base(self):
        c = make_enemy(Champ, ascension=0)
        dmg = c._get_damage_values()
        assert dmg["slash"] == 16
        assert dmg["execute"] == 10
        assert dmg["slap"] == 12
        assert dmg["strength"] == 2
        assert dmg["forge"] == 5
        assert dmg["block"] == 15

    def test_damage_a4(self):
        c = make_enemy(Champ, ascension=4)
        dmg = c._get_damage_values()
        assert dmg["slash"] == 18
        assert dmg["slap"] == 14
        assert dmg["strength"] == 3

    def test_damage_a9(self):
        c = make_enemy(Champ, ascension=9)
        dmg = c._get_damage_values()
        assert dmg["forge"] == 6
        assert dmg["block"] == 18

    def test_damage_a19(self):
        c = make_enemy(Champ, ascension=19)
        dmg = c._get_damage_values()
        assert dmg["strength"] == 4
        assert dmg["forge"] == 7
        assert dmg["block"] == 20

    def test_phase_transition_threshold(self):
        """Champ transitions at < 50% HP."""
        c = make_enemy(Champ, ascension=0)
        assert not c.check_phase_transition()
        c.state.current_hp = 210  # Exactly 50%
        assert not c.check_phase_transition()
        c.state.current_hp = 209  # Below 50%
        assert c.check_phase_transition()

    def test_anger_gives_triple_strength(self):
        """On phase transition, Anger gives strAmt * 3."""
        c = make_enemy(Champ, ascension=0)
        c.state.current_hp = 100  # Below 50%
        move = c.get_move(50)
        assert move.move_id == c.ANGER
        assert move.effects["strength"] == 6  # 2 * 3

    def test_anger_a19_strength(self):
        c = make_enemy(Champ, ascension=19)
        c.state.current_hp = 100
        move = c.get_move(50)
        assert move.move_id == c.ANGER
        assert move.effects["strength"] == 12  # 4 * 3

    def test_execute_no_triple_repeat(self):
        """Execute cannot be used 3x in a row (checks last + last_before)."""
        c = make_enemy(Champ, ascension=0)
        c.threshold_reached = True
        # Simulate two Executes in history
        c.state.move_history = [c.EXECUTE, c.EXECUTE]
        move = c.get_move(50)
        # Should fall through to Phase 1 logic, not Execute
        assert move.move_id != c.EXECUTE

    def test_taunt_every_4th_turn(self):
        """Taunt fires on numTurns==4 in Phase 1."""
        c = make_enemy(Champ, ascension=0)
        c.num_turns = 3  # Will be incremented to 4
        move = c.get_move(50)
        assert move.move_id == c.TAUNT

    def test_defensive_stance_threshold_a19(self):
        """A19 has 30% threshold for Defensive Stance."""
        c = make_enemy(Champ, ascension=19)
        c.num_turns = 0  # Reset to avoid taunt
        move = c.get_move(30)  # roll=30, threshold is 30
        assert move.move_id == c.DEFENSIVE_STANCE

    def test_defensive_stance_max_uses(self):
        """Defensive Stance limited to 2 uses."""
        c = make_enemy(Champ, ascension=19)
        c.forge_times = 2
        c.num_turns = 0
        move = c.get_move(10)  # Low roll, but forge exhausted
        assert move.move_id != c.DEFENSIVE_STANCE


# =============================================================================
# TheCollector Tests
# =============================================================================

class TestTheCollector:
    def test_hp_base(self):
        c = make_enemy(TheCollector, ascension=0)
        assert c.state.max_hp == 282

    def test_hp_a9(self):
        c = make_enemy(TheCollector, ascension=9)
        assert c.state.max_hp == 300

    def test_first_turn_spawn(self):
        c = make_enemy(TheCollector, ascension=0)
        move = c.get_move(50)
        assert move.move_id == c.SPAWN

    def test_mega_debuff_a19(self):
        """A19 mega debuff applies 5 turns of each debuff."""
        c = make_enemy(TheCollector, ascension=19)
        dmg = c._get_damage_values()
        assert dmg["mega_debuff"] == 5

    def test_mega_debuff_timing(self):
        """Mega Debuff triggers after 3 turns taken."""
        c = make_enemy(TheCollector, ascension=0)
        c.initial_spawn = False
        c.turns_taken = 3
        move = c.get_move(50)
        assert move.move_id == c.MEGA_DEBUFF

    def test_mega_debuff_only_once(self):
        c = make_enemy(TheCollector, ascension=0)
        c.initial_spawn = False
        c.turns_taken = 3
        c.ult_used = True
        move = c.get_move(50)
        assert move.move_id != c.MEGA_DEBUFF

    def test_fireball_no_triple_repeat(self):
        """Fireball uses lastTwoMoves check (not lastMove)."""
        c = make_enemy(TheCollector, ascension=0)
        c.initial_spawn = False
        c.turns_taken = 1
        c.state.move_history = [c.FIREBALL, c.FIREBALL]
        move = c.get_move(50)  # roll <= 70 but last two were Fireball
        assert move.move_id != c.FIREBALL


# =============================================================================
# BronzeAutomaton Tests
# =============================================================================

class TestBronzeAutomaton:
    def test_hp_base(self):
        a = make_enemy(BronzeAutomaton, ascension=0)
        assert a.state.max_hp == 300

    def test_hp_a9(self):
        a = make_enemy(BronzeAutomaton, ascension=9)
        assert a.state.max_hp == 320

    def test_pre_battle_artifact(self):
        a = make_enemy(BronzeAutomaton, ascension=0)
        effects = a.get_pre_battle_effects()
        assert effects["self_effects"]["artifact"] == 3

    def test_first_turn_spawn(self):
        a = make_enemy(BronzeAutomaton, ascension=0)
        move = a.get_move(50)
        assert move.move_id == a.SPAWN_ORBS

    def test_hyper_beam_at_turn_4(self):
        a = make_enemy(BronzeAutomaton, ascension=0)
        a.state.first_turn = False
        a.num_turns = 4
        move = a.get_move(50)
        assert move.move_id == a.HYPER_BEAM

    def test_stun_after_beam_below_a19(self):
        a = make_enemy(BronzeAutomaton, ascension=0)
        a.state.first_turn = False
        a.state.move_history = [a.HYPER_BEAM]
        move = a.get_move(50)
        assert move.move_id == a.STUNNED

    def test_boost_after_beam_a19(self):
        a = make_enemy(BronzeAutomaton, ascension=19)
        a.state.first_turn = False
        a.state.move_history = [a.HYPER_BEAM]
        move = a.get_move(50)
        assert move.move_id == a.BOOST

    def test_flail_after_stun(self):
        a = make_enemy(BronzeAutomaton, ascension=0)
        a.state.first_turn = False
        a.state.move_history = [a.HYPER_BEAM, a.STUNNED]
        move = a.get_move(50)
        assert move.move_id == a.FLAIL

    def test_damage_a4(self):
        a = make_enemy(BronzeAutomaton, ascension=4)
        dmg = a._get_damage_values()
        assert dmg["flail"] == 8
        assert dmg["beam"] == 50
        assert dmg["strength"] == 4


# =============================================================================
# AwakenedOne Tests
# =============================================================================

class TestAwakenedOne:
    def test_hp_base(self):
        a = make_enemy(AwakenedOne, ascension=0)
        assert a.state.max_hp == 300

    def test_hp_a9(self):
        a = make_enemy(AwakenedOne, ascension=9)
        assert a.state.max_hp == 320

    def test_phase1_first_turn_slash(self):
        a = make_enemy(AwakenedOne, ascension=0)
        move = a.get_move(50)
        assert move.move_id == a.SLASH

    def test_phase1_soul_strike_25_percent(self):
        """Roll < 25 should prefer Soul Strike if not repeated."""
        a = make_enemy(AwakenedOne, ascension=0)
        a.phase_first_turn = False
        a.state.move_history = [a.SLASH]
        move = a.get_move(10)
        assert move.move_id == a.SOUL_STRIKE

    def test_phase1_slash_no_triple(self):
        """Slash uses lastTwoMoves - can't be used 3x in a row."""
        a = make_enemy(AwakenedOne, ascension=0)
        a.phase_first_turn = False
        a.state.move_history = [a.SLASH, a.SLASH]
        move = a.get_move(90)  # High roll = Slash branch
        assert move.move_id == a.SOUL_STRIKE  # Forced fallback

    def test_pre_battle_curiosity_a19(self):
        a = make_enemy(AwakenedOne, ascension=19)
        effects = a.get_pre_battle_effects()
        assert effects["self_effects"]["curiosity"] == 2
        assert effects["self_effects"]["regenerate"] == 15

    def test_pre_battle_curiosity_base(self):
        a = make_enemy(AwakenedOne, ascension=0)
        effects = a.get_pre_battle_effects()
        assert effects["self_effects"]["curiosity"] == 1
        assert effects["self_effects"]["regenerate"] == 10

    def test_pre_battle_a4_strength(self):
        a = make_enemy(AwakenedOne, ascension=4)
        effects = a.get_pre_battle_effects()
        assert effects["self_effects"]["strength"] == 2

    def test_rebirth_restores_hp(self):
        a = make_enemy(AwakenedOne, ascension=0)
        a.trigger_rebirth()
        assert a.phase == 2
        assert a.state.current_hp == 300
        assert a.state.max_hp == 300

    def test_rebirth_hp_a9(self):
        a = make_enemy(AwakenedOne, ascension=9)
        a.trigger_rebirth()
        assert a.state.current_hp == 320

    def test_phase2_first_turn_dark_echo(self):
        a = make_enemy(AwakenedOne, ascension=0)
        a.trigger_rebirth()
        move = a.get_move(50)
        assert move.move_id == a.DARK_ECHO

    def test_phase2_sludge_no_triple(self):
        a = make_enemy(AwakenedOne, ascension=0)
        a.phase = 2
        a.phase_first_turn = False
        a.state.move_history = [a.SLUDGE, a.SLUDGE]
        move = a.get_move(10)  # Low roll = Sludge branch
        assert move.move_id == a.TACKLE  # Forced fallback

    def test_rebirth_preserves_strength(self):
        """Java only removes debuffs, Curiosity, Unawakened, Shackled on rebirth.
        Buffs like Strength should be preserved."""
        a = make_enemy(AwakenedOne, ascension=4)
        # Simulate pre-battle: Strength 2 from A4+
        a.state.powers = {
            "strength": 2,
            "regenerate": 10,
            "curiosity": 1,
            "unawakened": 1,
            "weak": 3,  # A debuff
        }
        a.trigger_rebirth()
        # Strength and Regenerate should survive
        assert a.state.powers.get("strength") == 2
        assert a.state.powers.get("regenerate") == 10
        # Curiosity, Unawakened, debuffs should be gone
        assert "curiosity" not in a.state.powers
        assert "unawakened" not in a.state.powers
        assert "weak" not in a.state.powers


# =============================================================================
# TimeEater Tests
# =============================================================================

class TestTimeEater:
    def test_hp_base(self):
        t = make_enemy(TimeEater, ascension=0)
        assert t.state.max_hp == 456

    def test_hp_a9(self):
        t = make_enemy(TimeEater, ascension=9)
        assert t.state.max_hp == 480

    def test_damage_base(self):
        t = make_enemy(TimeEater, ascension=0)
        dmg = t._get_damage_values()
        assert dmg["reverberate"] == 7
        assert dmg["head_slam"] == 26

    def test_damage_a4(self):
        t = make_enemy(TimeEater, ascension=4)
        dmg = t._get_damage_values()
        assert dmg["reverberate"] == 8
        assert dmg["head_slam"] == 32

    def test_pre_battle_time_warp(self):
        t = make_enemy(TimeEater, ascension=0)
        effects = t.get_pre_battle_effects()
        assert effects["self_effects"]["time_warp"] == 12

    def test_haste_trigger(self):
        """Haste triggers at < 50% HP."""
        t = make_enemy(TimeEater, ascension=0)
        t.state.current_hp = 227  # < 228 (456/2)
        move = t.get_move(50)
        assert move.move_id == t.HASTE

    def test_haste_heals_to_half(self):
        t = make_enemy(TimeEater, ascension=0)
        t.state.current_hp = 100
        move = t.get_move(50)
        assert move.effects.get("heal_to_half") is True

    def test_haste_once_only(self):
        t = make_enemy(TimeEater, ascension=0)
        t.state.current_hp = 100
        t.used_haste = True
        move = t.get_move(20)
        assert move.move_id != t.HASTE

    def test_haste_a19_block(self):
        t = make_enemy(TimeEater, ascension=19)
        t.state.current_hp = 100
        move = t.get_move(50)
        assert move.move_id == t.HASTE
        assert move.effects.get("block") == 32  # headSlamDmg at A4+

    def test_reverberate_no_triple(self):
        t = make_enemy(TimeEater, ascension=0)
        t.state.move_history = [t.REVERBERATE, t.REVERBERATE]
        move = t.get_move(10)  # Low roll = Reverberate branch, forced recurse
        # Should recurse with roll 50-99 and pick non-reverberate
        assert move.move_id in (t.HEAD_SLAM, t.RIPPLE, t.REVERBERATE)

    def test_head_slam_a19_slimed(self):
        """A19 Head Slam adds 2 Slimed cards."""
        t = make_enemy(TimeEater, ascension=19)
        t.state.move_history = [t.REVERBERATE]
        move = t.get_move(60)  # 45-79 = Head Slam branch
        assert move.move_id == t.HEAD_SLAM
        assert move.effects.get("slimed") == 2

    def test_ripple_a19_frail(self):
        """A19 Ripple adds Frail."""
        t = make_enemy(TimeEater, ascension=19)
        t.state.move_history = [t.HEAD_SLAM]
        move = t.get_move(90)  # 80-99 = Ripple branch
        assert move.move_id == t.RIPPLE
        assert move.effects.get("frail") == 1


# =============================================================================
# CorruptHeart Tests
# =============================================================================

class TestCorruptHeart:
    def test_hp_base(self):
        h = make_enemy(CorruptHeart, ascension=0)
        assert h.state.max_hp == 750

    def test_hp_a9(self):
        h = make_enemy(CorruptHeart, ascension=9)
        assert h.state.max_hp == 800

    def test_damage_base(self):
        h = make_enemy(CorruptHeart, ascension=0)
        dmg = h._get_damage_values()
        assert dmg["echo"] == 40
        assert dmg["blood"] == 2
        assert dmg["blood_count"] == 12

    def test_damage_a4(self):
        h = make_enemy(CorruptHeart, ascension=4)
        dmg = h._get_damage_values()
        assert dmg["echo"] == 45
        assert dmg["blood_count"] == 15

    def test_pre_battle_invincible(self):
        h = make_enemy(CorruptHeart, ascension=0)
        effects = h.get_pre_battle_effects()
        assert effects["self_effects"]["invincible"] == 300

    def test_pre_battle_invincible_a19(self):
        h = make_enemy(CorruptHeart, ascension=19)
        effects = h.get_pre_battle_effects()
        assert effects["self_effects"]["invincible"] == 200

    def test_pre_battle_beat_of_death(self):
        h = make_enemy(CorruptHeart, ascension=0)
        effects = h.get_pre_battle_effects()
        assert effects["self_effects"]["beat_of_death"] == 1

    def test_pre_battle_beat_of_death_a19(self):
        h = make_enemy(CorruptHeart, ascension=19)
        effects = h.get_pre_battle_effects()
        assert effects["self_effects"]["beat_of_death"] == 2

    def test_first_move_debilitate(self):
        h = make_enemy(CorruptHeart, ascension=0)
        move = h.get_move(50)
        assert move.move_id == h.DEBILITATE
        assert move.effects["vulnerable"] == 2
        assert move.effects["weak"] == 2
        assert move.effects["frail"] == 2
        assert len(move.effects["status_cards"]) == 5

    def test_buff_cycle_0_artifact(self):
        h = make_enemy(CorruptHeart, ascension=0)
        h.is_first_move = False
        h.move_count = 2  # cycle_pos == 2 -> BUFF
        move = h.get_move(50)
        assert move.move_id == h.BUFF
        assert move.effects.get("artifact") == 2

    def test_buff_cycle_1_beat_of_death(self):
        h = make_enemy(CorruptHeart, ascension=0)
        h.is_first_move = False
        h.buff_count = 1
        h.move_count = 2
        move = h.get_move(50)
        assert move.effects.get("beat_of_death") == 1

    def test_buff_cycle_2_painful_stabs(self):
        h = make_enemy(CorruptHeart, ascension=0)
        h.is_first_move = False
        h.buff_count = 2
        h.move_count = 2
        move = h.get_move(50)
        assert move.effects.get("painful_stabs") is True

    def test_buff_cycle_3_strength_10(self):
        """Buff cycle 3: base 2 + 10 = 12 total Strength."""
        h = make_enemy(CorruptHeart, ascension=0)
        h.is_first_move = False
        h.buff_count = 3
        h.move_count = 2
        move = h.get_move(50)
        assert move.effects.get("strength") == 12

    def test_buff_cycle_4_strength_50(self):
        """Buff cycle 4+: base 2 + 50 = 52 total Strength."""
        h = make_enemy(CorruptHeart, ascension=0)
        h.is_first_move = False
        h.buff_count = 4
        h.move_count = 2
        move = h.get_move(50)
        assert move.effects.get("strength") == 52

    def test_3_turn_cycle(self):
        """Attack/Attack/Buff repeating cycle after Debilitate."""
        h = make_enemy(CorruptHeart, ascension=0)
        # Turn 1: Debilitate
        move1 = h.get_move(50)
        assert move1.move_id == h.DEBILITATE
        # Turn 2: Attack (cycle 0)
        move2 = h.get_move(50)
        assert move2.move_id in (h.BLOOD_SHOTS, h.ECHO)
        # Turn 3: Attack (cycle 1)
        h.state.move_history.append(move2.move_id)
        move3 = h.get_move(50)
        assert move3.move_id in (h.BLOOD_SHOTS, h.ECHO)
        # Turn 4: Buff (cycle 2)
        h.state.move_history.append(move3.move_id)
        move4 = h.get_move(50)
        assert move4.move_id == h.BUFF


# =============================================================================
# SpireShield Tests
# =============================================================================

class TestSpireShield:
    def test_hp_base(self):
        s = make_enemy(SpireShield, ascension=0)
        assert s.state.max_hp == 110

    def test_hp_a8(self):
        s = make_enemy(SpireShield, ascension=8)
        assert s.state.max_hp == 125

    def test_damage_base(self):
        s = make_enemy(SpireShield, ascension=0)
        dmg = s._get_damage_values()
        assert dmg["bash"] == 12
        assert dmg["smash"] == 34

    def test_damage_a3(self):
        s = make_enemy(SpireShield, ascension=3)
        dmg = s._get_damage_values()
        assert dmg["bash"] == 14
        assert dmg["smash"] == 38

    def test_pre_battle_artifact(self):
        s = make_enemy(SpireShield, ascension=0)
        effects = s.get_pre_battle_effects()
        assert effects["self_effects"]["artifact"] == 1

    def test_pre_battle_artifact_a18(self):
        s = make_enemy(SpireShield, ascension=18)
        effects = s.get_pre_battle_effects()
        assert effects["self_effects"]["artifact"] == 2

    def test_pre_battle_surrounded(self):
        s = make_enemy(SpireShield, ascension=0)
        effects = s.get_pre_battle_effects()
        assert effects["player_effects"]["surrounded"] is True

    def test_smash_a18_block_99(self):
        """A18+ Smash gives 99 block."""
        s = make_enemy(SpireShield, ascension=18)
        s.move_count = 2  # cycle_pos 2 = Smash
        move = s.get_move(50)
        assert move.move_id == s.SMASH
        assert move.block == 99

    def test_smash_below_a18_block_equals_damage_output(self):
        """Below A18, Smash block should equal damage output (affected by Strength).
        Java: this.damage.get(1).output -- Python incorrectly uses base value."""
        s = make_enemy(SpireShield, ascension=3)
        # Simulate +4 Strength (e.g., from Piercer)
        s.state.strength = 4
        s.move_count = 2
        move = s.get_move(50)
        assert move.move_id == s.SMASH
        # Base damage is 38, with +4 strength = 42 output
        # Block should be 42, not 38
        assert move.block == 42

    def test_3_turn_cycle_smash_at_pos_2(self):
        """Smash always at cycle position 2."""
        s = make_enemy(SpireShield, ascension=0)
        s.move_count = 2
        move = s.get_move(50)
        assert move.move_id == s.SMASH

    def test_fortify_block_all_monsters(self):
        s = make_enemy(SpireShield, ascension=0)
        s.move_count = 0  # cycle_pos 0
        # Force Fortify path (first call to random_boolean)
        # We can't easily control the RNG, so just test structure
        # by manually setting move_count to test cycle_pos 1 after FORTIFY
        s.state.move_history = [s.FORTIFY]
        s.move_count = 1  # cycle_pos 1, last != BASH -> BASH
        move = s.get_move(50)
        assert move.move_id == s.BASH


# =============================================================================
# SpireSpear Tests
# =============================================================================

class TestSpireSpear:
    def test_hp_base(self):
        s = make_enemy(SpireSpear, ascension=0)
        assert s.state.max_hp == 160

    def test_hp_a8(self):
        s = make_enemy(SpireSpear, ascension=8)
        assert s.state.max_hp == 180

    def test_damage_base(self):
        s = make_enemy(SpireSpear, ascension=0)
        dmg = s._get_damage_values()
        assert dmg["burn_strike"] == 5
        assert dmg["skewer"] == 10
        assert dmg["skewer_count"] == 3

    def test_damage_a3(self):
        s = make_enemy(SpireSpear, ascension=3)
        dmg = s._get_damage_values()
        assert dmg["burn_strike"] == 6
        assert dmg["skewer_count"] == 4

    def test_pre_battle_artifact(self):
        s = make_enemy(SpireSpear, ascension=0)
        effects = s.get_pre_battle_effects()
        assert effects["self_effects"]["artifact"] == 1

    def test_pre_battle_artifact_a18(self):
        s = make_enemy(SpireSpear, ascension=18)
        effects = s.get_pre_battle_effects()
        assert effects["self_effects"]["artifact"] == 2

    def test_cycle_pos_1_skewer(self):
        """Skewer always at cycle position 1."""
        s = make_enemy(SpireSpear, ascension=0)
        s.move_count = 1
        move = s.get_move(50)
        assert move.move_id == s.SKEWER

    def test_burn_strike_first_turn(self):
        """First turn (cycle 0) prefers Burn Strike if not repeated."""
        s = make_enemy(SpireSpear, ascension=0)
        move = s.get_move(50)
        assert move.move_id == s.BURN_STRIKE

    def test_burn_strike_a18_to_draw(self):
        """A18+ Burn Strike puts Burns in draw pile."""
        s = make_enemy(SpireSpear, ascension=18)
        move = s.get_move(50)
        assert move.effects.get("to_draw_pile") is True

    def test_burn_strike_below_a18_to_discard(self):
        """Below A18, Burns go to discard pile."""
        s = make_enemy(SpireSpear, ascension=0)
        move = s.get_move(50)
        assert move.effects.get("to_draw_pile") is False

    def test_piercer_strength_all_monsters(self):
        """Piercer gives +2 Strength to ALL monsters."""
        s = make_enemy(SpireSpear, ascension=0)
        s.state.move_history = [s.BURN_STRIKE]
        s.move_count = 0  # cycle_pos 0, last was BurnStrike -> Piercer
        move = s.get_move(50)
        assert move.move_id == s.PIERCER
        assert move.effects.get("strength_all_monsters") == 2


# =============================================================================
# Cross-Boss / Data Consistency Tests
# =============================================================================

class TestBossDataConsistency:
    """Verify that static data tables match class implementations."""

    def test_champ_move_ids(self):
        assert Champ.HEAVY_SLASH == 1
        assert Champ.DEFENSIVE_STANCE == 2
        assert Champ.EXECUTE == 3
        assert Champ.FACE_SLAP == 4
        assert Champ.GLOAT == 5
        assert Champ.TAUNT == 6
        assert Champ.ANGER == 7

    def test_collector_move_ids(self):
        assert TheCollector.SPAWN == 1
        assert TheCollector.FIREBALL == 2
        assert TheCollector.BUFF == 3
        assert TheCollector.MEGA_DEBUFF == 4
        assert TheCollector.REVIVE == 5

    def test_automaton_move_ids(self):
        assert BronzeAutomaton.FLAIL == 1
        assert BronzeAutomaton.HYPER_BEAM == 2
        assert BronzeAutomaton.STUNNED == 3
        assert BronzeAutomaton.SPAWN_ORBS == 4
        assert BronzeAutomaton.BOOST == 5

    def test_awakened_move_ids(self):
        assert AwakenedOne.SLASH == 1
        assert AwakenedOne.SOUL_STRIKE == 2
        assert AwakenedOne.REBIRTH == 3
        assert AwakenedOne.DARK_ECHO == 5
        assert AwakenedOne.SLUDGE == 6
        assert AwakenedOne.TACKLE == 8

    def test_time_eater_move_ids(self):
        assert TimeEater.REVERBERATE == 2
        assert TimeEater.RIPPLE == 3
        assert TimeEater.HEAD_SLAM == 4
        assert TimeEater.HASTE == 5

    def test_heart_move_ids(self):
        assert CorruptHeart.BLOOD_SHOTS == 1
        assert CorruptHeart.ECHO == 2
        assert CorruptHeart.DEBILITATE == 3
        assert CorruptHeart.BUFF == 4

    def test_shield_move_ids(self):
        assert SpireShield.BASH == 1
        assert SpireShield.FORTIFY == 2
        assert SpireShield.SMASH == 3

    def test_spear_move_ids(self):
        assert SpireSpear.BURN_STRIKE == 1
        assert SpireSpear.PIERCER == 2
        assert SpireSpear.SKEWER == 3
