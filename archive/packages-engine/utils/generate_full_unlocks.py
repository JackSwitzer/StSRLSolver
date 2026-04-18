"""
Generate a complete STSUnlocks file with all content unlocked.

This ensures RNG predictions match the game by having identical relic pools.

Usage:
    python generate_full_unlocks.py [--backup] [--apply]

    --backup: Create backup of existing STSUnlocks before modification
    --apply: Actually write the file (otherwise just prints to stdout)
"""

import json
import shutil
import os
from pathlib import Path
from datetime import datetime

# All shared relics that can appear in pools
SHARED_RELICS = [
    "Abacus", "Akabeko", "Anchor", "Ancient Tea Set", "Art of War", "Astrolabe",
    "Bag of Marbles", "Bag of Preparation", "Bird Faced Urn", "Black Star",
    "Blood Vial", "Bloody Idol", "Blue Candle", "The Boot", "Bottled Flame",
    "Bottled Lightning", "Bottled Tornado", "Bronze Scales", "Busted Crown",
    "Calipers", "Calling Bell", "CaptainsWheel", "Cauldron", "Centennial Puzzle",
    "Ceramic Fish", "Chemical X", "Clockwork Souvenir", "Coffee Dripper", "Courier",
    "Cursed Key", "Darkstone Periapt", "Dead Branch", "Dollys Mirror", "Dream Catcher",
    "Du-Vu Doll", "Ectoplasm", "Empty Cage", "Eternal Feather", "FaceOfCleric",
    "Fossilized Helix", "Frozen Egg 2", "Frozen Eye", "Fusion Hammer", "Gambling Chip",
    "Ginger", "Girya", "Golden Idol", "Gremlin Horn", "HandDrill", "Happy Flower",
    "HornCleat", "Ice Cream", "Incense Burner", "Ink Bottle", "Juzu Bracelet",
    "Kunai", "Lantern", "Letter Opener", "Lizard Tail", "Mango", "Matryoshka",
    "Maw Bank", "Meal Ticket", "Meat on the Bone", "Medical Kit", "Membership Card",
    "Mercury Hourglass", "Molten Egg 2", "Mummified Hand", "Necronomicon",
    "Nilrys Codex", "Nunchaku", "Oddly Smooth Stone", "Odd Mushroom", "Old Coin",
    "Omamori", "Orange Pellets", "Orichalcum", "Ornamental Fan", "Orrery",
    "Pandora's Box", "Pantograph", "Peace Pipe", "Pear", "Pen Nib",
    "Philosopher's Stone", "Pocketwatch", "Potion Belt", "Prayer Wheel",
    "PreservedInsect", "Question Card", "Regal Pillow", "Runic Dome", "Runic Pyramid",
    "SacredBark", "Shovel", "Shuriken", "Singing Bowl", "SlaversCollar", "Sling",
    "Smiling Mask", "Snecko Eye", "Sozu", "StoneCalendar", "Strange Spoon",
    "Strawberry", "Sundial", "Thread and Needle", "Tiny Chest", "Tiny House",
    "Toolbox", "Torii", "Toxic Egg 2", "Toy Ornithopter", "Tungsten Rod", "Turnip",
    "Unceasing Top", "Vajra", "Velvet Choker", "Waffle", "War Paint", "WarpedTongs",
    "Whetstone", "White Beast Statue", "Wing Boots",
]

# Character-specific relics
IRONCLAD_RELICS = [
    "Burning Blood", "Black Blood", "Brimstone", "ChampionsBelt", "CharonsAshes",
    "Magic Flower", "Paper Crane", "Paper Frog", "RedSkull", "SelfFormingClay",
    "SnakeSkull",
]

SILENT_RELICS = [
    "Ring of the Snake", "Ring of the Serpent", "HoveringKite", "Ninja Scroll",
    "Paper Crane", "Paper Frog", "Specimen", "The Specimen", "TingShaw", "ToughBandages",
    "WristBlade",
]

DEFECT_RELICS = [
    "Cracked Core", "FrozenCore", "Cables", "DataDisk", "Emotion Chip",
    "GoldPlatedCables", "Inserter", "Nuclear Battery", "Runic Capacitor",
    "Symbiotic Virus",
]

WATCHER_RELICS = [
    "PureWater", "HolyWater", "CloakClasp", "Damaru", "GoldenEye", "Melange",
    "TeardropLocket", "VioletLotus", "Yang",
]

