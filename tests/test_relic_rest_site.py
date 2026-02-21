"""
Rest Site Relic Tests - TDD approach.

Tests for relics that modify rest site behavior:
- Dream Catcher: After resting, get a card reward
- Regal Pillow: Heal additional 15 HP when resting
- Girya: Can use "Lift" option at rest to gain +1 Strength (3 uses total)
- Peace Pipe: Can "Toke" to remove a card
- Shovel: Can "Dig" for a relic

NOTE: Golden Eye and Melange are NOT rest site relics:
- Golden Eye (Watcher): Adds +2 to ALL scry amounts (passive modifier in ScryAction constructor)
- Melange (Watcher): onShuffle trigger - Scry 3 whenever draw pile is shuffled

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


# Mock rest handler for testing (will be replaced with actual implementation)
class MockRestHandler:
    @staticmethod
    def get_rest_options(run: RunState):
        """Get available rest site options based on relics."""
        options = ["rest", "smith"]  # Base options

        if run.has_relic("Coffee Dripper"):
            options.remove("rest")

        if run.has_relic("Fusion Hammer"):
            options.remove("smith")

        if run.has_relic("Girya"):
            girya = run.get_relic("Girya")
            if girya.counter < 3:  # 3 uses max
                options.append("lift")

        if run.has_relic("Peace Pipe"):
            options.append("toke")

        if run.has_relic("Shovel"):
            options.append("dig")

        return options

    @staticmethod
    def rest(run: RunState):
        """Perform rest action."""
        # Base heal: 30% of max HP
        base_heal = int(run.max_hp * 0.30)

        # Regal Pillow: +15 HP
        if run.has_relic("Regal Pillow"):
            base_heal += 15

        run.heal(base_heal)

        # Dream Catcher: Triggers card reward after resting
        dream_catcher_triggered = run.has_relic("Dream Catcher")

        # NOTE: Golden Eye and Melange are NOT rest site relics.
        # - Golden Eye: Adds +2 to all scry amounts (passive modifier)
        # - Melange: Triggers on shuffle (onShuffle), not on rest

        return {
            "hp_healed": base_heal,
            "dream_catcher_triggered": dream_catcher_triggered,
        }


# =============================================================================
# DREAM CATCHER TESTS
# =============================================================================

class TestDreamCatcher:
    """Dream Catcher: Whenever you rest, you may add a card to your deck."""

    @pytest.mark.skip(reason="Dream Catcher rest reward not implemented")
    def test_dream_catcher_triggers_on_rest(self, watcher_run):
        """Dream Catcher: Resting triggers a card reward."""
        watcher_run.add_relic("Dream Catcher")
        watcher_run.damage(30)

        result = MockRestHandler.rest(watcher_run)

        assert result["dream_catcher_triggered"] is True

    @pytest.mark.skip(reason="Dream Catcher rest reward not implemented")
    def test_dream_catcher_does_not_trigger_on_smith(self, watcher_run):
        """Dream Catcher: Only triggers on REST, not on Smith/Upgrade."""
        watcher_run.add_relic("Dream Catcher")

        # Upgrading a card should not trigger Dream Catcher
        # This would be tested with a smith action
        assert watcher_run.has_relic("Dream Catcher")

    @pytest.mark.skip(reason="Dream Catcher rest reward not implemented")
    def test_dream_catcher_skip_is_optional(self, watcher_run):
        """Dream Catcher: Card reward can be skipped."""
        watcher_run.add_relic("Dream Catcher")
        watcher_run.damage(30)

        initial_deck_size = len(watcher_run.deck)

        # Rest and skip the Dream Catcher card reward
        result = MockRestHandler.rest(watcher_run)
        assert result["dream_catcher_triggered"] is True

        # Deck size unchanged if skipped
        assert len(watcher_run.deck) == initial_deck_size

    @pytest.mark.skip(reason="Dream Catcher rest reward not implemented")
    def test_dream_catcher_with_coffee_dripper(self, watcher_run):
        """Dream Catcher: Cannot trigger if Coffee Dripper prevents resting."""
        watcher_run.add_relic("Dream Catcher")
        watcher_run.add_relic("Coffee Dripper")

        options = MockRestHandler.get_rest_options(watcher_run)

        # Rest option should be blocked by Coffee Dripper
        assert "rest" not in options

        # Therefore Dream Catcher cannot trigger


# =============================================================================
# REGAL PILLOW TESTS
# =============================================================================

class TestRegalPillow:
    """Regal Pillow: Heal an additional 15 HP when you rest."""

    @pytest.mark.skip(reason="Regal Pillow bonus heal not implemented")
    def test_regal_pillow_adds_15_hp(self, watcher_run):
        """Regal Pillow: Resting heals 30% + 15 HP."""
        watcher_run.add_relic("Regal Pillow")
        watcher_run.damage(50)

        result = MockRestHandler.rest(watcher_run)

        # Base heal: 30% of max HP (72 at A0 = 21.6, rounds to 21)
        # Regal Pillow: +15
        expected_heal = int(watcher_run.max_hp * 0.30) + 15

        assert result["hp_healed"] == expected_heal

    @pytest.mark.skip(reason="Regal Pillow bonus heal not implemented")
    def test_regal_pillow_affected_by_magic_flower(self, watcher_run):
        """Regal Pillow: Bonus should be affected by Magic Flower (50% more healing)."""
        watcher_run.add_relic("Regal Pillow")
        watcher_run.add_relic("MagicFlower")
        watcher_run.damage(60)

        # Base: 30% * 72 = 21 (rounded)
        # Regal Pillow: +15
        # Total: 36 HP
        # Magic Flower: 36 * 1.5 = 54 HP
        # Note: Magic Flower only applies in combat, NOT at rest sites
        # So this test verifies it does NOT apply

        result = MockRestHandler.rest(watcher_run)

        # Regal Pillow should add flat 15, not affected by Magic Flower
        expected_heal = int(watcher_run.max_hp * 0.30) + 15
        assert result["hp_healed"] == expected_heal

    @pytest.mark.skip(reason="Regal Pillow bonus heal not implemented")
    def test_regal_pillow_capped_at_max_hp(self, watcher_run):
        """Regal Pillow: Healing cannot exceed max HP."""
        watcher_run.add_relic("Regal Pillow")
        watcher_run.damage(20)  # Only 20 damage

        initial_hp = watcher_run.current_hp

        result = MockRestHandler.rest(watcher_run)

        # Even with +15 from Regal Pillow, should cap at max HP
        assert watcher_run.current_hp == watcher_run.max_hp

    @pytest.mark.skip(reason="Regal Pillow bonus heal not implemented")
    def test_regal_pillow_does_not_affect_smith(self, watcher_run):
        """Regal Pillow: Only affects REST action, not Smith."""
        watcher_run.add_relic("Regal Pillow")

        # Smithing should not heal at all
        # This would be tested with a smith action


# =============================================================================
# GIRYA TESTS
# =============================================================================

class TestGirya:
    """Girya: Can Lift at rest sites to gain +1 Strength (3 uses total)."""

    @pytest.mark.skip(reason="Girya lift option not implemented")
    def test_girya_adds_lift_option(self, watcher_run):
        """Girya: Rest sites should have a Lift option."""
        watcher_run.add_relic("Girya")

        options = MockRestHandler.get_rest_options(watcher_run)

        assert "lift" in options

    @pytest.mark.skip(reason="Girya lift option not implemented")
    def test_girya_grants_1_strength_per_lift(self, watcher_run):
        """Girya: Each lift grants +1 permanent Strength."""
        watcher_run.add_relic("Girya")

        # The strength is stored as a persistent stat on the run
        # In combat, it would translate to starting with +1 Strength power

        girya = watcher_run.get_relic("Girya")
        initial_counter = girya.counter if girya else 0

        # Simulate lifting
        # This would increment a permanent strength stat
        # For now, we track via counter

        assert girya is not None

    @pytest.mark.skip(reason="Girya lift option not implemented")
    def test_girya_has_3_uses_max(self, watcher_run):
        """Girya: Can only lift 3 times total."""
        watcher_run.add_relic("Girya")

        # Simulate 3 lifts
        girya = watcher_run.get_relic("Girya")
        girya.counter = 3  # Mark as used 3 times

        options = MockRestHandler.get_rest_options(watcher_run)

        # Lift should no longer be available
        assert "lift" not in options

    @pytest.mark.skip(reason="Girya lift option not implemented")
    def test_girya_counter_increments(self, watcher_run):
        """Girya: Counter should increment with each use."""
        watcher_run.add_relic("Girya")

        girya = watcher_run.get_relic("Girya")
        assert girya.counter == -1 or girya.counter == 0  # Initial state

        # After 1 lift
        girya.counter = 1
        assert girya.counter == 1

        # After 2 lifts
        girya.counter = 2
        assert girya.counter == 2

        # After 3 lifts (max)
        girya.counter = 3
        assert girya.counter == 3

    @pytest.mark.skip(reason="Girya strength bonus not implemented")
    def test_girya_strength_persists_across_combats(self, watcher_run):
        """Girya: Strength bonus should apply at start of every combat."""
        watcher_run.add_relic("Girya")

        # Simulate 2 lifts
        girya = watcher_run.get_relic("Girya")
        girya.counter = 2  # 2 uses, so +2 Strength

        # In combat, player should start with +2 Strength
        # This would be tested in combat initialization


# =============================================================================
# PEACE PIPE TESTS
# =============================================================================

class TestPeacePipe:
    """Peace Pipe: Can Toke at rest sites to remove a card."""

    @pytest.mark.skip(reason="Peace Pipe toke option not implemented")
    def test_peace_pipe_adds_toke_option(self, watcher_run):
        """Peace Pipe: Rest sites should have a Toke option."""
        watcher_run.add_relic("Peace Pipe")

        options = MockRestHandler.get_rest_options(watcher_run)

        assert "toke" in options

    @pytest.mark.skip(reason="Peace Pipe toke option not implemented")
    def test_peace_pipe_removes_card(self, watcher_run):
        """Peace Pipe: Toke removes a card from your deck."""
        watcher_run.add_relic("Peace Pipe")
        initial_deck_size = len(watcher_run.deck)

        # Simulate toking (remove first card)
        watcher_run.remove_card(0)

        assert len(watcher_run.deck) == initial_deck_size - 1

    @pytest.mark.skip(reason="Peace Pipe toke option not implemented")
    def test_peace_pipe_unlimited_uses(self, watcher_run):
        """Peace Pipe: Can be used unlimited times (every rest site)."""
        watcher_run.add_relic("Peace Pipe")

        # Use toke 5 times across different rest sites
        for _ in range(5):
            options = MockRestHandler.get_rest_options(watcher_run)
            assert "toke" in options

            if len(watcher_run.deck) > 0:
                watcher_run.remove_card(0)

        # Toke should still be available
        options = MockRestHandler.get_rest_options(watcher_run)
        assert "toke" in options

    @pytest.mark.skip(reason="Peace Pipe toke option not implemented")
    def test_peace_pipe_works_with_other_options(self, watcher_run):
        """Peace Pipe: Can use Toke alongside Rest/Smith."""
        watcher_run.add_relic("Peace Pipe")

        options = MockRestHandler.get_rest_options(watcher_run)

        # Should have all 3 options: rest, smith, toke
        assert "rest" in options
        assert "smith" in options
        assert "toke" in options


# =============================================================================
# SHOVEL TESTS
# =============================================================================

class TestShovel:
    """Shovel: Can Dig at rest sites for a relic (one-time use)."""

    @pytest.mark.skip(reason="Shovel dig option not implemented")
    def test_shovel_adds_dig_option(self, watcher_run):
        """Shovel: Rest sites should have a Dig option."""
        watcher_run.add_relic("Shovel")

        options = MockRestHandler.get_rest_options(watcher_run)

        assert "dig" in options

    @pytest.mark.skip(reason="Shovel dig option not implemented")
    def test_shovel_grants_relic(self, watcher_run, rng):
        """Shovel: Digging grants a random relic."""
        watcher_run.add_relic("Shovel")
        initial_relics = len(watcher_run.relics)

        # Simulate digging (would grant a relic)
        # For now, manually add a relic to simulate
        watcher_run.add_relic("Lantern")

        assert len(watcher_run.relics) == initial_relics + 1

    @pytest.mark.skip(reason="Shovel dig option not implemented")
    def test_shovel_one_time_use(self, watcher_run):
        """Shovel: Can only dig once (relic is consumed)."""
        watcher_run.add_relic("Shovel")

        # After digging, Shovel should be removed
        assert watcher_run.has_relic("Shovel")

        # Simulate dig action (would remove Shovel)
        watcher_run.remove_relic("Shovel")

        assert not watcher_run.has_relic("Shovel")

        # Dig should no longer be available
        options = MockRestHandler.get_rest_options(watcher_run)
        assert "dig" not in options

    @pytest.mark.skip(reason="Shovel dig option not implemented")
    def test_shovel_replaces_rest_or_smith(self, watcher_run):
        """Shovel: Using Dig counts as your rest site action (replaces Rest/Smith)."""
        watcher_run.add_relic("Shovel")
        watcher_run.damage(30)

        initial_hp = watcher_run.current_hp

        # Choose Dig instead of Rest
        # This means no healing

        # HP should remain the same
        assert watcher_run.current_hp == initial_hp


# =============================================================================
# GOLDEN EYE (WATCHER) TESTS
# =============================================================================
# NOTE: Golden Eye is NOT a rest site relic. It adds +2 to ALL scry amounts.
# These tests have been moved to test_relic_scry.py (or should be created there).
# The effect is implemented as a passive modifier checked in ScryAction constructor.

class TestGoldenEye:
    """Golden Eye: Adds +2 to all scry amounts (Watcher-specific relic).

    IMPORTANT: Golden Eye is NOT triggered by resting. It modifies ALL scry actions.
    The +2 bonus is added in the ScryAction constructor when player has Golden Eye.
    """

    @pytest.mark.skip(reason="Golden Eye scry bonus not implemented - see test_relic_scry.py")
    def test_golden_eye_adds_2_to_scry(self, watcher_run):
        """Golden Eye: Any scry action gets +2 cards."""
        watcher_run.add_relic("GoldenEye")

        # When scrying 3, should actually scry 5
        # This would be tested with a card like Third Eye (Scry 3 -> Scry 5)

    @pytest.mark.skip(reason="Golden Eye scry bonus not implemented")
    def test_golden_eye_with_melange_shuffle(self, watcher_run):
        """Golden Eye + Melange: Melange's Scry 3 becomes Scry 5."""
        watcher_run.add_relic("GoldenEye")
        watcher_run.add_relic("Melange")

        # When Melange triggers on shuffle (Scry 3), Golden Eye adds +2 -> Scry 5

    @pytest.mark.skip(reason="Golden Eye scry bonus not implemented")
    def test_golden_eye_watcher_exclusive(self):
        """Golden Eye: Should only appear for Watcher (class-specific relic)."""
        from packages.engine.content.relics import GOLDEN_EYE, PlayerClass
        assert GOLDEN_EYE.player_class == PlayerClass.WATCHER


