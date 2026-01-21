"""
Seed-Enhanced VOD Extraction System

Uses GameRNGState to predict all deterministic elements, simplifying
the model's job to just identifying which choice was made.

Architecture:
1. First Pass: Detect seed from video
2. Prediction: Use GameRNGState to generate all deterministic elements
3. Extraction: Model identifies choices by index (not content)
4. Verification: Compare actual gameplay to predictions

This approach:
- Eliminates OCR errors for card/enemy names
- Validates our game engine implementation
- Creates simpler, more reliable model decisions
"""

import json
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Any, Tuple
from enum import Enum
import sys
import os

# Setup imports
_core_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "core")
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.state.game_rng import GameRNGState, RNGStream
from core.generation.rewards import generate_card_rewards, RewardState


class ChoiceType(str, Enum):
    """Types of choices the model needs to make."""
    RUN_START = "run_start"  # Detect new run + seed
    NEOW = "neow"  # 4 Neow options (0-3)
    PATH = "path"  # Map node selection
    CARD_PICK = "card_pick"  # Card reward choice (index or SKIP)
    SHOP_PURCHASE = "shop_purchase"  # Items bought
    SHOP_REMOVE = "shop_remove"  # Card removed
    REST_ACTION = "rest_action"  # rest/smith/dig/recall/toke
    EVENT_CHOICE = "event_choice"  # Event option
    BOSS_RELIC = "boss_relic"  # Boss relic choice (0-2)
    POTION_USE = "potion_use"  # Potion usage
    COMBAT_CARDS = "combat_cards"  # Cards played per turn
    MISMATCH = "mismatch"  # Prediction doesn't match actual


@dataclass
class PredictedOptions:
    """Predicted options for a decision point."""
    choice_type: ChoiceType
    floor: int
    options: List[str]
    metadata: Dict[str, Any] = field(default_factory=dict)

    def to_prompt(self) -> str:
        """Format options for model prompt."""
        opts = "\n".join(f"  [{i}] {opt}" for i, opt in enumerate(self.options))
        return f"Options:\n{opts}\n  [-1] SKIP (if applicable)"


@dataclass
class PlayerChoice:
    """A choice made by the player."""
    choice_type: ChoiceType
    floor: int
    chosen_index: int  # -1 for SKIP, 0+ for choice
    timestamp: Optional[str] = None

    # For verification
    predicted_options: Optional[List[str]] = None
    actual_observed: Optional[str] = None  # What model actually saw
    matches_prediction: bool = True


@dataclass
class VerificationResult:
    """Result of comparing prediction to actual gameplay."""
    floor: int
    choice_type: ChoiceType
    predicted: List[str]
    observed: List[str]
    matches: bool
    notes: str = ""


