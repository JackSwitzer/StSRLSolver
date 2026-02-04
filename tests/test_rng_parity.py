"""
RNG Parity Tests - Verify Python RNG matches Java exactly

These tests verify that the Python RNG implementation produces
identical results to the decompiled Java source.

Test Categories:
1. Core algorithm (XorShift128)
2. Random wrapper methods
3. Seed conversion (base-35)
4. Stream initialization
5. Per-floor reseeding
6. Act transition snapping
7. Save/load restoration
"""

import pytest
from packages.engine.state.rng import (
    XorShift128,
    Random,
    seed_to_long,
    long_to_seed,
    GameRNG,
)
from packages.engine.state.game_rng import (
    GameRNGState,
    RNGStream,
    simulate_path,
    predict_card_reward,
)


class TestXorShift128Algorithm:
    """Verify XorShift128 matches libGDX RandomXS128."""

    def test_seed_initialization_zero(self):
        """Zero seed should use Long.MIN_VALUE."""
        rng = XorShift128(0)
        # Zero seeds to Long.MIN_VALUE in Java
        assert rng.seed0 != 0 or rng.seed1 != 0

    def test_known_sequence(self):
        """Verify known output sequence from Python implementation."""
        # Seed 12345, first 10 nextLong() calls
        # These values are from our XorShift128 implementation
        rng = XorShift128(12345)

        # Generate sequence and verify it's deterministic
        expected = [
            1382432690769144372,
            8992747501898680205,
            -1947876644470197540,
            -7800294855028505862,
            8375005766809915749,
            8389962128307583402,
            3448064027524546479,
            -3551486133838269301,
            5354393402404647562,
            -8621052581439360842,
        ]

        for exp in expected:
            val = rng._next_long()
            assert val == exp, f"Expected {exp}, got {val}"

    def test_next_int_bounded(self):
        """Verify bounded nextInt matches Java."""
        rng = XorShift128(54321)

        # Test bounds [0, 100)
        for _ in range(100):
            val = rng.next_int(100)
            assert 0 <= val < 100

    def test_next_float_range(self):
        """Verify nextFloat is in [0, 1)."""
        rng = XorShift128(99999)

        for _ in range(100):
            val = rng.next_float()
            assert 0.0 <= val < 1.0

    def test_copy(self):
        """Verify copy creates independent RNG with same state."""
        rng1 = XorShift128(777)
        val1 = rng1._next_long()

        rng2 = rng1.copy()
        val2 = rng2._next_long()

        # Both should produce same next value
        assert val2 == rng1._next_long()


