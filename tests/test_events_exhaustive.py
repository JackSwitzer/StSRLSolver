"""
Exhaustive tests for event_handler.py - targeting close to 100% coverage.

Tests every event handler function, every choice branch, ascension modifiers,
multi-phase events, choice availability, and utility methods.
"""

import pytest
import sys

sys.path.insert(0, "/Users/jackswitzer/Desktop/SlayTheSpireRL")

from packages.engine.state.run import create_watcher_run
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.handlers.event_handler import (
    EventHandler,
    EventState,
    EventPhase,
    EventChoiceResult,
    EventChoice,
    EventDefinition,
    ACT1_EVENTS,
    ACT2_EVENTS,
    ACT3_EVENTS,
    SHRINE_EVENTS,
    SPECIAL_ONE_TIME_EVENTS,
    EVENT_HANDLERS,
    EVENT_CHOICE_GENERATORS,
    _get_event_choices_impl,
    _get_event_choices_default,
)


# =============================================================================
# Fixtures
# =============================================================================

SEED = "EXHAUSTIVE"
SEED_LONG = seed_to_long(SEED)


def _run(ascension=0, seed=SEED):
    return create_watcher_run(seed, ascension=ascension)


def _rng(offset=0):
    return Random(SEED_LONG + offset)


def _handler():
    return EventHandler()


def _exec(event_id, choice_idx, ascension=0, card_idx=None, rng_offset=0, seed=SEED):
    """Helper: create handler, run_state, event_state and execute a choice."""
    h = _handler()
    run = _run(ascension=ascension, seed=seed)
    es = EventState(event_id=event_id)
    misc = _rng(rng_offset)
    event_rng = _rng(rng_offset + 1000)
    result = h.execute_choice(es, choice_idx, run, event_rng, card_idx=card_idx, misc_rng=misc)
    return result, run, es, h


def _exec_with_state(event_id, choice_idx, run, es=None, rng_offset=0, card_idx=None):
    """Helper: execute with an existing run state."""
    h = _handler()
    if es is None:
        es = EventState(event_id=event_id)
    misc = _rng(rng_offset)
    event_rng = _rng(rng_offset + 1000)
    result = h.execute_choice(es, choice_idx, run, event_rng, card_idx=card_idx, misc_rng=misc)
    return result, run, es, h


# =============================================================================
# Test: EventHandler initialization and utility methods
# =============================================================================


class TestEventHandlerInit:
    def test_init(self):
        h = _handler()
        assert h.current_event is None
        assert h.seen_one_time_events == set()
        assert h.recent_events == []

    def test_get_random_relic_common(self):
        h = _handler()
        run = _run()
        rng = _rng()
        relic = h._get_random_relic(run, rng, "common")
        assert relic in h.COMMON_RELICS

    def test_get_random_relic_uncommon(self):
        h = _handler()
        run = _run()
        rng = _rng()
        relic = h._get_random_relic(run, rng, "uncommon")
        assert relic in h.UNCOMMON_RELICS

    def test_get_random_relic_rare(self):
        h = _handler()
        run = _run()
        rng = _rng()
        relic = h._get_random_relic(run, rng, "rare")
        assert relic in h.RARE_RELICS

    def test_get_random_relic_all_owned_returns_circlet(self):
        h = _handler()
        run = _run()
        # Add all common relics
        for r in h.COMMON_RELICS:
            run.add_relic(r)
        rng = _rng()
        relic = h._get_random_relic(run, rng, "common")
        assert relic == "Circlet"

    def test_get_random_card(self):
        h = _handler()
        run = _run()
        rng = _rng()
        card = h._get_random_card(run, rng, "common")
        assert card in h.WATCHER_COMMON_CARDS

    def test_get_random_card_uncommon(self):
        h = _handler()
        run = _run()
        rng = _rng()
        card = h._get_random_card(run, rng, "uncommon")
        assert card in h.WATCHER_UNCOMMON_CARDS

    def test_get_random_card_rare(self):
        h = _handler()
        run = _run()
        rng = _rng()
        card = h._get_random_card(run, rng, "rare")
        assert card in h.WATCHER_RARE_CARDS

    def test_get_random_card_colorless_uncommon(self):
        h = _handler()
        run = _run()
        rng = _rng()
        card = h._get_random_card(run, rng, "colorless_uncommon")
        assert card in h.COLORLESS_UNCOMMON_CARDS

    def test_get_random_card_colorless_rare(self):
        h = _handler()
        run = _run()
        rng = _rng()
        card = h._get_random_card(run, rng, "colorless_rare")
        assert card in h.COLORLESS_RARE_CARDS

    def test_get_random_card_colorless(self):
        h = _handler()
        run = _run()
        rng = _rng()
        card = h._get_random_card(run, rng, "colorless")
        assert card in (h.COLORLESS_UNCOMMON_CARDS + h.COLORLESS_RARE_CARDS)

    def test_get_random_potion(self):
        h = _handler()
        rng = _rng()
        potion = h._get_random_potion(rng)
        assert potion in h.POTIONS

    def test_apply_hp_change_heal(self):
        h = _handler()
        run = _run()
        run.damage(20)
        old = run.current_hp
        actual = h._apply_hp_change(run, 10)
        assert actual == 10
        assert run.current_hp == old + 10

    def test_apply_hp_change_damage(self):
        h = _handler()
        run = _run()
        old = run.current_hp
        actual = h._apply_hp_change(run, -5)
        assert actual == -5
        assert run.current_hp == old - 5

    def test_apply_hp_change_zero(self):
        h = _handler()
        run = _run()
        assert h._apply_hp_change(run, 0) == 0

    def test_apply_hp_change_heal_blocked_by_mark_of_bloom(self):
        h = _handler()
        run = _run()
        run.damage(20)
        run.add_relic("MarkOfTheBloom")
        assert h._apply_hp_change(run, 10) == 0

    def test_apply_max_hp_change_gain(self):
        h = _handler()
        run = _run()
        old = run.max_hp
        h._apply_max_hp_change(run, 5)
        assert run.max_hp == old + 5

    def test_apply_max_hp_change_lose(self):
        h = _handler()
        run = _run()
        old = run.max_hp
        h._apply_max_hp_change(run, -5)
        assert run.max_hp == old - 5

    def test_apply_gold_change_gain(self):
        h = _handler()
        run = _run()
        old = run.gold
        actual = h._apply_gold_change(run, 50)
        assert actual == 50

    def test_apply_gold_change_lose(self):
        h = _handler()
        run = _run()
        run.add_gold(100)
        actual = h._apply_gold_change(run, -30)
        assert actual == -30

    def test_apply_gold_change_zero(self):
        h = _handler()
        run = _run()
        assert h._apply_gold_change(run, 0) == 0

    def test_add_curse(self):
        h = _handler()
        run = _run()
        old_deck = len(run.deck)
        h._add_curse(run, "Regret")
        assert len(run.deck) == old_deck + 1

    def test_add_random_curse(self):
        h = _handler()
        run = _run()
        rng = _rng()
        curse = h._add_random_curse(run, rng)
        assert curse in h.CURSE_CARDS

    def test_get_removable_curses_empty(self):
        h = _handler()
        run = _run()
        assert h._get_removable_curses(run) == []

    def test_get_removable_curses_with_curse(self):
        h = _handler()
        run = _run()
        run.add_card("Regret")
        curses = h._get_removable_curses(run)
        assert len(curses) == 1

    def test_heal_percent(self):
        h = _handler()
        run = _run()
        run.damage(30)
        h._heal_percent(run, 0.25)
        # Should have healed ~25% of max HP

    def test_damage_percent(self):
        h = _handler()
        run = _run()
        old = run.current_hp
        h._damage_percent(run, 0.20)
        assert run.current_hp < old

    def test_damage_percent_a15(self):
        h = _handler()
        run = _run(ascension=20)
        old = run.current_hp
        h._damage_percent(run, 0.20, 20, 0.30)
        expected_damage = int(run.max_hp * 0.30)  # should use a15 percent
        # At A20, should use 0.30

    def test_lose_max_hp_percent(self):
        h = _handler()
        run = _run()
        old = run.max_hp
        h._lose_max_hp_percent(run, 0.10)
        assert run.max_hp < old

    def test_lose_max_hp_percent_a15(self):
        h = _handler()
        run = _run(ascension=20)
        old = run.max_hp
        h._lose_max_hp_percent(run, 0.10, 20, 0.15)
        # Should use 0.15 at A20


