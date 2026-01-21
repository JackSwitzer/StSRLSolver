"""Seed-verified VOD extraction.

Uses seed prediction to know expected card rewards, relics, etc.
Model verifies predictions and fills in decisions made.
"""

import json
from dataclasses import dataclass, field, asdict
from typing import Optional, Literal
from datetime import datetime
from pathlib import Path


@dataclass
class PredictedReward:
    """Predicted reward from seed analysis."""
    floor: int
    room_type: str  # "monster", "elite", "boss", "?"
    card_options: list[str]  # 1-4 cards based on relics
    has_second_reward: bool = False  # Prayer Wheel
    second_card_options: list[str] = field(default_factory=list)
    gold_range: tuple[int, int] = (10, 20)
    potion_chance: float = 0.4


@dataclass
class PredictedShop:
    """Predicted shop contents from seed."""
    floor: int
    cards_for_sale: list[dict]  # [{name, cost, rarity}]
    relics_for_sale: list[dict]
    potions_for_sale: list[dict]
    removal_cost: int


@dataclass
class VerifiedDecision:
    """A decision point that was verified against prediction."""
    floor: int
    decision_type: str
    predicted: dict  # What we expected from seed
    actual: dict     # What actually happened (from video)
    match: bool      # Did prediction match?
    timestamp: Optional[str] = None
    confidence: float = 1.0  # Model's confidence


@dataclass
class CombatRecord:
    """Combat with full card sequence tracking."""
    floor: int
    enemy: str

    # State before/after
    hp_before: int
    hp_after: int
    max_hp: int
    potions_before: list[str]
    potions_after: list[str]
    gold_before: int
    gold_after: int

    # Combat details
    turns: list[dict]  # [{turn: 1, cards: [...], potions: [...], enemy_move: "..."}]
    relics_triggered: list[str]
    total_damage_dealt: Optional[int] = None
    total_damage_taken: Optional[int] = None

    timestamp_start: Optional[str] = None
    timestamp_end: Optional[str] = None


@dataclass
class CardRewardDecision:
    """Card reward with seed-predicted options."""
    floor: int
    # Predicted from seed
    predicted_options: list[str]
    predicted_rarities: list[str]

    # Actual from video
    actual_options: list[str]  # May differ if prediction wrong
    chosen: str  # Card name or "SKIP"

    # Second reward (Prayer Wheel)
    has_second_reward: bool = False
    second_options: list[str] = field(default_factory=list)
    second_chosen: Optional[str] = None

    # Singing Bowl option
    singing_bowl_used: bool = False

    timestamp: Optional[str] = None
    prediction_matched: bool = True


@dataclass
class RunExtraction:
    """Complete run extraction with seed verification."""
    video_id: str
    video_url: str

    # Seed info
    seed: str
    seed_source: str = "detected"  # "detected", "provided", "guessed"

    # Run metadata
    character: str = "Watcher"
    ascension: int = 20

    # Neow
    neow_predicted_options: list[str] = field(default_factory=list)
    neow_actual_options: list[str] = field(default_factory=list)
    neow_chosen: Optional[str] = None
    neow_drawback: Optional[str] = None

    # Combats with full sequences
    combats: list[CombatRecord] = field(default_factory=list)

    # Card rewards (most important for training)
    card_rewards: list[CardRewardDecision] = field(default_factory=list)

    # Other decisions
    shop_visits: list[dict] = field(default_factory=list)
    rest_decisions: list[dict] = field(default_factory=list)
    boss_relics: list[dict] = field(default_factory=list)
    events: list[dict] = field(default_factory=list)

    # Path taken
    path: list[dict] = field(default_factory=list)  # [{floor, room_type, node_x, node_y}]

    # Result
    victory: bool = False
    heart_kill: bool = False
    floor_reached: int = 0
    final_hp: int = 0

    # Quality metrics
    prediction_accuracy: float = 0.0  # % of predictions that matched
    extraction_passes: int = 1
    extracted_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def to_dict(self) -> dict:
        return asdict(self)

    def save(self, output_dir: Path = Path("data/merl_extracted")):
        output_dir.mkdir(parents=True, exist_ok=True)
        path = output_dir / f"{self.video_id}_verified.json"
        with open(path, "w") as f:
            json.dump(self.to_dict(), f, indent=2, default=str)
        return path