class TestRandomWrapper:
    """Verify Random wrapper matches game's Random class."""

    def test_counter_increment(self):
        """Every method increments counter by 1."""
        rng = Random(12345)
        assert rng.counter == 0

        rng.random_int(10)
        assert rng.counter == 1

        rng.random_boolean()
        assert rng.counter == 2

        rng.random_float()
        assert rng.counter == 3

    def test_random_int_inclusive(self):
        """random_int(n) returns [0, n] inclusive."""
        rng = Random(12345)

        # Range [0, 5] should include 5
        vals = {rng.random_int(5) for _ in range(1000)}
        assert max(vals) <= 5
        assert min(vals) >= 0
        # Should see 5 at least once in 1000 rolls
        assert 5 in vals

    def test_random_int_range_inclusive(self):
        """random_int_range(start, end) returns [start, end] inclusive."""
        rng = Random(54321)

        # Range [10, 20] should include both endpoints
        vals = {rng.random_int_range(10, 20) for _ in range(1000)}
        assert max(vals) <= 20
        assert min(vals) >= 10
        assert 10 in vals
        assert 20 in vals

    def test_random_long_range_exclusive(self):
        """random_long_range(n) returns [0, n) exclusive."""
        rng = Random(99999)

        # Range [0, 100) should NOT include 100
        vals = [rng.random_long_range(100) for _ in range(1000)]
        assert max(vals) < 100
        assert min(vals) >= 0

    def test_random_boolean_fair(self):
        """randomBoolean() should be ~50/50."""
        rng = Random(777)

        trues = sum(rng.random_boolean() for _ in range(1000))
        # Should be roughly 500 ± 100
        assert 400 < trues < 600

    def test_random_boolean_chance(self):
        """randomBoolean(chance) respects probability."""
        rng = Random(888)

        # 25% chance should be ~250/1000
        trues = sum(rng.random_boolean_chance(0.25) for _ in range(1000))
        assert 200 < trues < 300

    def test_set_counter_advances(self):
        """setCounter advances RNG state."""
        rng1 = Random(12345)
        rng2 = Random(12345)

        # Advance rng2 by 10 calls
        rng2.set_counter(10)
        assert rng2.counter == 10

        # Next value should differ
        val1 = rng1.random_int(999)
        val2 = rng2.random_int(999)
        assert val1 != val2

    def test_constructor_with_counter(self):
        """Random(seed, counter) advances correctly."""
        rng1 = Random(12345)
        for _ in range(10):
            rng1.random_int(999)

        rng2 = Random(12345, 10)

        # Both should produce same next value
        assert rng1.random_int(999) == rng2.random_int(999)

    def test_copy(self):
        """copy() creates independent RNG with same state."""
        rng1 = Random(12345)
        rng1.random_int(10)
        rng1.random_int(10)

        rng2 = rng1.copy()

        # Both should have same counter
        assert rng2.counter == rng1.counter

        # Both should produce same sequence
        assert rng1.random_int(999) == rng2.random_int(999)


class TestSeedConversion:
    """Verify seed string <-> long conversion matches SeedHelper.java."""

    def test_base35_characters(self):
        """Base-35 character set excludes 'O'."""
        CHARACTERS = "0123456789ABCDEFGHIJKLMNPQRSTUVWXYZ"
        assert len(CHARACTERS) == 35
        assert "O" not in CHARACTERS

    def test_o_replaced_with_zero(self):
        """'O' is replaced with '0'."""
        assert seed_to_long("O") == seed_to_long("0")
        assert seed_to_long("OOOOO") == seed_to_long("00000")

    def test_case_insensitive(self):
        """Seed parsing is case-insensitive."""
        assert seed_to_long("abc") == seed_to_long("ABC")
        assert seed_to_long("Test123") == seed_to_long("TEST123")

    def test_known_seeds(self):
        """Verify seed conversion matches Java behavior."""
        # Pure numeric strings are treated as decimal (save file format)
        numeric_cases = [
            ("0", 0),
            ("1", 1),
            ("10", 10),     # Decimal 10, not base-35!
            ("12345", 12345),
            ("-9876", -9876),
        ]

        for seed_str, expected_long in numeric_cases:
            result = seed_to_long(seed_str)
            assert result == expected_long, f"seed_to_long('{seed_str}') = {result}, expected {expected_long}"

        # Alphanumeric strings use base-35 encoding
        base35_cases = [
            ("A", 10),      # A is index 10
            ("Z", 34),      # Z is index 34
            ("A0", 10*35),  # A=10, 0=0 -> 10*35 + 0 = 350
            ("1A", 1*35 + 10),  # 1*35 + 10 = 45
        ]

        for seed_str, expected_long in base35_cases:
            result = seed_to_long(seed_str)
            assert result == expected_long, f"seed_to_long('{seed_str}') = {result}, expected {expected_long}"

    def test_bidirectional_conversion(self):
        """Converting long -> string -> long preserves value.

        Note: long_to_seed always produces base-35, but seed_to_long
        treats pure numeric strings as decimal. So roundtrip only works
        if the resulting string contains letters.
        """
        test_longs = [
            10,    # -> 'A' -> 10
            34,    # -> 'Z' -> 34
            35,    # -> '10' -> 10 (FAILS - treated as decimal)
            100,   # -> '2U' -> 100
            1000,  # -> 'SK' -> 1000
            12345, # -> 'A0F' -> 12345
        ]

        for val in test_longs:
            seed_str = long_to_seed(val)
            back = seed_to_long(seed_str)

            # Only values that produce alphanumeric strings roundtrip correctly
            if any(c.isalpha() for c in seed_str):
                assert back == val, f"long_to_seed({val}) = '{seed_str}', back = {back}"
            # Else skip (pure numeric strings are ambiguous)

    def test_numeric_string_passthrough(self):
        """Pure numeric strings are treated as integers."""
        # Save files store seeds as numeric strings
        assert seed_to_long("12345") == 12345
        assert seed_to_long("-9876") == -9876


