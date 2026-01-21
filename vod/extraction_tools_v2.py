"""
VOD Extraction Tools V2 - Index-Only Selection

Key design principle: The model NEVER invents content.
- For choices WITH predictions: model reports INDEX only
- For verification: model matches observed content TO our predictions
- Mismatches are flagged for review

This prevents hallucination issues like "Rapture" (a non-existent card).
"""

from typing import List, Dict, Any


# ============================================================================
# TOOLS THAT REQUIRE INDEX-ONLY SELECTION (when predictions available)
# ============================================================================

CARD_PICK_TOOL = {
    "name": "card_pick",
    "description": """Record which card was picked from the reward.

IMPORTANT: When predictions are provided, you MUST use chosen_index to indicate
which card was selected. Do NOT invent card names.

If the cards shown don't match our predictions AT ALL, call prediction_mismatch instead.""",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number"
            },
            "chosen_index": {
                "type": "integer",
                "description": "Index of chosen card from predictions (0, 1, 2, ...) or -1 for SKIP"
            },
            "skipped": {
                "type": "boolean",
                "description": "True if player skipped the card reward"
            }
        },
        "required": ["floor", "chosen_index"]
    }
}

BOSS_RELIC_PICK_TOOL = {
    "name": "boss_relic_pick",
    "description": """Record which boss relic was chosen.

IMPORTANT: When predictions are provided, use chosen_index (0, 1, or 2).""",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number"
            },
            "chosen_index": {
                "type": "integer",
                "description": "Index of chosen relic from predictions (0, 1, or 2)"
            }
        },
        "required": ["floor", "chosen_index"]
    }
}


# ============================================================================
# TOOLS FOR CONTENT THE MODEL MUST IDENTIFY (no predictions available)
# ============================================================================

RUN_START_TOOL = {
    "name": "run_start",
    "description": "Detect a new run starting. Extract the seed if visible.",
    "parameters": {
        "type": "object",
        "properties": {
            "seed": {
                "type": "string",
                "description": "The seed string if visible (e.g., '1V2ZJKI0'). Seeds are alphanumeric."
            },
            "character": {
                "type": "string",
                "enum": ["WATCHER", "IRONCLAD", "SILENT", "DEFECT"],
                "description": "Character being played (look at starting cards/HP)"
            },
            "ascension": {
                "type": "integer",
                "description": "Ascension level (0-20), shown as 'A20' etc."
            },
            "timestamp": {
                "type": "string",
                "description": "Video timestamp (MM:SS)"
            }
        },
        "required": ["character"]
    }
}

NEOW_CHOICE_TOOL = {
    "name": "neow_choice",
    "description": """Record the Neow bonus chosen at the start of the run.

Neow options are one of these categories:
- HP bonus/penalty
- Gold bonus (e.g., 'Obtain 100 Gold')
- Card reward (e.g., 'Choose a Rare card')
- Relic (e.g., 'Random Common Relic')
- Boss relic swap
- Remove/Transform/Upgrade cards

Report the TEXT shown for the bonus, not your interpretation.""",
    "parameters": {
        "type": "object",
        "properties": {
            "chosen_bonus_text": {
                "type": "string",
                "description": "Exact text of the chosen bonus (e.g., 'Obtain 100 Gold')"
            },
            "drawback_text": {
                "type": "string",
                "description": "Exact text of any drawback (e.g., 'Take 7 damage')"
            },
            "is_boss_swap": {
                "type": "boolean",
                "description": "True if chose boss relic swap"
            },
            "boss_relic_name": {
                "type": "string",
                "description": "Name of boss relic if boss swap was chosen"
            }
        },
        "required": ["chosen_bonus_text"]
    }
}

PATH_CHOICE_TOOL = {
    "name": "path_choice",
    "description": """Record the map path choice.

Report the ROOM TYPE chosen, not the specific encounter.""",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number after this move (1-56)"
            },
            "room_type": {
                "type": "string",
                "enum": ["monster", "elite", "rest", "shop", "event", "chest", "boss"],
                "description": "Type of room entered"
            }
        },
        "required": ["floor", "room_type"]
    }
}