# =============================================================================
# Tool Definitions - Smarter with seed context
# =============================================================================

VERIFICATION_TOOLS = [
    {
        "name": "verify_seed",
        "description": "Confirm or correct the detected seed",
        "parameters": {
            "type": "object",
            "properties": {
                "seed_visible": {"type": "boolean", "description": "Was seed visible in video?"},
                "seed": {"type": "string", "description": "Seed if visible"},
                "timestamp": {"type": "string"}
            },
            "required": ["seed_visible"]
        }
    },
    {
        "name": "verify_neow",
        "description": "Verify Neow options and record choice. We predict options from seed.",
        "parameters": {
            "type": "object",
            "properties": {
                "predicted_options_correct": {
                    "type": "boolean",
                    "description": "Did predicted options match what was shown?"
                },
                "actual_options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Actual options if different from prediction"
                },
                "chosen_index": {
                    "type": "integer",
                    "description": "Index 0-3 of option chosen"
                },
                "chosen_description": {
                    "type": "string",
                    "description": "Description of what was chosen"
                },
                "drawback": {"type": "string"},
                "timestamp": {"type": "string"}
            },
            "required": ["chosen_index", "chosen_description"]
        }
    },
    {
        "name": "log_combat_full",
        "description": "Log complete combat with card sequence per turn",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "enemy": {"type": "string"},
                "hp_before": {"type": "integer"},
                "hp_after": {"type": "integer"},
                "max_hp": {"type": "integer"},
                "potions_before": {"type": "array", "items": {"type": "string"}},
                "potions_after": {"type": "array", "items": {"type": "string"}},
                "gold_before": {"type": "integer"},
                "gold_after": {"type": "integer"},
                "turns": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "turn": {"type": "integer"},
                            "cards_played": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Cards played in order"
                            },
                            "potions_used": {
                                "type": "array",
                                "items": {"type": "string"}
                            },
                            "enemy_intent": {"type": "string"}
                        }
                    },
                    "description": "Per-turn breakdown"
                },
                "relics_triggered": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Relics that activated (Lizard Tail, Orichalcum, etc.)"
                },
                "timestamp_start": {"type": "string"},
                "timestamp_end": {"type": "string"}
            },
            "required": ["floor", "enemy", "hp_before", "hp_after", "turns"]
        }
    },
    {
        "name": "verify_card_reward",
        "description": "Verify predicted card reward options and record choice",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "prediction_correct": {
                    "type": "boolean",
                    "description": "Did predicted cards match actual options?"
                },
                "actual_options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Actual cards shown (only if different from prediction)"
                },
                "chosen": {
                    "type": "string",
                    "description": "Card name chosen, or 'SKIP', or 'SINGING_BOWL'"
                },
                "has_second_reward": {
                    "type": "boolean",
                    "description": "Was there a second card reward (Prayer Wheel)?"
                },
                "second_options": {
                    "type": "array",
                    "items": {"type": "string"}
                },
                "second_chosen": {"type": "string"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "chosen"]
        }
    },
    {
        "name": "log_shop_visit",
        "description": "Log shop visit with all transactions",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "cards_purchased": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "card": {"type": "string"},
                            "cost": {"type": "integer"}
                        }
                    }
                },
                "relics_purchased": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "relic": {"type": "string"},
                            "cost": {"type": "integer"}
                        }
                    }
                },
                "potions_purchased": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "potion": {"type": "string"},
                            "cost": {"type": "integer"}
                        }
                    }
                },
                "cards_removed": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "card": {"type": "string"},
                            "cost": {"type": "integer"}
                        }
                    }
                },
                "timestamp": {"type": "string"}
            },
            "required": ["floor"]
        }
    },
    {
        "name": "log_rest_site",
        "description": "Log rest site decision",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "action": {
                    "type": "string",
                    "enum": ["rest", "smith", "lift", "dig", "recall", "toke"]
                },
                "card_upgraded": {"type": "string"},
                "hp_before": {"type": "integer"},
                "hp_after": {"type": "integer"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "action"]
        }
    },
    {
        "name": "log_boss_relic_choice",
        "description": "Log boss relic selection",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer", "description": "17, 34, or 51"},
                "options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "All 3 boss relics offered"
                },
                "chosen": {"type": "string"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "options", "chosen"]
        }
    },
    {
        "name": "log_event",
        "description": "Log event encounter",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "event_name": {"type": "string"},
                "choice_made": {"type": "string"},
                "outcome": {"type": "string"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "event_name", "choice_made"]
        }
    },
    {
        "name": "log_relic_obtained",
        "description": "Log relic obtained (elite, chest, event)",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "relic": {"type": "string"},
                "source": {"type": "string", "enum": ["elite", "chest", "event", "shop", "boss"]},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "relic", "source"]
        }
    },
    {
        "name": "log_path_choice",
        "description": "Log map pathing decision",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "room_type": {
                    "type": "string",
                    "enum": ["monster", "elite", "rest", "shop", "?", "treasure", "boss"]
                },
                "reasoning": {"type": "string"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "room_type"]
        }
    },
    {
        "name": "log_result",
        "description": "Log final run result",
        "parameters": {
            "type": "object",
            "properties": {
                "victory": {"type": "boolean"},
                "heart_kill": {"type": "boolean"},
                "floor_reached": {"type": "integer"},
                "final_hp": {"type": "integer"},
                "cause_of_death": {"type": "string", "description": "Enemy that killed, if loss"},
                "timestamp": {"type": "string"}
            },
            "required": ["victory", "floor_reached"]
        }
    }
]


