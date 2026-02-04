"""
Tests for handler modules:
- EventHandler (event_handler.py)
- ShopHandler (shop_handler.py)
- RestHandler (rooms.py)
- TreasureHandler (rooms.py)
- NeowHandler (rooms.py)
- RewardHandler (reward_handler.py)
"""

import pytest
import sys

sys.path.insert(0, "/Users/jackswitzer/Desktop/SlayTheSpireRL")

from packages.engine.state.run import create_watcher_run
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.handlers.rooms import (
    RestHandler,
    TreasureHandler,
    NeowHandler,
    NeowBlessingType,
    NeowDrawbackType,
    ChestType,
)
from packages.engine.handlers.event_handler import EventHandler
from packages.engine.handlers.shop_handler import (
    ShopHandler, ShopAction, ShopActionType,
)
from packages.engine.handlers.reward_handler import (
    RewardHandler,
    ClaimGoldAction, ClaimPotionAction, SkipPotionAction,
    PickCardAction, SkipCardAction, SingingBowlAction,
    ClaimRelicAction, ClaimEmeraldKeyAction, SkipEmeraldKeyAction,
    PickBossRelicAction, ProceedFromRewardsAction,
)
from packages.engine.content.events import Act, get_events_for_act


# =============================================================================
# Fixtures
# =============================================================================

SEED = "TESTHANDLERS"
SEED_LONG = seed_to_long(SEED)


def _make_run(ascension=0, seed=SEED):
    return create_watcher_run(seed, ascension=ascension)


def _make_rng(offset=0):
    return Random(SEED_LONG + offset)


def _make_shop(run, offset=1000):
    merchant_rng = _make_rng(offset)
    card_rng = _make_rng(offset + 4000)
    potion_rng = _make_rng(offset + 6000)
    return ShopHandler.create_shop(run, merchant_rng, card_rng, potion_rng)


# =============================================================================
# EventHandler Tests
# =============================================================================


class TestEventHandler:
    """Tests for EventHandler in event_handler.py."""

    def test_get_event_returns_event(self):
        run = _make_run()
        rng = _make_rng()
        handler = EventHandler()
        event_state = handler.select_event(run, rng)
        assert event_state is not None
        assert hasattr(event_state, "event_id")
        assert event_state.event_id is not None

    def test_get_event_deterministic(self):
        """Same seed + state = same event."""
        run1 = _make_run()
        run2 = _make_run()
        handler1 = EventHandler()
        handler2 = EventHandler()
        event1 = handler1.select_event(run1, Random(SEED_LONG))
        event2 = handler2.select_event(run2, Random(SEED_LONG))
        assert event1.event_id == event2.event_id

    def test_get_event_different_seeds(self):
        """Different seeds can produce different events."""
        events = set()
        for i in range(10):
            run = _make_run(seed=f"EVTSEED{i}")
            rng = Random(seed_to_long(f"EVTSEED{i}"))
            handler = EventHandler()
            ev = handler.select_event(run, rng)
            if ev:
                events.add(ev.event_id)
        # With 10 different seeds we should get at least 2 different events
        assert len(events) >= 2

    def test_get_choices_returns_list(self):
        run = _make_run()
        rng = _make_rng()
        handler = EventHandler()
        event = handler.select_event(run, rng)
        choices = handler.get_available_choices(event, run)
        assert isinstance(choices, list)
        assert len(choices) > 0

    def test_apply_choice_returns_result(self):
        run = _make_run()
        handler = EventHandler()
        event = handler.select_event(run, _make_rng())
        result = handler.execute_choice(event, 0, run, _make_rng(100), misc_rng=_make_rng(200))
        assert result.event_id == event.event_id
        assert result.choice_idx == 0

    def test_apply_choice_out_of_range(self):
        run = _make_run()
        handler = EventHandler()
        event = handler.select_event(run, _make_rng())
        result = handler.execute_choice(event, 999, run, _make_rng(100), misc_rng=_make_rng(200))
        # Should not crash, just return empty result
        assert result.event_id == event.event_id

    def test_event_choices_filtered_by_gold(self):
        """Choices requiring gold are filtered when broke."""
        run = _make_run()
        run.lose_gold(run.gold)  # Zero gold
        handler = EventHandler()
        event = handler.select_event(run, _make_rng())
        choices = handler.get_available_choices(event, run)
        for c in choices:
            if c.requires_gold is not None:
                assert c.requires_gold <= 0

    def test_event_per_act(self):
        """Events are available for each act."""
        for act_num in [1, 2, 3]:
            run = _make_run()
            run.act = act_num
            rng = Random(SEED_LONG + act_num * 1000)
            handler = EventHandler()
            event = handler.select_event(run, rng)
            assert event is not None, f"No event for act {act_num}"


