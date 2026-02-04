"""
Watcher Card Mechanics Tests

Comprehensive tests for all Watcher card implementations covering:
- Base damage/block values and upgrades
- Energy costs and cost modifications
- Stance-changing cards
- Retain mechanics
- Scry mechanics
- Mantra generation
- Wrath/Calm/Divinity entry/exit effects
- Multi-hit cards
- X-cost cards
- Exhaust cards
- Card draw effects
- Targeting modes
- Card generation/copy effects
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    get_card, get_starting_deck,
    # Basic cards
    STRIKE_W, DEFEND_W, ERUPTION, VIGILANCE, MIRACLE,
    # Common attacks
    BOWLING_BASH, CUT_THROUGH_FATE, EMPTY_FIST, FLURRY_OF_BLOWS,
    FLYING_SLEEVES, FOLLOW_UP, HALT, JUST_LUCKY, PRESSURE_POINTS,
    SASH_WHIP, TRANQUILITY, CRESCENDO, CONSECRATE, CRUSH_JOINTS,
    # Common skills
    EMPTY_BODY, EMPTY_MIND, EVALUATE, INNER_PEACE, PROTECT,
    THIRD_EYE, PROSTRATE,
    # Uncommon attacks
    TANTRUM, FEAR_NO_EVIL, REACH_HEAVEN, SANDS_OF_TIME,
    SIGNATURE_MOVE, TALK_TO_THE_HAND, WALLOP, WEAVE, WHEEL_KICK,
    WINDMILL_STRIKE, CONCLUDE, CARVE_REALITY,
    # Uncommon skills
    COLLECT, DECEIVE_REALITY, FORESIGHT, INDIGNATION, MEDITATE,
    PERSEVERANCE, SANCTITY, SWIVEL, WAVE_OF_THE_HAND, WORSHIP,
    WREATH_OF_FLAME,
    # Uncommon powers
    BATTLE_HYMN, ESTABLISHMENT, LIKE_WATER, MENTAL_FORTRESS,
    NIRVANA, RUSHDOWN, STUDY,
    # Rare attacks
    BRILLIANCE, JUDGMENT, LESSON_LEARNED, RAGNAROK,
    # Rare skills
    ALPHA, BLASPHEMY, CONJURE_BLADE, FOREIGN_INFLUENCE,
    OMNISCIENCE, SCRAWL, SPIRIT_SHIELD, VAULT, WISH,
    # Rare powers
    DEVA_FORM, DEVOTION, FASTING, MASTER_REALITY,
    # Special cards
    INSIGHT, SMITE, SAFETY, THROUGH_VIOLENCE, EXPUNGER, BETA, OMEGA,
    # Registry
    WATCHER_CARDS, ALL_CARDS,
)
from packages.engine.content.stances import StanceID, StanceManager, STANCES


# =============================================================================
# BASIC CARD TESTS
# =============================================================================

class TestBasicCards:
    """Test Watcher's basic starting cards."""

    def test_strike_base_stats(self):
        """Strike: 1 cost, 6 damage."""
        card = get_card("Strike_P")
        assert card.cost == 1
        assert card.damage == 6
        assert card.card_type == CardType.ATTACK
        assert card.rarity == CardRarity.BASIC
        assert card.target == CardTarget.ENEMY

    def test_strike_upgraded(self):
        """Strike+: 1 cost, 9 damage (+3)."""
        card = get_card("Strike_P", upgraded=True)
        assert card.cost == 1
        assert card.damage == 9
        assert card.upgrade_damage == 3

    def test_defend_base_stats(self):
        """Defend: 1 cost, 5 block."""
        card = get_card("Defend_P")
        assert card.cost == 1
        assert card.block == 5
        assert card.card_type == CardType.SKILL
        assert card.target == CardTarget.SELF

    def test_defend_upgraded(self):
        """Defend+: 1 cost, 8 block (+3)."""
        card = get_card("Defend_P", upgraded=True)
        assert card.cost == 1
        assert card.block == 8

    def test_eruption_base_stats(self):
        """Eruption: 2 cost, 9 damage, enters Wrath."""
        card = get_card("Eruption")
        assert card.cost == 2
        assert card.damage == 9
        assert card.enter_stance == "Wrath"
        assert card.card_type == CardType.ATTACK

    def test_eruption_upgraded(self):
        """Eruption+: 1 cost (reduced), 9 damage, enters Wrath."""
        card = get_card("Eruption", upgraded=True)
        assert card.current_cost == 1
        assert card.damage == 9  # Damage unchanged
        assert card.enter_stance == "Wrath"

    def test_vigilance_base_stats(self):
        """Vigilance: 2 cost, 8 block, enters Calm."""
        card = get_card("Vigilance")
        assert card.cost == 2
        assert card.block == 8
        assert card.enter_stance == "Calm"
        assert card.card_type == CardType.SKILL

    def test_vigilance_upgraded(self):
        """Vigilance+: 2 cost, 12 block (+4), enters Calm."""
        card = get_card("Vigilance", upgraded=True)
        assert card.cost == 2
        assert card.block == 12
        assert card.enter_stance == "Calm"

    def test_miracle_base_stats(self):
        """Miracle: 0 cost, retain, exhaust, special colorless."""
        card = get_card("Miracle")
        assert card.cost == 0
        assert card.retain == True
        assert card.exhaust == True
        assert "gain_1_energy" in card.effects
        assert card.rarity == CardRarity.SPECIAL
        assert card.color == CardColor.COLORLESS


# =============================================================================
# STANCE-CHANGING CARD TESTS
# =============================================================================

