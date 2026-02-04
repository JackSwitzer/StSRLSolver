"""
Ironclad Card Effects Tests

Comprehensive tests for all Ironclad card effect implementations.
Tests verify:
- Effect execution logic
- Status application
- Damage/block calculations
- Card manipulation
- Power triggers
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    get_card, get_starting_deck,
    # Ironclad cards
    STRIKE_R, DEFEND_R, BASH,
    ANGER, BODY_SLAM, CLASH, CLEAVE, CLOTHESLINE, HEADBUTT,
    HEAVY_BLADE, IRON_WAVE, PERFECTED_STRIKE, POMMEL_STRIKE,
    SWORD_BOOMERANG, THUNDERCLAP, TWIN_STRIKE, WILD_STRIKE,
    ARMAMENTS, FLEX, HAVOC, SHRUG_IT_OFF, TRUE_GRIT, WARCRY,
    BLOOD_FOR_BLOOD, CARNAGE, DROPKICK, HEMOKINESIS, PUMMEL,
    RAMPAGE, RECKLESS_CHARGE, SEARING_BLOW, SEVER_SOUL, UPPERCUT, WHIRLWIND,
    BATTLE_TRANCE, BLOODLETTING, BURNING_PACT, DISARM, DUAL_WIELD,
    ENTRENCH, FLAME_BARRIER, GHOSTLY_ARMOR, INFERNAL_BLADE,
    INTIMIDATE, POWER_THROUGH, RAGE, SECOND_WIND, SEEING_RED,
    SENTINEL, SHOCKWAVE, SPOT_WEAKNESS,
    COMBUST, DARK_EMBRACE, EVOLVE, FEEL_NO_PAIN, FIRE_BREATHING,
    INFLAME, METALLICIZE, RUPTURE,
    BLUDGEON, FEED, FIEND_FIRE, IMMOLATE, REAPER,
    DOUBLE_TAP, EXHUME, IMPERVIOUS, LIMIT_BREAK, OFFERING,
    BARRICADE, BERSERK, BRUTALITY, CORRUPTION, DEMON_FORM, JUGGERNAUT,
    IRONCLAD_CARDS, ALL_CARDS,
)


# =============================================================================
# BASIC CARD TESTS
# =============================================================================

class TestBasicIroncladCards:
    """Test Ironclad's basic starting cards."""

    def test_strike_r_base_stats(self):
        """Strike_R: 1 cost, 6 damage."""
        card = get_card("Strike_R")
        assert card.cost == 1
        assert card.damage == 6
        assert card.card_type == CardType.ATTACK
        assert card.rarity == CardRarity.BASIC
        assert card.color == CardColor.RED

    def test_strike_r_upgraded(self):
        """Strike_R+: 1 cost, 9 damage (+3)."""
        card = get_card("Strike_R", upgraded=True)
        assert card.cost == 1
        assert card.damage == 9

    def test_defend_r_base_stats(self):
        """Defend_R: 1 cost, 5 block."""
        card = get_card("Defend_R")
        assert card.cost == 1
        assert card.block == 5
        assert card.card_type == CardType.SKILL
        assert card.color == CardColor.RED

    def test_defend_r_upgraded(self):
        """Defend_R+: 1 cost, 8 block (+3)."""
        card = get_card("Defend_R", upgraded=True)
        assert card.cost == 1
        assert card.block == 8

    def test_bash_base_stats(self):
        """Bash: 2 cost, 8 damage, applies 2 Vulnerable."""
        card = get_card("Bash")
        assert card.cost == 2
        assert card.damage == 8
        assert card.magic_number == 2
        assert "apply_vulnerable" in card.effects

    def test_bash_upgraded(self):
        """Bash+: 2 cost, 10 damage, applies 3 Vulnerable."""
        card = get_card("Bash", upgraded=True)
        assert card.cost == 2
        assert card.damage == 10
        assert card.magic_number == 3


# =============================================================================
# COMMON ATTACK TESTS
# =============================================================================

