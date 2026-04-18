"""Rest site relic behavior tests backed by real handlers."""

import pytest

from packages.engine.handlers.rooms import RestHandler
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.state.run import create_watcher_run


@pytest.fixture
def watcher_run():
    return create_watcher_run("TEST_REST", ascension=0)


def _rng(seed: str) -> Random:
    return Random(seed_to_long(seed))


def test_base_rest_options_include_rest_and_smith_when_available(watcher_run):
    watcher_run.damage(10)

    options = RestHandler.get_options(watcher_run)

    assert "rest" in options
    assert "smith" in options


def test_coffee_dripper_blocks_rest_option_and_rest_action(watcher_run):
    watcher_run.add_relic("Coffee Dripper")
    watcher_run.damage(25)

    options = RestHandler.get_options(watcher_run)
    result = RestHandler.rest(watcher_run)

    assert "rest" not in options
    assert result.hp_healed == 0


def test_fusion_hammer_blocks_smith_option(watcher_run):
    watcher_run.add_relic("Fusion Hammer")

    options = RestHandler.get_options(watcher_run)

    assert "smith" not in options


def test_shovel_girya_and_peace_pipe_add_options(watcher_run):
    watcher_run.add_relic("Shovel")
    watcher_run.add_relic("Girya")
    watcher_run.add_relic("Peace Pipe")

    options = RestHandler.get_options(watcher_run)

    assert "dig" in options
    assert "lift" in options
    assert "toke" in options


def test_girya_lift_increments_and_caps_after_three_uses(watcher_run):
    watcher_run.add_relic("Girya")

    for expected_counter in (1, 2, 3):
        result = RestHandler.lift(watcher_run)
        assert result.strength_gained == 1
        assert watcher_run.get_relic("Girya").counter == expected_counter

    # Fourth lift gives no strength and option disappears.
    result = RestHandler.lift(watcher_run)
    options = RestHandler.get_options(watcher_run)

    assert result.strength_gained == 0
    assert "lift" not in options


def test_toke_removes_selected_card(watcher_run):
    watcher_run.add_relic("Peace Pipe")
    first_card_id = watcher_run.deck[0].id
    initial_size = len(watcher_run.deck)

    result = RestHandler.toke(watcher_run, 0)

    assert result.card_removed == first_card_id
    assert len(watcher_run.deck) == initial_size - 1


def test_dig_grants_a_relic(watcher_run):
    watcher_run.add_relic("Shovel")
    initial_relics = len(watcher_run.relics)

    result = RestHandler.dig(watcher_run, _rng("DIG_RELIC"))

    assert result.relic_gained is not None
    assert len(watcher_run.relics) == initial_relics + 1


def test_eternal_feather_heal_on_enter_rest_site_rounds_down(watcher_run):
    watcher_run.add_relic("Eternal Feather")
    watcher_run.damage(60)

    while len(watcher_run.deck) < 23:
        watcher_run.add_card("Strike_P")
    while len(watcher_run.deck) > 23:
        watcher_run.remove_card(0)

    healed = RestHandler.on_enter_rest_site(watcher_run)

    assert healed == 12  # floor(23 / 5) * 3


def test_rest_with_regal_pillow_and_dream_catcher(watcher_run):
    watcher_run.add_relic("Regal Pillow")
    watcher_run.add_relic("Dream Catcher")
    watcher_run.damage(50)

    missing_hp = watcher_run.max_hp - watcher_run.current_hp
    expected_heal = min(missing_hp, int(watcher_run.max_hp * 0.30) + 15)

    result = RestHandler.rest(watcher_run)

    assert result.hp_healed == expected_heal
    assert result.dream_catcher_triggered is True


def test_mark_of_bloom_blocks_rest_heal_but_keeps_dream_catcher_flag(watcher_run):
    watcher_run.add_relic("MarkOfBloom")
    watcher_run.add_relic("Dream Catcher")
    watcher_run.damage(40)
    before_hp = watcher_run.current_hp

    result = RestHandler.rest(watcher_run)

    assert watcher_run.current_hp == before_hp
    assert result.hp_healed == 0
    assert result.dream_catcher_triggered is True


def test_recall_available_in_act_three_and_sets_ruby_key(watcher_run):
    watcher_run.act = 3

    options = RestHandler.get_options(watcher_run)
    result = RestHandler.recall(watcher_run)

    assert "recall" in options
    assert result.action == "recall"
    assert watcher_run.has_ruby_key is True


def test_smith_upgrades_card_when_not_blocked(watcher_run):
    # Starter deck always has upgradeable cards.
    idx = watcher_run.get_upgradeable_cards()[0][0]

    result = RestHandler.smith(watcher_run, idx)

    assert result.card_upgraded is not None
    assert watcher_run.deck[idx].upgraded is True


def test_smith_blocked_by_fusion_hammer_returns_no_upgrade(watcher_run):
    watcher_run.add_relic("Fusion Hammer")
    idx = watcher_run.get_upgradeable_cards()[0][0]

    result = RestHandler.smith(watcher_run, idx)

    assert result.card_upgraded is None
    assert watcher_run.deck[idx].upgraded is False