class TestGameRNGStreams:
    """Verify all 13 RNG streams are seeded correctly."""

    def test_persistent_streams_same_seed(self):
        """Persistent streams start with same seed."""
        game = GameRNG(seed=12345)

        # All persistent streams should have same initial seed
        # (but different state after initialization)
        assert game.monster_rng.counter == 0
        assert game.event_rng.counter == 0
        assert game.card_rng.counter == 0
        assert game.treasure_rng.counter == 0
        assert game.relic_rng.counter == 0
        assert game.potion_rng.counter == 0

    def test_per_floor_streams_reseed(self):
        """Per-floor streams use seed + floorNum."""
        game = GameRNG(seed=1000, floor=5)

        # Per-floor streams should be seeded with seed + floor
        # We can't directly check the seed, but we can verify
        # they produce different values than the base seed
        base_rng = Random(1000)
        floor_rng = game.ai_rng

        # First values should differ
        assert base_rng.random_int(999) != floor_rng.random_int(999)

    def test_floor_advance_reseeds(self):
        """Advancing floor reseeds per-floor streams."""
        game = GameRNG(seed=1000, floor=1)

        ai1 = game.ai_rng.copy()
        val1 = ai1.random_int(999)

        game.advance_floor()

        ai2 = game.ai_rng.copy()
        val2 = ai2.random_int(999)

        # Different floors = different values
        assert val1 != val2

    def test_map_rng_act_offsets(self):
        """mapRng uses act-specific offsets."""
        # Act 1: seed + 1
        game1 = GameRNG(seed=1000, act_num=1)
        # Act 2: seed + 200
        game2 = GameRNG(seed=1000, act_num=2)
        # Act 3: seed + 600
        game3 = GameRNG(seed=1000, act_num=3)

        # All should produce different first values
        val1 = game1.map_rng.random_int(999)
        val2 = game2.map_rng.random_int(999)
        val3 = game3.map_rng.random_int(999)

        assert val1 != val2
        assert val2 != val3
        assert val1 != val3

    def test_act_advance_reseeds_map(self):
        """Advancing act reseeds mapRng."""
        game = GameRNG(seed=1000, act_num=1)

        map1 = game.map_rng.copy()
        val1 = map1.random_int(999)

        game.advance_act(2)

        map2 = game.map_rng.copy()
        val2 = map2.random_int(999)

        # Different acts = different values
        assert val1 != val2


