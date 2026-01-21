"""
Video chunking by decision boundaries.

Supports both static (floor-based) and dynamic (scan-first) chunking.
Dynamic chunking does a quick scan pass to identify actual event boundaries,
then creates chunks aligned with those boundaries.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional, Any


class ChunkType(str, Enum):
    """Types of video chunks based on game progression."""
    # Floor-based (static)
    NEOW = "neow"
    ACT1_EARLY = "act1_early"
    ACT1_MID = "act1_mid"
    ACT1_BOSS = "act1_boss"
    ACT2_EARLY = "act2_early"
    ACT2_MID = "act2_mid"
    ACT2_BOSS = "act2_boss"
    ACT3_EARLY = "act3_early"
    ACT3_MID = "act3_mid"
    ACT3_BOSS = "act3_boss"
    ACT4 = "act4"
    HEART = "heart"

    # Event-based (dynamic)
    COMBAT = "combat"
    ELITE = "elite"
    BOSS = "boss"
    SHOP = "shop"
    REST = "rest"
    EVENT = "event"
    CHEST = "chest"
    MAP_SCREEN = "map"


@dataclass
class DetectedEvent:
    """An event detected during the scan pass."""
    event_type: str  # combat, elite, boss, shop, rest, event, chest, neow, boss_relic, result
    floor: int
    timestamp_start: str
    timestamp_end: Optional[str] = None
    enemy: Optional[str] = None  # For combats
    details: Optional[dict] = None

    def duration_seconds(self) -> float:
        """Calculate duration in seconds."""
        if not self.timestamp_end:
            return 60.0  # Default estimate
        start = _parse_timestamp_to_seconds(self.timestamp_start)
        end = _parse_timestamp_to_seconds(self.timestamp_end)
        return max(end - start, 10.0)


@dataclass
class Chunk:
    """A segment of video to extract."""
    chunk_type: ChunkType
    floor_start: int
    floor_end: int
    act: int
    start_time: Optional[str] = None
    end_time: Optional[str] = None
    duration_estimate_minutes: float = 5.0

    # State snapshot to pass to LLM as context
    state_snapshot: Optional[dict] = None

    # Events in this chunk (from dynamic chunking)
    events: list[DetectedEvent] = field(default_factory=list)

    def __repr__(self) -> str:
        time_info = f" {self.start_time}-{self.end_time}" if self.start_time else ""
        return f"Chunk({self.chunk_type.value}, floors {self.floor_start}-{self.floor_end}{time_info})"

    def to_dict(self) -> dict:
        return {
            "type": self.chunk_type.value,
            "floor_start": self.floor_start,
            "floor_end": self.floor_end,
            "act": self.act,
            "start_time": self.start_time,
            "end_time": self.end_time,
            "duration_estimate": self.duration_estimate_minutes,
            "events": [
                {
                    "type": e.event_type,
                    "floor": e.floor,
                    "start": e.timestamp_start,
                    "end": e.timestamp_end,
                }
                for e in self.events
            ] if self.events else [],
        }


@dataclass
class ScanResult:
    """Result of the scan pass identifying major events."""
    events: list[DetectedEvent]
    final_floor: int
    victory: bool
    heart_kill: bool
    total_duration: str
    seed: Optional[str] = None

    def get_events_by_type(self, event_type: str) -> list[DetectedEvent]:
        return [e for e in self.events if e.event_type == event_type]

    def get_act_boundaries(self) -> dict[int, tuple[str, str]]:
        """Get timestamp boundaries for each act."""
        boundaries = {}
        boss_floors = {16: 1, 33: 2, 50: 3, 56: 4}

        for event in self.events:
            if event.event_type == "boss" and event.floor in boss_floors:
                act = boss_floors[event.floor]
                if act not in boundaries:
                    boundaries[act] = (event.timestamp_start, event.timestamp_end or event.timestamp_start)
                else:
                    boundaries[act] = (boundaries[act][0], event.timestamp_end or event.timestamp_start)

        return boundaries


# Default chunk definitions based on typical game progression
DEFAULT_CHUNKS: list[Chunk] = [
    Chunk(ChunkType.NEOW, floor_start=0, floor_end=1, act=1, duration_estimate_minutes=3.0),
    Chunk(ChunkType.ACT1_EARLY, floor_start=2, floor_end=8, act=1, duration_estimate_minutes=8.0),
    Chunk(ChunkType.ACT1_MID, floor_start=9, floor_end=15, act=1, duration_estimate_minutes=7.0),
    Chunk(ChunkType.ACT1_BOSS, floor_start=16, floor_end=17, act=1, duration_estimate_minutes=5.0),
    Chunk(ChunkType.ACT2_EARLY, floor_start=18, floor_end=25, act=2, duration_estimate_minutes=8.0),
    Chunk(ChunkType.ACT2_MID, floor_start=26, floor_end=32, act=2, duration_estimate_minutes=7.0),
    Chunk(ChunkType.ACT2_BOSS, floor_start=33, floor_end=34, act=2, duration_estimate_minutes=3.0),
    Chunk(ChunkType.ACT3_EARLY, floor_start=35, floor_end=42, act=3, duration_estimate_minutes=7.0),
    Chunk(ChunkType.ACT3_MID, floor_start=43, floor_end=50, act=3, duration_estimate_minutes=6.0),
    Chunk(ChunkType.ACT3_BOSS, floor_start=51, floor_end=52, act=3, duration_estimate_minutes=4.0),
    Chunk(ChunkType.ACT4, floor_start=53, floor_end=55, act=4, duration_estimate_minutes=4.0),
    Chunk(ChunkType.HEART, floor_start=56, floor_end=56, act=4, duration_estimate_minutes=6.0),
]


# Scan pass prompt - fast, high-level event detection
SCAN_PROMPT = """Quickly scan this Slay the Spire video and identify ALL major events with timestamps.

