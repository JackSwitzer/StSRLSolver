"""
Comprehensive RNG Test Suite for Seed 1ABCD

This test validates all RNG systems against verified game data.

Verified Data (from actual game):
- Floor 1: Jaw Worm (40 HP, 11 damage attack)
- Floor 1 Card Reward: Like Water, Bowling Bash, Deceive Reality
- Floor 2: Cultist (51 HP)
- Floor 2 Card Reward: Sash Whip, Evaluate, Worship

RNG Streams used:
- monsterRng: Encounter selection (which enemy)
- monsterHpRng: Enemy HP (reseeded per floor: seed + floorNum)
- cardRng: Card rewards (persistent across floors)
"""

import sys
import os
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import importlib.util

def load_module(name, filepath):
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
rng_mod = load_module("rng", os.path.join(core_dir, "state", "rng.py"))
rewards_mod = load_module("rewards", os.path.join(core_dir, "generation", "rewards.py"))
encounters_mod = load_module("encounters", os.path.join(core_dir, "generation", "encounters.py"))


class TestSeed1ABCD(unittest.TestCase):
    """Test all RNG systems for seed 1ABCD."""

    @classmethod
    def setUpClass(cls):
        cls.seed_str = "1ABCD"
        cls.seed = rng_mod.seed_to_long(cls.seed_str)

    def test_seed_conversion(self):
        """Test seed string to long conversion."""
        self.assertEqual(self.seed, 1943283)

    def test_encounter_generation(self):
        """Test encounter selection matches game."""
        monster_rng = rng_mod.Random(self.seed)
        normal, elite = encounters_mod.generate_exordium_encounters(monster_rng)

        # Verified from game
        self.assertEqual(normal[0], "Jaw Worm", "Floor 1 should be Jaw Worm")
        self.assertEqual(normal[1], "Cultist", "Floor 2 should be Cultist")
        self.assertEqual(normal[2], "Small Slimes", "Floor 3 should be Small Slimes")

    def test_enemy_hp_floor1(self):
        """Test Jaw Worm HP on floor 1."""
        # monsterHpRng is seeded with (seed + floorNum)
        hp_rng = rng_mod.Random(self.seed + 1)
        hp = encounters_mod.get_enemy_hp("Jaw Worm", hp_rng)
        self.assertEqual(hp, 40, "Jaw Worm should have 40 HP")

    def test_enemy_hp_floor2(self):
        """Test Cultist HP on floor 2."""
        hp_rng = rng_mod.Random(self.seed + 2)
        hp = encounters_mod.get_enemy_hp("Cultist", hp_rng)
        self.assertEqual(hp, 51, "Cultist should have 51 HP")

    def test_card_reward_floor1(self):
        """Test floor 1 card reward matches game."""
        card_rng = rng_mod.Random(self.seed)
        state = rewards_mod.RewardState()

        cards = rewards_mod.generate_card_rewards(
            card_rng, state, act=1, player_class="WATCHER"
        )

        card_names = [c.name for c in cards]
        expected = ["Like Water", "Bowling Bash", "Deceive Reality"]

        self.assertEqual(card_names, expected,
                        f"Floor 1 cards should be {expected}, got {card_names}")

    def test_card_reward_floor2(self):
        """Test floor 2 card reward matches game."""
        card_rng = rng_mod.Random(self.seed)
        state = rewards_mod.RewardState()

        # Generate floor 1 first (advances RNG)
        rewards_mod.generate_card_rewards(
            card_rng, state, act=1, player_class="WATCHER"
        )

        # Floor 2
        cards = rewards_mod.generate_card_rewards(
            card_rng, state, act=1, player_class="WATCHER"
        )

        card_names = [c.name for c in cards]
        expected = ["Sash Whip", "Evaluate", "Worship"]

        self.assertEqual(card_names, expected,
                        f"Floor 2 cards should be {expected}, got {card_names}")

    def test_card_rng_counter(self):
        """Test that card RNG counter is correct after each floor."""
        card_rng = rng_mod.Random(self.seed)
        state = rewards_mod.RewardState()

        # Floor 1: 3 cards * (rarity + index) + 3 upgrade checks = 9 calls
        rewards_mod.generate_card_rewards(
            card_rng, state, act=1, player_class="WATCHER"
        )
        self.assertEqual(card_rng.counter, 9, "Floor 1 should use 9 RNG calls")

        # Floor 2: another 9 calls = 18 total
        rewards_mod.generate_card_rewards(
            card_rng, state, act=1, player_class="WATCHER"
        )
        self.assertEqual(card_rng.counter, 18, "Floor 2 should use 18 total RNG calls")

    def test_card_blizzard_state(self):
        """Test card blizzard (pity timer) state."""
        card_rng = rng_mod.Random(self.seed)
        state = rewards_mod.RewardState()

        # Initial blizzard offset
        self.assertEqual(state.card_blizzard.offset, 5)

        # After floor 1: U, C, U -> 5 stays, 4 (C decreases), 4 stays
        rewards_mod.generate_card_rewards(
            card_rng, state, act=1, player_class="WATCHER"
        )
        self.assertEqual(state.card_blizzard.offset, 4)

        # After floor 2: C, C, U -> 3, 2, 2
        rewards_mod.generate_card_rewards(
            card_rng, state, act=1, player_class="WATCHER"
        )
        self.assertEqual(state.card_blizzard.offset, 2)


class TestRNGConsistency(unittest.TestCase):
    """Test RNG implementation consistency."""

    def test_xorshift_determinism(self):
        """Test that XorShift128 produces deterministic sequence."""
        seed = rng_mod.seed_to_long("1ABCD")

        rng1 = rng_mod.Random(seed)
        rng2 = rng_mod.Random(seed)

        for _ in range(100):
            self.assertEqual(rng1.random(99), rng2.random(99))

    def test_random_methods(self):
        """Test different random methods produce expected results."""
        seed = rng_mod.seed_to_long("1ABCD")
        rng = rng_mod.Random(seed)

        # First few random(99) values for seed 1ABCD
        expected = [23, 4, 92, 44, 30]
        actual = [rng.random(99) for _ in range(5)]

        self.assertEqual(actual, expected,
                        f"random(99) sequence should be {expected}")


def run_tests():
    """Run all tests and print summary."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    suite.addTests(loader.loadTestsFromTestCase(TestSeed1ABCD))
    suite.addTests(loader.loadTestsFromTestCase(TestRNGConsistency))

    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_tests()
    sys.exit(0 if success else 1)
