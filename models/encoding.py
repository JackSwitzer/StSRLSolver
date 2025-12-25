"""
State and action encoding for Slay the Spire Watcher.

Converts game states to tensors and decisions to action indices for BC training.
"""

import json
from pathlib import Path
from typing import List, Dict, Any, Tuple, Optional
import numpy as np

# Load card/relic catalogs
CATALOG_DIR = Path(__file__).parent.parent / "data" / "catalogs"

# === WATCHER CARDS ===
# Full list of Watcher cards (from wiki)
WATCHER_CARDS = [
    # Starter
    "Strike_P", "Defend_P", "Eruption", "Vigilance",
    # Attacks
    "BowlingBash", "CarveReality", "Conclude", "Consecrate", "CrushJoints",
    "CutThroughFate", "EmptyFist", "FearNoEvil", "FlyingSleeves", "FollowUp",
    "Halt", "JustLucky", "LessonLearned", "Ragnarok", "ReachHeaven", "SashWhip",
    "SignatureMove", "Smite", "TalkToTheHand", "Tantrum", "Wallop", "Weave", "WheelKick", "WindmillStrike",
    # Skills
    "Alpha", "BattleHymn", "Blasphemy", "Brilliance", "Collect", "ConjureBlade",
    "Crescendo", "Deceive", "DeusExMachina", "DevaForm", "Devotion",
    "EmptyBody", "EmptyMind", "Establishment", "Evaluate", "Fasting",
    "ForeignInfluence", "InnerPeace", "Judgment", "LikeWater", "Meditate",
    "MentalFortress", "Nirvana", "Omniscience", "PathToVictory", "Peace",
    "Perseverance", "Pray", "Pressure", "Prostrate", "Protect",
    "Rushdown", "Sanctity", "Scrawl", "SpiritShield", "Study", "Swivel",
    "Tranquility", "Vault", "Vengeance", "WaveOfTheHand", "Wish", "Worship", "WreathOfFlame",
    # Powers
    "Foresight", "MasterReality", "Wish",
    # Special
    "AscendersBane", "Miracle", "Insight", "Smite", "ThroughViolence", "Safety",
]

# Common cards all characters can get
COLORLESS_CARDS = [
    "Apotheosis", "Bandage", "BiteCard", "Blind", "Chrysalis", "DarkShackles",
    "DeepBreath", "Discovery", "DramaticEntrance", "Enlightenment", "Finesse",
    "FlashOfSteel", "Forethought", "GoodInstincts", "HandOfGreed", "Impatience",
    "JackOfAllTrades", "Madness", "Magnetism", "MasterOfStrategy", "Metamorphosis",
    "MindBlast", "Panacea", "PanicButton", "Purity", "RitualDagger", "SecretTechnique",
    "SecretWeapon", "Shiv", "SwiftStrike", "TheBomb", "ThinkingAhead", "Transmutation",
    "Trip", "Violence", "Bite", "JAX",
]

# Curses and statuses
CURSE_CARDS = [
    "AscendersBane", "Clumsy", "Curse of the Bell", "Decay", "Doubt",
    "Injury", "Necronomicurse", "Normality", "Pain", "Parasite",
    "Pride", "Regret", "Shame", "Writhe",
]

STATUS_CARDS = [
    "Burn", "Dazed", "Slimed", "Void", "Wound",
]

ALL_CARDS = WATCHER_CARDS + COLORLESS_CARDS + CURSE_CARDS + STATUS_CARDS

# Create card to index mapping
CARD_TO_IDX = {card: i for i, card in enumerate(ALL_CARDS)}
NUM_CARDS = len(ALL_CARDS)

# === RELICS ===
WATCHER_RELICS = [
    "PureWater", "CloakClasp", "GoldenEye", "Damaru", "Duality", "TeardropLocket",
    "VioletLotus",
]

