#!/usr/bin/env python3
"""
Complete card implementation analysis for Slay the Spire Python engine.

This script:
1. Counts total cards defined for each character
2. Checks which cards have effects implemented
3. Compares with Java decompiled source to find missing cards
4. Outputs JSON report
"""

import json
import sys
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

try:
    from packages.engine.content.cards import (
        IRONCLAD_CARDS, SILENT_CARDS, DEFECT_CARDS, WATCHER_CARDS,
        COLORLESS_CARDS, CURSE_CARDS, STATUS_CARDS
    )
except ImportError as e:
    print(f"Error importing cards: {e}")
    print(f"Project root: {project_root}")
    print(f"sys.path: {sys.path}")
    sys.exit(1)


def analyze_cards(card_dict, character_name):
    """Analyze a card dictionary."""
    total = len(card_dict)
    implemented = []
    missing = []

    for card_id, card in card_dict.items():
        # Check if card has effects list populated
        has_effect = bool(getattr(card, 'effects', []))

        if has_effect:
            implemented.append(card_id)
        else:
            missing.append(card_id)

    return {
        "total": total,
        "implemented": len(implemented),
        "missing": sorted(missing),
        "percentage": round(len(implemented) / total * 100, 1) if total > 0 else 0
    }


def main():
    print("=== SLAY THE SPIRE CARD IMPLEMENTATION ANALYSIS ===\n")

    # Analyze each character
    results = {
        "ironclad": analyze_cards(IRONCLAD_CARDS, "Ironclad"),
        "silent": analyze_cards(SILENT_CARDS, "Silent"),
        "defect": analyze_cards(DEFECT_CARDS, "Defect"),
        "watcher": analyze_cards(WATCHER_CARDS, "Watcher"),
        "colorless": analyze_cards(COLORLESS_CARDS, "Colorless"),
        "curses": analyze_cards(CURSE_CARDS, "Curses"),
        "status": analyze_cards(STATUS_CARDS, "Status"),
    }

    # Calculate totals
    total_cards = sum(r["total"] for r in results.values())
    total_implemented = sum(r["implemented"] for r in results.values())
    total_missing = sum(len(r["missing"]) for r in results.values())

    # Print summary table
    print(f"{'Character':<15} | {'Total':>5} | {'Implemented':>12} | {'Missing':>7} | {'%':>6}")
    print("-" * 70)

    char_names = {
        "ironclad": "Ironclad",
        "silent": "Silent",
        "defect": "Defect",
        "watcher": "Watcher",
        "colorless": "Colorless",
        "curses": "Curses",
        "status": "Status"
    }

    for key in ["ironclad", "silent", "defect", "watcher", "colorless", "curses", "status"]:
        data = results[key]
        name = char_names[key]
        print(f"{name:<15} | {data['total']:>5} | {data['implemented']:>12} | {len(data['missing']):>7} | {data['percentage']:>5.1f}%")

    print("-" * 70)
    overall_pct = round(total_implemented / total_cards * 100, 1) if total_cards > 0 else 0
    print(f"{'TOTAL':<15} | {total_cards:>5} | {total_implemented:>12} | {total_missing:>7} | {overall_pct:>5.1f}%")

    print("\n=== SAMPLE MISSING CARDS ===\n")

    for key in ["ironclad", "silent", "defect", "watcher"]:
        data = results[key]
        if data["missing"]:
            print(f"{char_names[key]}:")
            for card in data["missing"][:15]:
                print(f"  - {card}")
            if len(data["missing"]) > 15:
                print(f"  ... and {len(data['missing']) - 15} more")
            print()

    # Save JSON output
    output_file = Path(__file__).parent / "inventory_cards.json"
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)

    print(f"\nFull report saved to: {output_file}")

    # Expected counts from Java decompiled source
    print("\n=== COMPARISON WITH JAVA SOURCE ===\n")
    expected = {
        "Watcher (purple)": 77,
        "Ironclad (red)": 75,
        "Silent (green)": 75,
        "Defect (blue)": 76
    }

    actual = {
        "Watcher (purple)": results["watcher"]["total"],
        "Ironclad (red)": results["ironclad"]["total"],
        "Silent (green)": results["silent"]["total"],
        "Defect (blue)": results["defect"]["total"]
    }

    for char, count in expected.items():
        actual_count = actual[char]
        diff = actual_count - count
        status = "✓" if diff == 0 else f"({diff:+d})"
        print(f"{char:<20} Expected: {count:>3} | Actual: {actual_count:>3} {status}")

    print("\nAnalysis complete!")


if __name__ == "__main__":
    main()