class SeedEnhancedState:
    """
    State machine that tracks both predictions and actuals.

    Uses GameRNGState for predictions, tracks player choices,
    and maintains verification data.
    """

    def __init__(
        self,
        seed: Optional[str] = None,
        character: str = "WATCHER",
        ascension: int = 20,
    ):
        self.seed = seed
        self.character = character
        self.ascension = ascension

        # RNG state for predictions
        self.rng_state: Optional[GameRNGState] = None
        if seed:
            self.rng_state = GameRNGState(seed)

        # Card blizzard (pity timer) state
        self.reward_state = RewardState()

        # Game state
        self.floor = 0
        self.act = 1
        self.hp = 72
        self.max_hp = 72
        self.gold = 99

        # Relics that affect rewards
        self.has_question_card = False
        self.has_busted_crown = False
        self.has_prayer_wheel = False
        self.has_prismatic_shard = False
        self.has_singing_bowl = False

        # Tracking
        self.choices: List[PlayerChoice] = []
        self.verifications: List[VerificationResult] = []
        self.prediction_accuracy = 1.0

    def set_seed(self, seed: str):
        """Set/update seed and reinitialize RNG."""
        self.seed = seed
        self.rng_state = GameRNGState(seed)

    def predict_neow_options(self) -> PredictedOptions:
        """
        Predict the 4 Neow options.

        Note: Neow uses separate RNG stream, options are complex.
        For now, return generic option descriptions.
        Full implementation would use NeowEvent.rng stream.
        """
        # Neow always offers 4 options, but they're complex to predict
        # For extraction, we just need the model to identify which was chosen
        options = [
            "Option 1 (Blessing + Drawback)",
            "Option 2 (Blessing + Drawback)",
            "Option 3 (Blessing + Drawback)",
            "Option 4 (Boss Swap)",
        ]
        return PredictedOptions(
            choice_type=ChoiceType.NEOW,
            floor=0,
            options=options,
            metadata={"note": "Neow options are semi-random, model should describe choice"}
        )

    def predict_card_reward(self, room_type: str = "normal") -> PredictedOptions:
        """
        Predict card reward options using GameRNGState.

        Args:
            room_type: "normal", "elite", or "boss"

        Returns:
            PredictedOptions with card names
        """
        if not self.rng_state:
            return PredictedOptions(
                choice_type=ChoiceType.CARD_PICK,
                floor=self.floor,
                options=["Unknown (no seed)"],
                metadata={"error": "No seed set"}
            )

        # Get card RNG at current counter
        card_rng = self.rng_state.get_rng(RNGStream.CARD)

        # Determine number of cards
        num_cards = 3
        if self.has_busted_crown:
            num_cards -= 2
        if self.has_question_card:
            num_cards += 1
        num_cards = max(1, num_cards)

        # Generate predicted cards
        try:
            cards = generate_card_rewards(
                rng=card_rng,
                reward_state=self.reward_state,
                act=self.act,
                player_class=self.character,
                ascension=self.ascension,
                room_type=room_type,
                num_cards=num_cards,
                has_prismatic_shard=self.has_prismatic_shard,
                has_busted_crown=self.has_busted_crown,
                has_question_card=self.has_question_card,
            )

            options = [card.name for card in cards]

            # Add Singing Bowl option if present
            if self.has_singing_bowl:
                options.append("Skip (+2 Max HP)")

        except Exception as e:
            options = [f"Prediction error: {e}"]

        return PredictedOptions(
            choice_type=ChoiceType.CARD_PICK,
            floor=self.floor,
            options=options,
            metadata={
                "room_type": room_type,
                "num_cards": num_cards,
                "card_counter": self.rng_state.get_counter(RNGStream.CARD) if self.rng_state else None
            }
        )

    def predict_boss_relics(self) -> PredictedOptions:
        """Predict boss relic options (3 choices)."""
        # Boss relics use relicRng stream
        # For now, return generic - full implementation needs relic pool tracking
        return PredictedOptions(
            choice_type=ChoiceType.BOSS_RELIC,
            floor=self.floor,
            options=["Boss Relic 1", "Boss Relic 2", "Boss Relic 3"],
            metadata={"note": "Boss relics from shuffled pool"}
        )

    def apply_choice(self, choice: PlayerChoice):
        """
        Apply a player choice and update state.

        Also advances RNG counters appropriately.
        """
        self.choices.append(choice)

        if choice.choice_type == ChoiceType.NEOW:
            if self.rng_state:
                # Neow consumption depends on option
                # For now, use conservative estimate
                # Full implementation tracks specific options
                self.rng_state.apply_neow_choice("HUNDRED_GOLD")  # Placeholder

        elif choice.choice_type == ChoiceType.PATH:
            self.floor = choice.floor

        elif choice.choice_type == ChoiceType.CARD_PICK:
            if self.rng_state:
                # Card reward consumes ~9 cardRng calls
                self.rng_state.apply_combat("normal")  # This handles card consumption

        elif choice.choice_type == ChoiceType.SHOP_PURCHASE:
            if self.rng_state:
                self.rng_state.apply_shop()

    def record_verification(
        self,
        floor: int,
        choice_type: ChoiceType,
        predicted: List[str],
        observed: List[str],
        notes: str = ""
    ):
        """Record a verification comparing prediction to actual."""
        # Simple matching - check if observed is subset of predicted
        matches = all(obs in predicted for obs in observed) if observed else True

        self.verifications.append(VerificationResult(
            floor=floor,
            choice_type=choice_type,
            predicted=predicted,
            observed=observed,
            matches=matches,
            notes=notes
        ))

        # Update accuracy
        if self.verifications:
            match_count = sum(1 for v in self.verifications if v.matches)
            self.prediction_accuracy = match_count / len(self.verifications)

    def get_accuracy_report(self) -> Dict[str, Any]:
        """Get summary of prediction accuracy."""
        by_type: Dict[str, Dict[str, int]] = {}

        for v in self.verifications:
            key = v.choice_type.value
            if key not in by_type:
                by_type[key] = {"correct": 0, "total": 0}
            by_type[key]["total"] += 1
            if v.matches:
                by_type[key]["correct"] += 1

        return {
            "overall_accuracy": self.prediction_accuracy,
            "total_verifications": len(self.verifications),
            "by_type": {
                k: {
                    "accuracy": v["correct"] / v["total"] if v["total"] > 0 else 0,
                    **v
                }
                for k, v in by_type.items()
            },
            "mismatches": [
                {
                    "floor": v.floor,
                    "type": v.choice_type.value,
                    "predicted": v.predicted,
                    "observed": v.observed,
                    "notes": v.notes
                }
                for v in self.verifications if not v.matches
            ]
        }


