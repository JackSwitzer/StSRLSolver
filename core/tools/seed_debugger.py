#!/usr/bin/env python3
"""
Seed Debugger Tool - Query and debug seed predictions.

Usage:
    python3 -m core.tools.seed_debugger neow A20WIN
    python3 -m core.tools.seed_debugger encounters A20WIN --act 1
    python3 -m core.tools.seed_debugger hp A20WIN "Jaw Worm" -a 20 -f 1
    python3 -m core.tools.seed_debugger hp-scan A20WIN "Jaw Worm" -a 20
    python3 -m core.tools.seed_debugger cards A20WIN --floor 1 -a 20
    python3 -m core.tools.seed_debugger rng-trace A20WIN --calls 10

Key findings from decompiled game code:
- monsterHpRng is reseeded as: seed + floorNum (when entering floor)
- HP has NO multipliers - A7+ just uses different base ranges
- Jaw Worm: A0-A6 (40-44), A7+ (42-46)
- Damage thresholds: A2+ for most enemies, A17+ for some

IMPORTANT: Floor numbering in game:
- Neow counts as floor 1 (initializeFirstRoom increments 0->1)
- First map node is floor 2 (setCurrMapNode increments 1->2)
- So the first combat uses monsterHpRng(seed + 2), NOT seed + 1!
"""

import argparse
import sys
from pathlib import Path

# Add project root to path
_project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(_project_root))

from core.state.rng import Random, seed_to_long, long_to_seed


# =============================================================================
# NEOW DEBUGGING
# =============================================================================

def debug_neow(seed_str: str, verbose: bool = False):
    """Debug Neow option generation for a seed."""
    seed = seed_to_long(seed_str.upper())
    print(f"Seed: {seed_str.upper()} ({seed})")
    print()

    # Neow uses fresh RNG from seed
    neow_rng = Random(seed)

    # Category pools (exact order from NeowReward.java)
    cat0 = ["THREE_CARDS", "ONE_RANDOM_RARE_CARD", "REMOVE_CARD",
            "UPGRADE_CARD", "TRANSFORM_CARD", "RANDOM_COLORLESS"]
    cat1 = ["THREE_SMALL_POTIONS", "RANDOM_COMMON_RELIC", "TEN_PERCENT_HP_BONUS",
            "THREE_ENEMY_KILL", "HUNDRED_GOLD"]
    drawbacks = ["TEN_PERCENT_HP_LOSS", "NO_GOLD", "CURSE", "PERCENT_DAMAGE"]

    print("=== Neow Options (from NeowEvent.rng) ===")
    print()

    # Slot 0 (Category 0)
    roll0 = neow_rng.random(len(cat0) - 1)
    print(f"Slot 1: random({len(cat0)-1}) = {roll0}")
    print(f"  -> {cat0[roll0]}")
    if verbose:
        print(f"  Counter after: {neow_rng.counter}")
    print()

    # Slot 1 (Category 1)
    roll1 = neow_rng.random(len(cat1) - 1)
    print(f"Slot 2: random({len(cat1)-1}) = {roll1}")
    print(f"  -> {cat1[roll1]}")
    if verbose:
        print(f"  Counter after: {neow_rng.counter}")
    print()

    # Slot 2 - Drawback first, then conditional pool
    drawback_roll = neow_rng.random(len(drawbacks) - 1)
    drawback = drawbacks[drawback_roll]
    print(f"Slot 3 Drawback: random({len(drawbacks)-1}) = {drawback_roll}")
    print(f"  -> {drawback}")

    # Build conditional pool
    cat2 = ["RANDOM_COLORLESS_2"]
    if drawback != "CURSE":
        cat2.append("REMOVE_TWO")
    cat2.append("ONE_RARE_RELIC")
    cat2.append("THREE_RARE_CARDS")
    if drawback != "NO_GOLD":
        cat2.append("TWO_FIFTY_GOLD")
    cat2.append("TRANSFORM_TWO_CARDS")
    if drawback != "TEN_PERCENT_HP_LOSS":
        cat2.append("TWENTY_PERCENT_HP_BONUS")

    roll2 = neow_rng.random(len(cat2) - 1)
    print(f"Slot 3 Reward: random({len(cat2)-1}) = {roll2} (pool size: {len(cat2)})")
    print(f"  Pool: {cat2}")
    print(f"  -> {cat2[roll2]}")
    if verbose:
        print(f"  Counter after: {neow_rng.counter}")
    print()

    # Slot 3 (Boss Relic)
    print("Slot 4: BOSS_RELIC (always)")
    print()

    print("=== Summary ===")
    print(f"1. {cat0[roll0]}")
    print(f"2. {cat1[roll1]}")
    print(f"3. {cat2[roll2]} + {drawback}")
    print(f"4. BOSS_RELIC")
    print()

    # Note about relic selection
    print("NOTE: Actual relic names depend on relicRng pool shuffling,")
    print("which happens during dungeon init BEFORE Neow.")


# =============================================================================
# ENCOUNTER DEBUGGING
# =============================================================================

