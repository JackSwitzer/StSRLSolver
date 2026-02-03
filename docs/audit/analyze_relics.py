#!/usr/bin/env python3
"""
Analyze relic implementation status in the Python engine.

Compares Python relic definitions against Java source to determine:
- Which relics are defined (data_only)
- Which have triggers implemented (partial or full)
- Which are missing entirely
"""

import sys
import json
from pathlib import Path
from typing import Dict, List, Set, Tuple

# Add packages to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "packages"))

from engine.content.relics import (
    ALL_RELICS, RelicTier,
    STARTER_RELICS, COMMON_RELICS, UNCOMMON_RELICS, RARE_RELICS,
    BOSS_RELICS, SHOP_RELICS, SPECIAL_RELICS
)

# Read combat.py and game.py to find implemented triggers
COMBAT_PY = Path(__file__).parent.parent.parent / "packages/engine/handlers/combat.py"
GAME_PY = Path(__file__).parent.parent.parent / "packages/engine/game.py"

def get_implemented_relics(file_path: Path) -> Set[str]:
    """Extract relic IDs that have has_relic() checks in the file."""
    relics = set()
    with open(file_path, 'r') as f:
        for line in f:
            if 'has_relic(' in line:
                # Extract relic ID from has_relic("RelicID")
                parts = line.split('has_relic(')
                if len(parts) > 1:
                    relic_part = parts[1].split(')')[0].strip('"\'')
                    relics.add(relic_part)
    return relics

def get_java_relics() -> Dict[str, str]:
    """Get all Java relic files and map to tier."""
    java_dir = Path(__file__).parent.parent.parent / "decompiled/java-src/com/megacrit/cardcrawl/relics"
    java_relics = {}

    if java_dir.exists():
        for java_file in java_dir.glob("*.java"):
            if java_file.stem not in ["AbstractRelic", "deprecated"]:
                java_relics[java_file.stem] = "UNKNOWN"

    return java_relics

def analyze_relic_status(relic_id: str, relic_def, combat_impls: Set[str], game_impls: Set[str]) -> str:
    """Determine implementation status of a relic."""
    # Check if relic has any triggers implemented
    is_in_combat = relic_id in combat_impls
    is_in_game = relic_id in game_impls

    # Passive relics (no triggers needed, just stat bonuses)
    passive_only = (
        relic_def.energy_bonus > 0 or
        relic_def.max_hp_bonus > 0 or
        relic_def.potion_slots > 0 or
        relic_def.card_draw_bonus > 0 or
        relic_def.hand_size_bonus > 0 or
        relic_def.orb_slots > 0 or
        relic_def.prevents_healing or
        relic_def.prevents_gold_gain or
        relic_def.prevents_potions or
        relic_def.prevents_resting or
        relic_def.prevents_smithing or
        relic_def.hides_intent
    )

    has_effects = len(relic_def.effects) > 0

    if passive_only and not has_effects:
        # Pure passive (like Potion Belt, Strawberry)
        return "full" if not is_in_combat and not is_in_game else "full"

    if is_in_combat or is_in_game:
        # Has some implementation
        # TODO: Could check if ALL effects are implemented vs SOME
        return "partial"  # Conservative: assume partial unless proven full

    if has_effects:
        # Has effects defined but no triggers implemented
        return "data_only"

    # Defined but minimal (might be placeholder)
    return "data_only"

def categorize_relics():
    """Categorize all relics by tier and implementation status."""
    combat_impls = get_implemented_relics(COMBAT_PY)
    game_impls = get_implemented_relics(GAME_PY)
    java_relics = get_java_relics()

    # All implemented relics
    all_impls = combat_impls | game_impls

    print(f"Found {len(combat_impls)} relics in combat.py")
    print(f"Found {len(game_impls)} relics in game.py")
    print(f"Total unique implemented: {len(all_impls)}")
    print(f"Java relics: {len(java_relics)}")

    # Categorize by tier
    tier_stats = {}

    for tier_name, relic_ids in [
        ("starter", STARTER_RELICS),
        ("common", COMMON_RELICS),
        ("uncommon", UNCOMMON_RELICS),
        ("rare", RARE_RELICS),
        ("boss", BOSS_RELICS),
        ("shop", SHOP_RELICS),
        ("special", SPECIAL_RELICS),
    ]:
        total = len(relic_ids)
        full = 0
        partial = 0
        data_only = 0

        status_list = []

        for rid in relic_ids:
            relic = ALL_RELICS[rid]
            status = analyze_relic_status(rid, relic, combat_impls, game_impls)

            if status == "full":
                full += 1
            elif status == "partial":
                partial += 1
            elif status == "data_only":
                data_only += 1

            status_list.append({
                "id": rid,
                "name": relic.name,
                "status": status,
                "has_effects": len(relic.effects) > 0,
                "in_combat": rid in combat_impls,
                "in_game": rid in game_impls,
            })

        tier_stats[tier_name] = {
            "total": total,
            "full": full,
            "partial": partial,
            "data_only": data_only,
            "relics": status_list,
        }

    # Find missing relics (in Java but not in Python)
    python_relic_ids = set(ALL_RELICS.keys())
    java_relic_names = set(java_relics.keys())

    # Try to match by converting Python IDs to Java class names
    python_as_java = set()
    for rid in python_relic_ids:
        # Convert "Burning Blood" -> "BurningBlood"
        java_name = rid.replace(" ", "").replace("'", "")
        python_as_java.add(java_name)

    missing_in_python = java_relic_names - python_as_java

    # Build summary
    summary = {
        "total_java": len(java_relics),
        "total_python": len(ALL_RELICS),
        "total_implemented": len(all_impls),
        "missing_in_python_count": len(missing_in_python),
        "missing_in_python": sorted(list(missing_in_python)),
    }

    result = {
        "summary": summary,
        "by_tier": tier_stats,
    }

    return result

def main():
    """Generate relic inventory report."""
    print("=== Relic Implementation Inventory ===\n")

    result = categorize_relics()

    # Write JSON
    output_file = Path(__file__).parent / "inventory_relics.json"
    with open(output_file, 'w') as f:
        json.dump(result, f, indent=2)

    print(f"\nWrote {output_file}")

    # Print summary
    print("\n=== Summary ===")
    for k, v in result["summary"].items():
        if isinstance(v, list):
            continue
        print(f"  {k}: {v}")

    print("\n=== By Tier ===")
    for tier_name, stats in result["by_tier"].items():
        print(f"\n{tier_name.upper()}: {stats['total']} total")
        print(f"  Full: {stats['full']}")
        print(f"  Partial: {stats['partial']}")
        print(f"  Data only: {stats['data_only']}")

        # Show data_only relics
        data_only = [r for r in stats['relics'] if r['status'] == 'data_only']
        if data_only:
            print(f"  Data-only relics: {', '.join([r['name'] for r in data_only])}")

    print("\n=== Missing in Python ===")
    if result["summary"]["missing_in_python"]:
        for missing in result["summary"]["missing_in_python"][:20]:
            print(f"  - {missing}")
        if len(result["summary"]["missing_in_python"]) > 20:
            print(f"  ... and {len(result['summary']['missing_in_python']) - 20} more")

if __name__ == "__main__":
    main()