# =============================================================================
# ShopHandler Tests
# =============================================================================


class TestShopHandler:
    """Tests for ShopHandler in shop_handler.py."""

    def test_generate_shop_returns_inventory(self):
        run = _make_run()
        shop = _make_shop(run)
        assert hasattr(shop, "colored_cards")
        assert hasattr(shop, "relics")
        assert hasattr(shop, "potions")
        assert hasattr(shop, "purge_cost")
        assert hasattr(shop, "purge_available")

    def test_shop_has_cards(self):
        run = _make_run()
        shop = _make_shop(run)
        assert len(shop.colored_cards) >= 3
        assert len(shop.colorless_cards) >= 1

    def test_shop_has_relics(self):
        run = _make_run()
        shop = _make_shop(run)
        assert len(shop.relics) >= 1

    def test_shop_has_potions(self):
        run = _make_run()
        shop = _make_shop(run)
        assert len(shop.potions) >= 1

    def test_shop_purge_cost_positive(self):
        run = _make_run()
        shop = _make_shop(run)
        assert shop.purge_cost > 0

    def test_shop_deterministic(self):
        """Same seed = same shop."""
        run1 = _make_run()
        run2 = _make_run()
        shop1 = _make_shop(run1, offset=1000)
        shop2 = _make_shop(run2, offset=1000)
        assert len(shop1.colored_cards) == len(shop2.colored_cards)
        for c1, c2 in zip(shop1.colored_cards, shop2.colored_cards):
            assert c1.card.id == c2.card.id
            assert c1.price == c2.price

    def test_purchasable_items_respect_gold(self):
        run = _make_run()
        run.lose_gold(run.gold)  # Zero gold
        shop = _make_shop(run)
        actions = ShopHandler.get_available_actions(shop, run)
        assert all(a.action_type == ShopActionType.LEAVE for a in actions)

    def test_buy_card_success(self):
        run = _make_run()
        run.add_gold(500)
        shop = _make_shop(run)
        actions = ShopHandler.get_available_actions(shop, run)
        buy_action = next(
            (a for a in actions if a.action_type == ShopActionType.BUY_COLORED_CARD),
            None,
        )
        if buy_action:
            initial_deck = len(run.deck)
            initial_gold = run.gold
            result = ShopHandler.execute_action(buy_action, shop, run)
            assert result.success
            assert result.gold_spent > 0
            assert len(run.deck) == initial_deck + 1
            assert run.gold < initial_gold

    def test_buy_card_not_enough_gold(self):
        run = _make_run()
        run.lose_gold(run.gold)
        shop = _make_shop(run)
        if shop.colored_cards:
            action = ShopAction(
                action_type=ShopActionType.BUY_COLORED_CARD,
                item_index=shop.colored_cards[0].slot_index,
            )
            result = ShopHandler.execute_action(action, shop, run)
            assert not result.success

    def test_purge_card(self):
        run = _make_run()
        run.add_gold(500)
        shop = _make_shop(run)
        actions = ShopHandler.get_available_actions(shop, run)
        remove_action = next(
            (a for a in actions if a.action_type == ShopActionType.REMOVE_CARD),
            None,
        )
        if remove_action:
            initial_deck = len(run.deck)
            result = ShopHandler.execute_action(remove_action, shop, run)
            assert result.success
            assert len(run.deck) == initial_deck - 1
            assert not shop.purge_available

    def test_purge_twice_fails(self):
        run = _make_run()
        run.add_gold(500)
        shop = _make_shop(run)
        actions = ShopHandler.get_available_actions(shop, run)
        remove_action = next(
            (a for a in actions if a.action_type == ShopActionType.REMOVE_CARD),
            None,
        )
        if remove_action:
            ShopHandler.execute_action(remove_action, shop, run)
            result = ShopHandler.execute_action(remove_action, shop, run)
            assert not result.success

    def test_buy_relic(self):
        run = _make_run()
        run.add_gold(500)
        shop = _make_shop(run)
        actions = ShopHandler.get_available_actions(shop, run)
        buy_action = next(
            (a for a in actions if a.action_type == ShopActionType.BUY_RELIC),
            None,
        )
        if buy_action:
            initial_relics = len(run.relics)
            result = ShopHandler.execute_action(buy_action, shop, run)
            assert result.success
            assert len(run.relics) == initial_relics + 1

    def test_buy_potion(self):
        run = _make_run()
        run.add_gold(500)
        shop = _make_shop(run)
        actions = ShopHandler.get_available_actions(shop, run)
        buy_action = next(
            (a for a in actions if a.action_type == ShopActionType.BUY_POTION),
            None,
        )
        if buy_action:
            result = ShopHandler.execute_action(buy_action, shop, run)
            assert result.success

    def test_buy_potion_no_slots(self):
        run = _make_run()
        run.add_gold(500)
        # Fill all potion slots
        for slot in run.potion_slots:
            slot.potion_id = "FakePotion"
        shop = _make_shop(run)
        if shop.potions:
            action = ShopAction(
                action_type=ShopActionType.BUY_POTION,
                item_index=shop.potions[0].slot_index,
            )
            result = ShopHandler.execute_action(action, shop, run)
            assert not result.success

    def test_ascension_purge_cost(self):
        run = _make_run(ascension=15)
        shop = _make_shop(run)
        # A15+ has purge cost cap at 175
        assert shop.purge_cost <= 175


