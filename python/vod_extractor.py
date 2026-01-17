"""
VOD Decision Extractor for Slay the Spire

Extracts decision data from YouTube/Twitch VODs of expert players.
Pipeline: YouTube -> Frames -> Gemini Vision -> Structured Data

Target streamers:
- Lifecoach (52-win streak, methodical decision-making)
- Merl61 (38-win streak, Rushdown infinite pioneer)
- Baalorlord (educational content, tier lists)

Decision points to capture:
1. Card rewards (after combat) - which card picked or skipped
2. Shop decisions - purchases, removals
3. Event choices - which option selected
4. Path decisions - which node chosen on map
5. Boss relic selection
6. Potion usage
"""

import os
import json
import subprocess
from dataclasses import dataclass, field, asdict
from typing import List, Dict, Optional, Tuple
from pathlib import Path
from enum import Enum
import hashlib

# ============ CONFIGURATION ============

class DecisionType(Enum):
    CARD_REWARD = "card_reward"
    SHOP = "shop"
    EVENT = "event"
    PATH = "path"
    BOSS_RELIC = "boss_relic"
    POTION = "potion"
    COMBAT_TURN = "combat_turn"
    REST_SITE = "rest_site"

@dataclass
class StreamerConfig:
    """Configuration for a streamer's content."""
    name: str
    youtube_channel: str
    twitch_channel: str
    playlist_ids: List[str] = field(default_factory=list)
    video_ids: List[str] = field(default_factory=list)
    notes: str = ""

# Known expert streamers
STREAMERS = {
    "lifecoach": StreamerConfig(
        name="Lifecoach",
        youtube_channel="",  # Primarily Twitch
        twitch_channel="lifecoach1981",
        notes="52-win A20 Heart streak, methodical 15min+ decisions"
    ),
    "merl": StreamerConfig(
        name="Merl61",
        youtube_channel="",
        twitch_channel="merl61",
        notes="38-win A20 Heart streak, Rushdown infinite pioneer"
    ),
    "baalorlord": StreamerConfig(
        name="Baalorlord",
        youtube_channel="UCxVolGbShj4IrKFXC0Hs3lg",
        twitch_channel="baalorlord",
        video_ids=[
            "qEjvZwOq51E",  # Ranking every Watcher card
        ],
        notes="Educational content, tier lists, documented runs at baalorlord.tv/runs"
    ),
}

# ============ DATA STRUCTURES ============

@dataclass
class GameState:
    """Extracted game state from a frame."""
    # Meta
    act: int = 0
    floor: int = 0
    ascension: int = 20

    # Player state
    hp: int = 0
    max_hp: int = 0
    gold: int = 0
    potions: List[str] = field(default_factory=list)

    # Deck info (if visible)
    deck_size: int = 0
    visible_deck: List[str] = field(default_factory=list)

    # Relics
    relics: List[str] = field(default_factory=list)

    # Screen-specific
    screen_type: str = ""
    options: List[str] = field(default_factory=list)
    selected_option: Optional[str] = None

@dataclass
class Decision:
    """A single decision point extracted from a VOD."""
    # Source info
    video_id: str
    timestamp_seconds: float
    frame_path: str

    # Decision info
    decision_type: DecisionType
    game_state: GameState

    # What was chosen
    options_available: List[str]
    option_chosen: str
    option_index: int

    # Context
    streamer_commentary: str = ""
    confidence: float = 1.0  # Model confidence in extraction

@dataclass
class RunData:
    """All decisions from a single run."""
    video_id: str
    streamer: str
    character: str = "Watcher"
    ascension: int = 20
    result: str = ""  # "win" or "loss"
    decisions: List[Decision] = field(default_factory=list)

# ============ VIDEO PROCESSING ============

def download_video(video_id: str, output_dir: Path) -> Path:
    """
    Download YouTube video using yt-dlp.

    Returns path to downloaded video.
    """
    output_path = output_dir / f"{video_id}.mp4"

    if output_path.exists():
        print(f"Video {video_id} already downloaded")
        return output_path

    cmd = [
        "yt-dlp",
        "-f", "best[height<=720]",  # 720p max for efficiency
        "-o", str(output_path),
        f"https://www.youtube.com/watch?v={video_id}"
    ]

    print(f"Downloading {video_id}...")
    subprocess.run(cmd, check=True)
    return output_path

