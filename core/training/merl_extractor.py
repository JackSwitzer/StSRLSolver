"""High-fidelity decision extraction from Merl's Watcher VODs using Gemini 3.0 Flash."""

import os
import json
from pathlib import Path
from dataclasses import dataclass, field, asdict
from typing import Optional
from datetime import datetime


@dataclass
class NeowDecision:
    """Neow bonus selection at run start."""
    bonus_chosen: str
    bonus_options: list[str]
    drawback: Optional[str] = None
    timestamp: Optional[str] = None


@dataclass
class CardReward:
    """Card reward decision after combat."""
    floor: int
    cards_offered: list[str]
    card_picked: str  # or "SKIP"
    reasoning: Optional[str] = None
    timestamp: Optional[str] = None


@dataclass
class ShopDecision:
    """Shop purchase/remove decision."""
    floor: int
    action: str  # "buy", "remove", "skip"
    item: Optional[str] = None
    gold_spent: Optional[int] = None
    timestamp: Optional[str] = None


@dataclass
class RestSiteDecision:
    """Rest site decision."""
    floor: int
    action: str  # "rest", "smith", "lift", "dig", "recall"
    card_upgraded: Optional[str] = None
    timestamp: Optional[str] = None


@dataclass
class BossRelicDecision:
    """Boss relic selection."""
    floor: int  # 17, 34, 51
    relics_offered: list[str]
    relic_picked: str
    timestamp: Optional[str] = None


@dataclass
class PathDecision:
    """Map pathing decision."""
    floor: int
    path_taken: str  # description like "elite", "shop", "rest", "?"
    reasoning: Optional[str] = None
    timestamp: Optional[str] = None


@dataclass
class CombatHighlight:
    """Notable combat sequence."""
    floor: int
    enemy: str
    turn: int
    actions: list[str]
    stance_changes: list[str]
    reasoning: Optional[str] = None
    timestamp: Optional[str] = None


@dataclass
class RunResult:
    """Final run outcome."""
    victory: bool
    floor_reached: int
    final_hp: Optional[int] = None
    heart_kill: bool = False
    timestamp_start: Optional[str] = None  # Victory screen might be at video start
    timestamp_end: Optional[str] = None


@dataclass
class MerlRun:
    """Complete extracted run from a Merl VOD."""
    video_id: str
    video_url: str
    extracted_at: str = field(default_factory=lambda: datetime.now().isoformat())

    result: Optional[RunResult] = None
    neow: Optional[NeowDecision] = None
    card_rewards: list[CardReward] = field(default_factory=list)
    shop_decisions: list[ShopDecision] = field(default_factory=list)
    rest_decisions: list[RestSiteDecision] = field(default_factory=list)
    boss_relics: list[BossRelicDecision] = field(default_factory=list)
    path_decisions: list[PathDecision] = field(default_factory=list)
    combat_highlights: list[CombatHighlight] = field(default_factory=list)

    # Raw data for verification
    raw_response: Optional[str] = None

    def to_dict(self) -> dict:
        return {
            "video_id": self.video_id,
            "video_url": self.video_url,
            "extracted_at": self.extracted_at,
            "result": asdict(self.result) if self.result else None,
            "neow": asdict(self.neow) if self.neow else None,
            "card_rewards": [asdict(c) for c in self.card_rewards],
            "shop_decisions": [asdict(s) for s in self.shop_decisions],
            "rest_decisions": [asdict(r) for r in self.rest_decisions],
            "boss_relics": [asdict(b) for b in self.boss_relics],
            "path_decisions": [asdict(p) for p in self.path_decisions],
            "combat_highlights": [asdict(c) for c in self.combat_highlights],
        }

    def save(self, output_dir: Path = Path("data/merl_extracted")):
        """Save to JSON file."""
        output_dir.mkdir(parents=True, exist_ok=True)
        path = output_dir / f"{self.video_id}_run.json"
        with open(path, "w") as f:
            json.dump(self.to_dict(), f, indent=2)
        return path