# =============================================================================
# RestHandler Tests
# =============================================================================


class TestRestHandler:
    """Tests for RestHandler in rooms.py."""

    def test_get_options_includes_rest(self):
        run = _make_run()
        run.damage(20)
        options = RestHandler.get_options(run)
        assert "rest" in options

    def test_get_options_no_rest_at_full_hp(self):
        run = _make_run()
        options = RestHandler.get_options(run)
        assert "rest" not in options

    def test_get_options_includes_smith(self):
        run = _make_run()
        options = RestHandler.get_options(run)
        assert "smith" in options

    def test_rest_heals(self):
        run = _make_run()
        run.damage(30)
        old_hp = run.current_hp
        result = RestHandler.rest(run)
        assert result.hp_healed > 0
        assert run.current_hp > old_hp

    def test_rest_heals_30_percent(self):
        run = _make_run()
        run.damage(40)
        old_hp = run.current_hp
        result = RestHandler.rest(run)
        expected = int(run.max_hp * 0.30)
        assert result.hp_healed == min(expected, run.max_hp - old_hp)

    def test_smith_upgrades_card(self):
        run = _make_run()
        # Find an upgradeable card
        upgradeable = run.get_upgradeable_cards()
        assert len(upgradeable) > 0
        idx = upgradeable[0][0]
        result = RestHandler.smith(run, idx)
        assert result.card_upgraded is not None

    def test_smith_alias(self):
        run = _make_run()
        idx = run.get_upgradeable_cards()[0][0]
        result = RestHandler.upgrade(run, idx)
        assert result.card_upgraded is not None

    def test_coffee_dripper_blocks_rest(self):
        run = _make_run()
        run.add_relic("Coffee Dripper")
        run.damage(20)
        options = RestHandler.get_options(run)
        assert "rest" not in options

    def test_fusion_hammer_blocks_smith(self):
        run = _make_run()
        run.add_relic("Fusion Hammer")
        options = RestHandler.get_options(run)
        assert "smith" not in options

    def test_shovel_enables_dig(self):
        run = _make_run()
        run.add_relic("Shovel")
        options = RestHandler.get_options(run)
        assert "dig" in options

    def test_dig_gives_relic(self):
        run = _make_run()
        run.add_relic("Shovel")
        initial_relics = len(run.relics)
        result = RestHandler.dig(run, _make_rng(500))
        assert result.relic_gained is not None
        assert len(run.relics) > initial_relics

    def test_peace_pipe_enables_toke(self):
        run = _make_run()
        run.add_relic("Peace Pipe")
        options = RestHandler.get_options(run)
        assert "toke" in options

    def test_toke_removes_card(self):
        run = _make_run()
        run.add_relic("Peace Pipe")
        initial_deck = len(run.deck)
        result = RestHandler.toke(run, 0)
        assert result.card_removed is not None
        assert len(run.deck) == initial_deck - 1

    def test_recall_in_act3(self):
        run = _make_run()
        run.act = 3
        options = RestHandler.get_options(run)
        assert "recall" in options

    def test_recall_gives_ruby_key(self):
        run = _make_run()
        run.act = 3
        result = RestHandler.recall(run)
        assert run.has_ruby_key

    def test_recall_not_in_act1(self):
        run = _make_run()
        run.act = 1
        options = RestHandler.get_options(run)
        assert "recall" not in options

    def test_girya_enables_lift(self):
        run = _make_run()
        run.add_relic("Girya")
        options = RestHandler.get_options(run)
        assert "lift" in options

    def test_lift_increments_counter(self):
        run = _make_run()
        relic = run.add_relic("Girya")
        result = RestHandler.lift(run)
        assert result.strength_gained == 1
        girya = run.get_relic("Girya")
        assert girya.counter == 1

    def test_lift_max_3_times(self):
        run = _make_run()
        relic = run.add_relic("Girya")
        for _ in range(3):
            RestHandler.lift(run)
        result = RestHandler.lift(run)
        assert result.strength_gained == 0

    def test_regal_pillow_bonus(self):
        run = _make_run()
        run.damage(50)
        run.add_relic("Regal Pillow")
        old_hp = run.current_hp
        result = RestHandler.rest(run)
        expected_base = int(run.max_hp * 0.30)
        # With Regal Pillow: +15 flat
        assert result.hp_healed >= expected_base

    def test_eternal_feather_on_enter(self):
        run = _make_run()
        run.damage(30)
        run.add_relic("Eternal Feather")
        old_hp = run.current_hp
        healed = RestHandler.on_enter_rest_site(run)
        # 10 cards in starter deck -> 10//5 * 3 = 6 HP
        assert healed > 0