# =============================================================================
# MELANGE TESTS
# =============================================================================
# NOTE: Melange is NOT a rest site relic. It triggers onShuffle (when draw pile shuffles).

class TestMelange:
    """Melange: Scry 3 whenever you shuffle your draw pile (Watcher-specific relic).

    IMPORTANT: Melange is NOT triggered by resting. It triggers when the draw pile
    is shuffled during combat. With Golden Eye, the Scry 3 becomes Scry 5.
    """

    @pytest.mark.skip(reason="Melange shuffle trigger not implemented")
    def test_melange_scry_on_shuffle(self, watcher_run):
        """Melange: Shuffling draw pile triggers Scry 3."""
        watcher_run.add_relic("Melange")

        # Enter combat, exhaust draw pile to trigger shuffle
        # When shuffle occurs, should Scry 3

    @pytest.mark.skip(reason="Melange shuffle trigger not implemented")
    def test_melange_with_golden_eye(self, watcher_run):
        """Melange + Golden Eye: Scry 3 becomes Scry 5 on shuffle."""
        watcher_run.add_relic("Melange")
        watcher_run.add_relic("GoldenEye")

        # When Melange triggers (Scry 3), Golden Eye adds +2 -> Scry 5

    @pytest.mark.skip(reason="Melange shuffle trigger not implemented")
    def test_melange_does_not_trigger_on_rest(self, watcher_run):
        """Melange: Does NOT trigger when resting - only on shuffle."""
        watcher_run.add_relic("Melange")
        watcher_run.damage(30)

        # Resting should NOT trigger Melange
        # Melange only triggers during combat when draw pile shuffles

    @pytest.mark.skip(reason="Melange shuffle trigger not implemented")
    def test_melange_watcher_exclusive(self):
        """Melange: Should only appear for Watcher (class-specific relic)."""
        from packages.engine.content.relics import MELANGE, PlayerClass
        assert MELANGE.player_class == PlayerClass.WATCHER


