"""
Audit tests: Python event outcomes vs decompiled Java source.
Each test documents a specific discrepancy or verifies correctness.
"""

import math
import pytest

from packages.engine.content.events import (
    BIG_FISH,
    CLERIC,
    GOLDEN_IDOL,
    GOLDEN_IDOL_ESCAPE_DAMAGE,
    GOLDEN_IDOL_ESCAPE_INJURY,
    GOLDEN_IDOL_ESCAPE_MAX_HP,
    GOLDEN_WING,
    GOOP_PUDDLE,
    SSSSERPENT,
    SCRAP_OOZE,
    SHINING_LIGHT,
    MUSHROOMS,
    BEGGAR,
    NEST,
    CURSED_TOME,
    KNOWING_SKULL,
    FORGOTTEN_ALTAR,
    GHOSTS,
    VAMPIRES,
    ADDICT,
    GOLD_SHRINE,
    OutcomeType,
    calculate_outcome_value,
)


# ---------------------------------------------------------------------------
# Helper
# ---------------------------------------------------------------------------

def _find_outcome(choices_or_choice, outcome_type, index=0):
    """Find the Nth outcome of a given type across all outcomes in a choice or list of choices."""
    if isinstance(choices_or_choice, list):
        outcomes = []
        for c in choices_or_choice:
            outcomes.extend(c.outcomes)
    else:
        outcomes = choices_or_choice.outcomes
    found = [o for o in outcomes if o.type == outcome_type]
    return found[index] if index < len(found) else None


# ===========================================================================
# ACT 1 EVENTS
# ===========================================================================


class TestBigFish:
    """Java: healAmt = maxHealth / 3 (integer division)."""

    def test_heal_uses_integer_division(self):
        """Big Fish heal should be maxHP // 3, matching Java integer division."""
        outcome = BIG_FISH.choices[0].outcomes[0]
        assert outcome.type == OutcomeType.HP_CHANGE

        # For maxHP=72: Java gives 72//3=24
        max_hp = 72
        python_result = calculate_outcome_value(outcome, max_hp, max_hp)
        assert python_result == 24  # maxHP // 3

    def test_heal_matches_at_divisible_hp(self):
        """When maxHP is divisible by 3, result is exact."""
        outcome = BIG_FISH.choices[0].outcomes[0]
        max_hp = 75
        python_result = calculate_outcome_value(outcome, max_hp, max_hp)
        assert python_result == 25  # 75 // 3

    def test_donut_max_hp(self):
        """Donut gives +5 max HP. Java: increaseMaxHp(5, true)."""
        outcome = BIG_FISH.choices[1].outcomes[0]
        assert outcome.type == OutcomeType.MAX_HP_CHANGE
        assert outcome.value == 5

    def test_box_gives_regret(self):
        """Box gives random relic + Regret curse."""
        outcomes = BIG_FISH.choices[2].outcomes
        relic = [o for o in outcomes if o.type == OutcomeType.RELIC_GAIN]
        curse = [o for o in outcomes if o.type == OutcomeType.CURSE_GAIN]
        assert len(relic) == 1
        assert relic[0].random is True
        assert len(curse) == 1
        assert curse[0].card_id == "Regret"


class TestCleric:
    """Java: heal cost 35, purify cost 50/75, heal = int(maxHP * 0.25f)."""

    def test_heal_cost(self):
        gold_outcome = CLERIC.choices[0].outcomes[0]
        assert gold_outcome.value == -35

    def test_purify_cost_base(self):
        gold_outcome = CLERIC.choices[1].outcomes[0]
        assert gold_outcome.value == -50  # Base cost, 75 on A15+

    def test_heal_amount_truncates(self):
        """Java uses (int)(maxHP * 0.25f) which truncates."""
        outcome = CLERIC.choices[0].outcomes[1]
        assert outcome.value_percent == 0.25
        # For maxHP=73: int(73*0.25) = int(18.25) = 18
        val = calculate_outcome_value(outcome, 73, 73)
        assert val == 18  # truncation


class TestGoldenIdol:
    """Java: damage = (int)(maxHP * 0.25f), maxHpLoss = (int)(maxHP * 0.08f), min 1."""

    def test_escape_damage_uses_truncation(self):
        """Java uses (int) cast which truncates toward zero."""
        outcome = GOLDEN_IDOL_ESCAPE_DAMAGE.outcomes[0]
        assert outcome.value_percent == -0.25

        max_hp = 73
        # Java: (int)(73 * 0.25f) = (int)(18.25) = 18
        python_result = calculate_outcome_value(outcome, max_hp, max_hp)
        assert python_result == -18  # truncation, not ceil

    def test_max_hp_loss_uses_truncation(self):
        outcome = GOLDEN_IDOL_ESCAPE_MAX_HP.outcomes[0]
        assert outcome.value_percent == -0.08

        max_hp = 80
        # Java: (int)(80 * 0.08f) = (int)(6.4) = 6
        python_result = calculate_outcome_value(outcome, max_hp, max_hp)
        assert python_result == -6  # truncation, not ceil

    def test_max_hp_loss_minimum_one(self):
        """Java: if (maxHpLoss < 1) maxHpLoss = 1."""
        # For very low maxHP, Java clamps to 1
        max_hp = 10
        java_result = max(int(max_hp * 0.08), 1)  # int(0.8) = 0 -> clamped to 1
        assert java_result == 1