def debug_encounters(seed_str: str, act: int = 1, verbose: bool = False):
    """Debug encounter generation for a seed."""
    seed = seed_to_long(seed_str.upper())
    print(f"Seed: {seed_str.upper()} ({seed})")
    print(f"Act: {act}")
    print()

    try:
        from core.generation.encounters import generate_exordium_encounters
    except ImportError:
        print("ERROR: Could not import encounters module")
        return

    if act == 1:
        monster_rng = Random(seed)
        normal, elite = generate_exordium_encounters(monster_rng)

        print("=== Normal Encounters ===")
        for i, enc in enumerate(normal):
            weak = "WEAK" if i < 3 else "STRONG"
            print(f"  {i+1}. {enc} ({weak})")

        print()
        print("=== Elite Encounters ===")
        for i, enc in enumerate(elite[:5]):
            print(f"  {i+1}. {enc}")

        if verbose:
            print()
            print(f"monsterRng counter after: {monster_rng.counter}")
    else:
        print(f"Act {act} encounters not yet implemented")


# =============================================================================
# ENEMY DATA - Complete from decompiled game code
# =============================================================================
#
# Ascension thresholds:
# - HP: a7 (normal/elite), a8/a9 (elite/boss)
# - Damage: a2 (most), a3 (elites), a4 (bosses)
# - Special: a17/a18/a19 (extra scaling)
#
# Format: "Enemy Name": {
#   "hp": {"base": (min, max), "a7": (min, max), "a9": (min, max)},
#   "damage": {"move": value OR {"base": x, "a2": y}},
#   "first_move": "move_name",
#   "notes": "behavior"
# }

