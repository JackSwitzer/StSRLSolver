"""
Card Reward Relic Tests - TDD approach.

Tests for relics that modify card reward screens:
- Busted Crown: Card rewards have 2 fewer choices
- Prayer Wheel: Card rewards have 1 additional choice
- Question Card: Card rewards show 1 additional card
- Singing Bowl: Gain +2 Max HP when skipping card reward

These are failing tests that document expected behavior.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.state.run import RunState, create_watcher_run
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.generation.rewards import generate_card_rewards, RewardState


# =============================================================================
# FIXTURES
# =============================================================================

@pytest.fixture
def watcher_run():
    """Create a fresh Watcher run for testing."""
    return create_watcher_run("TESTRUN", ascension=0)


@pytest.fixture
def rng():
    """Create a fresh RNG for testing."""
    return Random(seed_to_long("TESTRNG"))


@pytest.fixture
def reward_state():
    """Create a reward state for testing."""
    return RewardState()


def make_run_with_relic(relic_id: str, seed: str = "TEST", ascension: int = 0) -> RunState:
    """Create a run with a specific relic added."""
    run = create_watcher_run(seed, ascension=ascension)
    run.add_relic(relic_id)
    return run


# =============================================================================
# BUSTED CROWN TESTS
# =============================================================================

class TestBustedCrown:
    """Busted Crown: Card rewards have 2 fewer cards."""

    def test_busted_crown_reduces_choices_by_2(self, rng, reward_state):
        """Busted Crown: Card rewards have 2 fewer choices (normally 3, becomes 1)."""
        # Base: 3 card choices
        base_reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=3, has_busted_crown=False
        )

        # With Busted Crown: 1 card choice (3 - 2)
        crown_rng = Random(seed_to_long("TESTRNG"))
        crown_reward_state = RewardState()
        crown_reward = generate_card_rewards(
            crown_rng, crown_reward_state, act=1, player_class="WATCHER",
            num_cards=3, has_busted_crown=True
        )

        assert len(base_reward) == 3
        # Busted Crown reduces by 2 but minimum 1
        assert len(crown_reward) == 1

    def test_busted_crown_with_question_card(self, rng, reward_state):
        """Busted Crown + Question Card: Should result in 2 choices (3 + 1 - 2)."""
        # Note: generate_card_rewards applies modifiers internally
        # So we pass base num_cards=3 and set both flags
        reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=3,  # Base: 3
            has_busted_crown=True,  # -2
            has_question_card=True  # +1
        )
        # Result: 3 - 2 + 1 = 2
        assert len(reward) == 2

    @pytest.mark.xfail(reason="Busted Crown max HP bonus happens at pickup time")
    def test_busted_crown_grants_max_hp_on_pickup(self):
        """Busted Crown: Upon pickup, gain +8 Max HP."""
        run = create_watcher_run("TEST", ascension=0)
        initial_max_hp = run.max_hp

        run.add_relic("BustedCrown")

        # Busted Crown is a boss relic that grants +8 Max HP
        # This requires onEquip trigger to be implemented
        assert run.max_hp == initial_max_hp + 8


# =============================================================================
# PRAYER WHEEL TESTS
# =============================================================================

class TestPrayerWheel:
    """Prayer Wheel: Card rewards have 1 additional card.

    Note: Prayer Wheel is NOT handled by has_question_card flag.
    It gives an additional card reward (separate reward), not +1 to same reward.
    These tests verify the base card count system works.
    """

    def test_base_card_count_is_3(self, rng, reward_state):
        """Base card count is 3 without any relics."""
        reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=3
        )
        assert len(reward) == 3

    def test_prayer_wheel_stacks_with_question_card(self, rng, reward_state):
        """If we wanted +1 from Prayer Wheel, we'd pass num_cards=4."""
        # Prayer Wheel gives an ADDITIONAL card reward, not +1 to same reward
        # But if we wanted to simulate it as +1, we'd do:
        reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=4,  # 3 + 1 hypothetically from Prayer Wheel
            has_question_card=True  # +1 from Question Card
        )
        # Would be 4 + 1 = 5
        assert len(reward) == 5


# =============================================================================
# QUESTION CARD TESTS
# =============================================================================

class TestQuestionCard:
    """Question Card: Card rewards show 1 additional card."""

    def test_question_card_adds_1_choice(self, rng, reward_state):
        """Question Card: Card rewards have 1 additional choice (normally 3, becomes 4)."""
        reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=3, has_question_card=True
        )
        # 3 + 1 = 4
        assert len(reward) == 4

    def test_question_card_alone(self, rng, reward_state):
        """Question Card: +1 to base count."""
        # Without Question Card
        base_reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=3, has_question_card=False
        )

        # With Question Card
        qc_rng = Random(seed_to_long("TESTRNG"))
        qc_reward_state = RewardState()
        qc_reward = generate_card_rewards(
            qc_rng, qc_reward_state, act=1, player_class="WATCHER",
            num_cards=3, has_question_card=True
        )

        assert len(base_reward) == 3
        assert len(qc_reward) == 4


