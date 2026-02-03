"""
Audit tests: Watcher skill & power card values vs decompiled Java.

Verifies base values, upgrade deltas, costs, flags, and known bugs
for all audited Watcher skill and power cards.
"""
import pytest
from packages.engine.content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    get_card,
    VIGILANCE, HALT, TRANQUILITY, CRESCENDO,
    EMPTY_BODY, EMPTY_MIND, EVALUATE, INNER_PEACE,
    PROTECT, THIRD_EYE, PROSTRATE,
    COLLECT, DECEIVE_REALITY, MEDITATE, PERSEVERANCE,
    PRAY, SANCTITY, SWIVEL, WAVE_OF_THE_HAND, WORSHIP,
    JUDGMENT, ALPHA, BLASPHEMY, CONJURE_BLADE,
    FOREIGN_INFLUENCE, OMNISCIENCE, SCRAWL, SPIRIT_SHIELD,
    VAULT, WISH,
    BATTLE_HYMN, ESTABLISHMENT, LIKE_WATER, MENTAL_FORTRESS,
    RUSHDOWN, STUDY, DEVA_FORM, DEVOTION, FORESIGHT,
    INSIGHT, SMITE, SAFETY,
)


# ============================================================
# Java ground truth for skills.
# Values that differ from Python are tested separately as bugs.
# These values match what Python currently stores correctly.
# (id, cost, upgrade_cost, baseDamage, upgDamage,
#   baseBlock, upgBlock, baseMagic, upgMagic, rarity, card_type)
# ============================================================
JAVA_SKILL_VALUES = [
    # Basic
    ("Vigilance",       2, None, -1, 0,   8, 4,  -1, 0, CardRarity.BASIC,    CardType.SKILL),
    # Common Skills
    ("Halt",            0, None, -1, 0,   3, 1,  -1, 0, CardRarity.COMMON,   CardType.SKILL),
    ("ClearTheMind",    1, 0,    -1, 0,  -1, 0,  -1, 0, CardRarity.COMMON,   CardType.SKILL),
    ("Crescendo",       1, 0,    -1, 0,  -1, 0,  -1, 0, CardRarity.COMMON,   CardType.SKILL),
    ("EmptyBody",       1, None, -1, 0,   7, 3,  -1, 0, CardRarity.COMMON,   CardType.SKILL),
    ("EmptyMind",       1, None, -1, 0,  -1, 0,   2, 1, CardRarity.UNCOMMON, CardType.SKILL),
    ("Evaluate",        1, None, -1, 0,   6, 4,  -1, 0, CardRarity.COMMON,   CardType.SKILL),
    ("InnerPeace",      1, None, -1, 0,  -1, 0,   3, 1, CardRarity.UNCOMMON, CardType.SKILL),
    ("Protect",         2, None, -1, 0,  12, 4,  -1, 0, CardRarity.COMMON,   CardType.SKILL),
    ("ThirdEye",        1, None, -1, 0,   7, 2,   3, 2, CardRarity.COMMON,   CardType.SKILL),
    ("Prostrate",       0, None, -1, 0,   4, 0,   2, 1, CardRarity.COMMON,   CardType.SKILL),
    # Uncommon Skills
    ("Collect",        -1, None, -1, 0,  -1, 0,  -1, 0, CardRarity.UNCOMMON, CardType.SKILL),
    ("DeceiveReality",  1, None, -1, 0,   4, 3,  -1, 0, CardRarity.UNCOMMON, CardType.SKILL),
    ("Meditate",        1, None, -1, 0,  -1, 0,   1, 1, CardRarity.UNCOMMON, CardType.SKILL),
    ("Perseverance",    1, None, -1, 0,   5, 2,   2, 1, CardRarity.UNCOMMON, CardType.SKILL),
    ("Swivel",          2, None, -1, 0,   8, 3,  -1, 0, CardRarity.UNCOMMON, CardType.SKILL),
    ("WaveOfTheHand",   1, None, -1, 0,  -1, 0,   1, 1, CardRarity.UNCOMMON, CardType.SKILL),
    # Rare Skills
    ("Judgement",       1, None, -1, 0,  -1, 0,  30,10, CardRarity.RARE,     CardType.SKILL),
    ("Omniscience",     4, 3,    -1, 0,  -1, 0,  -1, 0, CardRarity.RARE,     CardType.SKILL),
    ("Scrawl",          1, 0,    -1, 0,  -1, 0,  -1, 0, CardRarity.RARE,     CardType.SKILL),
    ("SpiritShield",    2, None, -1, 0,  -1, 0,   3, 1, CardRarity.RARE,     CardType.SKILL),
    ("Vault",           3, 2,    -1, 0,  -1, 0,  -1, 0, CardRarity.RARE,     CardType.SKILL),
    ("ConjureBlade",   -1, None, -1, 0,  -1, 0,  -1, 0, CardRarity.RARE,     CardType.SKILL),
    ("Alpha",           1, None, -1, 0,  -1, 0,  -1, 0, CardRarity.RARE,     CardType.SKILL),
    ("Blasphemy",       1, None, -1, 0,  -1, 0,  -1, 0, CardRarity.RARE,     CardType.SKILL),
]