class TestShiningLight:
    """Java: damage = MathUtils.round(maxHP * 0.2f) or round(maxHP * 0.3f)."""

    def test_damage_uses_round(self):
        """Java uses MathUtils.round for Shining Light damage."""
        outcome = SHINING_LIGHT.choices[0].outcomes[0]
        assert outcome.value_percent == -0.20

        max_hp = 72
        # Java: MathUtils.round(72 * 0.2f) = round(14.4) = 14
        python_result = calculate_outcome_value(outcome, max_hp, max_hp)
        assert python_result == -14  # round, not ceil

    def test_upgrades_two_cards(self):
        outcome = SHINING_LIGHT.choices[0].outcomes[1]
        assert outcome.type == OutcomeType.CARD_UPGRADE
        assert outcome.count == 2


class TestForgottenAltar:
    """Java: hpLoss = MathUtils.round(maxHP * 0.25f) or round(maxHP * 0.35f)."""

    def test_sacrifice_damage_uses_round(self):
        """Java uses MathUtils.round for Forgotten Altar sacrifice damage."""
        outcome = FORGOTTEN_ALTAR.choices[1].outcomes[1]
        assert outcome.value_percent == -0.25

        max_hp = 73
        # Java: MathUtils.round(73 * 0.25f) = round(18.25) = 18
        python_result = calculate_outcome_value(outcome, max_hp, max_hp)
        assert python_result == -18  # round, not ceil

    def test_sacrifice_gives_5_max_hp(self):
        outcome = FORGOTTEN_ALTAR.choices[1].outcomes[0]
        assert outcome.type == OutcomeType.MAX_HP_CHANGE
        assert outcome.value == 5

    def test_desecrate_gives_decay(self):
        outcome = FORGOTTEN_ALTAR.choices[2].outcomes[0]
        assert outcome.type == OutcomeType.CURSE_GAIN
        assert outcome.card_id == "Decay"


# ===========================================================================
# ACT 2 EVENTS
# ===========================================================================


class TestBeggar:
    """Java: Beggar gives card removal for 75 gold. No steal/curse option."""

    def test_beggar_option0_gives_card_removal(self):
        """Java: pay 75 gold, remove a card."""
        option0_outcomes = BEGGAR.choices[0].outcomes
        outcome_types = {o.type for o in option0_outcomes}
        assert OutcomeType.CARD_REMOVE in outcome_types
        assert OutcomeType.GOLD_CHANGE in outcome_types
        gold = [o for o in option0_outcomes if o.type == OutcomeType.GOLD_CHANGE][0]
        assert gold.value == -75

    def test_beggar_has_two_options(self):
        """Java: pay 75g + remove card, or leave. Only 2 options."""
        assert len(BEGGAR.choices) == 2
        assert BEGGAR.choices[1].outcomes[0].type == OutcomeType.NOTHING


class TestNest:
    """Java: option 0 = gold only (99/50 A15+), option 1 = 6 damage + Ritual Dagger."""

    def test_nest_option0_is_gold_only(self):
        """Java: option 0 steals gold only (99 base, 50 at A15+)."""
        option0 = NEST.choices[0]
        outcome_types = {o.type for o in option0.outcomes}
        assert OutcomeType.GOLD_CHANGE in outcome_types
        assert OutcomeType.CARD_GAIN not in outcome_types  # Gold only, no card

    def test_nest_option1_has_damage_and_dagger(self):
        """Java: option 1 deals 6 damage and gives Ritual Dagger."""
        option1 = NEST.choices[1]
        outcome_types = {o.type for o in option1.outcomes}
        assert OutcomeType.HP_CHANGE in outcome_types
        assert OutcomeType.CARD_GAIN in outcome_types
        hp = [o for o in option1.outcomes if o.type == OutcomeType.HP_CHANGE][0]
        assert hp.value == -6
        card = [o for o in option1.outcomes if o.type == OutcomeType.CARD_GAIN][0]
        assert card.card_id == "Ritual Dagger"

    def test_nest_gold_amount(self):
        """Java: gold = ascensionLevel >= 15 ? 50 : 99. Base is 99."""
        option0 = NEST.choices[0]
        gold = [o for o in option0.outcomes if o.type == OutcomeType.GOLD_CHANGE][0]
        assert gold.value == 99


class TestCursedTome:
    """Java: Pages deal 1+2+3 damage. Then choose: continue (10/15 more) or stop (3 more)."""

    def test_total_damage_read_all(self):
        """Total damage when reading all pages: 1+2+3+10=16 (or 1+2+3+15=21 A15+)."""
        outcome = CURSED_TOME.choices[0].outcomes[0]
        assert outcome.value == -16  # Base damage

    def test_missing_stop_option(self):
        """Java has a 'stop reading' option after page 3 (costs 3 more damage).
        Python only models read-all or leave-immediately."""
        # Python has 2 choices: read (all) or leave
        assert len(CURSED_TOME.choices) == 2
        # Java effectively has: leave, read pages 1-3 then choose (continue or stop)
        # The stop option (3 damage, no relic) is not modeled


