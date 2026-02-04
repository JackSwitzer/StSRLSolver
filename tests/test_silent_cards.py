"""
Silent Card Mechanics Tests

Comprehensive tests for all Silent card implementations covering:
- Base damage/block values and upgrades
- Energy costs and cost modifications
- Poison mechanics
- Shiv mechanics
- Discard triggers (Reflex, Tactician)
- Card selection effects
- X-cost cards
- Turn-based triggers
- Special effects (Intangible, etc.)
"""

import pytest

from packages.engine.content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    get_card,
    # Basic cards
    STRIKE_S, DEFEND_S, NEUTRALIZE, SURVIVOR_S,
    # Common attacks
    BANE, DAGGER_SPRAY, DAGGER_THROW, FLYING_KNEE, POISONED_STAB,
    QUICK_SLASH, SLICE, SNEAKY_STRIKE, SUCKER_PUNCH,
    # Common skills
    ACROBATICS, BACKFLIP, BLADE_DANCE, CLOAK_AND_DAGGER, DEADLY_POISON,
    DEFLECT, DODGE_AND_ROLL, OUTMANEUVER, PIERCING_WAIL, PREPARED,
    # Uncommon attacks
    ALL_OUT_ATTACK, BACKSTAB, CHOKE, DASH_S, ENDLESS_AGONY, EVISCERATE,
    FINISHER, FLECHETTES, HEEL_HOOK, MASTERFUL_STAB, PREDATOR,
    RIDDLE_WITH_HOLES, SKEWER,
    # Uncommon skills
    BLUR, BOUNCING_FLASK, CALCULATED_GAMBLE, CATALYST, CONCENTRATE,
    CRIPPLING_POISON, DISTRACTION, ESCAPE_PLAN, EXPERTISE, LEG_SWEEP,
    REFLEX, SETUP_S, TACTICIAN, TERROR,
    # Uncommon powers
    ACCURACY, CALTROPS, FOOTWORK, INFINITE_BLADES, NOXIOUS_FUMES, WELL_LAID_PLANS,
    # Rare attacks
    DIE_DIE_DIE, GLASS_KNIFE, GRAND_FINALE, UNLOAD,
    # Rare skills
    ADRENALINE, ALCHEMIZE, BULLET_TIME, BURST, CORPSE_EXPLOSION,
    DOPPELGANGER, MALAISE, NIGHTMARE, PHANTASMAL_KILLER, STORM_OF_STEEL,
    # Rare powers
    AFTER_IMAGE, A_THOUSAND_CUTS, ENVENOM, TOOLS_OF_THE_TRADE, WRAITH_FORM,
    # Special
    SHIV,
    # Registry
    SILENT_CARDS, ALL_CARDS,
)


# =============================================================================
# BASIC CARD TESTS
# =============================================================================

class TestBasicCards:
    """Test Silent's basic starting cards."""

    def test_strike_g_base_stats(self):
        """Strike: 1 cost, 6 damage."""
        card = get_card("Strike_G")
        assert card.cost == 1
        assert card.damage == 6
        assert card.card_type == CardType.ATTACK
        assert card.rarity == CardRarity.BASIC
        assert card.color == CardColor.GREEN

    def test_strike_g_upgraded(self):
        """Strike+: 1 cost, 9 damage (+3)."""
        card = get_card("Strike_G", upgraded=True)
        assert card.cost == 1
        assert card.damage == 9

    def test_defend_g_base_stats(self):
        """Defend: 1 cost, 5 block."""
        card = get_card("Defend_G")
        assert card.cost == 1
        assert card.block == 5
        assert card.card_type == CardType.SKILL
        assert card.color == CardColor.GREEN

    def test_defend_g_upgraded(self):
        """Defend+: 1 cost, 8 block (+3)."""
        card = get_card("Defend_G", upgraded=True)
        assert card.cost == 1
        assert card.block == 8

    def test_neutralize_base_stats(self):
        """Neutralize: 0 cost, 3 damage, apply 1 Weak."""
        card = get_card("Neutralize")
        assert card.cost == 0
        assert card.damage == 3
        assert card.magic_number == 1  # Weak amount
        assert "apply_weak" in card.effects
        assert card.rarity == CardRarity.BASIC

    def test_neutralize_upgraded(self):
        """Neutralize+: 0 cost, 4 damage, apply 2 Weak."""
        card = get_card("Neutralize", upgraded=True)
        assert card.cost == 0
        assert card.damage == 4
        assert card.magic_number == 2

    def test_survivor_base_stats(self):
        """Survivor: 1 cost, 8 block, discard 1."""
        card = get_card("Survivor")
        assert card.cost == 1
        assert card.block == 8
        assert "discard_1" in card.effects

    def test_survivor_upgraded(self):
        """Survivor+: 1 cost, 11 block (+3), discard 1."""
        card = get_card("Survivor", upgraded=True)
        assert card.cost == 1
        assert card.block == 11


