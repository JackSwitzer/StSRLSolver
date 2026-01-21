"""
Micro-chunking system for fine-grained VOD extraction.

Creates small, action-focused chunks (20-120 seconds) centered on
individual decisions. Uses transcript + video scan for boundary detection.
"""

import json
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
from pathlib import Path


class ActionType(str, Enum):
    """Types of actions/decisions to extract."""
    NEOW = "neow"
    MAP_SELECT = "map"
    COMBAT = "combat"
    CARD_REWARD = "card_reward"
    RELIC_REWARD = "relic_reward"
    POTION_REWARD = "potion_reward"
    SHOP = "shop"
    REST = "rest"
    EVENT = "event"
    BOSS_RELIC = "boss_relic"
    RESULT = "result"
    UNKNOWN = "unknown"


@dataclass
class ActionBoundary:
    """A detected action boundary from transcript/scan."""
    timestamp_start: float  # seconds
    timestamp_end: Optional[float] = None
    action_type: ActionType = ActionType.UNKNOWN
    floor: int = 0
    transcript_hint: str = ""
    confidence: float = 0.5

    @property
    def start_str(self) -> str:
        mins = int(self.timestamp_start) // 60
        secs = int(self.timestamp_start) % 60
        return f"{mins:02d}:{secs:02d}"

    @property
    def end_str(self) -> str:
        if self.timestamp_end:
            mins = int(self.timestamp_end) // 60
            secs = int(self.timestamp_end) % 60
            return f"{mins:02d}:{secs:02d}"
        return self.start_str

    def duration_seconds(self) -> float:
        if self.timestamp_end:
            return self.timestamp_end - self.timestamp_start
        return 30.0  # default


@dataclass
class MicroChunk:
    """A small, focused chunk for extracting 1-3 actions."""
    chunk_id: int
    start_seconds: float
    end_seconds: float
    expected_actions: list[ActionBoundary] = field(default_factory=list)
    floor_start: int = 0
    floor_end: int = 0

    # Context from previous chunks
    prior_context: str = ""

    @property
    def start_str(self) -> str:
        mins = int(self.start_seconds) // 60
        secs = int(self.start_seconds) % 60
        return f"{mins:02d}:{secs:02d}"

    @property
    def end_str(self) -> str:
        mins = int(self.end_seconds) // 60
        secs = int(self.end_seconds) % 60
        return f"{mins:02d}:{secs:02d}"

    @property
    def duration_seconds(self) -> float:
        return self.end_seconds - self.start_seconds

    def to_dict(self) -> dict:
        return {
            "id": self.chunk_id,
            "start": self.start_str,
            "end": self.end_str,
            "duration": self.duration_seconds,
            "floors": f"{self.floor_start}-{self.floor_end}",
            "expected_actions": [
                {"type": a.action_type.value, "time": a.start_str, "hint": a.transcript_hint}
                for a in self.expected_actions
            ]
        }


# Transcript keywords for action detection
ACTION_KEYWORDS = {
    ActionType.NEOW: ["neow", "100 gold", "whale", "bonus", "blessing", "drawback"],
    ActionType.MAP_SELECT: ["path", "elite", "rest", "shop", "event", "?", "question mark", "campfire"],
    ActionType.COMBAT: ["fight", "combat", "kill", "damage", "block", "wrath", "calm", "turn", "enemy"],
    ActionType.CARD_REWARD: ["skip", "take", "pick", "reward", "card reward", "singing bowl"],
    ActionType.SHOP: ["buy", "purchase", "remove", "shop", "merchant", "gold"],
    ActionType.REST: ["rest", "heal", "smith", "upgrade", "campfire", "dig", "lift", "recall"],
    ActionType.EVENT: ["event", "choice", "option"],
    ActionType.BOSS_RELIC: ["boss relic", "energy relic", "swap"],
    ActionType.RELIC_REWARD: ["relic", "obtained"],
    ActionType.POTION_REWARD: ["potion"],
    ActionType.RESULT: ["victory", "defeat", "won", "lost", "dead", "heart"],
}