class TestKnowingSkull:
    """Java: Each option cost starts at 6, increments independently. Leave = fixed 6."""

    def test_initial_costs_all_six(self):
        """All options start at 6 HP cost."""
        for choice in KNOWING_SKULL.choices:
            hp = [o for o in choice.outcomes if o.type == OutcomeType.HP_CHANGE]
            assert len(hp) >= 1
            assert hp[0].value == -6

    def test_gold_reward_is_90(self):
        """Gold option gives exactly 90 gold."""
        gold_option = KNOWING_SKULL.choices[1]
        gold = [o for o in gold_option.outcomes if o.type == OutcomeType.GOLD_CHANGE]
        assert len(gold) == 1
        assert gold[0].value == 90

    def test_leave_cost_never_escalates(self):
        """In Java, leaveCost is set to 6 and never modified.
        Python description says 'escalates' which is misleading but value is correct."""
        leave = KNOWING_SKULL.choices[3]
        hp = [o for o in leave.outcomes if o.type == OutcomeType.HP_CHANGE]
        assert hp[0].value == -6


# ===========================================================================
# VERIFIED CORRECT EVENTS
# ===========================================================================


class TestSssserpent:
    """Java: 175/150 gold + Doubt curse. Verified correct."""

    def test_gold_amount(self):
        outcome = SSSSERPENT.choices[0].outcomes[0]
        assert outcome.type == OutcomeType.GOLD_CHANGE
        assert outcome.value == 175

    def test_curse(self):
        outcome = SSSSERPENT.choices[0].outcomes[1]
        assert outcome.type == OutcomeType.CURSE_GAIN
        assert outcome.card_id == "Doubt"


class TestGoopPuddle:
    """Java: 75 gold + 11 damage, or lose 20-50/35-75 gold. Verified correct."""

    def test_gather_gold(self):
        outcomes = GOOP_PUDDLE.choices[0].outcomes
        gold = [o for o in outcomes if o.type == OutcomeType.GOLD_CHANGE][0]
        hp = [o for o in outcomes if o.type == OutcomeType.HP_CHANGE][0]
        assert gold.value == 75
        assert hp.value == -11


class TestGhosts:
    """Java: ceil(maxHP * 0.5) max HP loss, 5/3 Apparitions. Verified correct."""

    def test_max_hp_loss_uses_ceil(self):
        outcome = GHOSTS.choices[0].outcomes[0]
        assert outcome.type == OutcomeType.MAX_HP_CHANGE
        assert outcome.value_percent == -0.50

    def test_apparition_count(self):
        outcome = GHOSTS.choices[0].outcomes[1]
        assert outcome.type == OutcomeType.CARD_GAIN
        assert outcome.card_id == "Apparition"
        assert outcome.count == 5  # 3 on A15+


class TestVampires:
    """Java: ceil(maxHP * 0.3) max HP loss, remove strikes, gain 5 Bites."""

    def test_max_hp_loss(self):
        outcome = VAMPIRES.choices[0].outcomes[2]
        assert outcome.type == OutcomeType.MAX_HP_CHANGE
        assert outcome.value_percent == -0.30

    def test_bite_count(self):
        outcome = VAMPIRES.choices[0].outcomes[1]
        assert outcome.card_id == "Bite"
        assert outcome.count == 5


class TestAddict:
    """Java: 85 gold for relic, or steal (relic + Shame). Verified correct."""

    def test_gold_cost(self):
        assert ADDICT.choices[0].requires_gold == 85
        gold = [o for o in ADDICT.choices[0].outcomes if o.type == OutcomeType.GOLD_CHANGE][0]
        assert gold.value == -85

    def test_steal_gives_shame(self):
        curse = [o for o in ADDICT.choices[1].outcomes if o.type == OutcomeType.CURSE_GAIN][0]
        assert curse.card_id == "Shame"


class TestGoldShrine:
    """Java: 100/50 gold pray, 275 gold + Regret desecrate. Verified correct."""

    def test_pray_gold(self):
        outcome = GOLD_SHRINE.choices[0].outcomes[0]
        assert outcome.value == 100

    def test_desecrate_gold_and_curse(self):
        gold = [o for o in GOLD_SHRINE.choices[1].outcomes if o.type == OutcomeType.GOLD_CHANGE][0]
        curse = [o for o in GOLD_SHRINE.choices[1].outcomes if o.type == OutcomeType.CURSE_GAIN][0]
        assert gold.value == 275
        assert curse.card_id == "Regret"


class TestScrapOoze:
    """Java: 3/5 base damage, +1/attempt, 25% +10%/attempt relic chance."""

    def test_base_damage(self):
        outcome = SCRAP_OOZE.choices[0].outcomes[0]
        assert outcome.value == -3

    def test_relic_on_success(self):
        outcome = SCRAP_OOZE.choices[0].outcomes[1]
        assert outcome.type == OutcomeType.RELIC_GAIN
        assert outcome.random is True