class TestStanceChangingCards:
    """Test cards that change stances."""

    def test_wrath_entry_cards(self):
        """Cards that enter Wrath stance."""
        wrath_cards = [
            ("Eruption", CardType.ATTACK),
            ("Tantrum", CardType.ATTACK),
            ("Crescendo", CardType.SKILL),
        ]
        for card_id, expected_type in wrath_cards:
            card = get_card(card_id)
            assert card.enter_stance == "Wrath", f"{card_id} should enter Wrath"
            assert card.card_type == expected_type

    def test_calm_entry_cards(self):
        """Cards that enter Calm stance."""
        calm_cards = [
            ("Vigilance", CardType.SKILL),
            ("ClearTheMind", CardType.SKILL),  # Java ID for Tranquility
        ]
        for card_id, expected_type in calm_cards:
            card = get_card(card_id)
            assert card.enter_stance == "Calm", f"{card_id} should enter Calm"
            assert card.card_type == expected_type

    def test_stance_exit_cards(self):
        """Cards that exit current stance."""
        exit_cards = [
            ("EmptyFist", CardType.ATTACK, 9, 14),  # base damage, upgraded damage
            ("EmptyBody", CardType.SKILL, 7, 12),   # base block, upgraded block
            ("EmptyMind", CardType.SKILL, 2, 3),    # base draw, upgraded draw
        ]
        for card_id, expected_type, base_val, upgraded_val in exit_cards:
            card = get_card(card_id)
            assert card.exit_stance == True, f"{card_id} should exit stance"
            assert card.card_type == expected_type

    def test_empty_fist_stats(self):
        """Empty Fist: 1 cost, 9 damage (14 upgraded), exits stance."""
        card = get_card("EmptyFist")
        assert card.cost == 1
        assert card.damage == 9
        assert card.exit_stance == True

        upgraded = get_card("EmptyFist", upgraded=True)
        assert upgraded.damage == 14  # +5

    def test_empty_body_stats(self):
        """Empty Body: 1 cost, 7 block (10 upgraded), exits stance."""
        card = get_card("EmptyBody")
        assert card.cost == 1
        assert card.block == 7
        assert card.exit_stance == True

        upgraded = get_card("EmptyBody", upgraded=True)
        assert upgraded.block == 10  # +3

    def test_empty_mind_stats(self):
        """Empty Mind: 1 cost, draw 2 (3 upgraded), exits stance."""
        card = get_card("EmptyMind")
        assert card.cost == 1
        assert card.magic_number == 2
        assert card.exit_stance == True
        assert "draw_cards" in card.effects

        upgraded = get_card("EmptyMind", upgraded=True)
        assert upgraded.magic_number == 3

    def test_crescendo_mechanics(self):
        """Crescendo: 0 cost, enters Wrath, retain, exhaust."""
        card = get_card("Crescendo")
        assert card.cost == 1  # Base cost 1, upgrade to 0
        assert card.enter_stance == "Wrath"
        assert card.retain == True
        assert card.exhaust == True

    def test_tranquility_mechanics(self):
        """Tranquility: 0 cost, enters Calm, retain, exhaust."""
        card = get_card("ClearTheMind")  # Java ID for Tranquility
        assert card.cost == 1  # Base cost 1, upgrade to 0
        assert card.enter_stance == "Calm"
        assert card.retain == True
        assert card.exhaust == True

    def test_fear_no_evil_conditional(self):
        """Fear No Evil: enters Calm if enemy attacking."""
        card = get_card("FearNoEvil")
        assert card.cost == 1
        assert card.damage == 8
        assert "if_enemy_attacking_enter_calm" in card.effects

        upgraded = get_card("FearNoEvil", upgraded=True)
        assert upgraded.damage == 11  # +3

    def test_inner_peace_dual_effect(self):
        """Inner Peace: enter Calm OR draw 3 if in Calm."""
        card = get_card("InnerPeace")
        assert card.cost == 1
        assert "if_calm_draw_else_calm" in card.effects

    def test_indignation_dual_effect(self):
        """Indignation: enter Wrath OR gain 3 mantra if in Wrath."""
        card = get_card("Indignation")
        assert card.cost == 1
        assert card.magic_number == 3
        assert "if_wrath_gain_mantra_else_wrath" in card.effects

        upgraded = get_card("Indignation", upgraded=True)
        assert upgraded.magic_number == 5  # +2


# =============================================================================
# RETAIN MECHANIC TESTS
# =============================================================================