# =============================================================================
# Extraction System Prompt with Seed Context
# =============================================================================

def build_extraction_prompt(
    seed: Optional[str],
    predicted_neow: list[str],
    predicted_rewards: list[PredictedReward],
    current_floor: int = 0,
) -> str:
    """Build extraction prompt with seed predictions."""

    seed_info = f"SEED: {seed}" if seed else "SEED: Unknown - try to find it in pause menu"

    neow_info = ""
    if predicted_neow:
        neow_info = f"""
PREDICTED NEOW OPTIONS:
1. {predicted_neow[0]}
2. {predicted_neow[1]}
3. {predicted_neow[2]}
4. {predicted_neow[3]}

Verify these match what's shown and record which was chosen."""

    rewards_info = ""
    if predicted_rewards:
        rewards_preview = []
        for r in predicted_rewards[:10]:
            opts = ", ".join(r.card_options)
            rewards_preview.append(f"  F{r.floor} ({r.room_type}): [{opts}]")
        rewards_info = f"""
PREDICTED CARD REWARDS (verify and record choices):
{chr(10).join(rewards_preview)}
{"  ... more floors predicted" if len(predicted_rewards) > 10 else ""}

For each reward, confirm options match and record: card chosen OR 'SKIP' OR 'SINGING_BOWL'"""

    return f"""You are extracting training data from a Merl61 Watcher A20 VOD.

{seed_info}
{neow_info}
{rewards_info}

YOUR TASKS:
1. VERIFY predictions match what's shown in video
2. RECORD every decision made:
   - Card rewards: which card chosen (or SKIP/SINGING_BOWL)
   - Combats: HP before/after, cards played per turn, relics triggered
   - Shops: all purchases and removals
   - Rest sites: rest/smith/lift/dig
   - Boss relics: all 3 options and which chosen
   - Events: choice made and outcome

CRITICAL:
- Log EVERY card reward, including SKIPs
- For combats, try to capture the card play sequence per turn
- Note when relics trigger (Lizard Tail, Meat on the Bone, etc.)
- Record HP changes accurately

Call the appropriate logging tools for each decision point."""


# =============================================================================
# Main Extractor Class
# =============================================================================

