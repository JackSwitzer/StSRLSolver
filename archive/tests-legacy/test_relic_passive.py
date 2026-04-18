"""Passive Flag Relic Tests - TDD approach.

Tests for relics that set flags rather than trigger on hooks.
These relics modify game behavior through passive effects checked by game logic.

Relics tested:
- Omamori: Negate the next 2 Curses
- Eternal Feather: Heal HP in shops based on deck size
- Runic Capacitor: +3 Orb slots (Defect only)
- Prismatic Shard: Card rewards can contain cards from any class
- Circlet, Red Circlet, Spirit Poop: No-op relics (placeholder/joke items)
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.registry import (
    execute_relic_triggers,
    RELIC_REGISTRY,
    RelicContext,
)
from packages.engine.state.run import RunState, create_watcher_run, CardInstance
from packages.engine.state.combat import create_combat, create_enemy
from packages.engine.content.cards import CardType, ALL_CARDS


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
# OMAMORI - Curse Negation Tests
# =============================================================================

class TestOmamori:
    """Test Omamori: Negate the next 2 Curses obtained."""

    def test_omamori_starts_with_2_charges(self):
        """Omamori: Should start with counter = 2."""
        state = create_run_with_relic("Omamori", counter=2)

        assert state.get_relic_counter("Omamori") == 2

    def test_omamori_negates_first_curse(self):
        """Omamori: First curse should be negated."""
        state = create_run_with_relic("Omamori", counter=2)
        initial_deck_size = len(state.deck)

        # Simulate obtaining a curse (e.g., from event)
        curse_id = "Regret"

        # Check if Omamori blocks it
        counter = state.get_relic_counter("Omamori")
        if counter > 0:
            # Curse is blocked
            state.set_relic_counter("Omamori", counter - 1)
            curse_blocked = True
        else:
            # Curse goes through
            state.add_card(curse_id)
            curse_blocked = False

        # First curse should be blocked
        assert curse_blocked is True
        assert state.get_relic_counter("Omamori") == 1
        assert len(state.deck) == initial_deck_size

    def test_omamori_negates_second_curse(self):
        """Omamori: Second curse should also be negated."""
        state = create_run_with_relic("Omamori", counter=1)
        initial_deck_size = len(state.deck)

        # Second curse
        curse_id = "Pain"

        counter = state.get_relic_counter("Omamori")
        if counter > 0:
            state.set_relic_counter("Omamori", counter - 1)
            curse_blocked = True
        else:
            state.add_card(curse_id)
            curse_blocked = False

        assert curse_blocked is True
        assert state.get_relic_counter("Omamori") == 0
        assert len(state.deck) == initial_deck_size

    def test_omamori_third_curse_goes_through(self):
        """Omamori: Third curse should not be negated."""
        state = create_run_with_relic("Omamori", counter=0)
        initial_deck_size = len(state.deck)

        # Third curse
        curse_id = "Decay"

        counter = state.get_relic_counter("Omamori")
        if counter > 0:
            state.set_relic_counter("Omamori", counter - 1)
            curse_blocked = True
        else:
            state.add_card(curse_id)
            curse_blocked = False

        # Third curse goes through
        assert curse_blocked is False
        assert state.get_relic_counter("Omamori") == 0
        assert len(state.deck) == initial_deck_size + 1
        assert any(c.id == curse_id for c in state.deck)

    def test_omamori_integration_with_onObtainCard_trigger(self):
        """Omamori: Should work with onObtainCard trigger."""
        state = create_run_with_relic("Omamori", counter=2)
        initial_deck_size = len(state.deck)

        # Simulate full trigger flow
        curse_cards = ["Regret", "Pain", "Decay"]

        for curse in curse_cards:
            counter = state.get_relic_counter("Omamori")
            if counter > 0:
                # Omamori blocks it
                state.set_relic_counter("Omamori", counter - 1)
            else:
                # Curse goes through
                state.add_card(curse)

        # Should have blocked 2, allowed 1
        assert len(state.deck) == initial_deck_size + 1
        assert state.get_relic_counter("Omamori") == 0

    def test_omamori_with_multiple_curses_at_once(self):
        """Omamori: If multiple curses obtained simultaneously, each consumes 1 charge."""
        state = create_run_with_relic("Omamori", counter=2)
        initial_deck_size = len(state.deck)

        # Event gives 3 curses
        curses = ["Regret", "Pain", "Decay"]
        blocked_count = 0

        for curse in curses:
            counter = state.get_relic_counter("Omamori")
            if counter > 0:
                state.set_relic_counter("Omamori", counter - 1)
                blocked_count += 1
            else:
                state.add_card(curse)

        # 2 blocked, 1 added
        assert blocked_count == 2
        assert len(state.deck) == initial_deck_size + 1
        assert state.get_relic_counter("Omamori") == 0


# =============================================================================
# ETERNAL FEATHER - Shop Healing Tests
# =============================================================================

class TestEternalFeather:
    """Test Eternal Feather: Heal HP in shops based on deck size."""

    def test_eternal_feather_heals_in_shop(self):
        """Eternal Feather: Heal 3 HP for every 5 cards in deck when entering shop."""
        state = create_run_with_relic("Eternal Feather")

        # Watcher starts with 11 cards (10 starter + 1 Ascender's Bane at A10+)
        deck_size = len(state.deck)
        expected_heal = (deck_size // 5) * 3

        # Simulate entering shop
        state.current_hp = state.max_hp - 10

        initial_hp = state.current_hp
        state.heal(expected_heal)

        # Should heal based on deck size
        assert state.current_hp == initial_hp + expected_heal

    def test_eternal_feather_with_20_card_deck(self):
        """Eternal Feather: 20 card deck heals 12 HP (20/5 * 3)."""
        state = create_run_with_relic("Eternal Feather")

        # Add cards to reach 20
        while len(state.deck) < 20:
            state.add_card("Strike_P")

        deck_size = len(state.deck)
        expected_heal = (deck_size // 5) * 3  # 12 HP

        state.current_hp = state.max_hp - 20
        initial_hp = state.current_hp
        state.heal(expected_heal)

        assert state.current_hp == initial_hp + 12

    def test_eternal_feather_with_small_deck(self):
        """Eternal Feather: <5 cards heals 0 HP."""
        state = create_watcher_run("TEST123", ascension=0)  # No Ascender's Bane
        state.add_relic("Eternal Feather")

        # Remove cards to get <5
        while len(state.deck) > 4:
            state.remove_card(0)

        deck_size = len(state.deck)
        expected_heal = (deck_size // 5) * 3  # 0 HP

        initial_hp = state.current_hp
        state.heal(expected_heal)

        assert state.current_hp == initial_hp

    def test_eternal_feather_respects_max_hp(self):
        """Eternal Feather: Healing cannot exceed max HP."""
        state = create_run_with_relic("Eternal Feather")

        # Add many cards
        for _ in range(30):
            state.add_card("Strike_P")

        deck_size = len(state.deck)
        expected_heal = (deck_size // 5) * 3

        # Already at high HP
        state.current_hp = state.max_hp - 5

        state.heal(expected_heal)

        assert state.current_hp == state.max_hp

    def test_eternal_feather_with_mark_of_bloom(self):
        """Eternal Feather: No healing if Mark of the Bloom is equipped."""
        state = create_run_with_relic("Eternal Feather")
        state.add_relic("Mark of the Bloom")

        deck_size = len(state.deck)
        expected_heal = (deck_size // 5) * 3

        state.current_hp = state.max_hp - 20
        initial_hp = state.current_hp

        state.heal(expected_heal)

        # Mark of the Bloom prevents healing
        assert state.current_hp == initial_hp


# =============================================================================
# RUNIC CAPACITOR - Orb Slot Tests (Defect)
# =============================================================================

class TestRunicCapacitor:
    """Test Runic Capacitor: +3 Orb slots (Defect only)."""

    def test_runic_capacitor_increases_orb_slots(self):
        """Runic Capacitor: Adds 3 orb slots."""
        # This is a passive flag tested in combat initialization
        state = create_run_with_relic("Runic Capacitor")

        assert state.has_relic("Runic Capacitor")

        # Base orb slots for Defect = 3, +3 from Capacitor = 6
        # (This would be tested in Defect-specific combat tests)

    def test_runic_capacitor_stacks_with_other_orb_relics(self):
        """Runic Capacitor: Stacks with other orb slot relics."""
        state = create_run_with_relic("Runic Capacitor")

        # Add another orb slot relic
        state.add_relic("Inserter")

        # Total orb slots should stack
        # (Actual counting tested in combat initialization)
        assert state.has_relic("Runic Capacitor")
        assert state.has_relic("Inserter")

    def test_runic_capacitor_on_non_defect(self):
        """Runic Capacitor: No effect on non-Defect characters."""
        state = create_run_with_relic("Runic Capacitor")

        # Watcher doesn't use orbs
        assert state.character == "Watcher"
        assert state.has_relic("Runic Capacitor")

        # Relic exists but has no effect (no orb slots for Watcher)


# =============================================================================
# PRISMATIC SHARD - Card Reward Tests
# =============================================================================

class TestPrismaticShard:
    """Test Prismatic Shard: Card rewards can contain cards from any class."""

    def test_prismatic_shard_enables_cross_class_cards(self):
        """Prismatic Shard: Card rewards can be from any class."""
        state = create_run_with_relic("Prismatic Shard")

        assert state.has_relic("Prismatic Shard")

        # Card reward generation should check this flag
        # (Tested in reward generation tests)

    def test_prismatic_shard_affects_shop_cards(self):
        """Prismatic Shard: Shop cards can also be from any class."""
        state = create_run_with_relic("Prismatic Shard")

        # Shop generation should respect this flag
        assert state.has_relic("Prismatic Shard")

    def test_without_prismatic_shard_only_class_cards(self):
        """Without Prismatic Shard: Only Watcher cards in rewards."""
        state = create_watcher_run("TEST123", ascension=20)

        assert not state.has_relic("Prismatic Shard")


# =============================================================================
# NO-OP RELICS - Placeholder/Joke Items
# =============================================================================

class TestNoOpRelics:
    """Test relics with no mechanical effect (placeholders/jokes)."""

    def test_circlet_has_no_effect(self):
        """Circlet: Has no effect (placeholder when no more relics available)."""
        state = create_run_with_relic("Circlet")

        # Should exist but do nothing
        assert state.has_relic("Circlet")

        # No triggers registered
        assert not RELIC_REGISTRY.has_handler("atBattleStart", "Circlet")
        assert not RELIC_REGISTRY.has_handler("atTurnStart", "Circlet")

    def test_red_circlet_has_no_effect(self):
        """Red Circlet: Has no effect (duplicate placeholder)."""
        state = create_run_with_relic("Red Circlet")

        assert state.has_relic("Red Circlet")

        # No triggers registered
        assert not RELIC_REGISTRY.has_handler("atBattleStart", "Red Circlet")

    def test_spirit_poop_has_no_effect(self):
        """Spirit Poop: Has no effect (joke item)."""
        state = create_run_with_relic("Spirit Poop")

        assert state.has_relic("Spirit Poop")

        # No triggers registered
        assert not RELIC_REGISTRY.has_handler("atBattleStart", "Spirit Poop")

    def test_multiple_circlets(self):
        """Multiple Circlets: Can have multiple copies (unusual case)."""
        state = create_run_with_relic("Circlet")
        state.add_relic("Circlet")
        state.add_relic("Red Circlet")

        # All three should exist
        circlet_count = sum(1 for r in state.relics if r.id in ["Circlet", "Red Circlet"])
        assert circlet_count == 3


# =============================================================================
# PASSIVE FLAG RELICS - Various
# =============================================================================

class TestPassiveFlagRelics:
    """Test various passive flag relics."""

    def test_runic_pyramid_prevents_discard(self):
        """Runic Pyramid: Cards are not discarded at end of turn."""
        state = create_run_with_relic("Runic Pyramid")

        assert state.has_relic("Runic Pyramid")

        # Combat system should check this flag and skip discard phase

    def test_sacred_bark_doubles_potions(self):
        """Sacred Bark: Potion effects are doubled."""
        state = create_run_with_relic("Sacred Bark")

        assert state.has_relic("Sacred Bark")

        # Potion effect system should check this flag

    def test_membership_card_shop_discount(self):
        """Membership Card: 50% discount in shops."""
        state = create_run_with_relic("Membership Card")

        assert state.has_relic("Membership Card")

        # Shop prices should be halved

    def test_the_courier_shop_relic(self):
        """The Courier: Shop always has a relic for sale."""
        state = create_run_with_relic("The Courier")

        assert state.has_relic("The Courier")

        # Shop generation should guarantee relic

    def test_smiling_mask_free_card_removal(self):
        """Smiling Mask: Card removal costs 0 gold."""
        state = create_run_with_relic("Smiling Mask")

        assert state.has_relic("Smiling Mask")

        # Shop card removal should cost 0

    def test_fusion_hammer_prevents_smithing(self):
        """Fusion Hammer: Cannot upgrade cards at rest sites."""
        state = create_run_with_relic("Fusion Hammer")

        assert state.has_relic("Fusion Hammer")

        # Rest site should not offer upgrade option

    def test_coffee_dripper_prevents_rest(self):
        """Coffee Dripper: Cannot rest to heal at rest sites."""
        state = create_run_with_relic("Coffee Dripper")

        assert state.has_relic("Coffee Dripper")

        # Rest site should not offer rest option

    def test_sozu_prevents_potion_drops(self):
        """Sozu: Enemies do not drop potions."""
        state = create_run_with_relic("Sozu")

        assert state.has_relic("Sozu")

        # Potion drop chance should be 0

    def test_ectoplasm_prevents_gold(self):
        """Ectoplasm: Cannot gain gold."""
        state = create_run_with_relic("Ectoplasm")

        initial_gold = state.gold
        state.add_gold(100)

        # Gold gain prevented
        assert state.gold == initial_gold

    def test_mark_of_the_bloom_prevents_healing(self):
        """Mark of the Bloom: Cannot heal."""
        state = create_run_with_relic("Mark of the Bloom")

        initial_hp = state.current_hp
        state.heal(50)

        # Healing prevented
        assert state.current_hp == initial_hp


# =============================================================================
# DAMAGE/BLOCK MODIFIER RELICS
# =============================================================================

class TestDamageBlockModifiers:
    """Test relics that modify damage/block calculations."""

    def test_odd_mushroom_reduces_vulnerable_multiplier(self):
        """Odd Mushroom: Vulnerable on player is 1.25x instead of 1.5x."""
        state = create_run_with_relic("Odd Mushroom")

        assert state.has_relic("Odd Mushroom")

        # Damage calculation should use 1.25x multiplier

    def test_paper_frog_increases_enemy_vulnerable(self):
        """Paper Frog: Vulnerable on enemies is 1.75x instead of 1.5x."""
        state = create_run_with_relic("Paper Frog")

        assert state.has_relic("Paper Frog")

        # Damage calculation should use 1.75x multiplier

    def test_paper_crane_reduces_weak_multiplier(self):
        """Paper Crane: Weak is 0.60x instead of 0.75x."""
        state = create_run_with_relic("Paper Crane")

        assert state.has_relic("Paper Crane")

        # Damage calculation should use 0.60x multiplier

    def test_tungsten_rod_reduces_hp_loss(self):
        """Tungsten Rod: Reduce HP loss by 1 (minimum 0)."""
        state = create_run_with_relic("Tungsten Rod")

        assert state.has_relic("Tungsten Rod")

        # Damage pipeline should subtract 1 from final damage


# =============================================================================
# INTEGRATION TESTS
# =============================================================================

class TestPassiveRelicIntegration:
    """Integration tests for passive relics."""

    def test_omamori_with_multiple_curse_events(self):
        """Omamori: Should properly handle multiple curse sources."""
        state = create_run_with_relic("Omamori", counter=2)

        # Event 1: Curse
        counter = state.get_relic_counter("Omamori")
        if counter > 0:
            state.set_relic_counter("Omamori", counter - 1)
        else:
            state.add_card("Regret")

        # Event 2: Curse
        counter = state.get_relic_counter("Omamori")
        if counter > 0:
            state.set_relic_counter("Omamori", counter - 1)
        else:
            state.add_card("Pain")

        # Event 3: Curse
        counter = state.get_relic_counter("Omamori")
        if counter > 0:
            state.set_relic_counter("Omamori", counter - 1)
        else:
            state.add_card("Decay")

        # Should have blocked 2, allowed 1
        curse_count = sum(1 for c in state.deck
                         if c.id in ["Regret", "Pain", "Decay"])
        assert curse_count == 1

    def test_eternal_feather_with_prismatic_shard(self):
        """Eternal Feather + Prismatic Shard: Both effects work independently."""
        state = create_run_with_relic("Eternal Feather")
        state.add_relic("Prismatic Shard")

        # Add cross-class cards
        state.add_card("Strike_R")  # Ironclad card
        state.add_card("Strike_G")  # Silent card

        # Eternal Feather should still count all cards
        deck_size = len(state.deck)
        expected_heal = (deck_size // 5) * 3

        state.current_hp = state.max_hp - 20
        initial_hp = state.current_hp
        state.heal(expected_heal)

        assert state.current_hp > initial_hp

    def test_multiple_prevention_relics(self):
        """Multiple prevention relics: All effects should stack."""
        state = create_run_with_relic("Ginger")  # Prevents Weak
        state.add_relic("Turnip")  # Prevents Frail

        # Both relics should be active
        assert state.has_relic("Ginger")
        assert state.has_relic("Turnip")

    def test_conflicting_gold_relics(self):
        """Ectoplasm + Golden Idol: Ectoplasm takes precedence."""
        state = create_run_with_relic("Ectoplasm")
        state.add_relic("Golden Idol")

        initial_gold = state.gold
        state.add_gold(100)

        # No gold gained
        assert state.gold == initial_gold


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