class TestRetainMechanics:
    """Test cards with the Retain keyword."""

    def test_retain_cards_list(self):
        """All cards that should have retain."""
        retain_cards = [
            "Miracle", "FlyingSleeves", "ClearTheMind", "Crescendo",  # ClearTheMind = Tranquility
            "Protect", "SandsOfTime", "WindmillStrike", "Perseverance",
            # Note: Worship only gains retain on upgrade (not base)
            # Special cards
            "Insight", "Smite", "Safety", "ThroughViolence",
        ]
        for card_id in retain_cards:
            card = get_card(card_id)
            assert card.retain == True, f"{card_id} should have retain"

    def test_protect_stats(self):
        """Protect: 2 cost, 12 block (16 upgraded), retain."""
        card = get_card("Protect")
        assert card.cost == 2
        assert card.block == 12
        assert card.retain == True

        upgraded = get_card("Protect", upgraded=True)
        assert upgraded.block == 16

    def test_flying_sleeves_retain_attack(self):
        """Flying Sleeves: 1 cost, 4x2 damage (6x2 upgraded), retain."""
        card = get_card("FlyingSleeves")
        assert card.cost == 1
        assert card.damage == 4
        assert card.retain == True
        assert "damage_twice" in card.effects  # Hardcoded 2 hits

        upgraded = get_card("FlyingSleeves", upgraded=True)
        assert upgraded.damage == 6

    def test_sands_of_time_cost_reduction(self):
        """Sands of Time: 4 cost, 20 damage, retain, cost reduces each turn."""
        card = get_card("SandsOfTime")
        assert card.cost == 4
        assert card.damage == 20
        assert card.retain == True
        assert "cost_reduces_each_turn" in card.effects

        upgraded = get_card("SandsOfTime", upgraded=True)
        assert upgraded.damage == 26  # +6

    def test_windmill_strike_grows(self):
        """Windmill Strike: gains damage when retained."""
        card = get_card("WindmillStrike")
        assert card.cost == 2
        assert card.damage == 7
        assert card.retain == True
        assert "gain_damage_when_retained_4" in card.effects

        upgraded = get_card("WindmillStrike", upgraded=True)
        assert upgraded.damage == 10  # +3

    def test_perseverance_grows(self):
        """Perseverance: gains block when retained."""
        card = get_card("Perseverance")
        assert card.cost == 1
        assert card.block == 5
        assert card.retain == True
        assert "gains_block_when_retained" in card.effects

        upgraded = get_card("Perseverance", upgraded=True)
        assert upgraded.block == 7  # +2


# =============================================================================
# SCRY MECHANIC TESTS
# =============================================================================

class TestScryMechanics:
    """Test cards with Scry effects."""

    def test_cut_through_fate_scry(self):
        """Cut Through Fate: scry 2, draw 1, deal damage."""
        card = get_card("CutThroughFate")
        assert card.cost == 1
        assert card.damage == 7
        assert "scry" in card.effects
        assert "draw_1" in card.effects

        upgraded = get_card("CutThroughFate", upgraded=True)
        assert upgraded.damage == 9  # +2

    def test_just_lucky_scry(self):
        """Just Lucky: scry 1, gain 2 block, deal damage."""
        card = get_card("JustLucky")
        assert card.cost == 0
        assert card.damage == 3
        assert "scry" in card.effects
        assert "gain_block" in card.effects

        upgraded = get_card("JustLucky", upgraded=True)
        assert upgraded.damage == 4

    def test_third_eye_scry_and_block(self):
        """Third Eye: block + scry."""
        card = get_card("ThirdEye")
        assert card.cost == 1
        assert card.block == 7
        assert card.magic_number == 3  # scry amount
        assert "scry" in card.effects

        upgraded = get_card("ThirdEye", upgraded=True)
        assert upgraded.block == 9
        assert upgraded.magic_number == 5

    def test_weave_on_scry_trigger(self):
        """Weave: plays from discard when scrying."""
        card = get_card("Weave")
        assert card.cost == 0
        assert card.damage == 4
        assert "on_scry_play_from_discard" in card.effects

        upgraded = get_card("Weave", upgraded=True)
        assert upgraded.damage == 6

    def test_foresight_power(self):
        """Foresight: scry each turn."""
        card = get_card("Foresight")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 3
        assert "scry_each_turn" in card.effects

        upgraded = get_card("Foresight", upgraded=True)
        assert upgraded.magic_number == 4

    def test_nirvana_power(self):
        """Nirvana: gain block when scrying."""
        card = get_card("Nirvana")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 3
        assert "on_scry_gain_block" in card.effects

        upgraded = get_card("Nirvana", upgraded=True)
        assert upgraded.magic_number == 4


# =============================================================================
# MANTRA GENERATION TESTS
# =============================================================================

class TestMantraGeneration:
    """Test cards that generate Mantra for Divinity."""

    def test_prostrate_mantra(self):
        """Prostrate: 0 cost, 4 block, 2 mantra."""
        card = get_card("Prostrate")
        assert card.cost == 0
        assert card.block == 4
        assert card.magic_number == 2
        assert "gain_mantra" in card.effects

        upgraded = get_card("Prostrate", upgraded=True)
        assert upgraded.block == 4  # No block upgrade
        assert upgraded.magic_number == 3

    def test_worship_mantra(self):
        """Worship: 2 cost, 5 mantra. Retain only on upgrade (Java behavior)."""
        card = get_card("Worship")
        assert card.cost == 2
        assert card.magic_number == 5
        assert card.retain is False  # Java: no retain at base
        assert card.upgrade_retain is True  # Gains retain on upgrade
        assert "gain_mantra" in card.effects

        upgraded = get_card("Worship", upgraded=True)
        assert upgraded.magic_number == 5  # No magic upgrade; gains Retain instead

    def test_devotion_power(self):
        """Devotion: gain mantra each turn."""
        card = get_card("Devotion")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 2
        assert "gain_mantra_each_turn" in card.effects

        upgraded = get_card("Devotion", upgraded=True)
        assert upgraded.magic_number == 3

    def test_brilliance_mantra_damage(self):
        """Brilliance: damage + mantra gained this combat."""
        card = get_card("Brilliance")
        assert card.cost == 1
        assert card.damage == 12
        assert "damage_plus_mantra_gained" in card.effects

        upgraded = get_card("Brilliance", upgraded=True)
        assert upgraded.damage == 16


# =============================================================================
# STANCE EFFECT INTEGRATION TESTS
# =============================================================================

