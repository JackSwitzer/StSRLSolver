"""Tests for previously missing relic implementations.

Covers:
1. Girya atBattleStart - grants Strength equal to lift counter
2. Discerning Monocle - 20% shop discount
3. Dead onEquip handler correctness (Girya counter = 0, not 3)
"""

import pytest

from packages.engine.registry import (
    execute_relic_triggers,
    RELIC_REGISTRY,
)
from packages.engine.state.combat import (
    create_combat,
    create_enemy,
)
from packages.engine.state.run import create_watcher_run
from packages.engine.handlers.rooms import RestHandler
from packages.engine.handlers.combat import CombatRunner
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.content.enemies import JawWorm


# =============================================================================
# HELPERS
# =============================================================================


def _make_combat_with_relic(relic_id, relic_counter=-1):
    """Create a combat state with a specific relic and optional counter."""
    state = create_combat(
        player_hp=70,
        player_max_hp=80,
        enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
        deck=["Strike_P"] * 5 + ["Defend_P"] * 5,
        energy=3,
        relics=[relic_id],
    )
    if relic_counter >= 0:
        state.set_relic_counter(relic_id, relic_counter)
    return state


def _make_runner(relics=None, hp=80, max_hp=80):
    """Create a CombatRunner with given relics."""
    run = create_watcher_run("TEST_GIRYA", ascension=0)
    run.max_hp = max_hp
    run.current_hp = hp
    if relics:
        for relic_id in relics:
            run.add_relic(relic_id)
    rng = Random(12345)
    ai_rng = Random(12346)
    hp_rng = Random(12347)
    enemies = [JawWorm(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)]
    return CombatRunner(run_state=run, enemies=enemies, shuffle_rng=rng)


# =============================================================================
# GIRYA: atBattleStart grants Strength = counter
# =============================================================================


class TestGiryaCombatStrength:
    """Girya grants Strength at combat start equal to lift counter."""

    def test_girya_no_lifts_no_strength(self):
        """Girya with counter=0 should not grant Strength."""
        state = _make_combat_with_relic("Girya", relic_counter=0)
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Strength", 0) == 0

    def test_girya_one_lift_grants_1_strength(self):
        """Girya with counter=1 should grant 1 Strength."""
        state = _make_combat_with_relic("Girya", relic_counter=1)
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Strength", 0) == 1

    def test_girya_two_lifts_grants_2_strength(self):
        """Girya with counter=2 should grant 2 Strength."""
        state = _make_combat_with_relic("Girya", relic_counter=2)
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Strength", 0) == 2

    def test_girya_three_lifts_grants_3_strength(self):
        """Girya with counter=3 should grant 3 Strength."""
        state = _make_combat_with_relic("Girya", relic_counter=3)
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Strength", 0) == 3

    def test_girya_handler_registered(self):
        """Girya should have an atBattleStart handler."""
        assert RELIC_REGISTRY.has_handler("atBattleStart", "Girya")

    def test_girya_lift_then_combat_grants_strength(self):
        """Full flow: lift at rest site, then start combat with Strength."""
        run = create_watcher_run("TEST_GIRYA_FLOW", ascension=0)
        run.add_relic("Girya")

        # Lift twice
        RestHandler.lift(run)
        RestHandler.lift(run)
        assert run.get_relic("Girya").counter == 2

        # Start combat
        rng = Random(12345)
        ai_rng = Random(12346)
        hp_rng = Random(12347)
        enemies = [JawWorm(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)]
        runner = CombatRunner(run_state=run, enemies=enemies, shuffle_rng=rng)

        # Girya should have granted 2 Strength at combat start
        assert runner.state.player.statuses.get("Strength", 0) == 2

    def test_girya_negative_counter_no_strength(self):
        """Girya with default counter (-1) should not grant Strength."""
        state = _make_combat_with_relic("Girya")
        # Default counter is -1
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Strength", 0) == 0


# =============================================================================
# GIRYA: onEquip handler correctness
# =============================================================================


class TestGiryaOnEquip:
    """Girya onEquip should set counter to 0 (Java: this.counter = 0)."""

    def test_girya_onequip_handler_exists(self):
        """Girya should have an onEquip handler registered."""
        assert RELIC_REGISTRY.has_handler("onEquip", "Girya")

    def test_girya_onequip_sets_counter_to_zero(self):
        """Girya onEquip should initialize counter to 0, not 3."""
        state = _make_combat_with_relic("Girya")
        # Set counter to something else first
        state.set_relic_counter("Girya", 99)
        execute_relic_triggers("onEquip", state)
        assert state.get_relic_counter("Girya") == 0


