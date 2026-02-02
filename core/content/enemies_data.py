"""
Enemy Data - Pure data definitions for all enemies.

This module contains static data for enemies:
- HP ranges (base and ascension-scaled)
- Damage values (base and ascension-scaled)
- Move IDs and names
- Enemy type classifications

AI logic and move selection are in enemies_ai.py.
"""

from typing import Dict, Tuple, Any
from enum import Enum
from dataclasses import dataclass, field


class Intent(Enum):
    """Enemy intent types matching AbstractMonster.Intent."""
    ATTACK = "ATTACK"
    ATTACK_BUFF = "ATTACK_BUFF"
    ATTACK_DEBUFF = "ATTACK_DEBUFF"
    ATTACK_DEFEND = "ATTACK_DEFEND"
    BUFF = "BUFF"
    DEBUFF = "DEBUFF"
    STRONG_DEBUFF = "STRONG_DEBUFF"
    DEFEND = "DEFEND"
    DEFEND_BUFF = "DEFEND_BUFF"
    DEFEND_DEBUFF = "DEFEND_DEBUFF"
    ESCAPE = "ESCAPE"
    MAGIC = "MAGIC"
    SLEEP = "SLEEP"
    STUN = "STUN"
    UNKNOWN = "UNKNOWN"
    NONE = "NONE"


class EnemyType(Enum):
    """Enemy types matching AbstractMonster.EnemyType."""
    NORMAL = "NORMAL"
    ELITE = "ELITE"
    BOSS = "BOSS"


@dataclass
class MoveInfo:
    """Information about an enemy move."""
    move_id: int
    name: str
    intent: Intent
    base_damage: int = -1
    hits: int = 1
    is_multi: bool = False
    block: int = 0
    effects: Dict = field(default_factory=dict)  # Additional effects


@dataclass
class EnemyState:
    """Current state of an enemy during combat."""
    id: str
    name: str
    enemy_type: EnemyType

    # HP
    current_hp: int
    max_hp: int

    # Combat state
    block: int = 0
    strength: int = 0
    powers: Dict[str, int] = field(default_factory=dict)

    # Move tracking
    move_history: list = field(default_factory=list)
    next_move: MoveInfo = None
    first_turn: bool = True

    def last_move(self, move_id: int) -> bool:
        """Check if last move was the given ID."""
        if not self.move_history:
            return False
        return self.move_history[-1] == move_id

    def last_two_moves(self, move_id: int) -> bool:
        """Check if last TWO moves were both the given ID."""
        if len(self.move_history) < 2:
            return False
        return (self.move_history[-1] == move_id and
                self.move_history[-2] == move_id)

    def last_move_before(self, move_id: int) -> bool:
        """Check if move before last was the given ID."""
        if len(self.move_history) < 2:
            return False
        return self.move_history[-2] == move_id


# =============================================================================
# ENEMY DATA DEFINITIONS
# =============================================================================
# Format: {
#     "id": str,           # Game ID
#     "name": str,         # Display name
#     "type": EnemyType,   # NORMAL, ELITE, or BOSS
#     "hp": dict,          # {"base": (min, max), "a7": (min, max), ...}
#     "damage": dict,      # {"move_name": {"base": int, "a2": int, ...}, ...}
#     "moves": dict,       # {move_id: "MoveName", ...}
#     "passives": list,    # List of passive powers at combat start
# }
# =============================================================================


# =============================================================================
# EXORDIUM (ACT 1) - BASIC ENEMIES
# =============================================================================

JAW_WORM_DATA = {
    "id": "JawWorm",
    "name": "Jaw Worm",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (40, 44),
        "a7": (42, 46),
    },
    "damage": {
        "chomp": {"base": 11, "a2": 12},
        "thrash": {"base": 7},
        "thrash_block": {"base": 5},
        "bellow_str": {"base": 3, "a2": 4, "a17": 5},
        "bellow_block": {"base": 6, "a17": 9},
    },
    "moves": {1: "Chomp", 2: "Bellow", 3: "Thrash"},
    "move_ids": {"CHOMP": 1, "BELLOW": 2, "THRASH": 3},
    "passives": [],
}

CULTIST_DATA = {
    "id": "Cultist",
    "name": "Cultist",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (48, 54),
        "a7": (50, 56),
    },
    "damage": {
        "dark_strike": {"base": 6},
        "ritual": {"base": 3, "a2": 4, "a17": 5},
    },
    "moves": {1: "Incantation", 2: "Dark Strike"},
    "move_ids": {"INCANTATION": 1, "DARK_STRIKE": 2},
    "passives": [],
}

ACID_SLIME_M_DATA = {
    "id": "AcidSlime_M",
    "name": "Acid Slime (M)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (28, 32),
        "a7": (29, 34),
    },
    "damage": {
        "spit": {"base": 7, "a2": 8},
        "tackle": {"base": 10, "a2": 12},
    },
    "moves": {1: "Corrosive Spit", 2: "Tackle", 4: "Lick"},
    "move_ids": {"CORROSIVE_SPIT": 1, "TACKLE": 2, "LICK": 4},
    "passives": [],
}

ACID_SLIME_L_DATA = {
    "id": "AcidSlime_L",
    "name": "Acid Slime (L)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (65, 69),
        "a7": (68, 72),
    },
    "damage": {
        "spit": {"base": 11, "a2": 12},
        "tackle": {"base": 16, "a2": 18},
    },
    "moves": {1: "Corrosive Spit", 2: "Tackle", 3: "Split", 4: "Lick"},
    "move_ids": {"CORROSIVE_SPIT": 1, "TACKLE": 2, "SPLIT": 3, "LICK": 4},
    "passives": ["split"],
}

ACID_SLIME_S_DATA = {
    "id": "AcidSlime_S",
    "name": "Acid Slime (S)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (8, 12),
        "a7": (9, 13),
    },
    "damage": {
        "tackle": {"base": 3, "a2": 4},
    },
    "moves": {1: "Tackle", 2: "Lick"},
    "move_ids": {"TACKLE": 1, "LICK": 2},
    "passives": [],
}