# ============================================================================
# SIMPLIFIED TOOLS FOR MODEL
# ============================================================================

SEED_ENHANCED_TOOLS = [
    {
        "name": "run_start",
        "description": "Detect that a new run has started. Call when you see the character select or Neow room.",
        "parameters": {
            "type": "object",
            "properties": {
                "seed": {
                    "type": "string",
                    "description": "The seed string if visible (e.g., 'ABC123DEF'). Leave empty if not visible."
                },
                "character": {
                    "type": "string",
                    "enum": ["WATCHER", "IRONCLAD", "SILENT", "DEFECT"],
                    "description": "Which character is being played"
                },
                "ascension": {
                    "type": "integer",
                    "description": "Ascension level (0-20)"
                }
            },
            "required": ["character"]
        }
    },
    {
        "name": "neow_choice",
        "description": "Record Neow bonus selection. Describe the chosen bonus.",
        "parameters": {
            "type": "object",
            "properties": {
                "chosen_bonus": {
                    "type": "string",
                    "description": "Description of chosen Neow bonus (e.g., 'Obtain 100 Gold', 'Choose a Rare Card')"
                },
                "drawback": {
                    "type": "string",
                    "description": "Associated drawback if any (e.g., 'Lose all Gold', 'Take damage')"
                },
                "is_boss_swap": {
                    "type": "boolean",
                    "description": "True if chose to swap starting relic for boss relic"
                }
            },
            "required": ["chosen_bonus"]
        }
    },
    {
        "name": "path_choice",
        "description": "Record which map node was selected.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number (1-50+)"
                },
                "chosen_type": {
                    "type": "string",
                    "enum": ["monster", "elite", "rest", "shop", "event", "chest", "boss"],
                    "description": "Type of room chosen"
                },
                "available_types": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "All available room types at this node"
                }
            },
            "required": ["floor", "chosen_type"]
        }
    },
    {
        "name": "card_pick",
        "description": "Record card reward choice. Use predicted options if provided, or describe the card.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "chosen_index": {
                    "type": "integer",
                    "description": "Index of chosen card (0-based), or -1 for SKIP"
                },
                "chosen_name": {
                    "type": "string",
                    "description": "Name of chosen card (for verification), or 'SKIP'"
                },
                "observed_options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Card names observed in reward (for verification)"
                }
            },
            "required": ["floor", "chosen_index"]
        }
    },
    {
        "name": "shop_action",
        "description": "Record shop purchases and card removal.",
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
                    "description": "Names of cards purchased"
                },
                "relics_bought": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Names of relics purchased"
                },
                "potions_bought": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Names of potions purchased"
                },
                "card_removed": {
                    "type": "string",
                    "description": "Name of card removed (if any)"
                }
            },
            "required": ["floor"]
        }
    },
    {
        "name": "rest_action",
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
                    "enum": ["rest", "smith", "dig", "recall", "toke", "lift"],
                    "description": "Action taken at rest site"
                },
                "card_upgraded": {
                    "type": "string",
                    "description": "Name of card upgraded (if smith)"
                }
            },
            "required": ["floor", "action"]
        }
    },
    {
        "name": "event_choice",
        "description": "Record event decision.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "event_name": {
                    "type": "string",
                    "description": "Name of the event"
                },
                "choice_description": {
                    "type": "string",
                    "description": "Description of choice made"
                },
                "choice_index": {
                    "type": "integer",
                    "description": "Index of choice (if known)"
                }
            },
            "required": ["floor", "event_name", "choice_description"]
        }
    },
    {
        "name": "boss_relic_choice",
        "description": "Record boss relic selection after boss fight.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "chosen_relic": {
                    "type": "string",
                    "description": "Name of chosen boss relic"
                },
                "options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "All 3 boss relic options shown"
                }
            },
            "required": ["floor", "chosen_relic"]
        }
    },
    {
        "name": "combat_summary",
        "description": "Summary of a combat encounter.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "enemy": {
                    "type": "string",
                    "description": "Enemy name(s)"
                },
                "hp_before": {
                    "type": "integer",
                    "description": "HP before combat"
                },
                "hp_after": {
                    "type": "integer",
                    "description": "HP after combat"
                },
                "gold_gained": {
                    "type": "integer",
                    "description": "Gold gained from combat"
                },
                "total_turns": {
                    "type": "integer",
                    "description": "Number of turns combat took"
                }
            },
            "required": ["floor", "enemy"]
        }
    },
    {
        "name": "prediction_mismatch",
        "description": "Report when observed gameplay doesn't match seed prediction. Helps validate our game engine.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Floor number"
                },
                "type": {
                    "type": "string",
                    "description": "Type of mismatch (card_reward, enemy, shop, etc.)"
                },
                "predicted": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "What our seed prediction said"
                },
                "observed": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "What actually appeared in the video"
                },
                "notes": {
                    "type": "string",
                    "description": "Any additional context"
                }
            },
            "required": ["floor", "type", "predicted", "observed"]
        }
    },
    {
        "name": "run_end",
        "description": "Record that the run ended.",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {
                    "type": "integer",
                    "description": "Final floor reached"
                },
                "victory": {
                    "type": "boolean",
                    "description": "True if run was won"
                },
                "heart_kill": {
                    "type": "boolean",
                    "description": "True if heart was killed"
                },
                "cause_of_death": {
                    "type": "string",
                    "description": "What killed the player (if loss)"
                }
            },
            "required": ["floor", "victory"]
        }
    }
]