def download_twitch_vod(vod_id: str, output_dir: Path) -> Path:
    """
    Download Twitch VOD using yt-dlp.
    """
    output_path = output_dir / f"twitch_{vod_id}.mp4"

    if output_path.exists():
        print(f"VOD {vod_id} already downloaded")
        return output_path

    cmd = [
        "yt-dlp",
        "-f", "best[height<=720]",
        "-o", str(output_path),
        f"https://www.twitch.tv/videos/{vod_id}"
    ]

    print(f"Downloading Twitch VOD {vod_id}...")
    subprocess.run(cmd, check=True)
    return output_path

def extract_frames(
    video_path: Path,
    output_dir: Path,
    fps: float = 0.5,  # 1 frame every 2 seconds by default
    start_time: Optional[float] = None,
    end_time: Optional[float] = None,
) -> List[Path]:
    """
    Extract frames from video using ffmpeg.

    Args:
        video_path: Path to video file
        output_dir: Directory to save frames
        fps: Frames per second to extract
        start_time: Start time in seconds (optional)
        end_time: End time in seconds (optional)

    Returns:
        List of paths to extracted frames
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    video_name = video_path.stem
    frame_pattern = output_dir / f"{video_name}_frame_%06d.jpg"

    cmd = ["ffmpeg", "-i", str(video_path)]

    if start_time is not None:
        cmd.extend(["-ss", str(start_time)])
    if end_time is not None:
        cmd.extend(["-to", str(end_time)])

    cmd.extend([
        "-vf", f"fps={fps}",
        "-q:v", "2",  # High quality JPEG
        str(frame_pattern),
        "-y"  # Overwrite
    ])

    print(f"Extracting frames at {fps} FPS...")
    subprocess.run(cmd, check=True, capture_output=True)

    # Return list of extracted frames
    frames = sorted(output_dir.glob(f"{video_name}_frame_*.jpg"))
    print(f"Extracted {len(frames)} frames")
    return frames

def detect_decision_screens(frames: List[Path]) -> List[Tuple[Path, str]]:
    """
    Quick filter to identify frames that are likely decision screens.

    Uses simple heuristics before sending to expensive vision model.
    Decision screens in StS have distinct characteristics:
    - Card reward: 3 cards displayed
    - Shop: Grid of items
    - Event: Text + options
    - Map: Node graph
    - Rest site: Campfire options

    Returns: List of (frame_path, likely_screen_type)
    """
    # TODO: Implement quick CV-based detection
    # For now, return all frames for vision model analysis
    return [(f, "unknown") for f in frames]

# ============ GEMINI VISION ANALYSIS ============

GEMINI_PROMPT_CARD_REWARD = """
Analyze this Slay the Spire screenshot. This appears to be a card reward screen.

Extract the following information in JSON format:
{
    "screen_type": "card_reward",
    "act": <int>,
    "floor": <int>,
    "player_hp": <int>,
    "player_max_hp": <int>,
    "gold": <int>,
    "cards_offered": [
        {"name": "<card name>", "upgraded": <bool>, "rarity": "<common/uncommon/rare>"},
        ...
    ],
    "card_selected": "<card name or 'skip'>",
    "visible_relics": ["<relic1>", "<relic2>", ...],
    "visible_potions": ["<potion1>", ...],
    "confidence": <0.0-1.0>
}

If this is NOT a card reward screen, return:
{"screen_type": "not_card_reward", "actual_type": "<what it is>"}
"""

GEMINI_PROMPT_SHOP = """
Analyze this Slay the Spire shop screenshot.

Extract in JSON format:
{
    "screen_type": "shop",
    "act": <int>,
    "floor": <int>,
    "player_hp": <int>,
    "player_gold": <int>,
    "cards_for_sale": [{"name": "<name>", "cost": <int>, "purchased": <bool>}, ...],
    "relics_for_sale": [{"name": "<name>", "cost": <int>, "purchased": <bool>}, ...],
    "potions_for_sale": [{"name": "<name>", "cost": <int>, "purchased": <bool>}, ...],
    "card_removal_cost": <int>,
    "card_removed": <bool>,
    "purchases_made": ["<item1>", ...],
    "confidence": <0.0-1.0>
}
"""

GEMINI_PROMPT_EVENT = """
Analyze this Slay the Spire event screenshot.