# =============================================================================
# DISCERNING MONOCLE: 20% shop discount
# =============================================================================


class TestDiscerningMonocleShopDiscount:
    """Discerning Monocle should apply a 0.8x multiplier to shop prices."""

    def test_discount_applied_in_rewards_generation(self):
        """generate_shop_inventory should accept and apply discerning_monocle flag."""
        from packages.engine.generation.rewards import generate_shop_inventory, RewardState
        rng = Random(seed_to_long("SHOP_TEST"))

        # Without Discerning Monocle
        inv_normal = generate_shop_inventory(
            rng=Random(seed_to_long("SHOP_TEST")),
            reward_state=RewardState(),
            act=1,
            player_class="WATCHER",
            has_discerning_monocle=False,
        )

        # With Discerning Monocle
        inv_discount = generate_shop_inventory(
            rng=Random(seed_to_long("SHOP_TEST")),
            reward_state=RewardState(),
            act=1,
            player_class="WATCHER",
            has_discerning_monocle=True,
        )

        # All card prices should be ~80% of normal (within rounding)
        for (card_n, price_n), (card_d, price_d) in zip(
            inv_normal.colored_cards, inv_discount.colored_cards
        ):
            # Price with discount should be 80% of normal
            expected = int(price_n * 0.8)
            # Allow +-1 for float rounding
            assert abs(price_d - expected) <= 1, \
                f"Card {card_n.id}: expected ~{expected}, got {price_d} (normal={price_n})"

    def test_discount_stacks_with_membership_card(self):
        """Discerning Monocle stacks multiplicatively with Membership Card."""
        from packages.engine.generation.rewards import generate_shop_inventory, RewardState

        # With Membership Card only (50%)
        inv_membership = generate_shop_inventory(
            rng=Random(seed_to_long("SHOP_STACK")),
            reward_state=RewardState(),
            act=1,
            player_class="WATCHER",
            has_membership_card=True,
            has_discerning_monocle=False,
        )

        # With Membership Card + Discerning Monocle (50% * 80% = 40%)
        inv_both = generate_shop_inventory(
            rng=Random(seed_to_long("SHOP_STACK")),
            reward_state=RewardState(),
            act=1,
            player_class="WATCHER",
            has_membership_card=True,
            has_discerning_monocle=True,
        )

        for (_, price_m), (_, price_b) in zip(
            inv_membership.colored_cards, inv_both.colored_cards
        ):
            # With both, price should be 80% of membership-only price
            expected = int(price_m * 0.8)
            assert abs(price_b - expected) <= 1, \
                f"Expected ~{expected}, got {price_b} (membership={price_m})"

    def test_predict_shop_accepts_discerning_monocle_flag(self):
        """predict_shop_inventory should accept has_discerning_monocle."""
        from packages.engine.generation.shop import predict_shop_inventory

        # Should not raise
        result = predict_shop_inventory(
            seed="MONOCLE_TEST",
            card_counter=0,
            merchant_counter=0,
            potion_counter=0,
            act=1,
            player_class="WATCHER",
            has_discerning_monocle=True,
        )
        assert result is not None
        assert len(result.inventory.colored_cards) > 0

    def test_predict_shop_discount_applied(self):
        """predict_shop_inventory with Discerning Monocle should have lower prices."""
        from packages.engine.generation.shop import predict_shop_inventory

        result_normal = predict_shop_inventory(
            seed="MONO_PRICE",
            card_counter=0,
            merchant_counter=0,
            potion_counter=0,
            act=1,
            player_class="WATCHER",
            has_discerning_monocle=False,
        )

        result_discount = predict_shop_inventory(
            seed="MONO_PRICE",
            card_counter=0,
            merchant_counter=0,
            potion_counter=0,
            act=1,
            player_class="WATCHER",
            has_discerning_monocle=True,
        )

        # ShopCard has .card and .price attributes
        for sc_normal, sc_discount in zip(
            result_normal.inventory.colored_cards,
            result_discount.inventory.colored_cards,
        ):
            expected = int(sc_normal.price * 0.8)
            assert abs(sc_discount.price - expected) <= 1, \
                f"Card {sc_normal.card.id}: expected ~{expected}, got {sc_discount.price} (normal={sc_normal.price})"