# =============================================================================
# TreasureHandler Tests
# =============================================================================


class TestTreasureHandler:
    """Tests for TreasureHandler in rooms.py."""

    def test_determine_chest_type(self):
        rng = _make_rng(2000)
        ct = TreasureHandler.determine_chest_type(rng)
        assert ct in (ChestType.SMALL, ChestType.MEDIUM, ChestType.LARGE)

    def test_roll_relic_tier(self):
        rng = _make_rng(2000)
        for ct in [ChestType.SMALL, ChestType.MEDIUM, ChestType.LARGE]:
            tier = TreasureHandler.roll_relic_tier(rng, ct)
            assert tier in ("COMMON", "UNCOMMON", "RARE")

    def test_large_chest_no_common(self):
        """Large chests should never roll common tier."""
        tiers = set()
        for i in range(50):
            rng = Random(SEED_LONG + i * 100)
            tier = TreasureHandler.roll_relic_tier(rng, ChestType.LARGE)
            tiers.add(tier)
        assert "COMMON" not in tiers

    def test_open_chest_gives_relic(self):
        run = _make_run()
        initial_relics = len(run.relics)
        result = TreasureHandler.open_chest(run, _make_rng(3000), _make_rng(4000))
        assert result.relic_id is not None
        assert len(run.relics) > initial_relics

    def test_open_chest_deterministic(self):
        run1 = _make_run()
        run2 = _make_run()
        r1 = TreasureHandler.open_chest(run1, Random(SEED_LONG + 3000), Random(SEED_LONG + 4000))
        r2 = TreasureHandler.open_chest(run2, Random(SEED_LONG + 3000), Random(SEED_LONG + 4000))
        assert r1.relic_id == r2.relic_id
        assert r1.chest_type == r2.chest_type

    def test_sapphire_key_act3(self):
        run = _make_run()
        run.act = 3
        result = TreasureHandler.open_chest(
            run, _make_rng(3000), _make_rng(4000), take_sapphire_key=True
        )
        assert result.sapphire_key_taken
        assert run.has_sapphire_key

    def test_sapphire_key_not_act1(self):
        run = _make_run()
        run.act = 1
        result = TreasureHandler.open_chest(
            run, _make_rng(3000), _make_rng(4000), take_sapphire_key=True
        )
        assert not result.sapphire_key_taken

    def test_cursed_key_adds_curse(self):
        run = _make_run()
        run.add_relic("Cursed Key")
        initial_deck = len(run.deck)
        TreasureHandler.open_chest(run, _make_rng(3000), _make_rng(4000))
        # Should have gained a curse
        assert len(run.deck) > initial_deck

    def test_treasure_actions(self):
        run = _make_run()
        actions = TreasureHandler.get_treasure_actions(run)
        assert "open" in actions

    def test_treasure_actions_act3_sapphire(self):
        run = _make_run()
        run.act = 3
        actions = TreasureHandler.get_treasure_actions(run)
        assert "take_sapphire_key" in actions