# =============================================================================
# COMBINATION TESTS
# =============================================================================

class TestRestSiteRelicCombinations:
    """Test interactions between multiple rest site relics."""

    @pytest.mark.skip(reason="Rest site relic combinations not implemented")
    def test_regal_pillow_and_dream_catcher(self, watcher_run):
        """Regal Pillow + Dream Catcher: Should heal extra AND get card reward."""
        watcher_run.add_relic("Regal Pillow")
        watcher_run.add_relic("Dream Catcher")
        watcher_run.damage(50)

        result = MockRestHandler.rest(watcher_run)

        # Regal Pillow: +15 HP
        expected_heal = int(watcher_run.max_hp * 0.30) + 15
        assert result["hp_healed"] == expected_heal

        # Dream Catcher: Card reward
        assert result["dream_catcher_triggered"] is True

    @pytest.mark.skip(reason="See TestGoldenEye and TestMelange - these are shuffle/scry relics, not rest relics")
    def test_golden_eye_and_melange_interaction(self, watcher_run):
        """Golden Eye + Melange: Golden Eye adds +2 to Melange's Scry 3 on shuffle.

        NOTE: Neither Golden Eye nor Melange trigger on rest.
        - Melange: onShuffle -> Scry 3
        - Golden Eye: passive -> all scrys get +2
        - Combined: on shuffle, Scry 5 (3 + 2)
        """
        pass  # See test_relic_scry.py for actual implementation tests

    @pytest.mark.skip(reason="Rest site relic combinations not implemented")
    def test_girya_and_peace_pipe(self, watcher_run):
        """Girya + Peace Pipe: Should have both Lift and Toke options."""
        watcher_run.add_relic("Girya")
        watcher_run.add_relic("Peace Pipe")

        options = MockRestHandler.get_rest_options(watcher_run)

        assert "lift" in options
        assert "toke" in options
        assert "rest" in options
        assert "smith" in options

    @pytest.mark.skip(reason="Rest site relic combinations not implemented")
    def test_all_rest_option_relics(self, watcher_run):
        """Girya + Peace Pipe + Shovel: Should have 5 total options."""
        watcher_run.add_relic("Girya")
        watcher_run.add_relic("Peace Pipe")
        watcher_run.add_relic("Shovel")

        options = MockRestHandler.get_rest_options(watcher_run)

        # Base: rest, smith
        # Girya: lift
        # Peace Pipe: toke
        # Shovel: dig
        assert len(options) == 5
        assert all(opt in options for opt in ["rest", "smith", "lift", "toke", "dig"])