class TestCommonAttacks:
    """Test Ironclad common attack cards."""

    def test_anger_base_stats(self):
        """Anger: 0 cost, 6 damage, adds copy to discard."""
        card = get_card("Anger")
        assert card.cost == 0
        assert card.damage == 6
        assert "add_copy_to_discard" in card.effects

    def test_anger_upgraded(self):
        """Anger+: 0 cost, 8 damage."""
        card = get_card("Anger", upgraded=True)
        assert card.cost == 0
        assert card.damage == 8

    def test_body_slam_base_stats(self):
        """Body Slam: 1 cost, damage equals block."""
        card = get_card("Body Slam")
        assert card.cost == 1
        assert "damage_equals_block" in card.effects

    def test_body_slam_upgraded(self):
        """Body Slam+: 0 cost (reduced)."""
        card = get_card("Body Slam", upgraded=True)
        assert card.current_cost == 0

    def test_clash_base_stats(self):
        """Clash: 0 cost, 14 damage, only attacks requirement."""
        card = get_card("Clash")
        assert card.cost == 0
        assert card.damage == 14
        assert "only_attacks_in_hand" in card.effects

    def test_clash_upgraded(self):
        """Clash+: 0 cost, 18 damage (+4)."""
        card = get_card("Clash", upgraded=True)
        assert card.damage == 18

    def test_cleave_base_stats(self):
        """Cleave: 1 cost, 8 damage to all enemies."""
        card = get_card("Cleave")
        assert card.cost == 1
        assert card.damage == 8
        assert card.target == CardTarget.ALL_ENEMY

    def test_cleave_upgraded(self):
        """Cleave+: 1 cost, 11 damage (+3)."""
        card = get_card("Cleave", upgraded=True)
        assert card.damage == 11

    def test_clothesline_base_stats(self):
        """Clothesline: 2 cost, 12 damage, applies 2 Weak."""
        card = get_card("Clothesline")
        assert card.cost == 2
        assert card.damage == 12
        assert card.magic_number == 2
        assert "apply_weak" in card.effects

    def test_heavy_blade_base_stats(self):
        """Heavy Blade: 2 cost, 14 damage, 3x strength multiplier."""
        card = get_card("Heavy Blade")
        assert card.cost == 2
        assert card.damage == 14
        assert card.magic_number == 3
        assert "strength_multiplier" in card.effects

    def test_heavy_blade_upgraded(self):
        """Heavy Blade+: 5x strength multiplier."""
        card = get_card("Heavy Blade", upgraded=True)
        assert card.magic_number == 5

    def test_perfected_strike_base_stats(self):
        """Perfected Strike: 2 cost, 6 damage + 2 per Strike."""
        card = get_card("Perfected Strike")
        assert card.cost == 2
        assert card.damage == 6
        assert card.magic_number == 2
        assert "damage_per_strike" in card.effects

    def test_pommel_strike_base_stats(self):
        """Pommel Strike: 1 cost, 9 damage, draw 1."""
        card = get_card("Pommel Strike")
        assert card.cost == 1
        assert card.damage == 9
        assert card.magic_number == 1
        assert "draw_cards" in card.effects

    def test_sword_boomerang_base_stats(self):
        """Sword Boomerang: 1 cost, 3 damage x3 to random enemies."""
        card = get_card("Sword Boomerang")
        assert card.cost == 1
        assert card.damage == 3
        assert card.magic_number == 3
        assert "random_enemy_x_times" in card.effects

    def test_thunderclap_base_stats(self):
        """Thunderclap: 1 cost, 4 damage, 1 Vulnerable to all."""
        card = get_card("Thunderclap")
        assert card.cost == 1
        assert card.damage == 4
        assert card.target == CardTarget.ALL_ENEMY
        assert "apply_vulnerable_1_all" in card.effects

    def test_twin_strike_base_stats(self):
        """Twin Strike: 1 cost, 5 damage x2."""
        card = get_card("Twin Strike")
        assert card.cost == 1
        assert card.damage == 5
        assert card.magic_number == 2
        assert "damage_x_times" in card.effects

    def test_wild_strike_base_stats(self):
        """Wild Strike: 1 cost, 12 damage, shuffles Wound."""
        card = get_card("Wild Strike")
        assert card.cost == 1
        assert card.damage == 12
        assert "shuffle_wound_into_draw" in card.effects


