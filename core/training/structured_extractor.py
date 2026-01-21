"""Structured VOD extraction using tool calls and state tracking.

The model outputs structured tool calls that update a run state machine.
Multi-pass voting and chunked processing for reliability.
"""

import json
from dataclasses import dataclass, field, asdict
from typing import Optional, Literal
from datetime import datetime
from pathlib import Path


# =============================================================================
# State Tracking
# =============================================================================

@dataclass
class CombatSnapshot:
    """State snapshot before/after combat."""
    hp: int
    max_hp: int
    gold: int
    potions: list[str]
    relics: list[str]
    deck_size: int


@dataclass
class CombatRecord:
    """Full combat record with card sequence."""
    floor: int
    enemy: str
    before: CombatSnapshot
    after: CombatSnapshot
    card_sequence: list[dict]  # [{card, target, energy, turn}]
    relics_triggered: list[str]  # e.g., ["Lizard Tail", "Meat on the Bone"]
    potions_used: list[str]
    turns_taken: int
    timestamp_start: Optional[str] = None
    timestamp_end: Optional[str] = None


@dataclass
class CardRewardRecord:
    """Card reward with full context."""
    floor: int
    options: list[str]
    chosen: str  # Card name or "SKIP"
    singing_bowl_hp: bool = False  # If Singing Bowl was used
    timestamp: Optional[str] = None


@dataclass
class RewardBundle:
    """All rewards from a single combat/event."""
    floor: int
    card_reward: Optional[CardRewardRecord] = None
    potion: Optional[str] = None
    gold: Optional[int] = None
    relic: Optional[str] = None
    timestamp: Optional[str] = None


@dataclass
class ShopRecord:
    """Shop visit with all transactions."""
    floor: int
    purchases: list[dict]  # [{type: "card"|"relic"|"potion", item, cost}]
    removals: list[dict]   # [{card, cost}]
    timestamp: Optional[str] = None


@dataclass
class RestRecord:
    """Rest site decision."""
    floor: int
    action: Literal["rest", "smith", "lift", "dig", "recall", "toke"]
    card_upgraded: Optional[str] = None
    hp_before: Optional[int] = None
    hp_after: Optional[int] = None
    timestamp: Optional[str] = None


@dataclass
class EventRecord:
    """Event encounter."""
    floor: int
    event_name: str
    choice_made: str
    outcome: str  # e.g., "Gained 100 gold", "Lost 10 HP, gained relic"
    timestamp: Optional[str] = None


@dataclass
class RunState:
    """Full run state tracker."""
    video_id: str
    seed: Optional[str] = None
    character: str = "Watcher"
    ascension: int = 20

    # Current state
    floor: int = 0
    act: int = 1
    hp: int = 72
    max_hp: int = 72
    gold: int = 99
    potions: list[str] = field(default_factory=list)
    relics: list[str] = field(default_factory=lambda: ["PureWater"])
    deck: list[str] = field(default_factory=lambda: [
        "Strike", "Strike", "Strike", "Strike",
        "Defend", "Defend", "Defend", "Defend",
        "Eruption", "Vigilance"
    ])

    # Records
    neow_options: list[str] = field(default_factory=list)
    neow_chosen: Optional[str] = None
    neow_drawback: Optional[str] = None

    combats: list[CombatRecord] = field(default_factory=list)
    rewards: list[RewardBundle] = field(default_factory=list)
    shops: list[ShopRecord] = field(default_factory=list)
    rests: list[RestRecord] = field(default_factory=list)
    events: list[EventRecord] = field(default_factory=list)
    boss_relics: list[dict] = field(default_factory=list)  # [{floor, options, chosen}]

    # Result
    victory: bool = False
    heart_kill: bool = False
    floor_reached: int = 0

    # Metadata
    extracted_at: str = field(default_factory=lambda: datetime.now().isoformat())
    extraction_passes: int = 0
    confidence_scores: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        """Serialize to dict."""
        return {
            "video_id": self.video_id,
            "seed": self.seed,
            "character": self.character,
            "ascension": self.ascension,
            "result": {
                "victory": self.victory,
                "heart_kill": self.heart_kill,
                "floor_reached": self.floor_reached,
            },
            "neow": {
                "options": self.neow_options,
                "chosen": self.neow_chosen,
                "drawback": self.neow_drawback,
            },
            "final_state": {
                "hp": self.hp,
                "max_hp": self.max_hp,
                "gold": self.gold,
                "potions": self.potions,
                "relics": self.relics,
                "deck": self.deck,
            },
            "combats": [asdict(c) for c in self.combats],
            "rewards": [asdict(r) for r in self.rewards],
            "shops": [asdict(s) for s in self.shops],
            "rests": [asdict(r) for r in self.rests],
            "events": [asdict(e) for e in self.events],
            "boss_relics": self.boss_relics,
            "metadata": {
                "extracted_at": self.extracted_at,
                "extraction_passes": self.extraction_passes,
                "confidence_scores": self.confidence_scores,
            }
        }

    def save(self, output_dir: Path = Path("data/merl_extracted")):
        """Save to JSON."""
        output_dir.mkdir(parents=True, exist_ok=True)
        path = output_dir / f"{self.video_id}_structured.json"
        with open(path, "w") as f:
            json.dump(self.to_dict(), f, indent=2)
        return path