# Extraction prompt - optimized for Gemini 3.0 Flash
EXTRACTION_PROMPT = '''Analyze this Slay the Spire Watcher gameplay video from streamer Merl61.

IMPORTANT:
- The video may show a VICTORY screen at the very START (from a previous run) - note this!
- Look for the "W" or victory screen showing the run was won
- Extract ALL decisions with timestamps (MM:SS format)

Extract the following in JSON format:

{
  "result": {
    "victory": true/false,
    "floor_reached": <number>,
    "final_hp": <number or null>,
    "heart_kill": true/false,
    "timestamp_start": "MM:SS" if victory shown at start,
    "timestamp_end": "MM:SS" if victory shown at end
  },

  "neow": {
    "bonus_chosen": "description of bonus taken",
    "bonus_options": ["option1", "option2", "option3", "option4"],
    "drawback": "drawback if any",
    "timestamp": "MM:SS"
  },

  "card_rewards": [
    {
      "floor": <number>,
      "cards_offered": ["Card1", "Card2", "Card3"],
      "card_picked": "CardName or SKIP",
      "reasoning": "why picked if mentioned",
      "timestamp": "MM:SS"
    }
  ],

  "shop_decisions": [
    {
      "floor": <number>,
      "action": "buy/remove/skip",
      "item": "item name if bought/removed",
      "gold_spent": <number>,
      "timestamp": "MM:SS"
    }
  ],

  "rest_decisions": [
    {
      "floor": <number>,
      "action": "rest/smith/lift/dig/recall",
      "card_upgraded": "CardName if smith",
      "timestamp": "MM:SS"
    }
  ],

  "boss_relics": [
    {
      "floor": 17/34/51,
      "relics_offered": ["Relic1", "Relic2", "Relic3"],
      "relic_picked": "RelicName",
      "timestamp": "MM:SS"
    }
  ],

  "path_decisions": [
    {
      "floor": <number>,
      "path_taken": "elite/shop/rest/?/monster",
      "reasoning": "why if mentioned",
      "timestamp": "MM:SS"
    }
  ],

  "combat_highlights": [
    {
      "floor": <number>,
      "enemy": "Enemy name",
      "turn": <number>,
      "actions": ["Card1", "Card2", "End Turn"],
      "stance_changes": ["Wrath", "Calm"],
      "reasoning": "strategy if mentioned",
      "timestamp": "MM:SS"
    }
  ]
}

Focus on:
1. EVERY card reward decision - INCLUDING SKIPS! If Merl skips a card reward, record card_picked as "SKIP". This is critical for training.
2. Neow selection with all 4 options if visible
3. Shop removes (very important for Watcher)
4. Rest vs upgrade decisions
5. Notable combat lines especially against elites/bosses

IMPORTANT: A 56-floor Heart kill run should have ~20-30 card rewards. If you only find 7-10, you're missing skips!

Be thorough - extract ALL decisions visible in the video.'''


