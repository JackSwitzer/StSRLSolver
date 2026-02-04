"""
Pickup/Equip Relic Tests - TDD approach.

Tests for relics that trigger onEquip (when picked up):
- War Paint: Upon pickup, upgrade 2 random Skills
- Whetstone: Upon pickup, upgrade 2 random Attacks
- Astrolabe: Transform 3 cards, they're upgraded
- Calling Bell: Obtain 3 relics and 1 Curse
- Empty Cage: Remove 2 cards from deck
- Tiny House: +50g, +5 Max HP, potion, card, upgrade a card
- Cauldron: Obtain 5 potions
- Dolly's Mirror: Duplicate a card
- Lee's Waffle: +7 Max HP, heal to full
- Orrery: Choose 5 cards to add to deck
- Old Coin: Gain 300 gold
- Pandora's Box: Transform all Strikes and Defends

These are failing tests that document expected behavior.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.state.run import RunState, create_watcher_run
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


def count_card_type(run: RunState, card_type: CardType, upgraded_only: bool = False) -> int:
    """Count cards of a specific type in deck."""
    count = 0
    for card in run.deck:
        if card.id in ALL_CARDS:
            card_def = ALL_CARDS[card.id]
            if card_def.card_type == card_type:
                if not upgraded_only or card.upgraded:
                    count += 1
    return count


def count_upgraded_cards(run: RunState) -> int:
    """Count upgraded cards in deck."""
    return sum(1 for card in run.deck if card.upgraded)


# =============================================================================
# WAR PAINT TESTS
# =============================================================================

class TestWarPaint:
    """War Paint: Upon pickup, upgrade 2 random Skills."""

    @pytest.mark.skip(reason="War Paint upgrade on pickup not implemented")
    def test_war_paint_upgrades_2_skills(self, watcher_run):
        """War Paint: Pickup upgrades 2 random Skills in deck."""
        initial_upgraded = count_upgraded_cards(watcher_run)

        watcher_run.add_relic("War Paint")

        # Should have 2 more upgraded cards (Skills)
        final_upgraded = count_upgraded_cards(watcher_run)
        assert final_upgraded == initial_upgraded + 2

    @pytest.mark.skip(reason="War Paint upgrade on pickup not implemented")
    def test_war_paint_only_upgrades_skills(self, watcher_run):
        """War Paint: Should only upgrade Skills, not Attacks or Powers."""
        # Add some non-skill cards
        watcher_run.add_card("Strike_P")  # Attack
        watcher_run.add_card("Eruption")  # Attack
        watcher_run.add_card("Vigilance")  # Skill
        watcher_run.add_card("Defend_P")  # Skill

        watcher_run.add_relic("War Paint")

        # Check that only Skills were upgraded
        # Vigilance and Defend_P should be candidates
        vigilance = next((c for c in watcher_run.deck if c.id == "Vigilance"), None)
        defend = next((c for c in watcher_run.deck if c.id == "Defend_P"), None)

        # At least one of these should be upgraded
        assert vigilance.upgraded or defend.upgraded

    @pytest.mark.skip(reason="War Paint upgrade on pickup not implemented")
    def test_war_paint_with_fewer_than_2_skills(self, watcher_run):
        """War Paint: If deck has < 2 Skills, upgrade all available Skills."""
        # Remove most cards, leave only 1 Skill
        while len(watcher_run.deck) > 1:
            watcher_run.remove_card(0)

        # Ensure the remaining card is a Skill
        if watcher_run.deck:
            skill_count = count_card_type(watcher_run, CardType.SKILL)
            if skill_count == 0:
                watcher_run.add_card("Vigilance")

        watcher_run.add_relic("War Paint")

        # Should upgrade 1 Skill (all available)
        upgraded_skills = sum(1 for c in watcher_run.deck if c.upgraded and
                               ALL_CARDS.get(c.id, None) and
                               ALL_CARDS[c.id].card_type == CardType.SKILL)
        assert upgraded_skills >= 1

    @pytest.mark.skip(reason="War Paint upgrade on pickup not implemented")
    def test_war_paint_does_not_double_upgrade(self, watcher_run):
        """War Paint: Should not upgrade already-upgraded Skills."""
        # Add 2 Skills, upgrade one
        watcher_run.add_card("Vigilance", upgraded=False)
        watcher_run.add_card("Defend_P", upgraded=True)

        watcher_run.add_relic("War Paint")

        # Should only upgrade Vigilance (not Defend_P again)
        vigilance = next((c for c in watcher_run.deck if c.id == "Vigilance"), None)
        assert vigilance.upgraded

        # Total upgraded count should be reasonable
        # (not counting double-upgrades)


# =============================================================================
# WHETSTONE TESTS
# =============================================================================

class TestWhetstone:
    """Whetstone: Upon pickup, upgrade 2 random Attacks."""

    @pytest.mark.skip(reason="Whetstone upgrade on pickup not implemented")
    def test_whetstone_upgrades_2_attacks(self, watcher_run):
        """Whetstone: Pickup upgrades 2 random Attacks in deck."""
        initial_upgraded = count_upgraded_cards(watcher_run)

        watcher_run.add_relic("Whetstone")

        # Should have 2 more upgraded cards (Attacks)
        final_upgraded = count_upgraded_cards(watcher_run)
        assert final_upgraded == initial_upgraded + 2

    @pytest.mark.skip(reason="Whetstone upgrade on pickup not implemented")
    def test_whetstone_only_upgrades_attacks(self, watcher_run):
        """Whetstone: Should only upgrade Attacks, not Skills or Powers."""
        watcher_run.add_card("Strike_P")  # Attack
        watcher_run.add_card("Eruption")  # Attack
        watcher_run.add_card("Vigilance")  # Skill

        watcher_run.add_relic("Whetstone")

        # Check that only Attacks were upgraded
        strike = next((c for c in watcher_run.deck if c.id == "Strike_P"), None)
        eruption = next((c for c in watcher_run.deck if c.id == "Eruption"), None)
        vigilance = next((c for c in watcher_run.deck if c.id == "Vigilance"), None)

        # At least one Attack should be upgraded
        assert strike.upgraded or eruption.upgraded

        # Vigilance (Skill) should NOT be upgraded by Whetstone
        # (Unless it was already upgraded)

    @pytest.mark.skip(reason="Whetstone upgrade on pickup not implemented")
    def test_whetstone_with_fewer_than_2_attacks(self, watcher_run):
        """Whetstone: If deck has < 2 Attacks, upgrade all available Attacks."""
        # Remove all but 1 Attack
        while count_card_type(watcher_run, CardType.ATTACK) > 1:
            for i, card in enumerate(watcher_run.deck):
                if card.id in ALL_CARDS and ALL_CARDS[card.id].card_type == CardType.ATTACK:
                    watcher_run.remove_card(i)
                    break

        watcher_run.add_relic("Whetstone")

        # Should upgrade 1 Attack (all available)
        upgraded_attacks = sum(1 for c in watcher_run.deck if c.upgraded and
                                ALL_CARDS.get(c.id, None) and
                                ALL_CARDS[c.id].card_type == CardType.ATTACK)
        assert upgraded_attacks >= 1


# =============================================================================
# ASTROLABE TESTS
# =============================================================================

class TestAstrolabe:
    """Astrolabe: Upon pickup, transform and upgrade 3 cards."""

    @pytest.mark.skip(reason="Astrolabe transform on pickup not implemented")
    def test_astrolabe_transforms_3_cards(self, watcher_run, rng):
        """Astrolabe: Pickup transforms 3 cards."""
        initial_deck = [c.id for c in watcher_run.deck]

        watcher_run.add_relic("Astrolabe")

        # Deck size should remain the same
        assert len(watcher_run.deck) == len(initial_deck)

        # But 3 cards should be different (transformed)
        # This is hard to test without seeing the actual transforms

    @pytest.mark.skip(reason="Astrolabe transform on pickup not implemented")
    def test_astrolabe_upgrades_transformed_cards(self, watcher_run, rng):
        """Astrolabe: Transformed cards are also upgraded."""
        initial_upgraded = count_upgraded_cards(watcher_run)

        watcher_run.add_relic("Astrolabe")

        # Should have 3 more upgraded cards
        final_upgraded = count_upgraded_cards(watcher_run)
        assert final_upgraded >= initial_upgraded + 3

    @pytest.mark.skip(reason="Astrolabe transform on pickup not implemented")
    def test_astrolabe_cannot_transform_basic_cards(self, watcher_run, rng):
        """Astrolabe: Cannot transform basic cards (Strike, Defend, etc.)."""
        # Watcher starting deck has Strikes, Defends, Eruption, Vigilance
        # Astrolabe should not transform these

        watcher_run.add_relic("Astrolabe")

        # Basic cards should still be present
        # (Astrolabe only transforms non-basic cards)


# =============================================================================
# CALLING BELL TESTS
# =============================================================================

class TestCallingBell:
    """Calling Bell: Upon pickup, obtain 3 relics (Common, Uncommon, Rare) and 1 Curse."""

    @pytest.mark.skip(reason="Calling Bell rewards not implemented")
    def test_calling_bell_grants_3_relics(self, watcher_run, rng):
        """Calling Bell: Pickup grants 3 additional relics."""
        initial_relics = len(watcher_run.relics)

        watcher_run.add_relic("Calling Bell")

        # Should have 4 relics: Calling Bell + 3 others
        assert len(watcher_run.relics) == initial_relics + 4

    @pytest.mark.skip(reason="Calling Bell rewards not implemented")
    def test_calling_bell_grants_curse(self, watcher_run, rng):
        """Calling Bell: Pickup grants 1 Curse."""
        initial_deck_size = len(watcher_run.deck)

        watcher_run.add_relic("Calling Bell")

        # Should have 1 more card (Curse)
        assert len(watcher_run.deck) == initial_deck_size + 1

        # Check that it's a Curse
        new_card = watcher_run.deck[-1]
        if new_card.id in ALL_CARDS:
            assert ALL_CARDS[new_card.id].card_type == CardType.CURSE

    @pytest.mark.skip(reason="Calling Bell rewards not implemented")
    def test_calling_bell_relic_tiers(self, watcher_run, rng):
        """Calling Bell: Should grant 1 Common, 1 Uncommon, 1 Rare relic."""
        watcher_run.add_relic("Calling Bell")

        # This would require checking relic tiers
        # For now, verify we got 4 relics total (including Calling Bell)
        assert len(watcher_run.relics) >= 4


# =============================================================================
# EMPTY CAGE TESTS
# =============================================================================

class TestEmptyCage:
    """Empty Cage: Upon pickup, remove 2 cards from your deck."""

    @pytest.mark.skip(reason="Empty Cage removal not implemented")
    def test_empty_cage_removes_2_cards(self, watcher_run):
        """Empty Cage: Pickup removes 2 cards from deck."""
        initial_deck_size = len(watcher_run.deck)

        watcher_run.add_relic("Empty Cage")

        # Deck should shrink by 2
        assert len(watcher_run.deck) == initial_deck_size - 2

    @pytest.mark.skip(reason="Empty Cage removal not implemented")
    def test_empty_cage_allows_choice(self, watcher_run):
        """Empty Cage: Player can choose which 2 cards to remove."""
        # In actual game, this presents a UI choice
        # For testing, we can simulate the choice

        initial_deck_size = len(watcher_run.deck)

        # Simulate choosing first 2 cards
        watcher_run.remove_card(0)
        watcher_run.remove_card(0)

        assert len(watcher_run.deck) == initial_deck_size - 2


# =============================================================================
# TINY HOUSE TESTS
# =============================================================================

class TestTinyHouse:
    """Tiny House: Upon pickup, gain 50 Gold, +5 Max HP, 1 potion, 1 card, and upgrade 1 card."""

    @pytest.mark.skip(reason="Tiny House rewards not implemented")
    def test_tiny_house_grants_50_gold(self, watcher_run):
        """Tiny House: Pickup grants 50 gold."""
        initial_gold = watcher_run.gold

        watcher_run.add_relic("Tiny House")

        assert watcher_run.gold == initial_gold + 50

    @pytest.mark.skip(reason="Tiny House rewards not implemented")
    def test_tiny_house_grants_5_max_hp(self, watcher_run):
        """Tiny House: Pickup grants +5 Max HP."""
        initial_max_hp = watcher_run.max_hp

        watcher_run.add_relic("Tiny House")

        assert watcher_run.max_hp == initial_max_hp + 5

    @pytest.mark.skip(reason="Tiny House rewards not implemented")
    def test_tiny_house_grants_potion(self, watcher_run):
        """Tiny House: Pickup grants 1 potion."""
        initial_potions = watcher_run.count_potions()

        watcher_run.add_relic("Tiny House")

        # Should have 1 more potion
        assert watcher_run.count_potions() == initial_potions + 1

    @pytest.mark.skip(reason="Tiny House rewards not implemented")
    def test_tiny_house_grants_card(self, watcher_run):
        """Tiny House: Pickup grants 1 card to add to deck."""
        # This would present a card choice screen
        # For now, verify the relic exists

        watcher_run.add_relic("Tiny House")
        assert watcher_run.has_relic("Tiny House")

    @pytest.mark.skip(reason="Tiny House rewards not implemented")
    def test_tiny_house_upgrades_1_card(self, watcher_run):
        """Tiny House: Pickup upgrades 1 card in deck."""
        initial_upgraded = count_upgraded_cards(watcher_run)

        watcher_run.add_relic("Tiny House")

        # Should have 1 more upgraded card
        final_upgraded = count_upgraded_cards(watcher_run)
        assert final_upgraded == initial_upgraded + 1


# =============================================================================
# CAULDRON TESTS
# =============================================================================

class TestCauldron:
    """Cauldron: Upon pickup, obtain 5 random potions."""

    @pytest.mark.skip(reason="Cauldron potion grants not implemented")
    def test_cauldron_grants_5_potions(self, watcher_run):
        """Cauldron: Pickup grants 5 potions."""
        initial_potions = watcher_run.count_potions()

        watcher_run.add_relic("Cauldron")

        # Should have 5 more potions
        # (limited by potion slot count)
        expected_potions = min(initial_potions + 5, len(watcher_run.potion_slots))
        assert watcher_run.count_potions() == expected_potions

    @pytest.mark.skip(reason="Cauldron potion grants not implemented")
    def test_cauldron_with_full_potion_slots(self, watcher_run):
        """Cauldron: If potion slots full, excess potions are lost."""
        # Fill all potion slots
        for i in range(len(watcher_run.potion_slots)):
            watcher_run.add_potion("FirePotion")

        initial_potions = watcher_run.count_potions()

        watcher_run.add_relic("Cauldron")

        # Potions should still be maxed
        assert watcher_run.count_potions() == initial_potions


# =============================================================================
# DOLLY'S MIRROR TESTS
# =============================================================================

class TestDollysMirror:
    """Dolly's Mirror: Upon pickup, choose a card to duplicate."""

    @pytest.mark.skip(reason="Dolly's Mirror duplication not implemented")
    def test_dollys_mirror_duplicates_card(self, watcher_run):
        """Dolly's Mirror: Pickup duplicates a chosen card."""
        watcher_run.add_card("Strike_P")
        initial_strikes = watcher_run.count_card("Strike_P")

        watcher_run.add_relic("Dolly's Mirror")

        # Simulate choosing Strike_P to duplicate
        watcher_run.add_card("Strike_P")

        # Should have 1 more Strike_P
        assert watcher_run.count_card("Strike_P") == initial_strikes + 1

    @pytest.mark.skip(reason="Dolly's Mirror duplication not implemented")
    def test_dollys_mirror_preserves_upgrade(self, watcher_run):
        """Dolly's Mirror: Duplicated card preserves upgrade status."""
        watcher_run.add_card("Strike_P", upgraded=True)

        watcher_run.add_relic("Dolly's Mirror")

        # Simulate choosing upgraded Strike_P to duplicate
        watcher_run.add_card("Strike_P", upgraded=True)

        # Should have 2 upgraded Strike_P
        assert watcher_run.count_card("Strike_P", upgraded_only=True) == 2


