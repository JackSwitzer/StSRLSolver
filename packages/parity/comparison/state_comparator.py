#!/usr/bin/env python3
"""
State Comparator - Compare Python emulator state against actual game state.

This is the core tool for achieving full fidelity simulation.
Run the same seed in both Python emulator and actual game, compare states at checkpoints.
"""

from dataclasses import dataclass, field
from typing import List, Dict, Optional, Any, Tuple
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.comparison.save_reader import SaveState, RNGState
from core.state.game_rng import GameRNGState, RNGStream
from core.state.rng import seed_to_long


@dataclass
class Discrepancy:
    """A single discrepancy between emulator and game."""
    category: str  # "rng", "deck", "relic", "hp", "gold", etc.
    field: str
    expected: Any  # Emulator value
    actual: Any    # Game value
    severity: str = "error"  # "error", "warning", "info"
    message: str = ""


@dataclass
class ComparisonResult:
    """Result of comparing emulator state against game state."""
    match: bool
    discrepancies: List[Discrepancy] = field(default_factory=list)
    floor: int = 0
    act: int = 1
    checkpoint: str = ""

    def add_discrepancy(
        self,
        category: str,
        field: str,
        expected: Any,
        actual: Any,
        severity: str = "error",
        message: str = "",
    ):
        self.discrepancies.append(Discrepancy(
            category=category,
            field=field,
            expected=expected,
            actual=actual,
            severity=severity,
            message=message,
        ))
        if severity == "error":
            self.match = False

    def error_count(self) -> int:
        return sum(1 for d in self.discrepancies if d.severity == "error")

    def warning_count(self) -> int:
        return sum(1 for d in self.discrepancies if d.severity == "warning")


class StateComparator:
    """
    Compare Python emulator state against actual game state.

    Usage:
        comparator = StateComparator()

        # Load game state from save file
        game_state = read_save_file("path/to/save.autosave")

        # Get emulator state
        emulator_rng = GameRNGState(seed_string)

        # Compare
        result = comparator.compare_rng(emulator_rng, game_state.rng)
        print(result)
    """

    def compare_rng(
        self,
        emulator: GameRNGState,
        game: RNGState,
    ) -> ComparisonResult:
        """
        Compare RNG counter states.

        This is the most critical comparison - if RNG counters match,
        all deterministic outcomes (card rewards, etc.) will match.
        """
        result = ComparisonResult(match=True, checkpoint="RNG")

        # Map emulator stream names to save file names
        stream_mapping = {
            RNGStream.CARD: ("card", "card_seed_count"),
            RNGStream.MONSTER: ("monster", "monster_seed_count"),
            RNGStream.EVENT: ("event", "event_seed_count"),
            RNGStream.RELIC: ("relic", "relic_seed_count"),
            RNGStream.TREASURE: ("treasure", "treasure_seed_count"),
            RNGStream.POTION: ("potion", "potion_seed_count"),
            RNGStream.MERCHANT: ("merchant", "merchant_seed_count"),
        }

        for stream, (name, save_field) in stream_mapping.items():
            emu_counter = emulator.get_counter(stream)
            game_counter = getattr(game, save_field, 0)

            if emu_counter != game_counter:
                diff = emu_counter - game_counter
                result.add_discrepancy(
                    category="rng",
                    field=name,
                    expected=emu_counter,
                    actual=game_counter,
                    message=f"{name}Rng: emulator={emu_counter}, game={game_counter} (diff={diff:+d})"
                )

        return result

    def compare_deck(
        self,
        emulator_deck: List[str],  # List of card IDs
        game_deck: List['CardSave'],
    ) -> ComparisonResult:
        """Compare deck contents."""
        result = ComparisonResult(match=True, checkpoint="Deck")

        # Convert to comparable format
        emu_cards = sorted(emulator_deck)
        game_cards = sorted([c.id for c in game_deck])

        if len(emu_cards) != len(game_cards):
            result.add_discrepancy(
                category="deck",
                field="count",
                expected=len(emu_cards),
                actual=len(game_cards),
                message=f"Deck size mismatch: emulator={len(emu_cards)}, game={len(game_cards)}"
            )

        # Find missing/extra cards
        emu_set = set(emu_cards)
        game_set = set(game_cards)

        missing = emu_set - game_set
        extra = game_set - emu_set

        for card in missing:
            result.add_discrepancy(
                category="deck",
                field="card",
                expected=card,
                actual=None,
                severity="error",
                message=f"Card in emulator but not game: {card}"
            )

        for card in extra:
            result.add_discrepancy(
                category="deck",
                field="card",
                expected=None,
                actual=card,
                severity="error",
                message=f"Card in game but not emulator: {card}"
            )

        return result

    def compare_relics(
        self,
        emulator_relics: List[str],
        game_relics: List[str],
    ) -> ComparisonResult:
        """Compare relic collections."""
        result = ComparisonResult(match=True, checkpoint="Relics")

        emu_set = set(emulator_relics)
        game_set = set(game_relics)

        if len(emulator_relics) != len(game_relics):
            result.add_discrepancy(
                category="relics",
                field="count",
                expected=len(emulator_relics),
                actual=len(game_relics),
            )

        missing = emu_set - game_set
        extra = game_set - emu_set

        for relic in missing:
            result.add_discrepancy(
                category="relics",
                field="relic",
                expected=relic,
                actual=None,
                message=f"Relic in emulator but not game: {relic}"
            )

        for relic in extra:
            result.add_discrepancy(
                category="relics",
                field="relic",
                expected=None,
                actual=relic,
                message=f"Relic in game but not emulator: {relic}"
            )

        return result

    def compare_resources(
        self,
        emulator: Dict[str, int],  # {"hp": x, "max_hp": y, "gold": z}
        game: SaveState,
    ) -> ComparisonResult:
        """Compare HP, gold, etc."""
        result = ComparisonResult(match=True, checkpoint="Resources")

        checks = [
            ("current_hp", emulator.get("hp"), game.current_hp),
            ("max_hp", emulator.get("max_hp"), game.max_hp),
            ("gold", emulator.get("gold"), game.gold),
            ("floor", emulator.get("floor"), game.floor),
            ("act", emulator.get("act"), game.act),
        ]

        for field, emu_val, game_val in checks:
            if emu_val is not None and emu_val != game_val:
                result.add_discrepancy(
                    category="resources",
                    field=field,
                    expected=emu_val,
                    actual=game_val,
                )

        return result

    def compare_full(
        self,
        emulator_rng: GameRNGState,
        emulator_deck: List[str],
        emulator_relics: List[str],
        emulator_resources: Dict[str, int],
        game_state: SaveState,
    ) -> ComparisonResult:
        """
        Full comparison of all state.

        Returns combined result with all discrepancies.
        """
        result = ComparisonResult(
            match=True,
            floor=game_state.floor,
            act=game_state.act,
            checkpoint="Full"
        )

        # Compare each component
        rng_result = self.compare_rng(emulator_rng, game_state.rng)
        deck_result = self.compare_deck(emulator_deck, game_state.deck)
        relic_result = self.compare_relics(emulator_relics, game_state.relics)
        resource_result = self.compare_resources(emulator_resources, game_state)

        # Combine discrepancies
        for r in [rng_result, deck_result, relic_result, resource_result]:
            result.discrepancies.extend(r.discrepancies)
            if not r.match:
                result.match = False

        return result


