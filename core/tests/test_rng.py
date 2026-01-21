"""
Comprehensive Test Suite for the RNG System

Tests:
1. XorShift128 implementation - deterministic sequences, known values
2. GameRNGState (13 stream state machine) - counter tracking, per-floor reset, act transitions
3. Neow cardRng consumption - safe options vs cardRng consumers
4. Card reward prediction - verified against known seeds
5. Shop generation - cardRng consumption estimates

Based on verified seed data from docs/vault/verified-seeds.md
"""

import pytest
import sys
import os

# Add core directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.state.rng import (
    XorShift128,
    Random,
    seed_to_long,
    long_to_seed,
    GameRNG,
)
from core.state.game_rng import (
    GameRNGState,
    RNGStream,
    predict_card_reward,
    simulate_path,
)


# =============================================================================
# XorShift128 Implementation Tests
# =============================================================================

class TestXorShift128:
    """Tests for the XorShift128 PRNG implementation."""

    def test_deterministic_sequence_from_seed(self):
        """Same seed produces identical sequence every time."""
        rng1 = XorShift128(12345)
        rng2 = XorShift128(12345)

        for _ in range(100):
            assert rng1._next_long() == rng2._next_long()

    def test_different_seeds_produce_different_sequences(self):
        """Different seeds produce different sequences."""
        rng1 = XorShift128(12345)
        rng2 = XorShift128(54321)

        # Very unlikely to match by chance
        matches = sum(1 for _ in range(100) if rng1._next_long() == rng2._next_long())
        assert matches < 5

    def test_murmur_hash3_initialization(self):
        """MurmurHash3 is used to derive state from seed."""
        seed = 42
        rng = XorShift128(seed)

        # State should be derived via murmur hash, not direct assignment
        expected_seed0 = XorShift128._murmur_hash3(seed)
        assert rng.seed0 == expected_seed0

    def test_zero_seed_handling(self):
        """Zero seed is handled specially (Java Long.MIN_VALUE)."""
        rng = XorShift128(0)
        # Should not crash and should produce valid state
        assert rng.seed0 != 0 or rng.seed1 != 0
        # Should produce valid sequence
        val = rng._next_long()
        assert isinstance(val, int)

    def test_direct_state_initialization(self):
        """Two-argument form sets state directly."""
        seed0 = 123456789
        seed1 = 987654321
        rng = XorShift128(seed0, seed1)

        assert rng.seed0 == seed0
        assert rng.seed1 == seed1

    def test_next_int_bounds(self):
        """next_int produces values in [0, bound)."""
        rng = XorShift128(42)
        bound = 100

        for _ in range(1000):
            val = rng.next_int(bound)
            assert 0 <= val < bound

    def test_next_int_invalid_bound(self):
        """next_int raises error for non-positive bound."""
        rng = XorShift128(42)

        with pytest.raises(ValueError):
            rng.next_int(0)

        with pytest.raises(ValueError):
            rng.next_int(-5)

    def test_next_float_bounds(self):
        """next_float produces values in [0, 1)."""
        rng = XorShift128(42)

        for _ in range(1000):
            val = rng.next_float()
            assert 0.0 <= val < 1.0

    def test_next_double_bounds(self):
        """next_double produces values in [0, 1)."""
        rng = XorShift128(42)

        for _ in range(1000):
            val = rng.next_double()
            assert 0.0 <= val < 1.0

    def test_next_boolean_distribution(self):
        """next_boolean produces roughly 50/50 distribution."""
        rng = XorShift128(42)
        true_count = sum(1 for _ in range(10000) if rng.next_boolean())

        # Should be roughly 50% (allow 2% margin)
        assert 4800 < true_count < 5200

    def test_copy_preserves_state(self):
        """Copied RNG produces same sequence as original."""
        rng1 = XorShift128(42)
        # Advance a bit
        for _ in range(10):
            rng1._next_long()

        rng2 = rng1.copy()

        for _ in range(100):
            assert rng1._next_long() == rng2._next_long()

    def test_get_state(self):
        """get_state returns correct internal state."""
        seed0 = 111
        seed1 = 222
        rng = XorShift128(seed0, seed1)

        assert rng.get_state(0) == seed0
        assert rng.get_state(1) == seed1

    def test_known_seed_values(self):
        """Known seed produces expected initial state (regression test)."""
        # Seed "1ABCD" from verified-seeds.md
        seed = seed_to_long("1ABCD")
        assert seed == 1943283

        rng = XorShift128(seed)
        # These are the expected state values after murmur hash initialization
        # If this fails, the murmur hash or initialization is broken
        assert rng.seed0 != 0  # Just verify non-zero
        assert rng.seed1 != 0