For each event, call the `scan_event` tool with:
- event_type: One of: neow, combat, elite, boss, shop, rest, event, chest, boss_relic, result
- floor: Floor number (0 for Neow, 1-56 for the run)
- timestamp_start: When the event begins (MM:SS)
- timestamp_end: When the event ends (MM:SS)
- enemy: For combat/elite/boss, the enemy name(s)
- details: Any notable details (optional)

IMPORTANT:
1. Identify EVERY floor transition and major decision
2. Note act boss fights (floors 16, 33, 50) and boss relic selections
3. Note the final result (victory/defeat) and final floor
4. If you see the seed, note it with a `set_seed` call
5. Be fast - we just need timestamps, not detailed analysis

Start from the beginning of the video and work chronologically."""


SCAN_TOOL = {
    "name": "scan_event",
    "description": "Record a detected event during the scan pass",
    "parameters": {
        "type": "object",
        "properties": {
            "event_type": {
                "type": "string",
                "enum": ["neow", "combat", "elite", "boss", "shop", "rest", "event", "chest", "boss_relic", "result"],
                "description": "Type of event"
            },
            "floor": {
                "type": "integer",
                "description": "Floor number (0-56)"
            },
            "timestamp_start": {
                "type": "string",
                "description": "Start time (MM:SS)"
            },
            "timestamp_end": {
                "type": "string",
                "description": "End time (MM:SS)"
            },
            "enemy": {
                "type": "string",
                "description": "Enemy name for combat events"
            },
            "details": {
                "type": "object",
                "description": "Additional details (victory, seed, etc.)"
            }
        },
        "required": ["event_type", "floor", "timestamp_start"]
    }
}


def _parse_timestamp_to_seconds(timestamp: str) -> float:
    """Parse MM:SS or HH:MM:SS to seconds."""
    parts = timestamp.split(":")
    if len(parts) == 2:
        return int(parts[0]) * 60 + int(parts[1])
    elif len(parts) == 3:
        return int(parts[0]) * 3600 + int(parts[1]) * 60 + int(parts[2])
    return 0.0


def _format_timestamp(seconds: float) -> str:
    """Format seconds to MM:SS."""
    mins = int(seconds) // 60
    secs = int(seconds) % 60
    return f"{mins:02d}:{secs:02d}"


@dataclass
class DynamicChunker:
    """
    Creates chunks dynamically based on a scan pass.

    Usage:
        chunker = DynamicChunker()
        scan_result = chunker.scan(video_path, extractor)
        chunks = chunker.create_chunks(scan_result)
    """

    target_chunk_minutes: float = 6.0
    min_chunk_minutes: float = 2.0
    max_chunk_minutes: float = 10.0

    def scan(self, video_path: str, extractor: Any) -> ScanResult:
        """
        Run a quick scan pass to identify major events.

        Args:
            video_path: Path to video file
            extractor: GeminiExtractor instance

        Returns:
            ScanResult with detected events
        """
        # Use the scan tool
        from vod.extractor import ExtractionResult

        result = extractor.extract_from_video(
            video_path=video_path,
            prompt=SCAN_PROMPT,
        )

        return self._parse_scan_result(result.tool_calls)

    def _parse_scan_result(self, tool_calls: list[dict]) -> ScanResult:
        """Parse scan pass tool calls into ScanResult."""
        events = []
        final_floor = 0
        victory = False
        heart_kill = False
        total_duration = "00:00"
        seed = None

        for call in tool_calls:
            name = call.get("name", "")
            args = call.get("arguments", {})

            if name == "scan_event":
                event = DetectedEvent(
                    event_type=args.get("event_type", ""),
                    floor=args.get("floor", 0),
                    timestamp_start=args.get("timestamp_start", "00:00"),
                    timestamp_end=args.get("timestamp_end"),
                    enemy=args.get("enemy"),
                    details=args.get("details"),
                )
                events.append(event)

                # Track final floor
                if event.floor > final_floor:
                    final_floor = event.floor

                # Track result
                if event.event_type == "result":
                    details = event.details or {}
                    victory = details.get("victory", False)
                    heart_kill = details.get("heart_kill", False)

                # Update total duration
                if event.timestamp_end:
                    total_duration = event.timestamp_end

            elif name == "set_seed":
                seed = args.get("seed")

        # Sort by timestamp
        events.sort(key=lambda e: _parse_timestamp_to_seconds(e.timestamp_start))

        return ScanResult(
            events=events,
            final_floor=final_floor,
            victory=victory,
            heart_kill=heart_kill,
            total_duration=total_duration,
            seed=seed,
        )

    def create_chunks(self, scan_result: ScanResult) -> list[Chunk]:
        """
        Create optimized chunks based on scan results.

        Groups events into chunks of appropriate duration,
        keeping related events together (e.g., combat + reward).
        """
        if not scan_result.events:
            # Fall back to static chunking
            return VideoChunker().get_chunks(scan_result.final_floor)

        chunks = []
        current_events: list[DetectedEvent] = []
        current_start = "00:00"
        current_floor_start = 0
        current_act = 1

        for event in scan_result.events:
            event_seconds = _parse_timestamp_to_seconds(event.timestamp_start)
            current_seconds = _parse_timestamp_to_seconds(current_start)
            chunk_duration = (event_seconds - current_seconds) / 60

            # Determine act from floor
            event_act = self._floor_to_act(event.floor)

            # Check if we should start a new chunk
            should_split = (
                # Duration exceeded
                chunk_duration >= self.max_chunk_minutes or
                # Act boundary (always split at act transitions)
                (event_act != current_act and current_events) or
                # Boss fight (always its own chunk)
                event.event_type == "boss"
            )

            if should_split and current_events:
                # Finalize current chunk
                chunk = self._create_chunk_from_events(
                    current_events,
                    current_floor_start,
                    current_act,
                    current_start,
                )
                chunks.append(chunk)

                # Start new chunk
                current_events = []
                current_start = event.timestamp_start
                current_floor_start = event.floor
                current_act = event_act

            current_events.append(event)

        # Finalize last chunk
        if current_events:
            chunk = self._create_chunk_from_events(
                current_events,
                current_floor_start,
                current_act,
                current_start,
            )
            chunks.append(chunk)

        return chunks

    def _create_chunk_from_events(
        self,
        events: list[DetectedEvent],
        floor_start: int,
        act: int,
        start_time: str,
    ) -> Chunk:
        """Create a chunk from a group of events."""
        floor_end = max(e.floor for e in events)

        # Determine end time
        last_event = events[-1]
        end_time = last_event.timestamp_end or last_event.timestamp_start

        # Calculate duration
        start_sec = _parse_timestamp_to_seconds(start_time)
        end_sec = _parse_timestamp_to_seconds(end_time)
        duration_min = (end_sec - start_sec) / 60

        # Determine chunk type based on content
        chunk_type = self._determine_chunk_type(events, floor_start, floor_end, act)

        return Chunk(
            chunk_type=chunk_type,
            floor_start=floor_start,
            floor_end=floor_end,
            act=act,
            start_time=start_time,
            end_time=end_time,
            duration_estimate_minutes=duration_min,
            events=events,
        )

    def _determine_chunk_type(
        self,
        events: list[DetectedEvent],
        floor_start: int,
        floor_end: int,
        act: int,
    ) -> ChunkType:
        """Determine the appropriate chunk type based on events."""
        event_types = {e.event_type for e in events}

        # Special cases
        if "neow" in event_types or floor_start == 0:
            return ChunkType.NEOW
        if "boss" in event_types:
            if floor_end == 56:
                return ChunkType.HEART
            return {1: ChunkType.ACT1_BOSS, 2: ChunkType.ACT2_BOSS, 3: ChunkType.ACT3_BOSS}.get(act, ChunkType.BOSS)

        # Event-type based
        if len(event_types) == 1:
            single_type = list(event_types)[0]
            type_map = {
                "combat": ChunkType.COMBAT,
                "elite": ChunkType.ELITE,
                "shop": ChunkType.SHOP,
                "rest": ChunkType.REST,
                "event": ChunkType.EVENT,
                "chest": ChunkType.CHEST,
            }
            if single_type in type_map:
                return type_map[single_type]

        # Floor-based fallback
        if act == 1:
            return ChunkType.ACT1_EARLY if floor_end <= 8 else ChunkType.ACT1_MID
        elif act == 2:
            return ChunkType.ACT2_EARLY if floor_end <= 25 else ChunkType.ACT2_MID
        elif act == 3:
            return ChunkType.ACT3_EARLY if floor_end <= 42 else ChunkType.ACT3_MID
        else:
            return ChunkType.ACT4

    def _floor_to_act(self, floor: int) -> int:
        """Convert floor number to act."""
        if floor <= 17:
            return 1
        elif floor <= 34:
            return 2
        elif floor <= 52:
            return 3
        else:
            return 4


@dataclass
class VideoChunker:
    """
    Static chunker - segments VOD into fixed floor-based chunks.

    Use DynamicChunker for scan-first approach.
    """

    video_duration_minutes: float = 60.0
    target_chunk_minutes: float = 8.0
    min_chunk_minutes: float = 3.0
    max_chunk_minutes: float = 12.0

    def get_chunks(self, final_floor: int = 56) -> list[Chunk]:
        """Get chunk definitions up to the final floor reached."""
        chunks = []

        for template in DEFAULT_CHUNKS:
            if template.floor_start > final_floor:
                break

            chunk = Chunk(
                chunk_type=template.chunk_type,
                floor_start=template.floor_start,
                floor_end=min(template.floor_end, final_floor),
                act=template.act,
                duration_estimate_minutes=template.duration_estimate_minutes,
            )
            chunks.append(chunk)

        return chunks

    def estimate_timestamps(
        self,
        chunks: list[Chunk],
        total_duration: Optional[str] = None,
    ) -> list[Chunk]:
        """Estimate start/end timestamps for each chunk."""
        if total_duration:
            total_minutes = _parse_timestamp_to_seconds(total_duration) / 60
        else:
            total_minutes = self.video_duration_minutes

        total_estimated = sum(c.duration_estimate_minutes for c in chunks)
        scale = total_minutes / total_estimated if total_estimated > 0 else 1.0

        current_time = 0.0
        for chunk in chunks:
            chunk.start_time = _format_timestamp(current_time * 60)
            chunk_duration = chunk.duration_estimate_minutes * scale
            current_time += chunk_duration
            chunk.end_time = _format_timestamp(current_time * 60)

        return chunks

    def get_chunk_for_floor(self, floor: int) -> Optional[Chunk]:
        """Get the chunk that contains a specific floor."""
        for chunk in DEFAULT_CHUNKS:
            if chunk.floor_start <= floor <= chunk.floor_end:
                return Chunk(
                    chunk_type=chunk.chunk_type,
                    floor_start=chunk.floor_start,
                    floor_end=chunk.floor_end,
                    act=chunk.act,
                    duration_estimate_minutes=chunk.duration_estimate_minutes,
                )
        return None


def create_chunk_prompt(chunk: Chunk, state_snapshot: dict) -> str:
    """Create extraction prompt for a specific chunk."""
    # Time range hint if available
    time_hint = ""
    if chunk.start_time and chunk.end_time:
        time_hint = f"\nFocus on the video segment from {chunk.start_time} to {chunk.end_time}.\n"

    # Event hints from scan pass
    event_hints = ""
    if chunk.events:
        event_hints = "\nDETECTED EVENTS IN THIS SEGMENT:\n"
        for e in chunk.events:
            event_hints += f"- {e.timestamp_start}: {e.event_type}"
            if e.enemy:
                event_hints += f" ({e.enemy})"
            event_hints += f" at floor {e.floor}\n"

    prompt = f"""Analyze floors {chunk.floor_start}-{chunk.floor_end} of this Watcher Slay the Spire run.
{time_hint}
CURRENT STATE (at floor {state_snapshot.get('floor', 0)}):
- HP: {state_snapshot.get('hp', 0)}/{state_snapshot.get('max_hp', 0)}
- Gold: {state_snapshot.get('gold', 0)}
- Deck ({state_snapshot.get('deck_size', 0)} cards): {', '.join(state_snapshot.get('deck', [])[:10])}{'...' if state_snapshot.get('deck_size', 0) > 10 else ''}
- Relics: {', '.join(state_snapshot.get('relics', []))}
- Potions: {', '.join(state_snapshot.get('potions', [])) or 'None'}
- Seed: {state_snapshot.get('seed', 'UNKNOWN')}
{event_hints}
EXTRACTION INSTRUCTIONS:
1. Watch the video segment carefully
2. Call the appropriate tool for EVERY decision point:
   - `path` for each floor traversal (what room type was chosen)
   - `combat_start` when entering combat
   - `combat_turn` for each turn (list cards played in order)
   - `combat_end` with final HP after combat
   - `card_reward` for card selection (include SKIP if skipped)
   - `relic_reward` when obtaining a relic
   - `potion_reward` when obtaining a potion
   - `shop` for shop transactions
   - `rest` for rest site decisions
   - `event` for event encounters
   - `boss_relic` for boss relic selection (Act bosses)