class TestStanceEffects:
    """Test stance manager integration with cards."""

    def test_wrath_damage_multiplier(self):
        """Wrath stance doubles damage dealt."""
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)

        base_damage = 10
        modified = sm.at_damage_give(base_damage)
        assert modified == 20.0

    def test_wrath_incoming_multiplier(self):
        """Wrath stance doubles damage received."""
        sm = StanceManager()
        sm.change_stance(StanceID.WRATH)

        incoming = 10
        modified = sm.at_damage_receive(incoming)
        assert modified == 20.0

    def test_calm_exit_energy(self):
        """Exiting Calm grants 2 energy."""
        sm = StanceManager()
        sm.change_stance(StanceID.CALM)
        result = sm.exit_stance()
        assert result["energy_gained"] == 2
        assert result["exited"] == StanceID.CALM

    def test_calm_exit_energy_violet_lotus(self):
        """With Violet Lotus, exiting Calm grants 3 energy."""
        sm = StanceManager(has_violet_lotus=True)
        sm.change_stance(StanceID.CALM)
        result = sm.exit_stance()
        assert result["energy_gained"] == 3

    def test_divinity_entry_energy(self):
        """Entering Divinity grants 3 energy."""
        sm = StanceManager()
        result = sm.change_stance(StanceID.DIVINITY)
        assert result["energy_gained"] == 3
        assert result["entered"] == StanceID.DIVINITY

    def test_divinity_damage_multiplier(self):
        """Divinity stance triples damage dealt."""
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)

        base_damage = 10
        modified = sm.at_damage_give(base_damage)
        assert modified == 30.0

    def test_divinity_no_incoming_multiplier(self):
        """Divinity does NOT increase damage received."""
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)

        incoming = 10
        modified = sm.at_damage_receive(incoming)
        assert modified == 10.0  # No change

    def test_divinity_exits_at_turn_start(self):
        """Divinity exits at start of next turn (Java: DivinityStance.atStartOfTurn)."""
        sm = StanceManager()
        sm.change_stance(StanceID.DIVINITY)
        assert sm.current == StanceID.DIVINITY

        # Should NOT exit at end of turn
        result = sm.on_turn_end()
        assert sm.current == StanceID.DIVINITY

        # Should exit at start of next turn
        result = sm.on_turn_start()
        assert result.get("divinity_ended") == True
        assert sm.current == StanceID.NEUTRAL

    def test_mantra_triggers_divinity(self):
        """Gaining 10 mantra enters Divinity."""
        sm = StanceManager()

        # Add 9 mantra - not enough
        result = sm.add_mantra(9)
        assert result["divinity_triggered"] == False
        assert sm.mantra == 9

        # Add 1 more - triggers Divinity
        result = sm.add_mantra(1)
        assert result["divinity_triggered"] == True
        assert sm.current == StanceID.DIVINITY
        assert sm.mantra == 0  # Excess carries over (10 - 10 = 0)

    def test_mantra_excess_carries_over(self):
        """Excess mantra above 10 carries over."""
        sm = StanceManager()

        result = sm.add_mantra(13)
        assert result["divinity_triggered"] == True
        assert sm.mantra == 3  # 13 - 10 = 3

    def test_stance_change_triggers_flurry(self):
        """Flurry of Blows plays from discard on stance change."""
        card = get_card("FlurryOfBlows")
        assert "on_stance_change_play_from_discard" in card.effects

    def test_mental_fortress_power(self):
        """Mental Fortress: gain block on stance change."""
        card = get_card("MentalFortress")
        assert card.card_type == CardType.POWER
        assert card.magic_number == 4
        assert "on_stance_change_gain_block" in card.effects

        upgraded = get_card("MentalFortress", upgraded=True)
        assert upgraded.magic_number == 6

    def test_rushdown_power(self):
        """Rushdown: draw on entering Wrath."""
        card = get_card("Rushdown")
        assert card.card_type == CardType.POWER
        assert card.magic_number == 2
        assert "on_wrath_draw" in card.effects

        upgraded = get_card("Rushdown", upgraded=True)
        assert upgraded.current_cost == 0  # Upgrade reduces cost

    def test_like_water_power(self):
        """Like Water: block at end of turn if in Calm."""
        card = get_card("LikeWater")
        assert card.card_type == CardType.POWER
        assert card.magic_number == 5
        assert "if_calm_end_turn_gain_block" in card.effects

        upgraded = get_card("LikeWater", upgraded=True)
        assert upgraded.magic_number == 7


# =============================================================================
# MULTI-HIT CARD TESTS
# =============================================================================

class TestMultiHitCards:
    """Test cards that hit multiple times."""

    def test_flurry_of_blows(self):
        """Flurry of Blows: 0 cost, 4 damage single hit."""
        card = get_card("FlurryOfBlows")
        assert card.cost == 0
        assert card.damage == 4

        upgraded = get_card("FlurryOfBlows", upgraded=True)
        assert upgraded.damage == 6

    def test_flying_sleeves_multi_hit(self):
        """Flying Sleeves: 4 damage x 2 hits."""
        card = get_card("FlyingSleeves")
        assert card.damage == 4
        assert "damage_twice" in card.effects  # Hardcoded 2 hits

    def test_tantrum_multi_hit(self):
        """Tantrum: 3 damage x 3 hits (x4 upgraded)."""
        card = get_card("Tantrum")
        assert card.cost == 1
        assert card.damage == 3
        assert card.magic_number == 3
        assert card.shuffle_back == True
        assert card.enter_stance == "Wrath"
        assert "damage_x_times" in card.effects

        upgraded = get_card("Tantrum", upgraded=True)
        assert upgraded.magic_number == 4

    def test_ragnarok_multi_hit_all(self):
        """Ragnarok: 5 damage x 5 to random enemies."""
        card = get_card("Ragnarok")
        assert card.cost == 3
        assert card.damage == 5
        assert card.magic_number == 5
        assert card.target == CardTarget.ALL_ENEMY
        assert "damage_random_x_times" in card.effects

        upgraded = get_card("Ragnarok", upgraded=True)
        assert upgraded.damage == 6
        assert upgraded.magic_number == 6