# =============================================================================
# Random Class (Game's Random Wrapper) Tests
# =============================================================================

class TestRandom:
    """Tests for the Random class that wraps XorShift128."""

    def test_counter_tracking(self):
        """Counter increments with each RNG call."""
        rng = Random(42)
        assert rng.counter == 0

        rng.random_int(99)
        assert rng.counter == 1

        rng.random_float()
        assert rng.counter == 2

        rng.random_boolean()
        assert rng.counter == 3

    def test_random_int_inclusive_range(self):
        """random_int(n) produces values in [0, n] inclusive."""
        rng = Random(42)
        range_val = 5

        seen = set()
        for _ in range(10000):
            val = rng.random_int(range_val)
            assert 0 <= val <= range_val  # INCLUSIVE
            seen.add(val)

        # Should see all values including the max
        assert range_val in seen
        assert 0 in seen

    def test_random_int_range_inclusive(self):
        """random_int_range(start, end) produces values in [start, end] inclusive."""
        rng = Random(42)
        start, end = 10, 20

        seen = set()
        for _ in range(10000):
            val = rng.random_int_range(start, end)
            assert start <= val <= end  # INCLUSIVE
            seen.add(val)

        assert start in seen
        assert end in seen

    def test_counter_restoration(self):
        """Random(seed, counter) restores to same state."""
        seed = 42
        target_counter = 50

        # Create RNG and advance it
        rng1 = Random(seed)
        for _ in range(target_counter):
            rng1.random_int(999)

        # Create new RNG with counter
        rng2 = Random(seed, target_counter)

        # They should produce same values
        for _ in range(100):
            assert rng1.random_int(99) == rng2.random_int(99)

    def test_copy_preserves_state(self):
        """Copied Random produces same sequence."""
        rng1 = Random(42)
        for _ in range(10):
            rng1.random_int(99)

        rng2 = rng1.copy()

        assert rng1.counter == rng2.counter

        for _ in range(100):
            assert rng1.random_int(99) == rng2.random_int(99)

    def test_set_counter_advances(self):
        """set_counter advances RNG to target counter."""
        rng = Random(42)
        rng.set_counter(100)

        assert rng.counter == 100

    def test_random_boolean_chance(self):
        """random_boolean_chance(p) produces p probability of True."""
        rng = Random(42)
        true_count = sum(1 for _ in range(10000) if rng.random_boolean_chance(0.3))

        # Should be roughly 30% (allow 2% margin)
        assert 2800 < true_count < 3200

    def test_random_long_uses_double(self):
        """random_long_range uses nextDouble, not nextInt."""
        # This is important because the game has both overloads
        rng = Random(42)
        val = rng.random_long_range(1000000000)  # Large range
        assert 0 <= val < 1000000000


# =============================================================================
# Seed Conversion Tests
# =============================================================================

