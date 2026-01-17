"""
Transcript-Based Decision Extractor

Efficient approach: Use transcript for decision reasoning + final W screen for verification.

Streamers verbally explain:
- "Taking Rushdown here because..."
- "Skipping this card, deck is getting bloated"
- "Going elite path, need scaling"
- "Resting here, too low to upgrade"

Final victory screen gives us:
- Seed (can replay!)
- Final deck
- All relics
- Path taken
- Score breakdown
"""

import re
import json
from pathlib import Path
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass, field
from enum import Enum

# ============ DECISION PATTERNS ============

class DecisionType(Enum):
    CARD_PICK = "card_pick"
    CARD_SKIP = "card_skip"
    SHOP = "shop"
    EVENT = "event"
    PATH = "path"
    REST_SITE = "rest_site"
    BOSS_RELIC = "boss_relic"
    POTION = "potion"

# Keywords that indicate decision explanations - TIGHT patterns
DECISION_PATTERNS = {
    DecisionType.CARD_PICK: [
        # Explicit card pick statements
        r"i(?:'m| am) (?:gonna |going to )?take (?:the )?([a-z]+(?: [a-z]+)?)",
        r"(?:gonna|going to) take (?:the )?([a-z]+(?: [a-z]+)?)",
        r"i(?:'ll| will) pick (?:the )?([a-z]+(?: [a-z]+)?)",
        r"easy ([a-z]+(?: [a-z]+)?) here",
        r"snap ([a-z]+(?: [a-z]+)?)",
        r"([a-z]+(?: [a-z]+)?) is (?:the )?(?:best|obvious) (?:pick|choice)",
    ],
    DecisionType.CARD_SKIP: [
        r"(?:i'm |gonna |going to )?skip(?:ping)? (?:this|the card|here)",
        r"don't (?:need|want) any of these",
        r"none of these (?:cards )?(?:are |look )",
        r"deck is (?:getting )?(?:too )?(?:big|bloated)",
        r"passing on (?:this|these|the card)",
    ],
    DecisionType.REST_SITE: [
        r"(?:i'm |gonna |going to )?rest(?:ing)? here",
        r"(?:i'm |gonna |going to )?upgrade (?:the |my )?([a-z]+(?: [a-z]+)?)",
        r"(?:gonna |going to )?smith (?:the |my )?([a-z]+(?: [a-z]+)?)",
        r"(?:need to |gotta |have to )?heal",
        r"too low (?:hp )?to (?:upgrade|smith)",
    ],
    DecisionType.PATH: [
        r"(?:gonna |going to )?(?:go |take )(?:the )?elite",
        r"(?:gonna |going to )?(?:go |take )(?:the )?(?:safe|easy) (?:path|route)",
        r"(?:need|want) (?:to hit )?(?:the )?shop",
        r"(?:need|want) (?:a |the )?rest site",
        r"avoiding (?:the )?elite",
        r"can(?:'t| not) fight (?:the )?elite",
    ],
    DecisionType.BOSS_RELIC: [
        # Very specific to boss relic choice context
        r"(?:taking|picking|grabbing) ([a-z]+(?: [a-z]+)?) (?:as (?:the|my) )?(?:boss )?relic",
        r"([a-z]+(?: [a-z]+)?) (?:is |seems )?(?:the )?best (?:boss )?relic",
        r"easy ([a-z]+(?: [a-z]+)?) for (?:the )?(?:boss )?relic",
    ],
    DecisionType.EVENT: [
        r"(?:taking|choosing|picking) (?:the )?(?:first|second|third|top|bottom) option",
        r"(?:gonna |going to )?(?:take |choose )(?:the )?(?:gold|card|relic|remove|upgrade)",
    ],
}

# Watcher-specific card names for matching
WATCHER_CARDS = [
    "eruption", "vigilance", "bowling bash", "consecrate", "crescendo",
    "crush joints", "cut through fate", "empty body", "empty fist",
    "evaluate", "fear no evil", "flurry of blows", "flying sleeves",
    "follow up", "halt", "just lucky", "pressure points", "prostrate",
    "protect", "sash whip", "tranquility", "battle hymn", "carve reality",
    "collect", "conclude", "deceive reality", "empty mind", "fasting",
    "foreign influence", "indignation", "inner peace", "like water",
    "meditate", "mental fortress", "nirvana", "perseverance", "pray",
    "reach heaven", "rushdown", "sanctity", "sands of time", "signature move",
    "study", "swivel", "talk to the hand", "tantrum", "wallop", "wave of the hand",
    "weave", "wheel kick", "windmill strike", "worship", "wreath of flame",
    "alpha", "blasphemy", "brilliance", "conjure blade", "deus ex machina",
    "devotion", "establishment", "judgment", "lesson learned", "master reality",
    "omniscience", "ragnarok", "scrawl", "spirit shield", "vault", "wish",
]