def detect_actions_from_transcript(transcript_path: str) -> list[ActionBoundary]:
    """
    Detect action boundaries from transcript keywords.

    Returns list of potential action timestamps.
    """
    with open(transcript_path) as f:
        data = json.load(f)

    boundaries = []
    transcript = data.get("full_transcript", [])

    for item in transcript:
        text = item["text"].lower()
        start = item["start"]

        # Check each action type's keywords
        for action_type, keywords in ACTION_KEYWORDS.items():
            if any(kw in text for kw in keywords):
                boundaries.append(ActionBoundary(
                    timestamp_start=start,
                    action_type=action_type,
                    transcript_hint=item["text"],
                    confidence=0.6,  # Transcript-based detection
                ))
                break  # Only one action type per transcript segment

    # Sort by timestamp
    boundaries.sort(key=lambda b: b.timestamp_start)

    return boundaries


def merge_nearby_boundaries(
    boundaries: list[ActionBoundary],
    merge_threshold_seconds: float = 10.0,
) -> list[ActionBoundary]:
    """Merge boundaries that are very close together."""
    if not boundaries:
        return []

    merged = [boundaries[0]]

    for b in boundaries[1:]:
        last = merged[-1]

        # If same type and close together, extend the last one
        if (b.action_type == last.action_type and
            b.timestamp_start - last.timestamp_start < merge_threshold_seconds):
            last.timestamp_end = b.timestamp_start + 5  # Extend
            last.transcript_hint += f" | {b.transcript_hint}"
        else:
            merged.append(b)

    return merged


def estimate_action_duration(action_type: ActionType) -> float:
    """Estimate typical duration for each action type in seconds."""
    durations = {
        ActionType.NEOW: 30,
        ActionType.MAP_SELECT: 15,
        ActionType.COMBAT: 90,  # Varies a lot
        ActionType.CARD_REWARD: 20,
        ActionType.RELIC_REWARD: 10,
        ActionType.POTION_REWARD: 5,
        ActionType.SHOP: 45,
        ActionType.REST: 20,
        ActionType.EVENT: 40,
        ActionType.BOSS_RELIC: 30,
        ActionType.RESULT: 20,
        ActionType.UNKNOWN: 30,
    }
    return durations.get(action_type, 30)


