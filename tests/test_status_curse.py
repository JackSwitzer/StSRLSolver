"""
Comprehensive Status and Curse Card Tests

Tests all status and curse card mechanics including:
1. Status Cards: Wound, Dazed, Burn, Void, Slimed
2. Curse Cards: Ascender's Bane, Clumsy, Decay, Doubt, Injury, Necronomicurse,
                Normality, Pain, Parasite, Regret, Shame, Writhe
3. Draw effects (Void energy loss)
4. End of turn effects (Burn, Decay, Doubt, Regret, Shame)
5. Play restrictions (Normality, unplayable cards)
6. Removal mechanics (Ascender's Bane, Necronomicurse, Parasite)
7. Ethereal behavior
8. Relic interactions (Blue Candle, Omamori, Darkstone Periapt)
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from core.content.cards import (
    Card,
    CardType,
    CardRarity,
    CardColor,
    CardTarget,
    # Status cards
    WOUND,
    DAZED,
    BURN,
    VOID,
    SLIMED,
    # Curse cards
    ASCENDERS_BANE,
    CLUMSY,
    CURSE_OF_THE_BELL,
    DECAY,
    DOUBT,
    INJURY,
    NECRONOMICURSE,
    NORMALITY,
    PAIN,
    PARASITE,
    PRIDE,
    REGRET,
    SHAME,
    WRITHE,
    # Registry and helpers
    STATUS_CARDS,
    CURSE_CARDS,
    get_card,
)

from core.content.relics import (
    BLUE_CANDLE,
    OMAMORI,
    DARKSTONE_PERIAPT,
    MEDICAL_KIT,
    DU_VU_DOLL,
    get_relic,
)


# =============================================================================
# HELPER CLASSES FOR TESTING
# =============================================================================

class MockPlayer:
    """Mock player for testing status/curse effects."""

    def __init__(self, hp: int = 50, max_hp: int = 80, energy: int = 3):
        self.hp = hp
        self.max_hp = max_hp
        self.energy = energy
        self.hand = []
        self.draw_pile = []
        self.discard_pile = []
        self.exhaust_pile = []
        self.deck = []  # Full deck for removal operations
        self.relics = []
        self.powers = {}  # power_name -> amount
        self.cards_played_this_turn = 0
        self.omamori_charges = 0

    def add_to_hand(self, card: Card):
        self.hand.append(card)

    def add_to_deck(self, card: Card):
        self.deck.append(card)

    def draw_card(self, card: Card):
        """Simulate drawing a card with on-draw effects."""
        self.hand.append(card)
        # Void effect: lose 1 energy when drawn
        if card.id == "Void" and "lose_1_energy_when_drawn" in card.effects:
            self.energy = max(0, self.energy - 1)

    def take_damage(self, amount: int):
        self.hp = max(0, self.hp - amount)

    def lose_hp(self, amount: int):
        """HP loss that bypasses block (from curses)."""
        self.hp = max(0, self.hp - amount)

    def gain_max_hp(self, amount: int):
        self.max_hp += amount

    def lose_max_hp(self, amount: int):
        self.max_hp = max(1, self.max_hp - amount)
        self.hp = min(self.hp, self.max_hp)

    def apply_power(self, power_name: str, amount: int):
        if power_name in self.powers:
            self.powers[power_name] += amount
        else:
            self.powers[power_name] = amount

    def has_relic(self, relic_id: str) -> bool:
        return any(r.id == relic_id for r in self.relics)

    def exhaust_card(self, card: Card):
        """Exhaust a card, handling Necronomicurse special case."""
        if card in self.hand:
            self.hand.remove(card)
        if card in self.discard_pile:
            self.discard_pile.remove(card)
        self.exhaust_pile.append(card)

        # Necronomicurse returns when exhausted
        if card.id == "Necronomicurse" and "returns_when_exhausted_or_removed" in card.effects:
            # Returns to hand
            new_copy = card.copy()
            self.hand.append(new_copy)

    def remove_card_from_deck(self, card: Card) -> bool:
        """Remove a card from deck. Returns False if card cannot be removed."""
        if card.id == "AscendersBane" and "cannot_be_removed" in card.effects:
            return False
        if card.id == "CurseOfTheBell" and "cannot_be_removed" in card.effects:
            return False
        if card in self.deck:
            self.deck.remove(card)
            # Parasite effect: lose 3 max HP when removed
            if card.id == "Parasite" and "lose_3_max_hp_when_removed" in card.effects:
                self.lose_max_hp(3)
            # Necronomicurse returns when removed
            if card.id == "Necronomicurse" and "returns_when_exhausted_or_removed" in card.effects:
                self.deck.append(card.copy())
            return True
        return False

    def add_curse_to_deck(self, curse: Card) -> bool:
        """Add a curse to deck. Returns False if blocked by Omamori."""
        if self.omamori_charges > 0:
            self.omamori_charges -= 1
            return False  # Curse blocked
        self.deck.append(curse)
        # Darkstone Periapt: gain 6 max HP when obtaining curse
        if self.has_relic("Darkstone Periapt"):
            self.gain_max_hp(6)
        return True


class MockCombatState:
    """Mock combat state for testing end-of-turn effects."""

    def __init__(self, player: MockPlayer):
        self.player = player
        self.turn = 1

    def end_of_turn(self):
        """Process end of turn effects from cards in hand."""
        for card in self.player.hand[:]:  # Copy list to avoid modification during iteration
            self._process_end_of_turn_effect(card)
        self.turn += 1

    def _process_end_of_turn_effect(self, card: Card):
        """Process a single card's end of turn effect."""
        # Burn: take 2 damage (4 if upgraded)
        if card.id == "Burn" and "end_of_turn_take_damage" in card.effects:
            damage = card.base_magic  # 2, or 4 if upgraded
            if card.upgraded:
                damage = card.base_magic + card.upgrade_magic
            self.player.take_damage(damage)

        # Decay: take 2 damage
        if card.id == "Decay" and "end_of_turn_take_2_damage" in card.effects:
            self.player.take_damage(2)

        # Doubt: gain 1 weak
        if card.id == "Doubt" and "end_of_turn_gain_weak_1" in card.effects:
            self.player.apply_power("Weak", 1)

        # Shame: gain 1 frail
        if card.id == "Shame" and "end_of_turn_gain_frail_1" in card.effects:
            self.player.apply_power("Frail", 1)

        # Regret: lose HP equal to cards in hand
        if card.id == "Regret" and "end_of_turn_lose_hp_equal_to_hand_size" in card.effects:
            self.player.lose_hp(len(self.player.hand))

        # Handle ethereal cards (exhaust at end of turn if in hand)
        if card.ethereal and card in self.player.hand:
            self.player.hand.remove(card)
            self.player.exhaust_pile.append(card)