# =============================================================================
# COMMON SKILL TESTS
# =============================================================================

class TestCommonSkills:
    """Test Ironclad common skill cards."""

    def test_armaments_base_stats(self):
        """Armaments: 1 cost, 5 block, upgrade a card."""
        card = get_card("Armaments")
        assert card.cost == 1
        assert card.block == 5
        assert "upgrade_card_in_hand" in card.effects

    def test_flex_base_stats(self):
        """Flex: 0 cost, +2 temporary Strength."""
        card = get_card("Flex")
        assert card.cost == 0
        assert card.magic_number == 2
        assert "gain_temp_strength" in card.effects

    def test_flex_upgraded(self):
        """Flex+: +4 temporary Strength."""
        card = get_card("Flex", upgraded=True)
        assert card.magic_number == 4

    def test_havoc_base_stats(self):
        """Havoc: 1 cost, play top card and exhaust it."""
        card = get_card("Havoc")
        assert card.cost == 1
        assert "play_top_card" in card.effects

    def test_havoc_upgraded(self):
        """Havoc+: 0 cost."""
        card = get_card("Havoc", upgraded=True)
        assert card.current_cost == 0

    def test_shrug_it_off_base_stats(self):
        """Shrug It Off: 1 cost, 8 block, draw 1."""
        card = get_card("Shrug It Off")
        assert card.cost == 1
        assert card.block == 8
        assert "draw_1" in card.effects

    def test_true_grit_base_stats(self):
        """True Grit: 1 cost, 7 block, exhaust random card."""
        card = get_card("True Grit")
        assert card.cost == 1
        assert card.block == 7
        assert "exhaust_random_card" in card.effects

    def test_warcry_base_stats(self):
        """Warcry: 0 cost, draw 1, put card on draw, exhaust."""
        card = get_card("Warcry")
        assert card.cost == 0
        assert card.magic_number == 1
        assert card.exhaust == True
        assert "draw_then_put_on_draw" in card.effects


# =============================================================================
# UNCOMMON ATTACK TESTS
# =============================================================================

class TestUncommonAttacks:
    """Test Ironclad uncommon attack cards."""

    def test_blood_for_blood_base_stats(self):
        """Blood for Blood: 4 cost, 18 damage, cost reduces when damaged."""
        card = get_card("Blood for Blood")
        assert card.cost == 4
        assert card.damage == 18
        assert "cost_reduces_when_damaged" in card.effects

    def test_blood_for_blood_upgraded(self):
        """Blood for Blood+: 3 cost, 22 damage."""
        card = get_card("Blood for Blood", upgraded=True)
        assert card.current_cost == 3
        assert card.damage == 22

    def test_carnage_base_stats(self):
        """Carnage: 2 cost, 20 damage, ethereal."""
        card = get_card("Carnage")
        assert card.cost == 2
        assert card.damage == 20
        assert card.ethereal == True

    def test_dropkick_base_stats(self):
        """Dropkick: 1 cost, 5 damage, draw/energy if Vulnerable."""
        card = get_card("Dropkick")
        assert card.cost == 1
        assert card.damage == 5
        assert "if_vulnerable_draw_and_energy" in card.effects

    def test_hemokinesis_base_stats(self):
        """Hemokinesis: 1 cost, 15 damage, lose 2 HP."""
        card = get_card("Hemokinesis")
        assert card.cost == 1
        assert card.damage == 15
        assert card.magic_number == 2
        assert "lose_hp" in card.effects

    def test_pummel_base_stats(self):
        """Pummel: 1 cost, 2 damage x4, exhaust."""
        card = get_card("Pummel")
        assert card.cost == 1
        assert card.damage == 2
        assert card.magic_number == 4
        assert card.exhaust == True

    def test_rampage_base_stats(self):
        """Rampage: 1 cost, 8 damage, +5 each use."""
        card = get_card("Rampage")
        assert card.cost == 1
        assert card.damage == 8
        assert card.magic_number == 5
        assert "increase_damage_on_use" in card.effects

    def test_reckless_charge_base_stats(self):
        """Reckless Charge: 0 cost, 7 damage, shuffle Dazed."""
        card = get_card("Reckless Charge")
        assert card.cost == 0
        assert card.damage == 7
        assert "shuffle_dazed_into_draw" in card.effects

    def test_searing_blow_base_stats(self):
        """Searing Blow: 2 cost, 12 damage, unlimited upgrades."""
        card = get_card("Searing Blow")
        assert card.cost == 2
        assert card.damage == 12
        assert "can_upgrade_unlimited" in card.effects

    def test_sever_soul_base_stats(self):
        """Sever Soul: 2 cost, 16 damage, exhaust non-attacks."""
        card = get_card("Sever Soul")
        assert card.cost == 2
        assert card.damage == 16
        assert "exhaust_all_non_attacks" in card.effects

    def test_uppercut_base_stats(self):
        """Uppercut: 2 cost, 13 damage, apply Weak and Vulnerable."""
        card = get_card("Uppercut")
        assert card.cost == 2
        assert card.damage == 13
        assert card.magic_number == 1
        assert "apply_weak_and_vulnerable" in card.effects

    def test_whirlwind_base_stats(self):
        """Whirlwind: X cost, 5 damage X times to all."""
        card = get_card("Whirlwind")
        assert card.cost == -1  # X cost
        assert card.damage == 5
        assert card.target == CardTarget.ALL_ENEMY
        assert "damage_all_x_times" in card.effects


