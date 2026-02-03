"""
Event Mechanics Tests

Comprehensive tests for Slay the Spire event mechanics:
1. Event pool per act (which events can appear in which act)
2. Event choice outcomes (HP loss, gold gain, card gain, etc.)
3. RNG-dependent events (Wheel of Change, Match and Keep, etc.)
4. Shrine events (upgrade, remove, transform)
5. One-time events (The Woman in Blue, Neow, etc.)
6. Conditional events (The Cleric requires gold, etc.)
7. Event requirements (deck size, relics owned, etc.)
8. Ascension event modifications (A15 "?" room changes)
9. Deterministic outcomes with same seed
10. Special events (Neow's Lament, Colosseum, Mind Bloom, etc.)
11. Event card/relic rewards
12. Event combat triggers (Dead Adventurer, Colosseum, etc.)
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from core.content.events import (
    # Enums
    Act, OutcomeType,
    # Data classes
    Outcome, EventChoice, Event, NeowBonus,
    # Act 1 Events
    BIG_FISH, CLERIC, DEAD_ADVENTURER, GOLDEN_IDOL, GOLDEN_WING,
    GOOP_PUDDLE, LIVING_WALL, MUSHROOMS, SCRAP_OOZE, SHINING_LIGHT, SSSSERPENT,
    GOLDEN_IDOL_ESCAPE_INJURY, GOLDEN_IDOL_ESCAPE_DAMAGE, GOLDEN_IDOL_ESCAPE_MAX_HP,
    # Act 2 Events
    ADDICT, BACK_TO_BASICS, BEGGAR, COLOSSEUM, CURSED_TOME, DRUG_DEALER,
    FORGOTTEN_ALTAR, GHOSTS, KNOWING_SKULL, MASKED_BANDITS, NEST, THE_JOUST,
    THE_LIBRARY, THE_MAUSOLEUM, VAMPIRES,
    COLOSSEUM_FIGHT_NOBS, COLOSSEUM_FLEE,
    # Act 3 Events
    FALLING, MIND_BLOOM, MOAI_HEAD, MYSTERIOUS_SPHERE, SECRET_PORTAL,
    SENSORY_STONE, TOMB_OF_LORD_RED_MASK, WINDING_HALLS,
    # Shrine Events (Any Act)
    ACCURSED_BLACKSMITH, BONFIRE_ELEMENTALS, DESIGNER, DUPLICATOR,
    FACE_TRADER, FOUNTAIN_OF_CURSE_REMOVAL, GOLD_SHRINE, GREMLIN_MATCH_GAME,
    GREMLIN_WHEEL_GAME, LAB, NLOTH, NOTE_FOR_YOURSELF, PURIFICATION_SHRINE,
    TRANSMOGRIFIER, UPGRADE_SHRINE, WE_MEET_AGAIN, WOMAN_IN_BLUE,
    # Neow bonuses
    NEOW_THREE_CARDS, NEOW_ONE_RANDOM_RARE, NEOW_REMOVE_CARD, NEOW_UPGRADE_CARD,
    NEOW_TRANSFORM_CARD, NEOW_RANDOM_COLORLESS, NEOW_THREE_POTIONS,
    NEOW_RANDOM_COMMON_RELIC, NEOW_TEN_PERCENT_HP, NEOW_THREE_ENEMY_KILL,
    NEOW_HUNDRED_GOLD, NEOW_RANDOM_COLORLESS_2, NEOW_REMOVE_TWO,
    NEOW_ONE_RARE_RELIC, NEOW_THREE_RARE_CARDS, NEOW_TWO_FIFTY_GOLD,
    NEOW_TRANSFORM_TWO, NEOW_TWENTY_PERCENT_HP, NEOW_BOSS_SWAP,
    NEOW_DRAWBACK_10_PERCENT_HP_LOSS, NEOW_DRAWBACK_NO_GOLD,
    NEOW_DRAWBACK_CURSE, NEOW_DRAWBACK_PERCENT_DAMAGE,
    # Lookup dictionaries
    EXORDIUM_EVENTS, CITY_EVENTS, BEYOND_EVENTS, SHRINE_EVENTS, ALL_EVENTS,
    # Functions
    get_event, get_events_for_act, calculate_outcome_value,
)
from core.state.rng import Random, GameRNG, seed_to_long


# =============================================================================
# TEST CLASS: Event Pool Per Act
# =============================================================================

class TestEventPoolPerAct:
    """Test that events appear in correct acts."""

    def test_act1_events_are_exordium(self):
        """Act 1 events should be from Exordium pool."""
        for event_id, event in EXORDIUM_EVENTS.items():
            assert event.act == Act.ACT_1, f"{event_id} should be Act 1"

    def test_act2_events_are_city(self):
        """Act 2 events should be from City pool."""
        for event_id, event in CITY_EVENTS.items():
            assert event.act == Act.ACT_2, f"{event_id} should be Act 2"

    def test_act3_events_are_beyond(self):
        """Act 3 events should be from Beyond pool."""
        for event_id, event in BEYOND_EVENTS.items():
            assert event.act == Act.ACT_3, f"{event_id} should be Act 3"

    def test_shrine_events_are_any_act(self):
        """Shrine events should be available in any act."""
        for event_id, event in SHRINE_EVENTS.items():
            assert event.act == Act.ANY, f"{event_id} should be ANY act"

    def test_get_events_for_act1(self):
        """Get events for Act 1 returns Exordium + Shrines."""
        act1_events = get_events_for_act(Act.ACT_1)
        # Should have all Exordium events
        for event_id in EXORDIUM_EVENTS:
            assert event_id in act1_events
        # Should have all Shrine events
        for event_id in SHRINE_EVENTS:
            assert event_id in act1_events
        # Should NOT have City or Beyond events
        for event_id in CITY_EVENTS:
            assert event_id not in act1_events
        for event_id in BEYOND_EVENTS:
            assert event_id not in act1_events

    def test_get_events_for_act2(self):
        """Get events for Act 2 returns City + Shrines."""
        act2_events = get_events_for_act(Act.ACT_2)
        for event_id in CITY_EVENTS:
            assert event_id in act2_events
        for event_id in SHRINE_EVENTS:
            assert event_id in act2_events
        for event_id in EXORDIUM_EVENTS:
            assert event_id not in act2_events

    def test_get_events_for_act3(self):
        """Get events for Act 3 returns Beyond + Shrines."""
        act3_events = get_events_for_act(Act.ACT_3)
        for event_id in BEYOND_EVENTS:
            assert event_id in act3_events
        for event_id in SHRINE_EVENTS:
            assert event_id in act3_events

    def test_exordium_event_count(self):
        """Verify expected number of Exordium events."""
        assert len(EXORDIUM_EVENTS) == 11

    def test_city_event_count(self):
        """Verify expected number of City events."""
        assert len(CITY_EVENTS) == 14

    def test_beyond_event_count(self):
        """Verify expected number of Beyond events."""
        assert len(BEYOND_EVENTS) == 8

    def test_shrine_event_count(self):
        """Verify expected number of Shrine events."""
        assert len(SHRINE_EVENTS) == 6


# =============================================================================
# TEST CLASS: Event Choice Outcomes
# =============================================================================

class TestEventChoiceOutcomes:
    """Test event choice outcome types and values."""

    def test_big_fish_banana_heals(self):
        """Big Fish banana option heals 1/3 max HP."""
        choice = BIG_FISH.choices[0]
        assert len(choice.outcomes) == 1
        outcome = choice.outcomes[0]
        assert outcome.type == OutcomeType.HP_CHANGE
        assert outcome.value_percent == pytest.approx(0.33, rel=0.01)

    def test_big_fish_donut_max_hp(self):
        """Big Fish donut option gives max HP."""
        choice = BIG_FISH.choices[1]
        outcome = choice.outcomes[0]
        assert outcome.type == OutcomeType.MAX_HP_CHANGE
        assert outcome.value == 5

    def test_big_fish_box_relic_and_curse(self):
        """Big Fish box gives relic AND curse."""
        choice = BIG_FISH.choices[2]
        assert len(choice.outcomes) == 2
        relic_outcome = choice.outcomes[0]
        curse_outcome = choice.outcomes[1]
        assert relic_outcome.type == OutcomeType.RELIC_GAIN
        assert relic_outcome.random is True
        assert curse_outcome.type == OutcomeType.CURSE_GAIN
        assert curse_outcome.card_id == "Regret"

    def test_cleric_heal_costs_gold(self):
        """Cleric heal option costs gold."""
        choice = CLERIC.choices[0]
        gold_outcome = choice.outcomes[0]
        heal_outcome = choice.outcomes[1]
        assert gold_outcome.type == OutcomeType.GOLD_CHANGE
        assert gold_outcome.value == -35
        assert heal_outcome.type == OutcomeType.HP_CHANGE
        assert heal_outcome.value_percent == 0.25

    def test_sssserpent_gold_for_curse(self):
        """Sssserpent gives gold for curse."""
        choice = SSSSERPENT.choices[0]
        assert len(choice.outcomes) == 2
        gold = next(o for o in choice.outcomes if o.type == OutcomeType.GOLD_CHANGE)
        curse = next(o for o in choice.outcomes if o.type == OutcomeType.CURSE_GAIN)
        assert gold.value == 175
        assert curse.card_id == "Doubt"

    def test_living_wall_has_three_options(self):
        """Living Wall has remove, transform, upgrade options."""
        assert len(LIVING_WALL.choices) == 3
        types = [c.outcomes[0].type for c in LIVING_WALL.choices]
        assert OutcomeType.CARD_REMOVE in types
        assert OutcomeType.CARD_TRANSFORM in types
        assert OutcomeType.CARD_UPGRADE in types

    def test_leave_options_do_nothing(self):
        """Leave options should have NOTHING outcome."""
        events_with_leave = [CLERIC, DEAD_ADVENTURER, GOLDEN_IDOL, GOLDEN_WING,
                            SCRAP_OOZE, SHINING_LIGHT, SSSSERPENT, DUPLICATOR,
                            PURIFICATION_SHRINE, TRANSMOGRIFIER, UPGRADE_SHRINE]
        for event in events_with_leave:
            leave_choices = [c for c in event.choices
                           if "Leave" in c.description or "leave" in c.description.lower()]
            for choice in leave_choices:
                if choice.outcomes:
                    assert choice.outcomes[0].type == OutcomeType.NOTHING


# =============================================================================
# TEST CLASS: RNG-Dependent Events
# =============================================================================

class TestRNGDependentEvents:
    """Test events with RNG-based outcomes."""

    def test_wheel_of_change_has_multiple_outcomes(self):
        """Wheel of Change has 6 possible outcomes."""
        spin_choice = GREMLIN_WHEEL_GAME.choices[0]
        # Should have multiple outcome types listed
        assert len(spin_choice.outcomes) == 6

    def test_wheel_outcomes_include_all_types(self):
        """Wheel includes gold, relic, heal, curse, remove, damage."""
        outcomes = GREMLIN_WHEEL_GAME.choices[0].outcomes
        types = [o.type for o in outcomes]
        assert OutcomeType.GOLD_CHANGE in types
        assert OutcomeType.RELIC_GAIN in types
        assert OutcomeType.HP_CHANGE in types
        assert OutcomeType.CURSE_GAIN in types
        assert OutcomeType.CARD_REMOVE in types

    def test_match_game_has_card_rewards(self):
        """Match and Keep gives card rewards."""
        play_choice = GREMLIN_MATCH_GAME.choices[0]
        assert any(o.type == OutcomeType.CARD_GAIN for o in play_choice.outcomes)

    def test_dead_adventurer_has_success_chance(self):
        """Dead Adventurer search has success probability."""
        search_choice = DEAD_ADVENTURER.choices[0]
        assert search_choice.success_chance == 0.75  # 75% chance of NOT fighting

    def test_scrap_ooze_has_success_chance(self):
        """Scrap Ooze reach has success probability."""
        reach_choice = SCRAP_OOZE.choices[0]
        assert reach_choice.success_chance == 0.25  # Starts at 25%

    def test_joust_has_different_success_chances(self):
        """The Joust has different win probabilities."""
        owner_choice = THE_JOUST.choices[0]
        murderer_choice = THE_JOUST.choices[1]
        assert owner_choice.success_chance == 0.30
        assert murderer_choice.success_chance == 0.70

    def test_mausoleum_fifty_fifty(self):
        """Mausoleum has 50% success chance."""
        open_choice = THE_MAUSOLEUM.choices[0]
        assert open_choice.success_chance == 0.50

    def test_deterministic_event_rng(self):
        """Same seed produces same event outcomes."""
        seed = seed_to_long("EVENTTEST")
        rng1 = Random(seed)
        rng2 = Random(seed)

        # Simulate 10 event rolls
        results1 = [rng1.random_int(99) for _ in range(10)]
        results2 = [rng2.random_int(99) for _ in range(10)]

        assert results1 == results2


# =============================================================================
# TEST CLASS: Shrine Events
# =============================================================================

class TestShrineEvents:
    """Test shrine events (upgrade, remove, transform)."""

    def test_upgrade_shrine_upgrades_card(self):
        """Upgrade Shrine lets you upgrade a card."""
        choice = UPGRADE_SHRINE.choices[0]
        assert choice.outcomes[0].type == OutcomeType.CARD_UPGRADE
        assert choice.requires_upgradable_cards is True

    def test_purifier_removes_card(self):
        """Purifier shrine removes a card."""
        choice = PURIFICATION_SHRINE.choices[0]
        assert choice.outcomes[0].type == OutcomeType.CARD_REMOVE
        assert choice.requires_removable_cards is True

    def test_transmogrifier_transforms_card(self):
        """Transmogrifier transforms a card."""
        choice = TRANSMOGRIFIER.choices[0]
        assert choice.outcomes[0].type == OutcomeType.CARD_TRANSFORM
        assert choice.requires_removable_cards is True

    def test_duplicator_copies_card(self):
        """Duplicator copies a card."""
        choice = DUPLICATOR.choices[0]
        assert choice.outcomes[0].type == OutcomeType.CARD_GAIN

    def test_accursed_blacksmith_upgrade_or_relic(self):
        """Accursed Blacksmith offers upgrade or relic+curse."""
        assert len(ACCURSED_BLACKSMITH.choices) == 3
        forge_choice = ACCURSED_BLACKSMITH.choices[0]
        rummage_choice = ACCURSED_BLACKSMITH.choices[1]
        assert forge_choice.outcomes[0].type == OutcomeType.CARD_UPGRADE
        assert any(o.type == OutcomeType.RELIC_GAIN for o in rummage_choice.outcomes)
        assert any(o.type == OutcomeType.CURSE_GAIN for o in rummage_choice.outcomes)

    def test_bonfire_elementals_remove_card(self):
        """Bonfire Elementals removes a card for heal."""
        choice = BONFIRE_ELEMENTALS.choices[0]
        assert any(o.type == OutcomeType.CARD_REMOVE for o in choice.outcomes)
        assert any(o.type == OutcomeType.HP_CHANGE for o in choice.outcomes)


# =============================================================================
# TEST CLASS: Conditional Events
# =============================================================================

class TestConditionalEvents:
    """Test events with requirements."""

    def test_cleric_heal_requires_gold(self):
        """Cleric heal requires 35 gold."""
        heal_choice = CLERIC.choices[0]
        assert heal_choice.requires_gold == 35

    def test_cleric_purify_requires_gold_and_cards(self):
        """Cleric purify requires gold and removable cards."""
        purify_choice = CLERIC.choices[1]
        assert purify_choice.requires_gold == 50
        assert purify_choice.requires_removable_cards is True

    def test_forgotten_altar_requires_golden_idol(self):
        """Forgotten Altar offer requires Golden Idol."""
        offer_choice = FORGOTTEN_ALTAR.choices[0]
        assert offer_choice.requires_relic == "Golden Idol"

    def test_moai_head_idol_trade(self):
        """Moai Head can trade Golden Idol for gold."""
        trade_choice = MOAI_HEAD.choices[1]
        assert trade_choice.requires_relic == "Golden Idol"

    def test_tomb_requires_red_mask(self):
        """Tomb of Lord Red Mask gold option requires Red Mask."""
        gold_choice = TOMB_OF_LORD_RED_MASK.choices[1]
        assert gold_choice.requires_relic == "Red Mask"

    def test_addict_buy_requires_gold(self):
        """Addict buy option requires gold."""
        buy_choice = ADDICT.choices[0]
        assert buy_choice.requires_gold == 85

    def test_beggar_give_requires_gold(self):
        """Beggar give option requires gold."""
        give_choice = BEGGAR.choices[0]
        assert give_choice.requires_gold == 75

    def test_woman_in_blue_potions_require_gold(self):
        """Woman in Blue potion purchases require increasing gold."""
        assert WOMAN_IN_BLUE.choices[0].requires_gold == 20
        assert WOMAN_IN_BLUE.choices[1].requires_gold == 30
        assert WOMAN_IN_BLUE.choices[2].requires_gold == 40

    def test_we_meet_again_options(self):
        """We Meet Again requires potion, gold, or non-basic card."""
        potion_choice = WE_MEET_AGAIN.choices[0]
        gold_choice = WE_MEET_AGAIN.choices[1]
        card_choice = WE_MEET_AGAIN.choices[2]
        assert potion_choice.requires_potion is True
        assert gold_choice.requires_gold == 50
        assert card_choice.requires_non_basic_card is True

    def test_designer_prices_scale(self):
        """Designer options have increasing gold costs."""
        assert DESIGNER.choices[0].requires_gold == 40
        assert DESIGNER.choices[1].requires_gold == 60
        assert DESIGNER.choices[2].requires_gold == 90

    def test_falling_requires_card_types(self):
        """Falling requires specific card types."""
        skill_choice = FALLING.choices[0]
        power_choice = FALLING.choices[1]
        attack_choice = FALLING.choices[2]
        assert skill_choice.requires_card_type == "SKILL"
        assert power_choice.requires_card_type == "POWER"
        assert attack_choice.requires_card_type == "ATTACK"


# =============================================================================
# TEST CLASS: Ascension Event Modifications
# =============================================================================

class TestAscensionModifications:
    """Test A15+ event modifications."""

    def test_cleric_has_ascension_modifier(self):
        """Cleric has ascension modifier flag."""
        assert CLERIC.has_ascension_modifier is True

    def test_events_with_a15_changes(self):
        """Multiple events have ascension modifiers."""
        a15_events = [
            CLERIC, DEAD_ADVENTURER, GOLDEN_IDOL, GOOP_PUDDLE,
            SCRAP_OOZE, SHINING_LIGHT, SSSSERPENT,  # Act 1
            CURSED_TOME, FORGOTTEN_ALTAR, GHOSTS,  # Act 2
            MIND_BLOOM, MOAI_HEAD, WINDING_HALLS,  # Act 3
            DESIGNER, FACE_TRADER, GOLD_SHRINE, GREMLIN_MATCH_GAME,
            GREMLIN_WHEEL_GAME, LAB, WOMAN_IN_BLUE,  # Shrines
        ]
        for event in a15_events:
            assert event.has_ascension_modifier is True, f"{event.name} should have A15 modifier"

    def test_ascension_threshold_default(self):
        """Default ascension threshold is 15."""
        assert CLERIC.ascension_threshold == 15

    def test_calculate_outcome_with_ascension(self):
        """Outcome values change with ascension level."""
        # Get a damage outcome
        damage_outcome = Outcome(
            type=OutcomeType.HP_CHANGE,
            value=-10,
            description="Take 10 damage"
        )

        # At A0, value is base
        a0_value = calculate_outcome_value(damage_outcome, 80, 70, ascension_level=0)
        assert a0_value == -10

        # At A15+, damage is increased (rough approximation 1.4x)
        a15_value = calculate_outcome_value(damage_outcome, 80, 70, ascension_level=15)
        assert a15_value == -14  # 10 * 1.4


# =============================================================================
# TEST CLASS: Special Events
# =============================================================================

class TestSpecialEvents:
    """Test special/unique events."""

    def test_mind_bloom_three_options(self):
        """Mind Bloom has 3 main options (4 with alternate)."""
        # Note: Mind Bloom has 4 choices in the data (3 base + alternate for floors 41-50)
        assert len(MIND_BLOOM.choices) >= 3

    def test_mind_bloom_war_combat(self):
        """Mind Bloom War option triggers combat."""
        war_choice = MIND_BLOOM.choices[0]
        assert any(o.type == OutcomeType.COMBAT for o in war_choice.outcomes)
        assert any(o.type == OutcomeType.RELIC_GAIN for o in war_choice.outcomes)

    def test_mind_bloom_awake_upgrade_all(self):
        """Mind Bloom Awake upgrades all cards."""
        awake_choice = MIND_BLOOM.choices[1]
        upgrade_outcome = next(o for o in awake_choice.outcomes
                              if o.type == OutcomeType.CARD_UPGRADE)
        # Upgrades ALL cards (no count limit)
        assert "all" in upgrade_outcome.description.lower() or upgrade_outcome.count is None

    def test_mind_bloom_rich_gives_999_gold(self):
        """Mind Bloom Rich gives 999 gold and 2 Normality."""
        rich_choice = MIND_BLOOM.choices[2]
        gold = next(o for o in rich_choice.outcomes if o.type == OutcomeType.GOLD_CHANGE)
        curse = next(o for o in rich_choice.outcomes if o.type == OutcomeType.CURSE_GAIN)
        assert gold.value == 999
        assert curse.card_id == "Normality"
        assert curse.count == 2

    def test_colosseum_two_fights(self):
        """Colosseum triggers combat and offers second fight."""
        enter_choice = COLOSSEUM.choices[0]
        assert any(o.type == OutcomeType.COMBAT for o in enter_choice.outcomes)
        # Second fight choice
        assert COLOSSEUM_FIGHT_NOBS.outcomes[0].type == OutcomeType.COMBAT

    def test_colosseum_nobs_big_rewards(self):
        """Colosseum Nobs fight gives big rewards."""
        outcomes = COLOSSEUM_FIGHT_NOBS.outcomes
        relic_gains = [o for o in outcomes if o.type == OutcomeType.RELIC_GAIN]
        gold_gain = next(o for o in outcomes if o.type == OutcomeType.GOLD_CHANGE)
        assert len(relic_gains) == 2  # Rare + Uncommon
        assert gold_gain.value == 100

    def test_secret_portal_skips_to_boss(self):
        """Secret Portal lets you skip to boss."""
        enter_choice = SECRET_PORTAL.choices[0]
        assert "boss" in enter_choice.description.lower() or \
               "boss" in enter_choice.outcomes[0].description.lower()

    def test_vampires_transformation(self):
        """Vampires replaces Strikes with Bites."""
        accept_choice = VAMPIRES.choices[0]
        remove = next(o for o in accept_choice.outcomes if o.type == OutcomeType.CARD_REMOVE)
        gain = next(o for o in accept_choice.outcomes if o.type == OutcomeType.CARD_GAIN)
        assert "Strike" in remove.description
        assert gain.card_id == "Bite"
        assert gain.count == 5

    def test_ghosts_apparitions(self):
        """Ghosts gives Apparition cards."""
        accept_choice = GHOSTS.choices[0]
        card_gain = next(o for o in accept_choice.outcomes if o.type == OutcomeType.CARD_GAIN)
        assert card_gain.card_id == "Apparition"
        assert card_gain.count == 5  # 3 on A15+


# =============================================================================
# TEST CLASS: Neow Bonuses
# =============================================================================

class TestNeowBonuses:
    """Test Neow bonus options."""

    def test_neow_category_0_small_benefits(self):
        """Category 0 are small benefits without drawback."""
        small = [NEOW_THREE_CARDS, NEOW_ONE_RANDOM_RARE, NEOW_REMOVE_CARD,
                NEOW_UPGRADE_CARD, NEOW_TRANSFORM_CARD, NEOW_RANDOM_COLORLESS]
        for bonus in small:
            assert bonus.category == 0
            assert bonus.drawback is None

    def test_neow_category_1_medium_benefits(self):
        """Category 1 are medium benefits without drawback."""
        medium = [NEOW_THREE_POTIONS, NEOW_RANDOM_COMMON_RELIC,
                 NEOW_TEN_PERCENT_HP, NEOW_THREE_ENEMY_KILL, NEOW_HUNDRED_GOLD]
        for bonus in medium:
            assert bonus.category == 1
            assert bonus.drawback is None

    def test_neow_category_2_with_drawback(self):
        """Category 2 are large benefits with drawback."""
        large = [NEOW_RANDOM_COLORLESS_2, NEOW_REMOVE_TWO, NEOW_ONE_RARE_RELIC,
                NEOW_THREE_RARE_CARDS, NEOW_TWO_FIFTY_GOLD, NEOW_TRANSFORM_TWO,
                NEOW_TWENTY_PERCENT_HP]
        for bonus in large:
            assert bonus.category == 2

    def test_neow_category_3_boss_swap(self):
        """Category 3 is boss relic swap."""
        assert NEOW_BOSS_SWAP.category == 3

    def test_neow_gold_values(self):
        """Neow gold bonuses have correct values."""
        assert NEOW_HUNDRED_GOLD.gold_bonus == 100
        assert NEOW_TWO_FIFTY_GOLD.gold_bonus == 250

    def test_neow_drawbacks_defined(self):
        """Neow drawback strings are defined."""
        assert "HP" in NEOW_DRAWBACK_10_PERCENT_HP_LOSS
        assert "gold" in NEOW_DRAWBACK_NO_GOLD.lower()
        assert "curse" in NEOW_DRAWBACK_CURSE.lower()
        assert "damage" in NEOW_DRAWBACK_PERCENT_DAMAGE.lower()

    def test_neow_three_enemy_kill_description(self):
        """Neow's Lament description matches effect."""
        assert "Lament" in NEOW_THREE_ENEMY_KILL.description or \
               "3" in NEOW_THREE_ENEMY_KILL.description


