"""Treasure/chest acquisition relic behavior tests backed by real handlers."""

import pytest

from packages.engine.handlers.reward_handler import RewardHandler
from packages.engine.handlers.rooms import ChestType, TreasureHandler
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.state.run import create_watcher_run


@pytest.fixture
def watcher_run():
    return create_watcher_run("TEST_ACQ", ascension=0)


def _rng(seed: str) -> Random:
    return Random(seed_to_long(seed))


def _open_chest(run_state, tag: str, chest_type: ChestType = ChestType.MEDIUM, take_sapphire_key: bool = False):
    return TreasureHandler.open_chest(
        run_state,
        treasure_rng=_rng(f"TREASURE_{tag}"),
        relic_rng=_rng(f"RELIC_{tag}"),
        chest_type=chest_type,
        take_sapphire_key=take_sapphire_key,
    )


def test_tiny_chest_triggers_every_fourth_question_room(watcher_run):
    watcher_run.add_relic("Tiny Chest")

    for _ in range(3):
        assert TreasureHandler.on_enter_question_room(watcher_run) is False

    assert watcher_run.get_relic("Tiny Chest").counter == 3
    assert TreasureHandler.on_enter_question_room(watcher_run) is True
    assert watcher_run.get_relic("Tiny Chest").counter == 0


def test_matryoshka_grants_extra_relic_for_first_two_chests_only(watcher_run):
    watcher_run.add_relic("Matryoshka")
    matryoshka = watcher_run.get_relic("Matryoshka")
    assert matryoshka.counter == 2

    start_relics = len(watcher_run.relics)

    first = _open_chest(watcher_run, "M1", chest_type=ChestType.SMALL)
    assert matryoshka.counter == 1
    assert first.matryoshka_relics is not None

    second = _open_chest(watcher_run, "M2", chest_type=ChestType.MEDIUM)
    assert matryoshka.counter == 0
    assert second.matryoshka_relics is not None

    third = _open_chest(watcher_run, "M3", chest_type=ChestType.LARGE)
    assert matryoshka.counter == 0
    assert third.matryoshka_relics is None
    assert len(watcher_run.relics) >= start_relics + 4


def test_cursed_key_adds_curse_when_relic_taken_from_chest(watcher_run):
    watcher_run.add_relic("Cursed Key")
    deck_before = len(watcher_run.deck)

    result = _open_chest(watcher_run, "CK1")

    assert result.curse_added is not None
    assert len(watcher_run.deck) == deck_before + 1


def test_sapphire_key_choice_skips_chest_relic_and_cursed_key_curse(watcher_run):
    watcher_run.add_relic("Cursed Key")
    watcher_run.act = 3
    relics_before = len(watcher_run.relics)
    deck_before = len(watcher_run.deck)

    result = _open_chest(
        watcher_run,
        "SAPPHIRE",
        chest_type=ChestType.MEDIUM,
        take_sapphire_key=True,
    )

    assert result.sapphire_key_taken is True
    assert result.curse_added is None
    assert watcher_run.has_sapphire_key is True
    assert len(watcher_run.relics) == relics_before
    assert len(watcher_run.deck) == deck_before


def test_nloths_mask_removes_relic_from_next_chest_then_uses_up(watcher_run):
    watcher_run.add_relic("NlothsMask")
    mask = watcher_run.get_relic("NlothsMask")
    assert mask.counter == 1

    relics_before = len(watcher_run.relics)

    first = _open_chest(watcher_run, "NLOTH1", chest_type=ChestType.MEDIUM)
    assert first.relic_id == "None"
    assert len(watcher_run.relics) == relics_before
    assert mask.counter == -2

    second = _open_chest(watcher_run, "NLOTH2", chest_type=ChestType.MEDIUM)
    assert second.relic_id != "None"
    assert len(watcher_run.relics) == relics_before + 1


def test_nloths_mask_with_matryoshka_leaves_one_relic(watcher_run):
    watcher_run.add_relic("NlothsMask")
    watcher_run.add_relic("Matryoshka")

    relics_before = len(watcher_run.relics)
    result = _open_chest(watcher_run, "NLOTH_MAT", chest_type=ChestType.MEDIUM)

    # Base relic removed by N'loth; Matryoshka relic remains.
    assert result.relic_id != "None"
    assert len(watcher_run.relics) == relics_before + 1


def test_black_star_generates_second_elite_relic_reward(watcher_run):
    watcher_run.add_relic("Black Star")

    rewards = RewardHandler.generate_combat_rewards(
        watcher_run,
        room_type="elite",
        card_rng=_rng("ELITE_CARD"),
        treasure_rng=_rng("ELITE_TREASURE"),
        potion_rng=_rng("ELITE_POTION"),
        relic_rng=_rng("ELITE_RELIC"),
        enemies_killed=1,
    )

    assert rewards.relic is not None
    assert rewards.second_relic is not None


def test_without_black_star_no_second_elite_relic_reward(watcher_run):
    rewards = RewardHandler.generate_combat_rewards(
        watcher_run,
        room_type="elite",
        card_rng=_rng("ELITE_CARD_NO_BS"),
        treasure_rng=_rng("ELITE_TREASURE_NO_BS"),
        potion_rng=_rng("ELITE_POTION_NO_BS"),
        relic_rng=_rng("ELITE_RELIC_NO_BS"),
        enemies_killed=1,
    )

    assert rewards.relic is not None
    assert rewards.second_relic is None


def test_boss_rewards_offer_three_relic_choices(watcher_run):
    rewards = RewardHandler.generate_boss_rewards(
        watcher_run,
        card_rng=_rng("BOSS_CARD"),
        treasure_rng=_rng("BOSS_TREASURE"),
        potion_rng=_rng("BOSS_POTION"),
        relic_rng=_rng("BOSS_RELIC"),
    )

    assert rewards.boss_relics is not None
    assert len(rewards.boss_relics.relics) == 3