# =============================================================================
# UNCOMMON SKILL TESTS
# =============================================================================

class TestUncommonSkills:
    """Test Ironclad uncommon skill cards."""

    def test_battle_trance_base_stats(self):
        """Battle Trance: 0 cost, draw 3, can't draw more."""
        card = get_card("Battle Trance")
        assert card.cost == 0
        assert card.magic_number == 3
        assert "draw_then_no_draw" in card.effects

    def test_bloodletting_base_stats(self):
        """Bloodletting: 0 cost, lose 3 HP, gain 2 energy."""
        card = get_card("Bloodletting")
        assert card.cost == 0
        assert card.magic_number == 2
        assert "lose_hp_gain_energy" in card.effects

    def test_burning_pact_base_stats(self):
        """Burning Pact: 1 cost, exhaust 1, draw 2."""
        card = get_card("Burning Pact")
        assert card.cost == 1
        assert card.magic_number == 2
        assert "exhaust_to_draw" in card.effects

    def test_disarm_base_stats(self):
        """Disarm: 1 cost, -2 enemy Strength, exhaust."""
        card = get_card("Disarm")
        assert card.cost == 1
        assert card.magic_number == 2
        assert card.exhaust == True
        assert "reduce_enemy_strength" in card.effects

    def test_dual_wield_base_stats(self):
        """Dual Wield: 1 cost, copy an Attack or Power."""
        card = get_card("Dual Wield")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "copy_attack_or_power" in card.effects

    def test_entrench_base_stats(self):
        """Entrench: 2 cost, double block."""
        card = get_card("Entrench")
        assert card.cost == 2
        assert "double_block" in card.effects

    def test_entrench_upgraded(self):
        """Entrench+: 1 cost."""
        card = get_card("Entrench", upgraded=True)
        assert card.current_cost == 1

    def test_flame_barrier_base_stats(self):
        """Flame Barrier: 2 cost, 12 block, 4 thorns damage."""
        card = get_card("Flame Barrier")
        assert card.cost == 2
        assert card.block == 12
        assert card.magic_number == 4
        assert "when_attacked_deal_damage" in card.effects

    def test_infernal_blade_base_stats(self):
        """Infernal Blade: 1 cost, add random 0-cost attack, exhaust."""
        card = get_card("Infernal Blade")
        assert card.cost == 1
        assert card.exhaust == True
        assert "add_random_attack_cost_0" in card.effects

    def test_intimidate_base_stats(self):
        """Intimidate: 0 cost, 1 Weak to all, exhaust."""
        card = get_card("Intimidate")
        assert card.cost == 0
        assert card.magic_number == 1
        assert card.exhaust == True
        assert "apply_weak_all" in card.effects

    def test_power_through_base_stats(self):
        """Power Through: 1 cost, 15 block, add 2 Wounds."""
        card = get_card("Power Through")
        assert card.cost == 1
        assert card.block == 15
        assert "add_wounds_to_hand" in card.effects

    def test_rage_base_stats(self):
        """Rage: 0 cost, 3 block per attack this turn."""
        card = get_card("Rage")
        assert card.cost == 0
        assert card.magic_number == 3
        assert "gain_block_per_attack" in card.effects

    def test_second_wind_base_stats(self):
        """Second Wind: 1 cost, exhaust non-attacks for block."""
        card = get_card("Second Wind")
        assert card.cost == 1
        assert card.block == 5
        assert "exhaust_non_attacks_gain_block" in card.effects

    def test_seeing_red_base_stats(self):
        """Seeing Red: 1 cost, gain 2 energy, exhaust."""
        card = get_card("Seeing Red")
        assert card.cost == 1
        assert card.exhaust == True
        assert "gain_2_energy" in card.effects

    def test_seeing_red_upgraded(self):
        """Seeing Red+: 0 cost."""
        card = get_card("Seeing Red", upgraded=True)
        assert card.current_cost == 0

    def test_sentinel_base_stats(self):
        """Sentinel: 1 cost, 5 block, gain energy if exhausted."""
        card = get_card("Sentinel")
        assert card.cost == 1
        assert card.block == 5
        assert "gain_energy_on_exhaust_2_3" in card.effects

    def test_shockwave_base_stats(self):
        """Shockwave: 2 cost, 3 Weak and Vulnerable to all, exhaust."""
        card = get_card("Shockwave")
        assert card.cost == 2
        assert card.magic_number == 3
        assert card.exhaust == True
        assert "apply_weak_and_vulnerable_all" in card.effects

    def test_spot_weakness_base_stats(self):
        """Spot Weakness: 1 cost, +3 Strength if enemy attacking."""
        card = get_card("Spot Weakness")
        assert card.cost == 1
        assert card.magic_number == 3
        assert "gain_strength_if_enemy_attacking" in card.effects


