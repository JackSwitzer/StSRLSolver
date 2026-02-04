"""
Bottled Relic Tests - TDD approach.

Tests for Bottled relics that make cards Innate:
- Bottled Flame: At pickup, choose an Attack to become Innate
- Bottled Lightning: At pickup, choose a Skill to become Innate
- Bottled Tornado: At pickup, choose a Power to become Innate

Innate cards are drawn at the start of combat.

These are failing tests that document expected behavior.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.state.run import RunState, create_watcher_run
from packages.engine.state.combat import create_combat, create_enemy
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.content.cards import ALL_CARDS, CardType


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


def has_innate_card(run: RunState, card_id: str) -> bool:
    """Check if a card is marked as Innate in the deck."""
    # In Java, Bottled relics set a card ID in the relic
    # In Python, we might track this differently
    # For now, check if the relic has the card ID stored

    bottled_relics = ["Bottled Flame", "Bottled Lightning", "Bottled Tornado"]

    for relic_id in bottled_relics:
        relic = run.get_relic(relic_id)
        if relic and hasattr(relic, "card_id") and relic.card_id == card_id:
            return True

    return False


def get_bottled_card_id(run: RunState, relic_id: str) -> str:
    """Get the card ID that's bottled in a relic."""
    relic = run.get_relic(relic_id)
    if relic and hasattr(relic, "card_id"):
        return relic.card_id
    return None


# =============================================================================
# BOTTLED FLAME TESTS
# =============================================================================

class TestBottledFlame:
    """Bottled Flame: Upon pickup, choose an Attack to become Innate."""

    @pytest.mark.skip(reason="Bottled Flame selection not implemented")
    def test_bottled_flame_allows_attack_selection(self, watcher_run):
        """Bottled Flame: Pickup presents a choice of Attacks to bottle."""
        # Add some Attacks to choose from
        watcher_run.add_card("Strike_P")
        watcher_run.add_card("Eruption")
        watcher_run.add_card("Tantrum")

        watcher_run.add_relic("Bottled Flame")

        # Should have selected an Attack
        bottled_card = get_bottled_card_id(watcher_run, "Bottled Flame")
        assert bottled_card is not None

        # Verify it's an Attack
        if bottled_card in ALL_CARDS:
            assert ALL_CARDS[bottled_card].card_type == CardType.ATTACK

    @pytest.mark.skip(reason="Bottled Flame selection not implemented")
    def test_bottled_flame_only_shows_attacks(self, watcher_run):
        """Bottled Flame: Should only show Attacks as options, not Skills/Powers."""
        watcher_run.add_card("Strike_P")  # Attack
        watcher_run.add_card("Vigilance")  # Skill
        watcher_run.add_card("Meditate")  # Power

        watcher_run.add_relic("Bottled Flame")

        bottled_card = get_bottled_card_id(watcher_run, "Bottled Flame")

        # Should have bottled Strike_P (the only Attack added)
        assert bottled_card == "Strike_P"

    @pytest.mark.skip(reason="Bottled Flame innate behavior not implemented")
    def test_bottled_flame_card_starts_in_hand(self, watcher_run):
        """Bottled Flame: Bottled Attack starts in hand at combat start."""
        watcher_run.add_card("Tantrum")

        watcher_run.add_relic("Bottled Flame")

        # Simulate bottling Tantrum
        relic = watcher_run.get_relic("Bottled Flame")
        if relic:
            relic.card_id = "Tantrum"

        # Create combat
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # Tantrum should be in starting hand
        assert "Tantrum" in combat.hand

    @pytest.mark.skip(reason="Bottled Flame selection not implemented")
    def test_bottled_flame_with_no_attacks(self, watcher_run):
        """Bottled Flame: Should not activate if deck has no Attacks."""
        # Remove all Attacks from deck
        while True:
            found_attack = False
            for i, card in enumerate(watcher_run.deck):
                if card.id in ALL_CARDS and ALL_CARDS[card.id].card_type == CardType.ATTACK:
                    watcher_run.remove_card(i)
                    found_attack = True
                    break
            if not found_attack:
                break

        watcher_run.add_relic("Bottled Flame")

        # Should not have bottled anything
        bottled_card = get_bottled_card_id(watcher_run, "Bottled Flame")
        assert bottled_card is None

    @pytest.mark.skip(reason="Bottled Flame selection not implemented")
    def test_bottled_flame_preserved_on_save_load(self, watcher_run):
        """Bottled Flame: Bottled card should be preserved across save/load."""
        watcher_run.add_card("Tantrum")
        watcher_run.add_relic("Bottled Flame")

        relic = watcher_run.get_relic("Bottled Flame")
        if relic:
            relic.card_id = "Tantrum"

        # Serialize and deserialize
        save_dict = watcher_run.to_dict()
        loaded_run = RunState.from_dict(save_dict)

        # Bottled card should be preserved
        loaded_relic = loaded_run.get_relic("Bottled Flame")
        assert loaded_relic.card_id == "Tantrum"