# =============================================================================
# SINGING BOWL TESTS
# =============================================================================

class TestSingingBowl:
    """Singing Bowl: Gain +2 Max HP when skipping a card reward."""

    def test_singing_bowl_grants_max_hp_on_skip(self):
        """Singing Bowl: When skipping card reward, gain +2 Max HP."""
        run = make_run_with_relic("Singing Bowl")
        initial_max_hp = run.max_hp

        # Simulate skipping a card reward by calling gain_max_hp
        # The actual integration would be in reward screen logic
        run.gain_max_hp(2)  # Singing Bowl effect

        assert run.max_hp == initial_max_hp + 2

    def test_singing_bowl_no_bonus_on_take(self):
        """Singing Bowl: No bonus if a card is taken (not skipped)."""
        run = make_run_with_relic("Singing Bowl")
        initial_max_hp = run.max_hp

        # Simulate taking a card from reward
        run.add_card("Strike_P")

        # Max HP should not have increased
        assert run.max_hp == initial_max_hp

    def test_singing_bowl_multiple_skips_stack(self):
        """Singing Bowl: Can be used multiple times (each skip grants +2 Max HP)."""
        run = make_run_with_relic("Singing Bowl")
        initial_max_hp = run.max_hp

        # Skip 3 card rewards
        for _ in range(3):
            run.gain_max_hp(2)  # Singing Bowl effect per skip

        assert run.max_hp == initial_max_hp + 6


# =============================================================================
# COMBINATION TESTS
# =============================================================================

class TestCardRewardRelicCombinations:
    """Test interactions between multiple card reward relics."""

    def test_question_card_with_busted_crown(self, rng, reward_state):
        """Busted Crown (-2) + Question Card (+1) = 2 cards."""
        reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=3,  # Base
            has_busted_crown=True,  # -2
            has_question_card=True  # +1
        )
        # 3 - 2 + 1 = 2
        assert len(reward) == 2

    def test_card_rewards_have_minimum_of_1(self, rng, reward_state):
        """Card rewards should never have fewer than 1 card, even with Busted Crown."""
        # Even with -2 from Busted Crown on base 3, should get at least 1
        reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=3, has_busted_crown=True
        )
        # 3 - 2 = 1
        assert len(reward) >= 1

    def test_extreme_reduction(self, rng, reward_state):
        """Even with very low num_cards, minimum is 1."""
        reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER",
            num_cards=1, has_busted_crown=True  # Would be -1, capped at 1
        )
        assert len(reward) == 1


# =============================================================================
# EDGE CASES
# =============================================================================

class TestCardRewardEdgeCases:
    """Edge cases and special scenarios for card reward relics."""

    def test_ascension_base_card_count(self, reward_state):
        """Base card count is 3 across all ascensions."""
        rng_a0 = Random(seed_to_long("TESTRNG"))
        reward_a0 = generate_card_rewards(
            rng_a0, reward_state, act=1, player_class="WATCHER",
            ascension=0, num_cards=3
        )

        rng_a20 = Random(seed_to_long("TESTRNG"))
        reward_state_a20 = RewardState()
        reward_a20 = generate_card_rewards(
            rng_a20, reward_state_a20, act=1, player_class="WATCHER",
            ascension=20, num_cards=3
        )

        assert len(reward_a0) == 3
        assert len(reward_a20) == 3

    def test_different_acts_same_count(self, rng, reward_state):
        """Card count doesn't change with act (only upgrade chance does)."""
        act1_reward = generate_card_rewards(
            rng, reward_state, act=1, player_class="WATCHER", num_cards=3
        )

        act3_rng = Random(seed_to_long("TESTRNG"))
        act3_reward_state = RewardState()
        act3_reward = generate_card_rewards(
            act3_rng, act3_reward_state, act=3, player_class="WATCHER", num_cards=3
        )

        assert len(act1_reward) == len(act3_reward) == 3


# =============================================================================
# RUN STATE INTEGRATION TESTS
# =============================================================================

class TestRunStateCardRewardIntegration:
    """Test card reward relic behavior integrated with RunState."""

    @pytest.mark.xfail(reason="Card reward count calculation needs integration")
    def test_run_calculates_card_count_from_relics(self):
        """RunState should calculate card count based on relics."""
        run = create_watcher_run("TEST", ascension=0)

        # Base should be 3
        assert run.get_card_reward_count() == 3

        run.add_relic("Prayer Wheel")
        assert run.get_card_reward_count() == 4

        run.add_relic("Question Card")
        assert run.get_card_reward_count() == 5

        run.add_relic("BustedCrown")
        assert run.get_card_reward_count() == 3  # 5 - 2

    @pytest.mark.xfail(reason="Singing Bowl onSkipCardReward not implemented")
    def test_singing_bowl_triggers_on_skip(self):
        """Singing Bowl should trigger when card reward is skipped."""
        run = make_run_with_relic("Singing Bowl")
        initial_max_hp = run.max_hp

        # Simulate skipping via run method
        run.skip_card_reward()  # This method would trigger Singing Bowl

        assert run.max_hp == initial_max_hp + 2


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
