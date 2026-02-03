#!/usr/bin/env python3
"""
Slay the Spire Save File Editor

StS saves are base64-encoded, XOR-encrypted with key "key".

Usage:
    # Dump save as JSON
    python save_editor.py dump [CHARACTER]

    # Load JSON back into save
    python save_editor.py load <json_file> [CHARACTER]

    # Patch specific fields
    python save_editor.py patch [CHARACTER] key=value key2=value2 ...

    # Backup save
    python save_editor.py backup [CHARACTER]

    # Restore from backup
    python save_editor.py restore [CHARACTER]

CHARACTER defaults to WATCHER.
"""

import json
import base64
import shutil
import sys
import os
from pathlib import Path

SAVE_DIR = Path.home() / "Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/saves"


def save_path(character: str = "WATCHER") -> Path:
    return SAVE_DIR / f"{character}.autosave"


def backup_path(character: str = "WATCHER") -> Path:
    return SAVE_DIR / f"{character}.autosave.edit_bak"


XOR_KEY = b"key"


def decrypt(raw_b64: str) -> dict:
    """Decrypt a StS save file from base64+XOR to dict."""
    decoded = base64.b64decode(raw_b64)
    decrypted = bytes([decoded[i] ^ XOR_KEY[i % len(XOR_KEY)] for i in range(len(decoded))])
    return json.loads(decrypted.decode("utf-8"))


def encrypt(save: dict) -> str:
    """Encrypt a dict back to StS save format (XOR+base64)."""
    raw = json.dumps(save).encode("utf-8")
    encrypted = bytes([raw[i] ^ XOR_KEY[i % len(XOR_KEY)] for i in range(len(raw))])
    return base64.b64encode(encrypted).decode("utf-8")


def read_save(character: str = "WATCHER") -> dict:
    """Read and decrypt a save file."""
    with open(save_path(character), "r") as f:
        return decrypt(f.read())


def write_save(save: dict, character: str = "WATCHER"):
    """Encrypt and write a save file."""
    with open(save_path(character), "w") as f:
        f.write(encrypt(save))


def do_backup(character: str = "WATCHER"):
    """Create a backup of the current save."""
    src = save_path(character)
    dst = backup_path(character)
    if src.exists():
        shutil.copy(src, dst)
        print(f"Backed up to {dst}")
    else:
        print(f"No save found at {src}")


def do_restore(character: str = "WATCHER"):
    """Restore save from backup."""
    src = backup_path(character)
    dst = save_path(character)
    if src.exists():
        shutil.copy(src, dst)
        print(f"Restored from {src}")
    else:
        print(f"No backup found at {src}")


def do_dump(character: str = "WATCHER"):
    """Dump save as formatted JSON to stdout."""
    save = read_save(character)
    # Print summary first
    print(f"# Character: {character}", file=sys.stderr)
    print(f"# Floor: {save.get('floor_num')}", file=sys.stderr)
    print(f"# Act: {save.get('act_num')}", file=sys.stderr)
    print(f"# Room: {save.get('current_room')}", file=sys.stderr)
    print(f"# HP: {save.get('current_health')}/{save.get('max_health')}", file=sys.stderr)
    print(f"# Gold: {save.get('gold')}", file=sys.stderr)
    print(f"# Seed: {save.get('seed')}", file=sys.stderr)
    print(f"# Boss: {save.get('boss')}", file=sys.stderr)
    print(json.dumps(save, indent=2))


def do_patch(character: str, patches: list[str]):
    """Patch specific fields. Values are auto-parsed as JSON types."""
    save = read_save(character)
    do_backup(character)

    for patch in patches:
        if "=" not in patch:
            print(f"Skipping invalid patch (no =): {patch}", file=sys.stderr)
            continue
        key, value = patch.split("=", 1)
        # Try to parse as JSON (handles ints, lists, bools, etc.)
        try:
            parsed = json.loads(value)
        except json.JSONDecodeError:
            parsed = value  # Keep as string
        save[key] = parsed
        print(f"  {key} = {parsed!r}")

    write_save(save, character)
    print(f"Save patched ({len(patches)} fields)")


def do_load(json_file: str, character: str = "WATCHER"):
    """Load a JSON file as the save."""
    do_backup(character)
    with open(json_file, "r") as f:
        save = json.load(f)
    write_save(save, character)
    print(f"Loaded {json_file} into {save_path(character)}")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    cmd = sys.argv[1]
    character = "WATCHER"

    if cmd == "dump":
        if len(sys.argv) > 2:
            character = sys.argv[2]
        do_dump(character)

    elif cmd == "load":
        if len(sys.argv) < 3:
            print("Usage: save_editor.py load <json_file> [CHARACTER]")
            sys.exit(1)
        json_file = sys.argv[2]
        if len(sys.argv) > 3:
            character = sys.argv[3]
        do_load(json_file, character)

    elif cmd == "patch":
        # Find character and patches
        patches = []
        for arg in sys.argv[2:]:
            if "=" in arg:
                patches.append(arg)
            else:
                character = arg
        if not patches:
            print("Usage: save_editor.py patch [CHARACTER] key=value ...")
            sys.exit(1)
        do_patch(character, patches)

    elif cmd == "backup":
        if len(sys.argv) > 2:
            character = sys.argv[2]
        do_backup(character)

    elif cmd == "restore":
        if len(sys.argv) > 2:
            character = sys.argv[2]
        do_restore(character)

    else:
        print(f"Unknown command: {cmd}")
        print(__doc__)
        sys.exit(1)


if __name__ == "__main__":
    main()
