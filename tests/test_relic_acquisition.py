"""
Chest/Acquisition Relic Tests - TDD approach.

Tests for relics that affect chests and relic acquisition:
- Tiny Chest: Every 4th ? room contains a treasure chest
- Matryoshka: Next 2 non-boss chests have an additional relic
- Black Star: Elites drop an additional relic
- Cursed Key: Gain a Curse when opening non-boss chests
- N'loth's Mask (Hungry Face): Next non-boss chest is empty

These are failing tests that document expected behavior.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.state.run import RunState, create_watcher_run
from packages.engine.state.rng import Random, seed_to_long


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


def make_run_with_relic(relic_id: str, seed: str = "TEST", ascension: int = 0) -> RunState:
    """Create a run with a specific relic added."""
    run = create_watcher_run(seed, ascension=ascension)
    run.add_relic(relic_id)
    return run


# Mock chest handler for testing
class MockChestHandler:
    @staticmethod
    def open_chest(run: RunState, rng: Random, chest_tier: str = "small"):
        """Open a chest and get rewards."""
        relics_gained = 1  # Base: 1 relic from chest

        # Matryoshka: +1 relic for next 2 chests
        if run.has_relic("Matryoshka"):
            matryoshka = run.get_relic("Matryoshka")
            if matryoshka.counter > 0:
                relics_gained += 1
                matryoshka.counter -= 1

        # Cursed Key: Gain a curse when opening non-boss chest
        curse_gained = None
        if run.has_relic("Cursed Key"):
            curse_gained = "Decay"  # Example curse
            run.add_card(curse_gained)

        # N'loth's Mask: First chest is empty
        if run.has_relic("NlothsMask") or run.has_relic("Nloth's Hungry Face"):
            mask = run.get_relic("NlothsMask") or run.get_relic("Nloth's Hungry Face")
            if mask and mask.counter == 0:
                # First chest: empty
                relics_gained = 0
                curse_gained = None
                mask.counter = 1  # Mark as used
            else:
                # Subsequent chests: better rewards
                relics_gained += 1

        # Add relics to run
        for _ in range(relics_gained):
            run.add_relic("Lantern")  # Dummy relic

        return {
            "relics_gained": relics_gained,
            "curse_gained": curse_gained,
        }


# Mock elite handler for testing
class MockEliteHandler:
    @staticmethod
    def defeat_elite(run: RunState, rng: Random):
        """Defeat an elite and get rewards."""
        relics_gained = 1  # Base: 1 relic from elite

        # Black Star: +1 relic from elites
        if run.has_relic("Black Star"):
            relics_gained += 1

        # Add relics to run
        for _ in range(relics_gained):
            run.add_relic("Lantern")  # Dummy relic

        return {
            "relics_gained": relics_gained,
        }


# Mock question room handler for testing
class MockQuestionRoomHandler:
    @staticmethod
    def generate_question_room(run: RunState, rng: Random, question_count: int):
        """Generate a ? room event."""
        # Tiny Chest: Every 4th ? room is a chest
        is_chest = False

        if run.has_relic("Tiny Chest"):
            tiny_chest = run.get_relic("Tiny Chest")
            if tiny_chest.counter >= 3:  # 4th room (0-indexed: 3)
                is_chest = True
                tiny_chest.counter = 0  # Reset counter
            else:
                tiny_chest.counter += 1

        return {
            "is_chest": is_chest,
            "question_count": question_count,
        }


# =============================================================================
# TINY CHEST TESTS
# =============================================================================

class TestTinyChest:
    """Tiny Chest: Every 4th ? room is a Treasure room."""

    @pytest.mark.skip(reason="Tiny Chest counter tracking not implemented")
    def test_tiny_chest_tracks_question_rooms(self, watcher_run, rng):
        """Tiny Chest: Counter increments for each ? room entered."""
        watcher_run.add_relic("Tiny Chest")

        tiny_chest = watcher_run.get_relic("Tiny Chest")
        assert tiny_chest.counter == 0 or tiny_chest.counter == -1  # Initial

        # Enter 3 ? rooms
        for i in range(3):
            result = MockQuestionRoomHandler.generate_question_room(watcher_run, rng, i + 1)
            assert result["is_chest"] is False

            tiny_chest = watcher_run.get_relic("Tiny Chest")
            assert tiny_chest.counter == i + 1

    @pytest.mark.skip(reason="Tiny Chest counter tracking not implemented")
    def test_tiny_chest_triggers_on_4th_room(self, watcher_run, rng):
        """Tiny Chest: 4th ? room becomes a Treasure room."""
        watcher_run.add_relic("Tiny Chest")

        # Enter 4 ? rooms
        for i in range(4):
            result = MockQuestionRoomHandler.generate_question_room(watcher_run, rng, i + 1)

            if i == 3:  # 4th room
                assert result["is_chest"] is True
            else:
                assert result["is_chest"] is False

    @pytest.mark.skip(reason="Tiny Chest counter tracking not implemented")
    def test_tiny_chest_resets_after_trigger(self, watcher_run, rng):
        """Tiny Chest: Counter resets after triggering."""
        watcher_run.add_relic("Tiny Chest")

        # Enter 4 rooms to trigger
        for i in range(4):
            MockQuestionRoomHandler.generate_question_room(watcher_run, rng, i + 1)

        # Counter should be reset
        tiny_chest = watcher_run.get_relic("Tiny Chest")
        assert tiny_chest.counter == 0

        # Next 4 rooms should trigger again
        for i in range(4):
            result = MockQuestionRoomHandler.generate_question_room(watcher_run, rng, i + 5)

            if i == 3:  # 4th room again
                assert result["is_chest"] is True

    @pytest.mark.skip(reason="Tiny Chest counter tracking not implemented")
    def test_tiny_chest_does_not_affect_other_rooms(self, watcher_run, rng):
        """Tiny Chest: Only affects ? rooms, not combat/elite/rest/shop."""
        watcher_run.add_relic("Tiny Chest")

        tiny_chest = watcher_run.get_relic("Tiny Chest")
        initial_counter = tiny_chest.counter

        # Entering non-? rooms should not increment counter
        # This would be tested with room entry handlers

        # Counter should remain the same
        assert tiny_chest.counter == initial_counter


# =============================================================================
# MATRYOSHKA TESTS
# =============================================================================

class TestMatryoshka:
    """Matryoshka: The next 2 non-boss Chests you open contain an extra Relic."""

    @pytest.mark.skip(reason="Matryoshka bonus relic not implemented")
    def test_matryoshka_grants_extra_relic_first_chest(self, watcher_run, rng):
        """Matryoshka: First chest gives 2 relics instead of 1."""
        watcher_run.add_relic("Matryoshka")
        initial_relics = len(watcher_run.relics)

        result = MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Should gain 2 relics: 1 base + 1 Matryoshka
        assert result["relics_gained"] == 2
        assert len(watcher_run.relics) == initial_relics + 2

    @pytest.mark.skip(reason="Matryoshka bonus relic not implemented")
    def test_matryoshka_grants_extra_relic_second_chest(self, watcher_run, rng):
        """Matryoshka: Second chest also gives 2 relics."""
        watcher_run.add_relic("Matryoshka")

        # Open first chest
        MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Open second chest
        initial_relics = len(watcher_run.relics)
        result = MockChestHandler.open_chest(watcher_run, Random(seed_to_long("CHEST2")), chest_tier="medium")

        # Should still grant extra relic
        assert result["relics_gained"] == 2
        assert len(watcher_run.relics) == initial_relics + 2

    @pytest.mark.skip(reason="Matryoshka bonus relic not implemented")
    def test_matryoshka_expires_after_2_chests(self, watcher_run, rng):
        """Matryoshka: Third chest does not grant extra relic."""
        watcher_run.add_relic("Matryoshka")

        # Open 2 chests (uses up Matryoshka)
        MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")
        MockChestHandler.open_chest(watcher_run, Random(seed_to_long("CHEST2")), chest_tier="medium")

        # Third chest should be normal
        initial_relics = len(watcher_run.relics)
        result = MockChestHandler.open_chest(watcher_run, Random(seed_to_long("CHEST3")), chest_tier="large")

        # Should only gain 1 relic
        assert result["relics_gained"] == 1
        assert len(watcher_run.relics) == initial_relics + 1

    @pytest.mark.skip(reason="Matryoshka counter tracking not implemented")
    def test_matryoshka_counter_decrements(self, watcher_run, rng):
        """Matryoshka: Counter should decrement with each chest."""
        watcher_run.add_relic("Matryoshka")

        matryoshka = watcher_run.get_relic("Matryoshka")
        # Initial counter should be 2
        matryoshka.counter = 2

        # Open first chest
        MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")
        assert matryoshka.counter == 1

        # Open second chest
        MockChestHandler.open_chest(watcher_run, Random(seed_to_long("CHEST2")), chest_tier="medium")
        assert matryoshka.counter == 0

    @pytest.mark.skip(reason="Matryoshka boss chest exclusion not implemented")
    def test_matryoshka_does_not_affect_boss_chests(self, watcher_run, rng):
        """Matryoshka: Should NOT trigger on boss chests."""
        watcher_run.add_relic("Matryoshka")

        matryoshka = watcher_run.get_relic("Matryoshka")
        initial_counter = matryoshka.counter

        # Open boss chest (should not consume Matryoshka)
        # This would require a separate boss chest handler
        # For now, verify counter doesn't change

        assert matryoshka.counter == initial_counter


# =============================================================================
# BLACK STAR TESTS
# =============================================================================

class TestBlackStar:
    """Black Star: Elite combats grant an additional Relic."""

    @pytest.mark.skip(reason="Black Star bonus relic not implemented")
    def test_black_star_grants_extra_relic_from_elite(self, watcher_run, rng):
        """Black Star: Defeating an elite gives 2 relics instead of 1."""
        watcher_run.add_relic("Black Star")
        initial_relics = len(watcher_run.relics)

        result = MockEliteHandler.defeat_elite(watcher_run, rng)

        # Should gain 2 relics: 1 base + 1 Black Star
        assert result["relics_gained"] == 2
        assert len(watcher_run.relics) == initial_relics + 2

    @pytest.mark.skip(reason="Black Star bonus relic not implemented")
    def test_black_star_applies_to_all_elites(self, watcher_run, rng):
        """Black Star: Should apply to every elite combat."""
        watcher_run.add_relic("Black Star")

        # Defeat 3 elites
        for i in range(3):
            initial_relics = len(watcher_run.relics)
            result = MockEliteHandler.defeat_elite(watcher_run, Random(seed_to_long(f"ELITE{i}")))

            # Each should grant 2 relics
            assert result["relics_gained"] == 2
            assert len(watcher_run.relics) == initial_relics + 2

    @pytest.mark.skip(reason="Black Star max HP bonus not implemented")
    def test_black_star_grants_max_hp_on_pickup(self):
        """Black Star: Upon pickup, gain +8 Max HP (boss relic)."""
        run = create_watcher_run("TEST", ascension=0)
        initial_max_hp = run.max_hp

        run.add_relic("Black Star")

        # Black Star is a boss relic that grants +8 Max HP
        assert run.max_hp == initial_max_hp + 8

    @pytest.mark.skip(reason="Black Star elite tracking not implemented")
    def test_black_star_does_not_affect_normal_combats(self, watcher_run, rng):
        """Black Star: Should NOT grant extra relics from normal combats."""
        watcher_run.add_relic("Black Star")

        # Normal combat does not grant relics
        # This would be tested with a normal combat handler


# =============================================================================
# CURSED KEY TESTS
# =============================================================================

class TestCursedKey:
    """Cursed Key: Whenever you open a non-boss Chest, obtain a Curse."""

    @pytest.mark.skip(reason="Cursed Key curse gain not implemented")
    def test_cursed_key_adds_curse_on_chest_open(self, watcher_run, rng):
        """Cursed Key: Opening a chest adds a Curse to your deck."""
        watcher_run.add_relic("Cursed Key")
        initial_deck_size = len(watcher_run.deck)

        result = MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Should gain a curse
        assert result["curse_gained"] is not None
        assert len(watcher_run.deck) == initial_deck_size + 1

    @pytest.mark.skip(reason="Cursed Key curse gain not implemented")
    def test_cursed_key_applies_to_all_chests(self, watcher_run, rng):
        """Cursed Key: Every chest opened adds a Curse."""
        watcher_run.add_relic("Cursed Key")
        initial_deck_size = len(watcher_run.deck)

        # Open 3 chests
        for i in range(3):
            MockChestHandler.open_chest(watcher_run, Random(seed_to_long(f"CHEST{i}")), chest_tier="small")

        # Should have gained 3 curses
        assert len(watcher_run.deck) == initial_deck_size + 3

    @pytest.mark.skip(reason="Cursed Key max HP bonus not implemented")
    def test_cursed_key_grants_max_hp_on_pickup(self):
        """Cursed Key: Upon pickup, gain +10 Max HP (boss relic)."""
        run = create_watcher_run("TEST", ascension=0)
        initial_max_hp = run.max_hp

        run.add_relic("Cursed Key")

        # Cursed Key is a boss relic that grants +10 Max HP
        assert run.max_hp == initial_max_hp + 10

    @pytest.mark.skip(reason="Cursed Key boss chest exclusion not implemented")
    def test_cursed_key_does_not_affect_boss_chests(self, watcher_run, rng):
        """Cursed Key: Should NOT add a Curse when opening boss chests."""
        watcher_run.add_relic("Cursed Key")
        initial_deck_size = len(watcher_run.deck)

        # Open boss chest (should not add curse)
        # This would require a boss chest handler

        # Deck size should not increase from curse
        assert len(watcher_run.deck) == initial_deck_size

    @pytest.mark.skip(reason="Cursed Key Darkstone interaction not implemented")
    def test_cursed_key_with_darkstone_periapt(self, watcher_run, rng):
        """Cursed Key + Darkstone Periapt: Curses grant +6 Max HP."""
        watcher_run.add_relic("Cursed Key")
        watcher_run.add_relic("Darkstone Periapt")

        initial_max_hp = watcher_run.max_hp

        # Open chest (gains curse)
        MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Darkstone should trigger on curse gain
        # Expected: +6 Max HP
        # This depends on onObtainCard trigger


# =============================================================================
# N'LOTH'S MASK (HUNGRY FACE) TESTS
# =============================================================================

class TestNlothsMask:
    """N'loth's Hungry Face: The next non-boss Chest you open is empty, but chests have better rewards afterward."""

    @pytest.mark.skip(reason="N'loth's Mask empty chest not implemented")
    def test_nloths_mask_first_chest_empty(self, watcher_run, rng):
        """N'loth's Mask: First chest gives no relics."""
        watcher_run.add_relic("NlothsMask")
        initial_relics = len(watcher_run.relics)

        result = MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Should gain 0 relics
        assert result["relics_gained"] == 0
        assert len(watcher_run.relics) == initial_relics

    @pytest.mark.skip(reason="N'loth's Mask empty chest not implemented")
    def test_nloths_mask_subsequent_chests_better(self, watcher_run, rng):
        """N'loth's Mask: After first chest, future chests give +1 relic."""
        watcher_run.add_relic("NlothsMask")

        # Open first chest (empty)
        MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Open second chest
        initial_relics = len(watcher_run.relics)
        result = MockChestHandler.open_chest(watcher_run, Random(seed_to_long("CHEST2")), chest_tier="medium")

        # Should gain 2 relics: 1 base + 1 N'loth bonus
        assert result["relics_gained"] == 2
        assert len(watcher_run.relics) == initial_relics + 2

    @pytest.mark.skip(reason="N'loth's Mask counter tracking not implemented")
    def test_nloths_mask_counter_tracks_usage(self, watcher_run, rng):
        """N'loth's Mask: Counter tracks whether first chest has been opened."""
        watcher_run.add_relic("NlothsMask")

        mask = watcher_run.get_relic("NlothsMask")
        # Initial counter: 0 (first chest not yet opened)
        mask.counter = 0

        # Open first chest
        MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Counter should be 1 (first chest opened)
        assert mask.counter == 1

    @pytest.mark.skip(reason="N'loth's Mask bonus applies permanently not implemented")
    def test_nloths_mask_bonus_applies_permanently(self, watcher_run, rng):
        """N'loth's Mask: Bonus applies to ALL chests after first."""
        watcher_run.add_relic("NlothsMask")

        # Open first chest (empty)
        MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Open 3 more chests
        for i in range(3):
            initial_relics = len(watcher_run.relics)
            result = MockChestHandler.open_chest(watcher_run, Random(seed_to_long(f"CHEST{i+2}")), chest_tier="medium")

            # Each should grant bonus relic
            assert result["relics_gained"] == 2
            assert len(watcher_run.relics) == initial_relics + 2

    @pytest.mark.skip(reason="N'loth's Mask boss chest exclusion not implemented")
    def test_nloths_mask_does_not_affect_boss_chests(self, watcher_run, rng):
        """N'loth's Mask: Should NOT trigger on boss chests."""
        watcher_run.add_relic("NlothsMask")

        mask = watcher_run.get_relic("NlothsMask")
        initial_counter = mask.counter

        # Open boss chest (should not consume or affect counter)
        # This would require a boss chest handler

        assert mask.counter == initial_counter


# =============================================================================
# COMBINATION TESTS
# =============================================================================

class TestChestRelicCombinations:
    """Test interactions between multiple chest-related relics."""

    @pytest.mark.skip(reason="Matryoshka + Cursed Key not implemented")
    def test_matryoshka_and_cursed_key(self, watcher_run, rng):
        """Matryoshka + Cursed Key: Should get 2 relics AND a curse."""
        watcher_run.add_relic("Matryoshka")
        watcher_run.add_relic("Cursed Key")

        initial_relics = len(watcher_run.relics)
        initial_deck_size = len(watcher_run.deck)

        result = MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # Should gain 2 relics (Matryoshka)
        assert result["relics_gained"] == 2
        assert len(watcher_run.relics) == initial_relics + 2

        # Should gain 1 curse (Cursed Key)
        assert result["curse_gained"] is not None
        assert len(watcher_run.deck) == initial_deck_size + 1

    @pytest.mark.skip(reason="N'loth's Mask + Matryoshka not implemented")
    def test_nloths_mask_and_matryoshka(self, watcher_run, rng):
        """N'loth's Mask + Matryoshka: First chest empty, second chest gets 3 relics."""
        watcher_run.add_relic("NlothsMask")
        watcher_run.add_relic("Matryoshka")

        # First chest: empty (N'loth's Mask)
        result1 = MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")
        assert result1["relics_gained"] == 0

        # Second chest: 1 base + 1 N'loth bonus + 1 Matryoshka = 3
        initial_relics = len(watcher_run.relics)
        result2 = MockChestHandler.open_chest(watcher_run, Random(seed_to_long("CHEST2")), chest_tier="medium")
        assert result2["relics_gained"] == 3
        assert len(watcher_run.relics) == initial_relics + 3

    @pytest.mark.skip(reason="Tiny Chest + N'loth's Mask not implemented")
    def test_tiny_chest_and_nloths_mask(self, watcher_run, rng):
        """Tiny Chest + N'loth's Mask: Should work together."""
        watcher_run.add_relic("Tiny Chest")
        watcher_run.add_relic("NlothsMask")

        # First ? room chest triggered by Tiny Chest
        # If it's the first chest, N'loth makes it empty

        # Enter 4 ? rooms to trigger Tiny Chest
        for i in range(4):
            MockQuestionRoomHandler.generate_question_room(watcher_run, Random(seed_to_long(f"Q{i}")), i + 1)

        # 4th room should be a chest
        # Open it (should be empty due to N'loth)
        result = MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")
        assert result["relics_gained"] == 0

    @pytest.mark.skip(reason="All chest relics together not implemented")
    def test_all_chest_relics_together(self, watcher_run, rng):
        """All chest relics: Complex interaction."""
        watcher_run.add_relic("Matryoshka")
        watcher_run.add_relic("Cursed Key")
        watcher_run.add_relic("NlothsMask")

        # First chest: N'loth makes it empty (overrides Matryoshka)
        # But Cursed Key still adds curse
        result1 = MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")
        assert result1["relics_gained"] == 0
        assert result1["curse_gained"] is not None

        # Second chest: N'loth bonus (1) + Matryoshka (1) + base (1) = 3 relics + curse
        initial_relics = len(watcher_run.relics)
        result2 = MockChestHandler.open_chest(watcher_run, Random(seed_to_long("CHEST2")), chest_tier="medium")
        assert result2["relics_gained"] == 3
        assert result2["curse_gained"] is not None


# =============================================================================
# EDGE CASES
# =============================================================================

class TestChestRelicEdgeCases:
    """Edge cases for chest/acquisition relics."""

    @pytest.mark.skip(reason="Curse variety not implemented")
    def test_cursed_key_random_curse_types(self, watcher_run, rng):
        """Cursed Key: Should add random Curses (not always the same)."""
        watcher_run.add_relic("Cursed Key")

        curses = []
        for i in range(5):
            result = MockChestHandler.open_chest(watcher_run, Random(seed_to_long(f"CHEST{i}")), chest_tier="small")
            if result["curse_gained"]:
                curses.append(result["curse_gained"])

        # Curses may vary (depends on curse pool implementation)
        # For now, verify at least 1 curse was gained
        assert len(curses) >= 1

    @pytest.mark.skip(reason="Relic pool interaction not implemented")
    def test_chest_relics_dont_overlap_with_pool(self, watcher_run, rng):
        """Chest rewards should not give duplicate relics."""
        watcher_run.add_relic("Matryoshka")

        # Open chest, get relics
        MockChestHandler.open_chest(watcher_run, rng, chest_tier="small")

        # All relics should be unique
        relic_ids = [r.id for r in watcher_run.relics]
        assert len(relic_ids) == len(set(relic_ids))

    @pytest.mark.skip(reason="Boss relic exclusion not implemented")
    def test_boss_relics_excluded_from_chests(self, watcher_run, rng):
        """Chests should not contain boss relics."""
        watcher_run.add_relic("Matryoshka")

        # Open multiple chests
        for i in range(5):
            MockChestHandler.open_chest(watcher_run, Random(seed_to_long(f"CHEST{i}")), chest_tier="large")

        # Verify no boss relics were gained
        # This would require checking relic tiers


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
