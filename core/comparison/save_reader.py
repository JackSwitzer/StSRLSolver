#!/usr/bin/env python3
"""
Slay the Spire Save File Reader

Reads and decodes .autosave files to extract complete game state including RNG counters.
Save files are XOR obfuscated with Base64 encoding.
"""

import json
import base64
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Any


# XOR key used by the game (hardcoded in SaveFileObfuscator.class)
OBFUSCATION_KEY = "key"


def xor_with_key(data: bytes, key: str) -> bytes:
    """XOR data with repeating key."""
    key_bytes = key.encode('utf-8')
    result = bytearray(len(data))
    for i, byte in enumerate(data):
        result[i] = byte ^ key_bytes[i % len(key_bytes)]
    return bytes(result)


def decode_save_file(data: str) -> dict:
    """
    Decode an obfuscated save file.

    Args:
        data: Raw file contents (may be obfuscated or plain JSON)

    Returns:
        Parsed save file as dict
    """
    # Check if obfuscated (won't contain '{' if obfuscated)
    if '{' in data:
        # Plain JSON
        return json.loads(data)

    # Obfuscated: Base64 decode then XOR
    try:
        decoded_b64 = base64.b64decode(data)
        decrypted = xor_with_key(decoded_b64, OBFUSCATION_KEY)
        json_str = decrypted.decode('utf-8')
        return json.loads(json_str)
    except Exception as e:
        raise ValueError(f"Failed to decode save file: {e}")


def encode_save_file(data: dict) -> str:
    """
    Encode a save file with obfuscation.

    Args:
        data: Save file as dict

    Returns:
        Obfuscated string ready to write
    """
    json_str = json.dumps(data, indent=2)
    json_bytes = json_str.encode('utf-8')
    encrypted = xor_with_key(json_bytes, OBFUSCATION_KEY)
    return base64.b64encode(encrypted).decode('utf-8')


@dataclass
class RNGState:
    """RNG counter state from save file."""
    seed: int
    monster_seed_count: int = 0
    event_seed_count: int = 0
    merchant_seed_count: int = 0
    card_seed_count: int = 0
    treasure_seed_count: int = 0
    relic_seed_count: int = 0
    potion_seed_count: int = 0
    ai_seed_count: int = 0
    shuffle_seed_count: int = 0
    card_random_seed_count: int = 0

    def to_dict(self) -> Dict[str, int]:
        return {
            "seed": self.seed,
            "monster": self.monster_seed_count,
            "event": self.event_seed_count,
            "merchant": self.merchant_seed_count,
            "card": self.card_seed_count,
            "treasure": self.treasure_seed_count,
            "relic": self.relic_seed_count,
            "potion": self.potion_seed_count,
            "ai": self.ai_seed_count,
            "shuffle": self.shuffle_seed_count,
            "card_random": self.card_random_seed_count,
        }


@dataclass
class CardSave:
    """Card in deck."""
    id: str
    upgrades: int = 0
    misc: int = 0

    def __repr__(self):
        suffix = "+" * self.upgrades if self.upgrades else ""
        return f"{self.id}{suffix}"


@dataclass
class SaveState:
    """Complete extracted state from save file."""
    # Progression
    floor: int
    act: int
    room_x: int
    room_y: int
    current_room: str

    # Player
    current_hp: int
    max_hp: int
    gold: int

    # Collections
    deck: List[CardSave]
    relics: List[str]
    relic_counters: List[int]
    potions: List[str]

    # RNG
    rng: RNGState

    # Ascension
    ascension: int
    character: str

    # Keys
    has_ruby_key: bool = False
    has_emerald_key: bool = False
    has_sapphire_key: bool = False

    # Blizzard (pity timers)
    card_blizzard: int = 5
    potion_blizzard: int = 0

    # Path history
    path_x: List[int] = field(default_factory=list)
    path_y: List[int] = field(default_factory=list)

    # Raw data for debugging
    raw: Dict[str, Any] = field(default_factory=dict)


