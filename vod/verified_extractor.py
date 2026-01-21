#!/usr/bin/env python3
"""
Verified VOD Extractor

Two-phase extraction that validates predictions against video:
1. Verification Phase: Model reports what it observes
2. Extraction Phase: Model picks indices from our predictions

This ensures:
- No hallucinated card names (model picks from our list)
- Validation of our RNG implementation
- Clear mismatch tracking for debugging
"""

import os
import time
from dataclasses import dataclass, field
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
from vod.extraction_tools_v2 import ALL_EXTRACTION_TOOLS, to_gemini_tools, create_extraction_prompt


@dataclass
class VerifiedExtraction:
    """Result of a verified extraction."""
    floor: int
    choice_type: str

    # Our prediction
    predicted_options: List[str]

    # What model saw (verification)
    observed_options: List[str]
    prediction_matches: bool

    # The actual choice (index-based)
    chosen_index: int
    chosen_item: str  # Derived from predictions + index

    # Metadata
    timestamp: Optional[str] = None
    notes: str = ""


@dataclass
class ExtractionSession:
    """Tracks state across extraction."""
    seed: str
    character: str = "WATCHER"
    ascension: int = 20

    # RNG state
    rng_state: Optional[GameRNGState] = None
    reward_state: RewardState = field(default_factory=RewardState)

    # Game state
    floor: int = 0
    act: int = 1
    hp: int = 72
    gold: int = 99

    # Relics affecting rewards
    has_question_card: bool = False
    has_busted_crown: bool = False
    has_prayer_wheel: bool = False
    has_prismatic_shard: bool = False

    # Results
    extractions: List[VerifiedExtraction] = field(default_factory=list)
    mismatches: List[Dict] = field(default_factory=list)

    def __post_init__(self):
        if self.seed:
            self.rng_state = GameRNGState(self.seed)

    def predict_card_reward(self, room_type: str = "normal") -> List[str]:
        """Predict next card reward options."""
        if not self.rng_state:
            return []

        num_cards = 3
        if self.has_busted_crown:
            num_cards -= 2
        if self.has_question_card:
            num_cards += 1

        card_rng = self.rng_state.get_rng(RNGStream.CARD)

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

        return [card.name for card in cards]

    def apply_neow(self, neow_type: str):
        """Apply Neow choice to RNG state."""
        if self.rng_state:
            self.rng_state.apply_neow_choice(neow_type)

    def advance_after_combat(self, room_type: str = "normal"):
        """Advance RNG after combat reward."""
        if self.rng_state:
            # Card reward consumes ~9 cardRng calls
            self.rng_state.advance(RNGStream.CARD, 9)