ENEMY_DATA = {
    # =========================================================================
    # ACT 1 - EXORDIUM
    # =========================================================================

    # --- Normal Encounters ---
    "Jaw Worm": {
        "hp": {"base": (40, 44), "a7": (42, 46)},
        "damage": {"chomp": {"base": 11, "a2": 12}, "thrash": 7, "bellow": 0},
        "first_move": "chomp",
        "notes": "Always Chomp first. Bellow: +3 STR, +6 block (+4/+9 A17)."
    },
    "Cultist": {
        "hp": {"base": (48, 54), "a7": (50, 56)},
        "damage": {"dark_strike": 6},
        "first_move": "incantation",
        "notes": "Always Incantation first (+3 ritual, +5 A17). Then attacks."
    },
    "Blue Slaver": {
        "hp": {"base": (46, 50), "a7": (48, 52)},
        "damage": {"stab": {"base": 12, "a2": 13}, "rake": {"base": 7, "a2": 8}},
        "notes": "Rake applies 1 Weak (2 A17)."
    },
    "Red Slaver": {
        "hp": {"base": (46, 50), "a7": (48, 52)},
        "damage": {"stab": {"base": 13, "a2": 14}, "scrape": {"base": 8, "a2": 9}},
        "notes": "Scrape applies 1 Vuln (2 A17). Entangle at <50% HP."
    },
    "Looter": {
        "hp": {"base": (44, 48), "a7": (46, 50)},
        "damage": {"mug": {"base": 10, "a2": 11}, "lunge": {"base": 12, "a2": 14}, "smoke_bomb": 0},
        "notes": "Mug steals 15 gold. Escapes turn 3. Drops 10-20 gold on kill."
    },
    "Fungi Beast": {
        "hp": {"base": (22, 28), "a7": (24, 30)},
        "damage": {"bite": 6, "grow": 0},
        "notes": "Grow: +3 STR (+5 A17). Spore Cloud on death: 2 Vuln."
    },
    "Louse Green": {
        "hp": {"base": (11, 17), "a7": (12, 18)},
        "damage": {"bite": {"base": "5-7", "a2": "6-8"}},
        "notes": "Damage is random in range. Curl Up: 3-7 block. Spit Web: 2 Weak."
    },
    "Louse Red": {
        "hp": {"base": (10, 15), "a7": (11, 16)},
        "damage": {"bite": {"base": "5-7", "a2": "6-8"}},
        "notes": "Damage is random in range. Curl Up: 3-7 block. Grow: +3 STR."
    },

    # --- Slimes ---
    "Acid Slime L": {
        "hp": {"base": (65, 69), "a7": (67, 72)},
        "damage": {"tackle": {"base": 16, "a2": 18}, "corrosive_spit": {"base": 11, "a2": 12}, "lick": 0, "split": 0},
        "notes": "Splits at <50% HP into 2 Acid Slime M. Lick: 2 Weak."
    },
    "Acid Slime M": {
        "hp": {"base": (28, 32), "a7": (29, 34)},
        "damage": {"tackle": {"base": 10, "a2": 12}, "corrosive_spit": {"base": 7, "a2": 8}, "lick": 0},
        "notes": "Corrosive Spit shuffles 2 Slimed into discard. Lick: 1 Weak."
    },
    "Acid Slime S": {
        "hp": {"base": (8, 12), "a7": (9, 13)},
        "damage": {"tackle": {"base": 3, "a2": 4}, "lick": 0},
        "notes": "Lick applies 1 Weak."
    },
    "Spike Slime L": {
        "hp": {"base": (64, 70), "a7": (67, 73)},
        "damage": {"flame_tackle": {"base": 16, "a2": 18}, "lick": 0, "split": 0},
        "notes": "Splits at <50% HP. Flame Tackle shuffles 2 Slimed. Lick: 1 Frail."
    },
    "Spike Slime M": {
        "hp": {"base": (28, 32), "a7": (29, 34)},
        "damage": {"flame_tackle": {"base": 8, "a2": 10}, "lick": 0},
        "notes": "Flame Tackle shuffles 1 Slimed. Lick: 1 Frail."
    },
    "Spike Slime S": {
        "hp": {"base": (10, 14), "a7": (11, 15)},
        "damage": {"tackle": {"base": 5, "a2": 6}},
    },

    # --- Gremlins ---
    "Mad Gremlin": {
        "hp": {"base": (20, 24), "a7": (21, 25)},
        "damage": {"scratch": {"base": 4, "a2": 5}},
        "notes": "Part of Gremlin Gang."
    },
    "Sneaky Gremlin": {
        "hp": {"base": (10, 14), "a7": (11, 15)},
        "damage": {"puncture": {"base": 9, "a2": 10}},
        "notes": "Part of Gremlin Gang."
    },
    "Fat Gremlin": {
        "hp": {"base": (13, 17), "a7": (14, 18)},
        "damage": {"smash": {"base": 4, "a2": 5}},
        "notes": "Applies 1 Weak (2 A17). Part of Gremlin Gang."
    },
    "Gremlin Wizard": {
        "hp": {"base": (23, 27), "a7": (24, 28)},
        "damage": {"ultimate_blast": {"base": 25, "a2": 30}},
        "notes": "Charges 3 turns then blasts. Part of Gremlin Gang."
    },
    "Shield Gremlin": {
        "hp": {"base": (12, 15), "a7": (13, 17)},
        "damage": {"protect": 0, "shield_bash": {"base": 6, "a2": 8}},
        "notes": "Gives 7 block (11 A17) to front ally. Part of Gremlin Gang."
    },
    "Gremlin Tsundere": {
        "hp": {"base": (12, 15), "a7": (13, 17)},
        "damage": {"shield_bash": {"base": 6, "a2": 8}},
        "notes": "Shy gremlin. Protects self with 7 block (11 A17). Runs away."
    },
    "Apology Slime": {
        "hp": {"base": (8, 12), "a7": (8, 12)},
        "damage": {"tackle": 3, "lick": 0},
        "notes": "Special slime. Applies 1 Weak. Says sorry."
    },

    # --- Act 1 Elites ---
    "Gremlin Nob": {
        "hp": {"base": (82, 86), "a8": (85, 90)},
        "damage": {"bellow": 0, "rush": {"base": 14, "a3": 16}, "skull_bash": {"base": 6, "a3": 8}},
        "first_move": "bellow",
        "notes": "Bellow first (Enrage: +2 STR per skill, +3 A18). Rush, Skull Bash (Vuln 2)."
    },
    "Lagavulin": {
        "hp": {"base": (109, 111), "a8": (112, 115)},
        "damage": {"attack": {"base": 18, "a3": 20}, "siphon_soul": 0},
        "first_move": "sleep",
        "notes": "Sleeps 3 turns with 8 Metallicize. Siphon Soul: -1 STR, -1 DEX (-2 A18)."
    },
    "Sentry": {
        "hp": {"base": (38, 42), "a8": (39, 45)},
        "damage": {"beam": {"base": 9, "a3": 10}, "bolt": 0},
        "notes": "3 Sentries. Bolt adds 2 Daze (3 A18). Pattern: L-Beam R-Bolt M-Beam."
    },

    # --- Act 1 Bosses ---
    "Slime Boss": {
        "hp": {"base": (140, 140), "a9": (150, 150)},
        "damage": {"slam": {"base": 35, "a4": 38}, "preparing": 0, "split": 0},
        "first_move": "preparing",
        "notes": "Slam -> Preparing pattern. Goop: 3 Slimed. Splits <50% HP into 2 L Slimes."
    },
    "The Guardian": {
        "hp": {"base": (240, 240), "a9": (250, 250)},
        "damage": {"fierce_bash": {"base": 32, "a4": 36}, "roll_attack": {"base": 9, "a4": 10}, "twin_slam": {"base": 8, "a4": 9}, "whirlwind": {"base": 5, "a4": 6}},
        "notes": "Mode Shift at 30 dmg (40 A9+). Sharp Hide: 3 dmg when attacked (4 A19)."
    },
    "Hexaghost": {
        "hp": {"base": (250, 250), "a9": (264, 264)},
        "damage": {"divider": "6x(hp/12+1)", "sear": {"base": 6, "a4": 7}, "inferno": {"base": "2x6", "a19": "3x6"}, "tackle": {"base": 5, "a4": 6}},
        "first_move": "activate",
        "notes": "Divider = 6 hits of (your_hp/12 + 1). Sear: shuffles Burn+. Inferno: 6 hits."
    },

    # =========================================================================
    # ACT 2 - THE CITY
    # =========================================================================

    # --- Normal Encounters ---
    "Chosen": {
        "hp": {"base": (95, 99), "a7": (98, 103)},
        "damage": {"poke": {"base": 5, "a2": 6}, "zap": 0, "debilitate": 0, "hex": {"base": 6, "a2": 7}},
        "first_move": "poke",
        "notes": "Poke (x2). Zap: 3 Vuln + 3 Weak. Hex adds 1 Hex curse. Debilitate: 3 Vuln."
    },
    "Byrd": {
        "hp": {"base": (25, 31), "a7": (26, 33)},
        "damage": {"peck": {"base": 1, "a2": 1}, "swoop": {"base": 12, "a2": 14}, "headbutt": 3, "caw": 0},
        "notes": "Starts with 3 Flight (4 A17). Peck: 1x5. Caw: +1 STR. Grounded after stunned."
    },
    "Spheric Guardian": {
        "hp": {"base": (20, 20), "a7": (20, 20)},
        "damage": {"slam": {"base": 10, "a2": 11}, "activate": 0, "attack_debuff": 0},
        "notes": "Barricade. Slam (x2). Harden: 15 block. Starts with 40 block."
    },
    "Centurion": {
        "hp": {"base": (76, 80), "a7": (78, 83)},
        "damage": {"slash": {"base": 12, "a2": 14}, "fury": {"base": 6, "a2": 7}, "defend": 0},
        "notes": "Fury: x3 hits. Defend: 15 block (20 A17). Protect nearby Mystics."
    },
    "Mystic": {
        "hp": {"base": (48, 56), "a7": (50, 58)},
        "damage": {"attack": {"base": 8, "a2": 9}, "buff": 0, "heal": 0},
        "notes": "Heal: 16 HP to ally. Buff: +2 STR to ally (+3 A17)."
    },
    "Snecko": {
        "hp": {"base": (114, 120), "a7": (120, 125)},
        "damage": {"perplexing_glare": 0, "bite": {"base": 15, "a2": 18}, "tail_whip": {"base": 8, "a2": 10}},
        "first_move": "perplexing_glare",
        "notes": "Glare first (Confused 1 Vuln). Bite. Tail Whip: 2 Vuln."
    },
    "Snake Plant": {
        "hp": {"base": (75, 79), "a7": (78, 82)},
        "damage": {"chomp": {"base": 7, "a2": 8}, "enfeebling_spores": 0},
        "notes": "Chomp: x3 hits. Spores: 2 Weak + 2 Frail."
    },
    "Shelled Parasite": {
        "hp": {"base": (68, 72), "a7": (71, 76)},
        "damage": {"double_strike": {"base": 6, "a2": 7}, "suck": {"base": 10, "a2": 12}, "fell": {"base": 18, "a2": 21}},
        "notes": "Double Strike: x2. Suck heals. Fell: lose 2 Plated Armor."
    },
    "Mugger": {
        "hp": {"base": (48, 52), "a7": (50, 54)},
        "damage": {"mug": {"base": 10, "a2": 11}, "lunge": {"base": 16, "a2": 18}, "smoke_bomb": 0},
        "notes": "Mug steals 15 gold. Escapes after 2 lurkings."
    },

    # --- Thieves ---
    "Pointy": {
        "hp": {"base": (30, 30), "a7": (34, 34)},
        "damage": {"attack": {"base": 5, "a2": 6}},
        "notes": "Part of Romeo & Juliet (Thieves) encounter."
    },
    "Romeo": {
        "hp": {"base": (35, 39), "a7": (37, 41)},
        "damage": {"mock": 0, "agonizing_slash": {"base": 10, "a2": 12}},
        "notes": "Bandit Leader. Mock: 2 Weak. Agonizing Slash hits hard."
    },
    "Bear": {
        "hp": {"base": (38, 42), "a7": (40, 44)},
        "damage": {"maul": {"base": 18, "a2": 20}, "lunge": {"base": 9, "a2": 10}, "bear_hug": 0},
        "notes": "Bear Hug: -2 DEX (-4 A17). Lunge: 9 block."
    },

    # --- Minions ---
    "TorchHead": {
        "hp": {"base": (38, 40), "a7": (40, 45)},
        "damage": {"tackle": 7},
        "notes": "The Collector minion. Dies and respawns."
    },
    "Bronze Orb": {
        "hp": {"base": (52, 58), "a9": (54, 60)},
        "damage": {"beam": 8, "stasis": 0, "support_beam": 0},
        "notes": "Bronze Automaton minion. Stasis: shuffles card into draw pile. Support Beam: 12 block."
    },

    # --- Act 2 Elites ---
    "Gremlin Leader": {
        "hp": {"base": (140, 148), "a8": (145, 155)},
        "damage": {"rally": 0, "encourage": 0},
        "notes": "Rallies minions, buffs +3 STR (+5 A18). Spawns gremlins."
    },
    "Book of Stabbing": {
        "hp": {"base": (160, 164), "a8": (168, 172)},
        "damage": {"multi_stab": {"base": 6, "a3": 7}, "single_stab": {"base": 21, "a3": 24}},
        "notes": "Multi Stab: +1 hit per turn (starts at 2). Vulnerable target."
    },
    "Taskmaster": {
        "hp": {"base": (54, 60), "a8": (58, 64)},
        "damage": {"scouring_whip": {"base": 7, "a3": 8}},
        "notes": "Spawns 2 orbs. Whip adds 1 Wound (2 A18)."
    },

    # --- Act 2 Bosses ---
    "Bronze Automaton": {
        "hp": {"base": (300, 300), "a9": (320, 320)},
        "damage": {"flail": {"base": 7, "a4": 8}, "hyper_beam": {"base": 45, "a4": 50}, "boost": 0},
        "notes": "Spawns 2 orbs. Hyper Beam: -2 STR after. Boost: +10 STR."
    },
    "The Champ": {
        "hp": {"base": (420, 420), "a9": (440, 440)},
        "damage": {"face_slap": {"base": 12, "a4": 14}, "heavy_slash": {"base": 16, "a4": 18}, "gloat": 0, "taunt": 0, "execute": {"base": 10, "a4": 10}},
        "notes": "Execute: x2. Phase 2 at <50% HP: +6 STR, clear debuffs. Gloat: +2 STR (+3 A19)."
    },
    "The Collector": {
        "hp": {"base": (282, 282), "a9": (300, 300)},
        "damage": {"fireball": {"base": 18, "a4": 21}, "mega_debuff": 0, "spawn": 0},
        "notes": "Spawns 2 TorchHeads. Mega Debuff: 3 Weak + 3 Vuln + 3 Frail."
    },

    # =========================================================================
    # ACT 3 - THE BEYOND
    # =========================================================================

    # --- Normal Encounters ---
    "Darkling": {
        "hp": {"base": (48, 56), "a7": (50, 59)},
        "damage": {"nip": {"base": 7, "a2": 8}, "chomp": 0, "harden": 0, "reincarnate": 0},
        "notes": "3 Darklings. Revives at 50% HP unless all killed together. Harden: 12 block."
    },
    "Orb Walker": {
        "hp": {"base": (90, 96), "a7": (92, 102)},
        "damage": {"laser": {"base": 10, "a2": 11}, "claw": {"base": 15, "a2": 16}},
        "first_move": "laser",
        "notes": "Focus +2 at half HP (A17). Laser: burn card."
    },
    "Spiker": {
        "hp": {"base": (42, 56), "a7": (44, 60)},
        "damage": {"cut": {"base": 7, "a2": 8}, "spike": 0},
        "notes": "Thorns. Spikes: +2 Thorns."
    },
    "Exploder": {
        "hp": {"base": (30, 30), "a7": (30, 35)},
        "damage": {"slam": {"base": 9, "a2": 11}, "explode": {"base": 30, "a2": 30}},
        "notes": "Explodes turn 3 for 30 dmg."
    },
    "Repulsor": {
        "hp": {"base": (29, 35), "a7": (31, 37)},
        "damage": {"bash": {"base": 11, "a2": 13}, "repulse": 0},
        "notes": "Repulse: shuffles 2 Daze (Bash: 1 Daze A17)."
    },
    "Maw": {
        "hp": {"base": (300, 300), "a7": (300, 300)},
        "damage": {"slam": {"base": 25, "a2": 30}, "drool": 0, "roar": 0, "nom": {"base": 5, "a2": 5}},
        "notes": "Drool: 3 Weak + 3 Frail (5/5 A17). Roar: debuffs."
    },
    "Spire Growth": {
        "hp": {"base": (170, 190), "a7": (180, 200)},
        "damage": {"quick_tackle": {"base": 16, "a2": 18}, "smash": {"base": 22, "a2": 25}, "constrict": 0},
        "notes": "Constrict: 10 HP dmg/turn (12 A17)."
    },
    "Writhing Mass": {
        "hp": {"base": (160, 160), "a7": (175, 175)},
        "damage": {"multi_strike": 9, "flail": {"base": 15, "a2": 16}, "strong_strike": {"base": 32, "a2": 38}, "implant": 0},
        "notes": "Reactive: copies last played attack. Multi-strike: x3."
    },
    "Transient": {
        "hp": {"base": (999, 999), "a7": (999, 999)},
        "damage": {"attack": {"base": 30, "a2": 40}},
        "notes": "Fades after 5 turns. +10 STR per turn. Kill for gold/relic."
    },

    # --- Act 3 Elites ---
    "Giant Head": {
        "hp": {"base": (500, 500), "a8": (520, 520)},
        "damage": {"count": 13, "glare": 0},
        "notes": "Slow. Count charges, then big attack. Glare: 1 Weak."
    },
    "Nemesis": {
        "hp": {"base": (185, 185), "a8": (200, 200)},
        "damage": {"scythe": 45, "debuff": 0, "attack": {"base": 6, "a3": 7}},
        "notes": "Intangible. Burn cards. Attack: x3."
    },
    "Reptomancer": {
        "hp": {"base": (180, 190), "a8": (190, 200)},
        "damage": {"snake_strike": {"base": 13, "a3": 16}, "big_bite": {"base": 30, "a3": 34}, "summon": 0},
        "notes": "Spawns daggers. 2 daggers A18."
    },
    "Snake Dagger": {
        "hp": {"base": (20, 25), "a7": (20, 25)},
        "damage": {"stab": 9, "explode": 25},
        "notes": "Reptomancer minion. Explodes on death for 25 dmg."
    },

    # --- Act 3 Bosses ---
    "Awakened One": {
        "hp": {"base": (300, 300), "a9": (320, 320)},
        "damage": {"slash": 20, "soul_strike": 6, "sludge": 18, "tackle": 10},
        "notes": "Phase 2 at 0 HP: heals to 300 (320 A9). Reborn: +2 STR per power you play."
    },
    "Donu": {
        "hp": {"base": (250, 250), "a9": (265, 265)},
        "damage": {"circle_of_power": 0, "beam": {"base": 10, "a4": 12}},
        "notes": "Artifact 2. Circle: +3 STR to both. Beam: x2."
    },
    "Deca": {
        "hp": {"base": (250, 250), "a9": (265, 265)},
        "damage": {"square_of_protection": 0, "beam": {"base": 10, "a4": 12}},
        "notes": "Artifact 2. Square: 16 block to both. Beam: x2."
    },
    "Time Eater": {
        "hp": {"base": (456, 456), "a9": (480, 480)},
        "damage": {"reverberate": 7, "head_slam": {"base": 26, "a4": 32}, "ripple": 0, "haste": 0},
        "notes": "12 cards end turn, heal 2 HP. Reverberate: x3. Haste: +2 STR, clears."
    },

    # =========================================================================
    # ACT 4 - THE ENDING
    # =========================================================================

    "Spire Shield": {
        "hp": {"base": (110, 110), "a8": (120, 120)},
        "damage": {"bash": {"base": 12, "a3": 14}, "fortify": 0, "smash": {"base": 34, "a3": 38}},
        "notes": "Gives Spire Spear 30 Artifact. Fortify: 30 block."
    },
    "Spire Spear": {
        "hp": {"base": (160, 160), "a8": (180, 180)},
        "damage": {"burn_strike": {"base": 5, "a3": 6}, "piercer": {"base": 10, "a3": 12}, "skewer": {"base": 9, "a3": 10}},
        "notes": "Burns on hit. Burn Strike: x2. Piercer: x2."
    },
    "Corrupt Heart": {
        "hp": {"base": (750, 750), "a9": (800, 800)},
        "damage": {"blood_shots": {"base": "2x12", "a4": "2x15"}, "echo": {"base": 40, "a4": 45}, "debilitate": 0},
        "first_move": "debilitate",
        "notes": "Beat of Death: 1 dmg per card (2 A19). Invincible 300. Blood Shots hits 12 (15 A4) times."
    },
}