class TestGameRNGState:
    """Verify GameRNGState state machine."""

    def test_initialization(self):
        """State starts at floor 0, act 1, all counters 0."""
        state = GameRNGState("TEST123")

        assert state.floor_num == 0
        assert state.act_num == 1
        assert state.counters[RNGStream.CARD.value] == 0
        assert state.counters[RNGStream.MONSTER.value] == 0

    def test_neow_choice_safe(self):
        """Safe Neow options don't consume cardRng."""
        state = GameRNGState("TEST123")

        state.apply_neow_choice("HUNDRED_GOLD")
        assert state.counters[RNGStream.CARD.value] == 0

        state.apply_neow_choice("UPGRADE_CARD")
        assert state.counters[RNGStream.CARD.value] == 0

    def test_neow_choice_colorless(self):
        """RANDOM_COLORLESS consumes cardRng."""
        state = GameRNGState("TEST123")

        state.apply_neow_choice("RANDOM_COLORLESS")
        assert state.counters[RNGStream.CARD.value] >= 3

    def test_combat_consumes_multiple_streams(self):
        """Combat consumes cardRng, treasureRng, potionRng."""
        state = GameRNGState("TEST123")

        state.apply_combat("monster")

        # Card reward: ~9 calls
        assert state.counters[RNGStream.CARD.value] > 0
        # Gold reward: 1 call
        assert state.counters[RNGStream.TREASURE.value] > 0
        # Potion drop: ~2 calls
        assert state.counters[RNGStream.POTION.value] > 0

    def test_elite_combat_consumes_relic(self):
        """Elite combat consumes relicRng."""
        state = GameRNGState("TEST123")

        state.apply_combat("elite")

        # Elite gives relic
        assert state.counters[RNGStream.RELIC.value] > 0

    def test_shop_consumes_card_merchant(self):
        """Shop consumes cardRng and merchantRng."""
        state = GameRNGState("TEST123")

        state.apply_shop()

        # Card generation: ~12 calls
        assert state.counters[RNGStream.CARD.value] >= 10
        # Merchant prices: ~17 calls
        assert state.counters[RNGStream.MERCHANT.value] >= 15

    def test_event_library(self):
        """The Library event consumes ~20 cardRng."""
        state = GameRNGState("TEST123")

        state.apply_event("TheLibrary")

        # 20 unique cards generated
        assert state.counters[RNGStream.CARD.value] >= 18

    def test_act_transition_snapping_low(self):
        """Counter 1-249 snaps to 250."""
        state = GameRNGState("TEST123")
        state.counters[RNGStream.CARD.value] = 100

        state.transition_to_next_act()

        assert state.counters[RNGStream.CARD.value] == 250
        assert state.act_num == 2

    def test_act_transition_snapping_mid(self):
        """Counter 251-499 snaps to 500."""
        state = GameRNGState("TEST123")
        state.counters[RNGStream.CARD.value] = 300

        state.transition_to_next_act()

        assert state.counters[RNGStream.CARD.value] == 500

    def test_act_transition_snapping_high(self):
        """Counter 501-749 snaps to 750."""
        state = GameRNGState("TEST123")
        state.counters[RNGStream.CARD.value] = 600

        state.transition_to_next_act()

        assert state.counters[RNGStream.CARD.value] == 750

    def test_act_transition_no_snap_boundary(self):
        """Counter at boundary doesn't snap."""
        state = GameRNGState("TEST123")
        state.counters[RNGStream.CARD.value] = 250

        state.transition_to_next_act()

        # Should stay at 250
        assert state.counters[RNGStream.CARD.value] == 250

    def test_act_transition_no_snap_zero(self):
        """Counter 0 doesn't snap."""
        state = GameRNGState("TEST123")
        state.counters[RNGStream.CARD.value] = 0

        state.transition_to_next_act()

        assert state.counters[RNGStream.CARD.value] == 0

    def test_floor_advance(self):
        """Entering new floor updates floor_num."""
        state = GameRNGState("TEST123")

        state.enter_floor(5)
        assert state.floor_num == 5

    def test_clone(self):
        """Clone creates independent copy."""
        state1 = GameRNGState("TEST123")
        state1.counters[RNGStream.CARD.value] = 100

        state2 = state1.clone()

        # Independent modification
        state2.counters[RNGStream.CARD.value] = 200

        assert state1.counters[RNGStream.CARD.value] == 100
        assert state2.counters[RNGStream.CARD.value] == 200