SPIKE_SLIME_M_DATA = {
    "id": "SpikeSlime_M",
    "name": "Spike Slime (M)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (28, 32),
        "a7": (29, 34),
    },
    "damage": {
        "tackle": {"base": 8, "a2": 10},
        "frail": {"base": 1, "a17": 2},  # Debuff amount, not damage
    },
    "moves": {1: "Flame Tackle", 4: "Lick"},
    "move_ids": {"FLAME_TACKLE": 1, "LICK": 4},
    "passives": [],
}

SPIKE_SLIME_L_DATA = {
    "id": "SpikeSlime_L",
    "name": "Spike Slime (L)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (64, 70),
        "a7": (67, 73),
    },
    "damage": {
        "tackle": {"base": 16, "a2": 18},
        "frail": {"base": 2, "a17": 3},  # Debuff amount
    },
    "moves": {1: "Flame Tackle", 3: "Split", 4: "Lick"},
    "move_ids": {"FLAME_TACKLE": 1, "SPLIT": 3, "LICK": 4},
    "passives": ["split"],
}

SPIKE_SLIME_S_DATA = {
    "id": "SpikeSlime_S",
    "name": "Spike Slime (S)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (10, 14),
        "a7": (11, 15),
    },
    "damage": {
        "tackle": {"base": 5, "a2": 6},
    },
    "moves": {1: "Tackle"},
    "move_ids": {"TACKLE": 1},
    "passives": [],
}

LOUSE_DATA = {
    "id": "Louse",
    "name": "Louse",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (10, 15),
        "a7": (11, 17),
    },
    "damage": {
        # Damage is rolled at init: 5-7 base, 6-8 at A2
        "bite_min": {"base": 5, "a2": 6},
        "bite_max": {"base": 7, "a2": 8},
        "grow_str": {"base": 3, "a17": 4},
    },
    "moves": {3: "Bite", 4: "Grow"},
    "move_ids": {"BITE": 3, "GROW": 4},
    "passives": ["curl_up"],  # 3-7 block (a7: 4-8, a17: 9-12)
}

LOUSE_NORMAL_DATA = {
    "id": "FuzzyLouseNormal",
    "name": "Red Louse",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (10, 15),
        "a7": (11, 16),
    },
    "damage": {
        "bite_min": {"base": 5, "a2": 6},
        "bite_max": {"base": 7, "a2": 8},
        "grow_str": {"base": 3, "a17": 4},
    },
    "moves": {3: "Bite", 4: "Grow"},
    "move_ids": {"BITE": 3, "GROW": 4},
    "passives": ["curl_up"],
    "curl_up_range": {
        "base": (3, 7),
        "a7": (4, 8),
        "a17": (9, 12),
    },
}

LOUSE_DEFENSIVE_DATA = {
    "id": "FuzzyLouseDefensive",
    "name": "Green Louse",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (11, 17),
        "a7": (12, 18),
    },
    "damage": {
        "bite_min": {"base": 5, "a2": 6},
        "bite_max": {"base": 7, "a2": 8},
        "weak": {"base": 2},  # Spit Web weak amount
    },
    "moves": {3: "Bite", 4: "Spit Web"},
    "move_ids": {"BITE": 3, "SPIT_WEB": 4},
    "passives": ["curl_up"],
    "curl_up_range": {
        "base": (3, 7),
        "a7": (4, 8),
        "a17": (9, 12),
    },
}

FUNGI_BEAST_DATA = {
    "id": "FungiBeast",
    "name": "Fungi Beast",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (22, 28),
        "a7": (24, 28),
    },
    "damage": {
        "bite": {"base": 6},
        "grow_str": {"base": 3, "a2": 4, "a17": 5},
    },
    "moves": {1: "Bite", 2: "Grow"},
    "move_ids": {"BITE": 1, "GROW": 2},
    "passives": ["spore_cloud"],  # 2 vulnerable on death
}

LOOTER_DATA = {
    "id": "Looter",
    "name": "Looter",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (44, 48),
        "a7": (46, 50),
    },
    "damage": {
        "swipe": {"base": 10, "a2": 11},
        "lunge": {"base": 12, "a2": 14},
        "block": {"base": 6},
        "gold_steal": {"base": 15, "a17": 20},
    },
    "moves": {1: "Mug", 2: "Smoke Bomb", 3: "Escape", 4: "Lunge"},
    "move_ids": {"MUG": 1, "SMOKE_BOMB": 2, "ESCAPE": 3, "LUNGE": 4},
    "passives": ["thievery"],
}

MUGGER_DATA = {
    "id": "Mugger",
    "name": "Mugger",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (48, 52),
        "a7": (50, 54),
    },
    "damage": {
        "swipe": {"base": 10, "a2": 11},
        "bigswipe": {"base": 16, "a2": 18},
        "block": {"base": 11, "a17": 17},
        "gold_steal": {"base": 15, "a17": 20},
    },
    "moves": {1: "Mug", 2: "Smoke Bomb", 3: "Escape", 4: "Lunge"},
    "move_ids": {"MUG": 1, "SMOKE_BOMB": 2, "ESCAPE": 3, "BIGSWIPE": 4},
    "passives": ["thievery"],
}

SLAVER_BLUE_DATA = {
    "id": "SlaverBlue",
    "name": "Blue Slaver",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (46, 50),
        "a7": (48, 52),
    },
    "damage": {
        "stab": {"base": 12, "a2": 13},
        "rake": {"base": 7, "a2": 8},
        "weak": {"base": 1, "a17": 2},
    },
    "moves": {1: "Stab", 4: "Rake"},
    "move_ids": {"STAB": 1, "RAKE": 4},
    "passives": [],
}

SLAVER_RED_DATA = {
    "id": "SlaverRed",
    "name": "Red Slaver",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (46, 50),
        "a7": (48, 52),
    },
    "damage": {
        "stab": {"base": 13, "a2": 14},
        "scrape": {"base": 8, "a2": 9},
        "vulnerable": {"base": 1, "a17": 2},
    },
    "moves": {1: "Stab", 2: "Entangle", 4: "Scrape"},
    "move_ids": {"STAB": 1, "ENTANGLE": 2, "SCRAPE": 4},
    "passives": [],
}


# =============================================================================
# EXORDIUM (ACT 1) - ELITES
# =============================================================================