def get_hp_range(enemy_data: dict, ascension: int) -> tuple:
    """Get HP range for an enemy at given ascension.

    Thresholds from decompiled code:
    - a7: Normal enemies get +HP at A7
    - a8: Elites get +HP at A8
    - a9: Bosses get +HP at A9
    """
    hp_data = enemy_data.get("hp", {})
    # Check thresholds in order (highest first)
    if ascension >= 9 and "a9" in hp_data:
        return hp_data["a9"], "A9+"
    if ascension >= 8 and "a8" in hp_data:
        return hp_data["a8"], "A8+"
    if ascension >= 7 and "a7" in hp_data:
        return hp_data["a7"], "A7+"
    return hp_data.get("base", (0, 0)), "base"


def get_damage(damage_val, ascension: int) -> int:
    """Get damage value at given ascension."""
    if isinstance(damage_val, dict):
        # Check thresholds in order (highest first)
        for threshold in ["a19", "a17", "a4", "a3", "a2"]:
            if threshold in damage_val:
                asc_level = int(threshold[1:])
                if ascension >= asc_level:
                    return damage_val[threshold]
        return damage_val.get("base", 0)
    elif isinstance(damage_val, tuple):
        return damage_val  # Return range as-is
    return damage_val


def debug_hp(seed_str: str, enemy: str, ascension: int = 0, floor: int = 1, verbose: bool = False):
    """Debug enemy HP calculation for a seed."""
    seed = seed_to_long(seed_str.upper())
    game_floor = floor + 1  # Map floor 1 -> game floor 2 (Neow = floor 1)

    print(f"Seed: {seed_str.upper()} ({seed})")
    print(f"Enemy: {enemy}")
    print(f"Ascension: {ascension}")
    print(f"Map floor: {floor} (game floorNum: {game_floor})")
    print()

    hp_rng = Random(seed + game_floor)

    enemy_data = ENEMY_DATA.get(enemy)
    if not enemy_data:
        print(f"ERROR: Unknown enemy '{enemy}'")
        print(f"Known enemies: {sorted(ENEMY_DATA.keys())}")
        return

    hp_range, range_label = get_hp_range(enemy_data, ascension)
    final_hp = hp_rng.random_int_range(hp_range[0], hp_range[1])

    print(f"HP range ({range_label}): {hp_range[0]}-{hp_range[1]}")
    print(f"Final HP: {final_hp}")

    if verbose:
        print()
        print(f"monsterHpRng counter after: {hp_rng.counter}")


