#!/usr/bin/env python3
"""
Full Run Extraction Script

Extracts all decisions from a Slay the Spire VOD with seed-based predictions.
Outputs JSON for the verification UI.

Usage:
    python -m vod.extract_full_run VIDEO_PATH [SEED] [--output OUTPUT.json]
"""

import argparse
import json
import os
import sys
import time
from dataclasses import dataclass, field, asdict
from typing import Optional, List, Dict, Any
from dotenv import load_dotenv

load_dotenv()

try:
    from google import genai
    from google.genai import types
    GENAI_AVAILABLE = True
except ImportError:
    GENAI_AVAILABLE = False

from core.state.game_rng import GameRNGState, RNGStream
from core.generation.rewards import generate_card_rewards, RewardState


@dataclass
class Extraction:
    """A single extracted decision."""
    timestamp: str
    type: str
    floor: int = 0
    predicted_options: List[str] = field(default_factory=list)
    chosen_index: int = -1
    chosen_item: str = ""
    details: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self):
        return asdict(self)


class FullRunExtractor:
    """Extract all decisions from a complete VOD."""

    # ALWAYS use Gemini 3.0 models
    DEFAULT_MODEL = "gemini-3-flash-preview"

    EXTRACTION_PROMPT = """Analyze this Slay the Spire video segment from {start} to {end}.

You are extracting game decisions for a Watcher run. Call the appropriate function for EACH decision you observe.

IMPORTANT RULES:
1. Only report decisions you can CLEARLY see happen
2. For card rewards: report the timestamp, what cards were offered, and which was picked (or SKIP)
3. For paths: report floor number and room type chosen
4. For events: report event name and choice made
5. Use exact card/relic names as shown in the game

Current game state:
- Floor: {floor}
- Act: {act}
{predictions}

Watch carefully and extract ALL decisions in chronological order."""

    def __init__(self, model: str = None):
        if not GENAI_AVAILABLE:
            raise ImportError("google-genai package required")

        self.model = model or self.DEFAULT_MODEL
        self.client = genai.Client(api_key=os.environ.get("GOOGLE_API_KEY"))
        self.video_file = None

    def upload_video(self, video_path: str, timeout: int = 300):
        """Upload video to Gemini."""
        print(f"Uploading: {video_path}")
        start = time.time()

        self.video_file = self.client.files.upload(file=video_path)

        while not self.video_file.state or self.video_file.state.name != "ACTIVE":
            if time.time() - start > timeout:
                raise TimeoutError("Video processing timeout")
            print(f"  Processing... ({time.time() - start:.0f}s)")
            time.sleep(5)
            self.video_file = self.client.files.get(name=self.video_file.name)

        print(f"Video ready in {time.time() - start:.1f}s")

    def detect_seed(self) -> Optional[str]:
        """Detect seed from video."""
        if not self.video_file:
            raise RuntimeError("Video not uploaded")

        print("Detecting seed...")

        prompt = """Look for the SEED in this Slay the Spire video.

The seed is displayed as "Seed: XXXXX" in:
- Top corners during gameplay
- Pause menu (ESC)
- End of run screen
- Character select

Seeds are 4-15 alphanumeric characters like: 1V2ZJKI0, ABC123, 5F8M2N

Check timestamps: 0:15, 30:00, and 50:00 for the seed.

If found, respond with ONLY the seed characters (no extra text).
If not found, respond with "NOT FOUND"."""

        try:
            response = self.client.models.generate_content(
                model=self.model,
                contents=[self.video_file, prompt],
                config=types.GenerateContentConfig(
                    temperature=0.0,
                    max_output_tokens=100,
                ),
            )

            text = response.text.strip().upper()

            if "NOT FOUND" not in text:
                import re
                match = re.search(r'[A-Z0-9]{4,15}', text)
                if match:
                    print(f"  Detected: {match.group()}")
                    return match.group()

            print("  No seed found")
            return None

        except Exception as e:
            print(f"  Error: {e}")
            return None

    def extract_chunk(
        self,
        start_time: str,
        end_time: str,
        floor: int,
        act: int,
        predictions: str = "",
    ) -> List[Dict]:
        """Extract decisions from a video chunk."""
        if not self.video_file:
            raise RuntimeError("Video not uploaded")

        prompt = self.EXTRACTION_PROMPT.format(
            start=start_time,
            end=end_time,
            floor=floor,
            act=act,
            predictions=predictions
        )

        tools = [{
            "function_declarations": [
                {
                    "name": "neow_choice",
                    "description": "Record Neow bonus selection at start of run",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string", "description": "Video timestamp (MM:SS)"},
                            "chosen_bonus": {"type": "string", "description": "The bonus chosen (exact text)"},
                            "drawback": {"type": "string", "description": "Any drawback text"},
                            "is_boss_swap": {"type": "boolean", "description": "True if boss relic swap"}
                        },
                        "required": ["timestamp", "chosen_bonus"]
                    }
                },
                {
                    "name": "path_choice",
                    "description": "Record map path selection",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer", "description": "Floor number after move"},
                            "room_type": {"type": "string", "enum": ["monster", "elite", "rest", "shop", "event", "chest", "boss"]}
                        },
                        "required": ["timestamp", "floor", "room_type"]
                    }
                },
                {
                    "name": "card_reward",
                    "description": "Record card reward after combat",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer"},
                            "cards_offered": {"type": "array", "items": {"type": "string"}, "description": "Card names shown"},
                            "chosen_card": {"type": "string", "description": "Card picked, or 'SKIP'"},
                            "chosen_index": {"type": "integer", "description": "Index of chosen card (0,1,2) or -1 for skip"}
                        },
                        "required": ["timestamp", "floor", "cards_offered", "chosen_card"]
                    }
                },
                {
                    "name": "combat_end",
                    "description": "Record end of combat",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer"},
                            "enemy": {"type": "string", "description": "Enemy name(s)"},
                            "hp_remaining": {"type": "integer"}
                        },
                        "required": ["timestamp", "floor"]
                    }
                },
                {
                    "name": "rest_site",
                    "description": "Record rest site decision",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer"},
                            "action": {"type": "string", "enum": ["rest", "smith", "dig", "recall", "toke", "lift"]},
                            "card_upgraded": {"type": "string", "description": "If smith, which card"}
                        },
                        "required": ["timestamp", "floor", "action"]
                    }
                },
                {
                    "name": "shop_visit",
                    "description": "Record shop purchases",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer"},
                            "cards_bought": {"type": "array", "items": {"type": "string"}},
                            "relics_bought": {"type": "array", "items": {"type": "string"}},
                            "potions_bought": {"type": "array", "items": {"type": "string"}},
                            "card_removed": {"type": "string"}
                        },
                        "required": ["timestamp", "floor"]
                    }
                },
                {
                    "name": "event_choice",
                    "description": "Record event decision",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer"},
                            "event_name": {"type": "string"},
                            "choice_made": {"type": "string"},
                            "choice_index": {"type": "integer"}
                        },
                        "required": ["timestamp", "floor", "event_name", "choice_made"]
                    }
                },
                {
                    "name": "boss_relic",
                    "description": "Record boss relic selection",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer"},
                            "options": {"type": "array", "items": {"type": "string"}},
                            "chosen_relic": {"type": "string"},
                            "chosen_index": {"type": "integer"}
                        },
                        "required": ["timestamp", "floor", "chosen_relic"]
                    }
                },
                {
                    "name": "potion_use",
                    "description": "Record potion usage",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer"},
                            "potion": {"type": "string"}
                        },
                        "required": ["timestamp", "floor", "potion"]
                    }
                },
                {
                    "name": "run_end",
                    "description": "Record end of run",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "timestamp": {"type": "string"},
                            "floor": {"type": "integer"},
                            "victory": {"type": "boolean"},
                            "heart_kill": {"type": "boolean"}
                        },
                        "required": ["timestamp", "floor", "victory"]
                    }
                }
            ]
        }]

        try:
            response = self.client.models.generate_content(
                model=self.model,
                contents=[self.video_file, prompt],
                config=types.GenerateContentConfig(
                    temperature=0.1,
                    max_output_tokens=4096,
                    tools=tools,
                ),
            )

            # Parse tool calls
            tool_calls = []
            if response.candidates:
                for candidate in response.candidates:
                    if candidate.content and candidate.content.parts:
                        for part in candidate.content.parts:
                            if hasattr(part, 'function_call') and part.function_call:
                                fc = part.function_call
                                call_data = {"type": fc.name}
                                if fc.args:
                                    call_data.update(dict(fc.args))
                                tool_calls.append(call_data)

            return tool_calls

        except Exception as e:
            print(f"  Error extracting {start_time}-{end_time}: {e}")
            return []

    def extract_full_run(
        self,
        video_path: str,
        seed: Optional[str] = None,
        chunk_minutes: int = 5,
        video_duration_minutes: int = 55,
    ) -> List[Dict]:
        """Extract all decisions from a full video."""

        # Upload video
        self.upload_video(video_path)

        # Detect seed if not provided
        if not seed:
            seed = self.detect_seed()

        # Initialize RNG state if we have seed
        rng_state = None
        reward_state = RewardState()
        if seed:
            rng_state = GameRNGState(seed)
            rng_state.apply_neow_choice("HUNDRED_GOLD")  # Default assumption

        all_extractions = []
        floor = 0
        act = 1

        # Process in chunks
        total_chunks = (video_duration_minutes + chunk_minutes - 1) // chunk_minutes
        print(f"\nExtracting {total_chunks} chunks...")

        for i in range(total_chunks):
            start_min = i * chunk_minutes
            end_min = min((i + 1) * chunk_minutes, video_duration_minutes)
            start_time = f"{start_min:02d}:00"
            end_time = f"{end_min:02d}:00"

            # Generate predictions if we have seed
            predictions = ""
            if rng_state:
                try:
                    card_rng = rng_state.get_rng(RNGStream.CARD)
                    cards = generate_card_rewards(
                        rng=card_rng,
                        reward_state=reward_state,
                        act=act,
                        player_class="WATCHER",
                        ascension=20,
                        room_type="normal",
                        num_cards=3,
                    )
                    pred_cards = [c.name for c in cards]
                    predictions = f"\nPredicted next card reward: {pred_cards}"
                except:
                    pass

            print(f"  [{i+1}/{total_chunks}] {start_time} - {end_time}...")

            extractions = self.extract_chunk(
                start_time, end_time,
                floor=floor, act=act,
                predictions=predictions
            )

            # Process extractions and update state
            for ext in extractions:
                ext_type = ext.get("type", "")

                # Add predictions to card rewards
                if ext_type == "card_reward" and rng_state:
                    try:
                        card_rng = rng_state.get_rng(RNGStream.CARD)
                        cards = generate_card_rewards(
                            rng=card_rng,
                            reward_state=reward_state,
                            act=act,
                            player_class="WATCHER",
                            ascension=20,
                            room_type="normal",
                            num_cards=3,
                        )
                        ext["predicted_options"] = [c.name for c in cards]
                        # Advance RNG
                        rng_state.advance(RNGStream.CARD, 9)
                    except:
                        pass

                # Update floor tracking
                if "floor" in ext:
                    floor = max(floor, ext["floor"])

                # Update act
                if floor > 17 and act == 1:
                    act = 2
                elif floor > 34 and act == 2:
                    act = 3
                elif floor > 52 and act == 3:
                    act = 4

                all_extractions.append(ext)

            print(f"    Found {len(extractions)} decisions")

        return all_extractions, seed

    def cleanup(self):
        """Delete uploaded video."""
        if self.video_file:
            try:
                self.client.files.delete(name=self.video_file.name)
            except:
                pass


