#!/usr/bin/env python3
"""
Fast processor for November 2020 Slay the Spire data.
Extracts high-quality Watcher wins (A15+, one per seed, victories only).
"""

import json
import gzip
import sys
from pathlib import Path
from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor, as_completed
import multiprocessing

# Config
INPUT_DIR = Path("/Users/jackswitzer/Downloads/Monthly_2020_11")
OUTPUT_DIR = Path(__file__).parent / "data" / "watcher_training"
MIN_ASCENSION = 15  # A15+ for quality
MAX_ONE_PER_SEED = True

def process_file(filepath: Path) -> list:
    """Process a single gzipped JSON file."""
    results = []
    try:
        with gzip.open(filepath, 'rt', encoding='utf-8') as f:
            raw_data = json.load(f)

        for item in raw_data:
            # Data is nested under 'event' key
            run = item.get("event", item)
            # WATCHER WINS ONLY
            if run.get("character_chosen", "").upper() != "WATCHER":
                continue
            if not run.get("victory", False):
                continue

            # Quality filters
            asc = run.get("ascension_level", 0)
            if asc < MIN_ASCENSION:
                continue

            # Skip daily/endless/seeded
            if run.get("is_daily") or run.get("is_endless") or run.get("chose_seed"):
                continue

            # Must have card choices
            if not run.get("card_choices"):
                continue

            results.append({
                "seed": run.get("seed_played"),
                "ascension_level": asc,
                "floor_reached": run.get("floor_reached", 0),
                "master_deck": run.get("master_deck", []),
                "card_choices": run.get("card_choices", []),
                "relics": run.get("relics", []),
                "relics_obtained": run.get("relics_obtained", []),
                "boss_relics": run.get("boss_relics", []),
                "path_per_floor": run.get("path_per_floor", []),
                "path_taken": run.get("path_taken", []),
                "event_choices": run.get("event_choices", []),
                "campfire_choices": run.get("campfire_choices", []),
                "items_purchased": run.get("items_purchased", []),
                "items_purged": run.get("items_purged", []),
                "damage_taken": run.get("damage_taken", []),
                "current_hp_per_floor": run.get("current_hp_per_floor", []),
                "max_hp_per_floor": run.get("max_hp_per_floor", []),
                "gold": run.get("gold", 0),
                "neow_bonus": run.get("neow_bonus"),
                "neow_cost": run.get("neow_cost"),
                "potions_obtained": run.get("potions_obtained", []),
                "victory": True,
            })
    except Exception as e:
        print(f"Error processing {filepath.name}: {e}", file=sys.stderr)

    return results

def main():
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    # Find all files
    files = sorted(INPUT_DIR.glob("*.json.gz"))
    print(f"Found {len(files)} files to process")

    # Process in parallel
    all_runs = []
    num_workers = max(1, multiprocessing.cpu_count() - 1)
    print(f"Using {num_workers} workers...")

    with ProcessPoolExecutor(max_workers=num_workers) as executor:
        futures = {executor.submit(process_file, f): f for f in files}

        done = 0
        for future in as_completed(futures):
            done += 1
            if done % 100 == 0:
                print(f"  Processed {done}/{len(files)} files...")

            results = future.result()
            all_runs.extend(results)

    print(f"\nTotal Watcher A{MIN_ASCENSION}+ wins: {len(all_runs)}")

    # Dedupe by seed (keep highest ascension)
    if MAX_ONE_PER_SEED:
        by_seed = defaultdict(list)
        for run in all_runs:
            seed = run.get("seed")
            if seed:
                by_seed[seed].append(run)

        deduped = []
        for seed, runs in by_seed.items():
            # Keep the one with highest ascension, then most floors
            best = max(runs, key=lambda r: (r["ascension_level"], r["floor_reached"]))
            deduped.append(best)

        print(f"After deduping (one per seed): {len(deduped)}")
        all_runs = deduped

    # Split by ascension
    by_asc = defaultdict(list)
    for run in all_runs:
        by_asc[run["ascension_level"]].append(run)

    print("\nBy ascension level:")
    for asc in sorted(by_asc.keys()):
        print(f"  A{asc}: {len(by_asc[asc])} wins")

    # Save all
    all_file = OUTPUT_DIR / "watcher_wins_nov2020.json"
    with open(all_file, 'w') as f:
        json.dump(all_runs, f)
    print(f"\nSaved all to: {all_file}")

    # Save A20 only
    a20_runs = by_asc.get(20, [])
    if a20_runs:
        a20_file = OUTPUT_DIR / "watcher_a20_wins.json"
        with open(a20_file, 'w') as f:
            json.dump(a20_runs, f)
        print(f"Saved A20 to: {a20_file} ({len(a20_runs)} runs)")

    # Stats
    total_card_choices = sum(len(r.get("card_choices", [])) for r in all_runs)
    print(f"\nTotal card choice examples: {total_card_choices}")

if __name__ == "__main__":
    main()