class MerlExtractor:
    """Extract decisions from Merl VODs using Gemini 3.0 Flash."""

    MODEL = "gemini-3-flash-preview"  # Latest and best for video

    def __init__(self, api_key: Optional[str] = None):
        """Initialize with Google API key."""
        self.api_key = api_key or os.environ.get("GOOGLE_API_KEY")
        if not self.api_key:
            raise ValueError("GOOGLE_API_KEY required")

        from google import genai
        self.client = genai.Client(api_key=self.api_key)

    def extract_from_video(
        self,
        video_url: str,
        video_id: Optional[str] = None,
    ) -> MerlRun:
        """Extract all decisions from a Merl VOD.

        Args:
            video_url: YouTube URL
            video_id: Optional video ID (extracted from URL if not provided)

        Returns:
            MerlRun with all extracted decisions
        """
        import re

        if video_id is None:
            match = re.search(r"(?:v=|/)([a-zA-Z0-9_-]{11})", video_url)
            video_id = match.group(1) if match else "unknown"

        print(f"Extracting from {video_id}...")
        print(f"Using model: {self.MODEL}")
        print("This may take 1-2 minutes for video analysis...")

        response = self.client.models.generate_content(
            model=self.MODEL,
            contents=[video_url, EXTRACTION_PROMPT],
        )

        raw_text = response.text
        print(f"Got response ({len(raw_text)} chars)")

        # Parse JSON from response
        run = MerlRun(video_id=video_id, video_url=video_url, raw_response=raw_text)

        try:
            # Extract JSON from markdown code block if present
            json_text = raw_text
            if "```json" in json_text:
                json_text = json_text.split("```json")[1].split("```")[0]
            elif "```" in json_text:
                json_text = json_text.split("```")[1].split("```")[0]

            data = json.loads(json_text)

            # Parse result
            if data.get("result"):
                r = data["result"]
                run.result = RunResult(
                    victory=r.get("victory", False),
                    floor_reached=r.get("floor_reached", 0),
                    final_hp=r.get("final_hp"),
                    heart_kill=r.get("heart_kill", False),
                    timestamp_start=r.get("timestamp_start"),
                    timestamp_end=r.get("timestamp_end"),
                )

            # Parse neow
            if data.get("neow"):
                n = data["neow"]
                run.neow = NeowDecision(
                    bonus_chosen=n.get("bonus_chosen", ""),
                    bonus_options=n.get("bonus_options", []),
                    drawback=n.get("drawback"),
                    timestamp=n.get("timestamp"),
                )

            # Parse card rewards
            for cr in data.get("card_rewards", []):
                run.card_rewards.append(CardReward(
                    floor=cr.get("floor", 0),
                    cards_offered=cr.get("cards_offered", []),
                    card_picked=cr.get("card_picked", ""),
                    reasoning=cr.get("reasoning"),
                    timestamp=cr.get("timestamp"),
                ))

            # Parse shop decisions
            for sd in data.get("shop_decisions", []):
                run.shop_decisions.append(ShopDecision(
                    floor=sd.get("floor", 0),
                    action=sd.get("action", ""),
                    item=sd.get("item"),
                    gold_spent=sd.get("gold_spent"),
                    timestamp=sd.get("timestamp"),
                ))

            # Parse rest decisions
            for rd in data.get("rest_decisions", []):
                run.rest_decisions.append(RestSiteDecision(
                    floor=rd.get("floor", 0),
                    action=rd.get("action", ""),
                    card_upgraded=rd.get("card_upgraded"),
                    timestamp=rd.get("timestamp"),
                ))

            # Parse boss relics
            for br in data.get("boss_relics", []):
                run.boss_relics.append(BossRelicDecision(
                    floor=br.get("floor", 0),
                    relics_offered=br.get("relics_offered", []),
                    relic_picked=br.get("relic_picked", ""),
                    timestamp=br.get("timestamp"),
                ))

            # Parse path decisions
            for pd in data.get("path_decisions", []):
                run.path_decisions.append(PathDecision(
                    floor=pd.get("floor", 0),
                    path_taken=pd.get("path_taken", ""),
                    reasoning=pd.get("reasoning"),
                    timestamp=pd.get("timestamp"),
                ))

            # Parse combat highlights
            for ch in data.get("combat_highlights", []):
                run.combat_highlights.append(CombatHighlight(
                    floor=ch.get("floor", 0),
                    enemy=ch.get("enemy", ""),
                    turn=ch.get("turn", 1),
                    actions=ch.get("actions", []),
                    stance_changes=ch.get("stance_changes", []),
                    reasoning=ch.get("reasoning"),
                    timestamp=ch.get("timestamp"),
                ))

        except json.JSONDecodeError as e:
            print(f"Warning: Failed to parse JSON: {e}")
            print("Raw response saved for debugging")

        return run

    def print_summary(self, run: MerlRun):
        """Print a summary of extracted decisions."""
        print("\n" + "="*60)
        print(f"EXTRACTION SUMMARY: {run.video_id}")
        print("="*60)

        if run.result:
            status = "WIN" if run.result.victory else "LOSS"
            heart = " (Heart Kill!)" if run.result.heart_kill else ""
            print(f"Result: {status} - Floor {run.result.floor_reached}{heart}")
            if run.result.timestamp_start:
                print(f"  Victory shown at start: {run.result.timestamp_start}")
            if run.result.timestamp_end:
                print(f"  Victory shown at end: {run.result.timestamp_end}")

        if run.neow:
            print(f"\nNeow ({run.neow.timestamp}):")
            print(f"  Chose: {run.neow.bonus_chosen}")
            if run.neow.drawback:
                print(f"  Drawback: {run.neow.drawback}")

        print(f"\nCard Rewards: {len(run.card_rewards)}")
        for cr in run.card_rewards[:5]:  # Show first 5
            print(f"  F{cr.floor}: {cr.card_picked} from {cr.cards_offered}")
        if len(run.card_rewards) > 5:
            print(f"  ... and {len(run.card_rewards) - 5} more")

        print(f"\nShop Decisions: {len(run.shop_decisions)}")
        for sd in run.shop_decisions:
            print(f"  F{sd.floor}: {sd.action} {sd.item or ''}")

        print(f"\nRest Decisions: {len(run.rest_decisions)}")
        for rd in run.rest_decisions:
            print(f"  F{rd.floor}: {rd.action} {rd.card_upgraded or ''}")

        print(f"\nBoss Relics: {len(run.boss_relics)}")
        for br in run.boss_relics:
            print(f"  F{br.floor}: {br.relic_picked}")

        print(f"\nCombat Highlights: {len(run.combat_highlights)}")
        print("="*60)


def extract_merl_video(video_url: str) -> MerlRun:
    """Convenience function to extract from a Merl video."""
    from core.training.env import load_env
    load_env()

    extractor = MerlExtractor()
    run = extractor.extract_from_video(video_url)
    extractor.print_summary(run)

    path = run.save()
    print(f"\nSaved to: {path}")

    return run