def print_comparison_result(result: ComparisonResult):
    """Pretty print comparison result."""
    status = "✓ MATCH" if result.match else "✗ MISMATCH"
    print(f"\n{'='*60}")
    print(f"Comparison Result: {status}")
    print(f"Checkpoint: {result.checkpoint}")
    if result.floor > 0:
        print(f"Act {result.act}, Floor {result.floor}")
    print(f"{'='*60}")

    if result.discrepancies:
        errors = [d for d in result.discrepancies if d.severity == "error"]
        warnings = [d for d in result.discrepancies if d.severity == "warning"]

        if errors:
            print(f"\nErrors ({len(errors)}):")
            for d in errors:
                print(f"  [{d.category}] {d.field}: expected={d.expected}, actual={d.actual}")
                if d.message:
                    print(f"           {d.message}")

        if warnings:
            print(f"\nWarnings ({len(warnings)}):")
            for d in warnings:
                print(f"  [{d.category}] {d.field}: {d.message or f'{d.expected} vs {d.actual}'}")
    else:
        print("\nAll checks passed!")


# =============================================================================
# CLI
# =============================================================================

if __name__ == "__main__":
    from core.comparison.save_reader import read_save_file, get_default_save_path

    print("State Comparator - Compare emulator vs game")
    print()

    # Try to read a save file
    save_path = get_default_save_path()

    try:
        game_state = read_save_file(save_path)
        print(f"Loaded save: {save_path}")
        print(f"Seed: {game_state.rng.seed}")
        print(f"Act {game_state.act}, Floor {game_state.floor}")
        print()

        # Create emulator state from same seed
        from core.state.rng import long_to_seed
        seed_string = long_to_seed(game_state.rng.seed)
        emulator_rng = GameRNGState(seed_string)

        # Compare RNG (emulator starts at 0, game has advanced)
        comparator = StateComparator()
        result = comparator.compare_rng(emulator_rng, game_state.rng)
        print_comparison_result(result)

        print("\n" + "="*60)
        print("Game RNG counters (what emulator needs to match):")
        for name, value in game_state.rng.to_dict().items():
            if name != "seed" and value > 0:
                print(f"  {name}: {value}")

    except FileNotFoundError:
        print(f"No save file found at: {save_path}")
        print("Play the game and create a save to test comparison.")
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