3. Include timestamps when visible (MM:SS format)
4. For card rewards, list ALL options shown, not just the chosen card
5. Track HP changes accurately - note post-combat HP

Extract ALL decisions in chronological order. Be thorough."""

    # Add chunk-specific instructions
    if chunk.chunk_type == ChunkType.NEOW:
        prompt += """

NEOW BONUS:
- Call `set_seed` if the seed is visible
- Call `neow` with the bonus chosen and any drawback
- This is floor 0 before the first combat"""

    elif chunk.chunk_type in [ChunkType.ACT1_BOSS, ChunkType.ACT2_BOSS, ChunkType.ACT3_BOSS, ChunkType.BOSS]:
        prompt += f"""

BOSS FIGHT (Act {chunk.act}):
- Track the boss combat carefully with combat_start, combat_turn for each turn, combat_end
- After boss defeat, call `boss_relic` with all 3 options and the chosen relic
- Boss relic selection is a critical decision"""

    elif chunk.chunk_type == ChunkType.HEART:
        prompt += """

HEART FIGHT:
- This is the final boss (The Corrupted Heart)
- Call `result` at the end with victory status and final HP
- Track all combat turns carefully"""

    return prompt


def create_scan_tool_config() -> list[dict]:
    """Get the tool configuration for scan pass."""
    from vod.tools import TOOLS
    # Return scan tool + set_seed tool
    return [SCAN_TOOL, next(t for t in TOOLS if t["name"] == "set_seed")]