class MicroChunker:
    """
    Creates micro-chunks for fine-grained extraction.

    Strategy:
    1. Detect action boundaries from transcript
    2. Create small chunks (20-120s) around each action
    3. Group related actions (combat -> card_reward)
    4. Add padding for context

    Usage:
        chunker = MicroChunker(transcript_path="transcript.json")
        chunks = chunker.create_chunks(video_duration_seconds=3300)
    """

    def __init__(
        self,
        transcript_path: Optional[str] = None,
        min_chunk_seconds: float = 20,
        max_chunk_seconds: float = 120,
        target_actions_per_chunk: int = 2,
        padding_seconds: float = 5,
    ):
        self.transcript_path = transcript_path
        self.min_chunk_seconds = min_chunk_seconds
        self.max_chunk_seconds = max_chunk_seconds
        self.target_actions = target_actions_per_chunk
        self.padding = padding_seconds

        self.boundaries: list[ActionBoundary] = []
        if transcript_path:
            self.boundaries = detect_actions_from_transcript(transcript_path)
            self.boundaries = merge_nearby_boundaries(self.boundaries)

    def create_chunks(
        self,
        video_duration_seconds: float,
        scan_boundaries: Optional[list[ActionBoundary]] = None,
    ) -> list[MicroChunk]:
        """
        Create micro-chunks for the video.

        Args:
            video_duration_seconds: Total video length
            scan_boundaries: Additional boundaries from video scan (higher confidence)

        Returns:
            List of MicroChunks
        """
        # Merge transcript and scan boundaries
        all_boundaries = list(self.boundaries)
        if scan_boundaries:
            all_boundaries.extend(scan_boundaries)
            all_boundaries.sort(key=lambda b: b.timestamp_start)
            all_boundaries = merge_nearby_boundaries(all_boundaries, merge_threshold_seconds=15)

        # If no boundaries detected, fall back to fixed intervals
        if not all_boundaries:
            return self._create_fixed_chunks(video_duration_seconds)

        # Create chunks around boundaries - no overlap allowed
        chunks = []
        chunk_id = 0
        i = 0
        last_chunk_end = 0.0  # Track where last chunk ended

        while i < len(all_boundaries):
            # Start a new chunk
            start_boundary = all_boundaries[i]

            # Ensure we don't start before the last chunk ended
            chunk_start = max(last_chunk_end, start_boundary.timestamp_start - self.padding)
            chunk_actions = [start_boundary]

            # Add more actions if they fit within max duration
            j = i + 1
            while j < len(all_boundaries):
                next_boundary = all_boundaries[j]
                potential_end = next_boundary.timestamp_start + estimate_action_duration(next_boundary.action_type)

                # Check if adding this action would exceed limits
                if (potential_end - chunk_start > self.max_chunk_seconds or
                    len(chunk_actions) >= self.target_actions):
                    break

                # Group combat with its card reward (always include if close)
                if (chunk_actions[-1].action_type == ActionType.COMBAT and
                    next_boundary.action_type == ActionType.CARD_REWARD and
                    next_boundary.timestamp_start - chunk_actions[-1].timestamp_start < 60):
                    chunk_actions.append(next_boundary)
                    j += 1
                    continue

                # Add if close enough and within limits
                if next_boundary.timestamp_start - chunk_start < self.max_chunk_seconds:
                    chunk_actions.append(next_boundary)
                    j += 1
                else:
                    break

            # Calculate chunk end
            last_action = chunk_actions[-1]
            chunk_end = min(
                video_duration_seconds,
                last_action.timestamp_start + estimate_action_duration(last_action.action_type) + self.padding
            )

            # Ensure minimum duration
            if chunk_end - chunk_start < self.min_chunk_seconds:
                chunk_end = min(video_duration_seconds, chunk_start + self.min_chunk_seconds)

            chunks.append(MicroChunk(
                chunk_id=chunk_id,
                start_seconds=chunk_start,
                end_seconds=chunk_end,
                expected_actions=chunk_actions,
                floor_start=chunk_actions[0].floor,
                floor_end=chunk_actions[-1].floor,
            ))

            chunk_id += 1
            last_chunk_end = chunk_end  # Update for next iteration
            i = j  # Move to next unprocessed boundary

        return chunks

    def _create_fixed_chunks(self, video_duration_seconds: float) -> list[MicroChunk]:
        """Fallback: create fixed-duration chunks."""
        chunks = []
        chunk_duration = 60  # 1 minute chunks

        start = 0
        chunk_id = 0
        while start < video_duration_seconds:
            end = min(start + chunk_duration, video_duration_seconds)
            chunks.append(MicroChunk(
                chunk_id=chunk_id,
                start_seconds=start,
                end_seconds=end,
            ))
            start = end
            chunk_id += 1

        return chunks