@dataclass
class TranscriptDecision:
    """A decision extracted from transcript."""
    timestamp: float
    decision_type: DecisionType
    raw_text: str
    extracted_choice: Optional[str] = None
    confidence: float = 0.5
    context_before: str = ""
    context_after: str = ""

@dataclass
class VictoryScreenData:
    """Data from final victory screen."""
    seed: str
    final_deck: List[str]
    relics: List[str]
    score: int
    floor_reached: int
    path_taken: List[str]  # M, ?, $, T, E, R, B
    ascension: int = 20
    character: str = "Watcher"

@dataclass
class TranscriptRun:
    """A complete run extracted from transcript + victory screen."""
    video_id: str
    streamer: str
    transcript_decisions: List[TranscriptDecision]
    victory_data: Optional[VictoryScreenData] = None
    raw_transcript: List[Dict] = field(default_factory=list)

# ============ TRANSCRIPT PARSING ============

def get_transcript(video_id: str) -> List[Dict]:
    """Fetch YouTube transcript."""
    try:
        from youtube_transcript_api import YouTubeTranscriptApi
        api = YouTubeTranscriptApi()
        transcript = api.fetch(video_id)
        return [
            {"text": entry.text, "start": entry.start, "duration": entry.duration}
            for entry in transcript
        ]
    except Exception as e:
        print(f"Could not get transcript: {e}")
        return []

def extract_decisions_from_transcript(transcript: List[Dict]) -> List[TranscriptDecision]:
    """Parse transcript for decision explanations."""
    decisions = []

    # Combine nearby segments for context
    full_text_segments = []
    for i, seg in enumerate(transcript):
        context_before = " ".join(t["text"] for t in transcript[max(0, i-3):i])
        context_after = " ".join(t["text"] for t in transcript[i+1:i+4])
        full_text_segments.append({
            "text": seg["text"],
            "start": seg["start"],
            "context_before": context_before,
            "context_after": context_after,
        })

    for seg in full_text_segments:
        text = seg["text"].lower()

        for decision_type, patterns in DECISION_PATTERNS.items():
            for pattern in patterns:
                match = re.search(pattern, text, re.IGNORECASE)
                if match:
                    # Extract the choice if captured
                    choice = match.group(1) if match.lastindex else None

                    # Check if it's a Watcher card
                    confidence = 0.5
                    if choice:
                        choice_lower = choice.lower()
                        for card in WATCHER_CARDS:
                            if card in choice_lower or choice_lower in card:
                                confidence = 0.8
                                choice = card.title()
                                break

                    decision = TranscriptDecision(
                        timestamp=seg["start"],
                        decision_type=decision_type,
                        raw_text=seg["text"],
                        extracted_choice=choice,
                        confidence=confidence,
                        context_before=seg["context_before"],
                        context_after=seg["context_after"],
                    )
                    decisions.append(decision)
                    break  # One match per segment

    return decisions

def cluster_decisions(decisions: List[TranscriptDecision], time_window: float = 30.0) -> List[List[TranscriptDecision]]:
    """Group decisions that are close in time (likely same decision point)."""
    if not decisions:
        return []

    clusters = []
    current_cluster = [decisions[0]]

    for decision in decisions[1:]:
        if decision.timestamp - current_cluster[-1].timestamp < time_window:
            current_cluster.append(decision)
        else:
            clusters.append(current_cluster)
            current_cluster = [decision]

    clusters.append(current_cluster)
    return clusters

# ============ VICTORY SCREEN EXTRACTION ============

def extract_victory_screen(video_path: Path, output_path: Path) -> Path:
    """Extract the final frames of a video (victory screen)."""
    import subprocess

    # Get video duration
    result = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration",
         "-of", "default=noprint_wrappers=1:nokey=1", str(video_path)],
        capture_output=True, text=True
    )
    duration = float(result.stdout.strip())

    # Extract last 30 seconds at 0.5 fps
    start_time = max(0, duration - 30)

    output_pattern = output_path / "victory_%03d.jpg"
    subprocess.run([
        "ffmpeg", "-i", str(video_path),
        "-ss", str(start_time),
        "-vf", "fps=0.5",
        "-q:v", "2",
        str(output_pattern),
        "-y"
    ], capture_output=True)

    return output_path