# =============================================================================
# BOTTLED LIGHTNING TESTS
# =============================================================================

class TestBottledLightning:
    """Bottled Lightning: Upon pickup, choose a Skill to become Innate."""

    @pytest.mark.skip(reason="Bottled Lightning selection not implemented")
    def test_bottled_lightning_allows_skill_selection(self, watcher_run):
        """Bottled Lightning: Pickup presents a choice of Skills to bottle."""
        watcher_run.add_card("Vigilance")
        watcher_run.add_card("Defend_P")
        watcher_run.add_card("Tranquility")

        watcher_run.add_relic("Bottled Lightning")

        # Should have selected a Skill
        bottled_card = get_bottled_card_id(watcher_run, "Bottled Lightning")
        assert bottled_card is not None

        # Verify it's a Skill
        if bottled_card in ALL_CARDS:
            assert ALL_CARDS[bottled_card].card_type == CardType.SKILL

    @pytest.mark.skip(reason="Bottled Lightning selection not implemented")
    def test_bottled_lightning_only_shows_skills(self, watcher_run):
        """Bottled Lightning: Should only show Skills as options."""
        watcher_run.add_card("Strike_P")  # Attack
        watcher_run.add_card("Vigilance")  # Skill
        watcher_run.add_card("Meditate")  # Power

        watcher_run.add_relic("Bottled Lightning")

        bottled_card = get_bottled_card_id(watcher_run, "Bottled Lightning")

        # Should have bottled Vigilance (the only Skill)
        assert bottled_card == "Vigilance"

    @pytest.mark.skip(reason="Bottled Lightning innate behavior not implemented")
    def test_bottled_lightning_card_starts_in_hand(self, watcher_run):
        """Bottled Lightning: Bottled Skill starts in hand at combat start."""
        watcher_run.add_card("Vigilance")
        watcher_run.add_relic("Bottled Lightning")

        # Simulate bottling Vigilance
        relic = watcher_run.get_relic("Bottled Lightning")
        if relic:
            relic.card_id = "Vigilance"

        # Create combat
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # Vigilance should be in starting hand
        assert "Vigilance" in combat.hand

    @pytest.mark.skip(reason="Bottled Lightning selection not implemented")
    def test_bottled_lightning_with_no_skills(self, watcher_run):
        """Bottled Lightning: Should not activate if deck has no Skills."""
        # Remove all Skills from deck
        while True:
            found_skill = False
            for i, card in enumerate(watcher_run.deck):
                if card.id in ALL_CARDS and ALL_CARDS[card.id].card_type == CardType.SKILL:
                    watcher_run.remove_card(i)
                    found_skill = True
                    break
            if not found_skill:
                break

        watcher_run.add_relic("Bottled Lightning")

        # Should not have bottled anything
        bottled_card = get_bottled_card_id(watcher_run, "Bottled Lightning")
        assert bottled_card is None


# =============================================================================
# BOTTLED TORNADO TESTS
# =============================================================================