def debug_enemy(seed_str: str, enemy: str, ascension: int = 0, floor: int = 1):
    """Show comprehensive enemy info for a seed/floor."""
    seed = seed_to_long(seed_str.upper())
    game_floor = floor + 1

    print(f"=== {enemy} ===")
    print(f"Seed: {seed_str.upper()} | Map floor: {floor} | Ascension: {ascension}")
    print()

    enemy_data = ENEMY_DATA.get(enemy)
    if not enemy_data:
        print(f"ERROR: Unknown enemy '{enemy}'")
        print(f"Known enemies: {sorted(ENEMY_DATA.keys())}")
        return

    # HP
    hp_rng = Random(seed + game_floor)
    hp_range, range_label = get_hp_range(enemy_data, ascension)
    hp = hp_rng.random_int_range(hp_range[0], hp_range[1])
    print(f"HP: {hp} (range {hp_range[0]}-{hp_range[1]} at {range_label})")

    # Damage
    if "damage" in enemy_data:
        print()
        print("Damage:")
        for move, dmg in enemy_data["damage"].items():
            dmg_val = get_damage(dmg, ascension)
            if isinstance(dmg_val, tuple):
                print(f"  {move}: {dmg_val[0]}-{dmg_val[1]}")
            elif isinstance(dmg_val, str):
                print(f"  {move}: {dmg_val}")
            elif dmg_val == 0:
                print(f"  {move}: (no damage)")
            elif dmg_val == -1:
                print(f"  {move}: (debuff)")
            else:
                print(f"  {move}: {dmg_val}")

    # First move
    if "first_move" in enemy_data:
        print()
        print(f"First move: {enemy_data['first_move']}")

    # Notes
    if "notes" in enemy_data:
        print()
        print(f"Notes: {enemy_data['notes']}")


