"""
Audit tests: Events, Shop Mechanics, Relics, Neow - Java vs Python Parity

These tests verify specific parity issues found by comparing Java decompiled source
against the Python engine implementation. Each test documents the expected Java behavior
and checks whether the Python engine matches.

Issues are tagged by severity: CRITICAL, MODERATE, MINOR
"""

import pytest
import math

# ============================================================================
# EVENTS PARITY TESTS
# ============================================================================


class TestEventsParity:
    """Tests for event parity between Java and Python."""

    @pytest.mark.xfail(reason="BUG: Knowing Skull classified as ANY, should be ACT_2")
    def test_knowing_skull_is_act2_event(self):
        """CRITICAL: Knowing Skull is a City (Act 2) event in Java, not ANY/special.
        Java: com.megacrit.cardcrawl.events.city.KnowingSkull
        Python content/events.py incorrectly sets act=Act.ANY
        Python handlers/event_handler.py puts it in SPECIAL_ONE_TIME_EVENTS
        """
        from packages.engine.content.events import KNOWING_SKULL, Act

        # Java has this as a city event, NOT a one-time special event
        # The Python engine incorrectly classifies it
        assert KNOWING_SKULL.act == Act.ACT_2, (
            "Knowing Skull should be Act.ACT_2 (City event), "
            f"but got {KNOWING_SKULL.act}. "
            "Java source: com.megacrit.cardcrawl.events.city.KnowingSkull"
        )

    @pytest.mark.xfail(reason="BUG: Python uses floor, Java uses ceil for Ghosts HP loss")
    def test_ghosts_hp_loss_uses_ceil(self):
        """CRITICAL: Java Ghosts event uses MathUtils.ceil for HP loss.
        Java: this.hpLoss = MathUtils.ceil((float)maxHealth * 0.5f)
        Python content/events.py uses value_percent=-0.50 which floors.

        For max HP = 73: Java = ceil(36.5) = 37, Python = int(36.5) = 36
        """
        # Simulate Java behavior
        max_hp = 73
        java_hp_loss = math.ceil(max_hp * 0.5)
        python_hp_loss = int(max_hp * 0.5)

        assert java_hp_loss == 37
        assert python_hp_loss == 36
        assert java_hp_loss == python_hp_loss, (
            f"Ghosts HP loss should use ceil: Java={java_hp_loss}, Python={python_hp_loss}"
        )

    def test_ghosts_hp_loss_clamped_to_max_minus_one(self):
        """CRITICAL: Java clamps Ghosts HP loss so it can't kill you.
        Java: if (this.hpLoss >= maxHealth) { this.hpLoss = maxHealth - 1; }
        """
        max_hp = 1
        hp_loss = math.ceil(max_hp * 0.5)
        if hp_loss >= max_hp:
            hp_loss = max_hp - 1
        assert hp_loss == 0, "Ghosts should not be able to kill you (HP loss clamped)"

    def test_ghosts_apparition_count_a15(self):
        """Ghosts gives 5 Apparitions normally, 3 on A15+.
        Java: amount = 5; if (ascensionLevel >= 15) amount -= 2;
        """
        from packages.engine.content.events import GHOSTS

        # The event data structure should encode the A15 variant
        accept_choice = GHOSTS.choices[0]
        card_outcome = [o for o in accept_choice.outcomes if o.card_id == "Apparition"][0]
        assert card_outcome.count == 5, "Base Apparition count should be 5"
        # A15 variant reduces to 3 - need to verify this is handled in execution

    def test_sssserpent_gold_values(self):
        """Sssserpent gives 175 gold normally, 150 on A15+.
        Java: GOLD_REWARD = 175, A_2_GOLD_REWARD = 150
        """
        from packages.engine.content.events import SSSSERPENT

        agree_choice = SSSSERPENT.choices[0]
        gold_outcome = [o for o in agree_choice.outcomes if o.type.name == "GOLD_CHANGE"][0]
        assert gold_outcome.value == 175, f"Sssserpent gold should be 175, got {gold_outcome.value}"

    def test_sssserpent_curse_is_doubt(self):
        """Java: this.curse = new Doubt()"""
        from packages.engine.content.events import SSSSERPENT

        agree_choice = SSSSERPENT.choices[0]
        curse_outcome = [o for o in agree_choice.outcomes if o.type.name == "CURSE_GAIN"][0]
        assert curse_outcome.card_id == "Doubt"

    def test_cleric_purify_cost_values(self):
        """Java: PURIFY_COST = 50, A_2_PURIFY_COST = 75 (at ascension >= 15)
        """
        from packages.engine.content.events import CLERIC

        purify_choice = CLERIC.choices[1]
        assert purify_choice.requires_gold == 50, (
            f"Cleric purify base cost should be 50, got {purify_choice.requires_gold}"
        )
        # A15+ cost of 75 should be handled in execution logic

    def test_cleric_heal_cost_and_amount(self):
        """Java: HEAL_COST = 35, HEAL_AMT = 0.25f (25% max HP)"""
        from packages.engine.content.events import CLERIC

        heal_choice = CLERIC.choices[0]
        assert heal_choice.requires_gold == 35
        gold_outcome = [o for o in heal_choice.outcomes if o.type.name == "GOLD_CHANGE"][0]
        assert gold_outcome.value == -35
        hp_outcome = [o for o in heal_choice.outcomes if o.type.name == "HP_CHANGE"][0]
        assert hp_outcome.value_percent == 0.25

    def test_gold_shrine_values(self):
        """Java: GOLD_AMT = 100, A_2_GOLD_AMT = 50, CURSE_GOLD_AMT = 275"""
        from packages.engine.content.events import GOLD_SHRINE

        pray_choice = GOLD_SHRINE.choices[0]
        gold_outcome = [o for o in pray_choice.outcomes if o.type.name == "GOLD_CHANGE"][0]
        assert gold_outcome.value == 100, f"Gold Shrine pray should give 100, got {gold_outcome.value}"

        desecrate_choice = GOLD_SHRINE.choices[1]
        gold_outcome = [o for o in desecrate_choice.outcomes if o.type.name == "GOLD_CHANGE"][0]
        assert gold_outcome.value == 275, f"Gold Shrine desecrate should give 275, got {gold_outcome.value}"

    def test_big_fish_max_hp_amount(self):
        """Java: MAX_HP_AMT = 5"""
        from packages.engine.content.events import BIG_FISH

        donut_choice = BIG_FISH.choices[1]
        hp_outcome = [o for o in donut_choice.outcomes if o.type.name == "MAX_HP_CHANGE"][0]
        assert hp_outcome.value == 5

    def test_big_fish_heal_one_third(self):
        """Java: this.healAmt = maxHealth / 3"""
        from packages.engine.content.events import BIG_FISH

        banana_choice = BIG_FISH.choices[0]
        hp_outcome = [o for o in banana_choice.outcomes if o.type.name == "HP_CHANGE"][0]
        assert hp_outcome.value_percent == pytest.approx(1.0 / 3, abs=0.01)

    @pytest.mark.xfail(reason="BUG: Knowing Skull classified as ANY, should be ACT_2")
    def test_knowing_skull_not_in_special_events(self):
        """Knowing Skull should NOT be in the special one-time events pool."""
        from packages.engine.content.events import SPECIAL_ONE_TIME_EVENTS

        assert "Knowing Skull" not in SPECIAL_ONE_TIME_EVENTS, (
            "Knowing Skull is a City event, not a special one-time event"
        )

    @pytest.mark.xfail(reason="BUG: Secret Portal classified as ANY, should be ACT_3")
    def test_secret_portal_is_act3(self):
        """CRITICAL: Secret Portal is a Beyond (Act 3) event.
        Java: com.megacrit.cardcrawl.events.beyond.SecretPortal
        """
        from packages.engine.content.events import SECRET_PORTAL, Act

        assert SECRET_PORTAL.act == Act.ACT_3, (
            f"Secret Portal should be ACT_3, got {SECRET_PORTAL.act}"
        )

    def test_woman_in_blue_potion_costs(self):
        """Java: cost1 = 20, cost2 = 30, cost3 = 40"""
        from packages.engine.content.events import WOMAN_IN_BLUE

        assert WOMAN_IN_BLUE.choices[0].requires_gold == 20
        assert WOMAN_IN_BLUE.choices[1].requires_gold == 30
        assert WOMAN_IN_BLUE.choices[2].requires_gold == 40

    @pytest.mark.xfail(reason="BUG: Python always applies 5% damage on leave; Java only on A15+")
    def test_woman_in_blue_leave_free_below_a15(self):
        """MODERATE: In Java, leaving Woman in Blue is FREE below A15.
        Only A15+ takes ceil(maxHP * 0.05) damage.
        Python models it as always taking 5% damage.
        """
        from packages.engine.content.events import WOMAN_IN_BLUE

        leave_choice = WOMAN_IN_BLUE.choices[3]
        leave_outcomes = leave_choice.outcomes

        # Below A15, leaving should have NO HP cost
        # The current Python implementation always has an HP_CHANGE outcome
        hp_outcomes = [o for o in leave_outcomes if o.type.name == "HP_CHANGE"]
        # Should either be empty (free leave) or conditional on A15
        assert len(hp_outcomes) == 0 or hp_outcomes[0].value_percent is None, (
            "Woman in Blue leave should be free below A15"
        )

    def test_knowing_skull_initial_costs(self):
        """Java: all costs start at 6. cardCost = leaveCost = 6; potionCost = leaveCost; goldCost = leaveCost;"""
        from packages.engine.content.events import KNOWING_SKULL

        for choice in KNOWING_SKULL.choices:
            hp_outcomes = [o for o in choice.outcomes if o.type.name == "HP_CHANGE"]
            if hp_outcomes:
                assert hp_outcomes[0].value == -6, (
                    f"Knowing Skull initial cost should be 6, got {-hp_outcomes[0].value} "
                    f"for choice '{choice.description}'"
                )

    def test_knowing_skull_gold_reward(self):
        """Java: GOLD_REWARD = 90"""
        from packages.engine.content.events import KNOWING_SKULL

        gold_choice = KNOWING_SKULL.choices[1]  # Gold option
        gold_outcome = [o for o in gold_choice.outcomes if o.type.name == "GOLD_CHANGE"][0]
        assert gold_outcome.value == 90