# =============================================================================
# Tool Definitions for Gemini
# =============================================================================

EXTRACTION_TOOLS = [
    {
        "name": "log_seed",
        "description": "Record the run seed shown at start or in pause menu",
        "parameters": {
            "type": "object",
            "properties": {
                "seed": {"type": "string", "description": "The seed string (e.g., 'ABC123DEF')"},
                "timestamp": {"type": "string", "description": "MM:SS timestamp"}
            },
            "required": ["seed"]
        }
    },
    {
        "name": "log_neow",
        "description": "Record Neow bonus selection at run start",
        "parameters": {
            "type": "object",
            "properties": {
                "options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "All 4 Neow options shown"
                },
                "chosen": {"type": "string", "description": "The option selected"},
                "drawback": {"type": "string", "description": "Drawback if whale bonus chosen"},
                "timestamp": {"type": "string"}
            },
            "required": ["options", "chosen"]
        }
    },
    {
        "name": "start_combat",
        "description": "Mark beginning of a combat encounter",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "enemy": {"type": "string", "description": "Enemy name(s), e.g., 'Jaw Worm' or '2 Louses'"},
                "hp": {"type": "integer", "description": "Player HP at combat start"},
                "max_hp": {"type": "integer"},
                "potions": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Potions held at combat start"
                },
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "enemy", "hp"]
        }
    },
    {
        "name": "log_card_play",
        "description": "Record a card being played during combat",
        "parameters": {
            "type": "object",
            "properties": {
                "card": {"type": "string", "description": "Card name (include + if upgraded)"},
                "target": {"type": "string", "description": "Target if applicable"},
                "turn": {"type": "integer"},
                "energy_after": {"type": "integer", "description": "Energy remaining after play"},
                "timestamp": {"type": "string"}
            },
            "required": ["card", "turn"]
        }
    },
    {
        "name": "log_potion_use",
        "description": "Record potion used during combat",
        "parameters": {
            "type": "object",
            "properties": {
                "potion": {"type": "string"},
                "target": {"type": "string"},
                "turn": {"type": "integer"},
                "timestamp": {"type": "string"}
            },
            "required": ["potion", "turn"]
        }
    },
    {
        "name": "end_combat",
        "description": "Mark end of combat with final state",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "hp_after": {"type": "integer"},
                "potions_after": {"type": "array", "items": {"type": "string"}},
                "relics_triggered": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Relics that activated (Lizard Tail, Meat on the Bone, etc.)"
                },
                "turns_taken": {"type": "integer"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "hp_after"]
        }
    },
    {
        "name": "log_card_reward",
        "description": "Record card reward screen choice - ALWAYS call this after combat, even for SKIP",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Cards offered (usually 3)"
                },
                "chosen": {
                    "type": "string",
                    "description": "Card taken OR 'SKIP' if skipped"
                },
                "singing_bowl": {
                    "type": "boolean",
                    "description": "True if Singing Bowl +2 HP was used instead"
                },
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "options", "chosen"]
        }
    },
    {
        "name": "log_potion_reward",
        "description": "Record potion obtained from combat/event",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "potion": {"type": "string"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "potion"]
        }
    },
    {
        "name": "log_gold_reward",
        "description": "Record gold obtained from combat",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "amount": {"type": "integer"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "amount"]
        }
    },
    {
        "name": "log_relic_reward",
        "description": "Record relic obtained (from elite, chest, event)",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "relic": {"type": "string"},
                "source": {"type": "string", "description": "elite, chest, event, boss"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "relic"]
        }
    },
    {
        "name": "log_shop",
        "description": "Record all shop transactions in a single visit",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "purchases": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": {"type": "string", "enum": ["card", "relic", "potion"]},
                            "item": {"type": "string"},
                            "cost": {"type": "integer"}
                        }
                    }
                },
                "removals": {
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
        "name": "log_rest",
        "description": "Record rest site decision",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "action": {
                    "type": "string",
                    "enum": ["rest", "smith", "lift", "dig", "recall", "toke"],
                    "description": "Action taken"
                },
                "card_upgraded": {"type": "string", "description": "Card upgraded if smith"},
                "hp_before": {"type": "integer"},
                "hp_after": {"type": "integer"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "action"]
        }
    },
    {
        "name": "log_boss_relic",
        "description": "Record boss relic selection after act boss",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer", "description": "17, 34, or 51"},
                "options": {"type": "array", "items": {"type": "string"}},
                "chosen": {"type": "string"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "options", "chosen"]
        }
    },
    {
        "name": "log_event",
        "description": "Record event encounter and choice",
        "parameters": {
            "type": "object",
            "properties": {
                "floor": {"type": "integer"},
                "event_name": {"type": "string"},
                "choice": {"type": "string", "description": "Option selected"},
                "outcome": {"type": "string", "description": "What happened"},
                "timestamp": {"type": "string"}
            },
            "required": ["floor", "event_name", "choice"]
        }
    },
    {
        "name": "log_result",
        "description": "Record final run result",
        "parameters": {
            "type": "object",
            "properties": {
                "victory": {"type": "boolean"},
                "heart_kill": {"type": "boolean"},
                "floor_reached": {"type": "integer"},
                "final_hp": {"type": "integer"},
                "timestamp": {"type": "string"}
            },
            "required": ["victory", "floor_reached"]
        }
    }
]