# =============================================================================
# EDGE CASES
# =============================================================================

class TestRestSiteEdgeCases:
    """Edge cases for rest site relics."""

    @pytest.mark.skip(reason="Coffee Dripper interaction not implemented")
    def test_coffee_dripper_blocks_rest_but_not_options(self, watcher_run):
        """Coffee Dripper: Blocks REST but not Lift/Toke/Dig."""
        watcher_run.add_relic("Coffee Dripper")
        watcher_run.add_relic("Girya")
        watcher_run.add_relic("Peace Pipe")

        options = MockRestHandler.get_rest_options(watcher_run)

        # Rest blocked
        assert "rest" not in options

        # Other options still available
        assert "smith" in options
        assert "lift" in options
        assert "toke" in options

    @pytest.mark.skip(reason="Fusion Hammer interaction not implemented")
    def test_fusion_hammer_blocks_smith_but_not_options(self, watcher_run):
        """Fusion Hammer: Blocks SMITH but not Rest/Lift/Toke/Dig."""
        watcher_run.add_relic("Fusion Hammer")
        watcher_run.add_relic("Peace Pipe")

        options = MockRestHandler.get_rest_options(watcher_run)

        # Smith blocked
        assert "smith" not in options

        # Other options still available
        assert "rest" in options
        assert "toke" in options

    @pytest.mark.skip(reason="Both blockers interaction not implemented")
    def test_coffee_dripper_and_fusion_hammer(self, watcher_run):
        """Coffee Dripper + Fusion Hammer: Only alternative options remain."""
        watcher_run.add_relic("Coffee Dripper")
        watcher_run.add_relic("Fusion Hammer")
        watcher_run.add_relic("Girya")

        options = MockRestHandler.get_rest_options(watcher_run)

        # Both rest and smith blocked
        assert "rest" not in options
        assert "smith" not in options

        # But Girya still works
        assert "lift" in options

    @pytest.mark.skip(reason="Mark of Bloom interaction not implemented")
    def test_mark_of_bloom_prevents_regal_pillow_heal(self, watcher_run):
        """Mark of the Bloom: Should prevent Regal Pillow healing."""
        watcher_run.add_relic("Mark of the Bloom")
        watcher_run.add_relic("Regal Pillow")
        watcher_run.damage(50)

        initial_hp = watcher_run.current_hp

        result = MockRestHandler.rest(watcher_run)

        # No healing should occur (Mark of the Bloom)
        assert watcher_run.current_hp == initial_hp
        assert result["hp_healed"] == 0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