COMMON_RELICS = [
    # Starter
    "BurningBlood", "RingOfTheSerpent", "CrackedCore", "PureWater",
    # Common
    "Anchor", "AncientTeaSet", "ArtOfWar", "Bag of Marbles", "Bag of Preparation",
    "BloodVial", "BronzeScales", "CentennialPuzzle", "CeramicFish", "Dreamcatcher",
    "HappyFlower", "Juzu Bracelet", "Lantern", "MawBank", "MealTicket", "Nunchaku",
    "Oddly Smooth Stone", "Omamori", "Orichalcum", "PenNib", "Potion Belt",
    "PreservedInsect", "Regal Pillow", "SmoothStone", "Snecko Skull", "StrikeDummy",
    "Sundial", "Symbiotic Virus", "Teardrop Locket", "The Boot", "Tiny Chest",
    "Toy Ornithopter", "Vajra", "War Paint", "Whetstone",
    # Uncommon
    "Blue Candle", "Bottled Flame", "Bottled Lightning", "Bottled Tornado",
    "DarkstonePeriapt", "Eternal Feather", "Frozen Egg 2", "Gremlin Horn",
    "HornCleat", "InkBottle", "Kunai", "Letter Opener", "Matryoshka",
    "Meat on the Bone", "Mercury Hourglass", "Molten Egg 2", "Mummified Hand",
    "Ninja Scroll", "Ornamental Fan", "Pantograph", "Paper Krane", "Paper Phrog",
    "Pear", "Question Card", "Self-Forming Clay", "Shuriken", "Singing Bowl",
    "StrangeSpoon", "The Courier", "Toxic Egg 2", "White Beast Statue",
    # Rare
    "Bird-Faced Urn", "Calipers", "CaptainsWheel", "Champion Belt", "Charon's Ashes",
    "ClockworkSouvenir", "Dead Branch", "Du-Vu Doll", "Emotion Chip", "FossilizedHelix",
    "Gambling Chip", "Ginger", "Girya", "GoldenEye", "Ice Cream", "Incense Burner",
    "Lizard Tail", "Magic Flower", "Mango", "Old Coin", "Peace Pipe", "Pocketwatch",
    "Prayer Wheel", "Shovel", "Stone Calendar", "The Specimen", "Thread and Needle",
    "Tingsha", "Torii", "Tough Bandages", "Turnip", "Unceasing Top", "WingedGreaves",
    # Shop
    "Cauldron", "Chemical X", "ClockworkSouvenir", "Dolly's Mirror", "Frozen Eye",
    "Hand Drill", "Lee's Waffle", "Medical Kit", "Melange", "Membership Card",
    "Orange Pellets", "Orrery", "PrismaticShard", "Runic Pyramid", "Sling of Courage",
    "Strange Spoon", "The Abacus", "Toolbox",
    # Boss
    "Astrolabe", "Black Blood", "Black Star", "Busted Crown", "Calling Bell",
    "Coffee Dripper", "Cursed Key", "Ectoplasm", "Empty Cage", "Fusion Hammer",
    "HolyWater", "HoveringKite", "Mark of Pain", "Nuclear Battery", "Pandora's Box",
    "Philosopher's Stone", "Ring of the Snake", "Runic Dome", "Runic Cube",
    "Sacred Bark", "SlaversCollar", "Snecko Eye", "Sozu", "Tiny House",
    "Velvet Choker", "VioletLotus", "WristBlade",
    # Event
    "Bloody Idol", "Cultist Headpiece", "Enchiridion", "Face Of Cleric", "Golden Idol",
    "Gremlin Visage", "Mark of the Bloom", "Mutagenic Strength", "Nloth's Gift",
    "NlothsHungryFace", "Oddly Smooth Stone", "Red Mask", "Spirit Poop",
    "SsserpentHead", "Warped Tongs", "White Beast Statue",
]

ALL_RELICS = list(set(WATCHER_RELICS + COMMON_RELICS))
RELIC_TO_IDX = {relic: i for i, relic in enumerate(ALL_RELICS)}
NUM_RELICS = len(ALL_RELICS)

# === STANCES ===
STANCES = ["NEUTRAL", "WRATH", "CALM", "DIVINITY"]
STANCE_TO_IDX = {s: i for i, s in enumerate(STANCES)}

# === ACTIONS ===
# For card picks (behavioral cloning on card rewards)
CARD_PICK_ACTIONS = ["SKIP"] + ALL_CARDS
CARD_PICK_TO_IDX = {a: i for i, a in enumerate(CARD_PICK_ACTIONS)}

# For path choices
PATH_CHOICES = ["MONSTER", "ELITE", "REST", "SHOP", "EVENT", "TREASURE", "BOSS"]
PATH_TO_IDX = {p: i for i, p in enumerate(PATH_CHOICES)}

# === ENCODING FUNCTIONS ===

def normalize_card_name(card_name: str) -> str:
    """Normalize card name to match our catalog."""
    # Remove upgrade indicator
    name = card_name.replace("+1", "").replace("+", "").strip()
    # Handle common variations
    name = name.replace(" ", "").replace("_", "")
    return name

def encode_deck(deck: List[str], max_cards: int = 50) -> np.ndarray:
    """Encode a deck as a multi-hot vector with counts."""
    encoding = np.zeros(NUM_CARDS, dtype=np.float32)
    for card in deck[:max_cards]:
        normalized = normalize_card_name(card)
        # Try exact match first
        if normalized in CARD_TO_IDX:
            encoding[CARD_TO_IDX[normalized]] += 1
        else:
            # Try case-insensitive match
            for known_card, idx in CARD_TO_IDX.items():
                if known_card.lower() == normalized.lower():
                    encoding[idx] += 1
                    break
    return encoding