# =============================================================================
# POISON CARD TESTS
# =============================================================================

class TestPoisonCards:
    """Test cards that apply or interact with Poison."""

    def test_deadly_poison_base_stats(self):
        """Deadly Poison: 1 cost, apply 5 Poison."""
        card = get_card("Deadly Poison")
        assert card.cost == 1
        assert card.magic_number == 5
        assert "apply_poison" in card.effects
        assert card.target == CardTarget.ENEMY

    def test_deadly_poison_upgraded(self):
        """Deadly Poison+: 1 cost, apply 7 Poison."""
        card = get_card("Deadly Poison", upgraded=True)
        assert card.cost == 1
        assert card.magic_number == 7

    def test_poisoned_stab_base_stats(self):
        """Poisoned Stab: 1 cost, 6 damage, apply 3 Poison."""
        card = get_card("Poisoned Stab")
        assert card.cost == 1
        assert card.damage == 6
        assert card.magic_number == 3
        assert "apply_poison" in card.effects

    def test_poisoned_stab_upgraded(self):
        """Poisoned Stab+: 1 cost, 8 damage, apply 4 Poison."""
        card = get_card("Poisoned Stab", upgraded=True)
        assert card.damage == 8
        assert card.magic_number == 4

    def test_bane_base_stats(self):
        """Bane: 1 cost, 7 damage, double if poisoned."""
        card = get_card("Bane")
        assert card.cost == 1
        assert card.damage == 7
        assert "double_damage_if_poisoned" in card.effects

    def test_bane_upgraded(self):
        """Bane+: 1 cost, 10 damage, double if poisoned."""
        card = get_card("Bane", upgraded=True)
        assert card.damage == 10

    def test_catalyst_base_stats(self):
        """Catalyst: 1 cost, double Poison, exhaust."""
        card = get_card("Catalyst")
        assert card.cost == 1
        assert card.exhaust == True
        assert "double_poison" in card.effects

    def test_noxious_fumes_base_stats(self):
        """Noxious Fumes: 1 cost power, apply 2 Poison to all at start of turn."""
        card = get_card("Noxious Fumes")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 2
        assert "apply_poison_all_each_turn" in card.effects

    def test_noxious_fumes_upgraded(self):
        """Noxious Fumes+: 1 cost power, apply 3 Poison to all at start of turn."""
        card = get_card("Noxious Fumes", upgraded=True)
        assert card.magic_number == 3

    def test_bouncing_flask_base_stats(self):
        """Bouncing Flask: 2 cost, apply 3 Poison 3 times to random enemies."""
        card = get_card("Bouncing Flask")
        assert card.cost == 2
        assert card.magic_number == 3
        assert "apply_poison_random_3_times" in card.effects

    def test_bouncing_flask_upgraded(self):
        """Bouncing Flask+: 2 cost, apply 4 Poison 3 times."""
        card = get_card("Bouncing Flask", upgraded=True)
        assert card.magic_number == 4

    def test_crippling_poison_base_stats(self):
        """Crippling Poison: 2 cost, 4 Poison to all, 2 Weak to all, exhaust."""
        card = get_card("Crippling Poison")
        assert card.cost == 2
        assert card.magic_number == 4
        assert card.exhaust == True
        assert "apply_poison_all" in card.effects
        assert "apply_weak_2_all" in card.effects

    def test_crippling_poison_upgraded(self):
        """Crippling Poison+: 2 cost, 7 Poison to all, 2 Weak to all."""
        card = get_card("Crippling Poison", upgraded=True)
        assert card.magic_number == 7

    def test_corpse_explosion_base_stats(self):
        """Corpse Explosion: 2 cost, apply 6 Poison + Corpse Explosion."""
        card = get_card("Corpse Explosion")
        assert card.cost == 2
        assert card.magic_number == 6
        assert "apply_poison" in card.effects
        assert "apply_corpse_explosion" in card.effects

    def test_corpse_explosion_upgraded(self):
        """Corpse Explosion+: 2 cost, apply 9 Poison + Corpse Explosion."""
        card = get_card("Corpse Explosion", upgraded=True)
        assert card.magic_number == 9