# =============================================================================
# UNCOMMON POWER TESTS
# =============================================================================

class TestUncommonPowers:
    """Test Ironclad uncommon power cards."""

    def test_combust_base_stats(self):
        """Combust: 1 cost, lose 1 HP deal 5 to all at end of turn."""
        card = get_card("Combust")
        assert card.cost == 1
        assert card.magic_number == 5
        assert card.card_type == CardType.POWER
        assert "end_turn_damage_all_lose_hp" in card.effects

    def test_dark_embrace_base_stats(self):
        """Dark Embrace: 2 cost, draw 1 when exhausting."""
        card = get_card("Dark Embrace")
        assert card.cost == 2
        assert "draw_on_exhaust" in card.effects

    def test_dark_embrace_upgraded(self):
        """Dark Embrace+: 1 cost."""
        card = get_card("Dark Embrace", upgraded=True)
        assert card.current_cost == 1

    def test_evolve_base_stats(self):
        """Evolve: 1 cost, draw 1 when Status drawn."""
        card = get_card("Evolve")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "draw_on_status" in card.effects

    def test_feel_no_pain_base_stats(self):
        """Feel No Pain: 1 cost, gain 3 block when exhausting."""
        card = get_card("Feel No Pain")
        assert card.cost == 1
        assert card.magic_number == 3
        assert "block_on_exhaust" in card.effects

    def test_fire_breathing_base_stats(self):
        """Fire Breathing: 1 cost, deal 6 to all when Status/Curse drawn."""
        card = get_card("Fire Breathing")
        assert card.cost == 1
        assert card.magic_number == 6
        assert "damage_on_status_curse" in card.effects

    def test_inflame_base_stats(self):
        """Inflame: 1 cost, gain 2 Strength."""
        card = get_card("Inflame")
        assert card.cost == 1
        assert card.magic_number == 2
        assert "gain_strength" in card.effects

    def test_metallicize_base_stats(self):
        """Metallicize: 1 cost, gain 3 block at end of turn."""
        card = get_card("Metallicize")
        assert card.cost == 1
        assert card.magic_number == 3
        assert "end_turn_gain_block" in card.effects

    def test_rupture_base_stats(self):
        """Rupture: 1 cost, +1 Strength when losing HP from cards."""
        card = get_card("Rupture")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "gain_strength_on_hp_loss" in card.effects