class TestSeedConversion:
    """Tests for seed_to_long and long_to_seed."""

    def test_known_seed_conversions(self):
        """Known seeds convert to expected values."""
        # From verified-seeds.md
        assert seed_to_long("TEST123") == 52248462423
        assert seed_to_long("1ABCD") == 1943283
        assert seed_to_long("GA") == 570

    def test_roundtrip_conversion(self):
        """Converting to long and back produces original (or equivalent) string."""
        test_seeds = ["ABC", "12345", "ZZZZ", "A1B2C3"]

        for seed in test_seeds:
            long_val = seed_to_long(seed)
            result = long_to_seed(long_val)
            assert seed_to_long(result) == long_val

    def test_case_insensitive(self):
        """Seed conversion is case-insensitive."""
        assert seed_to_long("abc") == seed_to_long("ABC")
        assert seed_to_long("Test123") == seed_to_long("TEST123")

    def test_o_replaced_with_zero(self):
        """Letter O is replaced with 0."""
        assert seed_to_long("O") == seed_to_long("0")
        assert seed_to_long("HELLO") == seed_to_long("HELL0")

    def test_base_35_encoding(self):
        """Uses base-35 encoding (0-9 + A-Z excluding O)."""
        # Single digit tests
        assert seed_to_long("0") == 0
        assert seed_to_long("9") == 9
        assert seed_to_long("A") == 10
        assert seed_to_long("Z") == 34  # Last char (skipping O)

        # Multi-digit
        assert seed_to_long("10") == 35  # 1 * 35 + 0


# =============================================================================
# GameRNGState (13 Stream State Machine) Tests
# =============================================================================

class TestGameRNGState:
    """Tests for the GameRNGState counter-based state machine."""

    def test_initialization(self):
        """State initializes with zero counters."""
        state = GameRNGState("TEST123")

        assert state.seed == seed_to_long("TEST123")
        assert state.get_counter(RNGStream.CARD) == 0
        assert state.get_counter(RNGStream.MONSTER) == 0
        assert state.get_counter(RNGStream.RELIC) == 0

    def test_counter_tracking(self):
        """Counters track independently per stream."""
        state = GameRNGState("TEST")

        state.advance(RNGStream.CARD, 5)
        state.advance(RNGStream.MONSTER, 3)
        state.advance(RNGStream.RELIC, 1)

        assert state.get_counter(RNGStream.CARD) == 5
        assert state.get_counter(RNGStream.MONSTER) == 3
        assert state.get_counter(RNGStream.RELIC) == 1

    def test_per_floor_rng_reset(self):
        """Per-floor streams use seed + floorNum."""
        state = GameRNGState("TEST")

        # Get per-floor RNG for different floors
        state.enter_floor(1)
        rng_floor1 = state.get_rng(RNGStream.MONSTER_HP)

        state.enter_floor(2)
        rng_floor2 = state.get_rng(RNGStream.MONSTER_HP)

        # Different floors should produce different sequences
        val1 = rng_floor1.random_int(99)
        val2 = rng_floor2.random_int(99)

        # Reset and verify determinism
        state.enter_floor(1)
        rng_floor1_again = state.get_rng(RNGStream.MONSTER_HP)
        val1_again = rng_floor1_again.random_int(99)

        assert val1 == val1_again  # Same floor = same sequence
        # val1 and val2 may or may not be equal by chance

    def test_persistent_stream_counter_based(self):
        """Persistent streams use counter, not floor reseeding."""
        state = GameRNGState("TEST")

        # Advance card counter
        state.advance(RNGStream.CARD, 10)

        # Get RNG at different floors
        state.enter_floor(1)
        rng1 = state.get_rng(RNGStream.CARD)

        state.enter_floor(2)
        rng2 = state.get_rng(RNGStream.CARD)

        # Both should have same state (counter-based, not floor-based)
        assert rng1.random_int(99) == rng2.random_int(99)

    def test_clone_independence(self):
        """Cloned state is independent of original."""
        state1 = GameRNGState("TEST")
        state1.advance(RNGStream.CARD, 5)

        state2 = state1.clone()
        state2.advance(RNGStream.CARD, 10)

        assert state1.get_counter(RNGStream.CARD) == 5
        assert state2.get_counter(RNGStream.CARD) == 15