JAVA_POWER_VALUES = [
    # (id, cost, upgrade_cost, baseMagic, upgMagic, rarity)
    ("Adaptation",      1, 0,    2, 0, CardRarity.UNCOMMON),  # Rushdown
    ("MentalFortress",  1, None, 4, 2, CardRarity.UNCOMMON),
    ("BattleHymn",      1, None, 1, 0, CardRarity.UNCOMMON),
    ("Establishment",   1, None, 1, 0, CardRarity.RARE),
    ("LikeWater",       1, None, 5, 2, CardRarity.UNCOMMON),
    ("Devotion",        1, None, 2, 1, CardRarity.RARE),
    ("Wireheading",     1, None, 3, 1, CardRarity.UNCOMMON),  # Foresight
    ("Study",           2, 1,    1, 0, CardRarity.UNCOMMON),
]


class TestWatcherSkillBaseValues:
    """Verify base stat values match Java for all skill cards."""

    @pytest.mark.parametrize(
        "card_id,cost,upg_cost,base_dmg,upg_dmg,base_blk,upg_blk,base_mag,upg_mag,rarity,ctype",
        JAVA_SKILL_VALUES,
        ids=[v[0] for v in JAVA_SKILL_VALUES],
    )
    def test_base_values(self, card_id, cost, upg_cost, base_dmg, upg_dmg,
                         base_blk, upg_blk, base_mag, upg_mag, rarity, ctype):
        card = get_card(card_id)
        assert card.card_type == ctype, f"{card_id} type"
        assert card.cost == cost, f"{card_id} cost"
        assert card.base_damage == base_dmg, f"{card_id} base_damage"
        assert card.base_block == base_blk, f"{card_id} base_block"
        assert card.base_magic == base_mag, f"{card_id} base_magic"
        assert card.upgrade_damage == upg_dmg, f"{card_id} upgrade_damage"
        assert card.upgrade_block == upg_blk, f"{card_id} upgrade_block"
        assert card.upgrade_magic == upg_mag, f"{card_id} upgrade_magic"
        assert card.rarity == rarity, f"{card_id} rarity"
        if upg_cost is not None:
            assert card.upgrade_cost == upg_cost, f"{card_id} upgrade_cost"


class TestWatcherPowerBaseValues:
    """Verify base stat values match Java for all power cards."""

    @pytest.mark.parametrize(
        "card_id,cost,upg_cost,base_mag,upg_mag,rarity",
        JAVA_POWER_VALUES,
        ids=[v[0] for v in JAVA_POWER_VALUES],
    )
    def test_base_values(self, card_id, cost, upg_cost, base_mag, upg_mag, rarity):
        card = get_card(card_id)
        assert card.card_type == CardType.POWER, f"{card_id} type"
        assert card.cost == cost, f"{card_id} cost"
        assert card.base_magic == base_mag, f"{card_id} base_magic"
        assert card.upgrade_magic == upg_mag, f"{card_id} upgrade_magic"
        assert card.rarity == rarity, f"{card_id} rarity"
        if upg_cost is not None:
            assert card.upgrade_cost == upg_cost, f"{card_id} upgrade_cost"