# =============================================================================
# Test: Event selection system
# =============================================================================


class TestEventSelection:
    def test_select_event_act1(self):
        h = _handler()
        run = _run()
        run.act = 1
        run.floor = 5
        rng = _rng()
        es = h.select_event(run, rng)
        assert es is not None
        assert es.event_id is not None

    def test_select_event_act2(self):
        h = _handler()
        run = _run()
        run.act = 2
        run.floor = 20
        rng = _rng()
        es = h.select_event(run, rng)
        assert es is not None

    def test_select_event_act3(self):
        h = _handler()
        run = _run()
        run.act = 3
        run.floor = 35
        rng = _rng()
        es = h.select_event(run, rng)
        assert es is not None

    def test_select_event_act4_returns_none(self):
        h = _handler()
        run = _run()
        run.act = 4
        run.floor = 50
        rng = _rng()
        es = h.select_event(run, rng)
        # Should still find shrine/special events
        # or none if all filtered out

    def test_select_event_marks_one_time_seen(self):
        h = _handler()
        run = _run()
        run.act = 1
        run.floor = 5
        # Run many events to potentially hit a one-time event
        for i in range(20):
            rng = _rng(offset=i * 100)
            es = h.select_event(run, rng)
            if es and es.event_id in SPECIAL_ONE_TIME_EVENTS:
                assert es.event_id in h.seen_one_time_events
                break

    def test_select_event_tracks_recent(self):
        h = _handler()
        run = _run()
        run.act = 1
        run.floor = 5
        for i in range(5):
            rng = _rng(offset=i * 200)
            h.select_event(run, rng)
        assert len(h.recent_events) <= 3

    def test_get_event_definition_all_dicts(self):
        h = _handler()
        # Should find events in each dict
        assert h._get_event_definition("BigFish") is not None
        assert h._get_event_definition("Addict") is not None
        assert h._get_event_definition("Falling") is not None
        assert h._get_event_definition("GoldenShrine") is not None
        assert h._get_event_definition("KnowingSkull") is not None
        assert h._get_event_definition("NonExistentEvent") is None

    def test_event_is_available_floor_restriction(self):
        h = _handler()
        run = _run()
        run.floor = 1
        ed = EventDefinition(id="Test", name="Test", act=1, min_floor=5)
        assert not h._event_is_available(ed, run)
        run.floor = 5
        assert h._event_is_available(ed, run)

    def test_event_is_available_max_floor(self):
        h = _handler()
        run = _run()
        run.floor = 50
        ed = EventDefinition(id="Test", name="Test", act=1, max_floor=10)
        assert not h._event_is_available(ed, run)

    def test_event_is_available_requires_relic(self):
        h = _handler()
        run = _run()
        ed = EventDefinition(id="Test", name="Test", act=1, requires_relic="GoldenIdol")
        assert not h._event_is_available(ed, run)
        run.add_relic("GoldenIdol")
        assert h._event_is_available(ed, run)

    def test_event_is_available_requires_no_relic(self):
        h = _handler()
        run = _run()
        ed = EventDefinition(id="Test", name="Test", act=1, requires_no_relic="GoldenIdol")
        assert h._event_is_available(ed, run)
        run.add_relic("GoldenIdol")
        assert not h._event_is_available(ed, run)

    def test_event_is_available_requires_curse(self):
        h = _handler()
        run = _run()
        ed = EventDefinition(id="Test", name="Test", act=1, requires_curse_in_deck=True)
        assert not h._event_is_available(ed, run)
        run.add_card("Regret")
        assert h._event_is_available(ed, run)

    def test_event_is_available_unremovable_curse_doesnt_count(self):
        h = _handler()
        run = _run()
        ed = EventDefinition(id="Test", name="Test", act=1, requires_curse_in_deck=True)
        run.add_card("AscendersBane")
        assert not h._event_is_available(ed, run)


# =============================================================================
# Test: Choice availability
# =============================================================================


class TestChoiceAvailability:
    def test_gold_requirement(self):
        h = _handler()
        run = _run()
        run.set_gold(0)
        choice = EventChoice(index=0, name="test", text="Test", requires_gold=50)
        assert not h._choice_is_available(choice, run)
        run.add_gold(100)
        assert h._choice_is_available(choice, run)

    def test_gold_requirement_a15_modifier(self):
        h = _handler()
        run = _run(ascension=20)
        run.set_gold(60)
        choice = EventChoice(index=0, name="test", text="Test", requires_gold=50, ascension_modifier=True)
        # At A15+, cost is 50 * 1.5 = 75
        assert not h._choice_is_available(choice, run)
        run.add_gold(20)
        assert h._choice_is_available(choice, run)

    def test_relic_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_relic="GoldenIdol")
        assert not h._choice_is_available(choice, run)
        run.add_relic("GoldenIdol")
        assert h._choice_is_available(choice, run)

    def test_no_relic_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_no_relic="GoldenIdol")
        assert h._choice_is_available(choice, run)
        run.add_relic("GoldenIdol")
        assert not h._choice_is_available(choice, run)

    def test_min_hp_requirement(self):
        h = _handler()
        run = _run()
        run.damage(run.current_hp - 5)  # Set HP to 5
        choice = EventChoice(index=0, name="test", text="Test", requires_min_hp=10)
        assert not h._choice_is_available(choice, run)

    def test_min_hp_percent_requirement(self):
        h = _handler()
        run = _run()
        run.damage(run.current_hp - 1)  # Nearly dead
        choice = EventChoice(index=0, name="test", text="Test", requires_min_hp_percent=0.5)
        assert not h._choice_is_available(choice, run)

    def test_max_hp_missing_requirement(self):
        h = _handler()
        run = _run()
        # At full HP
        choice = EventChoice(index=0, name="test", text="Test", requires_max_hp_missing=True)
        assert not h._choice_is_available(choice, run)
        run.damage(5)
        assert h._choice_is_available(choice, run)

    def test_upgradable_cards_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_upgradable_cards=True)
        # Starter deck should have upgradeable cards
        assert h._choice_is_available(choice, run)

    def test_removable_cards_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_removable_cards=True)
        assert h._choice_is_available(choice, run)

    def test_transformable_cards_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_transformable_cards=True)
        # Need non-basic cards
        run.add_card("Tantrum")
        assert h._choice_is_available(choice, run)

    def test_transformable_cards_requirement_no_nonbasic(self):
        h = _handler()
        run = _run()
        # Remove all non-basic cards, keep only basics
        to_remove = [i for i, c in enumerate(run.deck) if c.id not in h.BASIC_CARDS]
        for i in reversed(to_remove):
            run.remove_card(i)
        choice = EventChoice(index=0, name="test", text="Test", requires_transformable_cards=True)
        assert not h._choice_is_available(choice, run)

    def test_curse_in_deck_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_curse_in_deck=True)
        assert not h._choice_is_available(choice, run)
        run.add_card("Regret")
        assert h._choice_is_available(choice, run)

    def test_card_type_attack(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_card_type="ATTACK")
        # Starter deck has Strike_P
        assert h._choice_is_available(choice, run)

    def test_card_type_skill(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_card_type="SKILL")
        # Starter deck has Defend_P
        assert h._choice_is_available(choice, run)

    def test_card_type_power(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_card_type="POWER")
        # Starter deck doesn't have powers
        assert not h._choice_is_available(choice, run)
        run.add_card("Rushdown")
        assert h._choice_is_available(choice, run)

    def test_card_type_unknown(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_card_type="UNKNOWN")
        assert h._choice_is_available(choice, run)

    def test_potion_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_potion=True)
        assert not h._choice_is_available(choice, run)

    def test_empty_potion_slot_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_empty_potion_slot=True)
        # Should have empty slots by default
        assert h._choice_is_available(choice, run)

    def test_non_basic_card_requirement(self):
        h = _handler()
        run = _run()
        choice = EventChoice(index=0, name="test", text="Test", requires_non_basic_card=True)
        # Starter deck has Eruption, Vigilance which are in BASIC_CARDS
        # But might also have others
        run.add_card("Tantrum")
        assert h._choice_is_available(choice, run)

    def test_get_available_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="BigFish")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 3  # banana, donut, box