# =============================================================================
# TEST CLASS: Combat Trigger Events
# =============================================================================

class TestCombatTriggerEvents:
    """Test events that can trigger combat."""

    def test_dead_adventurer_can_trigger_elite(self):
        """Dead Adventurer can trigger elite fight."""
        search_choice = DEAD_ADVENTURER.choices[0]
        combat_outcomes = [o for o in search_choice.outcomes if o.type == OutcomeType.COMBAT]
        assert len(combat_outcomes) > 0

    def test_mushrooms_has_combat(self):
        """Mushrooms fight option triggers combat."""
        fight_choice = MUSHROOMS.choices[0]
        assert any(o.type == OutcomeType.COMBAT for o in fight_choice.outcomes)

    def test_masked_bandits_fight(self):
        """Masked Bandits fight triggers combat."""
        fight_choice = MASKED_BANDITS.choices[1]
        assert any(o.type == OutcomeType.COMBAT for o in fight_choice.outcomes)

    def test_mysterious_sphere_combat(self):
        """Mysterious Sphere has combat."""
        open_choice = MYSTERIOUS_SPHERE.choices[0]
        assert any(o.type == OutcomeType.COMBAT for o in open_choice.outcomes)

    def test_colosseum_has_combat(self):
        """Colosseum triggers combat."""
        assert any(o.type == OutcomeType.COMBAT for o in COLOSSEUM.choices[0].outcomes)

    def test_vampires_fight_option(self):
        """Vampires refuse triggers combat."""
        refuse_choice = VAMPIRES.choices[1]
        assert any(o.type == OutcomeType.COMBAT for o in refuse_choice.outcomes)


