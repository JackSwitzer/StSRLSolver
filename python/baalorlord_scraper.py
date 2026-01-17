"""
Baalorlord.tv Run Data Scraper

Collects structured decision data from Baalorlord's Slay the Spire runs.
Data includes: card picks, event choices, campfire decisions, boss relics, etc.

This is high-quality expert A20 decision data.
"""

import json
import time
import asyncio
from pathlib import Path
from typing import List, Dict, Optional, Any
from dataclasses import dataclass, asdict

try:
    import aiohttp
except ImportError:
    print("Install: pip install aiohttp")
    aiohttp = None

# ============ CONFIG ============

BASE_URL = "https://baalorlord.tv"
PROFILES = {
    0: "BaalorA20",
    1: "BaalorLadder",
    2: "Baalor",
}

# ============ DATA STRUCTURES ============

@dataclass
class CardChoice:
    """A card reward decision."""
    floor: int
    picked: Optional[str]  # None if skipped
    not_picked: List[str]

@dataclass
class CampfireChoice:
    """A rest site decision."""
    floor: int
    key: str  # "REST", "SMITH", "RECALL", etc.
    data: Optional[str] = None  # Card upgraded, etc.

@dataclass
class EventChoice:
    """An event decision."""
    floor: int
    event_name: str
    player_choice: str
    damage_taken: int = 0
    gold_change: int = 0
    cards_obtained: List[str] = None
    relics_obtained: List[str] = None

@dataclass
class BossRelicChoice:
    """Boss relic selection."""
    floor: int
    picked: str
    not_picked: List[str]

@dataclass
class RunData:
    """Complete structured run data."""
    # Meta
    timestamp: int
    character: str
    ascension: int
    victory: bool
    floor_reached: int
    score: int
    seed: str
    playtime: int

    # Final state
    master_deck: List[str]
    relics: List[str]

    # Decisions
    card_choices: List[CardChoice]
    campfire_choices: List[CampfireChoice]
    event_choices: List[EventChoice]
    boss_relic_choices: List[BossRelicChoice]

    # Path data
    path_per_floor: List[Optional[str]]
    current_hp_per_floor: List[int]
    max_hp_per_floor: List[int]
    gold_per_floor: List[int]

    # Combat data
    damage_taken: List[Dict]

    # Raw data
    raw_json: Dict = None

# ============ PARSING ============

def parse_run_json(data: Dict) -> RunData:
    """Parse raw run JSON into structured RunData."""

    # Card choices
    card_choices = []
    for choice in data.get("card_choices", []):
        card_choices.append(CardChoice(
            floor=choice.get("floor", 0),
            picked=choice.get("picked"),
            not_picked=choice.get("not_picked", [])
        ))

    # Campfire choices
    campfire_choices = []
    for choice in data.get("campfire_choices", []):
        campfire_choices.append(CampfireChoice(
            floor=choice.get("floor", 0),
            key=choice.get("key", ""),
            data=choice.get("data")
        ))

    # Event choices
    event_choices = []
    for choice in data.get("event_choices", []):
        event_choices.append(EventChoice(
            floor=choice.get("floor", 0),
            event_name=choice.get("event_name", ""),
            player_choice=choice.get("player_choice", ""),
            damage_taken=choice.get("damage_taken", 0),
            gold_change=choice.get("gold_change", 0),
            cards_obtained=choice.get("cards_obtained", []),
            relics_obtained=choice.get("relics_obtained", [])
        ))

    # Boss relic choices
    boss_relic_choices = []
    for choice in data.get("boss_relics", []):
        if choice.get("picked"):
            boss_relic_choices.append(BossRelicChoice(
                floor=choice.get("floor", 0),
                picked=choice.get("picked"),
                not_picked=choice.get("not_picked", [])
            ))

    return RunData(
        timestamp=data.get("timestamp", 0),
        character=data.get("character_chosen", ""),
        ascension=data.get("ascension_level", 0),
        victory=data.get("victory", False),
        floor_reached=data.get("floor_reached", 0),
        score=data.get("score", 0),
        seed=data.get("seed_played", ""),
        playtime=data.get("playtime", 0),

        master_deck=data.get("master_deck", []),
        relics=data.get("relics", []),

        card_choices=card_choices,
        campfire_choices=campfire_choices,
        event_choices=event_choices,
        boss_relic_choices=boss_relic_choices,

        path_per_floor=data.get("path_per_floor", []),
        current_hp_per_floor=data.get("current_hp_per_floor", []),
        max_hp_per_floor=data.get("max_hp_per_floor", []),
        gold_per_floor=data.get("gold_per_floor", []),

        damage_taken=data.get("damage_taken", []),

        raw_json=data
    )