# =============================================================================
# SHIV CARD TESTS
# =============================================================================

class TestShivCards:
    """Test cards that create or interact with Shivs."""

    def test_shiv_base_stats(self):
        """Shiv: 0 cost, 4 damage, exhaust."""
        card = get_card("Shiv")
        assert card.cost == 0
        assert card.damage == 4
        assert card.exhaust == True
        assert card.rarity == CardRarity.SPECIAL
        assert card.color == CardColor.COLORLESS

    def test_shiv_upgraded(self):
        """Shiv+: 0 cost, 6 damage, exhaust."""
        card = get_card("Shiv", upgraded=True)
        assert card.damage == 6

    def test_blade_dance_base_stats(self):
        """Blade Dance: 1 cost, add 3 Shivs to hand."""
        card = get_card("Blade Dance")
        assert card.cost == 1
        assert card.magic_number == 3
        assert "add_shivs_to_hand" in card.effects

    def test_blade_dance_upgraded(self):
        """Blade Dance+: 1 cost, add 4 Shivs to hand."""
        card = get_card("Blade Dance", upgraded=True)
        assert card.magic_number == 4

    def test_cloak_and_dagger_base_stats(self):
        """Cloak and Dagger: 1 cost, 6 block, add 1 Shiv to hand."""
        card = get_card("Cloak And Dagger")
        assert card.cost == 1
        assert card.block == 6
        assert card.magic_number == 1
        assert "add_shivs_to_hand" in card.effects

    def test_cloak_and_dagger_upgraded(self):
        """Cloak and Dagger+: 1 cost, 6 block, add 2 Shivs to hand."""
        card = get_card("Cloak And Dagger", upgraded=True)
        assert card.magic_number == 2

    def test_accuracy_base_stats(self):
        """Accuracy: 1 cost power, Shivs deal +4 damage."""
        card = get_card("Accuracy")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 4
        assert "shivs_deal_more_damage" in card.effects

    def test_accuracy_upgraded(self):
        """Accuracy+: 1 cost power, Shivs deal +6 damage."""
        card = get_card("Accuracy", upgraded=True)
        assert card.magic_number == 6

    def test_infinite_blades_base_stats(self):
        """Infinite Blades: 1 cost power, add 1 Shiv at start of each turn."""
        card = get_card("Infinite Blades")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert "add_shiv_each_turn" in card.effects

    def test_storm_of_steel_base_stats(self):
        """Storm of Steel: 1 cost, discard hand, add Shivs equal to discarded."""
        card = get_card("Storm of Steel")
        assert card.cost == 1
        assert "discard_hand" in card.effects
        assert "add_shivs_equal_to_discarded" in card.effects


# =============================================================================
# DISCARD CARD TESTS
# =============================================================================