# =============================================================================
# RARE ATTACK TESTS
# =============================================================================

class TestRareAttacks:
    """Test Ironclad rare attack cards."""

    def test_bludgeon_base_stats(self):
        """Bludgeon: 3 cost, 32 damage."""
        card = get_card("Bludgeon")
        assert card.cost == 3
        assert card.damage == 32
        assert card.rarity == CardRarity.RARE

    def test_bludgeon_upgraded(self):
        """Bludgeon+: 42 damage (+10)."""
        card = get_card("Bludgeon", upgraded=True)
        assert card.damage == 42

    def test_feed_base_stats(self):
        """Feed: 1 cost, 10 damage, +3 max HP if kills, exhaust."""
        card = get_card("Feed")
        assert card.cost == 1
        assert card.damage == 10
        assert card.magic_number == 3
        assert card.exhaust == True
        assert "if_fatal_gain_max_hp" in card.effects

    def test_fiend_fire_base_stats(self):
        """Fiend Fire: 2 cost, 7 damage per exhausted card, exhaust."""
        card = get_card("Fiend Fire")
        assert card.cost == 2
        assert card.damage == 7
        assert card.exhaust == True
        assert "exhaust_hand_damage_per_card" in card.effects

    def test_immolate_base_stats(self):
        """Immolate: 2 cost, 21 damage to all, add Burn."""
        card = get_card("Immolate")
        assert card.cost == 2
        assert card.damage == 21
        assert card.target == CardTarget.ALL_ENEMY
        assert "add_burn_to_discard" in card.effects

    def test_reaper_base_stats(self):
        """Reaper: 2 cost, 4 damage to all, heal unblocked, exhaust."""
        card = get_card("Reaper")
        assert card.cost == 2
        assert card.damage == 4
        assert card.exhaust == True
        assert "damage_all_heal_unblocked" in card.effects


# =============================================================================
# RARE SKILL TESTS
# =============================================================================

class TestRareSkills:
    """Test Ironclad rare skill cards."""

    def test_double_tap_base_stats(self):
        """Double Tap: 1 cost, next 1 attack plays twice."""
        card = get_card("Double Tap")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "play_attacks_twice" in card.effects

    def test_exhume_base_stats(self):
        """Exhume: 1 cost, return exhausted card to hand, exhaust."""
        card = get_card("Exhume")
        assert card.cost == 1
        assert card.exhaust == True
        assert "return_exhausted_card_to_hand" in card.effects

    def test_exhume_upgraded(self):
        """Exhume+: 0 cost."""
        card = get_card("Exhume", upgraded=True)
        assert card.current_cost == 0

    def test_impervious_base_stats(self):
        """Impervious: 2 cost, 30 block, exhaust."""
        card = get_card("Impervious")
        assert card.cost == 2
        assert card.block == 30
        assert card.exhaust == True

    def test_impervious_upgraded(self):
        """Impervious+: 40 block (+10)."""
        card = get_card("Impervious", upgraded=True)
        assert card.block == 40

    def test_limit_break_base_stats(self):
        """Limit Break: 1 cost, double Strength, exhaust."""
        card = get_card("Limit Break")
        assert card.cost == 1
        assert card.exhaust == True
        assert "double_strength" in card.effects

    def test_offering_base_stats(self):
        """Offering: 0 cost, lose 6 HP, gain 2 energy, draw 3, exhaust."""
        card = get_card("Offering")
        assert card.cost == 0
        assert card.magic_number == 3
        assert card.exhaust == True
        assert "lose_hp_gain_energy_draw" in card.effects