# =============================================================================
# RNG TRACE
# =============================================================================

def trace_rng(seed_str: str, calls: int = 10, stream: str = "card"):
    """Trace RNG calls for a seed."""
    seed = seed_to_long(seed_str.upper())
    print(f"Seed: {seed_str.upper()} ({seed})")
    print(f"Stream: {stream}")
    print()

    rng = Random(seed)

    print(f"=== First {calls} RNG calls ===")
    print()
    print("Call | random(99) | random(5) | randomBool | counter")
    print("-----|------------|-----------|------------|--------")

    for i in range(calls):
        # Save state
        s0, s1 = rng._rng.seed0, rng._rng.seed1
        counter = rng.counter

        # Make calls with fresh RNG copies
        rng_copy = Random(seed, counter)
        r99 = rng_copy.random(99)

        rng_copy = Random(seed, counter)
        r5 = rng_copy.random(5)

        rng_copy = Random(seed, counter)
        rb = rng_copy.random_boolean()

        print(f"  {i+1:2} |    {r99:3}    |     {r5}     |   {str(rb):5}   |   {counter}")

        # Advance main RNG
        rng.random(99)


# =============================================================================
# CARD REWARD DEBUGGING
# =============================================================================

def debug_cards(seed_str: str, floor: int = 1, ascension: int = 0,
                neow_option: str = "NONE", verbose: bool = False):
    """Debug card reward generation for a seed."""
    seed = seed_to_long(seed_str.upper())
    print(f"Seed: {seed_str.upper()} ({seed})")
    print(f"Floor: {floor}")
    print(f"Ascension: {ascension}")
    print(f"Neow Option: {neow_option}")
    print()

    # cardRng starts fresh from seed
    card_rng = Random(seed)
    initial_counter = 0

    # Neow options that consume cardRng
    neow_consumption = {
        "NONE": 0,
        "HUNDRED_GOLD": 0,
        "THREE_ENEMY_KILL": 0,
        "UPGRADE_CARD": 0,
        "REMOVE_CARD": 0,
        "TEN_PERCENT_HP_BONUS": 0,
        "RANDOM_COMMON_RELIC": 0,
        "RANDOM_COLORLESS": 3,  # Generates colorless cards
        "RANDOM_COLORLESS_2": 3,
        "THREE_CARDS": 3,  # Card selection
        "THREE_RARE_CARDS": 3,
        "ONE_RANDOM_RARE_CARD": 1,
        "TRANSFORM_CARD": 1,
        "TRANSFORM_TWO_CARDS": 2,
        "BOSS_RELIC": 0,  # But may trigger Calling Bell etc.
    }

    consumed = neow_consumption.get(neow_option, 0)
    if consumed > 0:
        print(f"Neow consumes ~{consumed} cardRng calls")
        for _ in range(consumed):
            card_rng.random(99)
        print(f"cardRng counter after Neow: {card_rng.counter}")
        print()

    # Simulate card reward (simplified)
    print("=== Card Reward Simulation ===")
    print("(Simplified - actual game has cardBlizzRandomizer)")
    print()

    # 3 cards per reward
    rarities = []
    for i in range(3):
        roll = card_rng.random(99)
        if roll < 3:
            rarity = "RARE"
        elif roll < 40:
            rarity = "UNCOMMON"
        else:
            rarity = "COMMON"
        rarities.append((roll, rarity))
        print(f"Card {i+1}: roll={roll} -> {rarity}")
        # Card selection would consume another call
        card_rng.random(99)  # Pool selection

    if verbose:
        print()
        print(f"cardRng counter after 3 cards: {card_rng.counter}")


