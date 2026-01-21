"""
Tool definitions for VOD extraction.

These tools are called by the LLM (Gemini 2.0 Flash) to record
game state changes observed in the video.
"""

from typing import Any, Optional

# Tool definitions in OpenAI function calling format
TOOLS: list[dict[str, Any]] = [
    {
        "name": "set_seed",
        "description": "Record the game seed when visible on screen. Look for the seed in settings or at run start.",
        "parameters": {
            "type": "object",
            "properties": {
                "seed": {
                    "type": "string",
                    "description": "The seed string (e.g., 'ABC123DEF')"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp (MM:SS or HH:MM:SS)"
                }
            },
            "required": ["seed"]
        }
    },
    {
        "name": "neow",
        "description": "Record Neow bonus selection at run start. Called once per run.",
        "parameters": {
            "type": "object",
            "properties": {
                "chosen": {
                    "type": "string",
                    "description": "The bonus chosen (e.g., 'Choose a Rare Card', 'Max HP +8')"
                },
                "drawback": {
                    "type": "string",
                    "description": "The drawback if applicable (e.g., 'Lose all Gold')"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["chosen"]
        }
    },
    {
        "name": "path",
        "description": "Record map path choice - which node the player selected from available options.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number (1-56)"
                },
                "chosen": {
                    "type": "string",
                    "enum": ["monster", "elite", "rest", "shop", "?", "chest", "boss"],
                    "description": "Room type chosen"
                },
                "options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Available room types at this floor"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["floor", "chosen"]
        }
    },
    {
        "name": "combat_start",
        "description": "Mark the start of a combat encounter.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "enemy": {
                    "type": "string",
                    "description": "Enemy name(s) (e.g., 'Jaw Worm', 'Gremlin Gang', 'Lagavulin')"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp when combat begins"
                }
            },
            "required": ["floor", "enemy"]
        }
    },
    {
        "name": "combat_turn",
        "description": "Record cards played during a combat turn. Call once per turn.",
        "parameters": {
            "type": "object",
            "properties": {
                "turn": {
                    "type": "integer",
                    "description": "Turn number (1, 2, 3, ...)"
                },
                "cards": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Cards played this turn in order (e.g., ['Eruption', 'Vigilance', 'Strike'])"
                },
                "potions_used": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Potions used this turn"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["turn", "cards"]
        }
    },
    {
        "name": "combat_end",
        "description": "Mark the end of combat with final HP.",
        "parameters": {
            "type": "object",
            "properties": {
                "hp": {
                    "type": "integer",
                    "description": "HP remaining after combat"
                },
                "gold_gained": {
                    "type": "integer",
                    "description": "Gold gained from combat"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp when combat ends"
                }
            },
            "required": ["hp"]
        }
    },
    {
        "name": "card_reward",
        "description": "Record card reward selection after combat.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Card options offered (e.g., ['Tantrum', 'Crescendo', 'Evaluate'])"
                },
                "chosen": {
                    "type": "string",
                    "description": "Card picked or 'SKIP' if skipped"
                },
                "singing_bowl": {
                    "type": "boolean",
                    "description": "True if Singing Bowl +2 Max HP was taken instead"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["floor", "options", "chosen"]
        }
    },
    {
        "name": "relic_reward",
        "description": "Record a relic obtained (from combat, chest, or event).",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "relic": {
                    "type": "string",
                    "description": "Relic name (e.g., 'Vajra', 'Bag of Preparation')"
                },
                "source": {
                    "type": "string",
                    "enum": ["combat", "elite", "boss", "chest", "event", "shop"],
                    "description": "Where the relic came from"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["floor", "relic"]
        }
    },
    {
        "name": "potion_reward",
        "description": "Record a potion obtained.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "potion": {
                    "type": "string",
                    "description": "Potion name (e.g., 'Block Potion', 'Fairy in a Bottle')"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["floor", "potion"]
        }
    },
    {
        "name": "shop",
        "description": "Record all shop transactions at once.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "cards_bought": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Cards purchased"
                },
                "cards_removed": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Cards removed (usually Strikes/Defends)"
                },
                "relics_bought": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Relics purchased"
                },
                "potions_bought": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Potions purchased"
                },
                "gold_spent": {
                    "type": "integer",
                    "description": "Total gold spent"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["floor"]
        }
    },
    {
        "name": "rest",
        "description": "Record rest site decision.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "action": {
                    "type": "string",
                    "enum": ["rest", "smith", "lift", "toke", "dig", "recall", "ruby_key"],
                    "description": "Action taken (rest=heal, smith=upgrade, lift=Girya, toke=PeacePipe, dig=Shovel, recall=DreamCatcher, ruby_key=take key)"
                },
                "card_upgraded": {
                    "type": "string",
                    "description": "If smith, which card was upgraded"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["floor", "action"]
        }
    },
    {
        "name": "boss_relic",
        "description": "Record boss relic selection after defeating act boss.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number (16, 33, or 50)"
                },
                "options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Boss relics offered"
                },
                "chosen": {
                    "type": "string",
                    "description": "Relic chosen"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["floor", "options", "chosen"]
        }
    },
    {
        "name": "event",
        "description": "Record an event encounter and choice made.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "name": {
                    "type": "string",
                    "description": "Event name (e.g., 'The Library', 'Big Fish', 'Golden Idol')"
                },
                "choice": {
                    "type": "string",
                    "description": "Choice made at the event"
                },
                "outcome": {
                    "type": "string",
                    "description": "What happened as a result"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp"
                }
            },
            "required": ["floor", "name", "choice"]
        }
    },
    {
        "name": "result",
        "description": "Record final run result.",
        "parameters": {
            "type": "object",
            "properties": {
                "victory": {
                    "type": "boolean",
                    "description": "True if run was won"
                },
                "floor": {
                    "type": "integer",
                    "description": "Final floor reached"
                },
                "heart_kill": {
                    "type": "boolean",
                    "description": "True if Heart was defeated (floor 56)"
                },
                "final_hp": {
                    "type": "integer",
                    "description": "HP at end of run"
                },
                "timestamp": {
                    "type": "string",
                    "description": "Video timestamp of run end"
                }
            },
            "required": ["victory", "floor"]
        }
    }
]


def get_tool_definitions() -> list[dict[str, Any]]:
    """Get all tool definitions for the LLM."""
    return TOOLS


def get_tool_by_name(name: str) -> Optional[dict[str, Any]]:
    """Get a specific tool definition by name."""
    for tool in TOOLS:
        if tool["name"] == name:
            return tool
    return None


def get_tool_names() -> list[str]:
    """Get list of all tool names."""
    return [tool["name"] for tool in TOOLS]


# Gemini-specific format conversion
def to_gemini_tools() -> list[dict]:
    """Convert tools to Gemini's function calling format."""
    return [
        {
            "function_declarations": [
                {
                    "name": tool["name"],
                    "description": tool["description"],
                    "parameters": tool["parameters"]
                }
                for tool in TOOLS
            ]
        }
    ]


# Tool categories for documentation
TOOL_CATEGORIES = {
    "setup": ["set_seed", "neow"],
    "navigation": ["path"],
    "combat": ["combat_start", "combat_turn", "combat_end"],
    "rewards": ["card_reward", "relic_reward", "potion_reward", "boss_relic"],
    "rooms": ["shop", "rest", "event"],
    "meta": ["result"],
}


def get_tools_by_category(category: str) -> list[dict[str, Any]]:
    """Get tools in a specific category."""
    if category not in TOOL_CATEGORIES:
        return []
    names = TOOL_CATEGORIES[category]
    return [t for t in TOOLS if t["name"] in names]