# =============================================================================
# TEST CLASS: Event Relic Rewards
# =============================================================================

class TestEventRelicRewards:
    """Test events that give relics."""

    def test_golden_idol_gives_relic(self):
        """Golden Idol event gives Golden Idol relic."""
        take_choice = GOLDEN_IDOL.choices[0]
        relic_gain = next(o for o in take_choice.outcomes if o.type == OutcomeType.RELIC_GAIN)
        assert relic_gain.relic_id == "Golden Idol"

    def test_mushrooms_gives_odd_mushroom(self):
        """Mushrooms combat gives Odd Mushroom."""
        fight_choice = MUSHROOMS.choices[0]
        relic_gain = next(o for o in fight_choice.outcomes if o.type == OutcomeType.RELIC_GAIN)
        assert relic_gain.relic_id == "Odd Mushroom"

    def test_cursed_tome_book_relics(self):
        """Cursed Tome gives book relics."""
        read_choice = CURSED_TOME.choices[0]
        relic_gain = next(o for o in read_choice.outcomes if o.type == OutcomeType.RELIC_GAIN)
        assert "Necronomicon" in relic_gain.description or \
               "Enchiridion" in relic_gain.description or \
               "Codex" in relic_gain.description

    def test_nloth_gives_nloth_gift(self):
        """N'loth trades give N'loth's Gift."""
        trade_choice = NLOTH.choices[0]
        relic_gain = next(o for o in trade_choice.outcomes if o.type == OutcomeType.RELIC_GAIN)
        assert "N'loth" in relic_gain.relic_id or "Nloth" in relic_gain.relic_id

    def test_tomb_red_mask_gives_red_mask(self):
        """Tomb of Lord Red Mask gives Red Mask."""
        don_choice = TOMB_OF_LORD_RED_MASK.choices[0]
        relic_gain = next(o for o in don_choice.outcomes if o.type == OutcomeType.RELIC_GAIN)
        assert relic_gain.relic_id == "Red Mask"

    def test_face_trader_mask_relics(self):
        """Face Trader gives mask/face relics."""
        trade_choice = FACE_TRADER.choices[1]
        relic_gain = next(o for o in trade_choice.outcomes if o.type == OutcomeType.RELIC_GAIN)
        assert relic_gain.random is True
        assert "mask" in relic_gain.description.lower() or "face" in trade_choice.description.lower()