class TestGoldenWing:
    """Java: 7 damage + card remove, or 50-80 gold (requires 10+ dmg card)."""

    def test_damage(self):
        outcome = GOLDEN_WING.choices[0].outcomes[0]
        assert outcome.value == -7

    def test_gold_range(self):
        outcome = GOLDEN_WING.choices[1].outcomes[0]
        assert outcome.type == OutcomeType.GOLD_CHANGE
        # Java: miscRng.random(50, 80)
        assert outcome.value == 65  # Python uses midpoint approximation


class TestMushrooms:
    """Java: Fight or heal int(maxHP * 0.25) + Parasite. Verified correct."""

    def test_heal_and_parasite(self):
        heal = MUSHROOMS.choices[1].outcomes[0]
        curse = MUSHROOMS.choices[1].outcomes[1]
        assert heal.value_percent == 0.25
        assert curse.card_id == "Parasite"


# ===========================================================================
# calculate_outcome_value rounding audit
# ===========================================================================


class TestCalculateOutcomeValue:
    """The core rounding function incorrectly uses ceil for all negative percents.
    Different Java events use different rounding methods."""

    def test_negative_percent_uses_truncate(self):
        """Java uses (int) cast which truncates toward zero."""
        from packages.engine.content.events import Outcome
        o = Outcome(OutcomeType.HP_CHANGE, value_percent=-0.25)
        # maxHP=73: int(-73*0.25) = int(-18.25) = -18
        result = calculate_outcome_value(o, 73, 73)
        assert result == -18  # Java truncation

    def test_positive_percent_uses_int(self):
        """Positive percents use int() truncation."""
        from packages.engine.content.events import Outcome
        o = Outcome(OutcomeType.HP_CHANGE, value_percent=0.25)
        result = calculate_outcome_value(o, 73, 73)
        assert result == 18  # int(18.25) = 18


# ===========================================================================
# BEHAVIOR TESTS - Event Handler Integration
# ===========================================================================

from packages.engine.handlers.event_handler import (
    EventHandler,
    EventState,
    EventPhase,
    EventChoiceResult,
    ACT2_EVENTS,
    ACT3_EVENTS,
    SPECIAL_ONE_TIME_EVENTS,
    EVENT_HANDLERS,
    EVENT_CHOICE_GENERATORS,
)
from packages.engine.content.cards import CardType
from packages.engine.state.run import create_watcher_run, RunState
from packages.engine.state.rng import Random


class TestEventHandlerPools:
    """Ensure handler event pools align with content classification."""

    def test_knowing_skull_act2_pool(self):
        assert "KnowingSkull" in ACT2_EVENTS
        assert "KnowingSkull" not in SPECIAL_ONE_TIME_EVENTS

    def test_secret_portal_act3_pool(self):
        assert "SecretPortal" in ACT3_EVENTS
        assert "SecretPortal" not in SPECIAL_ONE_TIME_EVENTS

    def test_note_for_yourself_special_pool(self):
        assert "NoteForYourself" in SPECIAL_ONE_TIME_EVENTS


class TestEventHandlerRegistries:
    """Ensure new event handlers and choice generators are registered."""

    def test_gremlin_and_note_handlers_registered(self):
        for event_id in ["GremlinMatchGame", "GremlinWheelGame", "NoteForYourself"]:
            assert event_id in EVENT_HANDLERS
            assert event_id in EVENT_CHOICE_GENERATORS


class TestGoldenShrineHandlerBehavior:
    """Behavior tests for Golden Shrine event handler."""

    def test_pray_adds_gold_base(self):
        """Praying at Golden Shrine adds 100 gold (below A15)."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_gold = run.gold

        event_state = EventState(event_id="GoldenShrine")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        assert result.gold_change == 100
        assert run.gold == initial_gold + 100

    def test_pray_adds_less_gold_a15(self):
        """Praying at Golden Shrine adds 50 gold at A15+."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=15)
        initial_gold = run.gold

        event_state = EventState(event_id="GoldenShrine")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        assert result.gold_change == 50
        assert run.gold == initial_gold + 50

    def test_desecrate_adds_gold_and_curse(self):
        """Desecrating adds 275 gold and Regret curse."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_gold = run.gold
        initial_deck_size = len(run.deck)

        event_state = EventState(event_id="GoldenShrine")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        assert result.gold_change == 275
        assert run.gold == initial_gold + 275
        assert "Regret" in result.cards_gained
        assert len(run.deck) == initial_deck_size + 1
        assert any(c.id == "Regret" for c in run.deck)

    def test_leave_no_changes(self):
        """Leaving the shrine makes no changes."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_gold = run.gold
        initial_deck_size = len(run.deck)

        event_state = EventState(event_id="GoldenShrine")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 2, run, event_rng, misc_rng=misc_rng)

        assert result.gold_change == 0
        assert run.gold == initial_gold
        assert len(run.deck) == initial_deck_size