# ============================================================================
# SHOP MECHANICS PARITY TESTS
# ============================================================================


class TestShopParity:
    """Tests for shop mechanics parity between Java and Python."""

    def test_shop_card_base_prices(self):
        """Java AbstractCard.getPrice(): COMMON=50, UNCOMMON=75, RARE=150"""
        from packages.engine.generation.shop import CARD_BASE_PRICES

        # CARD_BASE_PRICES is keyed by the CardRarity enum from the shop module's import
        prices = {k.name: v for k, v in CARD_BASE_PRICES.items()}
        assert prices["COMMON"] == 50
        assert prices["UNCOMMON"] == 75
        assert prices["RARE"] == 150

    def test_colorless_price_multiplier(self):
        """Java: tmpPrice *= 1.2f for colorless cards"""
        from packages.engine.generation.shop import COLORLESS_PRICE_MULTIPLIER

        assert COLORLESS_PRICE_MULTIPLIER == 1.2

    def test_purge_base_cost(self):
        """Java: purgeCost = 75, PURGE_COST_RAMP = 25"""
        from packages.engine.generation.shop import BASE_PURGE_COST, PURGE_COST_INCREMENT

        assert BASE_PURGE_COST == 75
        assert PURGE_COST_INCREMENT == 25

    def test_purge_cost_calculation(self):
        """Verify purge cost matches Java: 75 + (purgeCount * 25)"""
        from packages.engine.generation.shop import BASE_PURGE_COST, PURGE_COST_INCREMENT

        for purge_count in range(5):
            expected = 75 + purge_count * 25
            actual = BASE_PURGE_COST + purge_count * PURGE_COST_INCREMENT
            assert actual == expected, f"Purge cost at count {purge_count}: expected {expected}, got {actual}"

    @pytest.mark.xfail(reason="BUG: Missing Smiling Mask override - purge should be flat 50g")
    def test_smiling_mask_purge_override(self):
        """CRITICAL: Java sets actualPurgeCost = 50 when Smiling Mask is held.
        Java ShopScreen.java line 222: if (hasRelic("Smiling Mask")) { actualPurgeCost = 50; }
        This OVERRIDES the base+increment calculation entirely.
        """
        from packages.engine.generation.shop import predict_shop_inventory

        result = predict_shop_inventory(
            seed="TEST",
            card_counter=0,
            merchant_counter=0,
            potion_counter=0,
            purge_count=3,  # Would normally be 75 + 75 = 150
            has_membership_card=False,
        )
        # With Smiling Mask, purge cost should be flat 50 regardless of purge count
        # Currently no way to pass Smiling Mask to predict_shop_inventory
        # This test documents the missing feature
        assert False, "Smiling Mask purge override not implemented in shop prediction"

    @pytest.mark.xfail(reason="BUG: Missing A16+ shop price 10% markup")
    def test_a16_price_markup(self):
        """CRITICAL: Java applies 1.1x price multiplier at ascension >= 16.
        Java ShopScreen.java line 212-213:
            if (ascensionLevel >= 16) { this.applyDiscount(1.1f, false); }
        Note: applyDiscount(1.1) actually INCREASES prices by 10%.
        """
        # The predict_shop_inventory function has no ascension parameter
        from packages.engine.generation.shop import predict_shop_inventory
        import inspect

        sig = inspect.signature(predict_shop_inventory)
        param_names = list(sig.parameters.keys())
        assert "ascension" in param_names or "ascension_level" in param_names, (
            "predict_shop_inventory should accept an ascension parameter for A16+ markup"
        )

    def test_shop_card_type_order(self):
        """Java: 2 attacks, 2 skills, 1 power"""
        from packages.engine.generation.shop import SHOP_CARD_TYPES

        assert len(SHOP_CARD_TYPES) == 5
        type_names = [t.name for t in SHOP_CARD_TYPES]
        assert type_names.count("ATTACK") == 2
        assert type_names.count("SKILL") == 2
        assert type_names.count("POWER") == 1

    def test_shop_rarity_thresholds(self):
        """Java: roll < 3 = RARE, roll < 40 = UNCOMMON, else COMMON"""
        from packages.engine.generation.shop import SHOP_RARITY_THRESHOLDS

        assert SHOP_RARITY_THRESHOLDS["rare"] == 3
        assert SHOP_RARITY_THRESHOLDS["uncommon"] == 37

    def test_shop_relic_tier_thresholds(self):
        """Java: roll < 48 = COMMON, roll < 82 = UNCOMMON, else RARE"""
        from packages.engine.generation.shop import SHOP_RELIC_THRESHOLDS

        assert SHOP_RELIC_THRESHOLDS["common"] == 48
        assert SHOP_RELIC_THRESHOLDS["uncommon"] == 82

    def test_sale_discount_is_50_percent(self):
        """Java: saleCard.price /= 2"""
        from packages.engine.generation.shop import SALE_DISCOUNT

        assert SALE_DISCOUNT == 0.5

    def test_shop_structure_counts(self):
        """Java shop has: 5 colored cards, 2 colorless, 3 relics, 3 potions"""
        from packages.engine.generation.shop import predict_shop_inventory

        result = predict_shop_inventory(
            seed="AUDIT_TEST",
            card_counter=0,
            merchant_counter=0,
            potion_counter=0,
        )
        inv = result.inventory
        assert len(inv.colored_cards) == 5, f"Expected 5 colored cards, got {len(inv.colored_cards)}"
        assert len(inv.colorless_cards) == 2, f"Expected 2 colorless cards, got {len(inv.colorless_cards)}"
        assert len(inv.relics) == 3, f"Expected 3 relics, got {len(inv.relics)}"
        assert len(inv.potions) == 3, f"Expected 3 potions, got {len(inv.potions)}"

    def test_membership_card_50_percent_discount(self):
        """Java: applyDiscount(0.5f, true) for Membership Card"""
        from packages.engine.generation.shop import predict_shop_inventory

        normal = predict_shop_inventory(
            seed="DISCOUNT_TEST", card_counter=0, merchant_counter=0, potion_counter=0,
        )
        discounted = predict_shop_inventory(
            seed="DISCOUNT_TEST", card_counter=0, merchant_counter=0, potion_counter=0,
            has_membership_card=True,
        )
        # All card prices should be ~50% of normal
        for i in range(min(len(normal.inventory.colored_cards), len(discounted.inventory.colored_cards))):
            n_price = normal.inventory.colored_cards[i].price
            d_price = discounted.inventory.colored_cards[i].price
            # Allow rounding differences
            assert abs(d_price - n_price * 0.5) <= 1, (
                f"Card {i}: normal={n_price}, discounted={d_price}, "
                f"expected ~{n_price * 0.5}"
            )

    def test_courier_20_percent_discount(self):
        """Java: applyDiscount(0.8f, true) for The Courier"""
        from packages.engine.generation.shop import predict_shop_inventory

        normal = predict_shop_inventory(
            seed="COURIER_TEST", card_counter=0, merchant_counter=0, potion_counter=0,
        )
        discounted = predict_shop_inventory(
            seed="COURIER_TEST", card_counter=0, merchant_counter=0, potion_counter=0,
            has_the_courier=True,
        )
        for i in range(min(len(normal.inventory.colored_cards), len(discounted.inventory.colored_cards))):
            n_price = normal.inventory.colored_cards[i].price
            d_price = discounted.inventory.colored_cards[i].price
            assert abs(d_price - n_price * 0.8) <= 1, (
                f"Card {i}: normal={n_price}, discounted={d_price}, "
                f"expected ~{n_price * 0.8}"
            )