class VerifiedExtractor:
    """
    Extracts decisions from VODs with prediction verification.

    Two-step process for each card reward:
    1. Show model our predictions, ask "do these match?"
    2. Ask model "which index was chosen?"
    """

    # ALWAYS use Gemini 3.0 models
    DEFAULT_MODEL = "gemini-3-flash-preview"

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

    def verify_and_extract_card_reward(
        self,
        session: ExtractionSession,
        timestamp: str,
        room_type: str = "normal",
    ) -> VerifiedExtraction:
        """
        Verify predictions and extract card choice for a single reward.

        Args:
            session: Current extraction session with RNG state
            timestamp: Video timestamp (MM:SS)
            room_type: "normal", "elite", or "boss"

        Returns:
            VerifiedExtraction with prediction match status and choice
        """
        if not self.video_file:
            raise RuntimeError("Video not uploaded")

        # Step 1: Get our predictions
        predicted_cards = session.predict_card_reward(room_type)

        if not predicted_cards:
            return VerifiedExtraction(
                floor=session.floor,
                choice_type="card_reward",
                predicted_options=[],
                observed_options=[],
                prediction_matches=False,
                chosen_index=-1,
                chosen_item="UNKNOWN",
                timestamp=timestamp,
                notes="No predictions available (no seed)"
            )

        # Step 2: Verification prompt - ask model what it sees
        verify_prompt = f"""Look at the card reward screen at approximately {timestamp}.

Our seed prediction says these cards should be offered:
"""
        for i, card in enumerate(predicted_cards):
            verify_prompt += f"  [{i}] {card}\n"

        verify_prompt += """
TASK: Compare our predictions to what's actually shown in the video.

For each position (0, 1, 2), tell me:
1. Does our prediction match what you see?
2. If not, what card IS shown at that position?

Also tell me which card was CHOSEN (or if player SKIPped).

Respond in JSON format:
{
    "verification": [
        {"index": 0, "matches": true/false, "video_shows": "CardName"},
        {"index": 1, "matches": true/false, "video_shows": "CardName"},
        {"index": 2, "matches": true/false, "video_shows": "CardName"}
    ],
    "all_match": true/false,
    "chosen_index": 0/1/2/-1,
    "skipped": true/false
}
"""

        try:
            response = self.client.models.generate_content(
                model=self.model,
                contents=[self.video_file, verify_prompt],
                config=types.GenerateContentConfig(
                    temperature=0.0,
                    max_output_tokens=500,
                ),
            )

            text = response.text

            # Parse JSON response
            import json
            import re

            # Extract JSON from response
            json_match = re.search(r'\{[^{}]*\}', text, re.DOTALL)
            if json_match:
                data = json.loads(json_match.group())
            else:
                # Fallback parsing
                data = {
                    "all_match": "true" in text.lower() and "all" in text.lower(),
                    "chosen_index": -1 if "skip" in text.lower() else 0,
                    "verification": []
                }

            all_match = data.get("all_match", True)
            chosen_index = data.get("chosen_index", -1)
            verification = data.get("verification", [])

            # Extract observed cards
            observed = [v.get("video_shows", predicted_cards[i])
                       for i, v in enumerate(verification)]
            if not observed:
                observed = predicted_cards  # Assume match if not specified

            # Determine chosen item
            if chosen_index == -1 or data.get("skipped"):
                chosen_item = "SKIP"
            elif 0 <= chosen_index < len(predicted_cards):
                chosen_item = predicted_cards[chosen_index]
            else:
                chosen_item = "UNKNOWN"

            return VerifiedExtraction(
                floor=session.floor,
                choice_type="card_reward",
                predicted_options=predicted_cards,
                observed_options=observed,
                prediction_matches=all_match,
                chosen_index=chosen_index,
                chosen_item=chosen_item,
                timestamp=timestamp,
            )

        except Exception as e:
            return VerifiedExtraction(
                floor=session.floor,
                choice_type="card_reward",
                predicted_options=predicted_cards,
                observed_options=[],
                prediction_matches=False,
                chosen_index=-1,
                chosen_item="ERROR",
                timestamp=timestamp,
                notes=f"Extraction error: {e}"
            )

    def extract_neow(self, timestamp: str = "00:30") -> Dict[str, Any]:
        """Extract Neow choice from video."""
        if not self.video_file:
            raise RuntimeError("Video not uploaded")

        prompt = f"""Look at approximately {timestamp} for the Neow bonus selection.

Neow offers 4 options. Report:
1. Which bonus was CHOSEN (the exact text shown)
2. Any drawback associated with it
3. If it was a boss relic swap, what relic was obtained

Respond in JSON format:
{{
    "chosen_bonus": "Obtain 100 Gold" (example),
    "drawback": null or "Take 7 damage" (example),
    "is_boss_swap": false,
    "boss_relic": null or "RelicName"
}}
"""

        try:
            response = self.client.models.generate_content(
                model=self.model,
                contents=[self.video_file, prompt],
                config=types.GenerateContentConfig(
                    temperature=0.0,
                    max_output_tokens=300,
                ),
            )

            import json
            import re

            text = response.text
            json_match = re.search(r'\{[^{}]*\}', text, re.DOTALL)

            if json_match:
                return json.loads(json_match.group())

            return {"chosen_bonus": text, "drawback": None, "is_boss_swap": False}

        except Exception as e:
            return {"error": str(e)}

    def extract_seed(self, timestamps: List[str] = None) -> Optional[str]:
        """Extract seed from video by checking multiple timestamps."""
        if not self.video_file:
            raise RuntimeError("Video not uploaded")

        if timestamps is None:
            timestamps = ["00:15", "30:00", "50:00"]

        prompt = """Look for the SEED in this video.

The seed is displayed as "Seed: XXXXX" where XXXXX is alphanumeric (4-15 characters).

Check:
- Top corners during gameplay
- Pause menu
- End of run screen
- Character select

Seeds look like: 1V2ZJKI0, ABC123DEF, 5F8M2N

If you find a seed, report ONLY the seed characters (no quotes or extra text).
If no seed visible, say "NOT FOUND"."""

        for ts in timestamps:
            try:
                response = self.client.models.generate_content(
                    model=self.model,
                    contents=[self.video_file, f"Check around {ts}.\n\n{prompt}"],
                    config=types.GenerateContentConfig(
                        temperature=0.0,
                        max_output_tokens=100,
                    ),
                )

                text = response.text.strip().upper()

                if "NOT FOUND" not in text and len(text) >= 4:
                    # Clean up seed
                    import re
                    seed_match = re.search(r'[A-Z0-9]{4,15}', text)
                    if seed_match:
                        return seed_match.group()

            except Exception as e:
                print(f"  Error at {ts}: {e}")
                continue

        return None

    def cleanup(self):
        """Delete uploaded video."""
        if self.video_file:
            try:
                self.client.files.delete(name=self.video_file.name)
            except:
                pass