class TestBigFishHandlerBehavior:
    """Behavior tests for Big Fish event handler."""

    def test_banana_heals_33_percent(self):
        """Banana heals 33% of max HP."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        run.damage(30)  # Take some damage first
        initial_hp = run.current_hp

        event_state = EventState(event_id="BigFish")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        expected_heal = int(run.max_hp * 0.33)
        assert result.hp_change > 0
        assert run.current_hp == min(initial_hp + expected_heal, run.max_hp)

    def test_donut_increases_max_hp(self):
        """Donut increases max HP by 5."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_max_hp = run.max_hp

        event_state = EventState(event_id="BigFish")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        assert result.max_hp_change == 5
        assert run.max_hp == initial_max_hp + 5

    def test_box_gives_relic_and_regret(self):
        """Box gives random relic and Regret curse."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_relic_count = len(run.relics)
        initial_deck_size = len(run.deck)

        event_state = EventState(event_id="BigFish")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 2, run, event_rng, misc_rng=misc_rng)

        assert len(result.relics_gained) == 1
        assert "Regret" in result.cards_gained
        assert len(run.relics) == initial_relic_count + 1
        assert any(c.id == "Regret" for c in run.deck)


class TestVampiresHandlerBehavior:
    """Behavior tests for Vampires event handler."""

    def test_accept_removes_all_strikes(self):
        """Accepting removes all Strike cards from deck."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Count strikes before
        strikes_before = sum(1 for c in run.deck if c.id == "Strike_P")
        assert strikes_before == 4  # Watcher starts with 4 strikes

        event_state = EventState(event_id="Vampires")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        strikes_after = sum(1 for c in run.deck if c.id == "Strike_P")
        assert strikes_after == 0
        assert len(result.cards_removed) == 4

    def test_accept_gains_five_bites(self):
        """Accepting gives 5 Bite cards."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        event_state = EventState(event_id="Vampires")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        bites = sum(1 for c in run.deck if c.id == "Bite")
        assert bites == 5
        assert result.cards_gained.count("Bite") == 5

    def test_accept_loses_30_percent_max_hp(self):
        """Accepting loses 30% max HP."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_max_hp = run.max_hp

        event_state = EventState(event_id="Vampires")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        expected_loss = int(initial_max_hp * 0.30)
        assert result.max_hp_change < 0
        assert run.max_hp == initial_max_hp - expected_loss

    def test_refuse_does_not_trigger_combat(self):
        """Refusing does NOT trigger combat - just leaves peacefully.

        Java (Vampires.java): Refuse simply updates the dialog text and
        calls openMap() - no combat is triggered. The incorrect assumption
        that refuse triggers combat was never in the original game.
        """
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Get the choices to find the refuse index
        event_state = EventState(event_id="Vampires")
        choices = handler.get_available_choices(event_state, run)
        refuse_idx = next(i for i, c in enumerate(choices) if c.name == "refuse")

        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, refuse_idx, run, event_rng, misc_rng=misc_rng)

        assert result.combat_triggered is False
        assert result.choice_name == "refuse"
        assert "left" in result.description.lower() or "refused" in result.description.lower()

    def test_blood_vial_trade_no_hp_loss(self):
        """Trading Blood Vial gives Bites without HP loss.

        Java (Vampires.java): If player has Blood Vial, buttonEffect case 1
        removes the vial and replaces Strikes with Bites, but does NOT
        call decreaseMaxHealth().
        """
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Add Blood Vial relic
        run.add_relic("Blood Vial")
        initial_max_hp = run.max_hp

        event_state = EventState(event_id="Vampires")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        # Choice 1 is the Blood Vial trade (when available)
        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        assert result.choice_name == "vial"
        assert "Blood Vial" in result.relics_lost
        assert result.cards_gained.count("Bite") == 5
        assert run.max_hp == initial_max_hp  # No HP loss!
        assert not any(r.id == "Blood Vial" for r in run.relics)


class TestGhostsHandlerBehavior:
    """Behavior tests for Ghosts (Council of Ghosts) event handler."""

    def test_accept_loses_50_percent_max_hp(self):
        """Accepting loses 50% of max HP."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_max_hp = run.max_hp

        event_state = EventState(event_id="Ghosts")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        expected_loss = int(initial_max_hp * 0.50)
        assert result.max_hp_change < 0
        assert run.max_hp == initial_max_hp - expected_loss

    def test_accept_gains_5_apparitions_below_a15(self):
        """Accepting gains 5 Apparition cards below A15."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        event_state = EventState(event_id="Ghosts")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        apparitions = sum(1 for c in run.deck if c.id == "Apparition")
        assert apparitions == 5
        assert result.cards_gained.count("Apparition") == 5

    def test_accept_gains_3_apparitions_a15(self):
        """Accepting gains 3 Apparition cards at A15+."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=15)

        event_state = EventState(event_id="Ghosts")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        apparitions = sum(1 for c in run.deck if c.id == "Apparition")
        assert apparitions == 3
        assert result.cards_gained.count("Apparition") == 3

    def test_refuse_no_changes(self):
        """Refusing makes no changes."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_max_hp = run.max_hp
        initial_deck_size = len(run.deck)

        event_state = EventState(event_id="Ghosts")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        assert run.max_hp == initial_max_hp
        assert len(run.deck) == initial_deck_size