# =============================================================================
# NeowHandler Tests
# =============================================================================


class TestNeowHandler:
    """Tests for NeowHandler in rooms.py."""

    def test_first_run_options(self):
        options = NeowHandler.get_first_run_options()
        assert len(options) == 4

    def test_get_blessing_options_returns_4(self):
        rng = _make_rng(5000)
        options = NeowHandler.get_blessing_options(rng, previous_score=100)
        assert len(options) == 4

    def test_get_blessing_options_deterministic(self):
        opts1 = NeowHandler.get_blessing_options(Random(SEED_LONG + 5000), previous_score=100)
        opts2 = NeowHandler.get_blessing_options(Random(SEED_LONG + 5000), previous_score=100)
        for o1, o2 in zip(opts1, opts2):
            assert o1.blessing_type == o2.blessing_type

    def test_boss_swap_always_last(self):
        rng = _make_rng(5000)
        options = NeowHandler.get_blessing_options(rng, previous_score=100)
        assert options[-1].blessing_type == NeowBlessingType.BOSS_SWAP

    def test_apply_hundred_gold(self):
        run = _make_run()
        blessing = NeowHandler.get_first_run_options()[1]  # HUNDRED_GOLD
        assert blessing.blessing_type == NeowBlessingType.HUNDRED_GOLD
        old_gold = run.gold
        result = NeowHandler.apply_blessing(
            run, blessing, _make_rng(6000), _make_rng(6001), _make_rng(6002), _make_rng(6003)
        )
        assert run.gold == old_gold + 100
        assert result.gold_change == 100

    def test_apply_ten_percent_hp(self):
        run = _make_run()
        blessing = NeowHandler.get_first_run_options()[2]  # TEN_PERCENT_HP_BONUS
        old_max = run.max_hp
        NeowHandler.apply_blessing(
            run, blessing, _make_rng(6000), _make_rng(6001), _make_rng(6002), _make_rng(6003)
        )
        assert run.max_hp > old_max

    def test_apply_common_relic(self):
        run = _make_run()
        rng = _make_rng(5000)
        options = NeowHandler.get_blessing_options(rng, previous_score=100)
        # Find RANDOM_COMMON_RELIC if present
        for opt in options:
            if opt.blessing_type == NeowBlessingType.RANDOM_COMMON_RELIC:
                initial_relics = len(run.relics)
                NeowHandler.apply_blessing(
                    run, opt, _make_rng(6000), _make_rng(6001), _make_rng(6002), _make_rng(6003)
                )
                assert len(run.relics) > initial_relics
                break

    def test_drawback_lose_gold(self):
        run = _make_run()
        from packages.engine.handlers.rooms import NeowBlessing
        blessing = NeowBlessing(
            NeowBlessingType.RANDOM_COLORLESS_RARE,
            "Test",
            NeowDrawbackType.LOSE_GOLD,
            "Lose all gold",
            0,
        )
        NeowHandler.apply_blessing(
            run, blessing, _make_rng(6000), _make_rng(6001), _make_rng(6002), _make_rng(6003)
        )
        assert run.gold == 0

    def test_drawback_gain_curse(self):
        run = _make_run()
        from packages.engine.handlers.rooms import NeowBlessing
        blessing = NeowBlessing(
            NeowBlessingType.RANDOM_COLORLESS_RARE,
            "Test",
            NeowDrawbackType.GAIN_CURSE,
            "Gain a curse",
            0,
        )
        initial_deck = len(run.deck)
        NeowHandler.apply_blessing(
            run, blessing, _make_rng(6000), _make_rng(6001), _make_rng(6002), _make_rng(6003)
        )
        assert len(run.deck) > initial_deck

    def test_get_neow_actions(self):
        options = NeowHandler.get_first_run_options()
        run = _make_run()
        actions = NeowHandler.get_neow_actions(run, options)
        assert len(actions) == 4
        for idx, desc in actions:
            assert isinstance(idx, int)
            assert isinstance(desc, str)