# ============ SCRAPING ============

async def fetch_runs_page(session: aiohttp.ClientSession, profile: int, page: int) -> List[str]:
    """Fetch run IDs from a runs page."""
    url = f"{BASE_URL}/profile/{profile}/runs?page={page}"

    async with session.get(url) as response:
        if response.status != 200:
            return []

        html = await response.text()

        # Extract run timestamps from links like /runs/1768506379
        import re
        run_ids = re.findall(r'/runs/(\d+)', html)
        return list(set(run_ids))  # Dedupe

async def fetch_run_raw(session: aiohttp.ClientSession, run_id: str) -> Optional[Dict]:
    """Fetch raw JSON for a single run."""
    url = f"{BASE_URL}/runs/{run_id}/raw"

    try:
        async with session.get(url) as response:
            if response.status != 200:
                return None
            return await response.json()
    except Exception as e:
        print(f"Error fetching {run_id}: {e}")
        return None

async def scrape_profile_runs(
    profile: int = 0,
    max_pages: int = 50,
    output_dir: Path = None,
    delay: float = 0.5,
    watcher_only: bool = True,
    victory_only: bool = False,
) -> List[RunData]:
    """
    Scrape all runs from a Baalorlord profile.

    Args:
        profile: Profile ID (0=BaalorA20, 1=BaalorLadder, 2=Baalor)
        max_pages: Maximum pages to scrape
        output_dir: Directory to save run JSONs
        delay: Delay between requests (be nice to the server)
        watcher_only: Only collect Watcher runs
        victory_only: Only collect winning runs

    Returns:
        List of parsed RunData objects
    """
    if aiohttp is None:
        print("aiohttp not installed")
        return []

    if output_dir:
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

    runs = []
    seen_ids = set()

    async with aiohttp.ClientSession() as session:
        # First pass: collect all run IDs
        print(f"Collecting run IDs from profile {profile} ({PROFILES.get(profile, 'unknown')})...")
        all_run_ids = []

        for page in range(1, max_pages + 1):
            run_ids = await fetch_runs_page(session, profile, page)
            if not run_ids:
                print(f"  No more runs at page {page}")
                break

            new_ids = [rid for rid in run_ids if rid not in seen_ids]
            seen_ids.update(new_ids)
            all_run_ids.extend(new_ids)

            print(f"  Page {page}: found {len(new_ids)} new runs (total: {len(all_run_ids)})")
            await asyncio.sleep(delay)

        # Second pass: fetch each run's raw JSON
        print(f"\nFetching {len(all_run_ids)} runs...")

        for i, run_id in enumerate(all_run_ids):
            data = await fetch_run_raw(session, run_id)
            if data is None:
                continue

            # Filter
            character = data.get("character_chosen", "")
            if watcher_only and character != "WATCHER":
                continue
            if victory_only and not data.get("victory"):
                continue

            # Parse
            run = parse_run_json(data)
            runs.append(run)

            # Save raw JSON
            if output_dir:
                filename = f"{run_id}_{character.lower()}_{'win' if run.victory else 'loss'}.json"
                with open(output_dir / filename, 'w') as f:
                    json.dump(data, f, indent=2)

            if (i + 1) % 10 == 0:
                print(f"  Processed {i + 1}/{len(all_run_ids)} runs, {len(runs)} kept")

            await asyncio.sleep(delay)

    print(f"\nCollected {len(runs)} runs")
    return runs

# ============ ANALYSIS ============