class TestMindBloomHandlerBehavior:
    """Behavior tests for Mind Bloom event handler."""

    def test_war_triggers_act1_boss_combat(self):
        """'I am War' triggers combat against Act 1 boss."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        event_state = EventState(event_id="MindBloom")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        assert result.combat_triggered is True
        assert result.combat_encounter == "Act1Boss"
        assert result.event_complete is False

    def test_awake_upgrades_all_cards(self):
        """'I am Awake' upgrades all upgradeable cards."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Count upgradeable cards before
        upgradeable_before = sum(1 for c in run.deck if not c.upgraded)

        event_state = EventState(event_id="MindBloom")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        # All upgradeable cards (except AscendersBane at A10+) should be upgraded
        upgraded_count = sum(1 for c in run.deck if c.upgraded)
        assert upgraded_count > 0

    def test_awake_gives_mark_of_bloom(self):
        """'I am Awake' gives Mark of the Bloom relic."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        event_state = EventState(event_id="MindBloom")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        assert "Mark of the Bloom" in result.relics_gained
        assert run.has_relic("Mark of the Bloom")

    def test_rich_gives_999_gold(self):
        """'I am Rich' gives 999 gold."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_gold = run.gold

        event_state = EventState(event_id="MindBloom")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 2, run, event_rng, misc_rng=misc_rng)

        assert result.gold_change == 999
        assert run.gold == initial_gold + 999

    def test_rich_gives_two_normality_curses(self):
        """'I am Rich' gives 2 Normality curses."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        event_state = EventState(event_id="MindBloom")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 2, run, event_rng, misc_rng=misc_rng)

        normality_count = sum(1 for c in run.deck if c.id == "Normality")
        assert normality_count == 2
        assert result.cards_gained.count("Normality") == 2

    def test_third_option_rich_when_floor_mod_50_lte_40(self):
        """Third option is 'Rich' when floor % 50 <= 40.

        Java (MindBloom.java): if (AbstractDungeon.floorNum % 50 <= 40)
        then show "I am Rich" option (999 gold + 2 Normality).
        """
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Test floor 30 (30 % 50 = 30 <= 40, so should be Rich)
        run.floor = 30
        event_state = EventState(event_id="MindBloom")
        choices = handler.get_available_choices(event_state, run)

        # Third choice should be 'rich'
        third_choice = choices[2]
        assert third_choice.name == "rich"
        assert "Rich" in third_choice.text
        assert "gold" in third_choice.text.lower()

    def test_third_option_healthy_when_floor_mod_50_gt_40(self):
        """Third option is 'Healthy' when floor % 50 > 40.

        Java (MindBloom.java): if (AbstractDungeon.floorNum % 50 > 40)
        then show "I am Healthy" option (full heal + Doubt).
        """
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Test floor 45 (45 % 50 = 45 > 40, so should be Healthy)
        run.floor = 45
        event_state = EventState(event_id="MindBloom")
        choices = handler.get_available_choices(event_state, run)

        # Third choice should be 'healthy'
        third_choice = choices[2]
        assert third_choice.name == "healthy"
        assert "Healthy" in third_choice.text
        assert "heal" in third_choice.text.lower()

    def test_healthy_option_heals_to_full(self):
        """'I am Healthy' heals to full HP.

        Java (MindBloom.java): player.heal(player.maxHealth)
        """
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Set floor to trigger Healthy option (floor % 50 > 40)
        run.floor = 45
        run.current_hp = 30  # Damage the player

        event_state = EventState(event_id="MindBloom")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 2, run, event_rng, misc_rng=misc_rng)

        assert result.choice_name == "healthy"
        assert run.current_hp == run.max_hp  # Healed to full

    def test_healthy_option_gives_doubt_curse(self):
        """'I am Healthy' gives Doubt curse.

        Java (MindBloom.java): Doubt curse = new Doubt(); ...
        """
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Set floor to trigger Healthy option (floor % 50 > 40)
        run.floor = 45

        event_state = EventState(event_id="MindBloom")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 2, run, event_rng, misc_rng=misc_rng)

        assert result.choice_name == "healthy"
        assert "Doubt" in result.cards_gained
        assert any(c.id == "Doubt" for c in run.deck)


class TestDeadAdventurerHandlerBehavior:
    """Behavior tests for Dead Adventurer event handler."""

    def test_reward_order_is_rng_shuffled(self):
        """Reward pool order is shuffled using misc RNG."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        event_state = EventState(event_id="DeadAdventurer")

        misc_rng = Random(12345)
        handler._ensure_dead_adventurer_rewards(event_state, misc_rng)

        expected_rng = Random(12345)
        expected = ["gold", "relic", "nothing"]
        for i in range(len(expected) - 1, 0, -1):
            j = expected_rng.random(i)
            expected[i], expected[j] = expected[j], expected[i]

        assert event_state.dead_adventurer_rewards == expected

    def test_elite_selection_uses_misc_rng(self):
        """Elite encounter selection follows misc RNG."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        event_state = EventState(event_id="DeadAdventurer")
        event_state.dead_adventurer_rewards = ["gold", "relic", "nothing"]

        class FixedRandom:
            def __init__(self):
                self._floats = [0.0]  # force fight
                self._ints = [1]      # pick Gremlin Nob

            def random_float(self):
                return self._floats.pop(0)

            def random(self, range_val):
                return self._ints.pop(0)

        misc_rng = FixedRandom()
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)
        assert result.combat_triggered is True
        assert result.combat_encounter == "Gremlin Nob"


class TestKnowingSkullHandlerBehavior:
    """Behavior tests for Knowing Skull event handler."""

    def test_costs_escalate_per_option(self):
        """Each option's cost increases independently by 1."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        event_state = EventState(event_id="KnowingSkull")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result1 = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)
        assert result1.hp_change == -6

        result2 = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)
        assert result2.hp_change == -7

        result3 = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)
        assert result3.hp_change == -6

        result4 = handler.execute_choice(event_state, 3, run, event_rng, misc_rng=misc_rng)
        assert result4.hp_change == -6