def parse_victory_screen_with_gemini(frame_path: Path) -> Optional[VictoryScreenData]:
    """Use Gemini to extract data from victory screen."""
    import os

    api_key = os.environ.get("GOOGLE_API_KEY")
    if not api_key:
        print("GOOGLE_API_KEY not set - cannot parse victory screen")
        return None

    try:
        import google.generativeai as genai
        from PIL import Image

        genai.configure(api_key=api_key)
        model = genai.GenerativeModel("gemini-2.0-flash")

        prompt = """
        Analyze this Slay the Spire victory screen and extract:

        Return JSON:
        {
            "seed": "<seed string>",
            "final_deck": ["card1", "card2", ...],
            "relics": ["relic1", "relic2", ...],
            "score": <int>,
            "floor_reached": <int>,
            "ascension": <int>,
            "character": "<character name>"
        }

        If this is not a victory screen, return: {"error": "not victory screen"}
        """

        image = Image.open(frame_path)
        response = model.generate_content([prompt, image])

        text = response.text
        start = text.find('{')
        end = text.rfind('}') + 1
        if start >= 0 and end > start:
            data = json.loads(text[start:end])
            if "error" not in data:
                return VictoryScreenData(
                    seed=data.get("seed", ""),
                    final_deck=data.get("final_deck", []),
                    relics=data.get("relics", []),
                    score=data.get("score", 0),
                    floor_reached=data.get("floor_reached", 0),
                    ascension=data.get("ascension", 20),
                    character=data.get("character", "Watcher"),
                    path_taken=[],
                )
    except Exception as e:
        print(f"Gemini error: {e}")

    return None

# ============ MAIN PIPELINE ============