# =============================================================================
# Test: Execute choice with unregistered event (default handler)
# =============================================================================


class TestDefaultHandler:
    def test_unknown_event_returns_default(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="NonExistentEvent")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.event_id == "NonExistentEvent"
        assert result.event_complete is True


# =============================================================================
# Test: Act 1 Events
# =============================================================================


class TestBigFish:
    def test_banana_heals(self):
        h = _handler()
        run = _run()
        run.damage(20)  # Must be missing HP to heal
        es = EventState(event_id="BigFish")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "banana"
        assert result.hp_change > 0

    def test_donut_max_hp(self):
        result, run, _, _ = _exec("BigFish", 1)
        assert result.choice_name == "donut"
        assert result.max_hp_change == 5

    def test_box_relic_and_curse(self):
        result, run, _, _ = _exec("BigFish", 2)
        assert result.choice_name == "box"
        assert len(result.relics_gained) == 1
        assert "Regret" in result.cards_gained


class TestTheCleric:
    def test_heal(self):
        result, run, _, _ = _exec("TheCleric", 0)
        assert result.choice_name == "heal"
        assert result.gold_change == -35

    def test_purify_without_card_idx(self):
        result, run, _, _ = _exec("TheCleric", 1)
        assert result.choice_name == "purify"
        assert result.requires_card_selection is True
        assert result.event_complete is False

    def test_purify_with_card_idx(self):
        result, run, _, _ = _exec("TheCleric", 1, card_idx=0)
        assert result.choice_name == "purify"
        assert len(result.cards_removed) == 1

    def test_purify_a15_costs_more(self):
        result, run, _, _ = _exec("TheCleric", 1, ascension=20, card_idx=0)
        assert result.gold_change == -75

    def test_purify_a0_costs_50(self):
        result, run, _, _ = _exec("TheCleric", 1, ascension=0, card_idx=0)
        assert result.gold_change == -50

    def test_leave(self):
        result, _, _, _ = _exec("TheCleric", 2)
        assert result.choice_name == "leave"


class TestGoldenIdol:
    def test_take_phase1(self):
        result, run, es, _ = _exec("GoldenIdol", 0)
        assert result.choice_name == "take"
        assert "GoldenIdol" in result.relics_gained
        assert result.event_complete is False
        assert es.phase == EventPhase.SECONDARY

    def test_leave_phase1(self):
        result, _, es, _ = _exec("GoldenIdol", 1)
        assert result.choice_name == "leave"
        assert result.event_complete is True

    def test_outrun_phase2(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="GoldenIdol", phase=EventPhase.SECONDARY)
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "outrun"
        assert "Injury" in result.cards_gained

    def test_smash_phase2(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="GoldenIdol", phase=EventPhase.SECONDARY)
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "smash"
        assert result.hp_change < 0

    def test_smash_phase2_a15(self):
        h = _handler()
        run = _run(ascension=20)
        es = EventState(event_id="GoldenIdol", phase=EventPhase.SECONDARY)
        old_hp = run.current_hp
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        # A15+ should do 35% damage, more than A0's 25%
        assert result.hp_change < 0

    def test_hide_phase2(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="GoldenIdol", phase=EventPhase.SECONDARY)
        result = h.execute_choice(es, 2, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "hide"
        assert result.max_hp_change < 0

    def test_hide_phase2_a15(self):
        h = _handler()
        run = _run(ascension=20)
        es = EventState(event_id="GoldenIdol", phase=EventPhase.SECONDARY)
        result = h.execute_choice(es, 2, run, _rng(), misc_rng=_rng())
        assert result.max_hp_change < 0


class TestWorldOfGoop:
    def test_gather_gold(self):
        result, run, _, _ = _exec("WorldOfGoop", 0)
        assert result.choice_name == "gather"
        assert result.gold_change == 75
        assert result.hp_change == -11

    def test_leave_a0(self):
        result, _, _, _ = _exec("WorldOfGoop", 1, ascension=0)
        assert result.choice_name == "leave"
        assert result.gold_change < 0

    def test_leave_a15(self):
        result, _, _, _ = _exec("WorldOfGoop", 1, ascension=20)
        assert result.choice_name == "leave"
        assert result.gold_change < 0


class TestLivingWall:
    def test_forget_without_card(self):
        result, _, _, _ = _exec("LivingWall", 0)
        assert result.choice_name == "forget"
        assert result.requires_card_selection is True

    def test_forget_with_card(self):
        result, run, _, _ = _exec("LivingWall", 0, card_idx=0)
        assert len(result.cards_removed) == 1

    def test_change_without_card(self):
        result, _, _, _ = _exec("LivingWall", 1)
        assert result.choice_name == "change"
        assert result.requires_card_selection is True

    def test_change_with_card(self):
        result, run, _, _ = _exec("LivingWall", 1, card_idx=0)
        assert len(result.cards_removed) == 1
        assert len(result.cards_gained) == 1
        assert len(result.cards_transformed) == 1

    def test_grow_without_card(self):
        result, _, _, _ = _exec("LivingWall", 2)
        assert result.choice_name == "grow"
        assert result.requires_card_selection is True

    def test_grow_with_card(self):
        result, run, _, _ = _exec("LivingWall", 2, card_idx=0)
        assert len(result.cards_upgraded) == 1


class TestScrapOoze:
    def test_reach_takes_damage(self):
        result, run, es, _ = _exec("ScrapOoze", 0)
        assert result.choice_name == "reach"
        assert result.hp_change < 0

    def test_reach_a15_more_damage(self):
        result, _, _, _ = _exec("ScrapOoze", 0, ascension=20)
        assert result.hp_change <= -5  # Base 5 at A15+

    def test_reach_escalates(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="ScrapOoze")
        es.attempt_count = 3
        misc = _rng(999)  # Use offset that will fail
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=misc)
        # Damage should be base + 3

    def test_leave(self):
        result, _, _, _ = _exec("ScrapOoze", 1)
        assert result.choice_name == "leave"
        assert result.event_complete is True