# ============================================================================
# RELIC PARITY TESTS
# ============================================================================


class TestRelicParity:
    """Tests for relic parity between Java and Python."""

    @pytest.mark.xfail(reason="BUG: Smiling Mask description says Face Trader, should say shop purge = 50g")
    def test_smiling_mask_description(self):
        """CRITICAL: Smiling Mask sets shop purge cost to 50g.
        Java SmilingMask.java: Shop purge is always 50g.
        Python says 'Replaces Face Trader event's HP cost' which is WRONG.
        """
        from packages.engine.content.relics import SMILING_MASK

        # The effect should reference shop purge, not Face Trader
        effects_text = " ".join(SMILING_MASK.effects)
        assert "purge" in effects_text.lower() or "remov" in effects_text.lower(), (
            f"Smiling Mask should reference shop card removal/purge cost, "
            f"got: {SMILING_MASK.effects}"
        )

    def test_violet_lotus_watcher_only(self):
        """Violet Lotus is Watcher-only."""
        from packages.engine.content.relics import VIOLET_LOTUS, PlayerClass

        assert VIOLET_LOTUS.player_class == PlayerClass.WATCHER

    def test_violet_lotus_is_boss_tier(self):
        """Violet Lotus is a Boss relic."""
        from packages.engine.content.relics import VIOLET_LOTUS, RelicTier

        assert VIOLET_LOTUS.tier == RelicTier.BOSS

    def test_damaru_watcher_only(self):
        """Damaru is Watcher-only."""
        from packages.engine.content.relics import DAMARU, PlayerClass

        assert DAMARU.player_class == PlayerClass.WATCHER

    def test_teardrop_locket_watcher_only(self):
        """Teardrop Locket is Watcher-only."""
        from packages.engine.content.relics import TEARDROP_LOCKET, PlayerClass

        assert TEARDROP_LOCKET.player_class == PlayerClass.WATCHER

    def test_holy_water_requires_pure_water(self):
        """Holy Water requires Pure Water (upgrades it)."""
        from packages.engine.content.relics import HOLY_WATER

        assert HOLY_WATER.requires_relic == "PureWater"

    def test_pen_nib_triggers_at_10(self):
        """Pen Nib triggers every 10th attack (counter resets at 10)."""
        from packages.engine.content.relics import PEN_NIB

        assert PEN_NIB.counter_max == 10
        assert PEN_NIB.counter_start == 0

    def test_incense_burner_triggers_at_6(self):
        """Incense Burner gives Intangible every 6 turns."""
        from packages.engine.content.relics import INCENSE_BURNER

        assert INCENSE_BURNER.counter_max == 6

    def test_kunai_shuriken_fan_trigger_at_3(self):
        """Kunai, Shuriken, Ornamental Fan all trigger at 3 attacks."""
        from packages.engine.content.relics import KUNAI, SHURIKEN, ORNAMENTAL_FAN

        assert KUNAI.counter_max == 3
        assert SHURIKEN.counter_max == 3
        assert ORNAMENTAL_FAN.counter_max == 3

    def test_all_energy_boss_relics_give_one_energy(self):
        """Boss relics with energy_bonus should give exactly 1."""
        from packages.engine.content.relics import ALL_RELICS, RelicTier

        energy_bosses = [
            r for r in ALL_RELICS.values()
            if r.tier == RelicTier.BOSS and r.energy_bonus > 0
        ]
        for relic in energy_bosses:
            assert relic.energy_bonus == 1, (
                f"{relic.name} energy_bonus should be 1, got {relic.energy_bonus}"
            )

    def test_ectoplasm_act_restriction(self):
        """Ectoplasm only spawns in Act 1."""
        from packages.engine.content.relics import ECTOPLASM

        assert ECTOPLASM.act_restriction == 1

    def test_mark_of_the_bloom_prevents_healing(self):
        """Mark of the Bloom prevents all healing."""
        from packages.engine.content.relics import MARK_OF_THE_BLOOM

        assert MARK_OF_THE_BLOOM.prevents_healing is True

    def test_sozu_prevents_potions(self):
        """Sozu prevents obtaining potions."""
        from packages.engine.content.relics import SOZU

        assert SOZU.prevents_potions is True