# Cards that need unlocking (just the unlock-gated ones)
UNLOCK_CARDS = [
    # Ironclad
    "Wild Strike", "Evolve", "Immolate", "Heavy Blade", "Spot Weakness", "Limit Break",
    # Silent
    "Concentrate", "Setup", "Grand Finale", "Cloak And Dagger", "Accuracy", "Storm of Steel",
    "Bane", "Catalyst", "Corpse Explosion",
    # Defect
    "Hyperbeam", "Recycle", "Core Surge", "Turbo", "Sunder", "Meteor Strike",
    "Rebound", "Undo", "Echo Form",
    # Watcher
    "SpiritShield", "Wish", "Wireheading", "ForeignInfluence", "Alpha",
    "MentalFortress", "Prostrate", "Blasphemy", "Devotion",
]

# Characters
CHARACTERS = ["The Silent", "Defect", "Watcher"]

# Bosses
BOSSES = ["SLIME", "CHAMP", "CROW", "DONUT", "GUARDIAN", "AUTOMATON", "GHOST", "COLLECTOR", "WIZARD"]


def generate_full_unlocks() -> dict:
    """Generate a complete unlock dictionary."""
    unlocks = {}

    # Add characters
    for char in CHARACTERS:
        unlocks[char] = "2"

    # Add all shared relics
    for relic in SHARED_RELICS:
        unlocks[relic] = "2"

    # Add character-specific relics
    for relic in IRONCLAD_RELICS + SILENT_RELICS + DEFECT_RELICS + WATCHER_RELICS:
        unlocks[relic] = "2"

    # Add unlock cards
    for card in UNLOCK_CARDS:
        unlocks[card] = "2"

    # Add bosses
    for boss in BOSSES:
        unlocks[boss] = "2"

    return unlocks


def get_sts_prefs_path() -> Path:
    """Get the STS preferences directory path."""
    # macOS path
    mac_path = Path.home() / "Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/preferences"
    if mac_path.exists():
        return mac_path

    # Windows path (common locations)
    win_paths = [
        Path("C:/Program Files (x86)/Steam/steamapps/common/SlayTheSpire/preferences"),
        Path("C:/Program Files/Steam/steamapps/common/SlayTheSpire/preferences"),
    ]
    for p in win_paths:
        if p.exists():
            return p

    # Linux path
    linux_path = Path.home() / ".steam/steam/steamapps/common/SlayTheSpire/preferences"
    if linux_path.exists():
        return linux_path

    raise FileNotFoundError("Could not find STS preferences directory")


def main():
    import sys

    do_backup = "--backup" in sys.argv
    do_apply = "--apply" in sys.argv

    unlocks = generate_full_unlocks()

    print(f"Generated unlock data for {len(unlocks)} items:")
    print(f"  - {len(SHARED_RELICS)} shared relics")
    print(f"  - {len(IRONCLAD_RELICS + SILENT_RELICS + DEFECT_RELICS + WATCHER_RELICS)} class relics")
    print(f"  - {len(UNLOCK_CARDS)} cards")
    print(f"  - {len(CHARACTERS)} characters")
    print(f"  - {len(BOSSES)} bosses")

    if not do_apply:
        print("\nJSON output:")
        print(json.dumps(unlocks, indent=2))
        print("\nUse --apply to write to STSUnlocks file")
        return

    try:
        prefs_path = get_sts_prefs_path()
        unlocks_file = prefs_path / "STSUnlocks"

        # Backup if requested
        if do_backup and unlocks_file.exists():
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            backup_path = prefs_path / f"STSUnlocks.backup_{timestamp}"
            shutil.copy(unlocks_file, backup_path)
            print(f"\nBacked up existing file to: {backup_path}")

        # Write new unlocks
        with open(unlocks_file, 'w') as f:
            json.dump(unlocks, f, indent=2)

        # Also write backup file (game expects .backUp)
        backup_file = prefs_path / "STSUnlocks.backUp"
        with open(backup_file, 'w') as f:
            json.dump(unlocks, f, indent=2)

        print(f"\nWrote unlock data to: {unlocks_file}")
        print("Restart Slay the Spire for changes to take effect.")

    except FileNotFoundError as e:
        print(f"\nError: {e}")
        print("Could not locate STS preferences. Manual installation required.")
        print("\nJSON to copy:")
        print(json.dumps(unlocks, indent=2))


if __name__ == "__main__":
    main()