def encode_relics(relics: List[str]) -> np.ndarray:
    """Encode relics as multi-hot vector."""
    encoding = np.zeros(NUM_RELICS, dtype=np.float32)
    for relic in relics:
        normalized = relic.replace(" ", "").replace("_", "")
        for known_relic, idx in RELIC_TO_IDX.items():
            if known_relic.lower() == normalized.lower():
                encoding[idx] = 1
                break
    return encoding

def encode_game_state(run_data: Dict[str, Any], floor: int) -> np.ndarray:
    """
    Encode full game state at a given floor.

    Returns a feature vector containing:
    - Deck composition (multi-hot with counts)
    - Relics (multi-hot)
    - HP ratio
    - Gold (normalized)
    - Ascension level
    - Floor number
    - Act number
    """
    features = []

    # Deck at this floor (need to reconstruct from card_choices up to this floor)
    deck = run_data.get("master_deck", [])
    deck_encoding = encode_deck(deck)
    features.append(deck_encoding)

    # Relics
    relics = run_data.get("relics", [])
    relic_encoding = encode_relics(relics)
    features.append(relic_encoding)

    # HP
    hp_per_floor = run_data.get("current_hp_per_floor", [])
    max_hp_per_floor = run_data.get("max_hp_per_floor", [])
    if floor < len(hp_per_floor) and floor < len(max_hp_per_floor):
        hp_ratio = hp_per_floor[floor] / max(max_hp_per_floor[floor], 1)
    else:
        hp_ratio = 0.5
    features.append(np.array([hp_ratio], dtype=np.float32))

    # Gold (normalized to 0-1 range, cap at 2000)
    gold = min(run_data.get("gold", 100), 2000) / 2000
    features.append(np.array([gold], dtype=np.float32))

    # Ascension (normalized to 0-1)
    ascension = run_data.get("ascension_level", 0) / 20
    features.append(np.array([ascension], dtype=np.float32))

    # Floor/Act info
    floor_norm = floor / 57  # Max floor is ~57 for heart kill
    act = min(floor // 17, 3) / 3  # Approximate act number
    features.append(np.array([floor_norm, act], dtype=np.float32))

    return np.concatenate(features)

def encode_card_choice(choice: Dict[str, Any]) -> Tuple[np.ndarray, int]:
    """
    Encode a card choice decision.

    Returns:
        - state: Feature vector at decision point
        - action: Index of chosen action (skip=0, or card index)
    """
    picked = choice.get("picked", "SKIP")
    if picked == "SKIP" or not picked:
        action = 0
    else:
        normalized = normalize_card_name(picked)
        if normalized in CARD_TO_IDX:
            action = CARD_TO_IDX[normalized] + 1  # +1 because 0 is SKIP
        else:
            action = 0  # Unknown card, treat as skip

    return action

def get_state_dim() -> int:
    """Return the dimension of the state vector."""
    return NUM_CARDS + NUM_RELICS + 5  # deck + relics + hp + gold + asc + floor + act

def get_card_pick_action_dim() -> int:
    """Return number of possible card pick actions."""
    return len(CARD_PICK_ACTIONS)

# === DATA LOADING ===

def load_training_examples(data_path: Path) -> List[Tuple[np.ndarray, int]]:
    """
    Load and process training examples from run data.

    Returns list of (state, action) tuples for card pick decisions.
    """
    examples = []

    with open(data_path) as f:
        runs = json.load(f)

    for run in runs:
        if not run.get("victory"):
            continue  # Only learn from wins

        card_choices = run.get("card_choices", [])
        for choice in card_choices:
            floor = int(choice.get("floor", 0))
            state = encode_game_state(run, floor)
            action = encode_card_choice(choice)
            examples.append((state, action))

    return examples

if __name__ == "__main__":
    print(f"State dimension: {get_state_dim()}")
    print(f"Card pick actions: {get_card_pick_action_dim()}")
    print(f"Number of cards: {NUM_CARDS}")
    print(f"Number of relics: {NUM_RELICS}")

    # Test encoding
    test_deck = ["Strike_P", "Defend_P", "Eruption", "Vigilance", "Tantrum+1"]
    deck_enc = encode_deck(test_deck)
    print(f"\nTest deck encoding (non-zero): {np.sum(deck_enc > 0)} cards")

    test_relics = ["PureWater", "Torii", "Snecko Eye"]
    relic_enc = encode_relics(test_relics)
    print(f"Test relic encoding (non-zero): {np.sum(relic_enc > 0)} relics")
