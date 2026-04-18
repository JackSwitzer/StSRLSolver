"""
Tests for Potion Drop Prediction System

Tests verify:
1. Drop chance calculations with blizzard modifier
2. Rarity distribution (65% common, 25% uncommon, 10% rare)
3. Selection loop behavior
4. White Beast Statue 100% drop
5. Sozu prevents drops
6. Rewards cap at 4
7. RNG counter consumption
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.generation.potions import (
    # Prediction functions
    predict_potion_drop,
    predict_potion_from_seed,
    predict_multiple_potion_drops,
    PotionPrediction,
    # Pool utilities
    get_potion_pool_for_class,
    get_potion_by_id,
    # Constants
    WATCHER_POTION_POOL,
    IRONCLAD_POTION_POOL,
    SILENT_POTION_POOL,
    DEFECT_POTION_POOL,
    BASE_DROP_CHANCE,
    BLIZZARD_MOD_STEP,
    POTION_COMMON_CHANCE,
    POTION_UNCOMMON_CHANCE,
)
from packages.engine.state.rng import Random, seed_to_long
# Import PotionRarity from the generation module to ensure enum identity matches
from packages.engine.generation.potions import PotionRarity, ALL_POTIONS


# ============================================================================
# SECTION 1: POTION POOL TESTS
# ============================================================================

class TestPotionPool:
    """Test potion pool structure and ordering."""

    def test_watcher_pool_size(self):
        """Watcher pool has 33 potions (3 class + 30 universal)."""
        assert len(WATCHER_POTION_POOL) == 33

    def test_ironclad_pool_size(self):
        """Ironclad pool has 33 potions."""
        assert len(IRONCLAD_POTION_POOL) == 33

    def test_silent_pool_size(self):
        """Silent pool has 33 potions."""
        assert len(SILENT_POTION_POOL) == 33

    def test_defect_pool_size(self):
        """Defect pool has 33 potions."""
        assert len(DEFECT_POTION_POOL) == 33

    def test_watcher_class_potions_first(self):
        """Watcher class-specific potions come first in pool."""
        pool = WATCHER_POTION_POOL
        assert pool[0] == "BottledMiracle"
        assert pool[1] == "StancePotion"
        assert pool[2] == "Ambrosia"

    def test_ironclad_class_potions_first(self):
        """Ironclad class-specific potions come first."""
        pool = IRONCLAD_POTION_POOL
        assert pool[0] == "BloodPotion"
        assert pool[1] == "ElixirPotion"
        assert pool[2] == "HeartOfIron"

    def test_silent_class_potions_first(self):
        """Silent class-specific potions come first."""
        pool = SILENT_POTION_POOL
        assert pool[0] == "Poison Potion"
        assert pool[1] == "CunningPotion"
        assert pool[2] == "GhostInAJar"

    def test_defect_class_potions_first(self):
        """Defect class-specific potions come first."""
        pool = DEFECT_POTION_POOL
        assert pool[0] == "FocusPotion"
        assert pool[1] == "PotionOfCapacity"
        assert pool[2] == "EssenceOfDarkness"

    def test_universal_potions_same_order(self):
        """Universal potions follow same order in all pools."""
        # Skip first 3 (class-specific), compare rest
        watcher_universal = WATCHER_POTION_POOL[3:]
        ironclad_universal = IRONCLAD_POTION_POOL[3:]
        silent_universal = SILENT_POTION_POOL[3:]
        defect_universal = DEFECT_POTION_POOL[3:]

        assert watcher_universal == ironclad_universal
        assert watcher_universal == silent_universal
        assert watcher_universal == defect_universal

    def test_all_pool_potions_exist(self):
        """All potions in pool exist in ALL_POTIONS."""
        for potion_id in WATCHER_POTION_POOL:
            assert potion_id in ALL_POTIONS, f"{potion_id} not in ALL_POTIONS"


# ============================================================================
# SECTION 2: DROP CHANCE TESTS
# ============================================================================

class TestDropChance:
    """Test drop chance calculations."""

    def test_base_drop_chance(self):
        """Base drop chance is 40."""
        assert BASE_DROP_CHANCE == 40

    def test_blizzard_mod_step(self):
        """Blizzard mod changes by 10 per drop/miss."""
        assert BLIZZARD_MOD_STEP == 10

    def test_drop_increases_blizzard_negative(self):
        """Successful drop decreases blizzard mod by 10."""
        seed_long = seed_to_long("DROPTEST1")
        rng = Random(seed_long)

        # Force a drop with high chance
        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=60,  # 100% chance (40+60)
            player_class="WATCHER",
        )

        if pred.will_drop:
            assert pred.new_blizzard_mod == 50  # 60 - 10

    def test_no_drop_increases_blizzard(self):
        """Failed drop increases blizzard mod by 10."""
        # Find a seed where we don't drop
        seed_long = seed_to_long("NODROP1")
        rng = Random(seed_long)

        # Use negative blizzard to make drop unlikely
        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=-30,  # 10% chance (40-30)
            player_class="WATCHER",
        )

        # Run enough times to hit a no-drop
        if not pred.will_drop:
            assert pred.new_blizzard_mod == -20  # -30 + 10


# ============================================================================
# SECTION 3: RELIC EFFECTS
# ============================================================================

class TestRelicEffects:
    """Test relic interactions with potion drops."""

    def test_white_beast_statue_guarantees_drop(self):
        """White Beast Statue gives 100% drop chance."""
        seed_long = seed_to_long("WHITEBEAST")
        rng = Random(seed_long)

        # Even with negative blizzard, should drop
        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=-100,  # Would be 0% normally
            player_class="WATCHER",
            has_white_beast_statue=True,
        )

        assert pred.will_drop == True
        assert pred.potion_id is not None

    def test_sozu_prevents_drops(self):
        """Sozu prevents all potion drops."""
        seed_long = seed_to_long("SOZUTEST")
        rng = Random(seed_long)
        initial_counter = rng.counter

        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=60,  # Would be 100% normally
            player_class="WATCHER",
            has_sozu=True,
        )

        assert pred.will_drop == False
        assert pred.potion_id is None
        # Sozu should not consume any RNG
        assert rng.counter == initial_counter

    def test_sozu_doesnt_change_blizzard(self):
        """Sozu doesn't affect blizzard modifier."""
        seed_long = seed_to_long("SOZUTEST2")
        rng = Random(seed_long)

        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=20,
            player_class="WATCHER",
            has_sozu=True,
        )

        assert pred.new_blizzard_mod == 20  # Unchanged