GREMLIN_NOB_DATA = {
    "id": "GremlinNob",
    "name": "Gremlin Nob",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (82, 86),
        "a8": (85, 90),
    },
    "damage": {
        "bellow_enrage": {"base": 2, "a18": 3},
        "rush": {"base": 14, "a3": 16},
        "skull_bash": {"base": 6, "a3": 8},
        "skull_bash_vuln": {"base": 2},
    },
    "moves": {1: "Bellow", 2: "Rush", 3: "Skull Bash"},
    "move_ids": {"BELLOW": 1, "RUSH": 2, "SKULL_BASH": 3},
    "passives": [],
}

LAGAVULIN_DATA = {
    "id": "Lagavulin",
    "name": "Lagavulin",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (109, 111),
        "a8": (112, 115),
    },
    "damage": {
        "attack": {"base": 18, "a3": 20},
        "siphon_str_dex": {"base": 1, "a18": 2},
    },
    "moves": {1: "Sleep", 2: "Attack", 3: "Siphon Soul"},
    "move_ids": {"SLEEP": 1, "ATTACK": 2, "SIPHON_SOUL": 3},
    "passives": ["metallicize"],  # 8 metallicize while asleep
    "metallicize_amount": 8,
}

SENTRY_DATA = {
    "id": "Sentry",
    "name": "Sentry",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (38, 42),
        "a8": (39, 45),
    },
    "damage": {
        "beam": {"base": 9, "a3": 10},
        "dazed": {"base": 2, "a18": 3},
    },
    "moves": {3: "Bolt", 4: "Beam"},
    "move_ids": {"BOLT": 3, "BEAM": 4},
    "passives": ["artifact"],
    "artifact_amount": 1,
}


# =============================================================================
# EXORDIUM (ACT 1) - BOSSES
# =============================================================================

SLIME_BOSS_DATA = {
    "id": "SlimeBoss",
    "name": "Slime Boss",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (140, 140),
        "a9": (150, 150),
    },
    "damage": {
        "slam": {"base": 35, "a4": 38},
        "slimed": {"base": 3, "a19": 5},
    },
    "moves": {1: "Goop Spray", 2: "Preparing", 3: "Slam", 4: "Split"},
    "move_ids": {"GOOP_SPRAY": 1, "PREPARING": 2, "SLAM": 3, "SPLIT": 4},
    "passives": ["split"],
}

THE_GUARDIAN_DATA = {
    "id": "TheGuardian",
    "name": "The Guardian",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (240, 240),
        "a9": (250, 250),
    },
    "damage": {
        "fierce_bash": {"base": 32, "a4": 36},
        "whirlwind": {"base": 5},
        "whirlwind_hits": {"base": 4},
        "roll": {"base": 9, "a4": 10},
        "twin_slam": {"base": 8},
        "twin_slam_hits": {"base": 2},
        "charge_block": {"base": 9},
    },
    "moves": {
        1: "Charge Up", 2: "Fierce Bash", 3: "Vent Steam",
        4: "Whirlwind", 5: "Roll Attack", 6: "Twin Slam"
    },
    "move_ids": {
        "CHARGE_UP": 1, "FIERCE_BASH": 2, "VENT_STEAM": 3,
        "WHIRLWIND": 4, "ROLL_ATTACK": 5, "TWIN_SLAM": 6
    },
    "passives": ["mode_shift"],
    "mode_shift_threshold": {"base": 30, "a9": 35, "a19": 40},
    "sharp_hide": {"base": 3, "a19": 4},
}

HEXAGHOST_DATA = {
    "id": "Hexaghost",
    "name": "Hexaghost",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (250, 250),
        "a9": (264, 264),
    },
    "damage": {
        "sear": {"base": 6},
        "sear_burn": {"base": 1, "a19": 2},
        "tackle": {"base": 5, "a4": 6},
        "tackle_hits": {"base": 2},
        "inflame_str": {"base": 2, "a19": 3},
        "inflame_block": {"base": 12},
        "inferno": {"base": 2, "a4": 3},
        "inferno_hits": {"base": 6},
    },
    "moves": {
        1: "Activate", 2: "Divider", 3: "Sear",
        4: "Tackle", 5: "Inflame", 6: "Inferno"
    },
    "move_ids": {
        "ACTIVATE": 1, "DIVIDER": 2, "SEAR": 3,
        "TACKLE": 4, "INFLAME": 5, "INFERNO": 6
    },
    "passives": [],
}


# =============================================================================
# CITY (ACT 2) - BASIC ENEMIES
# =============================================================================

CHOSEN_DATA = {
    "id": "Chosen",
    "name": "Chosen",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (95, 99),
        "a7": (98, 103),
    },
    "damage": {
        "poke": {"base": 5, "a2": 6},
        "poke_hits": {"base": 2},
        "zap": {"base": 18, "a2": 21},
        "debilitate": {"base": 10, "a2": 12},
    },
    "moves": {1: "Poke", 2: "Zap", 3: "Debilitate", 4: "Drain", 5: "Hex"},
    "move_ids": {"POKE": 1, "ZAP": 2, "DEBILITATE": 3, "DRAIN": 4, "HEX": 5},
    "passives": [],
}

BYRD_DATA = {
    "id": "Byrd",
    "name": "Byrd",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (25, 31),
        "a7": (26, 33),
    },
    "damage": {
        "peck": {"base": 1},
        "peck_hits": {"base": 5, "a2": 6},
        "swoop": {"base": 12, "a2": 14},
        "headbutt": {"base": 3},
    },
    "moves": {1: "Peck", 2: "Fly", 3: "Swoop", 4: "Stunned", 5: "Headbutt", 6: "Caw"},
    "move_ids": {
        "PECK": 1, "GO_AIRBORNE": 2, "SWOOP": 3,
        "STUNNED": 4, "HEADBUTT": 5, "CAW": 6
    },
    "passives": ["flight"],
    "flight_amount": {"base": 3, "a17": 4},
}

CENTURION_DATA = {
    "id": "Centurion",
    "name": "Centurion",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (76, 80),
        "a7": (78, 83),
    },
    "damage": {
        "slash": {"base": 12, "a2": 14},
        "fury": {"base": 6, "a2": 7},
        "fury_hits": {"base": 3},
        "block": {"base": 15, "a17": 20},
    },
    "moves": {1: "Slash", 2: "Defend", 3: "Fury"},
    "move_ids": {"SLASH": 1, "PROTECT": 2, "FURY": 3},
    "passives": [],
}