# =============================================================================
# X-COST CARD TESTS
# =============================================================================

class TestXCostCards:
    """Test cards with X energy cost."""

    def test_omniscience_high_cost(self):
        """Omniscience: 4 cost (3 upgraded), play card twice."""
        card = get_card("Omniscience")
        assert card.cost == 4
        assert card.exhaust == True
        assert "play_card_from_draw_twice" in card.effects

        upgraded = get_card("Omniscience", upgraded=True)
        assert upgraded.current_cost == 3

    def test_vault_high_cost(self):
        """Vault: 3 cost (2 upgraded), take extra turn."""
        card = get_card("Vault")
        assert card.cost == 3
        assert card.exhaust == True
        assert "take_extra_turn" in card.effects

        upgraded = get_card("Vault", upgraded=True)
        assert upgraded.current_cost == 2

    def test_conjure_blade_x_cost(self):
        """Conjure Blade: X cost, creates Expunger."""
        card = get_card("ConjureBlade")
        assert card.cost == -1  # X cost
        assert card.exhaust == True
        assert "add_expunger_to_hand" in card.effects

    def test_collect_x_cost(self):
        """Collect: X cost, put X Miracles on draw pile."""
        card = get_card("Collect")
        assert card.cost == -1  # X cost
        assert card.exhaust == True
        assert "put_x_miracles_on_draw" in card.effects


# =============================================================================
# EXHAUST CARD TESTS
# =============================================================================

class TestExhaustCards:
    """Test cards with the Exhaust keyword."""

    def test_exhaust_cards_list(self):
        """All cards that should exhaust."""
        exhaust_cards = [
            "Miracle", "ClearTheMind", "Crescendo",  # ClearTheMind = Tranquility
            "TalkToTheHand", "Collect", "Alpha",
            "ForeignInfluence", "Omniscience",
            "Scrawl", "Vault", "Wish", "LessonLearned",
            # Special cards
            "Insight", "Smite", "Safety", "ThroughViolence", "Beta",
        ]
        for card_id in exhaust_cards:
            card = get_card(card_id)
            assert card.exhaust == True, f"{card_id} should exhaust"

    def test_blasphemy_exhaust(self):
        """Blasphemy exhausts; retain only on upgrade (per Java source)."""
        card = get_card("Blasphemy")
        assert card.exhaust == True
        assert card.retain == False

    def test_talk_to_the_hand(self):
        """Talk to the Hand: 1 cost, 5 damage, exhaust."""
        card = get_card("TalkToTheHand")
        assert card.cost == 1
        assert card.damage == 5
        assert card.magic_number == 2  # block return amount
        assert card.exhaust == True
        assert "apply_block_return" in card.effects

        upgraded = get_card("TalkToTheHand", upgraded=True)
        assert upgraded.damage == 7
        assert upgraded.magic_number == 3

    def test_lesson_learned(self):
        """Lesson Learned: on kill, upgrade random card."""
        card = get_card("LessonLearned")
        assert card.cost == 2
        assert card.damage == 10
        assert card.exhaust == True
        assert "if_fatal_upgrade_random_card" in card.effects

        upgraded = get_card("LessonLearned", upgraded=True)
        assert upgraded.damage == 13

    def test_alpha_beta_omega_chain(self):
        """Alpha -> Beta -> Omega chain."""
        alpha = get_card("Alpha")
        assert alpha.cost == 1
        assert alpha.innate == False  # Innate only on upgrade (Java source)
        assert alpha.exhaust == True
        assert "shuffle_beta_into_draw" in alpha.effects

        alpha_plus = get_card("Alpha", upgraded=True)
        assert alpha_plus.current_cost == 1  # No cost change on upgrade

        beta = get_card("Beta")
        assert beta.cost == 2
        assert beta.exhaust == True
        assert "shuffle_omega_into_draw" in beta.effects

        beta_plus = get_card("Beta", upgraded=True)
        assert beta_plus.current_cost == 1

        omega = get_card("Omega")
        assert omega.cost == 3
        assert omega.card_type == CardType.POWER
        assert "deal_50_damage_end_turn" in omega.effects


# =============================================================================
# CARD DRAW EFFECT TESTS
# =============================================================================

class TestCardDrawEffects:
    """Test cards that draw other cards."""

    def test_empty_mind_draw(self):
        """Empty Mind: draw 2 (3 upgraded)."""
        card = get_card("EmptyMind")
        assert card.magic_number == 2
        assert "draw_cards" in card.effects

        upgraded = get_card("EmptyMind", upgraded=True)
        assert upgraded.magic_number == 3

    def test_wheel_kick_draw(self):
        """Wheel Kick: 15 damage + draw 2."""
        card = get_card("WheelKick")
        assert card.cost == 2
        assert card.damage == 15
        assert "draw_2" in card.effects

        upgraded = get_card("WheelKick", upgraded=True)
        assert upgraded.damage == 20

    def test_scrawl_draw_to_full(self):
        """Scrawl: draw until hand full."""
        card = get_card("Scrawl")
        assert card.cost == 1
        assert card.exhaust == True
        assert "draw_until_hand_full" in card.effects

        upgraded = get_card("Scrawl", upgraded=True)
        assert upgraded.current_cost == 0

    def test_insight_draw(self):
        """Insight (special): 0 cost, draw 2 (3 upgraded), retain, exhaust."""
        card = get_card("Insight")
        assert card.cost == 0
        assert card.magic_number == 2
        assert card.retain == True
        assert card.exhaust == True
        assert card.rarity == CardRarity.SPECIAL
        assert "draw_cards" in card.effects

        upgraded = get_card("Insight", upgraded=True)
        assert upgraded.magic_number == 3

    def test_evaluate_insight_generation(self):
        """Evaluate: add Insight to draw pile."""
        card = get_card("Evaluate")
        assert card.cost == 1
        assert card.block == 6
        assert "add_insight_to_draw" in card.effects

    def test_study_power(self):
        """Study: add Insight at end of turn."""
        card = get_card("Study")
        assert card.cost == 2
        assert card.card_type == CardType.POWER
        assert "add_insight_end_turn" in card.effects

        upgraded = get_card("Study", upgraded=True)
        assert upgraded.current_cost == 1