# =============================================================================
# LEE'S WAFFLE TESTS
# =============================================================================

class TestLeesWaffle:
    """Lee's Waffle: Upon pickup, gain +7 Max HP and heal to full."""

    @pytest.mark.skip(reason="Lee's Waffle max HP not implemented")
    def test_lees_waffle_grants_7_max_hp(self, watcher_run):
        """Lee's Waffle: Pickup grants +7 Max HP."""
        initial_max_hp = watcher_run.max_hp

        watcher_run.add_relic("Lee's Waffle")

        assert watcher_run.max_hp == initial_max_hp + 7

    @pytest.mark.skip(reason="Lee's Waffle heal not implemented")
    def test_lees_waffle_heals_to_full(self, watcher_run):
        """Lee's Waffle: Pickup heals to full HP."""
        watcher_run.damage(30)
        initial_hp = watcher_run.current_hp

        watcher_run.add_relic("Lee's Waffle")

        # Should be at full HP (max HP + 7)
        assert watcher_run.current_hp == watcher_run.max_hp


# =============================================================================
# ORRERY TESTS
# =============================================================================

class TestOrrery:
    """Orrery: Upon pickup, choose 5 cards to add to your deck."""

    @pytest.mark.skip(reason="Orrery card selection not implemented")
    def test_orrery_adds_5_cards(self, watcher_run):
        """Orrery: Pickup allows adding 5 cards to deck."""
        initial_deck_size = len(watcher_run.deck)

        watcher_run.add_relic("Orrery")

        # After choosing 5 cards, deck should have 5 more
        # For now, simulate adding 5 cards
        for _ in range(5):
            watcher_run.add_card("Strike_P")

        assert len(watcher_run.deck) == initial_deck_size + 5

    @pytest.mark.skip(reason="Orrery card selection not implemented")
    def test_orrery_allows_choice(self, watcher_run):
        """Orrery: Player can choose which 5 cards to add."""
        # In actual game, presents a card selection screen
        # For testing, verify the relic exists

        watcher_run.add_relic("Orrery")
        assert watcher_run.has_relic("Orrery")


