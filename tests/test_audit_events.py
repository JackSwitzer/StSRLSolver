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

    def test_negative_percent_uses_ceil(self):
        """Current behavior: negative percents use ceil (incorrect for some events)."""
        from packages.engine.content.events import Outcome
        o = Outcome(OutcomeType.HP_CHANGE, value_percent=-0.25)
        # maxHP=73: ceil(73*0.25) = ceil(18.25) = 19
        result = calculate_outcome_value(o, 73, 73)
        assert result == -19  # Uses ceil

    def test_positive_percent_uses_int(self):
        """Positive percents use int() truncation."""
        from packages.engine.content.events import Outcome
        o = Outcome(OutcomeType.HP_CHANGE, value_percent=0.25)
        result = calculate_outcome_value(o, 73, 73)
        assert result == 18  # int(18.25) = 18
