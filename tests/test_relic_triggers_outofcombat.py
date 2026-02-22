"""Cross-system out-of-combat relic integration tests (engine-backed)."""

import pytest

from packages.engine.game import GameRunner
from packages.engine.generation.map import MapRoomNode, RoomType
from packages.engine.handlers.reward_handler import (
    RewardHandler,
    SingingBowlAction,
)
from packages.engine.handlers.shop_handler import ShopHandler
from packages.engine.content.cards import ALL_CARDS, CardType
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.state.run import create_watcher_run


def _rng(seed: str) -> Random:
    return Random(seed_to_long(seed))


def _enter_room(runner: GameRunner, room_type: RoomType) -> None:
    node = MapRoomNode(x=0, y=0, room_type=room_type)
    runner._enter_room(node)


def _first_card_id(card_type: CardType) -> str:
    for card_id, card in ALL_CARDS.items():
        if card.card_type == card_type:
            return card_id
    raise AssertionError(f"No card found for type {card_type}")


def test_maw_bank_gains_12_gold_on_room_entry_until_spent():
    runner = GameRunner(seed="MAWBANK_ROOM", ascension=0, verbose=False)
    runner.run_state.add_relic("MawBank")

    start_gold = runner.run_state.gold
    _enter_room(runner, RoomType.REST)
    assert runner.run_state.gold == start_gold + 12

    runner.run_state.spend_gold(1)
    after_spend = runner.run_state.gold
    _enter_room(runner, RoomType.REST)

    assert runner.run_state.get_relic("MawBank").counter == -2
    assert runner.run_state.gold == after_spend


def test_maw_bank_triggers_on_shop_entry_before_disable():
    runner = GameRunner(seed="MAWBANK_SHOP", ascension=0, verbose=False)
    runner.run_state.add_relic("MawBank")

    start_gold = runner.run_state.gold
    _enter_room(runner, RoomType.SHOP)

    assert runner.run_state.gold == start_gold + 12


def test_meal_ticket_heals_on_shop_entry():
    runner = GameRunner(seed="MEAL_TICKET", ascension=0, verbose=False)
    runner.run_state.add_relic("MealTicket")
    runner.run_state.damage(30)

    before_hp = runner.run_state.current_hp
    _enter_room(runner, RoomType.SHOP)

    assert runner.run_state.current_hp == min(runner.run_state.max_hp, before_hp + 15)


def test_membership_card_discount_reduces_shop_card_prices_vs_baseline():
    base_run = create_watcher_run("SHOP_DISCOUNT", ascension=0)
    discounted_run = create_watcher_run("SHOP_DISCOUNT", ascension=0)
    discounted_run.add_relic("Membership Card")

    base_shop = ShopHandler.create_shop(base_run, _rng("MERCHANT_BASE"), _rng("CARD_BASE"), _rng("POTION_BASE"))
    discounted_shop = ShopHandler.create_shop(
        discounted_run,
        _rng("MERCHANT_BASE"),
        _rng("CARD_BASE"),
        _rng("POTION_BASE"),
    )

    for base_card, discounted_card in zip(base_shop.colored_cards, discounted_shop.colored_cards):
        assert discounted_card.price < base_card.price


def test_membership_and_courier_stack_discounts():
    membership_run = create_watcher_run("SHOP_STACK", ascension=0)
    membership_run.add_relic("Membership Card")

    stacked_run = create_watcher_run("SHOP_STACK", ascension=0)
    stacked_run.add_relic("Membership Card")
    stacked_run.add_relic("The Courier")

    membership_shop = ShopHandler.create_shop(
        membership_run,
        _rng("MERCHANT_STACK"),
        _rng("CARD_STACK"),
        _rng("POTION_STACK"),
    )
    stacked_shop = ShopHandler.create_shop(
        stacked_run,
        _rng("MERCHANT_STACK"),
        _rng("CARD_STACK"),
        _rng("POTION_STACK"),
    )

    for membership_card, stacked_card in zip(membership_shop.colored_cards, stacked_shop.colored_cards):
        assert stacked_card.price < membership_card.price


def test_smiling_mask_forces_purge_cost_to_50():
    run_state = create_watcher_run("SMILING_MASK", ascension=0)
    run_state.purge_count = 10
    run_state.add_relic("SmilingMask")

    shop = ShopHandler.create_shop(run_state, _rng("MERCHANT_SMILE"), _rng("CARD_SMILE"), _rng("POTION_SMILE"))

    assert shop.purge_cost == 50


