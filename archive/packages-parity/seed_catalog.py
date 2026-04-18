"""
Verified seed data for parity testing.

All data extracted from verified game runs (docs/vault/verified-seeds.md)
and verified_runs/ JSON files. This serves as ground truth for deterministic
parity tests between the Python engine and the Java game.
"""

# Seed string -> numeric seed conversions (verified against Java SeedHelper)
SEED_CONVERSIONS = {
    "TEST123": 52248462423,
    "1ABCD": 1943283,
    "GA": 570,
    "B": 11,
    "A": 10,
    "H": 17,
    "I": 18,
    "G": 16,
    "D": 13,
    "N": 23,
    "F": 15,
    "C": 12,
    "P": 24,
    "R": 26,
    "Y": 33,
    "L": 21,
    "GC": 572,
    "RELIC": 39642867,
    "33J85JVCVSPJY": -7966379614946285768,
}

# Verified seed data with known game outcomes
VERIFIED_SEEDS = {
    "TEST123": {
        "numeric_seed": 52248462423,
        "character": "WATCHER",
        "ascension": 0,
        "floor1_encounter": "Small Slimes",
        "floor2_encounter": "Jaw Worm",
        "floor3_encounter": "Cultist",
        "floor1_cards": ["Talk to the Hand", "Third Eye", "Empty Body"],
        "floor2_cards": ["Sands of Time", "Simmering Fury", "Tranquility"],
        "floor3_cards": ["Meditate", "Pressure Points", "Signature Move"],
        "neow_offset": 0,  # Unknown Neow choice, 0 offset verified
    },
    "1ABCD": {
        "numeric_seed": 1943283,
        "character": "WATCHER",
        "ascension": 0,
        "floor1_encounter": "Jaw Worm",
        "floor2_encounter": "Cultist",
        "floor3_encounter": "Small Slimes",
        "floor1_cards": ["Like Water", "Bowling Bash", "Deceive Reality"],
        "floor2_cards": ["Sash Whip", "Evaluate", "Worship"],
    },
    "A": {
        "numeric_seed": 10,
        "character": "WATCHER",
        "ascension": 0,
        "neow_choice": "HUNDRED_GOLD",
        "neow_offset": 0,
        "floor1_cards": ["Pray", "Weave", "Foreign Influence"],
        # All 3 floors verified to match
    },
    "H": {
        "numeric_seed": 17,
        "character": "WATCHER",
        "ascension": 0,
        "neow_choice": "REMOVE_CARD",
        "neow_offset": 0,
        "floor1_cards": ["Bowling Bash", "Wallop", "Collect"],
    },
    "I": {
        "numeric_seed": 18,
        "character": "WATCHER",
        "ascension": 0,
        "neow_choice": "PERCENT_DAMAGE",
        "neow_offset": 0,
        "floor1_cards": ["Tantrum", "Pray", "Evaluate"],
        # All 3 floors verified to match
    },
    "G": {
        "numeric_seed": 16,
        "character": "WATCHER",
        "ascension": 0,
        "neow_choice": "THREE_CARDS",
        "neow_offset": 0,
        "floor1_cards": ["Empty Body", "Third Eye", "Sash Whip"],
    },
    "D": {
        "numeric_seed": 13,
        "character": "WATCHER",
        "ascension": 0,
        "neow_choice": "ONE_RANDOM_RARE_CARD",
        "neow_offset": 0,
        "floor1_cards": ["Inner Peace", "Perseverance", "Tranquility"],
    },
    "N": {
        "numeric_seed": 23,
        "character": "WATCHER",
        "ascension": 0,
        "neow_choice": "THREE_ENEMY_KILL",
        "neow_offset": 0,
        "floor1_cards": ["Sanctity", "Meditate", "Talk to the Hand"],
    },
    "33J85JVCVSPJY": {
        "numeric_seed": -7966379614946285768,
        "character": "WATCHER",
        "ascension": 20,
        "neow_choice": "ONE_RANDOM_RARE_CARD",
        "neow_offset": 0,
        "card_rewards": {
            1: ["Consecrate", "Meditate", "Foreign Influence"],
            4: ["Consecrate", "Fasting", "Pressure Points"],
            10: ["Flurry of Blows", "Wave of the Hand", "Third Eye"],
            11: ["Consecrate", "Crush Joints", "Protect"],
            12: ["Prostrate", "Cut Through Fate", "Follow-Up"],
        },
        "path": ["M", "?", "?", "M", "?", "R", "E", "R", "T", "M", "M", "M", "?", "E", "R"],
        "cardRng_final": 62,
        "floors_verified": 7,
        "verification_status": "PERFECT_MATCH",
    },
}

# Verified Neow choice -> cardRng consumption mapping
NEOW_CARDRNG_CONSUMPTION = {
    "UPGRADE_CARD": 0,
    "HUNDRED_GOLD": 0,
    "TEN_PERCENT_HP_BONUS": 0,
    "RANDOM_COMMON_RELIC": 0,
    "THREE_ENEMY_KILL": 0,
    "THREE_CARDS": 0,
    "ONE_RANDOM_RARE_CARD": 0,
    "TRANSFORM_CARD": 0,
    "REMOVE_CARD": 0,
    "PERCENT_DAMAGE": 0,
    "RANDOM_COLORLESS": 3,
    "RANDOM_COLORLESS_2": 3,
    "CURSE": 1,
    "BOSS_SWAP_CALLING_BELL": 9,
}

# Boss selection verification data
# Format: seed -> {act: boss_name}
VERIFIED_BOSSES = {
    "GA": {
        "boss_swap_relic": "Calling Bell",
    },
    "B": {
        "boss_swap_relic": "Astrolabe",
    },
}

# RNG stream verification: first N outputs of XorShift128 for known seeds
# These allow verifying the core RNG implementation is correct
RNG_STREAM_CHECKS = {
    # seed=0 should use Long.MIN_VALUE internally
    0: {
        "description": "Zero seed uses Long.MIN_VALUE internally",
    },
}

# Boss lists (for determinism verification)
BOSS_LISTS = {
    1: ["The Guardian", "Hexaghost", "Slime Boss"],
    2: ["Automaton", "Collector", "Champ"],
    3: ["Awakened One", "Time Eater", "Donu and Deca"],
    4: ["Corrupt Heart"],
}

# Act 4 fixed encounters (no RNG)
ACT4_ENCOUNTERS = {
    "elite": "Spire Shield and Spire Spear",
    "boss": "Corrupt Heart",
}