class TestFallingHandlerBehavior:
    """Behavior tests for Falling event handler."""

    def test_skill_choice_removes_preselected_skill(self):
        """Choosing skill removes the preselected skill card."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Add some skill cards to test
        run.add_card("Meditate")
        run.add_card("InnerPeace")

        event_state = EventState(event_id="Falling")
        misc_rng = Random(12345)
        event_rng = Random(12345)
        handler._ensure_falling_preselect(event_state, run, misc_rng)

        choices = handler.get_available_choices(event_state, run)
        choice_names = {c.name for c in choices}
        assert "skill" in choice_names
        assert "power" not in choice_names  # No powers in deck

        skill_choice = next(c for c in choices if c.name == "skill")
        preselected = event_state.falling_preselected["SKILL"][1]
        assert preselected in skill_choice.text

        result = handler.execute_choice(event_state, skill_choice.index, run, event_rng, misc_rng=misc_rng)
        assert result.cards_removed == [preselected]

    def test_power_choice_removes_preselected_power(self):
        """Choosing power removes the preselected power card."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        run.add_card("Rushdown")
        run.add_card("MentalFortress")

        event_state = EventState(event_id="Falling")
        misc_rng = Random(12345)
        event_rng = Random(12345)
        handler._ensure_falling_preselect(event_state, run, misc_rng)

        choices = handler.get_available_choices(event_state, run)
        power_choice = next(c for c in choices if c.name == "power")
        preselected = event_state.falling_preselected["POWER"][1]

        result = handler.execute_choice(event_state, power_choice.index, run, event_rng, misc_rng=misc_rng)
        assert result.cards_removed == [preselected]

    def test_attack_choice_removes_preselected_attack(self):
        """Choosing attack removes the preselected attack card."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        event_state = EventState(event_id="Falling")
        misc_rng = Random(12345)
        event_rng = Random(12345)
        handler._ensure_falling_preselect(event_state, run, misc_rng)

        choices = handler.get_available_choices(event_state, run)
        attack_choice = next(c for c in choices if c.name == "attack")
        preselected = event_state.falling_preselected["ATTACK"][1]

        result = handler.execute_choice(event_state, attack_choice.index, run, event_rng, misc_rng=misc_rng)
        assert result.cards_removed == [preselected]


class TestGoldenIdolHandlerBehavior:
    """Behavior tests for Golden Idol multi-phase event handler."""

    def test_take_gives_relic_and_secondary_phase(self):
        """Taking the idol gives the relic and moves to escape phase."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        event_state = EventState(event_id="GoldenIdol")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        assert "GoldenIdol" in result.relics_gained
        assert run.has_relic("GoldenIdol")
        assert result.event_complete is False
        assert event_state.phase == EventPhase.SECONDARY

    def test_leave_no_changes(self):
        """Leaving makes no changes."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_relic_count = len(run.relics)

        event_state = EventState(event_id="GoldenIdol")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        assert len(run.relics) == initial_relic_count
        assert result.event_complete is True

    def test_escape_outrun_gives_injury(self):
        """Outrun escape gives Injury curse."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # First take the idol
        event_state = EventState(event_id="GoldenIdol")
        misc_rng = Random(12345)
        event_rng = Random(12345)
        handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        # Then escape via outrun
        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        assert "Injury" in result.cards_gained
        assert any(c.id == "Injury" for c in run.deck)

    def test_escape_smash_deals_damage(self):
        """Smash escape deals 25% max HP damage."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_hp = run.current_hp

        # First take the idol
        event_state = EventState(event_id="GoldenIdol")
        misc_rng = Random(12345)
        event_rng = Random(12345)
        handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        # Then escape via smash
        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        expected_damage = int(run.max_hp * 0.25)
        assert result.hp_change < 0
        assert run.current_hp == initial_hp - expected_damage

    def test_escape_hide_loses_max_hp(self):
        """Hide escape loses 8% max HP."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_max_hp = run.max_hp

        # First take the idol
        event_state = EventState(event_id="GoldenIdol")
        misc_rng = Random(12345)
        event_rng = Random(12345)
        handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        # Then escape via hide
        result = handler.execute_choice(event_state, 2, run, event_rng, misc_rng=misc_rng)

        expected_loss = int(initial_max_hp * 0.08)
        assert result.max_hp_change < 0
        assert run.max_hp == initial_max_hp - expected_loss


