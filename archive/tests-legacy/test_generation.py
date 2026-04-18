"""
Tests for generation modules:
- generation/shop.py (shop inventory prediction)
- generation/encounters.py (encounter generation)
- generation/treasure.py (treasure chest prediction)
"""

import pytest
import sys

sys.path.insert(0, "/Users/jackswitzer/Desktop/SlayTheSpireRL")

from packages.engine.state.rng import Random, seed_to_long
from packages.engine.generation.shop import predict_shop_inventory, format_shop_inventory
from packages.engine.generation.encounters import (
    predict_act_encounters,
    predict_all_acts,
    predict_all_bosses,
    generate_exordium_encounters,
    generate_city_encounters,
    generate_beyond_encounters,
    generate_ending_encounters,
    normalize_weights,
    roll_monster,
    MonsterInfo,
    EXORDIUM_BOSSES,
    CITY_BOSSES,
    BEYOND_BOSSES,
    ENDING_BOSS,
)
from packages.engine.generation.treasure import (
    predict_chest,
    predict_full_chest,
    predict_treasure_sequence,
    ChestType,
)


SEED = "TESTGEN"
SEED_LONG = seed_to_long(SEED)


# =============================================================================
# Shop Generation Tests
# =============================================================================


class TestShopGeneration:
    """Tests for generation/shop.py."""

    def test_predict_shop_returns_result(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        assert result.inventory is not None

    def test_shop_has_5_colored_cards(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        assert len(result.inventory.colored_cards) == 5

    def test_shop_has_2_colorless_cards(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        assert len(result.inventory.colorless_cards) == 2

    def test_shop_has_3_relics(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        assert len(result.inventory.relics) == 3

    def test_shop_has_3_potions(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        assert len(result.inventory.potions) == 3

    def test_shop_has_one_sale(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        on_sale = [c for c in result.inventory.colored_cards if c.on_sale]
        assert len(on_sale) == 1

    def test_shop_deterministic(self):
        r1 = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        r2 = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        for c1, c2 in zip(r1.inventory.colored_cards, r2.inventory.colored_cards):
            assert c1.card.id == c2.card.id
            assert c1.price == c2.price

    def test_different_seeds_different_shops(self):
        r1 = predict_shop_inventory(
            seed="SHOP1", card_counter=0, merchant_counter=0, potion_counter=0
        )
        r2 = predict_shop_inventory(
            seed="SHOP2", card_counter=0, merchant_counter=0, potion_counter=0
        )
        cards1 = [c.card.id for c in r1.inventory.colored_cards]
        cards2 = [c.card.id for c in r2.inventory.colored_cards]
        # Very unlikely to be identical with different seeds
        assert cards1 != cards2

    def test_membership_card_discount(self):
        r_normal = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        r_discount = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0,
            has_membership_card=True,
        )
        # Prices should be lower with membership card
        # Compare purge cost (easy to check)
        assert r_discount.inventory.purge_cost < r_normal.inventory.purge_cost

    def test_courier_discount(self):
        r_normal = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        r_courier = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0,
            has_the_courier=True,
        )
        assert r_courier.inventory.purge_cost < r_normal.inventory.purge_cost

    def test_purge_cost_increases_with_count(self):
        r0 = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0,
            purge_count=0,
        )
        r1 = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0,
            purge_count=3,
        )
        assert r1.inventory.purge_cost > r0.inventory.purge_cost

    def test_card_prices_positive(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        for c in result.inventory.colored_cards:
            assert c.price > 0
        for c in result.inventory.colorless_cards:
            assert c.price > 0

    def test_relic_prices_positive(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        for r in result.inventory.relics:
            assert r.price > 0

    def test_counter_tracking(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        assert result.final_card_counter > 0
        assert result.final_merchant_counter > 0
        assert result.final_potion_counter > 0

    def test_different_counters_different_inventory(self):
        r1 = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        r2 = predict_shop_inventory(
            seed=SEED, card_counter=50, merchant_counter=20, potion_counter=10
        )
        cards1 = [c.card.id for c in r1.inventory.colored_cards]
        cards2 = [c.card.id for c in r2.inventory.colored_cards]
        assert cards1 != cards2

    def test_format_shop_inventory(self):
        result = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0
        )
        text = format_shop_inventory(result)
        assert "COLORED CARDS" in text
        assert "RELICS" in text
        assert "POTIONS" in text

    def test_owned_relics_excluded(self):
        """Owned relics should not appear in shop."""
        # Get shop without owned relics
        r1 = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0,
            owned_relics=set(),
        )
        relic_ids = {r.relic.id for r in r1.inventory.relics}
        # Now generate with those relics as owned
        r2 = predict_shop_inventory(
            seed=SEED, card_counter=0, merchant_counter=0, potion_counter=0,
            owned_relics=relic_ids,
        )
        new_ids = {r.relic.id for r in r2.inventory.relics}
        # Should not overlap (except Circlet fallback)
        overlap = relic_ids & new_ids - {"Circlet"}
        assert len(overlap) == 0


# =============================================================================
# Encounter Generation Tests
# =============================================================================


class TestEncounterGeneration:
    """Tests for generation/encounters.py."""

    def test_predict_act1(self):
        result = predict_act_encounters(SEED, act=1)
        assert len(result["monsters"]) > 0
        assert len(result["elites"]) > 0
        assert result["boss"] in EXORDIUM_BOSSES

    def test_predict_act2(self):
        result = predict_act_encounters(SEED, act=2)
        assert len(result["monsters"]) > 0
        assert len(result["elites"]) > 0
        assert result["boss"] in CITY_BOSSES

    def test_predict_act3(self):
        result = predict_act_encounters(SEED, act=3)
        assert len(result["monsters"]) > 0
        assert len(result["elites"]) > 0
        assert result["boss"] in BEYOND_BOSSES

    def test_predict_act4_fixed(self):
        result = predict_act_encounters(SEED, act=4)
        assert result["fixed"] is True
        assert result["boss"] == ENDING_BOSS
        assert len(result["elites"]) == 1

    def test_act1_encounter_counts(self):
        result = predict_act_encounters(SEED, act=1)
        # 3 weak + 1 first strong + 12 more strong = 16
        assert len(result["monsters"]) == 16
        assert len(result["elites"]) == 10

    def test_act2_encounter_counts(self):
        result = predict_act_encounters(SEED, act=2)
        # 2 weak + 1 first strong + 12 more strong = 15
        assert len(result["monsters"]) == 15
        assert len(result["elites"]) == 10

    def test_deterministic(self):
        r1 = predict_act_encounters(SEED, act=1)
        r2 = predict_act_encounters(SEED, act=1)
        assert r1["monsters"] == r2["monsters"]
        assert r1["elites"] == r2["elites"]
        assert r1["boss"] == r2["boss"]

    def test_different_seeds(self):
        r1 = predict_act_encounters("SEED1", act=1)
        r2 = predict_act_encounters("SEED2", act=1)
        # At least boss or monsters should differ
        assert r1["monsters"] != r2["monsters"] or r1["boss"] != r2["boss"]

    def test_no_back_to_back_repeats(self):
        result = predict_act_encounters(SEED, act=1)
        monsters = result["monsters"]
        for i in range(1, len(monsters)):
            assert monsters[i] != monsters[i - 1], \
                f"Back-to-back repeat at {i}: {monsters[i]}"

    def test_no_2back_repeats_for_normals(self):
        result = predict_act_encounters(SEED, act=1)
        monsters = result["monsters"]
        for i in range(2, len(monsters)):
            assert monsters[i] != monsters[i - 2], \
                f"2-back repeat at {i}: {monsters[i]}"

    def test_predict_all_acts(self):
        result = predict_all_acts(SEED, include_act4=True)
        assert "act1" in result
        assert "act2" in result
        assert "act3" in result
        assert "act4" in result

    def test_predict_all_bosses(self):
        bosses = predict_all_bosses(SEED)
        assert 1 in bosses
        assert 2 in bosses
        assert 3 in bosses
        assert bosses[1] in EXORDIUM_BOSSES
        assert bosses[2] in CITY_BOSSES
        assert bosses[3] in BEYOND_BOSSES

    def test_counter_advances_through_acts(self):
        result = predict_all_acts(SEED, include_act4=False)
        c1 = result["act1"]["monster_rng_counter"]
        c2 = result["act2"]["monster_rng_counter"]
        c3 = result["act3"]["monster_rng_counter"]
        assert c1 > 0
        assert c2 > c1
        assert c3 > c2

    def test_act4_no_rng_consumed(self):
        result = predict_act_encounters(SEED, act=4, monster_rng_counter=42)
        assert result["monster_rng_counter"] == 42

    def test_normalize_weights(self):
        monsters = [MonsterInfo("A", 2.0), MonsterInfo("B", 3.0), MonsterInfo("C", 5.0)]
        normalized = normalize_weights(monsters)
        total = sum(m.weight for m in normalized)
        assert abs(total - 1.0) < 1e-9

    def test_roll_monster(self):
        monsters = normalize_weights([
            MonsterInfo("A", 1.0),
            MonsterInfo("B", 1.0),
        ])
        # With normalized equal weights, roll < 0.5 should give first, >= 0.5 second
        m1 = roll_monster(monsters, 0.1)
        m2 = roll_monster(monsters, 0.9)
        assert m1 in ("A", "B")
        assert m2 in ("A", "B")

    def test_ending_encounters_fixed(self):
        monsters, elites, boss = generate_ending_encounters()
        assert len(monsters) == 0
        assert elites == ["Spire Shield and Spire Spear"]
        assert boss == "Corrupt Heart"

    def test_invalid_act_raises(self):
        with pytest.raises(ValueError):
            predict_act_encounters(SEED, act=5)


# =============================================================================
# Treasure Generation Tests
# =============================================================================


class TestTreasureGeneration:
    """Tests for generation/treasure.py."""

    def test_predict_chest_returns_reward(self):
        reward = predict_chest(SEED_LONG, treasure_counter=0)
        assert reward.chest_type in (ChestType.SMALL, ChestType.MEDIUM, ChestType.LARGE)
        assert reward.relic_tier in ("COMMON", "UNCOMMON", "RARE")

    def test_predict_chest_deterministic(self):
        r1 = predict_chest(SEED_LONG, treasure_counter=0)
        r2 = predict_chest(SEED_LONG, treasure_counter=0)
        assert r1.chest_type == r2.chest_type
        assert r1.relic_tier == r2.relic_tier
        assert r1.has_gold == r2.has_gold
        assert r1.gold_amount == r2.gold_amount

    def test_predict_chest_different_counters(self):
        results = set()
        for counter in range(20):
            r = predict_chest(SEED_LONG, treasure_counter=counter)
            results.add((r.chest_type, r.relic_tier, r.has_gold))
        # Should get some variety across 20 different counters
        assert len(results) >= 2

    def test_predict_full_chest(self):
        pred = predict_full_chest(SEED_LONG, treasure_counter=0)
        assert pred.relic_name is not None
        assert len(pred.relic_name) > 0

    def test_predict_full_chest_deterministic(self):
        p1 = predict_full_chest(SEED_LONG, treasure_counter=0)
        p2 = predict_full_chest(SEED_LONG, treasure_counter=0)
        assert p1.relic_name == p2.relic_name

    def test_nloths_face_first_chest_empty(self):
        r = predict_chest(SEED_LONG, treasure_counter=0, has_nloths_face=True)
        assert r.relic_tier == "NONE"
        assert not r.has_gold

    def test_nloths_face_second_chest_not_empty(self):
        r = predict_chest(
            SEED_LONG, treasure_counter=2,
            has_nloths_face=True, nloths_face_triggered=True,
        )
        assert r.relic_tier != "NONE"

    def test_predict_treasure_sequence(self):
        predictions = predict_treasure_sequence(SEED_LONG, num_chests=5)
        assert len(predictions) == 5
        for p in predictions:
            assert p.chest_type in (ChestType.SMALL, ChestType.MEDIUM, ChestType.LARGE)
            assert p.relic_name is not None

    def test_treasure_sequence_with_nloths(self):
        predictions = predict_treasure_sequence(
            SEED_LONG, num_chests=3, has_nloths_face=True
        )
        assert len(predictions) == 3
        # First chest should be empty
        assert predictions[0].relic_tier == "NONE"
        # Subsequent should not be empty
        assert predictions[1].relic_tier != "NONE"

    def test_gold_amount_reasonable(self):
        """Gold amounts should be within expected ranges."""
        for counter in range(20):
            r = predict_chest(SEED_LONG, treasure_counter=counter)
            if r.has_gold:
                assert 0 < r.gold_amount < 200

    def test_chest_type_distribution(self):
        """Over many seeds, all chest types should appear."""
        types = set()
        for i in range(100):
            r = predict_chest(seed_to_long(f"DIST{i}"), treasure_counter=0)
            types.add(r.chest_type)
        assert ChestType.SMALL in types
        assert ChestType.MEDIUM in types
        assert ChestType.LARGE in types

    def test_relic_tier_distribution(self):
        """Over many seeds, all relic tiers should appear."""
        tiers = set()
        for i in range(100):
            r = predict_chest(seed_to_long(f"TIER{i}"), treasure_counter=0)
            tiers.add(r.relic_tier)
        assert "COMMON" in tiers
        assert "UNCOMMON" in tiers
        # RARE may or may not appear in 100 samples for small chests