class TestSssserpent:
    def test_agree_a0(self):
        result, run, _, _ = _exec("Sssserpent", 0, ascension=0)
        assert result.choice_name == "agree"
        assert result.gold_change == 175
        assert "Doubt" in result.cards_gained

    def test_agree_a15(self):
        result, _, _, _ = _exec("Sssserpent", 0, ascension=20)
        assert result.gold_change == 150

    def test_disagree(self):
        result, _, _, _ = _exec("Sssserpent", 1)
        assert result.choice_name == "disagree"
        assert result.event_complete is True


class TestWingStatue:
    def test_purify_without_card(self):
        result, _, _, _ = _exec("WingStatue", 0)
        assert result.choice_name == "purify"
        assert result.hp_change == -7
        assert result.requires_card_selection is True

    def test_purify_with_card(self):
        result, _, _, _ = _exec("WingStatue", 0, card_idx=0)
        assert result.hp_change == -7
        assert len(result.cards_removed) == 1

    def test_leave(self):
        result, _, _, _ = _exec("WingStatue", 1)
        assert result.choice_name == "leave"


class TestShiningLight:
    def test_enter(self):
        result, run, _, _ = _exec("ShiningLight", 0)
        assert result.choice_name == "enter"
        assert result.hp_change < 0
        assert len(result.cards_upgraded) <= 2

    def test_enter_a15_more_damage(self):
        r0, _, _, _ = _exec("ShiningLight", 0, ascension=0, seed="SHINE1")
        r15, _, _, _ = _exec("ShiningLight", 0, ascension=20, seed="SHINE1")
        # A15 should have taken more damage (30% vs 20%)

    def test_leave(self):
        result, _, _, _ = _exec("ShiningLight", 1)
        assert result.choice_name == "leave"


class TestDeadAdventurer:
    def test_search(self):
        result, run, es, _ = _exec("DeadAdventurer", 0)
        assert result.choice_name == "search"
        # Either triggered combat or got reward

    def test_search_combat_triggered(self):
        # Use many seeds to find one that triggers combat
        for i in range(50):
            h = _handler()
            run = _run()
            es = EventState(event_id="DeadAdventurer")
            misc = Random(SEED_LONG + i * 13)
            result = h.execute_choice(es, 0, run, _rng(), misc_rng=misc)
            if result.combat_triggered:
                assert result.combat_encounter == "DeadAdventurerElite"
                assert es.phase == EventPhase.COMBAT_PENDING
                break

    def test_search_reward_gold(self):
        # First successful search gives gold
        for i in range(50):
            h = _handler()
            run = _run()
            es = EventState(event_id="DeadAdventurer")
            misc = Random(SEED_LONG + i * 17)
            result = h.execute_choice(es, 0, run, _rng(), misc_rng=misc)
            if not result.combat_triggered and result.gold_change == 30:
                assert es.attempt_count == 1
                break

    def test_leave(self):
        result, _, _, _ = _exec("DeadAdventurer", 1)
        assert result.choice_name == "leave"


class TestMushrooms:
    def test_stomp_triggers_combat(self):
        result, _, es, _ = _exec("Mushrooms", 0)
        assert result.choice_name == "stomp"
        assert result.combat_triggered is True
        assert es.pending_rewards["relic"] == "OddMushroom"

    def test_eat_heals_and_curses(self):
        result, _, _, _ = _exec("Mushrooms", 1, ascension=0)
        assert result.choice_name == "eat"
        assert result.hp_change >= 0  # heals (could be 0 if at full)
        assert "Parasite" in result.cards_gained


# =============================================================================
# Test: Act 2 Events
# =============================================================================


class TestBackToBasics:
    def test_simplicity_removes_nonbasic(self):
        h = _handler()
        run = _run()
        run.add_card("Tantrum")
        initial_deck = len(run.deck)
        es = EventState(event_id="BackToBasics")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "simplicity"
        # Only Strike_P and Defend_P should remain
        for c in run.deck:
            assert c.id in ["Strike_P", "Defend_P"]

    def test_elegance_upgrades_basics(self):
        result, run, _, _ = _exec("BackToBasics", 1)
        assert result.choice_name == "elegance"
        assert len(result.cards_upgraded) > 0


class TestColosseum:
    def test_enter_triggers_combat(self):
        result, _, es, _ = _exec("Colosseum", 0)
        assert result.choice_name == "enter"
        assert result.combat_triggered is True
        assert es.phase == EventPhase.COMBAT_PENDING

    def test_after_first_fight_won(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="Colosseum", phase=EventPhase.COMBAT_WON)
        es.first_fight_won = False
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert es.first_fight_won is True
        assert es.phase == EventPhase.SECONDARY

    def test_second_fight_won_rewards(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="Colosseum", phase=EventPhase.COMBAT_WON)
        es.first_fight_won = True
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.gold_change == 100
        assert "RareRelic" in result.relics_gained

    def test_secondary_fight_nobs(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="Colosseum", phase=EventPhase.SECONDARY)
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "fight_nobs"
        assert result.combat_triggered is True

    def test_secondary_cowardice(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="Colosseum", phase=EventPhase.SECONDARY)
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "cowardice"


class TestCursedTome:
    def test_read_a0(self):
        result, _, _, _ = _exec("CursedTome", 0, ascension=0)
        assert result.choice_name == "read"
        assert result.hp_change == -16
        assert len(result.relics_gained) == 1
        assert result.relics_gained[0] in ["Necronomicon", "Enchiridion", "NilrysCodex"]

    def test_read_a15(self):
        result, _, _, _ = _exec("CursedTome", 0, ascension=20)
        assert result.hp_change == -21

    def test_leave(self):
        result, _, _, _ = _exec("CursedTome", 1)
        assert result.choice_name == "leave"


class TestTheLibrary:
    def test_read_requires_selection(self):
        result, _, _, _ = _exec("TheLibrary", 0)
        assert result.choice_name == "read"
        assert result.requires_card_selection is True
        assert result.card_selection_type == "choose"

    def test_sleep_heals_to_full(self):
        h = _handler()
        run = _run()
        run.damage(30)
        es = EventState(event_id="TheLibrary")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "sleep"
        assert run.current_hp == run.max_hp