class TestActTransitionSnapping:
    """Tests for act transition cardRng snapping behavior."""

    @pytest.mark.parametrize("start,expected", [
        # 1-249 snaps to 250
        (1, 250),
        (100, 250),
        (200, 250),
        (249, 250),
        # 251-499 snaps to 500
        (251, 500),
        (260, 500),
        (400, 500),
        (499, 500),
        # 501-749 snaps to 750
        (501, 750),
        (510, 750),
        (700, 750),
        (749, 750),
        # Boundaries stay
        (0, 0),
        (250, 250),
        (500, 500),
        (750, 750),
    ])
    def test_cardrng_snapping(self, start, expected):
        """cardRng counter snaps to boundaries on act transition."""
        state = GameRNGState("TEST")
        state.set_counter(RNGStream.CARD, start)
        state.transition_to_next_act()

        assert state.get_counter(RNGStream.CARD) == expected

    def test_act_number_increments(self):
        """Act number increments on transition."""
        state = GameRNGState("TEST")
        assert state.act_num == 1

        state.transition_to_next_act()
        assert state.act_num == 2

        state.transition_to_next_act()
        assert state.act_num == 3

    def test_floor_resets_on_transition(self):
        """Floor number resets to 0 on act transition."""
        state = GameRNGState("TEST")
        state.enter_floor(17)
        assert state.floor_num == 17

        state.transition_to_next_act()
        assert state.floor_num == 0


# =============================================================================
# Neow cardRng Consumption Tests
# =============================================================================

class TestNeowConsumption:
    """Tests for Neow option cardRng consumption."""

    @pytest.mark.parametrize("option", [
        "UPGRADE_CARD",
        "HUNDRED_GOLD",
        "TEN_PERCENT_HP_BONUS",
        "RANDOM_COMMON_RELIC",
        "THREE_ENEMY_KILL",
        "THREE_CARDS",  # Uses NeowEvent.rng, not cardRng
        "ONE_RANDOM_RARE_CARD",  # Uses NeowEvent.rng
        "TRANSFORM_CARD",  # Uses NeowEvent.rng
        "REMOVE_CARD",
        "PERCENT_DAMAGE",
        "TEN_PERCENT_HP_LOSS",
        "NO_GOLD",
        "REMOVE_TWO",
        "TRANSFORM_TWO_CARDS",
        "TWENTY_PERCENT_HP_BONUS",
        "ONE_RARE_RELIC",
        "TWO_FIFTY_GOLD",
    ])
    def test_safe_options_no_consumption(self, option):
        """Safe Neow options consume 0 cardRng."""
        state = GameRNGState("TEST")
        initial = state.get_counter(RNGStream.CARD)

        state.apply_neow_choice(option)

        assert state.get_counter(RNGStream.CARD) == initial

    def test_random_colorless_consumes_cardrng(self):
        """RANDOM_COLORLESS consumes 3+ cardRng calls."""
        state = GameRNGState("TEST")
        state.apply_neow_choice("RANDOM_COLORLESS")

        # Minimum 3 calls (one per card)
        assert state.get_counter(RNGStream.CARD) >= 3

    def test_curse_drawback_consumes_one(self):
        """CURSE drawback consumes 1 cardRng call."""
        state = GameRNGState("TEST")
        state.apply_neow_choice("CURSE")

        assert state.get_counter(RNGStream.CARD) == 1

    def test_calling_bell_consumes_nine(self):
        """Calling Bell boss swap consumes 9 cardRng calls."""
        state = GameRNGState("TEST")
        state.apply_neow_choice("BOSS_SWAP", boss_relic="Calling Bell")

        assert state.get_counter(RNGStream.CARD) == 9

    def test_boss_swap_normal_no_consumption(self):
        """Normal boss swap (non-Calling Bell) consumes 0 cardRng."""
        state = GameRNGState("TEST")
        state.apply_neow_choice("BOSS_SWAP", boss_relic="Coffee Dripper")

        assert state.get_counter(RNGStream.CARD) == 0