class TestDiscardCards:
    """Test cards that involve discarding."""

    def test_acrobatics_base_stats(self):
        """Acrobatics: 1 cost, draw 3, discard 1."""
        card = get_card("Acrobatics")
        assert card.cost == 1
        assert card.magic_number == 3  # Draw amount
        assert "draw_x" in card.effects
        assert "discard_1" in card.effects

    def test_acrobatics_upgraded(self):
        """Acrobatics+: 1 cost, draw 4, discard 1."""
        card = get_card("Acrobatics", upgraded=True)
        assert card.magic_number == 4

    def test_prepared_base_stats(self):
        """Prepared: 0 cost, draw 1, discard 1."""
        card = get_card("Prepared")
        assert card.cost == 0
        assert card.magic_number == 1
        assert "draw_x" in card.effects
        assert "discard_x" in card.effects

    def test_prepared_upgraded(self):
        """Prepared+: 0 cost, draw 2, discard 2."""
        card = get_card("Prepared", upgraded=True)
        assert card.magic_number == 2

    def test_calculated_gamble_base_stats(self):
        """Calculated Gamble: 0 cost, discard hand, draw same, exhaust."""
        card = get_card("Calculated Gamble")
        assert card.cost == 0
        assert card.exhaust == True
        assert "discard_hand_draw_same" in card.effects

    def test_reflex_base_stats(self):
        """Reflex: Unplayable, draw 2 when discarded."""
        card = get_card("Reflex")
        assert card.cost == -2  # Unplayable
        assert card.magic_number == 2
        assert "unplayable" in card.effects
        assert "when_discarded_draw" in card.effects

    def test_reflex_upgraded(self):
        """Reflex+: Unplayable, draw 3 when discarded."""
        card = get_card("Reflex", upgraded=True)
        assert card.magic_number == 3

    def test_tactician_base_stats(self):
        """Tactician: Unplayable, gain 1 energy when discarded."""
        card = get_card("Tactician")
        assert card.cost == -2  # Unplayable
        assert card.magic_number == 1
        assert "unplayable" in card.effects
        assert "when_discarded_gain_energy" in card.effects

    def test_tactician_upgraded(self):
        """Tactician+: Unplayable, gain 2 energy when discarded."""
        card = get_card("Tactician", upgraded=True)
        assert card.magic_number == 2

    def test_concentrate_base_stats(self):
        """Concentrate: 0 cost, discard 3, gain 2 energy."""
        card = get_card("Concentrate")
        assert card.cost == 0
        assert card.magic_number == 3  # Discard amount
        assert "discard_x" in card.effects
        assert "gain_energy_2" in card.effects

    def test_concentrate_upgraded(self):
        """Concentrate+: 0 cost, discard 2, gain 2 energy."""
        card = get_card("Concentrate", upgraded=True)
        assert card.magic_number == 2  # Reduced on upgrade


# =============================================================================
# X-COST CARD TESTS
# =============================================================================

class TestXCostCards:
    """Test X-cost cards."""

    def test_skewer_base_stats(self):
        """Skewer: X cost, deal 7 damage X times."""
        card = get_card("Skewer")
        assert card.cost == -1  # X cost
        assert card.damage == 7
        assert "damage_x_times_energy" in card.effects

    def test_skewer_upgraded(self):
        """Skewer+: X cost, deal 10 damage X times."""
        card = get_card("Skewer", upgraded=True)
        assert card.damage == 10

    def test_malaise_base_stats(self):
        """Malaise: X cost, apply X Weak and X Strength down, exhaust."""
        card = get_card("Malaise")
        assert card.cost == -1  # X cost
        assert card.exhaust == True
        assert "apply_weak_x" in card.effects
        assert "apply_strength_down_x" in card.effects

    def test_doppelganger_base_stats(self):
        """Doppelganger: X cost, draw X and gain X energy next turn, exhaust."""
        card = get_card("Doppelganger")
        assert card.cost == -1  # X cost
        assert card.exhaust == True
        assert "draw_x_next_turn" in card.effects
        assert "gain_x_energy_next_turn" in card.effects


# =============================================================================
# POWER CARD TESTS
# =============================================================================