class TestBottledTornado:
    """Bottled Tornado: Upon pickup, choose a Power to become Innate."""

    @pytest.mark.skip(reason="Bottled Tornado selection not implemented")
    def test_bottled_tornado_allows_power_selection(self, watcher_run):
        """Bottled Tornado: Pickup presents a choice of Powers to bottle."""
        watcher_run.add_card("Meditate")
        watcher_run.add_card("Worship")
        watcher_run.add_card("Nirvana")

        watcher_run.add_relic("Bottled Tornado")

        # Should have selected a Power
        bottled_card = get_bottled_card_id(watcher_run, "Bottled Tornado")
        assert bottled_card is not None

        # Verify it's a Power
        if bottled_card in ALL_CARDS:
            assert ALL_CARDS[bottled_card].card_type == CardType.POWER

    @pytest.mark.skip(reason="Bottled Tornado selection not implemented")
    def test_bottled_tornado_only_shows_powers(self, watcher_run):
        """Bottled Tornado: Should only show Powers as options."""
        watcher_run.add_card("Strike_P")  # Attack
        watcher_run.add_card("Vigilance")  # Skill
        watcher_run.add_card("Meditate")  # Power

        watcher_run.add_relic("Bottled Tornado")

        bottled_card = get_bottled_card_id(watcher_run, "Bottled Tornado")

        # Should have bottled Meditate (the only Power)
        assert bottled_card == "Meditate"

    @pytest.mark.skip(reason="Bottled Tornado innate behavior not implemented")
    def test_bottled_tornado_card_starts_in_hand(self, watcher_run):
        """Bottled Tornado: Bottled Power starts in hand at combat start."""
        watcher_run.add_card("Meditate")
        watcher_run.add_relic("Bottled Tornado")

        # Simulate bottling Meditate
        relic = watcher_run.get_relic("Bottled Tornado")
        if relic:
            relic.card_id = "Meditate"

        # Create combat
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # Meditate should be in starting hand
        assert "Meditate" in combat.hand

    @pytest.mark.skip(reason="Bottled Tornado selection not implemented")
    def test_bottled_tornado_with_no_powers(self, watcher_run):
        """Bottled Tornado: Should not activate if deck has no Powers."""
        # Remove all Powers from deck
        while True:
            found_power = False
            for i, card in enumerate(watcher_run.deck):
                if card.id in ALL_CARDS and ALL_CARDS[card.id].card_type == CardType.POWER:
                    watcher_run.remove_card(i)
                    found_power = True
                    break
            if not found_power:
                break

        watcher_run.add_relic("Bottled Tornado")

        # Should not have bottled anything
        bottled_card = get_bottled_card_id(watcher_run, "Bottled Tornado")
        assert bottled_card is None


# =============================================================================
# COMBINATION TESTS
# =============================================================================

class TestBottledRelicCombinations:
    """Test interactions between multiple Bottled relics."""

    @pytest.mark.skip(reason="Multiple bottled relics not implemented")
    def test_all_three_bottled_relics(self, watcher_run):
        """All 3 Bottled relics: Can bottle 1 Attack, 1 Skill, 1 Power."""
        watcher_run.add_card("Tantrum")  # Attack
        watcher_run.add_card("Vigilance")  # Skill
        watcher_run.add_card("Meditate")  # Power

        watcher_run.add_relic("Bottled Flame")
        watcher_run.add_relic("Bottled Lightning")
        watcher_run.add_relic("Bottled Tornado")

        # Simulate bottling choices
        flame_relic = watcher_run.get_relic("Bottled Flame")
        if flame_relic:
            flame_relic.card_id = "Tantrum"

        lightning_relic = watcher_run.get_relic("Bottled Lightning")
        if lightning_relic:
            lightning_relic.card_id = "Vigilance"

        tornado_relic = watcher_run.get_relic("Bottled Tornado")
        if tornado_relic:
            tornado_relic.card_id = "Meditate"

        # Verify all are bottled
        assert get_bottled_card_id(watcher_run, "Bottled Flame") == "Tantrum"
        assert get_bottled_card_id(watcher_run, "Bottled Lightning") == "Vigilance"
        assert get_bottled_card_id(watcher_run, "Bottled Tornado") == "Meditate"

    @pytest.mark.skip(reason="Multiple bottled relics combat start not implemented")
    def test_all_three_bottled_relics_combat_start(self, watcher_run):
        """All 3 Bottled relics: All 3 bottled cards start in hand."""
        watcher_run.add_card("Tantrum")
        watcher_run.add_card("Vigilance")
        watcher_run.add_card("Meditate")

        watcher_run.add_relic("Bottled Flame")
        watcher_run.add_relic("Bottled Lightning")
        watcher_run.add_relic("Bottled Tornado")

        # Simulate bottling
        flame = watcher_run.get_relic("Bottled Flame")
        if flame:
            flame.card_id = "Tantrum"

        lightning = watcher_run.get_relic("Bottled Lightning")
        if lightning:
            lightning.card_id = "Vigilance"

        tornado = watcher_run.get_relic("Bottled Tornado")
        if tornado:
            tornado.card_id = "Meditate"

        # Create combat
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # All 3 should be in hand
        assert "Tantrum" in combat.hand
        assert "Vigilance" in combat.hand
        assert "Meditate" in combat.hand

    @pytest.mark.skip(reason="Bottled relic hand size limit not implemented")
    def test_bottled_relics_with_small_hand_size(self, watcher_run):
        """Bottled relics: Should work even if multiple Innate cards exceed normal draw."""
        watcher_run.add_card("Tantrum")
        watcher_run.add_card("Vigilance")
        watcher_run.add_card("Meditate")

        watcher_run.add_relic("Bottled Flame")
        watcher_run.add_relic("Bottled Lightning")
        watcher_run.add_relic("Bottled Tornado")

        # Bottle all 3
        for relic_id, card_id in [
            ("Bottled Flame", "Tantrum"),
            ("Bottled Lightning", "Vigilance"),
            ("Bottled Tornado", "Meditate"),
        ]:
            relic = watcher_run.get_relic(relic_id)
            if relic:
                relic.card_id = card_id

        # Create combat with normal draw (5 cards)
        # The 3 Innate cards + 5 draw should work
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # Hand should have at least 8 cards (3 Innate + 5 draw)
        assert len(combat.hand) >= 8


