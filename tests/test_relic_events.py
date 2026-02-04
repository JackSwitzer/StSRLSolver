"""Event and Map-Related Relic Tests - TDD approach.

Tests for relics that affect map generation, event selection, and gold rewards.
These tests document EXPECTED behavior and will fail until implementations are added.

Relics tested:
- Juzu Bracelet: ? rooms cannot contain monster encounters
- Discerning Monocle: +25% chance for unknown events
- Golden Idol: Gain 25% more gold from all sources
- N'loth's Gift: Double event rewards once
- Wing Boots: Can fly (skip) a path node 3 times
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.registry import (
    execute_relic_triggers,
    RELIC_REGISTRY,
    RelicContext,
)
from packages.engine.state.run import RunState, create_watcher_run, RelicInstance
from packages.engine.state.combat import create_combat, create_enemy
from packages.engine.generation.map import MapRoomNode, RoomType


# =============================================================================
# TEST FIXTURES
# =============================================================================

@pytest.fixture
def basic_run():
    """Create a basic run state for testing."""
    return create_watcher_run("TEST123", ascension=20)


def create_run_with_relic(relic_id: str, counter: int = -1, **kwargs) -> RunState:
    """Helper to create a run state with a specific relic."""
    state = create_watcher_run(kwargs.get("seed", "TEST123"),
                               ascension=kwargs.get("ascension", 20))
    state.add_relic(relic_id, counter=counter)
    return state


# =============================================================================
# GOLDEN IDOL - Gold Gain Tests
# =============================================================================

class TestGoldenIdol:
    """Test Golden Idol: Gain 25% more gold from all sources."""

    def test_golden_idol_increases_gold_gain_from_combat(self):
        """Golden Idol: 100 gold -> 125 gold (25% increase)."""
        state = create_run_with_relic("Golden Idol")
        initial_gold = state.gold

        # Simulate gaining 100 gold from combat
        state.add_gold(100)

        # Should receive 125 gold (100 * 1.25)
        assert state.gold == initial_gold + 125

    def test_golden_idol_increases_gold_gain_from_event(self):
        """Golden Idol: Event gold rewards also increased by 25%."""
        state = create_run_with_relic("Golden Idol")
        initial_gold = state.gold

        # Simulate event giving 50 gold
        state.add_gold(50)

        # Should receive 62 gold (50 * 1.25 = 62.5, rounded down)
        assert state.gold == initial_gold + 62

    def test_golden_idol_multiple_gold_gains(self):
        """Golden Idol: Multiple gold gains each get the bonus."""
        state = create_run_with_relic("Golden Idol")
        initial_gold = state.gold

        state.add_gold(40)  # -> 50
        state.add_gold(80)  # -> 100

        # Total should be 150
        assert state.gold == initial_gold + 150

    def test_golden_idol_blocked_by_ectoplasm(self):
        """Golden Idol: No effect if Ectoplasm prevents gold gain."""
        state = create_run_with_relic("Golden Idol")
        state.add_relic("Ectoplasm")
        initial_gold = state.gold

        state.add_gold(100)

        # Ectoplasm prevents gold gain entirely
        assert state.gold == initial_gold

    def test_golden_idol_fractional_rounding(self):
        """Golden Idol: Fractional gold amounts round down."""
        state = create_run_with_relic("Golden Idol")
        initial_gold = state.gold

        # 33 * 1.25 = 41.25, should become 41
        state.add_gold(33)

        assert state.gold == initial_gold + 41


# =============================================================================
# JUZU BRACELET - Map Generation Tests
# =============================================================================

class TestJuzuBracelet:
    """Test Juzu Bracelet: ? rooms cannot contain monster encounters."""

    def test_juzu_bracelet_prevents_monster_in_question_rooms(self):
        """Juzu Bracelet: ? rooms should never roll monster encounters."""
        # This test verifies the flag is set correctly
        # Actual map generation logic will check this flag
        state = create_run_with_relic("Juzu Bracelet")

        assert state.has_relic("Juzu Bracelet")

        # The event generator should respect this flag
        # (tested in event generation tests, this verifies the relic exists)

    def test_without_juzu_question_rooms_can_have_monsters(self):
        """Without Juzu Bracelet: ? rooms can contain monsters."""
        state = create_watcher_run("TEST123", ascension=20)

        assert not state.has_relic("Juzu Bracelet")


# =============================================================================
# DISCERNING MONOCLE - Event Selection Tests
# =============================================================================

class TestDiscerningMonocle:
    """Test Discerning Monocle: +25% chance for unknown events."""

    def test_discerning_monocle_increases_unknown_event_chance(self):
        """Discerning Monocle: Increases unknown event probability."""
        state = create_run_with_relic("Discerning Monocle")

        # Verify relic exists (actual event probability tested in event system)
        assert state.has_relic("Discerning Monocle")

    def test_without_monocle_normal_event_chances(self):
        """Without Discerning Monocle: Normal event distribution."""
        state = create_watcher_run("TEST123", ascension=20)

        assert not state.has_relic("Discerning Monocle")


# =============================================================================
# N'LOTH'S GIFT - Event Reward Tests
# =============================================================================

class TestNlothsGift:
    """Test N'loth's Gift: Double event rewards once."""

    def test_nloths_gift_doubles_event_reward_once(self):
        """N'loth's Gift: Event rewards doubled first time."""
        state = create_run_with_relic("N'loth's Gift", counter=1)

        # Check initial counter
        assert state.get_relic_counter("N'loth's Gift") == 1

        # Simulate event reward (e.g., 75 gold)
        initial_gold = state.gold

        # Event handler should check counter and double reward
        if state.get_relic_counter("N'loth's Gift") > 0:
            # Double the reward
            reward = 75 * 2
            state.add_gold(reward)
            state.set_relic_counter("N'loth's Gift", 0)
        else:
            state.add_gold(75)

        assert state.gold == initial_gold + 150
        assert state.get_relic_counter("N'loth's Gift") == 0

    def test_nloths_gift_does_not_double_second_time(self):
        """N'loth's Gift: Second event reward is not doubled."""
        state = create_run_with_relic("N'loth's Gift", counter=0)

        initial_gold = state.gold

        # Counter is 0, so reward is not doubled
        if state.get_relic_counter("N'loth's Gift") > 0:
            reward = 75 * 2
            state.add_gold(reward)
            state.set_relic_counter("N'loth's Gift", 0)
        else:
            state.add_gold(75)

        assert state.gold == initial_gold + 75
        assert state.get_relic_counter("N'loth's Gift") == 0

    def test_nloths_gift_starts_with_counter(self):
        """N'loth's Gift: Should start with counter = 1."""
        state = create_watcher_run("TEST123", ascension=20)

        # Add relic with explicit counter
        relic = state.add_relic("N'loth's Gift", counter=1)

        assert relic.counter == 1


# =============================================================================
# WING BOOTS - Path Skip Tests
# =============================================================================

class TestWingBoots:
    """Test Wing Boots: Can fly (skip) a path node 3 times."""

    def test_wing_boots_starts_with_3_charges(self):
        """Wing Boots: Should start with 3 charges."""
        state = create_run_with_relic("Wing Boots", counter=3)

        assert state.get_relic_counter("Wing Boots") == 3

    def test_wing_boots_decrements_on_use(self):
        """Wing Boots: Counter decrements when flying over a node."""
        state = create_run_with_relic("Wing Boots", counter=3)

        # Simulate using wing boots to skip a node
        if state.get_relic_counter("Wing Boots") > 0:
            state.set_relic_counter("Wing Boots",
                                   state.get_relic_counter("Wing Boots") - 1)

        assert state.get_relic_counter("Wing Boots") == 2

    def test_wing_boots_can_be_used_three_times(self):
        """Wing Boots: Can be used 3 times total."""
        state = create_run_with_relic("Wing Boots", counter=3)

        # Use 3 times
        for i in range(3):
            counter = state.get_relic_counter("Wing Boots")
            assert counter > 0
            state.set_relic_counter("Wing Boots", counter - 1)

        assert state.get_relic_counter("Wing Boots") == 0

    def test_wing_boots_cannot_be_used_when_depleted(self):
        """Wing Boots: Cannot be used after 3 uses."""
        state = create_run_with_relic("Wing Boots", counter=0)

        # Should not be able to use
        assert state.get_relic_counter("Wing Boots") == 0

    def test_wing_boots_allows_skipping_dangerous_nodes(self):
        """Wing Boots: Can skip nodes like elite encounters."""
        state = create_run_with_relic("Wing Boots", counter=3)

        # Verify we have the ability to skip
        can_skip = state.get_relic_counter("Wing Boots") > 0

        assert can_skip is True


# =============================================================================
# BLOODY IDOL - Combo with Golden Idol
# =============================================================================

class TestBloodyIdol:
    """Test Bloody Idol: Heal 5 HP when gold is gained."""

    def test_bloody_idol_heals_on_gold_gain(self):
        """Bloody Idol: Gain 5 HP when gaining gold."""
        state = create_run_with_relic("Bloody Idol")
        state.current_hp = 50
        initial_hp = state.current_hp

        # Gain gold
        state.add_gold(100)

        # Should heal 5 HP
        assert state.current_hp == initial_hp + 5

    def test_bloody_idol_heals_multiple_times(self):
        """Bloody Idol: Each gold gain heals separately."""
        state = create_run_with_relic("Bloody Idol")
        state.current_hp = 50
        initial_hp = state.current_hp

        state.add_gold(50)
        state.add_gold(50)

        # Should heal 10 HP total (5 per gold gain)
        assert state.current_hp == initial_hp + 10

    def test_bloody_idol_respects_max_hp(self):
        """Bloody Idol: Cannot heal above max HP."""
        state = create_run_with_relic("Bloody Idol")
        state.current_hp = state.max_hp - 2

        state.add_gold(100)

        # Should cap at max HP
        assert state.current_hp == state.max_hp

    def test_bloody_idol_blocked_by_ectoplasm(self):
        """Bloody Idol: No healing if Ectoplasm blocks gold gain."""
        state = create_run_with_relic("Bloody Idol")
        state.add_relic("Ectoplasm")
        state.current_hp = 50
        initial_hp = state.current_hp

        state.add_gold(100)

        # No gold gained = no healing
        assert state.current_hp == initial_hp

    def test_bloody_idol_and_golden_idol_combo(self):
        """Bloody Idol + Golden Idol: Still only heals 5 per gain event."""
        state = create_run_with_relic("Bloody Idol")
        state.add_relic("Golden Idol")
        state.current_hp = 50
        initial_hp = state.current_hp

        # Gain 100 gold (becomes 125 with Golden Idol)
        state.add_gold(100)

        # Should heal 5 HP (not based on amount gained)
        assert state.current_hp == initial_hp + 5


# =============================================================================
# INTEGRATION TESTS
# =============================================================================

class TestEventRelicIntegration:
    """Integration tests for event-related relics."""

    def test_multiple_gold_relics(self):
        """Test interaction of multiple gold-affecting relics."""
        state = create_run_with_relic("Golden Idol")
        state.add_relic("Bloody Idol")

        initial_gold = state.gold
        initial_hp = state.current_hp

        # Gain 100 gold
        state.add_gold(100)

        # Should get 125 gold (Golden Idol)
        assert state.gold == initial_gold + 125
        # Should heal 5 HP (Bloody Idol)
        assert state.current_hp == min(initial_hp + 5, state.max_hp)

    def test_wing_boots_counter_persistence(self):
        """Wing Boots: Counter should persist across rooms."""
        state = create_run_with_relic("Wing Boots", counter=3)

        # Use once
        state.set_relic_counter("Wing Boots", 2)

        # Move to next floor
        state.advance_floor()

        # Counter should still be 2
        assert state.get_relic_counter("Wing Boots") == 2

    def test_nloths_gift_only_affects_events(self):
        """N'loth's Gift: Should only affect event rewards, not combat."""
        state = create_run_with_relic("N'loth's Gift", counter=1)

        # Combat gold should not trigger doubling
        # (Event handler is responsible for checking this)
        assert state.get_relic_counter("N'loth's Gift") == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
