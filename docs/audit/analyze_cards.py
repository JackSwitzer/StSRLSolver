#!/usr/bin/env python3
"""
Analyze card implementation status in the Slay the Spire Python engine.
"""

import json
import sys
from pathlib import Path

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from packages.engine.content.cards import (
    IRONCLAD_CARDS, SILENT_CARDS, DEFECT_CARDS, WATCHER_CARDS,
    COLORLESS_CARDS, CURSE_CARDS, STATUS_CARDS, Card
)
from packages.engine.effects.registry import _EFFECT_REGISTRY


def analyze_card_dict(card_dict: dict, character: str) -> dict:
    """Analyze a card dictionary for implementation status."""
    total = len(card_dict)
    implemented = []
    missing = []

    for card_id, card in card_dict.items():
        # Check if card has effects defined (list of effect names)
        has_effect = False

        # Cards have an 'effects' field which is a list of effect names
        if hasattr(card, 'effects') and card.effects:
            # Card has at least one effect defined
            has_effect = True
            # Optionally check if effects are registered
            # for effect_name in card.effects:
            #     if effect_name not in _EFFECT_REGISTRY:
            #         # Effect not registered
            #         pass

        if has_effect:
            implemented.append(card_id)
        else:
            missing.append(card_id)

    return {
        "character": character,
        "total": total,
        "implemented": len(implemented),
        "missing": sorted(missing),
        "implemented_cards": sorted(implemented)
    }


def main():
    """Main analysis function."""
    print("Analyzing card implementation status...")
    print(f"Total effects registered: {len(_EFFECT_REGISTRY)}")
    print(f"Registered effects: {sorted(_EFFECT_REGISTRY.keys())[:20]}...\n")

    results = {
        "ironclad": analyze_card_dict(IRONCLAD_CARDS, "Ironclad"),
        "silent": analyze_card_dict(SILENT_CARDS, "Silent"),
        "defect": analyze_card_dict(DEFECT_CARDS, "Defect"),
        "watcher": analyze_card_dict(WATCHER_CARDS, "Watcher"),
        "colorless": analyze_card_dict(COLORLESS_CARDS, "Colorless"),
        "curses": analyze_card_dict(CURSE_CARDS, "Curses"),
        "status": analyze_card_dict(STATUS_CARDS, "Status"),
    }

    # Print summary
    total_cards = sum(r["total"] for r in results.values())
    total_implemented = sum(r["implemented"] for r in results.values())
    total_missing = sum(len(r["missing"]) for r in results.values())

    print("\n=== CARD IMPLEMENTATION SUMMARY ===\n")
    for char_key, data in results.items():
        pct = (data["implemented"] / data["total"] * 100) if data["total"] > 0 else 0
        print(f"{data['character']:15} | Total: {data['total']:3} | Implemented: {data['implemented']:3} ({pct:5.1f}%) | Missing: {len(data['missing']):3}")

    print(f"\n{'TOTAL':15} | Total: {total_cards:3} | Implemented: {total_implemented:3} ({total_implemented/total_cards*100:5.1f}%) | Missing: {total_missing:3}")

    # Save to JSON
    output_path = Path(__file__).parent / "inventory_cards.json"

    # Simplify output for JSON (remove implemented_cards list to save space)
    json_output = {k: {kk: vv for kk, vv in v.items() if kk != "implemented_cards"}
                   for k, v in results.items()}

    with open(output_path, "w") as f:
        json.dump(json_output, f, indent=2)

    print(f"\nDetailed results saved to: {output_path}")

    # Print sample of missing cards for each character
    print("\n=== SAMPLE MISSING CARDS ===\n")
    for char_key, data in results.items():
        if data["missing"]:
            print(f"{data['character']}:")
            for card in data["missing"][:10]:
                print(f"  - {card}")
            if len(data["missing"]) > 10:
                print(f"  ... and {len(data['missing']) - 10} more")
            print()


if __name__ == "__main__":
    main()