HEALER_DATA = {
    "id": "Healer",
    "name": "Mystic",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (48, 56),
        "a7": (50, 58),
    },
    "damage": {
        "attack": {"base": 8, "a2": 9},
        "strength": {"base": 2, "a2": 3, "a17": 4},
        "heal": {"base": 16, "a17": 20},
    },
    "moves": {1: "Attack", 2: "Heal", 3: "Buff"},
    "move_ids": {"ATTACK": 1, "HEAL": 2, "BUFF": 3},
    "passives": [],
}

SNECKO_DATA = {
    "id": "Snecko",
    "name": "Snecko",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (114, 120),
        "a7": (120, 125),
    },
    "damage": {
        "bite": {"base": 15, "a2": 18},
        "tail": {"base": 8, "a2": 10},
    },
    "moves": {1: "Glare", 2: "Bite", 3: "Tail Whip"},
    "move_ids": {"GLARE": 1, "BITE": 2, "TAIL": 3},
    "passives": [],
}

SNAKE_PLANT_DATA = {
    "id": "SnakePlant",
    "name": "Snake Plant",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (75, 79),
        "a7": (78, 82),
    },
    "damage": {
        "chomp": {"base": 7, "a2": 8},
        "chomp_hits": {"base": 3},
    },
    "moves": {1: "Chomp", 2: "Spores"},
    "move_ids": {"CHOMP": 1, "SPORES": 2},
    "passives": ["malleable"],
}

SHELLED_PARASITE_DATA = {
    "id": "Shelled Parasite",
    "name": "Shelled Parasite",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (68, 72),
        "a7": (70, 75),
    },
    "damage": {
        "double_strike": {"base": 6, "a2": 7},
        "double_strike_hits": {"base": 2},
        "fell": {"base": 18, "a2": 21},
        "suck": {"base": 10, "a2": 12},
    },
    "moves": {1: "Double Strike", 2: "Fell", 3: "Stunned", 4: "Suck"},
    "move_ids": {"DOUBLE_STRIKE": 1, "FELL": 2, "STUNNED": 3, "SUCK": 4},
    "passives": ["plated_armor"],
    "plated_armor_amount": {"base": 14, "a7": 14, "a17": 19},
}

SPHERIC_GUARDIAN_DATA = {
    "id": "SphericGuardian",
    "name": "Spheric Guardian",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (20, 20),
        "a7": (20, 20),
    },
    "damage": {
        "slam": {"base": 10, "a2": 11},
        "activate": {"base": 10, "a17": 15},  # Block amount
    },
    "moves": {1: "Slam", 2: "Activate", 3: "Attack & Debuff", 4: "Harden"},
    "move_ids": {"SLAM": 1, "ACTIVATE": 2, "ATTACK_DEBUFF": 3, "HARDEN": 4},
    "passives": ["barricade", "artifact"],
    "artifact_amount": 3,
}

BANDIT_BEAR_DATA = {
    "id": "BanditBear",
    "name": "Bear",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (38, 42),
        "a7": (40, 44),
    },
    "damage": {
        "maul": {"base": 18, "a2": 20},
        "bear_hug": {"base": 8, "a2": 10},
        "lunge": {"base": 9, "a2": 10},
    },
    "moves": {1: "Maul", 2: "Bear Hug", 3: "Lunge"},
    "move_ids": {"MAUL": 1, "BEAR_HUG": 2, "LUNGE": 3},
    "passives": [],
}

BANDIT_LEADER_DATA = {
    "id": "BanditLeader",
    "name": "Romeo",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (62, 66),
        "a7": (64, 68),
    },
    "damage": {
        "mock": {"base": 0},  # Debuff only
        "agonizing_slash": {"base": 10, "a2": 12},
    },
    "moves": {1: "Mock", 2: "Agonizing Slash", 3: "Shiv"},
    "move_ids": {"MOCK": 1, "AGONIZING_SLASH": 2, "SHIV": 3},
    "passives": [],
}

BANDIT_POINTY_DATA = {
    "id": "BanditChild",
    "name": "Pointy",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (30, 34),
        "a7": (32, 36),
    },
    "damage": {
        "attack": {"base": 5, "a2": 6},
    },
    "moves": {1: "Attack"},
    "move_ids": {"ATTACK": 1},
    "passives": [],
}


# =============================================================================
# CITY (ACT 2) - ELITES
# =============================================================================

GREMLIN_LEADER_DATA = {
    "id": "GremlinLeader",
    "name": "Gremlin Leader",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (140, 148),
        "a8": (145, 155),
    },
    "damage": {
        "encourage_str": {"base": 3, "a3": 4, "a18": 5},
        "encourage_block": {"base": 6, "a18": 10},
        "stab": {"base": 6},
        "stab_hits": {"base": 3},
    },
    "moves": {1: "Encourage", 2: "Rally", 3: "Stab"},
    "move_ids": {"ENCOURAGE": 1, "RALLY": 2, "STAB": 3},
    "passives": [],
}

BOOK_OF_STABBING_DATA = {
    "id": "BookOfStabbing",
    "name": "Book of Stabbing",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (160, 164),
        "a8": (168, 172),
    },
    "damage": {
        "multi_stab": {"base": 6, "a3": 7},
        "single_stab": {"base": 21, "a3": 24},
    },
    "moves": {1: "Multi-Stab", 2: "Single Stab"},
    "move_ids": {"MULTI_STAB": 1, "SINGLE_STAB": 2},
    "passives": ["painful_stabs"],  # Add wound on unblocked damage
}

TASKMASTER_DATA = {
    "id": "SlaverBoss",
    "name": "Taskmaster",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (54, 60),
        "a8": (57, 64),
    },
    "damage": {
        "whip": {"base": 7},
        "wounds": {"base": 1, "a3": 2, "a18": 3},
    },
    "moves": {1: "Scouring Whip"},
    "move_ids": {"SCOURING_WHIP": 1},
    "passives": [],
    "gains_strength": {"a18": 1},  # A18+: gains 1 str per turn
}


# =============================================================================
# CITY (ACT 2) - BOSSES
# =============================================================================