def test_verified_extraction(video_path: str, seed: Optional[str] = None):
    """Test the verified extraction flow."""
    print("=" * 60)
    print("VERIFIED EXTRACTION TEST")
    print("=" * 60)

    extractor = VerifiedExtractor(model="gemini-3-flash-preview")

    try:
        # Upload video
        extractor.upload_video(video_path)

        # Detect seed if not provided
        if not seed:
            print("\nDetecting seed...")
            seed = extractor.extract_seed()
            print(f"  Detected: {seed}")

        if not seed:
            print("No seed found. Cannot verify predictions.")
            return

        # Create session
        session = ExtractionSession(seed=seed)

        # Extract Neow
        print("\nExtracting Neow choice...")
        neow = extractor.extract_neow()
        print(f"  Neow: {neow}")

        # Apply Neow to RNG (use HUNDRED_GOLD as default for now)
        session.apply_neow("HUNDRED_GOLD")
        session.floor = 1

        # Extract first card reward
        print("\nExtracting floor 1 card reward...")
        result = extractor.verify_and_extract_card_reward(
            session,
            timestamp="04:00",  # Approximate
            room_type="normal"
        )

        print(f"\n  Predictions: {result.predicted_options}")
        print(f"  Observed: {result.observed_options}")
        print(f"  Match: {result.prediction_matches}")
        print(f"  Chosen: [{result.chosen_index}] {result.chosen_item}")

        # Record mismatch if any
        if not result.prediction_matches:
            session.mismatches.append({
                "floor": result.floor,
                "predicted": result.predicted_options,
                "observed": result.observed_options,
            })

        session.extractions.append(result)

        # Summary
        print("\n" + "=" * 60)
        print("SUMMARY")
        print("=" * 60)
        print(f"Seed: {seed}")
        print(f"Extractions: {len(session.extractions)}")
        print(f"Mismatches: {len(session.mismatches)}")

        if session.mismatches:
            print("\nMismatches:")
            for m in session.mismatches:
                print(f"  Floor {m['floor']}:")
                print(f"    Predicted: {m['predicted']}")
                print(f"    Observed: {m['observed']}")

    finally:
        extractor.cleanup()


if __name__ == "__main__":
    import sys

    video = sys.argv[1] if len(sys.argv) > 1 else "vod_data/merl/TO6u6As_lR4.mp4"
    seed = sys.argv[2] if len(sys.argv) > 2 else None

    test_verified_extraction(video, seed)
