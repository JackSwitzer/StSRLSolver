#!/usr/bin/env python3
"""Quick card count script."""

import re

# Card dictionary line ranges from cards.py
card_ranges = {
    "DEFECT_CARDS": (2163, 2248),
    "WATCHER_CARDS": (2252, 2346),
    "IRONCLAD_CARDS": (2350, 2435),
    "COLORLESS_CARDS": (2439, 2482),
    "CURSE_CARDS": (2486, 2501),
    "STATUS_CARDS": (2505, 2511),
    "SILENT_CARDS": (3044, 3131),
}

cards_file = "/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/content/cards.py"

with open(cards_file, 'r') as f:
    lines = f.readlines()

results = {}
for name, (start, end) in card_ranges.items():
    # Count lines with pattern "    "CardName": CARD_VAR," (card entries)
    # Start and end are 1-indexed, convert to 0-indexed
    section = lines[start-1:end]

    # Count lines that match card entry pattern
    card_pattern = re.compile(r'^\s{4}"[^"]+": ')
    card_count = sum(1 for line in section if card_pattern.match(line))

    results[name] = card_count
    print(f"{name}: {card_count} cards")

print("\nTotal:", sum(results.values()))
