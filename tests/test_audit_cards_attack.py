"""
Audit tests: Watcher attack card values vs decompiled Java.

Verifies base damage, upgrade damage, cost, magic numbers, flags,
and effect lists for all 17 audited Watcher attack cards.
"""
import pytest
from packages.engine.content.cards import (
    Card, CardType, CardRarity, CardTarget,
    get_card,
    BOWLING_BASH, EMPTY_FIST, FLURRY_OF_BLOWS, FLYING_SLEEVES,
    FOLLOW_UP, JUST_LUCKY, SASH_WHIP, CRUSH_JOINTS,
    TANTRUM, FEAR_NO_EVIL, REACH_HEAVEN, SIGNATURE_MOVE,
    WALLOP, WHEEL_KICK, WINDMILL_STRIKE, CONCLUDE, RAGNAROK,
)


# ============================================================
# Java ground truth: (id, cost, baseDamage, upgradeDamage,
#   baseMagic, upgradeMagic, baseBlock, upgradeBlock, rarity)
# ============================================================
JAVA_VALUES = [
    ("BowlingBash",    1,  7, 3, -1, 0, -1, 0, CardRarity.COMMON),
    ("EmptyFist",      1,  9, 5, -1, 0, -1, 0, CardRarity.COMMON),
    ("FlurryOfBlows",  0,  4, 2, -1, 0, -1, 0, CardRarity.COMMON),
    ("FlyingSleeves",  1,  4, 2, -1, 0, -1, 0, CardRarity.COMMON),
    ("FollowUp",       1,  7, 4, -1, 0, -1, 0, CardRarity.COMMON),
    ("JustLucky",      0,  3, 1,  1, 1,  2, 1, CardRarity.COMMON),
    ("SashWhip",       1,  8, 2,  1, 1, -1, 0, CardRarity.COMMON),
    ("CrushJoints",    1,  8, 2,  1, 1, -1, 0, CardRarity.COMMON),
    ("Tantrum",        1,  3, 0,  3, 1, -1, 0, CardRarity.UNCOMMON),
    ("FearNoEvil",     1,  8, 3, -1, 0, -1, 0, CardRarity.UNCOMMON),
    ("ReachHeaven",    2, 10, 5, -1, 0, -1, 0, CardRarity.UNCOMMON),
    ("SignatureMove",  2, 30,10, -1, 0, -1, 0, CardRarity.UNCOMMON),
    ("Wallop",         2,  9, 3, -1, 0, -1, 0, CardRarity.UNCOMMON),
    ("WheelKick",      2, 15, 5, -1, 0, -1, 0, CardRarity.UNCOMMON),
    # WindmillStrike: Java base_magic=4 (retain bonus), Python encodes in effect name instead
    ("WindmillStrike", 2,  7, 3, -1, 0, -1, 0, CardRarity.UNCOMMON),
    ("Conclude",       1, 12, 4, -1, 0, -1, 0, CardRarity.UNCOMMON),
    ("Ragnarok",       3,  5, 1,  5, 1, -1, 0, CardRarity.RARE),
]