class TestPowerCards:
    """Test Silent power cards."""

    def test_footwork_base_stats(self):
        """Footwork: 1 cost power, gain 2 Dexterity."""
        card = get_card("Footwork")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 2
        assert "gain_dexterity" in card.effects

    def test_footwork_upgraded(self):
        """Footwork+: 1 cost power, gain 3 Dexterity."""
        card = get_card("Footwork", upgraded=True)
        assert card.magic_number == 3

    def test_caltrops_base_stats(self):
        """Caltrops: 1 cost power, gain 3 Thorns."""
        card = get_card("Caltrops")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert card.magic_number == 3
        assert "gain_thorns" in card.effects

    def test_caltrops_upgraded(self):
        """Caltrops+: 1 cost power, gain 5 Thorns."""
        card = get_card("Caltrops", upgraded=True)
        assert card.magic_number == 5

    def test_after_image_base_stats(self):
        """After Image: 1 cost power, gain 1 block per card played."""
        card = get_card("After Image")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert "gain_1_block_per_card_played" in card.effects

    def test_a_thousand_cuts_base_stats(self):
        """A Thousand Cuts: 2 cost power, deal 1 damage to all per card."""
        card = get_card("A Thousand Cuts")
        assert card.cost == 2
        assert card.card_type == CardType.POWER
        assert card.magic_number == 1
        assert "deal_damage_per_card_played" in card.effects

    def test_a_thousand_cuts_upgraded(self):
        """A Thousand Cuts+: 2 cost power, deal 2 damage to all per card."""
        card = get_card("A Thousand Cuts", upgraded=True)
        assert card.magic_number == 2

    def test_envenom_base_stats(self):
        """Envenom: 2 cost power, attacks apply Poison."""
        card = get_card("Envenom")
        assert card.cost == 2
        assert card.card_type == CardType.POWER
        assert "attacks_apply_poison" in card.effects

    def test_envenom_upgraded(self):
        """Envenom+: 1 cost power, attacks apply Poison."""
        card = get_card("Envenom", upgraded=True)
        assert card.current_cost == 1

    def test_tools_of_the_trade_base_stats(self):
        """Tools of the Trade: 1 cost power, draw 1, discard 1 at start of turn."""
        card = get_card("Tools of the Trade")
        assert card.cost == 1
        assert card.card_type == CardType.POWER
        assert "draw_1_discard_1_each_turn" in card.effects

    def test_tools_of_the_trade_upgraded(self):
        """Tools of the Trade+: 0 cost power."""
        card = get_card("Tools of the Trade", upgraded=True)
        assert card.current_cost == 0

    def test_wraith_form_base_stats(self):
        """Wraith Form: 3 cost power, gain 2 Intangible, lose 1 Dex each turn."""
        card = get_card("Wraith Form v2")
        assert card.cost == 3
        assert card.card_type == CardType.POWER
        assert card.magic_number == 2
        assert "gain_intangible" in card.effects
        assert "lose_1_dexterity_each_turn" in card.effects

    def test_wraith_form_upgraded(self):
        """Wraith Form+: 3 cost power, gain 3 Intangible."""
        card = get_card("Wraith Form v2", upgraded=True)
        assert card.magic_number == 3


# =============================================================================
# SPECIAL EFFECT CARD TESTS
# =============================================================================