# =============================================================================
# SECTION 1: STATUS CARD BASIC PROPERTIES
# =============================================================================

class TestStatusCardProperties:
    """Test basic properties of status cards."""

    def test_wound_is_status_type(self):
        """Wound should be STATUS type."""
        wound = get_card("Wound")
        assert wound.card_type == CardType.STATUS

    def test_wound_is_unplayable(self):
        """Wound has cost -2 (unplayable) and unplayable effect."""
        wound = get_card("Wound")
        assert wound.cost == -2
        assert "unplayable" in wound.effects

    def test_dazed_is_status_type(self):
        """Dazed should be STATUS type."""
        dazed = get_card("Dazed")
        assert dazed.card_type == CardType.STATUS

    def test_dazed_is_unplayable_and_ethereal(self):
        """Dazed is unplayable and ethereal."""
        dazed = get_card("Dazed")
        assert dazed.cost == -2
        assert "unplayable" in dazed.effects
        assert dazed.ethereal == True

    def test_burn_is_status_type(self):
        """Burn should be STATUS type."""
        burn = get_card("Burn")
        assert burn.card_type == CardType.STATUS

    def test_burn_is_unplayable_with_end_turn_damage(self):
        """Burn is unplayable and deals damage at end of turn."""
        burn = get_card("Burn")
        assert burn.cost == -2
        assert "unplayable" in burn.effects
        assert "end_of_turn_take_damage" in burn.effects

    def test_burn_base_damage_is_2(self):
        """Burn deals 2 damage (base)."""
        burn = get_card("Burn")
        assert burn.base_magic == 2

    def test_burn_upgraded_deals_4_damage(self):
        """Burn+ deals 4 damage."""
        burn = get_card("Burn", upgraded=True)
        assert burn.magic_number == 4  # 2 base + 2 upgrade

    def test_void_is_status_type(self):
        """Void should be STATUS type."""
        void = get_card("Void")
        assert void.card_type == CardType.STATUS

    def test_void_is_unplayable_ethereal_and_drains_energy(self):
        """Void is unplayable, ethereal, and loses 1 energy when drawn."""
        void = get_card("Void")
        assert void.cost == -2
        assert "unplayable" in void.effects
        assert void.ethereal == True
        assert "lose_1_energy_when_drawn" in void.effects

    def test_slimed_is_status_type(self):
        """Slimed should be STATUS type."""
        slimed = get_card("Slimed")
        assert slimed.card_type == CardType.STATUS

    def test_slimed_is_playable_and_exhausts(self):
        """Slimed costs 1, exhausts, and does nothing when played."""
        slimed = get_card("Slimed")
        assert slimed.cost == 1
        assert slimed.exhaust == True
        # Slimed has no special effects - just wastes energy
        assert len(slimed.effects) == 0