# =============================================================================
# TARGETING MODE TESTS
# =============================================================================

class TestTargetingModes:
    """Test different card targeting modes."""

    def test_single_enemy_target(self):
        """Cards targeting single enemy."""
        single_target = [
            "Strike_P", "Eruption", "CutThroughFate", "EmptyFist",
            "FlurryOfBlows", "TalkToTheHand", "Judgement",
        ]
        for card_id in single_target:
            card = get_card(card_id)
            assert card.target == CardTarget.ENEMY, f"{card_id} should target ENEMY"

    def test_all_enemy_target(self):
        """Cards targeting all enemies."""
        all_enemy = ["Consecrate", "Conclude", "Ragnarok"]
        for card_id in all_enemy:
            card = get_card(card_id)
            assert card.target == CardTarget.ALL_ENEMY, f"{card_id} should target ALL_ENEMY"

    def test_self_target(self):
        """Cards targeting self."""
        self_target = [
            "Defend_P", "Vigilance", "EmptyBody", "Protect",
            "Prostrate", "SpiritShield",
        ]
        for card_id in self_target:
            card = get_card(card_id)
            assert card.target == CardTarget.SELF, f"{card_id} should target SELF"

    def test_consecrate_all_enemy(self):
        """Consecrate: 0 cost, 5 damage to ALL enemies."""
        card = get_card("Consecrate")
        assert card.cost == 0
        assert card.damage == 5
        assert card.target == CardTarget.ALL_ENEMY

        upgraded = get_card("Consecrate", upgraded=True)
        assert upgraded.damage == 8

    def test_conclude_all_enemy_end_turn(self):
        """Conclude: 12 damage to ALL, ends turn."""
        card = get_card("Conclude")
        assert card.cost == 1
        assert card.damage == 12
        assert card.target == CardTarget.ALL_ENEMY
        assert "end_turn" in card.effects

        upgraded = get_card("Conclude", upgraded=True)
        assert upgraded.damage == 16


# =============================================================================
# CARD GENERATION TESTS
# =============================================================================

class TestCardGeneration:
    """Test cards that generate other cards."""

    def test_meditate_returns_cards(self):
        """Meditate: put cards from discard to hand, enter Calm, end turn."""
        card = get_card("Meditate")
        assert card.cost == 1
        assert card.magic_number == 1  # Number of cards
        assert "put_cards_from_discard_to_hand" in card.effects
        assert "enter_calm" in card.effects
        assert "end_turn" in card.effects

        upgraded = get_card("Meditate", upgraded=True)
        assert upgraded.magic_number == 2

    def test_reach_heaven_generates_through_violence(self):
        """Reach Heaven: adds Through Violence to draw pile."""
        card = get_card("ReachHeaven")
        assert card.cost == 2
        assert card.damage == 10
        assert "add_through_violence_to_draw" in card.effects

        upgraded = get_card("ReachHeaven", upgraded=True)
        assert upgraded.damage == 15

    def test_through_violence_special(self):
        """Through Violence (special): 0 cost, 20 damage, retain, exhaust."""
        card = get_card("ThroughViolence")
        assert card.cost == 0
        assert card.damage == 20
        assert card.retain == True
        assert card.exhaust == True
        assert card.rarity == CardRarity.SPECIAL

        upgraded = get_card("ThroughViolence", upgraded=True)
        assert upgraded.damage == 30

    def test_carve_reality_generates_smite(self):
        """Carve Reality: adds Smite to hand."""
        card = get_card("CarveReality")
        assert card.cost == 1
        assert card.damage == 6
        assert "add_smite_to_hand" in card.effects

        upgraded = get_card("CarveReality", upgraded=True)
        assert upgraded.damage == 10

    def test_smite_special(self):
        """Smite (special): 1 cost, 12 damage, retain, exhaust."""
        card = get_card("Smite")
        assert card.cost == 1
        assert card.damage == 12
        assert card.retain == True
        assert card.exhaust == True
        assert card.rarity == CardRarity.SPECIAL

        upgraded = get_card("Smite", upgraded=True)
        assert upgraded.damage == 16

    def test_deceive_reality_generates_safety(self):
        """Deceive Reality: block + add Safety to hand."""
        card = get_card("DeceiveReality")
        assert card.cost == 1
        assert card.block == 4
        assert "add_safety_to_hand" in card.effects

        upgraded = get_card("DeceiveReality", upgraded=True)
        assert upgraded.block == 7

    def test_safety_special(self):
        """Safety (special): 1 cost, 12 block, retain, exhaust."""
        card = get_card("Safety")
        assert card.cost == 1
        assert card.block == 12
        assert card.retain == True
        assert card.exhaust == True

        upgraded = get_card("Safety", upgraded=True)
        assert upgraded.block == 16

    def test_battle_hymn_generates_smites(self):
        """Battle Hymn: add Smite(s) each turn."""
        card = get_card("BattleHymn")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 1
        assert "add_smite_each_turn" in card.effects

        upgraded = get_card("BattleHymn", upgraded=True)
        assert upgraded.magic_number == 1  # No magic upgrade; upgrade makes innate

    def test_master_reality_upgrades_created(self):
        """Master Reality: created cards are upgraded."""
        card = get_card("MasterReality")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert "created_cards_upgraded" in card.effects

        upgraded = get_card("MasterReality", upgraded=True)
        assert upgraded.current_cost == 0