CHAMP_DATA = {
    "id": "Champ",
    "name": "The Champ",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (420, 420),
        "a9": (440, 440),
    },
    "damage": {
        "slash": {"base": 16, "a4": 18},
        "execute": {"base": 10},
        "execute_hits": {"base": 2},
        "slap": {"base": 12, "a4": 14},
        "strength": {"base": 2, "a4": 3, "a19": 4},
        "anger_str": {"base": 6, "a4": 9, "a19": 12},
        "forge": {"base": 5, "a9": 6, "a19": 7},
        "block": {"base": 15, "a9": 18, "a19": 20},
    },
    "moves": {
        1: "Heavy Slash", 2: "Defensive Stance", 3: "Gloat",
        4: "Taunt", 5: "Execute", 6: "Face Slap", 7: "Anger"
    },
    "move_ids": {
        "HEAVY_SLASH": 1, "DEFENSIVE_STANCE": 2, "GLOAT": 3,
        "TAUNT": 4, "EXECUTE": 5, "FACE_SLAP": 6, "ANGER": 7
    },
    "passives": [],
}

THE_COLLECTOR_DATA = {
    "id": "TheCollector",
    "name": "The Collector",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (282, 282),
        "a9": (300, 300),
    },
    "damage": {
        "fireball": {"base": 18, "a4": 21},
        "strength": {"base": 3, "a4": 4, "a19": 5},
        "block": {"base": 15, "a9": 18},
        "mega_debuff": {"base": 3, "a19": 5},
    },
    "moves": {
        1: "Spawn", 2: "Fireball", 3: "Buff", 4: "Mega Debuff", 5: "Revive"
    },
    "move_ids": {
        "SPAWN": 1, "FIREBALL": 2, "BUFF": 3, "MEGA_DEBUFF": 4, "REVIVE": 5
    },
    "passives": [],
}

BRONZE_AUTOMATON_DATA = {
    "id": "BronzeAutomaton",
    "name": "Bronze Automaton",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (300, 300),
        "a9": (320, 320),
    },
    "damage": {
        "flail": {"base": 7, "a4": 8},
        "flail_hits": {"base": 2},
        "beam": {"base": 45, "a4": 50},
        "strength": {"base": 3, "a4": 4},
        "block": {"base": 9, "a9": 12},
    },
    "moves": {
        1: "Flail", 2: "Hyper Beam", 3: "Stunned", 4: "Spawn Orbs", 5: "Boost"
    },
    "move_ids": {
        "FLAIL": 1, "HYPER_BEAM": 2, "STUNNED": 3, "SPAWN_ORBS": 4, "BOOST": 5
    },
    "passives": ["artifact"],
    "artifact_amount": 3,
}


# =============================================================================
# BEYOND (ACT 3) - BASIC ENEMIES
# =============================================================================

MAW_DATA = {
    "id": "Maw",
    "name": "Maw",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (300, 300),
        "a7": (300, 300),
    },
    "damage": {
        "roar": {"base": 0},  # Debuff only
        "drool": {"base": 0},  # Status cards
        "slam": {"base": 25, "a2": 30},
        "nom": {"base": 5, "a2": 5},  # NOM_DMG (healing attack)
    },
    "moves": {1: "Roar", 2: "Drool", 3: "Slam", 4: "Nom"},
    "move_ids": {"ROAR": 1, "DROOL": 2, "SLAM": 3, "NOM": 4},
    "passives": [],
}

DARKLING_DATA = {
    "id": "Darkling",
    "name": "Darkling",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (48, 56),
        "a7": (50, 59),
    },
    "damage": {
        "chomp": {"base": 8, "a2": 9},
        "chomp_hits": {"base": 2},
        "nip_min": {"base": 7, "a2": 9},
        "nip_max": {"base": 11, "a2": 13},
        "harden_block": {"base": 12},
        "harden_str": {"a17": 2},  # Only at A17
    },
    "moves": {1: "Chomp", 2: "Harden", 3: "Nip", 4: "Reincarnate"},
    "move_ids": {"CHOMP": 1, "HARDEN": 2, "NIP": 3, "REINCARNATE": 4},
    "passives": ["regrow"],
}

ORB_WALKER_DATA = {
    "id": "Orb Walker",
    "name": "Orb Walker",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (90, 96),
        "a7": (92, 98),
    },
    "damage": {
        "claw": {"base": 15, "a2": 16},
        "laser": {"base": 10, "a2": 11},
        "burn": {"base": 1, "a17": 2},  # Number of burns
    },
    "moves": {1: "Claw", 2: "Laser"},
    "move_ids": {"CLAW": 1, "LASER": 2},
    "passives": [],
}

SPIKER_DATA = {
    "id": "Spiker",
    "name": "Spiker",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (42, 56),
        "a7": (44, 58),
    },
    "damage": {
        "cut": {"base": 7, "a2": 9},
        "spike": {"base": 2, "a17": 3},  # Thorns amount
    },
    "moves": {1: "Cut", 2: "Spike"},
    "move_ids": {"CUT": 1, "SPIKE": 2},
    "passives": ["thorns"],
    "thorns_amount": {"base": 3, "a17": 4},
}

REPULSOR_DATA = {
    "id": "Repulsor",
    "name": "Repulsor",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (29, 35),
        "a7": (31, 37),
    },
    "damage": {
        "bash": {"base": 11, "a2": 13},
        "dazed": {"base": 2, "a17": 3},
    },
    "moves": {1: "Bash", 2: "Repulse"},
    "move_ids": {"BASH": 1, "REPULSE": 2},
    "passives": [],
}

WRITHING_MASS_DATA = {
    "id": "WrithingMass",
    "name": "Writhing Mass",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (160, 160),
        "a7": (175, 175),
    },
    "damage": {
        "strong_hit": {"base": 32, "a2": 38},
        "multi_strike": {"base": 7, "a2": 9},
        "multi_strike_hits": {"base": 3},
        "flail": {"base": 15, "a2": 16},
        "wither": {"base": 10, "a2": 12},
    },
    "moves": {1: "Strong Hit", 2: "Multi-Strike", 3: "Flail", 4: "Wither", 5: "Implant"},
    "move_ids": {
        "STRONG_HIT": 1, "MULTI_STRIKE": 2, "FLAIL": 3, "WITHER": 4, "IMPLANT": 5
    },
    "passives": ["reactive", "malleable"],
}