def analyze_card_choices(runs: List[RunData], character: str = "WATCHER") -> Dict[str, Dict]:
    """Analyze card pick rates from runs."""
    card_stats = {}

    for run in runs:
        if run.character != character:
            continue

        for choice in run.card_choices:
            # Track picked card
            if choice.picked:
                if choice.picked not in card_stats:
                    card_stats[choice.picked] = {"picked": 0, "offered": 0, "skipped_for": []}
                card_stats[choice.picked]["picked"] += 1
                card_stats[choice.picked]["offered"] += 1

            # Track not picked cards
            for card in choice.not_picked:
                if card not in card_stats:
                    card_stats[card] = {"picked": 0, "offered": 0, "skipped_for": []}
                card_stats[card]["offered"] += 1
                if choice.picked:
                    card_stats[card]["skipped_for"].append(choice.picked)

    # Calculate pick rates
    for card, stats in card_stats.items():
        stats["pick_rate"] = stats["picked"] / stats["offered"] if stats["offered"] > 0 else 0

    # Sort by pick rate
    sorted_cards = sorted(card_stats.items(), key=lambda x: x[1]["pick_rate"], reverse=True)

    return dict(sorted_cards)

def analyze_event_choices(runs: List[RunData]) -> Dict[str, Dict]:
    """Analyze event choice patterns."""
    event_stats = {}

    for run in runs:
        for choice in run.event_choices:
            event = choice.event_name
            if event not in event_stats:
                event_stats[event] = {"choices": {}, "total": 0}

            event_stats[event]["total"] += 1
            player_choice = choice.player_choice
            if player_choice not in event_stats[event]["choices"]:
                event_stats[event]["choices"][player_choice] = 0
            event_stats[event]["choices"][player_choice] += 1

    return event_stats

def export_training_data(runs: List[RunData], output_path: Path):
    """Export runs as training data for ML."""
    training_data = []

    for run in runs:
        # Each decision becomes a training example
        for choice in run.card_choices:
            if not choice.picked:  # Skip if no pick
                continue

            example = {
                "type": "card_reward",
                "floor": choice.floor,
                "character": run.character,
                "ascension": run.ascension,
                "options": [choice.picked] + choice.not_picked,
                "chosen": choice.picked,
                "chosen_idx": 0,
                "outcome": "win" if run.victory else "loss",
            }
            training_data.append(example)

        for choice in run.campfire_choices:
            example = {
                "type": "campfire",
                "floor": choice.floor,
                "character": run.character,
                "ascension": run.ascension,
                "action": choice.key,
                "data": choice.data,
                "outcome": "win" if run.victory else "loss",
            }
            training_data.append(example)

        for choice in run.boss_relic_choices:
            example = {
                "type": "boss_relic",
                "floor": choice.floor,
                "character": run.character,
                "ascension": run.ascension,
                "options": [choice.picked] + choice.not_picked,
                "chosen": choice.picked,
                "chosen_idx": 0,
                "outcome": "win" if run.victory else "loss",
            }
            training_data.append(example)

    with open(output_path, 'w') as f:
        json.dump(training_data, f, indent=2)

    print(f"Exported {len(training_data)} training examples to {output_path}")

# ============ CLI ============

async def main():
    import argparse

    parser = argparse.ArgumentParser(description="Scrape Baalorlord's run data")
    parser.add_argument("--profile", type=int, default=0, help="Profile ID (0=A20, 1=Ladder, 2=Baalor)")
    parser.add_argument("--pages", type=int, default=50, help="Max pages to scrape")
    parser.add_argument("--output", default="./baalorlord_data", help="Output directory")
    parser.add_argument("--watcher-only", action="store_true", help="Only Watcher runs")
    parser.add_argument("--victory-only", action="store_true", help="Only winning runs")
    parser.add_argument("--delay", type=float, default=0.5, help="Delay between requests")

    args = parser.parse_args()

    output_dir = Path(args.output)

    runs = await scrape_profile_runs(
        profile=args.profile,
        max_pages=args.pages,
        output_dir=output_dir / "raw",
        delay=args.delay,
        watcher_only=args.watcher_only,
        victory_only=args.victory_only,
    )

    if runs:
        # Export training data
        export_training_data(runs, output_dir / "training_data.json")

        # Analyze card choices
        card_stats = analyze_card_choices(runs)
        with open(output_dir / "card_stats.json", 'w') as f:
            json.dump(card_stats, f, indent=2)

        # Print top picks
        print("\nTop 10 card pick rates:")
        for card, stats in list(card_stats.items())[:10]:
            print(f"  {card}: {stats['pick_rate']:.1%} ({stats['picked']}/{stats['offered']})")

if __name__ == "__main__":
    asyncio.run(main())