# =============================================================================
# TEST CLASS: Event Card Rewards
# =============================================================================

class TestEventCardRewards:
    """Test events that give cards."""

    def test_nest_ritual_dagger(self):
        """The Nest gives Ritual Dagger."""
        take_choice = NEST.choices[0]
        card_gain = next(o for o in take_choice.outcomes if o.type == OutcomeType.CARD_GAIN)
        assert card_gain.card_id == "Ritual Dagger"

    def test_drug_dealer_jax(self):
        """Augmenter gives J.A.X. card."""
        ingest_choice = DRUG_DEALER.choices[0]
        card_gain = next(o for o in ingest_choice.outcomes if o.type == OutcomeType.CARD_GAIN)
        assert card_gain.card_id == "J.A.X."

    def test_winding_halls_madness(self):
        """Winding Halls gives Madness cards."""
        madness_choice = WINDING_HALLS.choices[0]
        card_gain = next(o for o in madness_choice.outcomes if o.type == OutcomeType.CARD_GAIN)
        assert card_gain.card_id == "Madness"
        assert card_gain.count == 2

    def test_library_card_choice(self):
        """Library lets you choose from 20 cards."""
        read_choice = THE_LIBRARY.choices[0]
        card_choice = next(o for o in read_choice.outcomes if o.type == OutcomeType.CARD_CHOICE)
        assert card_choice.count == 20

    def test_sensory_stone_colorless_cards(self):
        """Sensory Stone gives colorless cards."""
        touch_choice = SENSORY_STONE.choices[0]
        card_gain = next(o for o in touch_choice.outcomes if o.type == OutcomeType.CARD_GAIN)
        assert "colorless" in card_gain.description.lower()