def create_micro_chunk_prompt(
    chunk: MicroChunk,
    state_context: dict,
    extraction_format: str = "structured",
) -> str:
    """
    Create extraction prompt for a micro-chunk.

    Args:
        chunk: The MicroChunk to extract
        state_context: Current game state (hp, deck, etc.)
        extraction_format: "structured" for tool calls, "text" for free text
    """
    # Build expected actions hint
    action_hints = ""
    if chunk.expected_actions:
        action_hints = "\nEXPECTED ACTIONS IN THIS SEGMENT:\n"
        for a in chunk.expected_actions:
            action_hints += f"- ~{a.start_str}: {a.action_type.value}"
            if a.transcript_hint:
                action_hints += f' (player said: "{a.transcript_hint[:50]}")'
            action_hints += "\n"

    prompt = f"""Watch from {chunk.start_str} to {chunk.end_str} of this Slay the Spire Watcher run.

CURRENT STATE:
- HP: {state_context.get('hp', '?')}/{state_context.get('max_hp', '?')}
- Gold: {state_context.get('gold', '?')}
- Floor: {state_context.get('floor', '?')}
- Deck: {state_context.get('deck_size', '?')} cards
- Relics: {', '.join(state_context.get('relics', [])) or 'Pure Water (starting relic)'}
{action_hints}
IMPORTANT: Call the appropriate function for EACH decision you observe in this segment.

For Neow (whale room at start): call neow() with the chosen bonus
For path selections: call path() with floor, options array, and chosen
For combat start: call combat_start() with floor and enemy name
For combat turns: call combat_turn() with turn number and cards played
For combat end: call combat_end() with remaining HP and gold gained
For card rewards: call card_reward() with options array and chosen (or "SKIP")
For shops: call shop() with purchases and removals
For rest sites: call rest() with action (rest/smith/dig/recall)

Watch the video segment carefully. Call one function for each decision point you observe.
The enemies called "Louse" in the game should be recorded as "Lice" (plural) or "Louse" (singular)."""

    return prompt


# Quick video scan prompt for boundary detection
BOUNDARY_SCAN_PROMPT = """Quickly scan this video segment and identify decision timestamps.

For each major decision/action, output ONE LINE:
[MM:SS] action_type | brief description

Action types: neow, map, combat_start, combat_end, card_reward, shop, rest, event, boss_relic, result

Example output:
[00:10] neow | chose 100 gold
[00:28] map | picked monster
[00:30] combat_start | vs 2 Lice
[01:12] combat_end | won at 62 hp
[01:15] card_reward | skipped
[01:20] map | picked shop

List ALL decisions chronologically. Be fast - just timestamps and types."""


def parse_boundary_scan(response_text: str) -> list[ActionBoundary]:
    """Parse the boundary scan response into ActionBoundary objects."""
    boundaries = []

    for line in response_text.strip().split('\n'):
        line = line.strip()
        if not line or not line.startswith('['):
            continue

        try:
            # Parse [MM:SS] action_type | description
            time_part = line[1:line.index(']')]
            rest = line[line.index(']')+1:].strip()

            # Parse timestamp
            parts = time_part.split(':')
            if len(parts) == 2:
                seconds = int(parts[0]) * 60 + int(parts[1])
            else:
                continue

            # Parse action type
            if '|' in rest:
                action_str, desc = rest.split('|', 1)
            else:
                action_str, desc = rest, ""

            action_str = action_str.strip().lower().replace(' ', '_')

            # Map to ActionType
            action_map = {
                'neow': ActionType.NEOW,
                'map': ActionType.MAP_SELECT,
                'path': ActionType.MAP_SELECT,
                'combat_start': ActionType.COMBAT,
                'combat': ActionType.COMBAT,
                'combat_end': ActionType.COMBAT,
                'card_reward': ActionType.CARD_REWARD,
                'reward': ActionType.CARD_REWARD,
                'shop': ActionType.SHOP,
                'rest': ActionType.REST,
                'event': ActionType.EVENT,
                'boss_relic': ActionType.BOSS_RELIC,
                'relic': ActionType.RELIC_REWARD,
                'result': ActionType.RESULT,
            }

            action_type = action_map.get(action_str, ActionType.UNKNOWN)

            boundaries.append(ActionBoundary(
                timestamp_start=seconds,
                action_type=action_type,
                transcript_hint=desc.strip(),
                confidence=0.8,  # Video scan is higher confidence
            ))

        except (ValueError, IndexError):
            continue

    return boundaries
