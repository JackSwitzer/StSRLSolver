"""
Find seeds that produce specific boss relics for edge case testing.
"""
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.generation.relics import predict_neow_boss_swap
from core.state.rng import seed_to_long

# Target relics for edge case testing
TARGETS = [
    "Calling Bell",      # Tests full relic initialization (grants 3 random relics + curse)
    "Astrolabe",         # First in shared BOSS list - tests early HashMap iteration
    "VioletLotus",       # Watcher-specific boss relic
    "HolyWater",         # Watcher-specific, has canSpawn() check (requires PureWater)
    "Tiny House",        # Late in list
    "Velvet Choker",     # Late in list
]

def search_seeds():
    """Search for seeds that produce target boss relics."""
    found = {target: None for target in TARGETS}

    # Try common seed patterns
    test_patterns = [
        # Letters
        [f"{chr(c)}" for c in range(ord('A'), ord('Z')+1)],
        # Simple words
        ["WATCHER", "RELIC", "BOSS", "SWAP", "TEST", "GAME", "PLAY", "SLAY", "SPIRE"],
        # Letter combos
        [f"{chr(a)}{chr(b)}" for a in range(ord('A'), ord('Z')+1) for b in range(ord('A'), ord('D')+1)],
        # Numbers
        [str(i) for i in range(1, 1000)],
        # Common seed names
        [f"TEST{i}" for i in range(1, 100)],
        [f"SEED{i}" for i in range(1, 100)],
    ]

    all_seeds = []
    for pattern in test_patterns:
        all_seeds.extend(pattern)

    print(f"Searching {len(all_seeds)} seed candidates...")
    print(f"Looking for: {TARGETS}")
    print()

    for seed_str in all_seeds:
        try:
            seed = seed_to_long(seed_str)
            result = predict_neow_boss_swap(seed, "WATCHER")

            if result in found and found[result] is None:
                found[result] = seed_str
                print(f"  Found {result}: seed '{seed_str}'")

                # Check if we found all
                if all(v is not None for v in found.values()):
                    break
        except Exception as e:
            continue

    print()
    print("=" * 60)
    print("EDGE CASE TEST SEEDS")
    print("=" * 60)

    for target, seed_str in found.items():
        if seed_str:
            seed = seed_to_long(seed_str)
            print(f"  {target:20} -> seed '{seed_str}' ({seed})")
        else:
            print(f"  {target:20} -> NOT FOUND (need more search)")

    return found


if __name__ == "__main__":
    search_seeds()