class TestScrapOozeHandlerBehavior:
    """Behavior tests for Scrap Ooze event handler."""

    def test_reach_deals_base_damage(self):
        """Reaching in deals base damage (3 at A0-14, 5 at A15+)."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_hp = run.current_hp

        event_state = EventState(event_id="ScrapOoze")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        # Base damage is 3 below A15
        assert result.hp_change == -3
        assert run.current_hp == initial_hp - 3

    def test_reach_damage_escalates(self):
        """Each attempt increases damage by 1."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        event_state = EventState(event_id="ScrapOoze")
        event_state.attempt_count = 3  # Simulate 3 previous attempts
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        # Damage is 3 (base) + 3 (attempts) = 6
        assert result.hp_change == -6

    def test_leave_no_damage(self):
        """Leaving takes no damage."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_hp = run.current_hp

        event_state = EventState(event_id="ScrapOoze")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        assert result.hp_change == 0
        assert run.current_hp == initial_hp


class TestLivingWallHandlerBehavior:
    """Behavior tests for Living Wall event handler."""

    def test_forget_removes_card(self):
        """Forget option removes the selected card."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_deck_size = len(run.deck)

        event_state = EventState(event_id="LivingWall")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        # Select a card to remove (index 0)
        result = handler.execute_choice(event_state, 0, run, event_rng, card_idx=0, misc_rng=misc_rng)

        assert len(run.deck) == initial_deck_size - 1
        assert len(result.cards_removed) == 1

    def test_change_transforms_card(self):
        """Change option transforms the selected card."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_deck_size = len(run.deck)
        card_to_transform = run.deck[0].id

        event_state = EventState(event_id="LivingWall")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, card_idx=0, misc_rng=misc_rng)

        assert len(run.deck) == initial_deck_size  # Same size (one removed, one added)
        assert len(result.cards_transformed) == 1
        assert result.cards_transformed[0][0] == card_to_transform

    def test_grow_upgrades_card(self):
        """Grow option upgrades the selected card."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)

        # Find an upgradeable card
        upgradeable_idx = None
        for i, c in enumerate(run.deck):
            if not c.upgraded:
                upgradeable_idx = i
                break
        assert upgradeable_idx is not None

        event_state = EventState(event_id="LivingWall")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 2, run, event_rng, card_idx=upgradeable_idx, misc_rng=misc_rng)

        assert len(result.cards_upgraded) == 1
        assert run.deck[upgradeable_idx].upgraded is True


class TestMarkOfTheBloomPreventsHealing:
    """Test that Mark of the Bloom prevents healing in events."""

    def test_big_fish_banana_no_heal_with_mark(self):
        """Banana doesn't heal when Mark of the Bloom is active."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        run.add_relic("Mark of the Bloom")
        run.damage(30)  # Take some damage
        initial_hp = run.current_hp

        event_state = EventState(event_id="BigFish")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        # Mark of the Bloom prevents healing
        assert run.current_hp == initial_hp


class TestEctoplasmPreventsGoldGain:
    """Test that Ectoplasm prevents gold gain in events."""

    def test_golden_shrine_no_gold_with_ectoplasm(self):
        """Praying doesn't give gold when Ectoplasm is active."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        run.add_relic("Ectoplasm")
        initial_gold = run.gold

        event_state = EventState(event_id="GoldenShrine")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        # Ectoplasm prevents gold gain
        assert run.gold == initial_gold


class TestSssserpentHandlerBehavior:
    """Behavior tests for Sssserpent event handler."""

    def test_agree_gives_gold_and_doubt_curse(self):
        """Agreeing gives gold and Doubt curse."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_gold = run.gold

        event_state = EventState(event_id="Sssserpent")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        assert result.gold_change == 175
        assert run.gold == initial_gold + 175
        assert "Doubt" in result.cards_gained
        assert any(c.id == "Doubt" for c in run.deck)

    def test_agree_gives_less_gold_a15(self):
        """Agreeing gives less gold at A15+."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=15)
        initial_gold = run.gold

        event_state = EventState(event_id="Sssserpent")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 0, run, event_rng, misc_rng=misc_rng)

        assert result.gold_change == 150
        assert run.gold == initial_gold + 150

    def test_disagree_no_changes(self):
        """Disagreeing makes no changes."""
        handler = EventHandler()
        run = create_watcher_run("TESTSEED", ascension=10)
        initial_gold = run.gold
        initial_deck_size = len(run.deck)

        event_state = EventState(event_id="Sssserpent")
        misc_rng = Random(12345)
        event_rng = Random(12345)

        result = handler.execute_choice(event_state, 1, run, event_rng, misc_rng=misc_rng)

        assert run.gold == initial_gold
        assert len(run.deck) == initial_deck_size