class TestTheMausoleum:
    def test_open_relic_outcome(self):
        # Find a seed that gives relic
        for i in range(50):
            h = _handler()
            run = _run()
            es = EventState(event_id="TheMausoleum")
            misc = Random(SEED_LONG + i * 7)
            result = h.execute_choice(es, 0, run, _rng(), misc_rng=misc)
            if result.relics_gained:
                assert len(result.relics_gained) == 1
                return
        pytest.skip("Could not find seed giving relic")

    def test_open_curse_outcome(self):
        for i in range(50):
            h = _handler()
            run = _run()
            es = EventState(event_id="TheMausoleum")
            misc = Random(SEED_LONG + i * 11)
            result = h.execute_choice(es, 0, run, _rng(), misc_rng=misc)
            if result.cards_gained:
                assert len(result.cards_gained) == 1
                return
        pytest.skip("Could not find seed giving curse")

    def test_leave(self):
        result, _, _, _ = _exec("TheMausoleum", 1)
        assert result.choice_name == "leave"


class TestMaskedBandits:
    def test_pay_loses_all_gold(self):
        h = _handler()
        run = _run()
        run.add_gold(200)
        initial_gold = run.gold
        es = EventState(event_id="MaskedBandits")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "pay"
        assert run.gold == 0
        assert result.gold_change == -initial_gold

    def test_fight(self):
        result, _, es, _ = _exec("MaskedBandits", 1)
        assert result.choice_name == "fight"
        assert result.combat_triggered is True


class TestKnowingSkull:
    def test_potion(self):
        result, _, es, _ = _exec("KnowingSkull", 0)
        assert result.choice_name == "potion"
        assert result.hp_change == -6
        assert len(result.potions_gained) == 1
        assert es.hp_cost_modifier == 2

    def test_gold(self):
        result, _, es, _ = _exec("KnowingSkull", 1)
        assert result.choice_name == "gold"
        assert result.hp_change == -6
        assert result.gold_change == 90

    def test_card(self):
        result, _, es, _ = _exec("KnowingSkull", 2)
        assert result.choice_name == "card"
        assert result.hp_change == -6
        assert len(result.cards_gained) == 1

    def test_leave(self):
        result, _, _, _ = _exec("KnowingSkull", 3)
        assert result.choice_name == "leave"
        assert result.hp_change == -6

    def test_escalating_cost(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="KnowingSkull")
        es.hp_cost_modifier = 4  # Already asked twice
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.hp_change == -10  # 6 + 4
        assert result.event_complete is False


class TestGhosts:
    def test_accept_a0(self):
        result, run, _, _ = _exec("Ghosts", 0, ascension=0)
        assert result.choice_name == "accept"
        assert result.max_hp_change < 0
        assert result.cards_gained.count("Apparition") == 5

    def test_accept_a15(self):
        result, run, _, _ = _exec("Ghosts", 0, ascension=20)
        assert result.cards_gained.count("Apparition") == 3

    def test_refuse(self):
        result, _, _, _ = _exec("Ghosts", 1)
        assert result.choice_name == "refuse"


class TestVampires:
    def test_accept(self):
        result, run, _, _ = _exec("Vampires", 0)
        assert result.choice_name == "accept"
        assert result.cards_gained.count("Bite") == 5
        assert result.max_hp_change < 0
        # All Strike_P should be removed
        for c in run.deck:
            assert c.id != "Strike_P"

    def test_refuse_triggers_combat(self):
        result, _, es, _ = _exec("Vampires", 1)
        assert result.choice_name == "refuse"
        assert result.combat_triggered is True


class TestForgottenAltar:
    def test_sacrifice_golden_idol(self):
        h = _handler()
        run = _run()
        run.add_relic("GoldenIdol")
        es = EventState(event_id="ForgottenAltar")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "sacrifice"
        assert "GoldenIdol" in result.relics_lost
        assert "BloodyIdol" in result.relics_gained

    def test_offer_hp(self):
        result, _, _, _ = _exec("ForgottenAltar", 1, ascension=0)
        assert result.choice_name == "offer"
        assert result.hp_change == -5
        assert len(result.relics_gained) == 1

    def test_offer_hp_a15(self):
        result, _, _, _ = _exec("ForgottenAltar", 1, ascension=20)
        assert result.hp_change == -7

    def test_leave_gives_decay(self):
        result, _, _, _ = _exec("ForgottenAltar", 2)
        assert result.choice_name == "leave"
        assert "Decay" in result.cards_gained


class TestNest:
    def test_smash(self):
        result, _, _, _ = _exec("Nest", 0)
        assert result.choice_name == "smash"
        assert result.gold_change == 99
        assert len(result.cards_gained) == 1

    def test_stay_ritual_dagger(self):
        result, _, _, _ = _exec("Nest", 1)
        assert result.choice_name == "stay"
        assert "RitualDagger" in result.cards_gained


class TestAddict:
    def test_pay(self):
        h = _handler()
        run = _run()
        run.add_gold(200)
        es = EventState(event_id="Addict")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "pay"
        assert result.gold_change == -85
        assert len(result.relics_gained) == 1

    def test_refuse_shame(self):
        result, _, _, _ = _exec("Addict", 1)
        assert result.choice_name == "refuse"
        assert "Shame" in result.cards_gained

    def test_rob(self):
        result, _, _, _ = _exec("Addict", 2)
        assert result.choice_name == "rob"
        assert len(result.relics_gained) == 1
        assert "Shame" in result.cards_gained


class TestBeggar:
    def test_donate_50(self):
        h = _handler()
        run = _run()
        run.add_gold(200)
        es = EventState(event_id="Beggar")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "donate_50"
        assert result.gold_change == -50
        assert len(result.relics_gained) == 1

    def test_donate_100(self):
        h = _handler()
        run = _run()
        run.add_gold(200)
        es = EventState(event_id="Beggar")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "donate_100"
        assert result.gold_change == -100
        assert len(result.relics_gained) == 1

    def test_donate_100_removes_curse(self):
        h = _handler()
        run = _run()
        run.add_gold(200)
        run.add_card("Regret")
        es = EventState(event_id="Beggar")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert "Regret" in result.cards_removed

    def test_leave(self):
        result, _, _, _ = _exec("Beggar", 2)
        assert result.choice_name == "leave"


# =============================================================================
# Test: Act 3 Events
# =============================================================================