TRANSIENT_DATA = {
    "id": "Transient",
    "name": "Transient",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (999, 999),
    },
    "damage": {
        "attack": {"base": 30, "a2": 40},
        "attack_increment": {"base": 10},  # +10 each turn
    },
    "moves": {1: "Attack"},
    "move_ids": {"ATTACK": 1},
    "passives": ["fading"],  # Dies after 5 turns, escapes if low HP
    "fading_turns": 5,
}

EXPLODER_DATA = {
    "id": "Exploder",
    "name": "Exploder",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (30, 35),
        "a7": (32, 38),
    },
    "damage": {
        "slam": {"base": 9, "a2": 11},
        "explode": {"base": 30},
    },
    "moves": {1: "Slam", 2: "Explode"},
    "move_ids": {"SLAM": 1, "EXPLODE": 2},
    "passives": [],
}

SPIRE_GROWTH_DATA = {
    "id": "Serpent",
    "name": "Spire Growth",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (170, 190),
        "a7": (178, 198),
    },
    "damage": {
        "quick_tackle": {"base": 16, "a2": 18},
        "smash": {"base": 22, "a2": 25},
        "constrict": {"base": 10, "a2": 12},
    },
    "moves": {1: "Quick Tackle", 2: "Smash", 3: "Constrict"},
    "move_ids": {"QUICK_TACKLE": 1, "SMASH": 2, "CONSTRICT": 3},
    "passives": [],
}

SNAKE_DAGGER_DATA = {
    "id": "Dagger",
    "name": "Snake Dagger",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (20, 25),
        "a8": (22, 27),
    },
    "damage": {
        "stab": {"base": 9, "a3": 10},
        "explode": {"base": 25},
    },
    "moves": {1: "Stab", 2: "Explode"},
    "move_ids": {"STAB": 1, "EXPLODE": 2},
    "passives": ["minion"],
}


# =============================================================================
# BEYOND (ACT 3) - ELITES
# =============================================================================

GIANT_HEAD_DATA = {
    "id": "GiantHead",
    "name": "Giant Head",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (500, 500),
        "a8": (520, 520),
    },
    "damage": {
        "count": {"base": 13},
        "it_is_time": {"base": 30, "a3": 40},
        "it_is_time_increment": {"base": 5},
    },
    "moves": {1: "Count", 2: "Glare", 3: "It Is Time"},
    "move_ids": {"COUNT": 1, "GLARE": 2, "IT_IS_TIME": 3},
    "passives": ["slow"],  # First card played each turn costs +1
    "countdown": {"base": 5, "a18": 4},
}

NEMESIS_DATA = {
    "id": "Nemesis",
    "name": "Nemesis",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (185, 185),
        "a8": (200, 200),
    },
    "damage": {
        "scythe": {"base": 45},
        "fire": {"base": 6, "a3": 7},
        "fire_hits": {"base": 3},
        "burn": {"base": 3, "a18": 5},  # Number of burns
    },
    "moves": {1: "Scythe", 2: "Tri Attack", 3: "Burn"},
    "move_ids": {"SCYTHE": 1, "TRI_ATTACK": 2, "TRI_BURN": 3},
    "passives": ["intangible"],  # Gains intangible at end of turn
}

REPTOMANCER_DATA = {
    "id": "Reptomancer",
    "name": "Reptomancer",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (180, 190),
        "a8": (190, 200),
    },
    "damage": {
        "snake_strike": {"base": 13, "a3": 16},
        "snake_strike_hits": {"base": 2},
        "big_bite": {"base": 30, "a3": 34},
    },
    "moves": {1: "Snake Strike", 2: "Summon", 3: "Big Bite"},
    "move_ids": {"SNAKE_STRIKE": 1, "SPAWN_DAGGER": 2, "BIG_BITE": 3},
    "passives": [],
    "daggers_per_spawn": {"base": 1, "a18": 2},
}


# =============================================================================
# BEYOND (ACT 3) - BOSSES
# =============================================================================

AWAKENED_ONE_DATA = {
    "id": "AwakenedOne",
    "name": "Awakened One",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (300, 300),
        "a9": (320, 320),
    },
    "damage": {
        # Phase 1
        "slash": {"base": 20},
        "soul_strike": {"base": 6},
        "soul_strike_hits": {"base": 4},
        # Phase 2
        "dark_echo": {"base": 40},
        "sludge": {"base": 18},
        "tackle": {"base": 10},
        "tackle_hits": {"base": 3},
    },
    "moves": {
        1: "Slash", 2: "Soul Strike", 3: "Rebirth",
        5: "Dark Echo", 6: "Sludge", 8: "Tackle"
    },
    "move_ids": {
        "SLASH": 1, "SOUL_STRIKE": 2, "REBIRTH": 3,
        "DARK_ECHO": 5, "SLUDGE": 6, "TACKLE": 8
    },
    "passives": ["curiosity", "regenerate"],
    "curiosity_str": {"base": 1, "a19": 2},
    "regenerate_hp": {"base": 10, "a19": 15},
    "starting_strength": {"a4": 2},
}

TIME_EATER_DATA = {
    "id": "TimeEater",
    "name": "Time Eater",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (456, 456),
        "a9": (480, 480),
    },
    "damage": {
        "reverberate": {"base": 7, "a4": 8},
        "reverberate_hits": {"base": 3},
        "head_slam": {"base": 26, "a4": 32},
        "ripple_block": {"base": 20},
    },
    "moves": {1: "Reverberate", 2: "Head Slam", 3: "Ripple", 4: "Haste"},
    "move_ids": {"REVERBERATE": 1, "HEAD_SLAM": 2, "RIPPLE": 3, "HASTE": 4},
    "passives": ["time_warp"],  # Ends turn after 12 cards, gains 2 str
    "cards_until_warp": 12,
}

DONU_DATA = {
    "id": "Donu",
    "name": "Donu",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (250, 250),
        "a9": (265, 265),
    },
    "damage": {
        "beam": {"base": 10, "a4": 12},
        "beam_hits": {"base": 2},
        "circle_str": {"base": 3},
    },
    "moves": {1: "Circle of Power", 2: "Beam"},
    "move_ids": {"CIRCLE": 1, "BEAM": 2},
    "passives": ["artifact"],
    "artifact_amount": {"base": 2, "a19": 3},
}

