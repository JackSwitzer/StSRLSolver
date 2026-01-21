"""Extract game decisions from VOD transcripts using LLM analysis."""

import json
from pathlib import Path
from typing import Optional, Any
from dataclasses import dataclass, field, asdict
from enum import Enum

from .llm_client import OpenRouterClient, get_client
from .youtube_client import VideoTranscript


class DecisionType(str, Enum):
    """Types of decisions in Slay the Spire."""
    NEOW = "neow"
    PATH = "path"
    CARD_REWARD = "card_reward"
    CARD_PLAY = "card_play"
    POTION_USE = "potion_use"
    REST_SITE = "rest_site"
    SHOP = "shop"
    EVENT = "event"
    BOSS_RELIC = "boss_relic"


@dataclass
class Decision:
    """A single game decision extracted from VOD."""
    type: DecisionType
    floor: Optional[int]
    options: list[str]
    chosen: str
    reasoning: str
    timestamp_seconds: Optional[float] = None
    context: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        d = asdict(self)
        d["type"] = self.type.value
        return d


@dataclass
class CombatLine:
    """A line of play in combat."""
    floor: int
    turn: int
    actions: list[str]  # e.g., ["Eruption", "Strike", "End Turn"]
    reasoning: str
    enemy_state: Optional[str] = None
    player_hp: Optional[int] = None
    timestamp_seconds: Optional[float] = None

    def to_dict(self) -> dict:
        return asdict(self)


@dataclass
class GameRun:
    """A full game run with all decisions."""
    video_id: str
    streamer: str
    character: str = "WATCHER"
    ascension: int = 20
    decisions: list[Decision] = field(default_factory=list)
    combat_lines: list[CombatLine] = field(default_factory=list)
    outcome: str = "unknown"  # win/loss/unknown

    def to_dict(self) -> dict:
        return {
            "video_id": self.video_id,
            "streamer": self.streamer,
            "character": self.character,
            "ascension": self.ascension,
            "outcome": self.outcome,
            "decisions": [d.to_dict() for d in self.decisions],
            "combat_lines": [c.to_dict() for c in self.combat_lines],
        }


SYSTEM_PROMPT = """You are an expert Slay the Spire analyst, specifically for the Watcher character at Ascension 20.
Your task is to extract game decisions from streamer commentary transcripts.

Key Watcher concepts:
- Stances: Calm (exit gives +2 energy), Wrath (double damage dealt/taken), Divinity (+3 energy, triple damage)
- Key cards: Eruption, Vigilance, Tantrum, Rushdown, Mental Fortress, Talk to the Hand
- Important relics: Violet Lotus, Teardrop Locket, Damaru, Duality

When extracting decisions, focus on:
1. WHY the player chose what they did (reasoning)
2. What alternatives were available
3. The game state context (floor, HP, deck state if mentioned)

Output ONLY valid JSON. No explanations outside the JSON."""


DECISION_EXTRACTION_PROMPT = """Analyze this Slay the Spire Watcher transcript segment and extract ALL game decisions mentioned.

TRANSCRIPT:
{transcript}

Extract decisions into this JSON format:
{{
  "decisions": [
    {{
      "type": "card_reward|neow|path|rest_site|shop|event|boss_relic|card_play|potion_use",
      "floor": <floor number or null>,
      "options": ["option1", "option2", ...],
      "chosen": "chosen option",
      "reasoning": "why player chose this",
      "timestamp_hint": "any time reference in transcript"
    }}
  ],
  "combat_lines": [
    {{
      "floor": <floor number>,
      "turn": <turn number>,
      "actions": ["card1", "card2", "End Turn"],
      "reasoning": "why this sequence",
      "enemy_state": "description if mentioned"
    }}
  ],
  "game_context": {{
    "current_floor": <if determinable>,
    "player_hp": <if mentioned>,
    "notable_cards": ["cards mentioned as being in deck"],
    "notable_relics": ["relics mentioned"]
  }}
}}

Focus on explicit decisions where the player explains their choice. If they don't explain, note "implicit" for reasoning.
Include card play sequences in combat_lines when the player discusses specific plays."""