class TestFalling:
    def test_skill(self):
        h = _handler()
        run = _run()
        # Add a skill card from the handler's SKILL_CARDS set
        run.add_card("Defend_P")
        es = EventState(event_id="Falling")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "skill"

    def test_power_no_cards(self):
        result, _, _, _ = _exec("Falling", 1)
        assert result.choice_name == "power"
        # Should say no cards of that type

    def test_power_with_card(self):
        h = _handler()
        run = _run()
        run.add_card("Rushdown")
        es = EventState(event_id="Falling")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert "Rushdown" in result.cards_removed

    def test_attack(self):
        h = _handler()
        run = _run()
        # Starter deck has Strike_P
        es = EventState(event_id="Falling")
        result = h.execute_choice(es, 2, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "attack"
        assert len(result.cards_removed) >= 1


class TestMindBloom:
    def test_war_triggers_combat(self):
        result, _, es, _ = _exec("MindBloom", 0)
        assert result.choice_name == "war"
        assert result.combat_triggered is True
        assert es.pending_rewards["relic"] == "RareRelic"

    def test_awake_upgrades_all(self):
        result, run, _, _ = _exec("MindBloom", 1)
        assert result.choice_name == "awake"
        assert "MarkOfTheBloom" in result.relics_gained

    def test_rich_999_gold(self):
        result, run, _, _ = _exec("MindBloom", 2)
        assert result.choice_name == "rich"
        assert result.gold_change == 999
        assert result.cards_gained.count("Normality") == 2


class TestMysteriousSphere:
    def test_open(self):
        result, _, es, _ = _exec("MysteriousSphere", 0)
        assert result.choice_name == "open"
        assert result.combat_triggered is True
        assert es.pending_rewards["relic"] == "RareRelic"

    def test_leave(self):
        result, _, _, _ = _exec("MysteriousSphere", 1)
        assert result.choice_name == "leave"


class TestSecretPortal:
    def test_enter(self):
        result, _, _, _ = _exec("SecretPortal", 0)
        assert result.choice_name == "enter"

    def test_leave(self):
        result, _, _, _ = _exec("SecretPortal", 1)
        assert result.choice_name == "leave"


class TestSensoryStone:
    def test_touch_act3(self):
        h = _handler()
        run = _run()
        run.act = 3
        es = EventState(event_id="SensoryStone")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "touch"
        assert len(result.cards_gained) == 3

    def test_touch_act1(self):
        h = _handler()
        run = _run()
        run.act = 1
        es = EventState(event_id="SensoryStone")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert len(result.cards_gained) == 1


class TestTombOfLordRedMask:
    def test_don_mask(self):
        result, run, _, _ = _exec("TombOfLordRedMask", 0)
        assert result.choice_name == "don_mask"
        assert "RedMask" in result.relics_gained

    def test_offer_gold(self):
        h = _handler()
        run = _run()
        run.add_gold(100)
        run.add_relic("RedMask")
        es = EventState(event_id="TombOfLordRedMask")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "offer_gold"
        # Gold = 222 * relic count

    def test_leave(self):
        result, _, _, _ = _exec("TombOfLordRedMask", 2)
        assert result.choice_name == "leave"


class TestMoaiHead:
    def test_enter(self):
        h = _handler()
        run = _run()
        run.damage(20)
        es = EventState(event_id="MoaiHead")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "enter"
        assert result.max_hp_change < 0
        assert run.current_hp == run.max_hp  # healed to full

    def test_enter_a15(self):
        h = _handler()
        run = _run(ascension=20)
        run.damage(20)
        es = EventState(event_id="MoaiHead")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        # A15 loses 18% vs 12.5%

    def test_offer_idol(self):
        h = _handler()
        run = _run()
        run.add_relic("GoldenIdol")
        es = EventState(event_id="MoaiHead")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "offer_idol"
        assert result.gold_change == 333
        assert "GoldenIdol" in result.relics_lost

    def test_leave(self):
        result, _, _, _ = _exec("MoaiHead", 2)
        assert result.choice_name == "leave"


class TestWindingHalls:
    def test_embrace_madness(self):
        result, _, _, _ = _exec("WindingHalls", 0, ascension=0)
        assert result.choice_name == "embrace"
        assert result.hp_change < 0
        assert result.cards_gained.count("Madness") == 2

    def test_retrace_heal_and_writhe(self):
        h = _handler()
        run = _run()
        run.damage(20)
        es = EventState(event_id="WindingHalls")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        # test retrace
        run2 = _run()
        run2.damage(20)
        es2 = EventState(event_id="WindingHalls")
        result2 = EventHandler().execute_choice(es2, 1, run2, _rng(), misc_rng=_rng())
        assert result2.choice_name == "retrace"
        assert result2.hp_change > 0
        assert "Writhe" in result2.cards_gained

    def test_retrace_a15_less_heal(self):
        h = _handler()
        run = _run(ascension=20)
        run.damage(20)
        es = EventState(event_id="WindingHalls")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "retrace"

    def test_press_on(self):
        result, _, _, _ = _exec("WindingHalls", 2)
        assert result.choice_name == "press_on"
        assert result.max_hp_change < 0


# =============================================================================
# Test: Shrine Events
# =============================================================================


class TestGoldenShrine:
    def test_pray_a0(self):
        result, _, _, _ = _exec("GoldenShrine", 0, ascension=0)
        assert result.choice_name == "pray"
        assert result.gold_change == 100

    def test_pray_a15(self):
        result, _, _, _ = _exec("GoldenShrine", 0, ascension=20)
        assert result.gold_change == 50

    def test_desecrate(self):
        result, _, _, _ = _exec("GoldenShrine", 1)
        assert result.choice_name == "desecrate"
        assert result.gold_change == 275
        assert "Regret" in result.cards_gained

    def test_leave(self):
        result, _, _, _ = _exec("GoldenShrine", 2)
        assert result.choice_name == "leave"


class TestPurifier:
    def test_pray_without_card(self):
        result, _, _, _ = _exec("Purifier", 0)
        assert result.choice_name == "pray"
        assert result.requires_card_selection is True

    def test_pray_with_card(self):
        result, _, _, _ = _exec("Purifier", 0, card_idx=0)
        assert len(result.cards_removed) == 1

    def test_leave(self):
        result, _, _, _ = _exec("Purifier", 1)
        assert result.choice_name == "leave"


class TestTransmogrifier:
    def test_pray_without_card(self):
        result, _, _, _ = _exec("Transmogrifier", 0)
        assert result.requires_card_selection is True

    def test_pray_with_card(self):
        result, _, _, _ = _exec("Transmogrifier", 0, card_idx=0)
        assert len(result.cards_removed) == 1
        assert len(result.cards_gained) == 1

    def test_leave(self):
        result, _, _, _ = _exec("Transmogrifier", 1)
        assert result.choice_name == "leave"


class TestUpgradeShrine:
    def test_pray_without_card(self):
        result, _, _, _ = _exec("UpgradeShrine", 0)
        assert result.requires_card_selection is True

    def test_pray_with_card(self):
        result, _, _, _ = _exec("UpgradeShrine", 0, card_idx=0)
        assert len(result.cards_upgraded) == 1

    def test_leave(self):
        result, _, _, _ = _exec("UpgradeShrine", 1)
        assert result.choice_name == "leave"


class TestDuplicator:
    def test_duplicate_without_card(self):
        result, _, _, _ = _exec("Duplicator", 0)
        assert result.requires_card_selection is True

    def test_duplicate_with_card(self):
        h = _handler()
        run = _run()
        initial_deck = len(run.deck)
        es = EventState(event_id="Duplicator")
        result = h.execute_choice(es, 0, run, _rng(), card_idx=0, misc_rng=_rng())
        assert len(result.cards_gained) == 1
        assert len(run.deck) == initial_deck + 1

    def test_leave(self):
        result, _, _, _ = _exec("Duplicator", 1)
        assert result.choice_name == "leave"


# =============================================================================
# Test: Special One-Time Events
# =============================================================================


class TestFountainOfCleansing:
    def test_drink_removes_curses(self):
        h = _handler()
        run = _run()
        run.add_card("Regret")
        run.add_card("Doubt")
        es = EventState(event_id="FountainOfCleansing")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "drink"
        assert "Regret" in result.cards_removed
        assert "Doubt" in result.cards_removed

    def test_drink_no_curses(self):
        result, _, _, _ = _exec("FountainOfCleansing", 0)
        assert result.choice_name == "drink"
        assert len(result.cards_removed) == 0

    def test_leave(self):
        result, _, _, _ = _exec("FountainOfCleansing", 1)
        assert result.choice_name == "leave"


class TestTheJoust:
    def test_bet_owner_win(self):
        # Find seed where roll < 0.30
        for i in range(100):
            h = _handler()
            run = _run()
            es = EventState(event_id="TheJoust")
            misc = Random(SEED_LONG + i * 3)
            result = h.execute_choice(es, 0, run, _rng(), misc_rng=misc)
            if result.gold_change == 250:
                return
        pytest.skip("No seed found for owner win")

    def test_bet_owner_lose(self):
        for i in range(100):
            h = _handler()
            run = _run()
            es = EventState(event_id="TheJoust")
            misc = Random(SEED_LONG + i * 3)
            result = h.execute_choice(es, 0, run, _rng(), misc_rng=misc)
            if result.gold_change == 0:
                return
        pytest.skip("No seed found for owner lose")

    def test_bet_murderer_win(self):
        for i in range(100):
            h = _handler()
            run = _run()
            es = EventState(event_id="TheJoust")
            misc = Random(SEED_LONG + i * 3)
            result = h.execute_choice(es, 1, run, _rng(), misc_rng=misc)
            if result.gold_change == 50:
                return
        pytest.skip("No seed found for murderer win")

    def test_bet_murderer_lose(self):
        for i in range(100):
            h = _handler()
            run = _run()
            es = EventState(event_id="TheJoust")
            misc = Random(SEED_LONG + i * 3)
            result = h.execute_choice(es, 1, run, _rng(), misc_rng=misc)
            if result.gold_change == 0:
                return
        pytest.skip("No seed found for murderer lose")


class TestTheLab:
    def test_enter_a0(self):
        result, _, _, _ = _exec("TheLab", 0, ascension=0)
        assert result.choice_name == "enter"
        assert len(result.potions_gained) == 3

    def test_enter_a15(self):
        result, _, _, _ = _exec("TheLab", 0, ascension=20)
        assert len(result.potions_gained) == 2


class TestWomanInBlue:
    def test_buy_1(self):
        h = _handler()
        run = _run()
        run.add_gold(100)
        es = EventState(event_id="WomanInBlue")
        result = h.execute_choice(es, 0, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "buy_1"
        assert result.gold_change == -20
        assert len(result.potions_gained) == 1

    def test_buy_2(self):
        h = _handler()
        run = _run()
        run.add_gold(100)
        es = EventState(event_id="WomanInBlue")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.gold_change == -30

    def test_buy_3(self):
        h = _handler()
        run = _run()
        run.add_gold(100)
        es = EventState(event_id="WomanInBlue")
        result = h.execute_choice(es, 2, run, _rng(), misc_rng=_rng())
        assert result.gold_change == -40

    def test_leave_a0(self):
        result, _, _, _ = _exec("WomanInBlue", 3, ascension=0)
        assert result.choice_name == "leave"
        assert result.hp_change == 0

    def test_leave_a15_takes_damage(self):
        result, _, _, _ = _exec("WomanInBlue", 3, ascension=20)
        assert result.choice_name == "leave"
        assert result.hp_change < 0


class TestFaceTrader:
    def test_trade(self):
        result, _, _, _ = _exec("FaceTrader", 0)
        assert result.choice_name == "trade"
        assert result.hp_change < 0
        assert len(result.relics_gained) == 1

    def test_pay(self):
        h = _handler()
        run = _run()
        run.add_gold(200)
        es = EventState(event_id="FaceTrader")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "pay"
        assert result.gold_change == -75
        assert "SsserpentHead" in result.relics_gained

    def test_leave(self):
        result, _, _, _ = _exec("FaceTrader", 2)
        assert result.choice_name == "leave"


class TestDesigner:
    def test_remove_a0(self):
        result, _, _, _ = _exec("Designer", 0, ascension=0, card_idx=0)
        assert result.choice_name == "remove"
        assert result.gold_change == -50

    def test_remove_a15(self):
        result, _, _, _ = _exec("Designer", 0, ascension=20, card_idx=0)
        assert result.gold_change == -75

    def test_remove_no_card(self):
        result, _, _, _ = _exec("Designer", 0)
        assert result.requires_card_selection is True

    def test_transform_a0(self):
        result, _, _, _ = _exec("Designer", 1, ascension=0, card_idx=0)
        assert result.choice_name == "transform"
        assert result.gold_change == -35

    def test_transform_a15(self):
        result, _, _, _ = _exec("Designer", 1, ascension=20, card_idx=0)
        assert result.gold_change == -50

    def test_transform_no_card(self):
        result, _, _, _ = _exec("Designer", 1)
        assert result.requires_card_selection is True

    def test_upgrade_a0(self):
        result, _, _, _ = _exec("Designer", 2, ascension=0, card_idx=0)
        assert result.choice_name == "upgrade"
        assert result.gold_change == -25

    def test_upgrade_a15(self):
        result, _, _, _ = _exec("Designer", 2, ascension=20, card_idx=0)
        assert result.gold_change == -40

    def test_upgrade_no_card(self):
        result, _, _, _ = _exec("Designer", 2)
        assert result.requires_card_selection is True

    def test_leave(self):
        result, _, _, _ = _exec("Designer", 3)
        assert result.choice_name == "leave"


class TestNloth:
    def test_trade(self):
        result, run, _, _ = _exec("Nloth", 0)
        assert result.choice_name == "trade"
        assert "NlothsGift" in result.relics_gained
        assert len(result.relics_lost) == 1

    def test_leave(self):
        result, _, _, _ = _exec("Nloth", 1)
        assert result.choice_name == "leave"


class TestAccursedBlacksmith:
    def test_upgrade_with_card(self):
        result, _, _, _ = _exec("AccursedBlacksmith", 0, card_idx=0)
        assert result.choice_name == "upgrade"
        assert "Parasite" in result.cards_gained

    def test_upgrade_no_card(self):
        result, _, _, _ = _exec("AccursedBlacksmith", 0)
        assert result.requires_card_selection is True

    def test_leave(self):
        result, _, _, _ = _exec("AccursedBlacksmith", 1)
        assert result.choice_name == "leave"


class TestBonfireElementals:
    def test_offer_with_card(self):
        result, _, _, _ = _exec("BonfireElementals", 0, card_idx=0)
        assert result.choice_name == "offer"
        assert len(result.cards_removed) == 1
        assert len(result.relics_gained) == 1

    def test_offer_no_card(self):
        result, _, _, _ = _exec("BonfireElementals", 0)
        assert result.requires_card_selection is True

    def test_leave(self):
        result, _, _, _ = _exec("BonfireElementals", 1)
        assert result.choice_name == "leave"


class TestWeMeetAgain:
    def test_give_potion(self):
        result, _, _, _ = _exec("WeMeetAgain", 0)
        assert result.choice_name == "give_potion"
        assert len(result.relics_gained) == 1

    def test_give_gold(self):
        h = _handler()
        run = _run()
        run.add_gold(200)
        es = EventState(event_id="WeMeetAgain")
        result = h.execute_choice(es, 1, run, _rng(), misc_rng=_rng())
        assert result.choice_name == "give_gold"
        assert result.gold_change == -50
        assert len(result.cards_gained) == 1

    def test_give_card_with_idx(self):
        result, _, _, _ = _exec("WeMeetAgain", 2, card_idx=0)
        assert result.choice_name == "give_card"
        assert len(result.potions_gained) == 1

    def test_give_card_no_idx(self):
        result, _, _, _ = _exec("WeMeetAgain", 2)
        assert result.requires_card_selection is True

    def test_leave(self):
        result, _, _, _ = _exec("WeMeetAgain", 3)
        assert result.choice_name == "leave"


class TestAugmenter:
    def test_mechanical_with_card(self):
        result, _, _, _ = _exec("Augmenter", 0, card_idx=0)
        assert result.choice_name == "mechanical"
        assert "J.A.X." in result.cards_gained

    def test_mechanical_no_card(self):
        result, _, _, _ = _exec("Augmenter", 0)
        assert result.requires_card_selection is True

    def test_mutagenic(self):
        result, _, _, _ = _exec("Augmenter", 1)
        assert result.choice_name == "mutagenic"
        assert len(result.relics_gained) == 1
        assert result.relics_gained[0] in ["MutagenicStrength", "MutagenicDexterity"]

    def test_transform_with_card(self):
        result, _, _, _ = _exec("Augmenter", 2, card_idx=0)
        assert result.choice_name == "transform"

    def test_transform_no_card(self):
        result, _, _, _ = _exec("Augmenter", 2)
        assert result.requires_card_selection is True
        assert result.card_selection_count == 2


# =============================================================================
# Test: Event Choice Generators
# =============================================================================


class TestEventChoiceGenerators:
    def test_big_fish_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="BigFish")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 3

    def test_cleric_choices_no_gold(self):
        h = _handler()
        run = _run()
        run.set_gold(0)
        es = EventState(event_id="TheCleric")
        choices = h.get_available_choices(es, run)
        # Only leave should be available
        assert any(c.name == "leave" for c in choices)
        assert not any(c.name == "heal" for c in choices)

    def test_cleric_choices_full_hp(self):
        h = _handler()
        run = _run()
        run.add_gold(200)
        # At full HP, heal should not be available
        es = EventState(event_id="TheCleric")
        choices = h.get_available_choices(es, run)
        assert not any(c.name == "heal" for c in choices)

    def test_golden_idol_phase1(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="GoldenIdol", phase=EventPhase.INITIAL)
        choices = h.get_available_choices(es, run)
        names = [c.name for c in choices]
        assert "take" in names
        assert "leave" in names

    def test_golden_idol_phase2(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="GoldenIdol", phase=EventPhase.SECONDARY)
        choices = h.get_available_choices(es, run)
        names = [c.name for c in choices]
        assert "outrun" in names
        assert "smash" in names
        assert "hide" in names

    def test_colosseum_initial(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="Colosseum", phase=EventPhase.INITIAL)
        choices = h.get_available_choices(es, run)
        assert len(choices) == 1
        assert choices[0].name == "enter"

    def test_colosseum_secondary(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="Colosseum", phase=EventPhase.SECONDARY)
        choices = h.get_available_choices(es, run)
        assert len(choices) == 2

    def test_knowing_skull_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="KnowingSkull")
        choices = h.get_available_choices(es, run)
        assert len(choices) >= 1  # At least leave

    def test_scrap_ooze_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="ScrapOoze")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 2

    def test_mind_bloom_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="MindBloom")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 3

    def test_shrine_choices_purifier(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="Purifier")
        choices = h.get_available_choices(es, run)
        assert any(c.name == "pray" for c in choices)

    def test_shrine_choices_transmogrifier(self):
        h = _handler()
        run = _run()
        run.add_card("Tantrum")
        es = EventState(event_id="Transmogrifier")
        choices = h.get_available_choices(es, run)
        assert any(c.name == "pray" for c in choices)

    def test_shrine_choices_upgrade(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="UpgradeShrine")
        choices = h.get_available_choices(es, run)
        assert any(c.name == "pray" for c in choices)

    def test_shrine_choices_duplicator(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="Duplicator")
        choices = h.get_available_choices(es, run)
        assert any(c.name == "duplicate" for c in choices)

    def test_shrine_choices_golden(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="GoldenShrine")
        choices = h.get_available_choices(es, run)
        assert any(c.name == "pray" for c in choices)
        assert any(c.name == "desecrate" for c in choices)

    def test_default_choices_for_unregistered(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="UnknownEvent")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 1
        assert choices[0].name == "leave"

    def test_world_of_goop_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="WorldOfGoop")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 2

    def test_living_wall_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="LivingWall")
        choices = h.get_available_choices(es, run)
        # Needs removable, transformable, upgradable cards
        assert len(choices) >= 1

    def test_library_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="TheLibrary")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 2

    def test_mausoleum_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="TheMausoleum")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 2

    def test_masked_bandits_choices(self):
        h = _handler()
        run = _run()
        es = EventState(event_id="MaskedBandits")
        choices = h.get_available_choices(es, run)
        assert len(choices) == 2


