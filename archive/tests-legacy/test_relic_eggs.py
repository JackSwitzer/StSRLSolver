"""Egg relic integration tests for run-state card acquisition flows."""

import pytest

from packages.engine.content.cards import ALL_CARDS, CardType
from packages.engine.handlers.reward_handler import PickCardAction, RewardHandler
from packages.engine.handlers.shop_handler import ShopAction, ShopActionType, ShopHandler
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.state.run import create_watcher_run


def _rng(seed: str) -> Random:
    return Random(seed_to_long(seed))


def _first_card_id(card_type: CardType) -> str:
    for card_id, card in ALL_CARDS.items():
        if card.card_type == card_type:
            return card_id
    raise AssertionError(f"No card found for type {card_type}")


def _buy_first_shop_card_of_type(run_state, card_type: CardType) -> None:
    shop = ShopHandler.create_shop(
        run_state,
        merchant_rng=_rng("EGG_MERCHANT"),
        card_rng=_rng("EGG_CARD"),
        potion_rng=_rng("EGG_POTION"),
    )

    target = next(
        (c for c in shop.colored_cards if getattr(c.card.card_type, "value", c.card.card_type) == card_type.value),
        None,
    )
    assert target is not None, f"Expected a shop card of type {card_type}"

    result = ShopHandler.execute_action(
        ShopAction(action_type=ShopActionType.BUY_COLORED_CARD, item_index=target.slot_index),
        shop,
        run_state,
    )
    assert result.success is True


def _claim_first_reward_card_of_type(run_state, card_type: CardType) -> None:
    for idx in range(64):
        rewards = RewardHandler.generate_combat_rewards(
            run_state,
            room_type="monster",
            card_rng=_rng(f"EGG_REWARD_CARD_{idx}"),
            treasure_rng=_rng(f"EGG_REWARD_TREASURE_{idx}"),
            potion_rng=_rng(f"EGG_REWARD_POTION_{idx}"),
            relic_rng=_rng(f"EGG_REWARD_RELIC_{idx}"),
        )
        if not rewards.card_rewards:
            continue

        cards = rewards.card_rewards[0].cards
        chosen_index = next(
            (i for i, card in enumerate(cards) if ALL_CARDS.get(card.id) and ALL_CARDS[card.id].card_type == card_type),
            None,
        )
        if chosen_index is None:
            continue

        result = RewardHandler.handle_action(
            PickCardAction(card_reward_index=0, card_index=chosen_index),
            run_state,
            rewards,
        )
        assert result["success"] is True
        return

    raise AssertionError(f"Could not find reward card of type {card_type} in sampled seeds")


@pytest.mark.parametrize(
    "relic_id,card_type",
    [
        ("Toxic Egg 2", CardType.SKILL),
        ("Molten Egg 2", CardType.ATTACK),
        ("Frozen Egg 2", CardType.POWER),
    ],
)
def test_egg_upgrades_cards_from_shop_purchase(relic_id, card_type):
    run_state = create_watcher_run("EGG_SHOP", ascension=0)
    run_state.gold = 999
    run_state.add_relic(relic_id)

    _buy_first_shop_card_of_type(run_state, card_type)

    assert run_state.deck[-1].upgraded is True


@pytest.mark.parametrize(
    "relic_id,card_type",
    [
        ("Toxic Egg 2", CardType.SKILL),
        ("Molten Egg 2", CardType.ATTACK),
        ("Frozen Egg 2", CardType.POWER),
    ],
)
def test_egg_upgrades_cards_from_combat_reward_pick(relic_id, card_type):
    run_state = create_watcher_run("EGG_REWARD", ascension=0)
    run_state.add_relic(relic_id)

    _claim_first_reward_card_of_type(run_state, card_type)

    assert run_state.deck[-1].upgraded is True


def test_eggs_only_affect_cards_obtained_after_relic_pickup():
    run_state = create_watcher_run("EGG_TIMING", ascension=0)
    before = run_state.add_card("Vigilance", upgraded=False)

    run_state.add_relic("Toxic Egg 2")
    after = run_state.add_card("Vigilance", upgraded=False)

    assert before.upgraded is False
    assert after.upgraded is True


def test_multiple_eggs_upgrade_their_respective_card_types():
    run_state = create_watcher_run("EGG_MULTI", ascension=0)
    run_state.add_relic("Toxic Egg 2")
    run_state.add_relic("Molten Egg 2")
    run_state.add_relic("Frozen Egg 2")

    skill = run_state.add_card(_first_card_id(CardType.SKILL))
    attack = run_state.add_card(_first_card_id(CardType.ATTACK))
    power = run_state.add_card(_first_card_id(CardType.POWER))

    assert skill.upgraded is True
    assert attack.upgraded is True
    assert power.upgraded is True


def test_toxic_egg_upgrades_off_class_skills():
    """Toxic Egg upgrades all skills obtained after pickup, including off-class cards."""
    run_state = create_watcher_run("EGG_OFFCLASS", ascension=0)
    run_state.add_relic("Toxic Egg 2")

    off_class_skill = next(
        card_id
        for card_id, card in ALL_CARDS.items()
        if card.card_type == CardType.SKILL and getattr(card.color, "value", "") == "RED"
    )
    obtained = run_state.add_card(off_class_skill, upgraded=False)

    assert obtained.upgraded is True