def read_save_file(path: str) -> SaveState:
    """
    Read and parse a save file.

    Args:
        path: Path to .autosave file

    Returns:
        SaveState with extracted data
    """
    with open(path, 'r') as f:
        raw_data = f.read()

    data = decode_save_file(raw_data)

    # Extract RNG state
    rng = RNGState(
        seed=data.get("seed", 0),
        monster_seed_count=data.get("monster_seed_count", 0),
        event_seed_count=data.get("event_seed_count", 0),
        merchant_seed_count=data.get("merchant_seed_count", 0),
        card_seed_count=data.get("card_seed_count", 0),
        treasure_seed_count=data.get("treasure_seed_count", 0),
        relic_seed_count=data.get("relic_seed_count", 0),
        potion_seed_count=data.get("potion_seed_count", 0),
        ai_seed_count=data.get("ai_seed_count", 0),
        shuffle_seed_count=data.get("shuffle_seed_count", 0),
        card_random_seed_count=data.get("card_random_seed_count", 0),
    )

    # Extract deck
    deck = []
    for card in data.get("cards", []):
        deck.append(CardSave(
            id=card.get("id", ""),
            upgrades=card.get("upgrades", 0),
            misc=card.get("misc", 0),
        ))

    # Build state
    return SaveState(
        floor=data.get("floor_num", 0),
        act=data.get("act_num", 1),
        room_x=data.get("room_x", -1),
        room_y=data.get("room_y", -1),
        current_room=data.get("current_room", ""),
        current_hp=data.get("current_health", 0),
        max_hp=data.get("max_health", 0),
        gold=data.get("gold", 0),
        deck=deck,
        relics=data.get("relics", []),
        relic_counters=data.get("relic_counters", []),
        potions=data.get("potions", []),
        rng=rng,
        ascension=data.get("ascension_level", 0),
        character=data.get("name", ""),
        has_ruby_key=data.get("has_ruby_key", False),
        has_emerald_key=data.get("has_emerald_key", False),
        has_sapphire_key=data.get("has_sapphire_key", False),
        card_blizzard=data.get("card_blizzard", 5),
        potion_blizzard=data.get("potion_blizzard", 0),
        path_x=data.get("path_x", []),
        path_y=data.get("path_y", []),
        raw=data,
    )


def get_default_save_path(character: str = "WATCHER", slot: int = 3) -> str:
    """
    Get the default save file path for a character.

    Args:
        character: Character name (WATCHER, IRONCLAD, etc.)
        slot: Save slot (0-3)

    Returns:
        Path to save file
    """
    import os

    # macOS path
    base = os.path.expanduser("~/Library/Preferences/SlayTheSpire/saves")

    if slot == 0:
        filename = f"{character}.autosave"
    else:
        filename = f"{slot}_{character}.autosave"

    return os.path.join(base, filename)


# =============================================================================
# CLI
# =============================================================================

if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        # Try default Watcher save
        path = get_default_save_path()
        print(f"No path provided, trying: {path}")
    else:
        path = sys.argv[1]

    try:
        state = read_save_file(path)

        print("=" * 60)
        print("SAVE FILE STATE")
        print("=" * 60)
        print(f"Character: {state.character}")
        print(f"Ascension: {state.ascension}")
        print(f"Act {state.act}, Floor {state.floor}")
        print(f"Position: ({state.room_x}, {state.room_y})")
        print(f"Current room: {state.current_room}")
        print()
        print(f"HP: {state.current_hp}/{state.max_hp}")
        print(f"Gold: {state.gold}")
        print()
        print(f"Deck ({len(state.deck)} cards):")
        for card in state.deck:
            print(f"  - {card}")
        print()
        print(f"Relics ({len(state.relics)}):")
        for i, relic in enumerate(state.relics):
            counter = state.relic_counters[i] if i < len(state.relic_counters) else -1
            counter_str = f" ({counter})" if counter >= 0 else ""
            print(f"  - {relic}{counter_str}")
        print()
        print(f"Potions: {state.potions}")
        print()
        print("RNG Counters:")
        for name, value in state.rng.to_dict().items():
            print(f"  {name}: {value}")
        print()
        print(f"Card Blizzard: {state.card_blizzard}")
        print(f"Potion Blizzard: {state.potion_blizzard}")

    except FileNotFoundError:
        print(f"Save file not found: {path}")
    except Exception as e:
        print(f"Error reading save: {e}")
        import traceback
        traceback.print_exc()