DECA_DATA = {
    "id": "Deca",
    "name": "Deca",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (250, 250),
        "a9": (265, 265),
    },
    "damage": {
        "beam": {"base": 10, "a4": 12},
        "beam_hits": {"base": 2},
        "square_block": {"base": 16},
        "plated_armor": {"a19": 3},  # Only at A19
    },
    "moves": {1: "Square of Protection", 2: "Beam"},
    "move_ids": {"SQUARE": 1, "BEAM": 2},
    "passives": ["artifact"],
    "artifact_amount": {"base": 2, "a19": 3},
}


# =============================================================================
# ACT 4 - ENDING
# =============================================================================

SPIRE_SHIELD_DATA = {
    "id": "SpireShield",
    "name": "Spire Shield",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (110, 110),
        "a9": (120, 120),
    },
    "damage": {
        "bash": {"base": 12, "a4": 14},
        "fortify_block": {"base": 30},
        "smash": {"base": 34, "a4": 38},
    },
    "moves": {1: "Bash", 2: "Fortify", 3: "Smash"},
    "move_ids": {"BASH": 1, "FORTIFY": 2, "SMASH": 3},
    "passives": ["artifact"],
    "artifact_amount": 1,
}

SPIRE_SPEAR_DATA = {
    "id": "SpireSpear",
    "name": "Spire Spear",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (160, 160),
        "a9": (180, 180),
    },
    "damage": {
        "burn_strike": {"base": 5, "a4": 6},
        "burn_strike_hits": {"base": 2},
        "skewer": {"base": 10, "a4": 11},
        "skewer_hits": {"base": 3},
    },
    "moves": {1: "Burn Strike", 2: "Piercer", 3: "Skewer"},
    "move_ids": {"BURN_STRIKE": 1, "PIERCER": 2, "SKEWER": 3},
    "passives": ["artifact"],
    "artifact_amount": 1,
}

CORRUPT_HEART_DATA = {
    "id": "CorruptHeart",
    "name": "Corrupt Heart",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (750, 750),
        "a9": (800, 800),
    },
    "damage": {
        "blood_shots": {"base": 2},
        "blood_shots_hits": {"base": 12, "a4": 15},
        "echo": {"base": 40, "a4": 45},
        "buff_str": {"base": 2},
    },
    "moves": {1: "Debilitate", 2: "Blood Shots", 3: "Echo", 4: "Buff"},
    "move_ids": {"DEBILITATE": 1, "BLOOD_SHOTS": 2, "ECHO": 3, "BUFF": 4},
    "passives": ["invincible", "beat_of_death"],
    "invincible_threshold": {"base": 300, "a19": 200},
    "beat_of_death": {"base": 1, "a19": 2},
}


# =============================================================================
# MINIONS / SUMMONS
# =============================================================================

TORCH_HEAD_DATA = {
    "id": "TorchHead",
    "name": "Torch Head",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (38, 40),
        "a9": (40, 44),
    },
    "damage": {
        "tackle": {"base": 7},
    },
    "moves": {1: "Tackle"},
    "move_ids": {"TACKLE": 1},
    "passives": ["minion"],
}

BRONZE_ORB_DATA = {
    "id": "BronzeOrb",
    "name": "Bronze Orb",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (52, 58),
        "a9": (54, 60),
    },
    "damage": {
        "beam": {"base": 8},
    },
    "moves": {1: "Stasis", 2: "Beam", 3: "Support Beam"},
    "move_ids": {"STASIS": 1, "BEAM": 2, "SUPPORT_BEAM": 3},
    "passives": ["minion"],
}

GREMLIN_FAT_DATA = {
    "id": "GremlinFat",
    "name": "Fat Gremlin",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (13, 17),
        "a8": (14, 18),
    },
    "damage": {
        "smash": {"base": 4, "a3": 5},
        "weak": {"base": 1, "a18": 2},
    },
    "moves": {1: "Smash"},
    "move_ids": {"SMASH": 1},
    "passives": [],
}

GREMLIN_THIEF_DATA = {
    "id": "GremlinThief",
    "name": "Sneaky Gremlin",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (10, 14),
        "a8": (11, 15),
    },
    "damage": {
        "puncture": {"base": 9, "a3": 10},
    },
    "moves": {1: "Puncture"},
    "move_ids": {"PUNCTURE": 1},
    "passives": [],
}

GREMLIN_TSUNDERE_DATA = {
    "id": "GremlinTsundere",
    "name": "Shield Gremlin",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (12, 15),
        "a8": (13, 16),
    },
    "damage": {
        "protect": {"base": 7, "a18": 11},  # Block amount
        "shield_bash": {"base": 6, "a3": 8},
    },
    "moves": {1: "Protect", 2: "Shield Bash"},
    "move_ids": {"PROTECT": 1, "SHIELD_BASH": 2},
    "passives": [],
}

GREMLIN_WARRIOR_DATA = {
    "id": "GremlinWarrior",
    "name": "Mad Gremlin",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (20, 24),
        "a8": (21, 25),
    },
    "damage": {
        "scratch": {"base": 4, "a3": 5},
    },
    "moves": {1: "Scratch"},
    "move_ids": {"SCRATCH": 1},
    "passives": ["angry"],  # Gains 1 str when damaged
    "angry_str": {"base": 1, "a18": 2},
}

GREMLIN_WIZARD_DATA = {
    "id": "GremlinWizard",
    "name": "Gremlin Wizard",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (22, 26),
        "a8": (23, 27),
    },
    "damage": {
        "ultimate_blast": {"base": 25},
    },
    "moves": {1: "Charging", 2: "Ultimate Blast"},
    "move_ids": {"CHARGING": 1, "ULTIMATE_BLAST": 2},
    "passives": [],
    "charge_turns": 3,
}


# =============================================================================
# ENEMY DATA REGISTRY
# =============================================================================