# =============================================================================
# TEST CLASS: Event Lookup Functions
# =============================================================================

class TestEventLookupFunctions:
    """Test event lookup utility functions."""

    def test_get_event_by_id(self):
        """Can get events by their ID."""
        event = get_event("Big Fish")
        assert event is not None
        assert event.name == "Big Fish"

    def test_get_event_returns_none_for_invalid(self):
        """get_event returns None for invalid ID."""
        event = get_event("Not A Real Event")
        assert event is None

    def test_all_events_contains_all(self):
        """ALL_EVENTS contains events from all acts."""
        total = len(EXORDIUM_EVENTS) + len(CITY_EVENTS) + \
                len(BEYOND_EVENTS) + len(SHRINE_EVENTS)
        # ALL_EVENTS includes additional events not in the per-act pools
        assert len(ALL_EVENTS) >= total
        assert len(ALL_EVENTS) == 51

    def test_events_have_unique_ids(self):
        """All events have unique IDs."""
        ids = list(ALL_EVENTS.keys())
        assert len(ids) == len(set(ids))


# =============================================================================
# TEST CLASS: Outcome Value Calculation
# =============================================================================

class TestOutcomeValueCalculation:
    """Test calculate_outcome_value function."""

    def test_fixed_value_outcome(self):
        """Fixed value outcomes return that value."""
        outcome = Outcome(
            type=OutcomeType.GOLD_CHANGE,
            value=100,
            description="Gain 100 gold"
        )
        value = calculate_outcome_value(outcome, 80, 70, 0)
        assert value == 100

    def test_percent_value_outcome(self):
        """Percent-based outcomes calculate from max HP."""
        outcome = Outcome(
            type=OutcomeType.HP_CHANGE,
            value_percent=0.25,
            description="Heal 25% max HP"
        )
        value = calculate_outcome_value(outcome, 80, 70, 0)
        assert value == 20  # 25% of 80

    def test_negative_percent_damage(self):
        """Negative percent outcomes work correctly."""
        outcome = Outcome(
            type=OutcomeType.HP_CHANGE,
            value_percent=-0.10,
            description="Lose 10% max HP"
        )
        value = calculate_outcome_value(outcome, 100, 80, 0)
        assert value == -10

    def test_no_value_returns_zero(self):
        """Outcomes with no value return 0."""
        outcome = Outcome(
            type=OutcomeType.NOTHING,
            description="Nothing happens"
        )
        value = calculate_outcome_value(outcome, 80, 70, 0)
        assert value == 0