# ============================================================================
# SECTION 4: REWARDS CAP
# ============================================================================

class TestRewardsCap:
    """Test 4+ rewards preventing potion drops."""

    def test_four_rewards_no_drop(self):
        """4 existing rewards prevents potion drop."""
        seed_long = seed_to_long("REWARDCAP")
        rng = Random(seed_long)

        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=60,  # Would be 100% normally
            player_class="WATCHER",
            current_rewards=4,
        )

        assert pred.will_drop == False

    def test_three_rewards_can_drop(self):
        """3 existing rewards still allows potion drop."""
        seed_long = seed_to_long("REWARDOK")
        rng = Random(seed_long)

        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=60,  # 100% chance
            player_class="WATCHER",
            current_rewards=3,
        )

        # With 100% chance, should drop
        assert pred.will_drop == True


# ============================================================================
# SECTION 5: RARITY DISTRIBUTION
# ============================================================================

class TestRarityDistribution:
    """Test potion rarity distribution."""

    def test_rarity_chances(self):
        """Verify rarity chance constants."""
        assert POTION_COMMON_CHANCE == 65
        assert POTION_UNCOMMON_CHANCE == 25
        # Rare is 100 - 65 - 25 = 10

    def test_rarity_distribution_over_many_drops(self):
        """Test rarity distribution over many samples."""
        seed_long = seed_to_long("RARITYTEST")
        rng = Random(seed_long)

        common_count = 0
        uncommon_count = 0
        rare_count = 0
        total_drops = 0

        # Simulate many drops with White Beast Statue (100% drop)
        for _ in range(200):
            pred = predict_potion_drop(
                potion_rng=rng,
                blizzard_mod=0,
                player_class="WATCHER",
                has_white_beast_statue=True,
            )

            if pred.will_drop:
                total_drops += 1
                if pred.rarity == PotionRarity.COMMON:
                    common_count += 1
                elif pred.rarity == PotionRarity.UNCOMMON:
                    uncommon_count += 1
                elif pred.rarity == PotionRarity.RARE:
                    rare_count += 1

        # With 200 drops, we should have roughly:
        # Common: ~65% -> ~130
        # Uncommon: ~25% -> ~50
        # Rare: ~10% -> ~20

        # Allow reasonable variance
        assert common_count > 80, f"Too few common: {common_count}"
        assert common_count < 180, f"Too many common: {common_count}"
        assert uncommon_count > 20, f"Too few uncommon: {uncommon_count}"
        assert uncommon_count < 80, f"Too many uncommon: {uncommon_count}"
        assert rare_count > 5, f"Too few rare: {rare_count}"
        assert rare_count < 40, f"Too many rare: {rare_count}"