# =============================================================================
# HP SCAN - MULTI-FLOOR TRACE
# =============================================================================

def scan_hp(seed_str: str, enemy: str, ascension: int = 0, floors: int = 5):
    """Scan HP values across multiple map floors to help verify predictions."""
    seed = seed_to_long(seed_str.upper())
    print(f"Seed: {seed_str.upper()} ({seed})")
    print(f"Enemy: {enemy}")
    print(f"Ascension: {ascension}")
    print()

    enemy_data = ENEMY_DATA.get(enemy)
    if not enemy_data:
        print(f"ERROR: Unknown enemy '{enemy}'")
        print(f"Known enemies: {sorted(ENEMY_DATA.keys())}")
        return

    hp_range, range_label = get_hp_range(enemy_data, ascension)

    print(f"HP range ({range_label}): {hp_range[0]}-{hp_range[1]}")
    print()
    print("NOTE: Map floor 1 = game floorNum 2 (Neow counts as floor 1)")
    print()
    print("Map Floor | Game Floor | HP")
    print("----------|------------|----")

    for map_floor in range(1, floors + 1):
        game_floor = map_floor + 1
        hp_rng = Random(seed + game_floor)
        hp = hp_rng.random_int_range(hp_range[0], hp_range[1])
        print(f"    {map_floor:3}    |     {game_floor:3}     | {hp:3}")