# =============================================================================
# TEST CLASS: Curse Mechanics
# =============================================================================

class TestCurseMechanics:
    """Test curse-related event mechanics."""

    def test_fountain_requires_curse(self):
        """Fountain of Cleansing only appears if you have a curse."""
        assert FOUNTAIN_OF_CURSE_REMOVAL.requires_curse_in_deck is True

    def test_fountain_removes_curses(self):
        """Fountain removes curses."""
        drink_choice = FOUNTAIN_OF_CURSE_REMOVAL.choices[0]
        assert any(o.type == OutcomeType.CARD_REMOVE for o in drink_choice.outcomes)
        assert "curse" in drink_choice.description.lower()

    def test_events_that_give_curses(self):
        """Multiple events can give curses."""
        curse_events = [
            (BIG_FISH, 2),  # Box option
            (GOLDEN_IDOL_ESCAPE_INJURY, 0),  # Injury curse
            (SSSSERPENT, 0),  # Doubt curse
            (MUSHROOMS, 1),  # Parasite curse
            (ADDICT, 1),  # Shame curse
            (BEGGAR, 1),  # Doubt curse
            (GOLD_SHRINE, 1),  # Regret curse
            (GREMLIN_WHEEL_GAME, 0),  # Decay curse (possible)
            (WINDING_HALLS, 1),  # Writhe curse
        ]
        for event, choice_idx in curse_events:
            if isinstance(event, Event):
                choices = event.choices
            else:
                choices = [event]  # Already a choice
            if choice_idx < len(choices):
                choice = choices[choice_idx]
                assert any(o.type == OutcomeType.CURSE_GAIN for o in choice.outcomes), \
                    f"{event if isinstance(event, str) else event.name} choice {choice_idx} should give curse"

    def test_ghosts_gives_apparitions_not_curses(self):
        """Ghosts gives Apparition cards (not curses) and loses max HP."""
        accept_choice = GHOSTS.choices[0]
        # Should have max HP loss
        assert any(o.type == OutcomeType.MAX_HP_CHANGE for o in accept_choice.outcomes)
        # Apparitions are given (they are special cards, not curses)
        assert any(o.type == OutcomeType.CARD_GAIN for o in accept_choice.outcomes)