# ============================================================================
# SECTION 6: SEED DETERMINISM
# ============================================================================

class TestSeedDeterminism:
    """Test that predictions are deterministic from seed."""

    def test_same_seed_same_result(self):
        """Same seed produces same potion."""
        seed = "DETERMINISM"

        pred1 = predict_potion_from_seed(
            seed=seed,
            potion_counter=0,
            blizzard_mod=60,  # High chance to drop
            player_class="WATCHER",
        )

        pred2 = predict_potion_from_seed(
            seed=seed,
            potion_counter=0,
            blizzard_mod=60,
            player_class="WATCHER",
        )

        assert pred1.will_drop == pred2.will_drop
        assert pred1.potion_id == pred2.potion_id
        assert pred1.rarity == pred2.rarity

    def test_counter_affects_result(self):
        """Different counter can produce different results."""
        seed = "COUNTERTEST"

        results = set()
        for counter in range(10):
            pred = predict_potion_from_seed(
                seed=seed,
                potion_counter=counter,
                blizzard_mod=60,
                player_class="WATCHER",
                has_white_beast_statue=True,  # Force drops
            )
            if pred.will_drop:
                results.add(pred.potion_id)

        # Should get at least a few different potions
        assert len(results) >= 2


# ============================================================================
# SECTION 7: RNG CONSUMPTION
# ============================================================================

class TestRngConsumption:
    """Test RNG counter consumption patterns."""

    def test_drop_roll_always_consumes_rng(self):
        """Drop roll always consumes 1 RNG call."""
        seed_long = seed_to_long("RNGCOUNT1")
        rng = Random(seed_long)
        initial = rng.counter

        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=-40,  # 0% chance - no drop
            player_class="WATCHER",
        )

        # Should consume exactly 1 call (the drop roll)
        assert rng.counter == initial + 1
        assert pred.will_drop == False

    def test_drop_consumes_at_least_3_rng(self):
        """Successful drop consumes at least 3 RNG calls."""
        seed_long = seed_to_long("RNGCOUNT2")
        rng = Random(seed_long)

        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=60,  # 100% chance
            player_class="WATCHER",
        )

        if pred.will_drop:
            # Minimum: 1 drop roll + 1 rarity roll + 1 selection
            assert rng.counter >= 3
            assert pred.selection_attempts >= 1


# ============================================================================
# SECTION 8: MULTIPLE DROPS
# ============================================================================

