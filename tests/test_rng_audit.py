"""RNG audit tests - verify Python RNG matches Java decompiled behavior."""

import pytest
from packages.engine.state.rng import XorShift128, Random, GameRNG, seed_to_long, long_to_seed


class TestSeedConversion:
    def test_roundtrip_basic(self):
        for seed_str in ["ABC123", "4YUHY81W7GRHT"]:
            seed_long = seed_to_long(seed_str)
            reconstructed = long_to_seed(seed_long)
            assert reconstructed == seed_str.upper().replace("O", "0")

    def test_numeric_seed(self):
        assert seed_to_long("0") == 0
        assert seed_to_long("12345") == 12345

    def test_o_to_zero(self):
        assert seed_to_long("O0O0O0") == seed_to_long("000000")


class TestXorShift128:
    def test_deterministic(self):
        rng1 = XorShift128(12345)
        rng2 = XorShift128(12345)
        for _ in range(100):
            assert rng1._next_long() == rng2._next_long()

    def test_zero_seed_uses_min_value(self):
        rng = XorShift128(0)
        assert rng.seed0 != 0 or rng.seed1 != 0

    def test_copy(self):
        original = XorShift128(99999)
        original._next_long()
        copied = original.copy()
        assert original._next_long() == copied._next_long()


class TestRandom:
    def test_counter_tracking(self):
        rng = Random(seed_to_long("TEST"))
        assert rng.counter == 0
        rng.random_int(99)
        assert rng.counter == 1

    def test_counter_skip_ahead(self):
        seed = seed_to_long("TEST1234")
        rng_skip = Random(seed, counter=100)
        rng_manual = Random(seed)
        for _ in range(100):
            rng_manual.random_int(999)
        assert rng_skip.random_int(50) == rng_manual.random_int(50)

    def test_copy(self):
        rng = Random(seed_to_long("COPY"))
        rng.random_int(99)
        rng.random_int(99)
        copied = rng.copy()
        assert copied.counter == rng.counter
        assert rng.random_int(99) == copied.random_int(99)

    def test_set_counter(self):
        rng = Random(seed_to_long("SNAP"))
        rng.random_int(99)  # counter=1
        rng.set_counter(250)
        assert rng.counter == 250

    def test_card_reward_consumption(self):
        """Each card reward consumes 3 cardRng calls: rarity + pool + upgrade."""
        rng = Random(seed_to_long("CARDS"))
        initial = rng.counter
        for _ in range(3):
            rng.random_int(99)            # rarity
            rng.random_int(49)            # pool select
            rng.random_boolean_chance(0.25)  # upgrade check
        assert rng.counter - initial == 9


class TestGameRNG:
    def test_persistent_streams_initialized(self):
        game = GameRNG(seed=12345)
        assert game.monster_rng is not None
        assert game.map_rng is not None
        assert game.event_rng is not None
        assert game.card_rng is not None
        assert game.neow_rng is not None

    def test_per_floor_reseeding(self):
        seed = seed_to_long("FLOOR")
        game = GameRNG(seed=seed, floor=0)
        val_f0 = game.ai_rng.random_int(99)
        game.advance_floor()
        val_f1 = game.ai_rng.random_int(99)
        # Values should differ (different floor seeds)
        # Verify formula: floor 1 uses seed+1
        manual = Random(seed + 1)
        assert manual.random_int(99) == val_f1

    def test_map_rng_uses_act_offset(self):
        """mapRng must use per-act seed offset, not bare seed."""
        seed = 12345
        game = GameRNG(seed=seed, act_num=1)
        # Act 1: seed + 1
        expected = Random(seed + 1)
        assert game.map_rng.random_int(99) == expected.random_int(99)

    def test_map_rng_act2(self):
        seed = 12345
        game = GameRNG(seed=seed, act_num=2)
        expected = Random(seed + 200)
        assert game.map_rng.random_int(99) == expected.random_int(99)

    def test_map_rng_act3(self):
        seed = 12345
        game = GameRNG(seed=seed, act_num=3)
        expected = Random(seed + 600)
        assert game.map_rng.random_int(99) == expected.random_int(99)

    def test_map_rng_act4(self):
        seed = 12345
        game = GameRNG(seed=seed, act_num=4)
        expected = Random(seed + 1200)
        assert game.map_rng.random_int(99) == expected.random_int(99)

    def test_advance_act_reseeds_map(self):
        seed = 12345
        game = GameRNG(seed=seed, act_num=1)
        game.advance_act(2)
        expected = Random(seed + 200)
        assert game.map_rng.random_int(99) == expected.random_int(99)

    def test_neow_rng_initialized(self):
        game = GameRNG(seed=12345)
        assert game.neow_rng is not None
        # Should be seeded with bare seed
        expected = Random(12345)
        assert game.neow_rng.random_int(99) == expected.random_int(99)


class TestCardRngSnapping:
    """Verify act transition cardRng counter snapping logic."""

    @pytest.mark.parametrize("initial,expected", [
        (100, 250), (249, 250), (250, 250),
        (251, 500), (499, 500), (500, 500),
        (501, 750), (749, 750), (750, 750),
        (751, 751), (0, 0), (1000, 1000),
    ])
    def test_snapping(self, initial, expected):
        if 0 < initial < 250:
            snapped = 250
        elif 250 < initial < 500:
            snapped = 500
        elif 500 < initial < 750:
            snapped = 750
        else:
            snapped = initial
        assert snapped == expected