# =============================================================================
# SPECIAL MECHANIC TESTS
# =============================================================================

class TestSpecialMechanics:
    """Test cards with unique special mechanics."""

    def test_pressure_points_mark(self):
        """Pressure Points: apply Mark, trigger all Marks."""
        card = get_card("PathToVictory")  # Java ID for Pressure Points
        assert card.cost == 1
        assert card.card_type == CardType.SKILL
        assert card.magic_number == 8
        assert "apply_mark" in card.effects
        assert "trigger_all_marks" in card.effects

        upgraded = get_card("PathToVictory", upgraded=True)
        assert upgraded.magic_number == 11

    def test_judgment_execute(self):
        """Judgment: kill enemy if HP below threshold."""
        card = get_card("Judgement")
        assert card.cost == 1
        assert card.magic_number == 30
        assert "if_enemy_hp_below_kill" in card.effects

        upgraded = get_card("Judgement", upgraded=True)
        assert upgraded.magic_number == 40

    def test_signature_move_restriction(self):
        """Signature Move: only usable if only attack in hand."""
        card = get_card("SignatureMove")
        assert card.cost == 2
        assert card.damage == 30
        assert "only_attack_in_hand" in card.effects

        upgraded = get_card("SignatureMove", upgraded=True)
        assert upgraded.damage == 40

    def test_wallop_block_from_damage(self):
        """Wallop: gain block equal to unblocked damage."""
        card = get_card("Wallop")
        assert card.cost == 2
        assert card.damage == 9
        assert "gain_block_equal_unblocked_damage" in card.effects

        upgraded = get_card("Wallop", upgraded=True)
        assert upgraded.damage == 12  # 9 + 3 per Java source

    def test_spirit_shield_block_per_card(self):
        """Spirit Shield: gain block per card in hand."""
        card = get_card("SpiritShield")
        assert card.cost == 2
        assert card.magic_number == 3
        assert "gain_block_per_card_in_hand" in card.effects

        upgraded = get_card("SpiritShield", upgraded=True)
        assert upgraded.magic_number == 4

    def test_blasphemy_divinity_and_death(self):
        """Blasphemy: enter Divinity, die next turn."""
        card = get_card("Blasphemy")
        assert card.cost == 1
        assert card.retain == False  # Retain only on upgrade per Java source
        assert "enter_divinity" in card.effects
        assert "die_next_turn" in card.effects

        upgraded = get_card("Blasphemy", upgraded=True)
        assert upgraded.current_cost == 1  # No cost change on upgrade

    def test_wish_choices(self):
        """Wish: choose between plated armor, strength, or gold."""
        card = get_card("Wish")
        assert card.cost == 3
        assert card.exhaust == True
        assert "choose_plated_armor_or_strength_or_gold" in card.effects

    def test_bowling_bash_per_enemy(self):
        """Bowling Bash: damage per enemy."""
        card = get_card("BowlingBash")
        assert card.cost == 1
        assert card.damage == 7
        assert "damage_per_enemy" in card.effects

        upgraded = get_card("BowlingBash", upgraded=True)
        assert upgraded.damage == 10

    def test_halt_wrath_bonus(self):
        """Halt: extra block in Wrath."""
        card = get_card("Halt")
        assert card.cost == 0
        assert card.block == 3
        assert "if_in_wrath_extra_block_6" in card.effects

        upgraded = get_card("Halt", upgraded=True)
        assert upgraded.block == 4


# =============================================================================
# CONDITIONAL EFFECT TESTS
# =============================================================================

class TestConditionalEffects:
    """Test cards with conditional triggers."""

    def test_follow_up_energy(self):
        """Follow-Up: gain energy if last card was attack."""
        card = get_card("FollowUp")
        assert card.cost == 1
        assert card.damage == 7
        assert "if_last_card_attack_gain_energy" in card.effects

        upgraded = get_card("FollowUp", upgraded=True)
        assert upgraded.damage == 11

    def test_sash_whip_weak(self):
        """Sash Whip: apply Weak if last card was attack."""
        card = get_card("SashWhip")
        assert card.cost == 1
        assert card.damage == 8
        assert "if_last_card_attack_weak" in card.effects

        upgraded = get_card("SashWhip", upgraded=True)
        assert upgraded.damage == 10

    def test_crush_joints_vulnerable(self):
        """Crush Joints: apply Vulnerable if last card was skill."""
        card = get_card("CrushJoints")
        assert card.cost == 1
        assert card.damage == 8
        assert "if_last_card_skill_vulnerable" in card.effects

        upgraded = get_card("CrushJoints", upgraded=True)
        assert upgraded.damage == 10

    def test_sanctity_draw(self):
        """Sanctity: draw if last card was skill."""
        card = get_card("Sanctity")
        assert card.cost == 1
        assert card.block == 6
        assert "if_last_skill_draw_2" in card.effects

        upgraded = get_card("Sanctity", upgraded=True)
        assert upgraded.block == 9


# =============================================================================
# POWER CARD TESTS
# =============================================================================