class SeedVerifiedExtractor:
    """Extract with seed prediction verification."""

    MODEL = "gemini-3-flash-preview"  # Only 3.0+ supports video

    def __init__(self, api_key: Optional[str] = None):
        import os
        self.api_key = api_key or os.environ.get("GOOGLE_API_KEY")
        if not self.api_key:
            raise ValueError("GOOGLE_API_KEY required")

        from google import genai
        self.client = genai.Client(api_key=self.api_key)

    def predict_rewards_from_seed(self, seed: str, path: list[dict]) -> list[PredictedReward]:
        """Use our RNG system to predict card rewards.

        TODO: Integrate with core/state/game_rng.py
        """
        # Placeholder - will integrate with actual seed prediction
        return []

    def extract(
        self,
        video_url: str,
        seed: Optional[str] = None,
        passes: int = 3,
    ) -> RunExtraction:
        """Extract run with seed verification.

        Args:
            video_url: YouTube URL
            seed: Known seed (optional, will try to detect)
            passes: Number of extraction passes for voting
        """
        import re
        from google.genai import types

        match = re.search(r"(?:v=|/)([a-zA-Z0-9_-]{11})", video_url)
        video_id = match.group(1) if match else "unknown"

        print(f"Extracting {video_id}...")
        print(f"  Seed: {seed or 'detecting...'}")
        print(f"  Passes: {passes}")

        # Build tools
        tools = [
            types.Tool(function_declarations=[
                types.FunctionDeclaration(
                    name=t["name"],
                    description=t["description"],
                    parameters=t["parameters"]
                )
                for t in VERIFICATION_TOOLS
            ])
        ]

        # Predict rewards if seed known
        predicted_neow = []  # TODO: predict from seed
        predicted_rewards = []  # TODO: predict from seed

        prompt = build_extraction_prompt(seed, predicted_neow, predicted_rewards)

        # Multi-pass extraction
        all_results = []
        for i in range(passes):
            print(f"  Pass {i+1}/{passes}...")
            try:
                response = self.client.models.generate_content(
                    model=self.MODEL,
                    contents=[video_url, prompt],
                    config=types.GenerateContentConfig(
                        tools=tools,
                        tool_config=types.ToolConfig(
                            function_calling_config=types.FunctionCallingConfig(
                                mode="AUTO"
                            )
                        )
                    )
                )

                # Extract tool calls
                calls = []
                for part in response.candidates[0].content.parts:
                    if hasattr(part, 'function_call') and part.function_call:
                        fc = part.function_call
                        calls.append({
                            "name": fc.name,
                            "args": dict(fc.args) if fc.args else {}
                        })

                all_results.append(calls)
                print(f"    Got {len(calls)} tool calls")

            except Exception as e:
                print(f"    Error: {e}")

        # Merge results with voting
        extraction = RunExtraction(
            video_id=video_id,
            video_url=video_url,
            seed=seed or "unknown",
            extraction_passes=passes,
        )

        # Apply calls (use first pass for now, TODO: voting)
        if all_results:
            self._apply_calls(extraction, all_results[0])

        return extraction

    def _apply_calls(self, extraction: RunExtraction, calls: list[dict]):
        """Apply tool calls to extraction."""
        for call in calls:
            name = call["name"]
            args = call["args"]

            if name == "verify_seed":
                if args.get("seed"):
                    extraction.seed = args["seed"]
                    extraction.seed_source = "detected"

            elif name == "verify_neow":
                extraction.neow_chosen = args.get("chosen_description")
                extraction.neow_drawback = args.get("drawback")
                if args.get("actual_options"):
                    extraction.neow_actual_options = args["actual_options"]

            elif name == "log_combat_full":
                combat = CombatRecord(
                    floor=args.get("floor", 0),
                    enemy=args.get("enemy", "Unknown"),
                    hp_before=args.get("hp_before", 0),
                    hp_after=args.get("hp_after", 0),
                    max_hp=args.get("max_hp", 72),
                    potions_before=args.get("potions_before", []),
                    potions_after=args.get("potions_after", []),
                    gold_before=args.get("gold_before", 0),
                    gold_after=args.get("gold_after", 0),
                    turns=args.get("turns", []),
                    relics_triggered=args.get("relics_triggered", []),
                    timestamp_start=args.get("timestamp_start"),
                    timestamp_end=args.get("timestamp_end"),
                )
                extraction.combats.append(combat)

            elif name == "verify_card_reward":
                reward = CardRewardDecision(
                    floor=args.get("floor", 0),
                    predicted_options=[],  # TODO: from seed
                    predicted_rarities=[],
                    actual_options=args.get("actual_options", []),
                    chosen=args.get("chosen", "SKIP"),
                    has_second_reward=args.get("has_second_reward", False),
                    second_options=args.get("second_options", []),
                    second_chosen=args.get("second_chosen"),
                    singing_bowl_used=args.get("chosen") == "SINGING_BOWL",
                    timestamp=args.get("timestamp"),
                    prediction_matched=args.get("prediction_correct", True),
                )
                extraction.card_rewards.append(reward)

            elif name == "log_shop_visit":
                extraction.shop_visits.append(args)

            elif name == "log_rest_site":
                extraction.rest_decisions.append(args)

            elif name == "log_boss_relic_choice":
                extraction.boss_relics.append(args)

            elif name == "log_event":
                extraction.events.append(args)

            elif name == "log_path_choice":
                extraction.path.append(args)

            elif name == "log_result":
                extraction.victory = args.get("victory", False)
                extraction.heart_kill = args.get("heart_kill", False)
                extraction.floor_reached = args.get("floor_reached", 0)
                extraction.final_hp = args.get("final_hp", 0)

    def print_summary(self, extraction: RunExtraction):
        """Print extraction summary."""
        print("\n" + "="*60)
        print(f"SEED-VERIFIED EXTRACTION: {extraction.video_id}")
        print("="*60)

        print(f"Seed: {extraction.seed} ({extraction.seed_source})")

        result = "WIN" if extraction.victory else "LOSS"
        heart = " (Heart Kill!)" if extraction.heart_kill else ""
        print(f"Result: {result} - Floor {extraction.floor_reached}{heart}")

        if extraction.neow_chosen:
            print(f"\nNeow: {extraction.neow_chosen}")
            if extraction.neow_drawback:
                print(f"  Drawback: {extraction.neow_drawback}")

        print(f"\nCombats: {len(extraction.combats)}")
        for c in extraction.combats[:3]:
            hp_change = c.hp_after - c.hp_before
            sign = "+" if hp_change >= 0 else ""
            print(f"  F{c.floor} {c.enemy}: {c.hp_before}→{c.hp_after} ({sign}{hp_change})")
            if c.turns:
                for t in c.turns[:2]:
                    cards = t.get("cards_played", [])
                    print(f"    T{t.get('turn', '?')}: {' → '.join(cards[:5])}")
        if len(extraction.combats) > 3:
            print(f"  ... and {len(extraction.combats)-3} more")

        print(f"\nCard Rewards: {len(extraction.card_rewards)}")
        skips = sum(1 for r in extraction.card_rewards if r.chosen == "SKIP")
        picks = len(extraction.card_rewards) - skips
        print(f"  Picks: {picks}, Skips: {skips}")
        for r in extraction.card_rewards[:5]:
            opts = ", ".join(r.actual_options) if r.actual_options else "?"
            print(f"  F{r.floor}: {r.chosen} from [{opts}]")

        print(f"\nShops: {len(extraction.shop_visits)}")
        print(f"Rests: {len(extraction.rest_decisions)}")
        print(f"Events: {len(extraction.events)}")
        print(f"Boss Relics: {len(extraction.boss_relics)}")

        print("="*60)


def extract_verified(video_url: str, seed: Optional[str] = None, passes: int = 1) -> RunExtraction:
    """Convenience function."""
    from core.training.env import load_env
    load_env()

    extractor = SeedVerifiedExtractor()
    extraction = extractor.extract(video_url, seed=seed, passes=passes)
    extractor.print_summary(extraction)

    path = extraction.save()
    print(f"\nSaved to: {path}")

    return extraction