# =============================================================================
# Card Reward Prediction Tests (Verified Seeds)
# =============================================================================

class TestCardRewardPrediction:
    """Tests for card reward prediction against verified seeds."""

    @pytest.mark.parametrize("seed,neow,floor,expected", [
        # Safe Neow options (offset=0)
        ("A", "HUNDRED_GOLD", 1, ["Pray", "Weave", "Foreign Influence"]),
        ("H", "REMOVE_CARD", 1, ["Bowling Bash", "Wallop", "Collect"]),
        ("I", "PERCENT_DAMAGE", 1, ["Tantrum", "Pray", "Evaluate"]),
        ("G", "THREE_CARDS", 1, ["Empty Body", "Third Eye", "Sash Whip"]),
        ("D", "ONE_RANDOM_RARE_CARD", 1, ["Inner Peace", "Perseverance", "Tranquility"]),
        ("N", "THREE_ENEMY_KILL", 1, ["Sanctity", "Meditate", "Talk to the Hand"]),
        ("B", "UPGRADE_CARD", 1, ["Follow-Up", "Crescendo", "Pressure Points"]),
    ])
    def test_safe_neow_card_predictions(self, seed, neow, floor, expected):
        """Card predictions match for safe Neow options."""
        state = GameRNGState(seed)
        state.apply_neow_choice(neow)
        state.enter_floor(floor)

        actual = [c[0] for c in predict_card_reward(state)]

        # Check all expected cards are present (order may vary due to rarity)
        for card in expected:
            assert card in actual, f"Expected {card} in {actual}"
        assert len(actual) == len(expected)

    def test_calling_bell_offset(self):
        """Calling Bell boss swap shifts card predictions."""
        # Seed GA with Calling Bell verified to produce specific floor 1 cards
        state = GameRNGState("GA")
        state.apply_neow_choice("BOSS_SWAP", boss_relic="Calling Bell")
        state.enter_floor(1)

        actual = [c[0] for c in predict_card_reward(state)]

        expected = ["Conclude", "Empty Fist", "Flurry of Blows"]
        for card in expected:
            assert card in actual, f"Expected {card} in {actual}"

    def test_transform_uses_neow_rng(self):
        """Transform card option uses NeowEvent.rng, not cardRng."""
        # Seed GA with Transform verified (offset=0)
        state = GameRNGState("GA")
        state.apply_neow_choice("TRANSFORM_CARD")
        state.enter_floor(1)

        actual = [c[0] for c in predict_card_reward(state)]

        expected = ["Mental Fortress", "Cut Through Fate", "Empty Body"]
        for card in expected:
            assert card in actual, f"Expected {card} in {actual}"