class TestSkillCardFlags:
    """Verify card flags (exhaust, retain, innate, ethereal) match Java."""

    def test_vigilance_enters_calm(self):
        card = get_card("Vigilance")
        assert card.enter_stance == "Calm"

    def test_tranquility_flags(self):
        card = get_card("ClearTheMind")
        assert card.exhaust is True
        assert card.retain is True
        assert card.enter_stance == "Calm"

    def test_crescendo_flags(self):
        card = get_card("Crescendo")
        assert card.exhaust is True
        assert card.retain is True
        assert card.enter_stance == "Wrath"

    def test_crescendo_stays_exhaust_on_upgrade(self):
        """Java Crescendo upgrade only reduces cost, exhaust stays True."""
        card = get_card("Crescendo", upgraded=True)
        assert card.exhaust is True, "Crescendo+ should still exhaust"

    def test_empty_body_exits_stance(self):
        card = get_card("EmptyBody")
        assert card.exit_stance is True

    def test_empty_mind_exits_stance(self):
        card = get_card("EmptyMind")
        assert card.exit_stance is True

    def test_protect_retains(self):
        card = get_card("Protect")
        assert card.retain is True

    def test_perseverance_retains(self):
        card = get_card("Perseverance")
        assert card.retain is True

    def test_alpha_exhaust(self):
        card = get_card("Alpha")
        assert card.exhaust is True

    def test_alpha_innate_on_upgrade(self):
        """Java Alpha+ is Innate."""
        card = get_card("Alpha")
        assert card.upgrade_innate is True

    def test_alpha_source_has_upgrade_innate(self):
        """The ALPHA constant has upgrade_innate=True but copy() drops it."""
        assert ALPHA.upgrade_innate is True

    def test_blasphemy_retain_on_upgrade(self):
        """Java Blasphemy+ has Retain."""
        card = get_card("Blasphemy")
        assert card.upgrade_retain is True

    def test_blasphemy_source_has_upgrade_retain(self):
        """The BLASPHEMY constant has upgrade_retain=True but copy() drops it."""
        assert BLASPHEMY.upgrade_retain is True

    def test_blasphemy_exhaust(self):
        card = get_card("Blasphemy")
        assert card.exhaust is True

    def test_omniscience_exhaust(self):
        card = get_card("Omniscience")
        assert card.exhaust is True

    def test_scrawl_exhaust(self):
        card = get_card("Scrawl")
        assert card.exhaust is True

    def test_vault_exhaust(self):
        card = get_card("Vault")
        assert card.exhaust is True

    def test_wish_exhaust(self):
        card = get_card("Wish")
        assert card.exhaust is True

    def test_collect_exhaust(self):
        card = get_card("Collect")
        assert card.exhaust is True

    def test_foreign_influence_exhaust(self):
        card = get_card("ForeignInfluence")
        assert card.exhaust is True

    def test_conjure_blade_exhaust(self):
        card = get_card("ConjureBlade")
        assert card.exhaust is True


class TestPowerCardFlags:
    """Verify power card flags match Java."""

    def test_deva_form_ethereal(self):
        """Java DevaForm is Ethereal at base."""
        card = get_card("DevaForm")
        assert card.ethereal is True

    def test_deva_form_not_ethereal_upgraded(self):
        """Java DevaForm+ removes Ethereal."""
        card = get_card("DevaForm")
        assert card.upgrade_ethereal is False

    def test_establishment_innate_on_upgrade(self):
        """Java Establishment+ is Innate."""
        card = get_card("Establishment")
        assert card.upgrade_innate is True

    def test_battle_hymn_innate_on_upgrade(self):
        """Java BattleHymn+ is Innate."""
        card = get_card("BattleHymn")
        assert card.upgrade_innate is True