def process_win_video(
    video_id: str,
    streamer: str,
    output_dir: Path,
    download_video: bool = False,
) -> TranscriptRun:
    """
    Process a winning run video.

    Efficient approach:
    1. Get transcript (free, instant)
    2. Extract decisions from transcript
    3. Optionally download video for victory screen
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # 1. Get transcript
    print(f"Fetching transcript for {video_id}...")
    transcript = get_transcript(video_id)
    print(f"  Got {len(transcript)} segments")

    # 2. Extract decisions
    print("Extracting decisions from transcript...")
    decisions = extract_decisions_from_transcript(transcript)
    print(f"  Found {len(decisions)} decision mentions")

    # 3. Cluster decisions
    clusters = cluster_decisions(decisions)
    print(f"  Clustered into {len(clusters)} decision points")

    # Create run data
    run = TranscriptRun(
        video_id=video_id,
        streamer=streamer,
        transcript_decisions=decisions,
        raw_transcript=transcript,
    )

    # 4. Optionally extract victory screen
    if download_video:
        print("Victory screen extraction requires video download + Gemini API")
        # Implementation would go here

    # Save results
    save_transcript_run(run, output_dir / f"{video_id}_transcript.json")

    return run

def save_transcript_run(run: TranscriptRun, output_path: Path):
    """Save transcript run data - keep FULL transcript for later distillation."""
    data = {
        "video_id": run.video_id,
        "streamer": run.streamer,
        # Full transcript - can distill with LLM later
        "full_transcript": run.raw_transcript,
        # Combined text for easy reading
        "full_text": " ".join(seg["text"] for seg in run.raw_transcript),
        # Pattern-matched decisions (first pass, may have noise)
        "pattern_decisions": [
            {
                "timestamp": d.timestamp,
                "type": d.decision_type.value,
                "raw_text": d.raw_text,
                "choice": d.extracted_choice,
                "confidence": d.confidence,
            }
            for d in run.transcript_decisions
        ],
        "stats": {
            "transcript_segments": len(run.raw_transcript),
            "pattern_matches": len(run.transcript_decisions),
            "duration_seconds": run.raw_transcript[-1]["start"] if run.raw_transcript else 0,
        }
    }

    if run.victory_data:
        data["victory"] = {
            "seed": run.victory_data.seed,
            "final_deck": run.victory_data.final_deck,
            "relics": run.victory_data.relics,
            "score": run.victory_data.score,
        }

    with open(output_path, 'w') as f:
        json.dump(data, f, indent=2)

    print(f"Saved to {output_path}")

def distill_transcript_with_llm(transcript_path: Path, output_path: Path):
    """
    Use LLM to distill transcript into structured decisions.

    This is the REAL extraction - pattern matching is just a first pass.
    """
    import os

    with open(transcript_path) as f:
        data = json.load(f)

    full_text = data.get("full_text", "")
    if not full_text:
        print("No transcript text found")
        return

    # Check for API key
    api_key = os.environ.get("GOOGLE_API_KEY") or os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("No API key found (GOOGLE_API_KEY or OPENROUTER_API_KEY)")
        print("Saving raw transcript for manual processing")
        return

    prompt = f"""
    Analyze this Slay the Spire Watcher run transcript and extract ALL decisions made.

    For each decision, identify:
    1. Type: card_pick, card_skip, rest_site, path, boss_relic, event, shop
    2. Choice made
    3. Reasoning given (why they made this choice)
    4. Alternatives considered

    Focus on:
    - Card reward choices (what card picked or skipped, why)
    - Rest site decisions (rest vs upgrade, which card upgraded)
    - Path decisions (elite vs safe path, shop priority)
    - Boss relic choices
    - Event choices
    - Shop purchases/removals

    Return as JSON array:
    [
        {{
            "type": "card_pick",
            "choice": "Rushdown",
            "reasoning": "Need draw engine for deck",
            "alternatives": ["Tantrum", "Conclude"],
            "approximate_timestamp": "early act 1"
        }},
        ...
    ]

    TRANSCRIPT:
    {full_text[:50000]}  # Truncate if too long
    """

    try:
        import google.generativeai as genai
        genai.configure(api_key=api_key)
        model = genai.GenerativeModel("gemini-2.0-flash")

        response = model.generate_content(prompt)
        text = response.text

        # Extract JSON
        start = text.find('[')
        end = text.rfind(']') + 1
        if start >= 0 and end > start:
            decisions = json.loads(text[start:end])

            output = {
                "video_id": data["video_id"],
                "streamer": data["streamer"],
                "distilled_decisions": decisions,
                "decision_count": len(decisions),
            }

            with open(output_path, 'w') as f:
                json.dump(output, f, indent=2)

            print(f"Distilled {len(decisions)} decisions to {output_path}")
            return decisions

    except Exception as e:
        print(f"LLM distillation failed: {e}")
        return None

# ============ FINDING WIN VIDEOS ============

def search_watcher_wins(channel: str, max_results: int = 50) -> List[Dict]:
    """Search YouTube for Watcher win videos."""
    import subprocess

    query = f"{channel} watcher win ascension 20 slay the spire"

    result = subprocess.run([
        "yt-dlp",
        f"ytsearch{max_results}:{query}",
        "--print", "%(id)s|%(title)s|%(duration_string)s",
        "--no-download",
    ], capture_output=True, text=True)

    videos = []
    for line in result.stdout.strip().split('\n'):
        if '|' in line:
            parts = line.split('|')
            if len(parts) >= 3:
                videos.append({
                    "id": parts[0],
                    "title": parts[1],
                    "duration": parts[2],
                })

    # Filter for likely wins
    win_keywords = ["win", "victory", "heart", "streak", "w"]
    filtered = [
        v for v in videos
        if any(kw in v["title"].lower() for kw in win_keywords)
    ]

    return filtered

# ============ CLI ============

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Extract decisions from VOD transcripts")
    parser.add_argument("--video-id", help="YouTube video ID")
    parser.add_argument("--search", help="Search for wins by channel name")
    parser.add_argument("--streamer", default="unknown", help="Streamer name")
    parser.add_argument("--output", default="./transcript_data", help="Output directory")

    args = parser.parse_args()

    if args.search:
        print(f"Searching for {args.search} Watcher wins...")
        videos = search_watcher_wins(args.search)
        print(f"Found {len(videos)} potential win videos:")
        for v in videos[:10]:
            print(f"  {v['id']}: {v['title']} ({v['duration']})")

    elif args.video_id:
        run = process_win_video(
            video_id=args.video_id,
            streamer=args.streamer,
            output_dir=Path(args.output),
        )
        print(f"\nExtracted {len(run.transcript_decisions)} decisions")