def test_ectoplasm_blocks_gold_and_tracks_blocked_amount():
    run_state = create_watcher_run("ECTO", ascension=0)
    run_state.add_relic("Ectoplasm")
    gold_before = run_state.gold

    run_state.add_gold(100)

    assert run_state.gold == gold_before
    assert run_state.gold_blocked == 100


def test_golden_idol_and_bloody_idol_stack():
    run_state = create_watcher_run("IDOLS", ascension=0)
    run_state.add_relic("Golden Idol")
    run_state.add_relic("Bloody Idol")
    run_state.damage(20)

    hp_before = run_state.current_hp
    gold_before = run_state.gold

    run_state.add_gold(100)

    assert run_state.gold == gold_before + 125
    assert run_state.current_hp == min(run_state.max_hp, hp_before + 5)


def test_potion_belt_adds_two_slots_and_cauldron_respects_capacity():
    run_state = create_watcher_run("POTION_BELT", ascension=0)
    base_slots = len(run_state.potion_slots)

    run_state.add_relic("Potion Belt")
    run_state.add_relic("Cauldron", potion_rng=_rng("CAULDRON_POTION"))

    assert len(run_state.potion_slots) == base_slots + 2
    assert run_state.count_potions() <= len(run_state.potion_slots)


def test_question_card_and_busted_crown_net_card_reward_count():
    run_state = create_watcher_run("CARD_COUNT", ascension=0)
    run_state.add_relic("Question Card")
    run_state.add_relic("BustedCrown")

    assert run_state.get_card_reward_count() == 2


def test_singing_bowl_action_gives_max_hp_in_reward_handler():
    run_state = create_watcher_run("SINGING_BOWL", ascension=0)
    run_state.add_relic("Singing Bowl")

    rewards = RewardHandler.generate_combat_rewards(
        run_state,
        room_type="monster",
        card_rng=_rng("SB_CARD"),
        treasure_rng=_rng("SB_TREASURE"),
        potion_rng=_rng("SB_POTION"),
        relic_rng=_rng("SB_RELIC"),
    )

    assert rewards.card_rewards, "Expected at least one card reward"

    max_hp_before = run_state.max_hp
    result = RewardHandler.handle_action(SingingBowlAction(card_reward_index=0), run_state, rewards)

    assert result["success"] is True
    assert run_state.max_hp == max_hp_before + 2


def test_sozu_suppresses_potion_rewards():
    run_state = create_watcher_run("SOZU", ascension=0)
    run_state.add_relic("Sozu")

    rewards = RewardHandler.generate_combat_rewards(
        run_state,
        room_type="monster",
        card_rng=_rng("SOZU_CARD"),
        treasure_rng=_rng("SOZU_TREASURE"),
        potion_rng=_rng("SOZU_POTION"),
        relic_rng=_rng("SOZU_RELIC"),
    )

    assert rewards.potion is None


def test_ceramic_fish_grants_9_gold_on_card_obtain():
    run_state = create_watcher_run("CERAMIC_FISH", ascension=0)
    run_state.add_relic("CeramicFish")
    gold_before = run_state.gold

    run_state.add_card("Strike_P")

    assert run_state.gold == gold_before + 9


def test_frozen_egg_upgrades_obtained_power_cards():
    run_state = create_watcher_run("FROZEN_EGG", ascension=0)
    run_state.add_relic("Frozen Egg 2")
    power_card_id = _first_card_id(CardType.POWER)

    card = run_state.add_card(power_card_id)

    assert card.upgraded is True


def test_molten_egg_upgrades_obtained_attack_cards():
    run_state = create_watcher_run("MOLTEN_EGG", ascension=0)
    run_state.add_relic("Molten Egg 2")
    attack_card_id = _first_card_id(CardType.ATTACK)

    card = run_state.add_card(attack_card_id)

    assert card.upgraded is True


def test_toxic_egg_upgrades_obtained_skill_cards():
    run_state = create_watcher_run("TOXIC_EGG", ascension=0)
    run_state.add_relic("Toxic Egg 2")
    skill_card_id = _first_card_id(CardType.SKILL)

    card = run_state.add_card(skill_card_id)

    assert card.upgraded is True


def test_darkstone_periapt_grants_max_hp_on_curse_obtain():
    run_state = create_watcher_run("DARKSTONE", ascension=0)
    run_state.add_relic("Darkstone Periapt")
    max_hp_before = run_state.max_hp
    curse_id = _first_card_id(CardType.CURSE)

    run_state.add_card(curse_id)

    assert run_state.max_hp == max_hp_before + 6