REST_ACTION_TOOL = {
    "name": "rest_action",
    "description": "Record rest site action.",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number"
            },
            "action": {
                "type": "string",
                "enum": ["rest", "smith", "dig", "recall", "toke", "lift"],
                "description": "Action taken"
            },
            "card_upgraded_index": {
                "type": "integer",
                "description": "If smithing and predictions provided, index of upgraded card"
            }
        },
        "required": ["floor", "action"]
    }
}

EVENT_CHOICE_TOOL = {
    "name": "event_choice",
    "description": """Record event choice.

Report the event name and which option was selected.""",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number"
            },
            "event_name": {
                "type": "string",
                "description": "Name of the event (e.g., 'The Cleric', 'Golden Idol')"
            },
            "option_index": {
                "type": "integer",
                "description": "Index of option chosen (0, 1, 2, ...)"
            },
            "option_text": {
                "type": "string",
                "description": "Text of the chosen option"
            }
        },
        "required": ["floor", "event_name", "option_index"]
    }
}

SHOP_ACTION_TOOL = {
    "name": "shop_action",
    "description": """Record shop actions.

When predictions are provided, use indices for purchases.
When no predictions, describe what was bought.""",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number"
            },
            "card_indices_bought": {
                "type": "array",
                "items": {"type": "integer"},
                "description": "Indices of cards bought from predictions"
            },
            "relic_indices_bought": {
                "type": "array",
                "items": {"type": "integer"},
                "description": "Indices of relics bought from predictions"
            },
            "potion_indices_bought": {
                "type": "array",
                "items": {"type": "integer"},
                "description": "Indices of potions bought from predictions"
            },
            "removed_card": {
                "type": "boolean",
                "description": "True if card removal was purchased"
            },
            "nothing_purchased": {
                "type": "boolean",
                "description": "True if player left without buying anything"
            }
        },
        "required": ["floor"]
    }
}

COMBAT_END_TOOL = {
    "name": "combat_end",
    "description": "Record end of combat.",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number"
            },
            "hp_after": {
                "type": "integer",
                "description": "HP remaining after combat"
            },
            "gold_gained": {
                "type": "integer",
                "description": "Gold gained from combat"
            },
            "potion_dropped": {
                "type": "boolean",
                "description": "True if a potion dropped"
            }
        },
        "required": ["floor", "hp_after"]
    }
}

RUN_END_TOOL = {
    "name": "run_end",
    "description": "Record end of run.",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Final floor"
            },
            "victory": {
                "type": "boolean",
                "description": "True if run was won"
            },
            "heart_kill": {
                "type": "boolean",
                "description": "True if heart was killed"
            }
        },
        "required": ["floor", "victory"]
    }
}


# ============================================================================
# VERIFICATION TOOLS - For comparing predictions to reality
# ============================================================================

VERIFY_CARD_REWARD_TOOL = {
    "name": "verify_card_reward",
    "description": """Compare predicted card options to what's shown in the video.

Use this to validate our seed predictions. For each prediction, report if it matches what you see.""",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number"
            },
            "matches": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "predicted_index": {"type": "integer"},
                        "matches_video": {"type": "boolean"},
                        "video_shows": {"type": "string", "description": "What the video actually shows at this position"}
                    }
                },
                "description": "For each predicted card, whether it matches the video"
            },
            "all_match": {
                "type": "boolean",
                "description": "True if ALL predicted cards match the video"
            }
        },
        "required": ["floor", "matches", "all_match"]
    }
}