ENEMY_DATA: Dict[str, Dict[str, Any]] = {
    # Exordium Basic
    "JawWorm": JAW_WORM_DATA,
    "Cultist": CULTIST_DATA,
    "AcidSlime_M": ACID_SLIME_M_DATA,
    "AcidSlime_L": ACID_SLIME_L_DATA,
    "AcidSlime_S": ACID_SLIME_S_DATA,
    "SpikeSlime_M": SPIKE_SLIME_M_DATA,
    "SpikeSlime_L": SPIKE_SLIME_L_DATA,
    "SpikeSlime_S": SPIKE_SLIME_S_DATA,
    "Louse": LOUSE_DATA,
    "FuzzyLouseNormal": LOUSE_NORMAL_DATA,
    "FuzzyLouseDefensive": LOUSE_DEFENSIVE_DATA,
    "FungiBeast": FUNGI_BEAST_DATA,
    "Looter": LOOTER_DATA,
    "Mugger": MUGGER_DATA,
    "SlaverBlue": SLAVER_BLUE_DATA,
    "SlaverRed": SLAVER_RED_DATA,
    # Exordium Elites
    "GremlinNob": GREMLIN_NOB_DATA,
    "Lagavulin": LAGAVULIN_DATA,
    "Sentry": SENTRY_DATA,
    # Exordium Bosses
    "SlimeBoss": SLIME_BOSS_DATA,
    "TheGuardian": THE_GUARDIAN_DATA,
    "Hexaghost": HEXAGHOST_DATA,
    # City Basic
    "Chosen": CHOSEN_DATA,
    "Byrd": BYRD_DATA,
    "Centurion": CENTURION_DATA,
    "Healer": HEALER_DATA,
    "Snecko": SNECKO_DATA,
    "SnakePlant": SNAKE_PLANT_DATA,
    "Shelled Parasite": SHELLED_PARASITE_DATA,
    "SphericGuardian": SPHERIC_GUARDIAN_DATA,
    "BanditBear": BANDIT_BEAR_DATA,
    "BanditLeader": BANDIT_LEADER_DATA,
    "BanditChild": BANDIT_POINTY_DATA,
    # City Elites
    "GremlinLeader": GREMLIN_LEADER_DATA,
    "BookOfStabbing": BOOK_OF_STABBING_DATA,
    "SlaverBoss": TASKMASTER_DATA,
    "Taskmaster": TASKMASTER_DATA,
    # City Bosses
    "Champ": CHAMP_DATA,
    "TheCollector": THE_COLLECTOR_DATA,
    "BronzeAutomaton": BRONZE_AUTOMATON_DATA,
    # Beyond Basic
    "Maw": MAW_DATA,
    "Darkling": DARKLING_DATA,
    "Orb Walker": ORB_WALKER_DATA,
    "Spiker": SPIKER_DATA,
    "Repulsor": REPULSOR_DATA,
    "WrithingMass": WRITHING_MASS_DATA,
    "Transient": TRANSIENT_DATA,
    "Exploder": EXPLODER_DATA,
    "Serpent": SPIRE_GROWTH_DATA,
    "Dagger": SNAKE_DAGGER_DATA,
    # Beyond Elites
    "GiantHead": GIANT_HEAD_DATA,
    "Nemesis": NEMESIS_DATA,
    "Reptomancer": REPTOMANCER_DATA,
    # Beyond Bosses
    "AwakenedOne": AWAKENED_ONE_DATA,
    "TimeEater": TIME_EATER_DATA,
    "Donu": DONU_DATA,
    "Deca": DECA_DATA,
    # Act 4
    "SpireShield": SPIRE_SHIELD_DATA,
    "SpireSpear": SPIRE_SPEAR_DATA,
    "CorruptHeart": CORRUPT_HEART_DATA,
    # Minions
    "TorchHead": TORCH_HEAD_DATA,
    "BronzeOrb": BRONZE_ORB_DATA,
    "GremlinFat": GREMLIN_FAT_DATA,
    "GremlinThief": GREMLIN_THIEF_DATA,
    "GremlinTsundere": GREMLIN_TSUNDERE_DATA,
    "GremlinWarrior": GREMLIN_WARRIOR_DATA,
    "GremlinWizard": GREMLIN_WIZARD_DATA,
}


# =============================================================================
# HELPER FUNCTIONS
# =============================================================================

def get_hp_range(enemy_id: str, ascension: int = 0) -> Tuple[int, int]:
    """Get HP range for an enemy at given ascension level."""
    data = ENEMY_DATA.get(enemy_id)
    if not data:
        return (1, 1)

    hp_data = data.get("hp", {})
    result = hp_data.get("base", (1, 1))

    # Check ascension thresholds (sorted numerically, not alphabetically)
    asc_keys = [k for k in hp_data.keys() if k.startswith("a")]
    for key in sorted(asc_keys, key=lambda x: int(x[1:])):
        threshold = int(key[1:])
        if ascension >= threshold:
            result = hp_data[key]

    return result


def get_damage_value(enemy_id: str, move_key: str, ascension: int = 0) -> int:
    """Get damage value for an enemy move at given ascension level."""
    data = ENEMY_DATA.get(enemy_id)
    if not data:
        return 0

    damage_data = data.get("damage", {}).get(move_key, {})
    if isinstance(damage_data, int):
        return damage_data

    result = damage_data.get("base", 0)

    # Check ascension thresholds (sorted numerically, not alphabetically)
    asc_keys = [k for k in damage_data.keys() if k.startswith("a")]
    for key in sorted(asc_keys, key=lambda x: int(x[1:])):
        threshold = int(key[1:])
        if ascension >= threshold:
            result = damage_data[key]

    return result


def get_damage_values(enemy_id: str, ascension: int = 0) -> Dict[str, int]:
    """Get all damage values for an enemy at given ascension level."""
    data = ENEMY_DATA.get(enemy_id)
    if not data:
        return {}

    result = {}
    for move_key in data.get("damage", {}).keys():
        result[move_key] = get_damage_value(enemy_id, move_key, ascension)

    return result


def get_enemy_type(enemy_id: str) -> EnemyType:
    """Get the type of an enemy."""
    data = ENEMY_DATA.get(enemy_id)
    if data:
        return data.get("type", EnemyType.NORMAL)
    return EnemyType.NORMAL


def get_move_name(enemy_id: str, move_id: int) -> str:
    """Get the name of a move by ID."""
    data = ENEMY_DATA.get(enemy_id)
    if data:
        return data.get("moves", {}).get(move_id, "Unknown")
    return "Unknown"