class TestSpecialEffectCards:
    """Test cards with unique mechanics."""

    def test_blur_base_stats(self):
        """Blur: 1 cost, 5 block, block not removed next turn."""
        card = get_card("Blur")
        assert card.cost == 1
        assert card.block == 5
        assert "block_not_removed_next_turn" in card.effects

    def test_blur_upgraded(self):
        """Blur+: 1 cost, 8 block."""
        card = get_card("Blur", upgraded=True)
        assert card.block == 8

    def test_bullet_time_base_stats(self):
        """Bullet Time: 3 cost, no draw this turn, cards cost 0 this turn."""
        card = get_card("Bullet Time")
        assert card.cost == 3
        assert "no_draw_this_turn" in card.effects
        assert "cards_cost_0_this_turn" in card.effects

    def test_bullet_time_upgraded(self):
        """Bullet Time+: 2 cost."""
        card = get_card("Bullet Time", upgraded=True)
        assert card.current_cost == 2

    def test_burst_base_stats(self):
        """Burst: 1 cost, next skill is played twice."""
        card = get_card("Burst")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "double_next_skills" in card.effects

    def test_burst_upgraded(self):
        """Burst+: 1 cost, next 2 skills are played twice."""
        card = get_card("Burst", upgraded=True)
        assert card.magic_number == 2

    def test_phantasmal_killer_base_stats(self):
        """Phantasmal Killer: 1 cost, double damage next turn."""
        card = get_card("Phantasmal Killer")
        assert card.cost == 1
        assert "double_damage_next_turn" in card.effects

    def test_phantasmal_killer_upgraded(self):
        """Phantasmal Killer+: 0 cost."""
        card = get_card("Phantasmal Killer", upgraded=True)
        assert card.current_cost == 0

    def test_grand_finale_base_stats(self):
        """Grand Finale: 0 cost, 50 damage to all, only if draw pile empty."""
        card = get_card("Grand Finale")
        assert card.cost == 0
        assert card.damage == 50
        assert card.target == CardTarget.ALL_ENEMY
        assert "only_playable_if_draw_pile_empty" in card.effects

    def test_grand_finale_upgraded(self):
        """Grand Finale+: 0 cost, 60 damage to all."""
        card = get_card("Grand Finale", upgraded=True)
        assert card.damage == 60

    def test_heel_hook_base_stats(self):
        """Heel Hook: 1 cost, 5 damage, if target weak gain energy and draw."""
        card = get_card("Heel Hook")
        assert card.cost == 1
        assert card.damage == 5
        assert "if_target_weak_gain_energy_draw" in card.effects

    def test_heel_hook_upgraded(self):
        """Heel Hook+: 1 cost, 8 damage."""
        card = get_card("Heel Hook", upgraded=True)
        assert card.damage == 8

    def test_finisher_base_stats(self):
        """Finisher: 1 cost, deal 6 damage per attack played this turn."""
        card = get_card("Finisher")
        assert card.cost == 1
        assert card.damage == 6
        assert "damage_per_attack_this_turn" in card.effects

    def test_finisher_upgraded(self):
        """Finisher+: 1 cost, deal 8 damage per attack."""
        card = get_card("Finisher", upgraded=True)
        assert card.damage == 8

    def test_flechettes_base_stats(self):
        """Flechettes: 1 cost, deal 4 damage per skill in hand."""
        card = get_card("Flechettes")
        assert card.cost == 1
        assert card.damage == 4
        assert "damage_per_skill_in_hand" in card.effects

    def test_flechettes_upgraded(self):
        """Flechettes+: 1 cost, deal 6 damage per skill."""
        card = get_card("Flechettes", upgraded=True)
        assert card.damage == 6


# =============================================================================
# REGISTRY TESTS
# =============================================================================

class TestSilentCardRegistry:
    """Test Silent card registry."""

    def test_silent_cards_exist(self):
        """Verify all Silent cards are in the registry."""
        assert len(SILENT_CARDS) > 40

        # Check some key cards exist
        assert "Strike_G" in SILENT_CARDS
        assert "Defend_G" in SILENT_CARDS
        assert "Neutralize" in SILENT_CARDS
        assert "Blade Dance" in SILENT_CARDS
        assert "Deadly Poison" in SILENT_CARDS
        assert "Accuracy" in SILENT_CARDS
        assert "Noxious Fumes" in SILENT_CARDS
        assert "Wraith Form v2" in SILENT_CARDS

    def test_all_silent_cards_green(self):
        """Verify all Silent cards are green (except Shiv which is colorless)."""
        for card_id, card in SILENT_CARDS.items():
            if card_id == "Shiv":
                assert card.color == CardColor.COLORLESS
            else:
                assert card.color == CardColor.GREEN, f"{card_id} should be GREEN"

    def test_silent_cards_in_all_cards(self):
        """Verify Silent cards are in ALL_CARDS."""
        for card_id in SILENT_CARDS:
            assert card_id in ALL_CARDS, f"{card_id} should be in ALL_CARDS"