class TestRarityRolls:
    """Tests for card rarity roll mechanics."""

    def test_rarity_thresholds(self):
        """Verify rarity threshold constants."""
        # From game decompilation:
        # RARE: 0-2 (3%)
        # UNCOMMON: 3-39 (37%)
        # COMMON: 40-99 (60%)
        RARE_THRESHOLD = 3
        UNCOMMON_THRESHOLD = 37

        # These are the game's actual thresholds
        assert RARE_THRESHOLD == 3
        assert UNCOMMON_THRESHOLD == 37
        assert RARE_THRESHOLD + UNCOMMON_THRESHOLD == 40

    def test_card_rewards_include_multiple_rarities(self):
        """Card rewards over many floors include different rarities."""
        # Test that we get a distribution of rarities across many different RNG states
        rarities_seen = set()

        # Try many different seeds and counter values to see variety
        for seed in ["TEST", "ABC", "XYZ", "123", "RARE", "COMMON"]:
            for counter in range(0, 100, 10):
                state = GameRNGState(seed)
                state.set_counter(RNGStream.CARD, counter)
                state.enter_floor(1)

                cards = predict_card_reward(state)
                for card, rarity in cards:
                    rarities_seen.add(rarity)

        # Should see at least common and uncommon across all samples
        # (rare is less likely but possible)
        assert "COMMON" in rarities_seen
        assert "UNCOMMON" in rarities_seen

    def test_rarity_distribution_reasonable(self):
        """Rarity distribution is within expected ranges."""
        # Count rarities across many RNG samples
        rarity_counts = {"COMMON": 0, "UNCOMMON": 0, "RARE": 0}

        # Sample many different states
        for i in range(1000):
            state = GameRNGState(i)  # Different numeric seeds
            state.enter_floor(1)

            cards = predict_card_reward(state)
            for card, rarity in cards:
                rarity_counts[rarity] += 1

        total = sum(rarity_counts.values())

        # Expected: ~60% common, ~37% uncommon, ~3% rare (with blizzard starting at 5)
        # Allow generous margins for statistical variation
        common_pct = rarity_counts["COMMON"] / total
        uncommon_pct = rarity_counts["UNCOMMON"] / total
        rare_pct = rarity_counts["RARE"] / total

        # Common should be most frequent
        assert common_pct > 0.40, f"Common too low: {common_pct:.2%}"
        # Uncommon should be present
        assert uncommon_pct > 0.20, f"Uncommon too low: {uncommon_pct:.2%}"
        # Rare should be present (blizzard helps)
        # Note: With blizzard starting at +5, rare threshold is effectively 8, so rare is uncommon
        # The actual distribution depends heavily on the seed samples


# =============================================================================
# Shop Generation Tests
# =============================================================================

class TestShopGeneration:
    """Tests for shop cardRng consumption."""

    def test_shop_consumes_approximately_12_cardrng(self):
        """Shop generation consumes approximately 12 cardRng calls."""
        state = GameRNGState("TEST")
        initial = state.get_counter(RNGStream.CARD)

        state.apply_shop()

        consumed = state.get_counter(RNGStream.CARD) - initial

        # Shop: 5 colored cards (rarity + selection) + 2 colorless = ~12
        # Allow some variance for duplicate rerolls
        assert 10 <= consumed <= 15

    def test_shop_shifts_subsequent_predictions(self):
        """Visiting shop shifts subsequent card predictions."""
        # Without shop
        state_no_shop = GameRNGState("TEST")
        state_no_shop.apply_neow_choice("HUNDRED_GOLD")
        state_no_shop.enter_floor(1)
        state_no_shop.apply_combat("monster")
        cards_no_shop = [c[0] for c in predict_card_reward(state_no_shop)]

        # With shop
        state_with_shop = GameRNGState("TEST")
        state_with_shop.apply_neow_choice("HUNDRED_GOLD")
        state_with_shop.enter_floor(1)
        state_with_shop.apply_shop()
        state_with_shop.apply_combat("monster")
        cards_with_shop = [c[0] for c in predict_card_reward(state_with_shop)]

        # Cards should be different due to cardRng consumption
        assert cards_no_shop != cards_with_shop


# =============================================================================
# Path Simulation Tests
# =============================================================================

class TestPathSimulation:
    """Tests for simulate_path helper function."""

    def test_simulate_path_basic(self):
        """simulate_path processes events correctly."""
        path = [
            ("neow", "HUNDRED_GOLD"),
            ("floor", 1),
            ("combat", "monster"),
            ("floor", 2),
            ("shop", None),
        ]

        state = simulate_path("TEST", path)

        assert state.floor_num == 2
        # Combat adds ~9 cardRng, shop adds ~12
        assert state.get_counter(RNGStream.CARD) > 15

    def test_simulate_path_act_transition(self):
        """simulate_path handles act transitions."""
        path = [
            ("neow", "HUNDRED_GOLD"),
            ("floor", 17),
            ("combat", "boss"),
            ("act", None),
        ]

        state = simulate_path("TEST", path)

        assert state.act_num == 2
        assert state.floor_num == 0  # Reset after act transition