# =============================================================================
# SECTION 2: CURSE CARD BASIC PROPERTIES
# =============================================================================

class TestCurseCardProperties:
    """Test basic properties of curse cards."""

    def test_ascenders_bane_is_curse_type(self):
        """Ascender's Bane should be CURSE type."""
        curse = get_card("AscendersBane")
        assert curse.card_type == CardType.CURSE

    def test_ascenders_bane_cannot_be_removed(self):
        """Ascender's Bane has cannot_be_removed effect."""
        curse = get_card("AscendersBane")
        assert "cannot_be_removed" in curse.effects
        assert "unplayable" in curse.effects
        assert curse.ethereal == True

    def test_clumsy_is_curse_type(self):
        """Clumsy should be CURSE type."""
        curse = get_card("Clumsy")
        assert curse.card_type == CardType.CURSE

    def test_clumsy_is_unplayable_ethereal(self):
        """Clumsy is unplayable and ethereal."""
        curse = get_card("Clumsy")
        assert curse.cost == -2
        assert "unplayable" in curse.effects
        assert curse.ethereal == True

    def test_decay_is_curse_type(self):
        """Decay should be CURSE type."""
        curse = get_card("Decay")
        assert curse.card_type == CardType.CURSE

    def test_decay_deals_end_of_turn_damage(self):
        """Decay deals 2 damage at end of turn."""
        curse = get_card("Decay")
        assert "end_of_turn_take_2_damage" in curse.effects
        assert "unplayable" in curse.effects

    def test_doubt_is_curse_type(self):
        """Doubt should be CURSE type."""
        curse = get_card("Doubt")
        assert curse.card_type == CardType.CURSE

    def test_doubt_applies_weak_at_end_of_turn(self):
        """Doubt applies 1 Weak at end of turn."""
        curse = get_card("Doubt")
        assert "end_of_turn_gain_weak_1" in curse.effects
        assert "unplayable" in curse.effects

    def test_injury_is_curse_type(self):
        """Injury should be CURSE type."""
        curse = get_card("Injury")
        assert curse.card_type == CardType.CURSE

    def test_injury_is_simple_unplayable(self):
        """Injury is just unplayable with no other effects."""
        curse = get_card("Injury")
        assert "unplayable" in curse.effects
        assert curse.cost == -2

    def test_necronomicurse_is_curse_type(self):
        """Necronomicurse should be CURSE type."""
        curse = get_card("Necronomicurse")
        assert curse.card_type == CardType.CURSE

    def test_necronomicurse_returns_when_exhausted(self):
        """Necronomicurse returns when exhausted or removed."""
        curse = get_card("Necronomicurse")
        assert "returns_when_exhausted_or_removed" in curse.effects
        assert "unplayable" in curse.effects

    def test_normality_is_curse_type(self):
        """Normality should be CURSE type."""
        curse = get_card("Normality")
        assert curse.card_type == CardType.CURSE

    def test_normality_limits_cards_per_turn(self):
        """Normality limits to 3 cards per turn."""
        curse = get_card("Normality")
        assert "limit_3_cards_per_turn" in curse.effects
        assert "unplayable" in curse.effects

    def test_pain_is_curse_type(self):
        """Pain should be CURSE type."""
        curse = get_card("Pain")
        assert curse.card_type == CardType.CURSE

    def test_pain_causes_hp_loss_on_card_play(self):
        """Pain causes 1 HP loss when other cards are played."""
        curse = get_card("Pain")
        assert "lose_1_hp_when_other_card_played" in curse.effects
        assert "unplayable" in curse.effects

    def test_parasite_is_curse_type(self):
        """Parasite should be CURSE type."""
        curse = get_card("Parasite")
        assert curse.card_type == CardType.CURSE

    def test_parasite_loses_max_hp_on_removal(self):
        """Parasite causes 3 max HP loss when removed."""
        curse = get_card("Parasite")
        assert "lose_3_max_hp_when_removed" in curse.effects
        assert "unplayable" in curse.effects

    def test_regret_is_curse_type(self):
        """Regret should be CURSE type."""
        curse = get_card("Regret")
        assert curse.card_type == CardType.CURSE

    def test_regret_loses_hp_equal_to_hand_size(self):
        """Regret loses HP equal to hand size at end of turn."""
        curse = get_card("Regret")
        assert "end_of_turn_lose_hp_equal_to_hand_size" in curse.effects
        assert "unplayable" in curse.effects

    def test_shame_is_curse_type(self):
        """Shame should be CURSE type."""
        curse = get_card("Shame")
        assert curse.card_type == CardType.CURSE

    def test_shame_applies_frail_at_end_of_turn(self):
        """Shame applies 1 Frail at end of turn."""
        curse = get_card("Shame")
        assert "end_of_turn_gain_frail_1" in curse.effects
        assert "unplayable" in curse.effects

    def test_writhe_is_curse_type(self):
        """Writhe should be CURSE type."""
        curse = get_card("Writhe")
        assert curse.card_type == CardType.CURSE

    def test_writhe_is_innate_and_unplayable(self):
        """Writhe is innate (starts in hand) and unplayable."""
        curse = get_card("Writhe")
        assert curse.innate == True
        assert "unplayable" in curse.effects
        assert curse.cost == -2