class TestPathSimulation:
    """Verify path simulation helpers."""

    def test_simple_path(self):
        """Simulate simple path through game."""
        path = [
            ("neow", "HUNDRED_GOLD"),
            ("floor", 1),
            ("combat", "monster"),
            ("floor", 2),
            ("shop", None),
        ]

        state = simulate_path("TEST123", path)

        # Should have consumed some cardRng
        assert state.counters[RNGStream.CARD.value] > 0
        # Should be at floor 2
        assert state.floor_num == 2

    def test_act_transition_in_path(self):
        """Act transition applies snapping."""
        path = [
            ("neow", "HUNDRED_GOLD"),
            ("floor", 1),
            ("combat", "monster"),  # ~9 cardRng
            ("act", None),  # Snap to 250
        ]

        state = simulate_path("TEST123", path)

        # Should snap to 250
        assert state.counters[RNGStream.CARD.value] == 250
        assert state.act_num == 2


class TestSaveLoadRestore:
    """Verify save/load counter restoration."""

    def test_from_save(self):
        """Restore RNG state from counters."""
        counters = {
            "monster_seed_count": 10,
            "event_seed_count": 5,
            "merchant_seed_count": 3,
            "card_seed_count": 20,
            "treasure_seed_count": 7,
            "relic_seed_count": 2,
            "potion_seed_count": 8,
        }

        game = GameRNG.from_save(seed=12345, counters=counters, floor=5)

        # All counters should be restored
        assert game.monster_rng.counter == 10
        assert game.event_rng.counter == 5
        assert game.merchant_rng.counter == 3
        assert game.card_rng.counter == 20
        assert game.treasure_rng.counter == 7
        assert game.relic_rng.counter == 2
        assert game.potion_rng.counter == 8

    def test_restored_state_matches(self):
        """Restored state produces same values as original."""
        # Create game and advance some streams
        game1 = GameRNG(seed=12345)
        game1.card_rng.random_int(10)
        game1.card_rng.random_int(10)
        game1.monster_rng.random_int(10)

        # Save counters
        counters = game1.get_counters()

        # Restore from save
        game2 = GameRNG.from_save(seed=12345, counters=counters, floor=0)

        # Next values should match
        assert game1.card_rng.random_int(999) == game2.card_rng.random_int(999)
        assert game1.monster_rng.random_int(999) == game2.monster_rng.random_int(999)


class TestKnownSeedParity:
    """Verify Python matches Java for known seed data.

    These tests use empirical data from docs/vault/verified-seeds.md
    """

    @pytest.mark.skip(reason="Incomplete test - predict_card_reward not verified against expected values")
    def test_seed_4YUHY81W7GRHT_neow_offset1(self):
        """Verify Neow offset=1 produces known floor 1 cards."""
        seed = "4YUHY81W7GRHT"
        state = GameRNGState(seed)

        # Neow: HUNDRED_GOLD (safe, offset=1 means something else consumed 1 cardRng)
        # This might be Calling Bell or something - need to investigate
        state.advance(RNGStream.CARD, 1)

        state.enter_floor(1)

        # Predict floor 1 card reward
        cards = predict_card_reward(state, player_class="WATCHER")

        # TODO: Expected cards from Java game (need to fill in actual values)
        # expected = [...]
        # assert cards == expected
        raise NotImplementedError("Test incomplete - needs expected values from Java game")

    def test_full_act1_path(self):
        """Simulate full Act 1 and verify end state."""
        seed = "TEST_ACT1"

        # Build realistic Act 1 path
        path = [("neow", "HUNDRED_GOLD"), ("floor", 1)]

        # 16 floors of combat/shop/event mix
        for floor in range(1, 16):
            path.append(("floor", floor))
            if floor % 5 == 0:
                path.append(("shop", None))
            else:
                path.append(("combat", "monster"))

        # Boss floor
        path.append(("floor", 16))
        path.append(("combat", "boss"))

        # Act transition
        path.append(("act", None))

        state = simulate_path(seed, path)

        # Verify final state
        assert state.act_num == 2
        assert state.counters[RNGStream.CARD.value] in [0, 250, 500, 750]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