class TestWatcherAttackBaseValues:
    """Verify base stat values match Java for all 17 attack cards."""

    @pytest.mark.parametrize(
        "card_id,cost,base_dmg,upg_dmg,base_mag,upg_mag,base_blk,upg_blk,rarity",
        JAVA_VALUES,
        ids=[v[0] for v in JAVA_VALUES],
    )
    def test_base_values(self, card_id, cost, base_dmg, upg_dmg,
                         base_mag, upg_mag, base_blk, upg_blk, rarity):
        card = get_card(card_id)
        assert card.card_type == CardType.ATTACK, f"{card_id} should be ATTACK"
        assert card.cost == cost, f"{card_id} cost"
        assert card.base_damage == base_dmg, f"{card_id} base_damage"
        assert card.upgrade_damage == upg_dmg, f"{card_id} upgrade_damage"
        assert card.base_magic == base_mag, f"{card_id} base_magic"
        assert card.upgrade_magic == upg_mag, f"{card_id} upgrade_magic"
        assert card.base_block == base_blk, f"{card_id} base_block"
        assert card.upgrade_block == upg_blk, f"{card_id} upgrade_block"
        assert card.rarity == rarity, f"{card_id} rarity"

    @pytest.mark.parametrize(
        "card_id,cost,base_dmg,upg_dmg,base_mag,upg_mag,base_blk,upg_blk,rarity",
        JAVA_VALUES,
        ids=[v[0] for v in JAVA_VALUES],
    )
    def test_upgraded_values(self, card_id, cost, base_dmg, upg_dmg,
                             base_mag, upg_mag, base_blk, upg_blk, rarity):
        card = get_card(card_id, upgraded=True)
        assert card.upgraded is True
        expected_dmg = base_dmg + upg_dmg
        assert card.damage == expected_dmg, f"{card_id}+ damage should be {expected_dmg}"
        if base_mag >= 0:
            expected_mag = base_mag + upg_mag
            assert card.magic_number == expected_mag, f"{card_id}+ magic_number"
        if base_blk >= 0:
            expected_blk = base_blk + upg_blk
            assert card.block == expected_blk, f"{card_id}+ block"


class TestWatcherAttackFlags:
    """Verify card flags match Java."""

    def test_flying_sleeves_retain(self):
        assert FLYING_SLEEVES.retain is True

    def test_windmill_strike_retain(self):
        assert WINDMILL_STRIKE.retain is True

    def test_tantrum_shuffle_back(self):
        assert TANTRUM.shuffle_back is True

    def test_tantrum_enter_wrath(self):
        assert TANTRUM.enter_stance == "Wrath"

    def test_empty_fist_exit_stance(self):
        assert EMPTY_FIST.exit_stance is True

    def test_conclude_target_all(self):
        assert CONCLUDE.target == CardTarget.ALL_ENEMY

    def test_ragnarok_target_all(self):
        assert RAGNAROK.target == CardTarget.ALL_ENEMY

    def test_flurry_of_blows_zero_cost(self):
        assert FLURRY_OF_BLOWS.cost == 0

    def test_just_lucky_zero_cost(self):
        assert JUST_LUCKY.cost == 0


class TestWatcherAttackEffects:
    """Verify effect lists are registered correctly."""

    def test_bowling_bash_effects(self):
        assert "damage_per_enemy" in BOWLING_BASH.effects

    def test_flying_sleeves_effects(self):
        assert "damage_twice" in FLYING_SLEEVES.effects

    def test_follow_up_effects(self):
        assert "if_last_card_attack_gain_energy" in FOLLOW_UP.effects

    def test_sash_whip_effects(self):
        assert "if_last_card_attack_weak" in SASH_WHIP.effects

    def test_crush_joints_effects(self):
        assert "if_last_card_skill_vulnerable" in CRUSH_JOINTS.effects

    def test_tantrum_effects(self):
        assert "damage_x_times" in TANTRUM.effects

    def test_fear_no_evil_effects(self):
        assert "if_enemy_attacking_enter_calm" in FEAR_NO_EVIL.effects

    def test_reach_heaven_effects(self):
        assert "add_through_violence_to_draw" in REACH_HEAVEN.effects

    def test_signature_move_effects(self):
        assert "only_attack_in_hand" in SIGNATURE_MOVE.effects

    def test_wallop_effects(self):
        assert "gain_block_equal_unblocked_damage" in WALLOP.effects

    def test_wheel_kick_effects(self):
        assert "draw_2" in WHEEL_KICK.effects

    def test_windmill_strike_effects(self):
        assert "gain_damage_when_retained_4" in WINDMILL_STRIKE.effects

    def test_conclude_effects(self):
        assert "end_turn" in CONCLUDE.effects

    def test_ragnarok_effects(self):
        assert "damage_random_x_times" in RAGNAROK.effects

    def test_flurry_of_blows_effects(self):
        assert "on_stance_change_play_from_discard" in FLURRY_OF_BLOWS.effects

    def test_just_lucky_effects(self):
        assert "scry" in JUST_LUCKY.effects
        assert "gain_block" in JUST_LUCKY.effects