# =============================================================================
# SECTION 3: STATUS CARD DRAW EFFECTS
# =============================================================================

class TestStatusCardDrawEffects:
    """Test draw effects of status cards, particularly Void."""

    def test_void_loses_1_energy_when_drawn(self):
        """Drawing Void loses 1 energy."""
        player = MockPlayer(energy=3)
        void = get_card("Void")
        player.draw_card(void)
        assert player.energy == 2

    def test_void_energy_loss_cannot_go_negative(self):
        """Void energy loss cannot reduce energy below 0."""
        player = MockPlayer(energy=0)
        void = get_card("Void")
        player.draw_card(void)
        assert player.energy == 0

    def test_multiple_voids_lose_multiple_energy(self):
        """Drawing multiple Voids loses multiple energy."""
        player = MockPlayer(energy=5)
        for _ in range(3):
            void = get_card("Void")
            player.draw_card(void)
        assert player.energy == 2

    def test_wound_no_draw_effect(self):
        """Wound has no special effect when drawn."""
        player = MockPlayer(energy=3)
        wound = get_card("Wound")
        player.draw_card(wound)
        assert player.energy == 3
        assert wound in player.hand

    def test_dazed_no_draw_effect(self):
        """Dazed has no special effect when drawn (only ethereal at end of turn)."""
        player = MockPlayer(energy=3)
        dazed = get_card("Dazed")
        player.draw_card(dazed)
        assert player.energy == 3
        assert dazed in player.hand


# =============================================================================
# SECTION 4: END OF TURN EFFECTS
# =============================================================================