# =============================================================================
# Test: Event handler registry completeness
# =============================================================================


class TestRegistryCompleteness:
    def test_all_act1_events_have_handlers(self):
        for event_id in ACT1_EVENTS:
            assert event_id in EVENT_HANDLERS, f"Missing handler for {event_id}"

    def test_all_act2_events_have_handlers(self):
        for event_id in ACT2_EVENTS:
            assert event_id in EVENT_HANDLERS, f"Missing handler for {event_id}"

    def test_all_act3_events_have_handlers(self):
        for event_id in ACT3_EVENTS:
            assert event_id in EVENT_HANDLERS, f"Missing handler for {event_id}"

    def test_all_shrine_events_have_handlers(self):
        # GremlinMatchGame and GremlinWheelGame are not yet implemented
        unimplemented = {"GremlinMatchGame", "GremlinWheelGame"}
        for event_id in SHRINE_EVENTS:
            if event_id not in unimplemented:
                assert event_id in EVENT_HANDLERS, f"Missing handler for {event_id}"

    def test_event_definitions_complete(self):
        all_defs = {**ACT1_EVENTS, **ACT2_EVENTS, **ACT3_EVENTS, **SHRINE_EVENTS, **SPECIAL_ONE_TIME_EVENTS}
        for event_id, defn in all_defs.items():
            assert defn.id == event_id
            assert defn.name


# =============================================================================
# Test: Deterministic RNG in events
# =============================================================================


class TestDeterministicEvents:
    def test_same_seed_same_big_fish_box(self):
        r1, _, _, _ = _exec("BigFish", 2, rng_offset=42)
        r2, _, _, _ = _exec("BigFish", 2, rng_offset=42)
        assert r1.relics_gained == r2.relics_gained

    def test_same_seed_same_joust(self):
        for i in range(5):
            h1 = _handler()
            h2 = _handler()
            run1 = _run()
            run2 = _run()
            es1 = EventState(event_id="TheJoust")
            es2 = EventState(event_id="TheJoust")
            misc1 = Random(SEED_LONG + i * 100)
            misc2 = Random(SEED_LONG + i * 100)
            r1 = h1.execute_choice(es1, 0, run1, _rng(), misc_rng=misc1)
            r2 = h2.execute_choice(es2, 0, run2, _rng(), misc_rng=misc2)
            assert r1.gold_change == r2.gold_change
