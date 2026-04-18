#!/usr/bin/env python3
"""
Interactive Seed Verifier

Step through a game interactively, telling the tool what room type you entered.
The tool predicts card rewards and tracks state.
"""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from typing import List, Optional
from packages.engine.state.game_rng import GameRNGState, RNGStream
from packages.engine.generation.rewards import generate_card_rewards, RewardState


class InteractiveVerifier:
    """
    Interactive verification tool.

    Usage:
        verifier = InteractiveVerifier("TEST123", neow="BOSS_SWAP")
        verifier.enter_combat("monster")  # Get prediction, compare
        verifier.enter_shop()
        verifier.enter_event("The Cleric")
        verifier.enter_combat("elite")
        ...
    """

    def __init__(self, seed: str, neow: str = "NONE", ascension: int = 20):
        self.seed = seed.upper()
        self.ascension = ascension
        self.act = 1
        self.floor = 0

        # Initialize RNG
        self.rng = GameRNGState(self.seed)
        if neow != "NONE":
            self.rng.apply_neow_choice(neow)
            print(f"Applied Neow: {neow}")

        self.reward_state = RewardState()

        # Tracking
        self.history = []
        self.mismatches = []

        print(f"\n{'='*60}")
        print(f"INTERACTIVE SEED VERIFIER")
        print(f"{'='*60}")
        print(f"Seed: {self.seed}")
        print(f"Neow: {neow}")
        print(f"Ascension: {ascension}")
        print(f"Initial cardRng: {self.rng.get_counter(RNGStream.CARD)}")
        print(f"{'='*60}\n")

    def get_card_counter(self) -> int:
        return self.rng.get_counter(RNGStream.CARD)

    def enter_floor(self, floor: int):
        """Enter a new floor."""
        self.floor = floor
        if floor == 17:
            self.act = 2
            self.rng.transition_to_next_act()
            print(f"  [ACT 2] cardRng snapped to: {self.get_card_counter()}")
        elif floor == 34:
            self.act = 3
            self.rng.transition_to_next_act()
            print(f"  [ACT 3] cardRng snapped to: {self.get_card_counter()}")
        elif floor == 52:
            self.act = 4
            self.rng.transition_to_next_act()
            print(f"  [ACT 4] cardRng snapped to: {self.get_card_counter()}")

    def enter_combat(
        self,
        room_type: str = "monster",
        num_cards: int = 3,
    ) -> List[str]:
        """
        Enter a combat room and get card reward prediction.

        Args:
            room_type: "monster", "elite", or "boss"
            num_cards: Number of cards (usually 3, but Question Card adds 1)

        Returns:
            List of predicted card names
        """
        counter_before = self.get_card_counter()
        card_rng = self.rng.get_rng(RNGStream.CARD)

        cards = generate_card_rewards(
            rng=card_rng,
            reward_state=self.reward_state,
            act=self.act,
            player_class="WATCHER",
            ascension=self.ascension,
            room_type="elite" if room_type == "elite" else "normal",
            num_cards=num_cards,
        )

        counter_after = card_rng.counter
        self.rng.set_counter(RNGStream.CARD, counter_after)

        card_names = [c.name for c in cards]

        print(f"Floor {self.floor} [{room_type.upper()}]")
        print(f"  cardRng: {counter_before} -> {counter_after}")
        print(f"  PREDICTION: {card_names}")

        self.history.append({
            "floor": self.floor,
            "type": room_type,
            "counter_before": counter_before,
            "counter_after": counter_after,
            "predicted": card_names,
        })

        return card_names

    def verify_combat(
        self,
        observed: List[str],
        room_type: str = "monster",
        num_cards: int = 3,
    ) -> bool:
        """
        Enter combat and verify against observed cards.

        Returns True if match.
        """
        predicted = self.enter_combat(room_type, num_cards)

        # Normalize for comparison
        pred_norm = set(c.lower().replace("+", "").strip() for c in predicted)
        obs_norm = set(c.lower().replace("+", "").strip() for c in observed)

        match = pred_norm == obs_norm

        if match:
            print(f"  ✓ MATCH!")
        else:
            print(f"  ✗ MISMATCH!")
            print(f"  OBSERVED: {observed}")
            self.mismatches.append({
                "floor": self.floor,
                "predicted": predicted,
                "observed": observed,
            })

        print()
        return match

    def enter_shop(self):
        """Enter a shop (consumes ~12 cardRng)."""
        counter_before = self.get_card_counter()
        self.rng.apply_shop()
        counter_after = self.get_card_counter()

        print(f"Floor {self.floor} [SHOP]")
        print(f"  cardRng: {counter_before} -> {counter_after} (+{counter_after - counter_before})")
        print()

        self.history.append({
            "floor": self.floor,
            "type": "shop",
            "counter_before": counter_before,
            "counter_after": counter_after,
        })

    def enter_event(self, name: str = ""):
        """Enter an event (usually no cardRng consumption)."""
        counter_before = self.get_card_counter()
        self.rng.apply_event(name)
        counter_after = self.get_card_counter()

        print(f"Floor {self.floor} [EVENT: {name or 'unknown'}]")
        if counter_after != counter_before:
            print(f"  cardRng: {counter_before} -> {counter_after}")
        else:
            print(f"  cardRng: {counter_before} (unchanged)")
        print()

        self.history.append({
            "floor": self.floor,
            "type": "event",
            "name": name,
            "counter_before": counter_before,
            "counter_after": counter_after,
        })

    def enter_rest(self):
        """Enter a rest site (no cardRng consumption)."""
        print(f"Floor {self.floor} [REST]")
        print(f"  cardRng: {self.get_card_counter()} (unchanged)")
        print()

        self.history.append({
            "floor": self.floor,
            "type": "rest",
        })

    def enter_treasure(self):
        """Enter a treasure room (uses treasureRng, not cardRng)."""
        counter_before = self.get_card_counter()
        self.rng.apply_treasure()

        print(f"Floor {self.floor} [TREASURE]")
        print(f"  cardRng: {counter_before} (unchanged - uses treasureRng)")
        print()

        self.history.append({
            "floor": self.floor,
            "type": "treasure",
        })

    def summary(self):
        """Print verification summary."""
        print(f"\n{'='*60}")
        print("VERIFICATION SUMMARY")
        print(f"{'='*60}")
        print(f"Seed: {self.seed}")
        print(f"Floors checked: {len(self.history)}")
        print(f"Final cardRng: {self.get_card_counter()}")

        combats = [h for h in self.history if h.get("type") in ["monster", "elite", "boss"]]
        print(f"Card rewards: {len(combats)}")
        print(f"Mismatches: {len(self.mismatches)}")

        if self.mismatches:
            print(f"\nMISMATCHES:")
            for m in self.mismatches:
                print(f"  Floor {m['floor']}:")
                print(f"    Predicted: {m['predicted']}")
                print(f"    Observed:  {m['observed']}")
        else:
            print(f"\n✓ ALL MATCHES!")