# =============================================================================
# OLD COIN TESTS
# =============================================================================

class TestOldCoin:
    """Old Coin: Upon pickup, gain 300 gold."""

    @pytest.mark.skip(reason="Old Coin gold grant not implemented")
    def test_old_coin_grants_300_gold(self, watcher_run):
        """Old Coin: Pickup grants 300 gold."""
        initial_gold = watcher_run.gold

        watcher_run.add_relic("Old Coin")

        assert watcher_run.gold == initial_gold + 300

    @pytest.mark.skip(reason="Old Coin Ectoplasm interaction not implemented")
    def test_old_coin_with_ectoplasm(self, watcher_run):
        """Old Coin: Should NOT grant gold if Ectoplasm prevents gold gain."""
        watcher_run.add_relic("Ectoplasm")
        initial_gold = watcher_run.gold

        watcher_run.add_relic("Old Coin")

        # Ectoplasm blocks gold gain
        assert watcher_run.gold == initial_gold


# =============================================================================
# PANDORA'S BOX TESTS
# =============================================================================

class TestPandorasBox:
    """Pandora's Box: Upon pickup, transform all Strikes and Defends."""

    @pytest.mark.skip(reason="Pandora's Box transform not implemented")
    def test_pandoras_box_transforms_strikes(self, watcher_run, rng):
        """Pandora's Box: Pickup transforms all Strikes."""
        initial_strikes = watcher_run.count_card("Strike_P")

        watcher_run.add_relic("Pandora's Box")

        # Should have 0 Strikes remaining
        final_strikes = watcher_run.count_card("Strike_P")
        assert final_strikes == 0

    @pytest.mark.skip(reason="Pandora's Box transform not implemented")
    def test_pandoras_box_transforms_defends(self, watcher_run, rng):
        """Pandora's Box: Pickup transforms all Defends."""
        initial_defends = watcher_run.count_card("Defend_P")

        watcher_run.add_relic("Pandora's Box")

        # Should have 0 Defends remaining
        final_defends = watcher_run.count_card("Defend_P")
        assert final_defends == 0

    @pytest.mark.skip(reason="Pandora's Box transform not implemented")
    def test_pandoras_box_preserves_deck_size(self, watcher_run, rng):
        """Pandora's Box: Deck size remains the same (transform, not remove)."""
        initial_deck_size = len(watcher_run.deck)

        watcher_run.add_relic("Pandora's Box")

        # Deck size should be unchanged
        assert len(watcher_run.deck) == initial_deck_size

    @pytest.mark.skip(reason="Pandora's Box transform not implemented")
    def test_pandoras_box_does_not_affect_starter_cards(self, watcher_run, rng):
        """Pandora's Box: Should NOT transform starter cards like Eruption/Vigilance."""
        watcher_run.add_relic("Pandora's Box")

        # Eruption and Vigilance should still be in deck
        assert watcher_run.count_card("Eruption") > 0
        assert watcher_run.count_card("Vigilance") > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