class TestEndOfTurnEffects:
    """Test end of turn effects from status and curse cards."""

    def test_burn_deals_2_damage_at_end_of_turn(self):
        """Burn deals 2 damage at end of turn."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Burn"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.hp == 48

    def test_burn_upgraded_deals_4_damage(self):
        """Burn+ deals 4 damage at end of turn."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Burn", upgraded=True))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.hp == 46

    def test_multiple_burns_deal_multiple_damage(self):
        """Multiple Burns in hand deal cumulative damage."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Burn"))
        player.add_to_hand(get_card("Burn"))
        player.add_to_hand(get_card("Burn"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.hp == 44  # 3 * 2 = 6 damage

    def test_decay_deals_2_damage_at_end_of_turn(self):
        """Decay deals 2 damage at end of turn."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Decay"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.hp == 48

    def test_doubt_applies_weak_at_end_of_turn(self):
        """Doubt applies 1 Weak at end of turn."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Doubt"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.powers.get("Weak", 0) == 1

    def test_doubt_stacks_weak_each_turn(self):
        """Doubt applies Weak each turn."""
        player = MockPlayer(hp=50)
        doubt = get_card("Doubt")
        player.add_to_hand(doubt)
        combat = MockCombatState(player)
        combat.end_of_turn()
        # Doubt is not ethereal, so it stays in hand
        # It already triggers once, so Weak = 1
        assert player.powers.get("Weak", 0) == 1
        # Simulate next turn - doubt is still there
        combat.end_of_turn()
        # Should have 2 weak from 2 turns
        assert player.powers.get("Weak", 0) == 2

    def test_shame_applies_frail_at_end_of_turn(self):
        """Shame applies 1 Frail at end of turn."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Shame"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.powers.get("Frail", 0) == 1

    def test_regret_loses_hp_equal_to_hand_size(self):
        """Regret loses HP equal to number of cards in hand."""
        player = MockPlayer(hp=50)
        # Hand with 5 cards including Regret
        player.add_to_hand(get_card("Regret"))
        player.add_to_hand(get_card("Wound"))
        player.add_to_hand(get_card("Wound"))
        player.add_to_hand(get_card("Wound"))
        player.add_to_hand(get_card("Wound"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.hp == 45  # Lost 5 HP for 5 cards in hand

    def test_regret_with_empty_hand(self):
        """Regret with empty hand loses 1 HP (just Regret itself)."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Regret"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.hp == 49  # Lost 1 HP for Regret itself


# =============================================================================
# SECTION 5: ETHEREAL BEHAVIOR
# =============================================================================

class TestEtherealBehavior:
    """Test ethereal card behavior (exhaust at end of turn if in hand)."""

    def test_dazed_exhausts_at_end_of_turn(self):
        """Dazed exhausts at end of turn if still in hand."""
        player = MockPlayer()
        dazed = get_card("Dazed")
        player.add_to_hand(dazed)
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert dazed not in player.hand
        assert dazed in player.exhaust_pile

    def test_void_exhausts_at_end_of_turn(self):
        """Void exhausts at end of turn if still in hand."""
        player = MockPlayer()
        void = get_card("Void")
        player.add_to_hand(void)
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert void not in player.hand
        assert void in player.exhaust_pile

    def test_clumsy_exhausts_at_end_of_turn(self):
        """Clumsy exhausts at end of turn if still in hand."""
        player = MockPlayer()
        clumsy = get_card("Clumsy")
        player.add_to_hand(clumsy)
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert clumsy not in player.hand
        assert clumsy in player.exhaust_pile

    def test_ascenders_bane_exhausts_at_end_of_turn(self):
        """Ascender's Bane exhausts at end of turn if still in hand."""
        player = MockPlayer()
        curse = get_card("AscendersBane")
        player.add_to_hand(curse)
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert curse not in player.hand
        assert curse in player.exhaust_pile

    def test_wound_does_not_exhaust_not_ethereal(self):
        """Wound does not exhaust at end of turn (not ethereal)."""
        player = MockPlayer()
        wound = get_card("Wound")
        player.add_to_hand(wound)
        combat = MockCombatState(player)
        combat.end_of_turn()
        # Wound should still be in hand (would be discarded in real game)
        assert wound in player.hand
        assert wound not in player.exhaust_pile


# =============================================================================
# SECTION 6: REMOVAL MECHANICS
# =============================================================================

class TestRemovalMechanics:
    """Test curse removal mechanics."""

    def test_ascenders_bane_cannot_be_removed(self):
        """Ascender's Bane cannot be removed from deck."""
        player = MockPlayer()
        curse = get_card("AscendersBane")
        player.add_to_deck(curse)
        result = player.remove_card_from_deck(curse)
        assert result == False
        assert curse in player.deck

    def test_curse_of_the_bell_cannot_be_removed(self):
        """Curse of the Bell cannot be removed from deck."""
        player = MockPlayer()
        curse = get_card("CurseOfTheBell")
        player.add_to_deck(curse)
        result = player.remove_card_from_deck(curse)
        assert result == False
        assert curse in player.deck

    def test_parasite_loses_3_max_hp_on_removal(self):
        """Removing Parasite loses 3 max HP."""
        player = MockPlayer(max_hp=80, hp=50)
        curse = get_card("Parasite")
        player.add_to_deck(curse)
        player.remove_card_from_deck(curse)
        assert player.max_hp == 77
        assert curse not in player.deck

    def test_parasite_removal_adjusts_current_hp(self):
        """If current HP exceeds new max HP after Parasite removal, it adjusts."""
        player = MockPlayer(max_hp=80, hp=79)
        curse = get_card("Parasite")
        player.add_to_deck(curse)
        player.remove_card_from_deck(curse)
        assert player.max_hp == 77
        assert player.hp == 77  # Adjusted to new max

    def test_necronomicurse_returns_when_removed(self):
        """Necronomicurse returns to deck when removed."""
        player = MockPlayer()
        curse = get_card("Necronomicurse")
        player.add_to_deck(curse)
        player.remove_card_from_deck(curse)
        # Should have a new copy in deck
        assert len([c for c in player.deck if c.id == "Necronomicurse"]) == 1

    def test_necronomicurse_returns_when_exhausted(self):
        """Necronomicurse returns to hand when exhausted."""
        player = MockPlayer()
        curse = get_card("Necronomicurse")
        player.add_to_hand(curse)
        player.exhaust_card(curse)
        assert curse in player.exhaust_pile
        # Should have a new copy in hand
        assert len([c for c in player.hand if c.id == "Necronomicurse"]) == 1

    def test_normal_curse_can_be_removed(self):
        """Normal curses like Injury can be removed."""
        player = MockPlayer()
        curse = get_card("Injury")
        player.add_to_deck(curse)
        result = player.remove_card_from_deck(curse)
        assert result == True
        assert curse not in player.deck


# =============================================================================
# SECTION 7: PLAY RESTRICTIONS
# =============================================================================

class TestPlayRestrictions:
    """Test card play restrictions from Normality and unplayable cards."""

    def test_unplayable_cards_have_cost_minus_2(self):
        """All unplayable cards have cost -2."""
        unplayable_cards = [
            "Wound", "Dazed", "Burn", "Void",
            "AscendersBane", "Clumsy", "Decay", "Doubt",
            "Injury", "Necronomicurse", "Normality", "Pain",
            "Parasite", "Regret", "Shame", "Writhe"
        ]
        for card_id in unplayable_cards:
            card = get_card(card_id)
            if "unplayable" in card.effects:
                assert card.cost == -2, f"{card_id} should have cost -2"

    def test_slimed_is_playable(self):
        """Slimed is the only playable status card."""
        slimed = get_card("Slimed")
        assert slimed.cost == 1
        assert "unplayable" not in slimed.effects

    def test_normality_has_3_card_limit_effect(self):
        """Normality has the 3 card per turn limit effect."""
        normality = get_card("Normality")
        assert "limit_3_cards_per_turn" in normality.effects


# =============================================================================
# SECTION 8: RELIC INTERACTIONS - BLUE CANDLE
# =============================================================================

class TestBlueCandle:
    """Test Blue Candle relic interactions with curses."""

    def test_blue_candle_allows_curse_play(self):
        """Blue Candle allows playing curses (they exhaust and deal 1 HP loss)."""
        relic = get_relic("Blue Candle")
        # Check the effect description mentions curse playing
        assert any("Curse" in effect and "played" in effect for effect in relic.effects)

    def test_blue_candle_exists_in_registry(self):
        """Blue Candle relic exists and has correct tier."""
        from core.content.relics import RelicTier
        relic = get_relic("Blue Candle")
        assert relic.tier == RelicTier.UNCOMMON


# =============================================================================
# SECTION 9: RELIC INTERACTIONS - OMAMORI
# =============================================================================

class TestOmamori:
    """Test Omamori relic interactions with curses."""

    def test_omamori_blocks_curse(self):
        """Omamori negates curses added to deck."""
        player = MockPlayer()
        player.omamori_charges = 2
        curse = get_card("Injury")
        result = player.add_curse_to_deck(curse)
        assert result == False  # Curse blocked
        assert curse not in player.deck
        assert player.omamori_charges == 1

    def test_omamori_blocks_two_curses(self):
        """Omamori can block 2 curses total."""
        player = MockPlayer()
        player.omamori_charges = 2
        player.add_curse_to_deck(get_card("Injury"))
        player.add_curse_to_deck(get_card("Decay"))
        assert player.omamori_charges == 0
        assert len(player.deck) == 0

    def test_omamori_third_curse_goes_through(self):
        """Third curse is not blocked after Omamori charges depleted."""
        player = MockPlayer()
        player.omamori_charges = 2
        player.add_curse_to_deck(get_card("Injury"))
        player.add_curse_to_deck(get_card("Decay"))
        third_curse = get_card("Doubt")
        player.add_curse_to_deck(third_curse)
        assert player.omamori_charges == 0
        assert third_curse in player.deck

    def test_omamori_has_2_charges_by_default(self):
        """Omamori starts with 2 charges."""
        relic = get_relic("Omamori")
        assert relic.counter_start == 2


# =============================================================================
# SECTION 10: RELIC INTERACTIONS - DARKSTONE PERIAPT
# =============================================================================

class TestDarkstonePeriapt:
    """Test Darkstone Periapt relic interactions with curses."""

    def test_darkstone_periapt_grants_max_hp_on_curse(self):
        """Darkstone Periapt grants 6 max HP when obtaining a curse."""
        player = MockPlayer(max_hp=80)
        player.relics.append(get_relic("Darkstone Periapt"))
        curse = get_card("Injury")
        player.add_curse_to_deck(curse)
        assert player.max_hp == 86

    def test_darkstone_periapt_stacks_with_multiple_curses(self):
        """Darkstone Periapt grants max HP for each curse obtained."""
        player = MockPlayer(max_hp=80)
        player.relics.append(get_relic("Darkstone Periapt"))
        player.add_curse_to_deck(get_card("Injury"))
        player.add_curse_to_deck(get_card("Decay"))
        player.add_curse_to_deck(get_card("Doubt"))
        assert player.max_hp == 98  # 80 + 6*3

    def test_darkstone_periapt_relic_exists(self):
        """Darkstone Periapt relic exists with correct effect description."""
        relic = get_relic("Darkstone Periapt")
        assert any("6 Max HP" in effect for effect in relic.effects)


# =============================================================================
# SECTION 11: RELIC INTERACTIONS - MEDICAL KIT
# =============================================================================

class TestMedicalKit:
    """Test Medical Kit relic interactions with status cards."""

    def test_medical_kit_allows_status_play(self):
        """Medical Kit allows playing status cards (they exhaust)."""
        relic = get_relic("Medical Kit")
        # Check the effect description mentions status playing
        assert any("Status" in effect and "played" in effect for effect in relic.effects)

    def test_medical_kit_exists_as_shop_relic(self):
        """Medical Kit relic exists as shop tier."""
        from core.content.relics import RelicTier
        relic = get_relic("Medical Kit")
        assert relic.tier == RelicTier.SHOP


# =============================================================================
# SECTION 12: RELIC INTERACTIONS - DU-VU DOLL
# =============================================================================

class TestDuVuDoll:
    """Test Du-Vu Doll relic synergy with curses."""

    def test_du_vu_doll_effect_description(self):
        """Du-Vu Doll gains 1 Strength per curse in deck."""
        relic = get_relic("Du-Vu Doll")
        assert any("Strength" in effect and "Curse" in effect for effect in relic.effects)

    def test_du_vu_doll_exists_as_rare_relic(self):
        """Du-Vu Doll exists as rare tier."""
        from core.content.relics import RelicTier
        relic = get_relic("Du-Vu Doll")
        assert relic.tier == RelicTier.RARE


# =============================================================================
# SECTION 13: STATUS CARD REGISTRY
# =============================================================================

class TestStatusCardRegistry:
    """Test status card registry completeness."""

    def test_status_registry_has_5_cards(self):
        """Status registry should have exactly 5 cards."""
        assert len(STATUS_CARDS) == 5

    def test_status_registry_contains_wound(self):
        """Status registry contains Wound."""
        assert "Wound" in STATUS_CARDS

    def test_status_registry_contains_dazed(self):
        """Status registry contains Dazed."""
        assert "Dazed" in STATUS_CARDS

    def test_status_registry_contains_burn(self):
        """Status registry contains Burn."""
        assert "Burn" in STATUS_CARDS

    def test_status_registry_contains_void(self):
        """Status registry contains Void."""
        assert "Void" in STATUS_CARDS

    def test_status_registry_contains_slimed(self):
        """Status registry contains Slimed."""
        assert "Slimed" in STATUS_CARDS

    def test_all_status_cards_are_status_type(self):
        """All cards in status registry are STATUS type."""
        for card_id, card in STATUS_CARDS.items():
            assert card.card_type == CardType.STATUS, f"{card_id} should be STATUS type"


# =============================================================================
# SECTION 14: CURSE CARD REGISTRY
# =============================================================================

class TestCurseCardRegistry:
    """Test curse card registry completeness."""

    def test_curse_registry_has_14_cards(self):
        """Curse registry should have 14 cards."""
        assert len(CURSE_CARDS) == 14

    def test_curse_registry_contains_all_curses(self):
        """Curse registry contains all expected curses."""
        expected = [
            "AscendersBane", "Clumsy", "CurseOfTheBell", "Decay",
            "Doubt", "Injury", "Necronomicurse", "Normality",
            "Pain", "Parasite", "Pride", "Regret", "Shame", "Writhe"
        ]
        for curse_id in expected:
            assert curse_id in CURSE_CARDS, f"Missing curse: {curse_id}"

    def test_all_curse_cards_are_curse_type(self):
        """All cards in curse registry are CURSE type."""
        for card_id, card in CURSE_CARDS.items():
            assert card.card_type == CardType.CURSE, f"{card_id} should be CURSE type"

    def test_all_curses_have_curse_color(self):
        """All curse cards have CURSE color."""
        for card_id, card in CURSE_CARDS.items():
            assert card.color == CardColor.CURSE, f"{card_id} should have CURSE color"


# =============================================================================
# SECTION 15: INNATE CURSE BEHAVIOR
# =============================================================================

class TestInnateCurseBehavior:
    """Test innate curses like Writhe."""

    def test_writhe_is_innate(self):
        """Writhe has innate flag set."""
        writhe = get_card("Writhe")
        assert writhe.innate == True

    def test_pride_is_innate(self):
        """Pride has innate flag set."""
        pride = get_card("Pride")
        assert pride.innate == True

    def test_most_curses_not_innate(self):
        """Most curses are not innate."""
        non_innate_curses = [
            "AscendersBane", "Clumsy", "Decay", "Doubt",
            "Injury", "Necronomicurse", "Normality", "Pain",
            "Parasite", "Regret", "Shame"
        ]
        for curse_id in non_innate_curses:
            curse = get_card(curse_id)
            assert curse.innate == False, f"{curse_id} should not be innate"


# =============================================================================
# SECTION 16: PRIDE SPECIAL CURSE
# =============================================================================

class TestPrideCurse:
    """Test Pride curse special behavior."""

    def test_pride_is_playable(self):
        """Pride is playable unlike most curses (costs 1)."""
        pride = get_card("Pride")
        assert pride.cost == 1

    def test_pride_exhausts(self):
        """Pride exhausts when played."""
        pride = get_card("Pride")
        assert pride.exhaust == True

    def test_pride_adds_copy_at_end_of_turn(self):
        """Pride adds a copy to draw pile at end of turn."""
        pride = get_card("Pride")
        assert "end_of_turn_add_copy_to_draw" in pride.effects


# =============================================================================
# SECTION 17: CURSE OF THE BELL
# =============================================================================

class TestCurseOfTheBell:
    """Test Curse of the Bell special behavior."""

    def test_curse_of_the_bell_cannot_be_removed(self):
        """Curse of the Bell cannot be removed."""
        curse = get_card("CurseOfTheBell")
        assert "cannot_be_removed" in curse.effects

    def test_curse_of_the_bell_is_special_rarity(self):
        """Curse of the Bell is SPECIAL rarity (from Calling Bell)."""
        curse = get_card("CurseOfTheBell")
        assert curse.rarity == CardRarity.SPECIAL


# =============================================================================
# SECTION 18: SLIMED SPECIAL BEHAVIOR
# =============================================================================

class TestSlimedBehavior:
    """Test Slimed special behavior as the only playable status."""

    def test_slimed_costs_1_energy(self):
        """Slimed costs 1 energy to play."""
        slimed = get_card("Slimed")
        assert slimed.cost == 1

    def test_slimed_exhausts_when_played(self):
        """Slimed exhausts when played."""
        slimed = get_card("Slimed")
        assert slimed.exhaust == True

    def test_slimed_has_no_effects(self):
        """Slimed has no effects (just wastes energy)."""
        slimed = get_card("Slimed")
        assert len(slimed.effects) == 0

    def test_slimed_targets_self(self):
        """Slimed targets self (no enemy target needed)."""
        slimed = get_card("Slimed")
        assert slimed.target == CardTarget.SELF


# =============================================================================
# SECTION 19: COMBINED END OF TURN SCENARIOS
# =============================================================================

class TestCombinedEndOfTurnScenarios:
    """Test scenarios with multiple status/curse end of turn effects."""

    def test_burn_and_decay_stack_damage(self):
        """Burn and Decay damage stacks at end of turn."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Burn"))
        player.add_to_hand(get_card("Decay"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.hp == 46  # 2 + 2 damage

    def test_regret_with_multiple_curses_in_hand(self):
        """Regret counts all cards including other curses."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Regret"))
        player.add_to_hand(get_card("Decay"))
        player.add_to_hand(get_card("Shame"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        # Regret loses 3 HP (3 cards), Decay deals 2, Shame applies frail
        assert player.hp == 45  # 50 - 3 (regret) - 2 (decay) = 45
        assert player.powers.get("Frail", 0) == 1

    def test_doubt_and_shame_both_apply(self):
        """Doubt and Shame both apply their debuffs."""
        player = MockPlayer(hp=50)
        player.add_to_hand(get_card("Doubt"))
        player.add_to_hand(get_card("Shame"))
        combat = MockCombatState(player)
        combat.end_of_turn()
        assert player.powers.get("Weak", 0) == 1
        assert player.powers.get("Frail", 0) == 1


# =============================================================================
# SECTION 20: COPY BEHAVIOR
# =============================================================================

class TestCardCopyBehavior:
    """Test that card copying works correctly for status/curse cards."""

    def test_status_card_copy_maintains_properties(self):
        """Copied status cards maintain all properties."""
        original = get_card("Burn")
        copy = original.copy()
        assert copy.id == original.id
        assert copy.card_type == original.card_type
        assert copy.ethereal == original.ethereal
        assert copy.effects == original.effects
        assert copy is not original

    def test_curse_card_copy_maintains_properties(self):
        """Copied curse cards maintain all properties."""
        original = get_card("Necronomicurse")
        copy = original.copy()
        assert copy.id == original.id
        assert copy.card_type == original.card_type
        assert copy.effects == original.effects
        assert "returns_when_exhausted_or_removed" in copy.effects
        assert copy is not original

    def test_upgraded_burn_copy_maintains_upgrade(self):
        """Copied upgraded Burn maintains upgraded state."""
        original = get_card("Burn", upgraded=True)
        copy = original.copy()
        assert copy.upgraded == True
        assert copy.magic_number == 4