CHUNK_SIZE = 4000  # Characters per chunk for analysis


class DecisionExtractor:
    """Extract game decisions from VOD transcripts using LLM."""

    def __init__(
        self,
        client: Optional[OpenRouterClient] = None,
        model: str = OpenRouterClient.GEMINI_PRO,
    ):
        """Initialize extractor.

        Args:
            client: OpenRouter client (created if not provided)
            model: Model to use for extraction
        """
        self.client = client
        self.model = model
        self._owns_client = client is None

    def _get_client(self) -> OpenRouterClient:
        """Get or create client."""
        if self.client is None:
            self.client = get_client()
        return self.client

    def extract_from_transcript(
        self,
        transcript: VideoTranscript,
        streamer: str,
    ) -> GameRun:
        """Extract all decisions from a video transcript.

        Args:
            transcript: Video transcript to analyze
            streamer: Streamer name

        Returns:
            GameRun with extracted decisions
        """
        client = self._get_client()
        run = GameRun(video_id=transcript.video_id, streamer=streamer)

        # Process transcript in chunks
        full_text = transcript.full_text
        chunks = self._chunk_text(full_text, CHUNK_SIZE)

        print(f"Processing {len(chunks)} chunks from {transcript.video_id}...")

        for i, chunk in enumerate(chunks):
            print(f"  Chunk {i+1}/{len(chunks)}...")

            try:
                response = client.complete(
                    prompt=DECISION_EXTRACTION_PROMPT.format(transcript=chunk),
                    model=self.model,
                    system=SYSTEM_PROMPT,
                    temperature=0.2,
                    json_mode=True,
                )

                data = client.extract_json(response)

                # Process decisions
                for d in data.get("decisions", []):
                    try:
                        decision = Decision(
                            type=DecisionType(d["type"]),
                            floor=d.get("floor"),
                            options=d.get("options", []),
                            chosen=d.get("chosen", ""),
                            reasoning=d.get("reasoning", ""),
                        )
                        run.decisions.append(decision)
                    except (KeyError, ValueError) as e:
                        print(f"    Skipping malformed decision: {e}")

                # Process combat lines
                for c in data.get("combat_lines", []):
                    try:
                        combat = CombatLine(
                            floor=c.get("floor", 0),
                            turn=c.get("turn", 1),
                            actions=c.get("actions", []),
                            reasoning=c.get("reasoning", ""),
                            enemy_state=c.get("enemy_state"),
                        )
                        run.combat_lines.append(combat)
                    except (KeyError, ValueError) as e:
                        print(f"    Skipping malformed combat line: {e}")

            except Exception as e:
                print(f"    Error processing chunk: {e}")
                continue

        print(f"Extracted {len(run.decisions)} decisions, {len(run.combat_lines)} combat lines")
        return run

    def extract_from_text(self, text: str, video_id: str = "manual") -> GameRun:
        """Extract decisions from raw text (for testing)."""
        # Create a fake transcript
        from .youtube_client import TranscriptSegment
        transcript = VideoTranscript(
            video_id=video_id,
            segments=[TranscriptSegment(text=text, start=0, duration=0)],
        )
        return self.extract_from_transcript(transcript, "manual")

    def _chunk_text(self, text: str, chunk_size: int) -> list[str]:
        """Split text into chunks for processing."""
        words = text.split()
        chunks = []
        current_chunk = []
        current_length = 0

        for word in words:
            word_len = len(word) + 1  # +1 for space
            if current_length + word_len > chunk_size and current_chunk:
                chunks.append(" ".join(current_chunk))
                current_chunk = []
                current_length = 0
            current_chunk.append(word)
            current_length += word_len

        if current_chunk:
            chunks.append(" ".join(current_chunk))

        return chunks

    def save_run(self, run: GameRun, output_dir: Path):
        """Save extracted run to JSON file."""
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"{run.video_id}_decisions.json"

        with open(output_path, "w") as f:
            json.dump(run.to_dict(), f, indent=2)

        print(f"Saved to {output_path}")

    def close(self):
        """Close client if we own it."""
        if self._owns_client and self.client:
            self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
