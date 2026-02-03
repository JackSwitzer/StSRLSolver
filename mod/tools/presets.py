#!/usr/bin/env python3
"""
Save file presets for common patching scenarios.

Usage:
    python presets.py act4          # Patch current save to start of Act 4
    python presets.py act4_heart    # Patch to Heart fight directly
    python presets.py heal          # Full heal
    python presets.py gold <N>      # Set gold
    python presets.py info          # Print save summary
"""

import json
import sys
from save_editor import read_save, write_save, do_backup


ACT4_MONSTERS = ["Shield and Spear"] * 3
ACT4_ELITES = ["Shield and Spear"] * 3
ACT4_BOSSES = ["The Heart"] * 3


def preset_act4(character: str = "WATCHER"):
    """Patch save to start of Act 4 with correct encounters."""
    save = read_save(character)
    do_backup(character)

    save["act_num"] = 4
    save["level_name"] = "TheEnding"
    save["floor_num"] = 53
    save["boss"] = "The Heart"
    save["boss_list"] = ACT4_BOSSES
    save["monster_list"] = ACT4_MONSTERS
    save["elite_monster_list"] = ACT4_ELITES
    save["room_x"] = 3
    save["room_y"] = 0
    save["current_room"] = "com.megacrit.cardcrawl.rooms.RestRoom"
    save["post_combat"] = False
    save["path_x"] = [3]
    save["path_y"] = [0]

    write_save(save, character)
    print("Patched to Act 4 start (RestRoom -> Shop -> Shield&Spear -> Heart)")


def preset_act4_heart(character: str = "WATCHER"):
    """Patch save to just before the Heart fight."""
    save = read_save(character)
    do_backup(character)

    save["act_num"] = 4
    save["level_name"] = "TheEnding"
    save["floor_num"] = 56
    save["boss"] = "The Heart"
    save["boss_list"] = ACT4_BOSSES
    save["monster_list"] = ACT4_MONSTERS
    save["elite_monster_list"] = ACT4_ELITES
    # Node y=3 is the boss room entrance in Act 4 map
    save["room_x"] = 3
    save["room_y"] = 3
    save["current_room"] = "com.megacrit.cardcrawl.rooms.MonsterRoomBoss"
    save["post_combat"] = False
    save["path_x"] = [3, 3, 3, 3]
    save["path_y"] = [0, 1, 2, 3]

    write_save(save, character)
    print("Patched to Heart fight")


def preset_heal(character: str = "WATCHER"):
    """Full heal."""
    save = read_save(character)
    do_backup(character)
    save["current_health"] = save["max_health"]
    write_save(save, character)
    print(f"Healed to {save['max_health']} HP")


def preset_gold(amount: int, character: str = "WATCHER"):
    """Set gold."""
    save = read_save(character)
    do_backup(character)
    save["gold"] = amount
    write_save(save, character)
    print(f"Gold set to {amount}")


def preset_info(character: str = "WATCHER"):
    """Print save summary."""
    save = read_save(character)
    print(f"Character:  {character}")
    print(f"Seed:       {save.get('seed')}")
    print(f"Floor:      {save.get('floor_num')}")
    print(f"Act:        {save.get('act_num')}")
    print(f"Level:      {save.get('level_name')}")
    print(f"Room:       {save.get('current_room')}")
    print(f"HP:         {save.get('current_health')}/{save.get('max_health')}")
    print(f"Gold:       {save.get('gold')}")
    print(f"Boss:       {save.get('boss')}")
    print(f"Boss list:  {save.get('boss_list')}")
    print(f"Monsters:   {save.get('monster_list')}")
    print(f"Elites:     {save.get('elite_monster_list', [])[:3]}...")
    print(f"Room pos:   ({save.get('room_x')}, {save.get('room_y')})")
    print(f"Path X:     {save.get('path_x')}")
    print(f"Path Y:     {save.get('path_y')}")
    print(f"Keys:       E={save.get('has_emerald_key')} R={save.get('has_ruby_key')} S={save.get('has_sapphire_key')}")
    print(f"Final act:  {save.get('is_final_act_on')}")

    # Deck summary
    cards = save.get("cards", [])
    print(f"Deck:       {len(cards)} cards")
    for c in cards:
        up = "+" if c.get("upgrades", 0) > 0 else ""
        print(f"  - {c['id']}{up}")

    # Relics
    relics = save.get("relics", [])
    print(f"Relics:     {len(relics)}")
    for r in relics:
        print(f"  - {r}")

    # Potions
    potions = save.get("potions", [])
    print(f"Potions:    {potions}")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    cmd = sys.argv[1]
    character = "WATCHER"

    if cmd == "act4":
        preset_act4(character)
    elif cmd == "act4_heart":
        preset_act4_heart(character)
    elif cmd == "heal":
        preset_heal(character)
    elif cmd == "gold":
        if len(sys.argv) < 3:
            print("Usage: presets.py gold <amount>")
            sys.exit(1)
        preset_gold(int(sys.argv[2]), character)
    elif cmd == "info":
        preset_info(character)
    else:
        print(f"Unknown preset: {cmd}")
        print(__doc__)
        sys.exit(1)


if __name__ == "__main__":
    main()