class TestPowerCards:
    """Test Watcher power cards."""

    def test_establishment_cost_reduction(self):
        """Establishment: retained cards cost less."""
        card = get_card("Establishment")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 1
        assert "retained_cards_cost_less" in card.effects

        upgraded = get_card("Establishment", upgraded=True)
        assert upgraded.current_cost == 1  # No cost reduction; upgrade makes innate

    def test_deva_form_energy_scaling(self):
        """Deva Form: gain energy each turn (stacking)."""
        card = get_card("DevaForm")
        assert card.cost == 3
        assert card.card_type == CardType.POWER
        assert card.ethereal == True
        assert "gain_energy_each_turn_stacking" in card.effects

    def test_fasting_stats(self):
        """Fasting: gain Strength and Dexterity."""
        card = get_card("Fasting2")  # Java ID
        assert card.cost == 2
        assert card.card_type == CardType.POWER
        assert card.magic_number == 3
        assert "gain_strength_and_dex_lose_focus" in card.effects

        upgraded = get_card("Fasting2", upgraded=True)
        assert upgraded.magic_number == 4

    def test_wave_of_the_hand_weak(self):
        """Wave of the Hand: apply Weak when gaining block."""
        card = get_card("WaveOfTheHand")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "block_gain_applies_weak" in card.effects

        upgraded = get_card("WaveOfTheHand", upgraded=True)
        assert upgraded.magic_number == 2

    def test_wreath_of_flame_damage_boost(self):
        """Wreath of Flame: next attack deals bonus damage."""
        card = get_card("WreathOfFlame")
        assert card.cost == 1
        assert card.magic_number == 5
        assert "next_attack_plus_damage" in card.effects

        upgraded = get_card("WreathOfFlame", upgraded=True)
        assert upgraded.magic_number == 8


# =============================================================================
# STARTING DECK TESTS
# =============================================================================

class TestStartingDeck:
    """Test Watcher's starting deck composition."""

    def test_starting_deck_size(self):
        """Starting deck should have 10 cards."""
        deck = get_starting_deck()
        assert len(deck) == 10

    def test_starting_deck_composition(self):
        """Starting deck: 4 Strike, 4 Defend, 1 Eruption, 1 Vigilance."""
        deck = get_starting_deck()
        card_counts = {}
        for card in deck:
            card_counts[card.name] = card_counts.get(card.name, 0) + 1

        assert card_counts.get("Strike", 0) == 4
        assert card_counts.get("Defend", 0) == 4
        assert card_counts.get("Eruption", 0) == 1
        assert card_counts.get("Vigilance", 0) == 1

    def test_starting_cards_not_upgraded(self):
        """Starting cards should not be upgraded."""
        deck = get_starting_deck()
        for card in deck:
            assert card.upgraded == False


# =============================================================================
# CARD COPY TESTS
# =============================================================================

class TestCardCopy:
    """Test card copy functionality."""

    def test_card_copy_independence(self):
        """Copied cards should be independent."""
        original = get_card("Strike_P")
        copy = original.copy()

        copy.upgrade()
        assert copy.upgraded == True
        assert original.upgraded == False

    def test_card_copy_preserves_values(self):
        """Copied cards should have same values."""
        original = get_card("Eruption", upgraded=True)
        copy = original.copy()

        assert copy.damage == original.damage
        assert copy.current_cost == original.current_cost
        assert copy.enter_stance == original.enter_stance
        assert copy.upgraded == original.upgraded


# =============================================================================
# CARD REGISTRY TESTS
# =============================================================================

class TestCardRegistry:
    """Test card registry completeness."""

    def test_watcher_card_count(self):
        """Watcher should have all expected cards in registry."""
        # Basic: 5, Common: 21, Uncommon: 23, Rare: 17, Special: 7
        assert len(WATCHER_CARDS) >= 50  # At least 50 Watcher cards

    def test_get_card_by_id(self):
        """Cards should be retrievable by ID."""
        for card_id in WATCHER_CARDS.keys():
            card = get_card(card_id)
            assert card is not None
            assert card.id == card_id

    def test_invalid_card_raises(self):
        """Getting invalid card should raise."""
        with pytest.raises(ValueError):
            get_card("NonexistentCard")


# =============================================================================
# ETHEREAL CARD TESTS
# =============================================================================

class TestEtherealCards:
    """Test cards with the Ethereal keyword."""

    def test_deva_form_ethereal(self):
        """Deva Form is ethereal."""
        card = get_card("DevaForm")
        assert card.ethereal == True

    def test_ethereal_cards_list(self):
        """Check all ethereal cards."""
        ethereal_cards = ["DevaForm"]
        for card_id in ethereal_cards:
            card = get_card(card_id)
            assert card.ethereal == True, f"{card_id} should be ethereal"


# =============================================================================
# INNATE CARD TESTS
# =============================================================================

class TestInnateCards:
    """Test cards with the Innate keyword."""

    def test_alpha_innate_on_upgrade(self):
        """Alpha is NOT innate at base; innate only on upgrade (Java source)."""
        card = get_card("Alpha")
        assert card.innate == False

    def test_innate_cards_list(self):
        """No base Watcher cards are innate (Alpha gains innate on upgrade only)."""
        # Alpha is innate only when upgraded
        alpha = get_card("Alpha")
        assert alpha.innate == False


# =============================================================================
# SHUFFLE BACK MECHANIC TESTS
# =============================================================================

class TestShuffleBackCards:
    """Test cards that shuffle back into draw pile."""

    def test_tantrum_shuffle_back(self):
        """Tantrum shuffles back into draw pile."""
        card = get_card("Tantrum")
        assert card.shuffle_back == True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