Extract in JSON format:
{
    "screen_type": "event",
    "event_name": "<name if identifiable>",
    "act": <int>,
    "floor": <int>,
    "player_hp": <int>,
    "options": [
        {"text": "<option text>", "selected": <bool>},
        ...
    ],
    "option_selected_index": <int>,
    "confidence": <0.0-1.0>
}
"""

GEMINI_PROMPT_GENERAL = """
Analyze this Slay the Spire screenshot and identify what type of decision screen it is.

Possible screen types:
- card_reward: Choosing from 3 cards after combat
- shop: Shopping screen with cards/relics/potions
- event: Random event with choices
- map: Path selection on dungeon map
- rest_site: Campfire with rest/upgrade/etc options
- boss_relic: Choosing boss relic reward
- combat: Active combat (not a decision screen for our purposes)
- other: Menu, loading, or non-decision screen

Return JSON:
{
    "screen_type": "<type>",
    "is_decision_screen": <bool>,
    "game_visible": <bool>,
    "act": <int or null>,
    "floor": <int or null>,
    "player_hp": <int or null>,
    "player_max_hp": <int or null>,
    "gold": <int or null>,
    "brief_description": "<what's happening>",
    "confidence": <0.0-1.0>
}
"""

def analyze_frame_with_gemini(
    frame_path: Path,
    prompt: str,
    model: str = "gemini-2.5-flash"
) -> Dict:
    """
    Analyze a frame using Gemini Vision API.

    Requires GOOGLE_API_KEY environment variable.
    """
    try:
        import google.generativeai as genai
        from PIL import Image
    except ImportError:
        print("Install: pip install google-generativeai pillow")
        return {"error": "missing dependencies"}

    api_key = os.environ.get("GOOGLE_API_KEY")
    if not api_key:
        return {"error": "GOOGLE_API_KEY not set"}

    genai.configure(api_key=api_key)
    model = genai.GenerativeModel(model)

    image = Image.open(frame_path)

    response = model.generate_content([prompt, image])

    # Parse JSON from response
    try:
        # Extract JSON from response text
        text = response.text
        # Find JSON in response
        start = text.find('{')
        end = text.rfind('}') + 1
        if start >= 0 and end > start:
            return json.loads(text[start:end])
        return {"error": "no JSON in response", "raw": text}
    except json.JSONDecodeError as e:
        return {"error": f"JSON parse error: {e}", "raw": response.text}

def batch_analyze_frames(
    frames: List[Path],
    output_path: Path,
    checkpoint_every: int = 10
) -> List[Dict]:
    """
    Analyze multiple frames, with checkpointing.

    Saves progress to avoid re-processing on failure.
    """
    results = []
    checkpoint_file = output_path / "analysis_checkpoint.json"

    # Load existing progress
    if checkpoint_file.exists():
        with open(checkpoint_file) as f:
            checkpoint = json.load(f)
            results = checkpoint.get("results", [])
            processed = set(checkpoint.get("processed", []))
    else:
        processed = set()

    for i, frame in enumerate(frames):
        frame_id = frame.stem

        if frame_id in processed:
            continue

        print(f"Analyzing {i+1}/{len(frames)}: {frame.name}")

        # First pass: identify screen type
        result = analyze_frame_with_gemini(frame, GEMINI_PROMPT_GENERAL)
        result["frame_path"] = str(frame)
        result["frame_id"] = frame_id

        # If it's a decision screen, do detailed analysis
        if result.get("is_decision_screen"):
            screen_type = result.get("screen_type")
            if screen_type == "card_reward":
                detailed = analyze_frame_with_gemini(frame, GEMINI_PROMPT_CARD_REWARD)
                result["detailed"] = detailed
            elif screen_type == "shop":
                detailed = analyze_frame_with_gemini(frame, GEMINI_PROMPT_SHOP)
                result["detailed"] = detailed
            elif screen_type == "event":
                detailed = analyze_frame_with_gemini(frame, GEMINI_PROMPT_EVENT)
                result["detailed"] = detailed

        results.append(result)
        processed.add(frame_id)

        # Checkpoint
        if (i + 1) % checkpoint_every == 0:
            with open(checkpoint_file, 'w') as f:
                json.dump({"results": results, "processed": list(processed)}, f)
            print(f"Checkpoint saved at {i+1} frames")

    # Final save
    with open(checkpoint_file, 'w') as f:
        json.dump({"results": results, "processed": list(processed)}, f)

    return results

# ============ DATA PROCESSING ============

def extract_decisions_from_analysis(
    analysis_results: List[Dict],
    video_id: str
) -> List[Decision]:
    """
    Convert frame analysis results into structured decisions.
    """
    decisions = []

    for result in analysis_results:
        if not result.get("is_decision_screen"):
            continue

        screen_type = result.get("screen_type")
        detailed = result.get("detailed", {})

        # Map screen type to DecisionType
        type_map = {
            "card_reward": DecisionType.CARD_REWARD,
            "shop": DecisionType.SHOP,
            "event": DecisionType.EVENT,
            "map": DecisionType.PATH,
            "rest_site": DecisionType.REST_SITE,
            "boss_relic": DecisionType.BOSS_RELIC,
        }

        decision_type = type_map.get(screen_type)
        if not decision_type:
            continue

        # Extract frame number to estimate timestamp
        frame_id = result.get("frame_id", "")
        try:
            frame_num = int(frame_id.split("_")[-1])
            timestamp = frame_num * 2  # Assuming 0.5 FPS
        except:
            timestamp = 0

        # Build game state
        game_state = GameState(
            act=result.get("act") or detailed.get("act", 0),
            floor=result.get("floor") or detailed.get("floor", 0),
            hp=result.get("player_hp") or detailed.get("player_hp", 0),
            max_hp=result.get("player_max_hp") or detailed.get("player_max_hp", 0),
            gold=result.get("gold") or detailed.get("player_gold", 0),
            screen_type=screen_type,
        )

        # Extract options and selection based on screen type
        options = []
        selected = None

        if screen_type == "card_reward":
            cards = detailed.get("cards_offered", [])
            options = [c.get("name", "unknown") for c in cards]
            options.append("skip")
            selected = detailed.get("card_selected", "skip")

        elif screen_type == "shop":
            purchases = detailed.get("purchases_made", [])
            if detailed.get("card_removed"):
                purchases.append("card_removal")
            options = ["purchase", "skip"]
            selected = "purchase" if purchases else "skip"

        elif screen_type == "event":
            event_options = detailed.get("options", [])
            options = [o.get("text", "unknown") for o in event_options]
            selected_idx = detailed.get("option_selected_index", 0)
            selected = options[selected_idx] if selected_idx < len(options) else "unknown"

        if options and selected:
            decision = Decision(
                video_id=video_id,
                timestamp_seconds=timestamp,
                frame_path=result.get("frame_path", ""),
                decision_type=decision_type,
                game_state=game_state,
                options_available=options,
                option_chosen=selected,
                option_index=options.index(selected) if selected in options else -1,
                confidence=result.get("confidence", detailed.get("confidence", 0.5)),
            )
            decisions.append(decision)

    return decisions

def save_run_data(run: RunData, output_path: Path):
    """Save run data to JSON."""
    data = {
        "video_id": run.video_id,
        "streamer": run.streamer,
        "character": run.character,
        "ascension": run.ascension,
        "result": run.result,
        "decisions": [
            {
                **asdict(d),
                "decision_type": d.decision_type.value,
                "game_state": asdict(d.game_state),
            }
            for d in run.decisions
        ]
    }

    with open(output_path, 'w') as f:
        json.dump(data, f, indent=2)
    print(f"Saved {len(run.decisions)} decisions to {output_path}")

# ============ TRANSCRIPT EXTRACTION ============

def get_youtube_transcript(video_id: str) -> List[Dict]:
    """
    Get auto-generated transcript from YouTube.

    Returns list of {text, start, duration} dicts.
    """
    try:
        from youtube_transcript_api import YouTubeTranscriptApi
    except ImportError:
        print("Install: pip install youtube-transcript-api")
        return []

    try:
        api = YouTubeTranscriptApi()
        transcript = api.fetch(video_id)
        # Convert to list of dicts with expected format
        return [
            {"text": entry.text, "start": entry.start, "duration": entry.duration}
            for entry in transcript
        ]
    except Exception as e:
        print(f"Could not get transcript: {e}")
        return []

def align_transcript_with_decisions(
    transcript: List[Dict],
    decisions: List[Decision]
) -> List[Decision]:
    """
    Align transcript segments with decisions based on timestamps.

    Adds streamer commentary to decisions.
    """
    for decision in decisions:
        # Find transcript segments within ~10 seconds of decision
        relevant_text = []
        for segment in transcript:
            seg_start = segment["start"]
            seg_end = seg_start + segment["duration"]

            # Check if segment overlaps with decision time window
            decision_start = decision.timestamp_seconds - 5
            decision_end = decision.timestamp_seconds + 10

            if seg_start < decision_end and seg_end > decision_start:
                relevant_text.append(segment["text"])

        decision.streamer_commentary = " ".join(relevant_text)

    return decisions

# ============ MAIN PIPELINE ============

def process_video(
    video_id: str,
    streamer: str,
    output_dir: Path,
    source: str = "youtube",  # or "twitch"
    fps: float = 0.5,
) -> RunData:
    """
    Full pipeline: download -> extract frames -> analyze -> structure data.
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    video_dir = output_dir / "videos"
    frames_dir = output_dir / "frames" / video_id
    analysis_dir = output_dir / "analysis"

    video_dir.mkdir(exist_ok=True)
    analysis_dir.mkdir(exist_ok=True)

    # 1. Download video
    print(f"\n=== Step 1: Download Video ===")
    if source == "youtube":
        video_path = download_video(video_id, video_dir)
    else:
        video_path = download_twitch_vod(video_id, video_dir)

    # 2. Extract frames
    print(f"\n=== Step 2: Extract Frames ===")
    frames = extract_frames(video_path, frames_dir, fps=fps)

    # 3. Get transcript (for context)
    print(f"\n=== Step 3: Get Transcript ===")
    transcript = get_youtube_transcript(video_id) if source == "youtube" else []
    print(f"Got {len(transcript)} transcript segments")

    # 4. Analyze frames with Gemini
    print(f"\n=== Step 4: Analyze Frames ===")
    analysis_results = batch_analyze_frames(frames, analysis_dir)

    # 5. Extract decisions
    print(f"\n=== Step 5: Extract Decisions ===")
    decisions = extract_decisions_from_analysis(analysis_results, video_id)
    print(f"Extracted {len(decisions)} decisions")

    # 6. Align with transcript
    if transcript:
        print(f"\n=== Step 6: Align Transcript ===")
        decisions = align_transcript_with_decisions(transcript, decisions)

    # 7. Create run data
    run = RunData(
        video_id=video_id,
        streamer=streamer,
        character="Watcher",
        ascension=20,
        decisions=decisions,
    )

    # 8. Save
    output_file = analysis_dir / f"{video_id}_decisions.json"
    save_run_data(run, output_file)

    return run

# ============ CLI ============

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Extract decisions from StS VODs")
    parser.add_argument("--video-id", required=True, help="YouTube video ID")
    parser.add_argument("--streamer", default="unknown", help="Streamer name")
    parser.add_argument("--output", default="./vod_data", help="Output directory")
    parser.add_argument("--fps", type=float, default=0.5, help="Frames per second")
    parser.add_argument("--source", choices=["youtube", "twitch"], default="youtube")

    args = parser.parse_args()

    run = process_video(
        video_id=args.video_id,
        streamer=args.streamer,
        output_dir=Path(args.output),
        source=args.source,
        fps=args.fps,
    )

    print(f"\n=== Complete ===")
    print(f"Extracted {len(run.decisions)} decisions from {args.video_id}")