def verify_game6():
    """
    Verify the actual game 6 path we know from VOD extraction.
    """
    print("="*60)
    print("VERIFYING GAME 6 (seed: 33J85JVCVSPJY)")
    print("="*60)

    v = InteractiveVerifier("33J85JVCVSPJY", neow="BOSS_SWAP")

    # Floor 1: Monster - card reward
    v.enter_floor(1)
    v.verify_combat(
        ["Consecrate", "Meditate", "Foreign Influence"],
        room_type="monster"
    )

    # Floor 2: Event (The Cleric)
    v.enter_floor(2)
    v.enter_event("The Cleric")

    # Floor 3: Monster - card reward
    v.enter_floor(3)
    v.verify_combat(
        ["Consecrate", "Fasting", "Pressure Points"],
        room_type="monster"
    )

    # Floor 4: Shop
    v.enter_floor(4)
    v.enter_shop()

    # Floor 5: Event (Living Wall)
    v.enter_floor(5)
    v.enter_event("Living Wall")

    # Floor 6: Rest
    v.enter_floor(6)
    v.enter_rest()

    # Floor 7: Elite (Sentries) - card reward
    v.enter_floor(7)
    v.verify_combat(
        ["Third Eye", "Protect", "Prostrate"],
        room_type="elite"
    )

    # Floor 8: Monster - card reward (this is where divergence started in VOD)
    v.enter_floor(8)
    v.verify_combat(
        # From VOD extraction (may be wrong due to Gemini errors)
        ["Protect", "Reach Heaven", "Pray"],  # Gemini said "Defend" but we correct to Protect
        room_type="monster"
    )

    v.summary()


# =============================================================================
# CLI
# =============================================================================

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Interactive seed verification")
    parser.add_argument("--verify-game6", action="store_true", help="Verify game 6 path")
    parser.add_argument("--seed", help="Seed to verify")
    parser.add_argument("--neow", default="NONE", help="Neow choice")

    args = parser.parse_args()

    if args.verify_game6:
        verify_game6()
    elif args.seed:
        v = InteractiveVerifier(args.seed, neow=args.neow)
        print("\nVerifier ready. Use in Python REPL:")
        print("  v.enter_floor(1)")
        print("  v.enter_combat('monster')")
        print("  v.verify_combat(['Card1', 'Card2', 'Card3'])")
        print("  v.enter_shop()")
        print("  v.enter_event('Event Name')")
        print("  v.summary()")
    else:
        verify_game6()