PREDICTION_MISMATCH_TOOL = {
    "name": "prediction_mismatch",
    "description": """Report when our seed prediction doesn't match what's in the video.

This helps us fix bugs in our game engine. Be specific about what differs.""",
    "parameters": {
        "type": "object",
        "properties": {
            "floor": {
                "type": "integer",
                "description": "Floor number"
            },
            "mismatch_type": {
                "type": "string",
                "enum": ["card_reward", "enemy", "shop_items", "relic", "potion", "event", "other"],
                "description": "Type of mismatch"
            },
            "prediction_index": {
                "type": "integer",
                "description": "Which predicted item doesn't match (0-indexed)"
            },
            "we_predicted": {
                "type": "string",
                "description": "What our prediction said"
            },
            "video_shows": {
                "type": "string",
                "description": "What the video actually shows"
            },
            "notes": {
                "type": "string",
                "description": "Any additional context"
            }
        },
        "required": ["floor", "mismatch_type", "we_predicted", "video_shows"]
    }
}


# ============================================================================
# COMBINED TOOL LIST
# ============================================================================

ALL_EXTRACTION_TOOLS = [
    RUN_START_TOOL,
    NEOW_CHOICE_TOOL,
    PATH_CHOICE_TOOL,
    CARD_PICK_TOOL,
    BOSS_RELIC_PICK_TOOL,
    REST_ACTION_TOOL,
    EVENT_CHOICE_TOOL,
    SHOP_ACTION_TOOL,
    COMBAT_END_TOOL,
    RUN_END_TOOL,
    VERIFY_CARD_REWARD_TOOL,
    PREDICTION_MISMATCH_TOOL,
]


def to_gemini_tools() -> List[Dict]:
    """Convert tools to Gemini function calling format."""
    return [{
        "function_declarations": [
            {
                "name": tool["name"],
                "description": tool["description"],
                "parameters": tool["parameters"]
            }
            for tool in ALL_EXTRACTION_TOOLS
        ]
    }]


def create_extraction_prompt(
    predictions: Dict[str, Any],
    chunk_start: str,
    chunk_end: str,
) -> str:
    """
    Create extraction prompt with predictions embedded.

    Args:
        predictions: Dict containing predicted options for this chunk
        chunk_start: Start timestamp
        chunk_end: End timestamp

    Returns:
        Prompt string for Gemini
    """
    prompt = f"""Analyze this Slay the Spire video from {chunk_start} to {chunk_end}.

"""

    # Add predictions if available
    if "card_reward" in predictions:
        cards = predictions["card_reward"]
        prompt += "PREDICTED CARD REWARD OPTIONS:\n"
        for i, card in enumerate(cards):
            prompt += f"  [{i}] {card}\n"
        prompt += "  [-1] SKIP\n\n"
        prompt += """For card_pick, you MUST use chosen_index from the list above.
If the cards shown don't match these predictions, call verify_card_reward first,
then call card_pick with the index that best matches what was chosen.

"""

    if "shop_cards" in predictions:
        prompt += "PREDICTED SHOP CARDS:\n"
        for i, card in enumerate(predictions["shop_cards"]):
            prompt += f"  [{i}] {card}\n"
        prompt += "\n"

    if "boss_relics" in predictions:
        prompt += "PREDICTED BOSS RELICS:\n"
        for i, relic in enumerate(predictions["boss_relics"]):
            prompt += f"  [{i}] {relic}\n"
        prompt += "\n"

    prompt += """INSTRUCTIONS:
1. Watch the video segment carefully
2. For each decision point, call the appropriate tool
3. When predictions are provided, use INDEX-BASED selection (0, 1, 2, etc.)
4. If you see something that doesn't match our predictions, call prediction_mismatch
5. Report ALL decisions in chronological order

IMPORTANT: Do NOT invent card/relic names. If predictions are provided,
select by index. If no predictions, describe what you see exactly as shown."""

    return prompt


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    print("Extraction Tools V2")
    print("=" * 50)
    print(f"Total tools: {len(ALL_EXTRACTION_TOOLS)}")
    print()

    for tool in ALL_EXTRACTION_TOOLS:
        print(f"  - {tool['name']}")

    print()
    print("Sample prompt with predictions:")
    print("-" * 50)

    predictions = {
        "card_reward": ["Consecrate", "Pressure Points", "Crush Joints"]
    }

    print(create_extraction_prompt(predictions, "05:00", "07:00"))