def create_extraction_prompt(
    state: SeedEnhancedState,
    chunk_start: str,
    chunk_end: str,
    predicted_options: Optional[PredictedOptions] = None,
) -> str:
    """
    Create prompt for model with seed-based predictions.

    If predictions are available, include them for the model to verify.
    """
    prompt = f"""Watch from {chunk_start} to {chunk_end} of this Slay the Spire Watcher run.

CURRENT STATE:
- Floor: {state.floor}
- HP: {state.hp}/{state.max_hp}
- Gold: {state.gold}
- Act: {state.act}
"""

    if state.seed:
        prompt += f"- Seed: {state.seed}\n"

    if predicted_options:
        prompt += f"""
PREDICTED OPTIONS (from seed):
{predicted_options.to_prompt()}

NOTE: If the actual options in the video don't match these predictions,
call prediction_mismatch() to help us validate our game engine.
"""

    prompt += """
YOUR TASK:
1. Call the appropriate function for each decision point you observe
2. Use index-based choices when predictions are provided
3. If predictions don't match actual gameplay, call prediction_mismatch()

For card rewards: Use chosen_index (0, 1, 2, etc.) or -1 for SKIP
For path choices: Report the room TYPE chosen (monster, elite, shop, etc.)
For shops: List all purchases and any card removal
For events: Describe the choice made

Watch carefully and report ALL decisions in this segment."""

    return prompt


def to_gemini_tools() -> List[Dict]:
    """Convert tools to Gemini function calling format."""
    return [{
        "function_declarations": [
            {
                "name": tool["name"],
                "description": tool["description"],
                "parameters": tool["parameters"]
            }
            for tool in SEED_ENHANCED_TOOLS
        ]
    }]