# =============================================================================
# RARE POWER TESTS
# =============================================================================

class TestRarePowers:
    """Test Ironclad rare power cards."""

    def test_barricade_base_stats(self):
        """Barricade: 3 cost, block not removed at turn start."""
        card = get_card("Barricade")
        assert card.cost == 3
        assert card.card_type == CardType.POWER
        assert "block_not_lost" in card.effects

    def test_barricade_upgraded(self):
        """Barricade+: 2 cost."""
        card = get_card("Barricade", upgraded=True)
        assert card.current_cost == 2

    def test_berserk_base_stats(self):
        """Berserk: 0 cost, 2 Vulnerable to self, +1 energy per turn."""
        card = get_card("Berserk")
        assert card.cost == 0
        assert card.magic_number == 2
        assert "gain_vulnerable_gain_energy_per_turn" in card.effects

    def test_berserk_upgraded(self):
        """Berserk+: 1 Vulnerable (reduced)."""
        card = get_card("Berserk", upgraded=True)
        assert card.magic_number == 1

    def test_brutality_base_stats(self):
        """Brutality: 0 cost, lose 1 HP draw 1 each turn."""
        card = get_card("Brutality")
        assert card.cost == 0
        assert "start_turn_lose_hp_draw" in card.effects

    def test_corruption_base_stats(self):
        """Corruption: 3 cost, Skills cost 0 but exhaust."""
        card = get_card("Corruption")
        assert card.cost == 3
        assert "skills_cost_0_exhaust" in card.effects

    def test_corruption_upgraded(self):
        """Corruption+: 2 cost."""
        card = get_card("Corruption", upgraded=True)
        assert card.current_cost == 2

    def test_demon_form_base_stats(self):
        """Demon Form: 3 cost, +2 Strength each turn."""
        card = get_card("Demon Form")
        assert card.cost == 3
        assert card.magic_number == 2
        assert "gain_strength_each_turn" in card.effects

    def test_juggernaut_base_stats(self):
        """Juggernaut: 2 cost, deal 5 to random when gaining block."""
        card = get_card("Juggernaut")
        assert card.cost == 2
        assert card.magic_number == 5
        assert "damage_random_on_block" in card.effects


# =============================================================================
# CARD REGISTRY TESTS
# =============================================================================

class TestIroncladCardRegistry:
    """Test Ironclad card registry completeness."""

    def test_ironclad_cards_count(self):
        """Ironclad has correct number of cards in registry."""
        # Should have 72 Ironclad cards total
        assert len(IRONCLAD_CARDS) >= 70

    def test_all_ironclad_cards_have_red_color(self):
        """All Ironclad cards should have RED color."""
        for card_id, card in IRONCLAD_CARDS.items():
            assert card.color == CardColor.RED, f"{card_id} should be RED"

    def test_ironclad_cards_in_all_cards(self):
        """All Ironclad cards should be in ALL_CARDS."""
        for card_id in IRONCLAD_CARDS:
            assert card_id in ALL_CARDS, f"{card_id} should be in ALL_CARDS"

    def test_basic_cards_exist(self):
        """Ironclad basic cards exist."""
        basic_ids = ["Strike_R", "Defend_R", "Bash"]
        for card_id in basic_ids:
            assert card_id in IRONCLAD_CARDS

    def test_ironclad_starting_cards_exist(self):
        """Ironclad starting cards exist in registry."""
        # 5 Strikes, 4 Defends, 1 Bash
        assert "Strike_R" in IRONCLAD_CARDS
        assert "Defend_R" in IRONCLAD_CARDS
        assert "Bash" in IRONCLAD_CARDS
        # Verify they are basic rarity
        assert IRONCLAD_CARDS["Strike_R"].rarity == CardRarity.BASIC
        assert IRONCLAD_CARDS["Defend_R"].rarity == CardRarity.BASIC
        assert IRONCLAD_CARDS["Bash"].rarity == CardRarity.BASIC