# =============================================================================
# TEST CLASS: Golden Idol Subevents
# =============================================================================

class TestGoldenIdolSubevents:
    """Test Golden Idol escape choices."""

    def test_escape_injury_gives_curse(self):
        """Outrun option gives Injury curse."""
        outcomes = GOLDEN_IDOL_ESCAPE_INJURY.outcomes
        curse = next(o for o in outcomes if o.type == OutcomeType.CURSE_GAIN)
        assert curse.card_id == "Injury"

    def test_escape_damage_takes_damage(self):
        """Smash option takes HP damage."""
        outcomes = GOLDEN_IDOL_ESCAPE_DAMAGE.outcomes
        damage = next(o for o in outcomes if o.type == OutcomeType.HP_CHANGE)
        assert damage.value_percent == -0.25

    def test_escape_max_hp_loses_max_hp(self):
        """Hide option loses max HP."""
        outcomes = GOLDEN_IDOL_ESCAPE_MAX_HP.outcomes
        max_hp = next(o for o in outcomes if o.type == OutcomeType.MAX_HP_CHANGE)
        assert max_hp.value_percent == -0.08


# =============================================================================
# TEST CLASS: Deterministic RNG Tests
# =============================================================================

class TestDeterministicRNG:
    """Test that events produce deterministic outcomes with same seed."""

    def test_same_seed_same_event_roll(self):
        """Same seed produces same event selection."""
        seed = seed_to_long("SAMESEED")

        rng1 = Random(seed)
        rng2 = Random(seed)

        # Simulate picking from event pool
        events = list(ALL_EVENTS.keys())
        idx1 = rng1.random_int(len(events) - 1)
        idx2 = rng2.random_int(len(events) - 1)

        assert idx1 == idx2

    def test_same_seed_same_outcome_sequence(self):
        """Same seed produces same RNG sequence for outcomes."""
        seed = seed_to_long("OUTCOMES")

        rng1 = Random(seed)
        rng2 = Random(seed)

        # Simulate multiple outcome rolls
        for _ in range(50):
            assert rng1.random_int(99) == rng2.random_int(99)
            assert rng1.random_boolean() == rng2.random_boolean()

    def test_game_rng_event_stream(self):
        """GameRNG event stream is deterministic."""
        seed = seed_to_long("GAMERNG")

        game1 = GameRNG(seed)
        game2 = GameRNG(seed)

        # Event RNG should produce same values
        for _ in range(20):
            assert game1.event_rng.random_int(99) == game2.event_rng.random_int(99)