# =============================================================================
# RewardHandler Tests
# =============================================================================


class TestRewardHandler:
    """Tests for RewardHandler in reward_handler.py."""

    def test_generate_combat_rewards_normal(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "monster",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
        )
        assert rewards.gold is not None
        assert rewards.gold.amount > 0
        assert len(rewards.card_rewards) > 0

    def test_generate_combat_rewards_elite(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "elite",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
        )
        assert rewards.gold is not None
        assert rewards.gold.amount > 0
        assert rewards.relic is not None

    def test_generate_combat_rewards_deterministic(self):
        run1 = _make_run()
        run2 = _make_run()
        r1 = RewardHandler.generate_combat_rewards(
            run1, "monster",
            Random(SEED_LONG + 100), Random(SEED_LONG + 200),
            Random(SEED_LONG + 300), Random(SEED_LONG + 400),
        )
        r2 = RewardHandler.generate_combat_rewards(
            run2, "monster",
            Random(SEED_LONG + 100), Random(SEED_LONG + 200),
            Random(SEED_LONG + 300), Random(SEED_LONG + 400),
        )
        assert r1.gold.amount == r2.gold.amount
        assert [c.id for c in r1.card_rewards[0].cards] == [c.id for c in r2.card_rewards[0].cards]

    def test_take_gold(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "monster",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
        )
        old_gold = run.gold
        result = RewardHandler.execute_action(ClaimGoldAction(), run, rewards)
        assert result["success"]
        assert run.gold == old_gold + rewards.gold.amount

    def test_take_card(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "monster",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
        )
        if rewards.card_rewards:
            initial_deck = len(run.deck)
            action = PickCardAction(card_reward_index=0, card_index=0)
            result = RewardHandler.execute_action(action, run, rewards)
            assert result["success"]
            assert len(run.deck) == initial_deck + 1

    def test_take_potion(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "monster",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
        )
        actions = RewardHandler.get_available_actions(run, rewards)
        potion_action = next((a for a in actions if isinstance(a, ClaimPotionAction)), None)
        if potion_action:
            result = RewardHandler.execute_action(potion_action, run, rewards)
            assert result["success"]

    def test_take_potion_no_slots(self):
        from packages.engine.content.potions import ALL_POTIONS
        from packages.engine.handlers.reward_handler import CombatRewards, PotionReward
        run = _make_run()
        for slot in run.potion_slots:
            slot.potion_id = "FakePotion"
        potion = list(ALL_POTIONS.values())[0]
        rewards = CombatRewards(room_type="monster", enemies_killed=1)
        rewards.potion = PotionReward(potion=potion)
        result = RewardHandler.execute_action(ClaimPotionAction(), run, rewards)
        assert not result["success"]

    def test_take_emerald_key(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "elite",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
            is_burning_elite=True,
        )
        result = RewardHandler.execute_action(ClaimEmeraldKeyAction(), run, rewards)
        assert result["success"]
        assert run.has_emerald_key

    def test_take_emerald_key_twice(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "elite",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
            is_burning_elite=True,
        )
        RewardHandler.execute_action(ClaimEmeraldKeyAction(), run, rewards)
        result = RewardHandler.execute_action(ClaimEmeraldKeyAction(), run, rewards)
        assert not result["success"]

    def test_skip_rewards(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "monster",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
        )
        result = RewardHandler.execute_action(ProceedFromRewardsAction(), run, rewards)
        assert result["success"]
        assert result.get("proceeding_to_map") is True

    def test_burning_elite_has_emerald_key(self):
        run = _make_run()
        rewards = RewardHandler.generate_combat_rewards(
            run, "elite",
            _make_rng(100), _make_rng(200), _make_rng(300), _make_rng(400),
            is_burning_elite=True,
        )
        assert rewards.emerald_key is not None