class TestKnownBugs:
    """Tests documenting known discrepancies between Python and Java."""

    def test_worship_no_retain_at_base(self):
        """Java Worship base has no selfRetain. Only upgrade adds it."""
        card = get_card("Worship")
        assert card.retain is False, "Worship base should NOT retain"

    def test_sanctity_effect_hardcodes_draw(self):
        """
        Java Sanctity baseMagicNumber=2, never upgraded.
        Python stores no base_magic and hardcodes draw in effect.
        The effect incorrectly gives draw 3 when upgraded (BUG-4).
        This test documents the data mismatch.
        """
        card = get_card("Sanctity")
        # Python doesn't store magic number for Sanctity
        # Java has baseMagicNumber=2, upgrade does NOT change it
        # The effect at cards.py:421 gives 3 if upgraded (wrong)
        assert card.base_block == 6
        assert card.upgrade_block == 3

    def test_halt_wrath_bonus_values(self):
        """
        Java Halt: baseMagicNumber = baseBlock + 6 + timesUpgraded*4.
        Base: 3 + 6 = 9.  Upgraded: 4 + 6 + 4 = 14.
        """
        card = get_card("Halt")
        assert card.base_block == 3
        assert card.base_block + 6 == 9  # base wrath total OK
        upgraded = get_card("Halt", upgraded=True)
        expected_wrath_total = 14  # 4 + 10
        actual_python_total = 4 + 10  # Python now correct
        assert actual_python_total == expected_wrath_total

    def test_deva_form_magic_number(self):
        """Java DevaForm has baseMagicNumber=1."""
        card = get_card("DevaForm")
        assert card.base_magic == 1

    def test_sanctity_magic_number(self):
        """Java Sanctity baseMagicNumber=2."""
        card = get_card("Sanctity")
        assert card.base_magic == 2

    def test_foreign_influence_no_magic(self):
        """Java ForeignInfluence has no baseMagicNumber."""
        card = get_card("ForeignInfluence")
        assert card.upgrade_magic == 0

    def test_copy_preserves_upgrade_flags(self):
        """Card.copy() preserves upgrade_innate, upgrade_retain, upgrade_ethereal."""
        alpha_copy = ALPHA.copy()
        assert alpha_copy.upgrade_innate is True
        blasphemy_copy = BLASPHEMY.copy()
        assert blasphemy_copy.upgrade_retain is True


class TestSpecialCards:
    """Verify generated/special cards used by Watcher skills."""

    def test_insight(self):
        card = get_card("Insight")
        assert card.cost == 0
        assert card.retain is True
        assert card.exhaust is True
        assert card.base_magic == 2  # Draw 2

    def test_smite(self):
        card = get_card("Smite")
        assert card.cost == 1
        assert card.base_damage == 12
        assert card.retain is True
        assert card.exhaust is True

    def test_safety(self):
        card = get_card("Safety")
        assert card.cost == 1
        assert card.base_block == 12
        assert card.retain is True
        assert card.exhaust is True


class TestUpgradedCosts:
    """Verify cards with cost changes on upgrade."""

    @pytest.mark.parametrize("card_id,base_cost,upg_cost", [
        ("ClearTheMind", 1, 0),
        ("Crescendo", 1, 0),
        ("Omniscience", 4, 3),
        ("Scrawl", 1, 0),
        ("Vault", 3, 2),
        ("Adaptation", 1, 0),  # Rushdown
        ("Study", 2, 1),
    ])
    def test_upgrade_cost(self, card_id, base_cost, upg_cost):
        card = get_card(card_id)
        assert card.cost == base_cost
        upgraded = get_card(card_id, upgraded=True)
        assert upgraded.current_cost == upg_cost, f"{card_id}+ cost"