# =============================================================================
# Extraction Prompts
# =============================================================================

SYSTEM_PROMPT = """You are analyzing a Slay the Spire Watcher gameplay VOD from expert streamer Merl61.
Your job is to extract EVERY decision point by calling the appropriate logging tools.

CRITICAL RULES:
1. Call log_card_reward after EVERY combat, even if the player SKIPS (chosen="SKIP")
2. Track HP before and after every combat
3. Note when relics trigger (Lizard Tail revive, Meat on the Bone heal, etc.)
4. Record the seed if visible
5. For card plays, try to track the sequence within each turn

The video may show a VICTORY screen at the START from a previous run - note this with log_result.

Be thorough - a typical Heart-kill run has:
- 1 Neow decision
- 20-30+ card rewards (including SKIPs!)
- 3-6 shop visits
- 3-5 rest sites
- 2 boss relic selections
- Multiple events
- 15-20+ combats with full card sequences for important fights"""


CHUNK_PROMPTS = {
    "neow": """Focus on the run START (first 2-3 minutes):
1. Look for the seed in pause menu or run start
2. Extract ALL 4 Neow options and which was chosen
3. Record the first combat and card reward
4. Note starting relics if whale bonus was taken""",

    "act1": """Focus on Act 1 (floors 1-16):
- Every combat: enemy, HP before/after, key card plays
- EVERY card reward including SKIPs
- Shop visits with all purchases/removals
- Rest site decisions
- Events and choices
- Elite combats in detail""",

    "boss1": """Focus on the Act 1 boss fight and transition:
- Full card sequence for boss fight
- HP tracking through the fight
- Boss relic selection (all 3 options + chosen)
- Any relics that triggered""",

    "act2": """Focus on Act 2 (floors 17-33):
- Same tracking as Act 1
- Note the increased difficulty
- Track potion usage carefully""",

    "boss2": """Focus on Act 2 boss and transition:
- Full boss fight sequence
- Boss relic selection
- State going into Act 3""",

    "act3": """Focus on Act 3 (floors 34-50):
- Key combats especially elites
- Deck should be mostly complete - note any additions
- Shop removes are critical""",

    "heart": """Focus on Act 3 boss and Heart:
- Full Act 3 boss fight sequence
- Keys obtained
- Heart fight in FULL detail - every card play
- Final result with HP"""
}