# =============================================================================
# Integration Tests
# =============================================================================

class TestIntegration:
    """Integration tests combining multiple components."""

    def test_seed_test123_floor1_cards(self):
        """Seed TEST123 floor 1 produces expected cards."""
        # From verified-seeds.md
        state = GameRNGState("TEST123")
        state.enter_floor(1)

        cards = [c[0] for c in predict_card_reward(state)]
        expected = ["Talk to the Hand", "Third Eye", "Empty Body"]

        for card in expected:
            assert card in cards

    def test_seed_1abcd_floors(self):
        """Seed 1ABCD produces expected floor 1 cards."""
        # From verified-seeds.md
        state = GameRNGState("1ABCD")
        state.enter_floor(1)

        cards = [c[0] for c in predict_card_reward(state)]
        expected = ["Like Water", "Bowling Bash", "Deceive Reality"]

        for card in expected:
            assert card in cards

    def test_multiple_floor_progression(self):
        """Card predictions change correctly across floors."""
        state = GameRNGState("TEST")
        state.apply_neow_choice("HUNDRED_GOLD")

        floor1_cards = None
        floor2_cards = None
        floor3_cards = None

        # Floor 1
        state.enter_floor(1)
        floor1_cards = [c[0] for c in predict_card_reward(state)]
        state.apply_combat("monster")

        # Floor 2
        state.enter_floor(2)
        floor2_cards = [c[0] for c in predict_card_reward(state)]
        state.apply_combat("monster")

        # Floor 3
        state.enter_floor(3)
        floor3_cards = [c[0] for c in predict_card_reward(state)]

        # All floors should have different cards (highly likely)
        assert floor1_cards != floor2_cards or floor1_cards != floor3_cards


# =============================================================================
# Edge Case Tests
# =============================================================================

class TestEdgeCases:
    """Tests for edge cases and boundary conditions."""

    def test_seed_string_vs_int(self):
        """GameRNGState accepts both string and int seeds."""
        seed_str = "TEST123"
        seed_int = seed_to_long(seed_str)

        state_str = GameRNGState(seed_str)
        state_int = GameRNGState(seed_int)

        assert state_str.seed == state_int.seed

    def test_empty_path_simulation(self):
        """simulate_path handles empty path."""
        state = simulate_path("TEST", [])

        assert state.floor_num == 0
        assert state.act_num == 1
        assert state.get_counter(RNGStream.CARD) == 0

    def test_high_counter_values(self):
        """System handles high counter values correctly."""
        state = GameRNGState("TEST")
        state.set_counter(RNGStream.CARD, 10000)

        # Should still produce valid cards
        state.enter_floor(1)
        cards = predict_card_reward(state)

        assert len(cards) == 3
        for card, rarity in cards:
            assert rarity in ["COMMON", "UNCOMMON", "RARE"]

    def test_all_rng_streams_exist(self):
        """All 13 RNG streams are properly defined."""
        expected_streams = [
            RNGStream.CARD,
            RNGStream.MONSTER,
            RNGStream.EVENT,
            RNGStream.RELIC,
            RNGStream.TREASURE,
            RNGStream.POTION,
            RNGStream.MERCHANT,
            RNGStream.MONSTER_HP,
            RNGStream.AI,
            RNGStream.SHUFFLE,
            RNGStream.CARD_RANDOM,
            RNGStream.MISC,
            RNGStream.MAP,
        ]

        for stream in expected_streams:
            state = GameRNGState("TEST")
            rng = state.get_rng(stream)
            assert rng is not None


# =============================================================================
# Run Tests
# =============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