# =============================================================================
# DAMAGE/BLOCK TESTS
# =============================================================================

class TestDamageBlockStats:
    """Test damage and block values for Silent cards."""

    def test_dagger_spray_stats(self):
        """Dagger Spray: 4 damage x2 to all enemies."""
        card = get_card("Dagger Spray")
        assert card.damage == 4
        assert card.magic_number == 2
        assert card.target == CardTarget.ALL_ENEMY

    def test_dagger_spray_upgraded(self):
        """Dagger Spray+: 6 damage x2."""
        card = get_card("Dagger Spray", upgraded=True)
        assert card.damage == 6

    def test_riddle_with_holes_stats(self):
        """Riddle with Holes: 3 damage x5."""
        card = get_card("Riddle With Holes")
        assert card.damage == 3
        assert card.magic_number == 5

    def test_riddle_with_holes_upgraded(self):
        """Riddle with Holes+: 4 damage x5."""
        card = get_card("Riddle With Holes", upgraded=True)
        assert card.damage == 4

    def test_die_die_die_stats(self):
        """Die Die Die: 13 damage to all, exhaust."""
        card = get_card("Die Die Die")
        assert card.damage == 13
        assert card.target == CardTarget.ALL_ENEMY
        assert card.exhaust == True

    def test_die_die_die_upgraded(self):
        """Die Die Die+: 17 damage to all."""
        card = get_card("Die Die Die", upgraded=True)
        assert card.damage == 17

    def test_glass_knife_stats(self):
        """Glass Knife: 8 damage x2, loses 2 damage each play."""
        card = get_card("Glass Knife")
        assert card.damage == 8
        assert card.magic_number == 2
        assert "reduce_damage_by_2" in card.effects

    def test_glass_knife_upgraded(self):
        """Glass Knife+: 12 damage x2."""
        card = get_card("Glass Knife", upgraded=True)
        assert card.damage == 12


# =============================================================================
# ENERGY/DRAW TESTS
# =============================================================================

class TestEnergyDrawCards:
    """Test cards that affect energy and draw."""

    def test_adrenaline_base_stats(self):
        """Adrenaline: 0 cost, gain 1 energy, draw 2, exhaust."""
        card = get_card("Adrenaline")
        assert card.cost == 0
        assert card.magic_number == 1
        assert card.exhaust == True
        assert "gain_energy" in card.effects
        assert "draw_2" in card.effects

    def test_adrenaline_upgraded(self):
        """Adrenaline+: 0 cost, gain 2 energy, draw 2."""
        card = get_card("Adrenaline", upgraded=True)
        assert card.magic_number == 2

    def test_backflip_base_stats(self):
        """Backflip: 1 cost, 5 block, draw 2."""
        card = get_card("Backflip")
        assert card.cost == 1
        assert card.block == 5
        assert "draw_2" in card.effects

    def test_backflip_upgraded(self):
        """Backflip+: 1 cost, 8 block, draw 2."""
        card = get_card("Backflip", upgraded=True)
        assert card.block == 8

    def test_outmaneuver_base_stats(self):
        """Outmaneuver: 1 cost, gain 2 energy next turn."""
        card = get_card("Outmaneuver")
        assert card.cost == 1
        assert card.magic_number == 2
        assert "gain_energy_next_turn" in card.effects

    def test_outmaneuver_upgraded(self):
        """Outmaneuver+: 1 cost, gain 3 energy next turn."""
        card = get_card("Outmaneuver", upgraded=True)
        assert card.magic_number == 3

    def test_expertise_base_stats(self):
        """Expertise: 1 cost, draw until you have 6 cards."""
        card = get_card("Expertise")
        assert card.cost == 1
        assert card.magic_number == 6
        assert "draw_to_x_cards" in card.effects

    def test_expertise_upgraded(self):
        """Expertise+: 1 cost, draw until you have 7 cards."""
        card = get_card("Expertise", upgraded=True)
        assert card.magic_number == 7
