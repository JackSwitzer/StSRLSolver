#!/usr/bin/env python3
"""Read and display current STS save file data."""
import base64
import json
import sys
from pathlib import Path

SAVE_DIR = Path.home() / "Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/saves"
XOR_KEY = b"key"

def decrypt_save(data: bytes) -> dict:
    """Decrypt XOR-encrypted save file."""
    decrypted = bytes([data[i] ^ XOR_KEY[i % len(XOR_KEY)] for i in range(len(data))])
    return json.loads(decrypted.decode('utf-8'))

def load_save(character: str = "WATCHER") -> dict | None:
    """Load save file for character."""
    save_path = SAVE_DIR / f"{character}.autosave"
    if not save_path.exists():
        return None
    with open(save_path, "rb") as f:
        return decrypt_save(base64.b64decode(f.read()))

def print_save(save: dict):
    """Print save data in readable format."""
    print(f"{'='*50}")
    print(f"SEED: {save['seed']}")
    print(f"FLOOR: {save['floor_num']} | ACT: {save['level_name']}")
    print(f"HP: {save['current_health']}/{save.get('max_health', '?')} | GOLD: {save['gold']}")
    print(f"NEOW: {save['neow_bonus']}")
    print(f"BOSS: {save['boss']}")
    print(f"ASCENSION: {save.get('ascension_level', 0)}")
    print(f"{'='*50}")

    print(f"\nRELICS ({len(save['relics'])}):")
    print(f"  {', '.join(save['relics'])}")

    print(f"\nDECK ({len(save.get('cards', []))}):")
    for card in save.get('cards', []):
        upg = '+' if card.get('upgrades', 0) > 0 else ''
        print(f"  {card['id']}{upg}")

    print(f"\nPOTIONS:")
    for p in save.get('potions', []):
        print(f"  {p}")

    print(f"\nPATH: {' -> '.join(save.get('metric_path_per_floor', []))}")

    print(f"\nMONSTER LIST (upcoming):")
    for i, m in enumerate(save.get('monster_list', [])[:8], 1):
        print(f"  {i}. {m}")

    print(f"\nELITE LIST (upcoming):")
    for i, e in enumerate(save.get('elite_list', save.get('elite_monster_list', []))[:4], 1):
        print(f"  {i}. {e}")

    print(f"\nRNG COUNTERS:")
    print(f"  card_seed_count: {save.get('card_seed_count', 0)}")
    print(f"  relic_seed_count: {save.get('relic_seed_count', 0)}")
    print(f"  potion_seed_count: {save.get('potion_seed_count', 0)}")
    print(f"  event_seed_count: {save.get('event_seed_count', 0)}")
    print(f"  monster_seed_count: {save.get('monster_seed_count', 0)}")

def main():
    character = sys.argv[1].upper() if len(sys.argv) > 1 else "WATCHER"
    save = load_save(character)
    if save:
        print_save(save)
    else:
        print(f"No save file found for {character}")
        print(f"Looking in: {SAVE_DIR}")

if __name__ == "__main__":
    main()