class TestMultipleDrops:
    """Test predicting multiple consecutive drops."""

    def test_predict_multiple_drops(self):
        """predict_multiple_potion_drops returns correct count."""
        seed_long = seed_to_long("MULTIPLEDROP")
        rng = Random(seed_long)

        predictions = predict_multiple_potion_drops(
            potion_rng=rng,
            num_combats=5,
            blizzard_mod=0,
            player_class="WATCHER",
        )

        assert len(predictions) == 5

    def test_blizzard_accumulates_correctly(self):
        """Blizzard modifier accumulates across combats."""
        seed_long = seed_to_long("BLIZZARDACC")
        rng = Random(seed_long)

        predictions = predict_multiple_potion_drops(
            potion_rng=rng,
            num_combats=10,
            blizzard_mod=0,
            player_class="WATCHER",
        )

        # Verify blizzard chain is consistent
        current_blizzard = 0
        for pred in predictions:
            if pred.will_drop:
                assert pred.new_blizzard_mod == current_blizzard - 10
            else:
                assert pred.new_blizzard_mod == current_blizzard + 10
            current_blizzard = pred.new_blizzard_mod


# ============================================================================
# SECTION 9: CLASS-SPECIFIC POTIONS
# ============================================================================

class TestClassSpecificPotions:
    """Test that class-specific potions appear for correct classes."""

    def test_watcher_can_get_watcher_potions(self):
        """Watcher can get BottledMiracle, StancePotion, Ambrosia."""
        seed_long = seed_to_long("WATCHERCLASS")
        rng = Random(seed_long)

        potions_seen = set()
        for _ in range(100):
            pred = predict_potion_drop(
                potion_rng=rng,
                blizzard_mod=0,
                player_class="WATCHER",
                has_white_beast_statue=True,
            )
            if pred.will_drop:
                potions_seen.add(pred.potion_id)

        # Should eventually see at least one Watcher potion
        watcher_potions = {"BottledMiracle", "StancePotion", "Ambrosia"}
        assert len(potions_seen & watcher_potions) > 0

    def test_ironclad_gets_ironclad_potions(self):
        """Ironclad can get BloodPotion, ElixirPotion, HeartOfIron."""
        seed_long = seed_to_long("IRONCLADCLASS")
        rng = Random(seed_long)

        potions_seen = set()
        for _ in range(100):
            pred = predict_potion_drop(
                potion_rng=rng,
                blizzard_mod=0,
                player_class="IRONCLAD",
                has_white_beast_statue=True,
            )
            if pred.will_drop:
                potions_seen.add(pred.potion_id)

        # Should see at least one Ironclad potion
        ironclad_potions = {"BloodPotion", "ElixirPotion", "HeartOfIron"}
        assert len(potions_seen & ironclad_potions) > 0


# ============================================================================
# SECTION 10: EDGE CASES
# ============================================================================

class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_invalid_room_type_no_drop(self):
        """Invalid room type results in no drop."""
        seed_long = seed_to_long("INVALIDROOM")
        rng = Random(seed_long)

        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=60,
            player_class="WATCHER",
            room_type="boss",  # Invalid for potion drops
        )

        assert pred.will_drop == False

    def test_negative_blizzard_clamps_to_zero(self):
        """Heavily negative blizzard doesn't go below 0% chance."""
        seed_long = seed_to_long("NEGBLIZZARD")
        rng = Random(seed_long)

        pred = predict_potion_drop(
            potion_rng=rng,
            blizzard_mod=-100,  # Would be -60% without clamping
            player_class="WATCHER",
        )

        # With 0% chance, should never drop
        assert pred.will_drop == False

    def test_get_potion_by_id_valid(self):
        """get_potion_by_id returns correct potion."""
        potion = get_potion_by_id("Fire Potion")
        assert potion is not None
        assert potion.name == "Fire Potion"

    def test_get_potion_by_id_invalid(self):
        """get_potion_by_id returns None for invalid ID."""
        potion = get_potion_by_id("Invalid Potion Name")
        assert potion is None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