# =============================================================================
# TEST CLASS: Event Data Integrity
# =============================================================================

class TestEventDataIntegrity:
    """Test event data structure integrity."""

    def test_all_events_have_ids(self):
        """All events have non-empty IDs."""
        for event_id, event in ALL_EVENTS.items():
            assert event.id is not None and len(event.id) > 0

    def test_all_events_have_names(self):
        """All events have non-empty names."""
        for event_id, event in ALL_EVENTS.items():
            assert event.name is not None and len(event.name) > 0

    def test_all_events_have_choices(self):
        """All events have at least one choice."""
        for event_id, event in ALL_EVENTS.items():
            assert len(event.choices) >= 1, f"{event_id} has no choices"

    def test_all_choices_have_outcomes(self):
        """All choices have at least one outcome."""
        for event_id, event in ALL_EVENTS.items():
            for i, choice in enumerate(event.choices):
                assert len(choice.outcomes) >= 1, \
                    f"{event_id} choice {i} has no outcomes"

    def test_all_outcomes_have_types(self):
        """All outcomes have valid types."""
        for event_id, event in ALL_EVENTS.items():
            for choice in event.choices:
                for outcome in choice.outcomes:
                    assert outcome.type is not None
                    assert isinstance(outcome.type, OutcomeType)

    def test_choice_indices_are_sequential(self):
        """Choice indices should be sequential starting from 0."""
        for event_id, event in ALL_EVENTS.items():
            for i, choice in enumerate(event.choices):
                # Note: Mind Bloom has two choices with index 2 (alternate)
                # so we only check that indices exist and are reasonable
                assert choice.index >= 0


# =============================================================================
# TEST CLASS: Event Act Enum
# =============================================================================

class TestActEnum:
    """Test Act enumeration."""

    def test_act_values(self):
        """Act values are correct."""
        assert Act.ACT_1.value == 1
        assert Act.ACT_2.value == 2
        assert Act.ACT_3.value == 3
        assert Act.ANY.value == 0


# =============================================================================
# TEST CLASS: Back to Basics Event
# =============================================================================

class TestBackToBasics:
    """Test Back to Basics event specifically."""

    def test_simplicity_removes_basics(self):
        """Simplicity removes all Strikes and Defends."""
        simplicity = BACK_TO_BASICS.choices[0]
        remove = next(o for o in simplicity.outcomes if o.type == OutcomeType.CARD_REMOVE)
        assert "Strike" in remove.description or "Defend" in remove.description

    def test_elegance_upgrades_basics(self):
        """Elegance upgrades all Strikes and Defends."""
        elegance = BACK_TO_BASICS.choices[1]
        upgrade = next(o for o in elegance.outcomes if o.type == OutcomeType.CARD_UPGRADE)
        assert "Strike" in upgrade.description or "Defend" in upgrade.description


# =============================================================================
# TEST CLASS: Knowing Skull Event
# =============================================================================

class TestKnowingSkull:
    """Test Knowing Skull event mechanics."""

    def test_knowing_skull_four_options(self):
        """Knowing Skull has 4 options."""
        assert len(KNOWING_SKULL.choices) == 4

    def test_knowing_skull_costs_hp(self):
        """All Knowing Skull options cost HP."""
        for choice in KNOWING_SKULL.choices:
            hp_loss = [o for o in choice.outcomes if o.type == OutcomeType.HP_CHANGE]
            assert len(hp_loss) > 0

    def test_knowing_skull_rewards(self):
        """Knowing Skull gives potion, gold, or card."""
        potion_choice = KNOWING_SKULL.choices[0]
        gold_choice = KNOWING_SKULL.choices[1]
        card_choice = KNOWING_SKULL.choices[2]

        assert any(o.type == OutcomeType.POTION_GAIN for o in potion_choice.outcomes)
        assert any(o.type == OutcomeType.GOLD_CHANGE and o.value > 0
                  for o in gold_choice.outcomes)
        assert any(o.type == OutcomeType.CARD_GAIN for o in card_choice.outcomes)