def list_enemies():
    """List all known enemies."""
    print("=== Known Enemies ===")
    print()
    for name in sorted(ENEMY_DATA.keys()):
        data = ENEMY_DATA[name]
        hp = data.get("hp", {}).get("base", (0, 0))
        notes = data.get("notes", "")
        print(f"  {name}: {hp[0]}-{hp[1]} HP" + (f" - {notes}" if notes else ""))


# =============================================================================
# MAIN
# =============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Seed Debugger - Query and debug seed predictions",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s neow A20WIN                           # Show all 4 Neow options
  %(prog)s encounters A20WIN                     # Show encounter queue
  %(prog)s enemy A20WIN "Jaw Worm" -a 20 -f 1    # Full enemy info
  %(prog)s hp A20WIN "Jaw Worm" -a 20 -f 1       # Just HP
  %(prog)s hp-scan A20WIN "Jaw Worm" -a 20       # HP across floors
  %(prog)s enemies                               # List all known enemies
  %(prog)s cards A20WIN -f 1 -a 20               # Card rewards
  %(prog)s rng-trace A20WIN -n 10                # Raw RNG trace
        """
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # neow command
    neow_parser = subparsers.add_parser("neow", help="Debug Neow options")
    neow_parser.add_argument("seed", help="Seed string (e.g., A20WIN)")
    neow_parser.add_argument("-v", "--verbose", action="store_true")

    # encounters command
    enc_parser = subparsers.add_parser("encounters", help="Debug encounter queue")
    enc_parser.add_argument("seed", help="Seed string")
    enc_parser.add_argument("--act", type=int, default=1, help="Act number (1-4)")
    enc_parser.add_argument("-v", "--verbose", action="store_true")

    # enemy command (comprehensive)
    enemy_parser = subparsers.add_parser("enemy", help="Full enemy info (HP, damage, moves)")
    enemy_parser.add_argument("seed", help="Seed string")
    enemy_parser.add_argument("enemy", help="Enemy name (e.g., 'Jaw Worm')")
    enemy_parser.add_argument("--ascension", "-a", type=int, default=0)
    enemy_parser.add_argument("--floor", "-f", type=int, default=1)

    # enemies command (list all)
    subparsers.add_parser("enemies", help="List all known enemies")

    # hp command
    hp_parser = subparsers.add_parser("hp", help="Debug enemy HP only")
    hp_parser.add_argument("seed", help="Seed string")
    hp_parser.add_argument("enemy", help="Enemy name (e.g., 'Jaw Worm')")
    hp_parser.add_argument("--ascension", "-a", type=int, default=0)
    hp_parser.add_argument("--floor", "-f", type=int, default=1)
    hp_parser.add_argument("-v", "--verbose", action="store_true")

    # hp-scan command
    scan_parser = subparsers.add_parser("hp-scan", help="Scan HP across floors")
    scan_parser.add_argument("seed", help="Seed string")
    scan_parser.add_argument("enemy", help="Enemy name")
    scan_parser.add_argument("--ascension", "-a", type=int, default=0)
    scan_parser.add_argument("--floors", "-n", type=int, default=5, help="Number of floors to scan")

    # cards command
    cards_parser = subparsers.add_parser("cards", help="Debug card rewards")
    cards_parser.add_argument("seed", help="Seed string")
    cards_parser.add_argument("--floor", "-f", type=int, default=1)
    cards_parser.add_argument("--ascension", "-a", type=int, default=0)
    cards_parser.add_argument("--neow", default="NONE", help="Neow option taken")
    cards_parser.add_argument("-v", "--verbose", action="store_true")

    # rng-trace command
    trace_parser = subparsers.add_parser("rng-trace", help="Trace RNG calls")
    trace_parser.add_argument("seed", help="Seed string")
    trace_parser.add_argument("--calls", "-n", type=int, default=10)
    trace_parser.add_argument("--stream", "-s", default="card")

    args = parser.parse_args()

    if args.command == "neow":
        debug_neow(args.seed, args.verbose)
    elif args.command == "encounters":
        debug_encounters(args.seed, args.act, args.verbose)
    elif args.command == "enemy":
        debug_enemy(args.seed, args.enemy, args.ascension, args.floor)
    elif args.command == "enemies":
        list_enemies()
    elif args.command == "hp":
        debug_hp(args.seed, args.enemy, args.ascension, args.floor, args.verbose)
    elif args.command == "hp-scan":
        scan_hp(args.seed, args.enemy, args.ascension, args.floors)
    elif args.command == "cards":
        debug_cards(args.seed, args.floor, args.ascension, args.neow, args.verbose)
    elif args.command == "rng-trace":
        trace_rng(args.seed, args.calls, args.stream)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