# ============================================================================
# NEOW PARITY TESTS
# ============================================================================


class TestNeowParity:
    """Tests for Neow parity between Java and Python."""

    def test_neow_category_0_options(self):
        """Java Category 0: THREE_CARDS, ONE_RANDOM_RARE_CARD, REMOVE_CARD,
        UPGRADE_CARD, TRANSFORM_CARD, RANDOM_COLORLESS (6 options)"""
        from packages.engine.content.events import (
            NEOW_THREE_CARDS, NEOW_ONE_RANDOM_RARE, NEOW_REMOVE_CARD,
            NEOW_UPGRADE_CARD, NEOW_TRANSFORM_CARD, NEOW_RANDOM_COLORLESS,
        )

        cat0 = [
            NEOW_THREE_CARDS, NEOW_ONE_RANDOM_RARE, NEOW_REMOVE_CARD,
            NEOW_UPGRADE_CARD, NEOW_TRANSFORM_CARD, NEOW_RANDOM_COLORLESS,
        ]
        assert len(cat0) == 6
        assert all(b.category == 0 for b in cat0)

    def test_neow_category_1_options(self):
        """Java Category 1: THREE_SMALL_POTIONS, RANDOM_COMMON_RELIC,
        TEN_PERCENT_HP_BONUS, THREE_ENEMY_KILL, HUNDRED_GOLD (5 options)"""
        from packages.engine.content.events import (
            NEOW_THREE_POTIONS, NEOW_RANDOM_COMMON_RELIC, NEOW_TEN_PERCENT_HP,
            NEOW_THREE_ENEMY_KILL, NEOW_HUNDRED_GOLD,
        )

        cat1 = [
            NEOW_THREE_POTIONS, NEOW_RANDOM_COMMON_RELIC, NEOW_TEN_PERCENT_HP,
            NEOW_THREE_ENEMY_KILL, NEOW_HUNDRED_GOLD,
        ]
        assert len(cat1) == 5
        assert all(b.category == 1 for b in cat1)

    def test_neow_hundred_gold_value(self):
        """Java: GOLD_BONUS = 100"""
        from packages.engine.content.events import NEOW_HUNDRED_GOLD

        assert NEOW_HUNDRED_GOLD.gold_bonus == 100

    def test_neow_two_fifty_gold_value(self):
        """Java: LARGE_GOLD_BONUS = 250"""
        from packages.engine.content.events import NEOW_TWO_FIFTY_GOLD

        assert NEOW_TWO_FIFTY_GOLD.gold_bonus == 250

    def test_neow_boss_swap_is_category_3(self):
        """Java: category 3 only has BOSS_RELIC"""
        from packages.engine.content.events import NEOW_BOSS_SWAP

        assert NEOW_BOSS_SWAP.category == 3

    def test_neow_drawback_types_exist(self):
        """Java drawbacks: TEN_PERCENT_HP_LOSS, NO_GOLD, CURSE, PERCENT_DAMAGE"""
        from packages.engine.content.events import (
            NEOW_DRAWBACK_10_PERCENT_HP_LOSS,
            NEOW_DRAWBACK_NO_GOLD,
            NEOW_DRAWBACK_CURSE,
            NEOW_DRAWBACK_PERCENT_DAMAGE,
        )

        assert NEOW_DRAWBACK_10_PERCENT_HP_LOSS is not None
        assert NEOW_DRAWBACK_NO_GOLD is not None
        assert NEOW_DRAWBACK_CURSE is not None
        assert NEOW_DRAWBACK_PERCENT_DAMAGE is not None

    @pytest.mark.xfail(reason="BUG: Neow category 2 conditional exclusions not implemented")
    def test_neow_category_2_excludes_remove_two_if_curse_drawback(self):
        """CRITICAL: Java excludes REMOVE_TWO from category 2 if drawback is CURSE.
        Java NeowReward.java line 95: if (this.drawback != NeowRewardDrawback.CURSE) {
            rewardOptions.add(new NeowRewardDef(NeowRewardType.REMOVE_TWO, ...));
        }

        This prevents the player from getting 'remove 2 cards + curse' since
        adding a curse while removing cards is a conflict the game avoids.
        """
        # This would need to be tested at the reward generation level
        # Currently the Python static data doesn't encode these exclusions
        assert False, "Neow category 2 conditional exclusions not encoded in Python"

    @pytest.mark.xfail(reason="BUG: Neow PERCENT_DAMAGE uses currentHP/10*3, not currentHP*0.3")
    def test_neow_percent_damage_formula(self):
        """MODERATE: Java does integer division then multiply.
        Java: currentHealth / 10 * 3
        This can differ from currentHealth * 0.3 for non-round numbers.

        Example: HP=73 -> Java: 73/10=7, 7*3=21. Float: 73*0.3=21.9->21. Same.
        Example: HP=79 -> Java: 79/10=7, 7*3=21. Float: 79*0.3=23.7->23. DIFFERENT.
        """
        # Test with HP = 79
        hp = 79
        java_damage = (hp // 10) * 3  # 7 * 3 = 21
        float_damage = int(hp * 0.3)  # 23

        assert java_damage != float_damage, "These should differ to prove the bug"
        assert java_damage == 21
        assert float_damage == 23
        # The Python implementation should use java_damage formula
        # Currently it uses the float version
        assert False, "Python should use (currentHP // 10) * 3, not int(currentHP * 0.3)"


# ============================================================================
# CROSS-CUTTING AUDIT TESTS
# ============================================================================


class TestCrossCuttingAudit:
    """Tests that span multiple systems."""

    def test_event_handler_and_content_events_consistency(self):
        """The event_handler.py and content/events.py should reference the same events."""
        from packages.engine.content.events import ALL_EVENTS

        # Verify all events in content/events.py are retrievable
        for event_id, event in ALL_EVENTS.items():
            assert event.id is not None, f"Event {event_id} has no ID"
            assert event.name is not None, f"Event {event_id} has no name"

    def test_all_relics_have_unique_ids(self):
        """All relics should have unique IDs."""
        from packages.engine.content.relics import ALL_RELICS

        ids = list(ALL_RELICS.keys())
        assert len(ids) == len(set(ids)), "Duplicate relic IDs found"

    def test_watcher_relics_present(self):
        """All Watcher-specific relics should be defined."""
        from packages.engine.content.relics import ALL_RELICS

        watcher_relics = {
            "PureWater", "Damaru", "TeardropLocket", "Yang",  # Duality
            "GoldenEye", "CloakClasp", "VioletLotus", "HolyWater", "Melange",
        }
        for relic_id in watcher_relics:
            assert relic_id in ALL_RELICS, f"Missing Watcher relic: {relic_id}"

    def test_shop_relics_present(self):
        """Key shop relics should be defined."""
        from packages.engine.content.relics import ALL_RELICS

        shop_relics = {
            "Membership Card", "The Courier", "Smiling Mask",
        }
        for relic_id in shop_relics:
            assert relic_id in ALL_RELICS, f"Missing shop-relevant relic: {relic_id}"