# =============================================================================
# EDGE CASES
# =============================================================================

class TestBottledRelicEdgeCases:
    """Edge cases for Bottled relics."""

    @pytest.mark.skip(reason="Bottled relic with upgraded card not implemented")
    def test_bottled_flame_with_upgraded_attack(self, watcher_run):
        """Bottled Flame: Can bottle an upgraded Attack."""
        watcher_run.add_card("Tantrum", upgraded=True)
        watcher_run.add_relic("Bottled Flame")

        relic = watcher_run.get_relic("Bottled Flame")
        if relic:
            relic.card_id = "Tantrum"

        # Bottled card should preserve upgrade status
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # Tantrum+ should be in hand
        assert "Tantrum+" in combat.hand

    @pytest.mark.skip(reason="Bottled relic with duplicate cards not implemented")
    def test_bottled_flame_with_duplicate_attacks(self, watcher_run):
        """Bottled Flame: If multiple copies of same Attack, only one becomes Innate."""
        watcher_run.add_card("Tantrum")
        watcher_run.add_card("Tantrum")
        watcher_run.add_card("Tantrum")

        watcher_run.add_relic("Bottled Flame")

        relic = watcher_run.get_relic("Bottled Flame")
        if relic:
            relic.card_id = "Tantrum"

        # Create combat
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # Only 1 Tantrum should be in starting hand (the Innate one)
        # The others should be in draw pile
        tantrum_in_hand = sum(1 for c in combat.hand if c == "Tantrum")
        assert tantrum_in_hand == 1

    @pytest.mark.skip(reason="Bottled relic removal not implemented")
    def test_bottled_flame_when_bottled_card_removed(self, watcher_run):
        """Bottled Flame: If bottled card is removed from deck, relic has no effect."""
        watcher_run.add_card("Tantrum")
        watcher_run.add_relic("Bottled Flame")

        relic = watcher_run.get_relic("Bottled Flame")
        if relic:
            relic.card_id = "Tantrum"

        # Remove Tantrum from deck
        for i, card in enumerate(watcher_run.deck):
            if card.id == "Tantrum":
                watcher_run.remove_card(i)
                break

        # Create combat
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # Tantrum should NOT be in hand (was removed)
        assert "Tantrum" not in combat.hand

    @pytest.mark.skip(reason="Bottled relic with transformed card not implemented")
    def test_bottled_flame_when_bottled_card_transformed(self, watcher_run):
        """Bottled Flame: If bottled card is transformed, new card is NOT Innate."""
        watcher_run.add_card("Tantrum")
        watcher_run.add_relic("Bottled Flame")

        relic = watcher_run.get_relic("Bottled Flame")
        if relic:
            relic.card_id = "Tantrum"

        # Transform Tantrum into another card (e.g., via event)
        for i, card in enumerate(watcher_run.deck):
            if card.id == "Tantrum":
                watcher_run.remove_card(i)
                watcher_run.add_card("Conclude")  # Different Attack
                break

        # Create combat
        combat = create_combat(
            player_hp=70,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)],
            deck=[c.id + ("+" if c.upgraded else "") for c in watcher_run.deck],
            energy=3,
            relics=[r.id for r in watcher_run.relics],
        )

        # Tantrum should NOT be in hand (was transformed)
        # Conclude should NOT be Innate (wasn't the bottled card)
        assert "Tantrum" not in combat.hand


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