# =============================================================================
# Structured Extractor
# =============================================================================

class StructuredExtractor:
    """Extract run data using structured tool calls."""

    MODEL = "gemini-2.0-flash"  # Use stable model for tool calling

    def __init__(self, api_key: Optional[str] = None):
        import os
        self.api_key = api_key or os.environ.get("GOOGLE_API_KEY")
        if not self.api_key:
            raise ValueError("GOOGLE_API_KEY required")

        from google import genai
        self.client = genai.Client(api_key=self.api_key)

    def extract_chunk(
        self,
        video_url: str,
        chunk_prompt: str,
        state: RunState,
    ) -> list[dict]:
        """Extract from a video chunk using tool calls.

        Returns list of tool calls made.
        """
        from google.genai import types

        # Build tools config
        tools = [
            types.Tool(function_declarations=[
                types.FunctionDeclaration(
                    name=t["name"],
                    description=t["description"],
                    parameters=t["parameters"]
                )
                for t in EXTRACTION_TOOLS
            ])
        ]

        prompt = f"""{SYSTEM_PROMPT}

CURRENT RUN STATE:
- Floor: {state.floor}
- HP: {state.hp}/{state.max_hp}
- Gold: {state.gold}
- Potions: {state.potions}
- Relics: {state.relics}
- Deck size: {len(state.deck)}

FOCUS FOR THIS SEGMENT:
{chunk_prompt}

Analyze the video and call the appropriate logging tools for each decision point."""

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

        # Extract tool calls from response
        tool_calls = []
        for part in response.candidates[0].content.parts:
            if hasattr(part, 'function_call') and part.function_call:
                fc = part.function_call
                tool_calls.append({
                    "name": fc.name,
                    "args": dict(fc.args) if fc.args else {}
                })

        return tool_calls

    def apply_tool_calls(self, state: RunState, tool_calls: list[dict]):
        """Apply tool calls to update run state."""
        current_combat = None

        for call in tool_calls:
            name = call["name"]
            args = call["args"]

            if name == "log_seed":
                state.seed = args.get("seed")

            elif name == "log_neow":
                state.neow_options = args.get("options", [])
                state.neow_chosen = args.get("chosen")
                state.neow_drawback = args.get("drawback")

            elif name == "start_combat":
                state.floor = args.get("floor", state.floor)
                current_combat = {
                    "floor": state.floor,
                    "enemy": args.get("enemy", "Unknown"),
                    "before": CombatSnapshot(
                        hp=args.get("hp", state.hp),
                        max_hp=args.get("max_hp", state.max_hp),
                        gold=state.gold,
                        potions=args.get("potions", state.potions.copy()),
                        relics=state.relics.copy(),
                        deck_size=len(state.deck)
                    ),
                    "card_sequence": [],
                    "potions_used": [],
                    "timestamp_start": args.get("timestamp")
                }

            elif name == "log_card_play" and current_combat:
                current_combat["card_sequence"].append({
                    "card": args.get("card"),
                    "target": args.get("target"),
                    "turn": args.get("turn", 1),
                    "energy_after": args.get("energy_after"),
                    "timestamp": args.get("timestamp")
                })

            elif name == "log_potion_use" and current_combat:
                current_combat["potions_used"].append(args.get("potion"))

            elif name == "end_combat":
                if current_combat:
                    hp_after = args.get("hp_after", state.hp)
                    current_combat["after"] = CombatSnapshot(
                        hp=hp_after,
                        max_hp=state.max_hp,
                        gold=state.gold,
                        potions=args.get("potions_after", state.potions.copy()),
                        relics=state.relics.copy(),
                        deck_size=len(state.deck)
                    )
                    current_combat["relics_triggered"] = args.get("relics_triggered", [])
                    current_combat["turns_taken"] = args.get("turns_taken", 1)
                    current_combat["timestamp_end"] = args.get("timestamp")

                    # Create CombatRecord
                    record = CombatRecord(
                        floor=current_combat["floor"],
                        enemy=current_combat["enemy"],
                        before=current_combat["before"],
                        after=current_combat["after"],
                        card_sequence=current_combat["card_sequence"],
                        relics_triggered=current_combat["relics_triggered"],
                        potions_used=current_combat["potions_used"],
                        turns_taken=current_combat["turns_taken"],
                        timestamp_start=current_combat["timestamp_start"],
                        timestamp_end=current_combat["timestamp_end"]
                    )
                    state.combats.append(record)
                    state.hp = hp_after
                    current_combat = None

            elif name == "log_card_reward":
                reward = CardRewardRecord(
                    floor=args.get("floor", state.floor),
                    options=args.get("options", []),
                    chosen=args.get("chosen", "SKIP"),
                    singing_bowl_hp=args.get("singing_bowl", False),
                    timestamp=args.get("timestamp")
                )

                # Add to deck if not skip
                if reward.chosen != "SKIP" and reward.chosen:
                    state.deck.append(reward.chosen)

                # Create or update reward bundle
                bundle = RewardBundle(floor=reward.floor, card_reward=reward)
                state.rewards.append(bundle)

            elif name == "log_potion_reward":
                state.potions.append(args.get("potion"))

            elif name == "log_gold_reward":
                state.gold += args.get("amount", 0)

            elif name == "log_relic_reward":
                state.relics.append(args.get("relic"))

            elif name == "log_shop":
                shop = ShopRecord(
                    floor=args.get("floor", state.floor),
                    purchases=args.get("purchases", []),
                    removals=args.get("removals", []),
                    timestamp=args.get("timestamp")
                )
                state.shops.append(shop)

                # Update state
                for p in shop.purchases:
                    state.gold -= p.get("cost", 0)
                    if p.get("type") == "card":
                        state.deck.append(p.get("item"))
                    elif p.get("type") == "relic":
                        state.relics.append(p.get("item"))
                    elif p.get("type") == "potion":
                        state.potions.append(p.get("item"))

                for r in shop.removals:
                    state.gold -= r.get("cost", 0)
                    card = r.get("card")
                    if card in state.deck:
                        state.deck.remove(card)

            elif name == "log_rest":
                rest = RestRecord(
                    floor=args.get("floor", state.floor),
                    action=args.get("action", "rest"),
                    card_upgraded=args.get("card_upgraded"),
                    hp_before=args.get("hp_before"),
                    hp_after=args.get("hp_after"),
                    timestamp=args.get("timestamp")
                )
                state.rests.append(rest)

                if rest.hp_after:
                    state.hp = rest.hp_after
                elif rest.action == "rest":
                    state.hp = min(state.max_hp, state.hp + int(state.max_hp * 0.3))

            elif name == "log_boss_relic":
                state.boss_relics.append({
                    "floor": args.get("floor"),
                    "options": args.get("options", []),
                    "chosen": args.get("chosen"),
                    "timestamp": args.get("timestamp")
                })
                state.relics.append(args.get("chosen"))

            elif name == "log_event":
                event = EventRecord(
                    floor=args.get("floor", state.floor),
                    event_name=args.get("event_name", "Unknown"),
                    choice_made=args.get("choice", ""),
                    outcome=args.get("outcome", ""),
                    timestamp=args.get("timestamp")
                )
                state.events.append(event)

            elif name == "log_result":
                state.victory = args.get("victory", False)
                state.heart_kill = args.get("heart_kill", False)
                state.floor_reached = args.get("floor_reached", state.floor)
                if args.get("final_hp"):
                    state.hp = args["final_hp"]

    def extract_full_run(
        self,
        video_url: str,
        video_id: str,
        passes: int = 3,
    ) -> RunState:
        """Extract full run with multi-pass voting.

        Args:
            video_url: YouTube URL
            video_id: Video ID for saving
            passes: Number of extraction passes for voting

        Returns:
            RunState with extracted data
        """
        import re

        if video_id is None:
            match = re.search(r"(?:v=|/)([a-zA-Z0-9_-]{11})", video_url)
            video_id = match.group(1) if match else "unknown"

        print(f"Extracting {video_id} with {passes} passes...")

        # Run multiple passes
        all_calls = []
        for i in range(passes):
            print(f"  Pass {i+1}/{passes}...")
            try:
                # Single pass over full video
                calls = self.extract_chunk(
                    video_url,
                    SYSTEM_PROMPT,
                    RunState(video_id=video_id)
                )
                all_calls.append(calls)
                print(f"    Got {len(calls)} tool calls")
            except Exception as e:
                print(f"    Error: {e}")

        # Voting: merge results
        state = RunState(video_id=video_id)
        state.extraction_passes = passes

        # For now, use first successful pass
        # TODO: Implement proper voting
        if all_calls:
            self.apply_tool_calls(state, all_calls[0])

        return state

    def print_summary(self, state: RunState):
        """Print extraction summary."""
        print("\n" + "="*60)
        print(f"STRUCTURED EXTRACTION: {state.video_id}")
        print("="*60)

        if state.seed:
            print(f"Seed: {state.seed}")

        result = "WIN" if state.victory else "LOSS"
        heart = " (Heart Kill!)" if state.heart_kill else ""
        print(f"Result: {result} - Floor {state.floor_reached}{heart}")
        print(f"Final HP: {state.hp}/{state.max_hp}")

        if state.neow_chosen:
            print(f"\nNeow: {state.neow_chosen}")
            if state.neow_drawback:
                print(f"  Drawback: {state.neow_drawback}")

        print(f"\nCombats: {len(state.combats)}")
        for c in state.combats[:3]:
            hp_change = c.after.hp - c.before.hp
            sign = "+" if hp_change >= 0 else ""
            print(f"  F{c.floor} {c.enemy}: {c.before.hp}→{c.after.hp} ({sign}{hp_change})")
            if c.card_sequence:
                cards = [cs["card"] for cs in c.card_sequence[:5]]
                print(f"    Cards: {' → '.join(cards)}")
        if len(state.combats) > 3:
            print(f"  ... and {len(state.combats)-3} more")

        print(f"\nCard Rewards: {len(state.rewards)}")
        picks = [r.card_reward.chosen for r in state.rewards if r.card_reward]
        skips = picks.count("SKIP")
        print(f"  Picks: {len(picks) - skips}, Skips: {skips}")

        print(f"\nShops: {len(state.shops)}")
        print(f"Rests: {len(state.rests)}")
        print(f"Events: {len(state.events)}")
        print(f"Boss Relics: {len(state.boss_relics)}")

        print(f"\nFinal Deck ({len(state.deck)} cards):")
        print(f"  {', '.join(state.deck[:10])}")
        if len(state.deck) > 10:
            print(f"  ... and {len(state.deck)-10} more")

        print("="*60)


def extract_structured(video_url: str, passes: int = 1) -> RunState:
    """Convenience function for structured extraction."""
    from core.training.env import load_env
    load_env()

    import re
    match = re.search(r"(?:v=|/)([a-zA-Z0-9_-]{11})", video_url)
    video_id = match.group(1) if match else "unknown"

    extractor = StructuredExtractor()
    state = extractor.extract_full_run(video_url, video_id, passes=passes)
    extractor.print_summary(state)

    path = state.save()
    print(f"\nSaved to: {path}")

    return state