def main():
    parser = argparse.ArgumentParser(description="Extract full run from VOD")
    parser.add_argument("video", help="Path to video file")
    parser.add_argument("seed", nargs="?", help="Known seed (optional)")
    parser.add_argument("--output", "-o", default="extractions.json",
                        help="Output JSON file")
    parser.add_argument("--duration", type=int, default=55,
                        help="Video duration in minutes")
    parser.add_argument("--chunk", type=int, default=5,
                        help="Chunk size in minutes")

    args = parser.parse_args()

    if not os.path.exists(args.video):
        print(f"Error: Video not found: {args.video}")
        sys.exit(1)

    extractor = FullRunExtractor()

    try:
        extractions, detected_seed = extractor.extract_full_run(
            args.video,
            seed=args.seed,
            chunk_minutes=args.chunk,
            video_duration_minutes=args.duration,
        )

        # Save results
        output = {
            "video": args.video,
            "seed": detected_seed,
            "extraction_timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "total_extractions": len(extractions),
            "extractions": extractions,
        }

        with open(args.output, "w") as f:
            json.dump(output, f, indent=2)

        print(f"\n{'='*60}")
        print("EXTRACTION COMPLETE")
        print(f"{'='*60}")
        print(f"Seed: {detected_seed}")
        print(f"Total extractions: {len(extractions)}")
        print(f"Output: {args.output}")

        # Summary by type
        by_type = {}
        for ext in extractions:
            t = ext.get("type", "unknown")
            by_type[t] = by_type.get(t, 0) + 1

        print("\nBy type:")
        for t, count in sorted(by_type.items()):
            print(f"  {t}: {count}")

    finally:
        extractor.cleanup()


if __name__ == "__main__":
    main()
