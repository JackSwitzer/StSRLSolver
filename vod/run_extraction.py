#!/usr/bin/env python3
"""
VOD Extraction Script - Extract decisions from Slay the Spire VOD

Uses the NEW google.genai package (not deprecated google.generativeai).

Extracts: neow choice, path choices, card rewards, shop visits, rest sites, events, boss relics
Includes RNG predictions and mismatch detection.
"""

import json
import os
import sys
import time
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Any, Set

from dotenv import load_dotenv
load_dotenv()

# Import google.genai (the new package)
from google import genai
from google.genai import types

# Add parent to path for core imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.state.game_rng import GameRNGState, RNGStream
from core.generation.rewards import generate_card_rewards, RewardState


class VODExtractor:
    """Extract all decisions from a Slay the Spire VOD using Gemini."""

    def __init__(self, model: str = "gemini-3-flash-preview"):
        self.model = model
        api_key = os.environ.get("GOOGLE_API_KEY")
        if not api_key:
            raise ValueError("GOOGLE_API_KEY environment variable not set")
        self.client = genai.Client(api_key=api_key)
        self.video_file = None

    def upload_video(self, video_path: str, timeout: int = 600):
        """Upload video to Gemini for processing."""
        print(f"Uploading video: {video_path}")
        start = time.time()

        self.video_file = self.client.files.upload(file=video_path)

        # Wait for video processing to complete
        while True:
            self.video_file = self.client.files.get(name=self.video_file.name)
            state = getattr(self.video_file, 'state', None)
            if state:
                state_name = getattr(state, 'name', str(state))
                if state_name == "ACTIVE":
                    break
            if time.time() - start > timeout:
                raise TimeoutError(f"Video processing timeout after {timeout}s")
            elapsed = time.time() - start
            print(f"  Processing... ({elapsed:.0f}s)")
            time.sleep(10)

        print(f"Video ready in {time.time() - start:.1f}s")
        return self.video_file

    def extract_chunk(
        self,
        start_time: str,
        end_time: str,
        floor: int,
        act: int,
        predicted_cards: Optional[List[str]] = None,
    ) -> List[Dict]:
        """Extract decisions from a video chunk."""

        if not self.video_file:
            raise RuntimeError("Video not uploaded - call upload_video first")

        predictions_text = ""
        if predicted_cards:
            predictions_text = f"\n\nPREDICTED NEXT CARD REWARD (from RNG): {predicted_cards}\nIf you see a card reward, check if these cards match. Report any mismatch."

        prompt = f"""Analyze this Slay the Spire video segment from {start_time} to {end_time}.

You are extracting game decisions for a Watcher run. Return a JSON array of decision objects.

IMPORTANT RULES:
1. Only report decisions you can CLEARLY see happen
2. Use exact card/relic names as shown in the game
3. For card rewards: report timestamp, cards offered, and chosen card (or "SKIP" if skipped)
4. For paths: report floor number and room type chosen
5. For events: report event name and choice made
6. For shops: report any purchases and card removal
7. For rest sites: report action (rest/smith/dig/recall/toke/lift)
8. For boss relics: report options shown and which was chosen

Current game state estimate:
- Floor: {floor}
- Act: {act}
{predictions_text}

Return a JSON array with objects like:
- {{"timestamp": "MM:SS", "type": "neow_choice", "floor": 0, "chosen_bonus": "...", "drawback": "..."}}
- {{"timestamp": "MM:SS", "type": "path_choice", "floor": N, "room_type": "monster|elite|rest|shop|event|chest|boss"}}
- {{"timestamp": "MM:SS", "type": "card_reward", "floor": N, "cards_offered": ["Card1", "Card2", "Card3"], "chosen_card": "Card1|SKIP", "chosen_index": 0-2 or -1 for skip}}
- {{"timestamp": "MM:SS", "type": "rest_site", "floor": N, "action": "rest|smith", "card_upgraded": "CardName" if smith}}
- {{"timestamp": "MM:SS", "type": "shop_visit", "floor": N, "cards_bought": [], "relics_bought": [], "card_removed": "..."}}
- {{"timestamp": "MM:SS", "type": "event_choice", "floor": N, "event_name": "...", "choice_made": "..."}}
- {{"timestamp": "MM:SS", "type": "boss_relic", "floor": N, "options": ["Relic1", "Relic2", "Relic3"], "chosen_relic": "..."}}
- {{"timestamp": "MM:SS", "type": "combat_end", "floor": N, "enemy": "EnemyName"}}
- {{"timestamp": "MM:SS", "type": "run_end", "floor": N, "victory": true|false}}

ONLY return the JSON array, no other text. If no decisions in this segment, return []."""

        try:
            response = self.client.models.generate_content(
                model=self.model,
                contents=[self.video_file, prompt],
                config=types.GenerateContentConfig(
                    temperature=0.1,
                    max_output_tokens=8192,
                ),
            )

            # Parse JSON response
            text = response.text.strip()

            # Clean up response - extract JSON array
            if text.startswith("```json"):
                text = text[7:]
            if text.startswith("```"):
                text = text[3:]
            if text.endswith("```"):
                text = text[:-3]
            text = text.strip()

            if not text or text == "[]":
                return []

            try:
                extractions = json.loads(text)
                if not isinstance(extractions, list):
                    extractions = [extractions]
                return extractions
            except json.JSONDecodeError as e:
                print(f"  JSON parse error: {e}")
                print(f"  Raw response: {text[:500]}...")
                return []

        except Exception as e:
            print(f"  Error extracting {start_time}-{end_time}: {e}")
            return []

    def cleanup(self):
        """Delete uploaded video from Gemini."""
        if self.video_file:
            try:
                self.client.files.delete(name=self.video_file.name)
                print("Cleaned up video file")
            except Exception as e:
                print(f"Warning: Could not delete video file: {e}")


def normalize_card_name(name: str) -> str:
    """Normalize card name for comparison."""
    # Remove upgrade indicator
    name = name.replace("+", "").strip()
    # Lowercase for comparison
    return name.lower()


def check_card_mismatch(predicted: List[str], observed: List[str]) -> bool:
    """Check if predicted cards match observed cards."""
    if not predicted or not observed:
        return False

    pred_normalized = set(normalize_card_name(c) for c in predicted)
    obs_normalized = set(normalize_card_name(c) for c in observed)

    # Check if any predicted card is in observed
    intersection = pred_normalized & obs_normalized

    # Mismatch if no overlap at all
    return len(intersection) == 0


def run_extraction(
    video_path: str,
    seed: str,
    output_path: str,
    chunk_minutes: int = 5,
    video_duration_minutes: int = 55,
):
    """Run full extraction on a video."""

    print(f"Starting extraction")
    print(f"  Video: {video_path}")
    print(f"  Seed: {seed}")
    print(f"  Output: {output_path}")
    print(f"  Chunks: {chunk_minutes}min, Duration: {video_duration_minutes}min")
    print()

    # Initialize extractor
    extractor = VODExtractor()

    # Upload video
    extractor.upload_video(video_path)

    # Initialize RNG state
    rng_state = GameRNGState(seed)
    # Default Neow assumption - 100 gold (safe, no cardRng consumption)
    rng_state.apply_neow_choice("HUNDRED_GOLD")

    reward_state = RewardState()

    all_extractions = []
    floor = 0
    act = 1
    combat_count = 0  # Track combats for card prediction

    # Process in chunks
    total_chunks = (video_duration_minutes + chunk_minutes - 1) // chunk_minutes
    print(f"Processing {total_chunks} chunks...")

    for i in range(total_chunks):
        start_min = i * chunk_minutes
        end_min = min((i + 1) * chunk_minutes, video_duration_minutes)
        start_time = f"{start_min:02d}:00"
        end_time = f"{end_min:02d}:00"

        # Generate card prediction for this chunk
        predicted_cards = None
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
            predicted_cards = [c.name for c in cards]
        except Exception as e:
            print(f"  Warning: Could not generate prediction: {e}")

        print(f"  [{i+1}/{total_chunks}] {start_time} - {end_time}")
        if predicted_cards:
            print(f"    Predicted cards: {predicted_cards}")

        # Extract chunk
        extractions = extractor.extract_chunk(
            start_time, end_time,
            floor=floor, act=act,
            predicted_cards=predicted_cards,
        )

        # Process extractions
        for ext in extractions:
            ext_type = ext.get("type", "")

            # Add predictions and check for mismatch on card rewards
            if ext_type == "card_reward":
                ext["predicted_options"] = predicted_cards or []

                observed = ext.get("cards_offered", [])
                if predicted_cards and observed:
                    mismatch = check_card_mismatch(predicted_cards, observed)
                    ext["mismatch"] = mismatch
                    if mismatch:
                        print(f"    MISMATCH! Predicted: {predicted_cards}, Observed: {observed}")
                else:
                    ext["mismatch"] = False

                # Advance RNG state for next card reward
                # Combat consumes ~9 cardRng calls
                rng_state.advance(RNGStream.CARD, 9)
                combat_count += 1

            # Update floor tracking
            if "floor" in ext:
                new_floor = ext["floor"]
                if new_floor > floor:
                    floor = new_floor
                    rng_state.enter_floor(floor)

            # Track room types for RNG advancement
            if ext_type == "path_choice":
                room_type = ext.get("room_type", "")
                if room_type == "shop":
                    rng_state.apply_shop()
                elif room_type == "event":
                    event_name = ext.get("event_name", "")
                    rng_state.apply_event(event_name)
                elif room_type == "rest":
                    rng_state.apply_rest()
                elif room_type in ("monster", "elite"):
                    # Combat handled by card_reward
                    pass

            # Handle Neow choice
            if ext_type == "neow_choice":
                chosen = ext.get("chosen_bonus", "")
                # Try to match to known option
                neow_map = {
                    "100 gold": "HUNDRED_GOLD",
                    "upgrade": "UPGRADE_CARD",
                    "max hp": "TEN_PERCENT_HP_BONUS",
                    "common relic": "RANDOM_COMMON_RELIC",
                    "random rare card": "ONE_RANDOM_RARE_CARD",
                    "remove card": "REMOVE_CARD",
                    "transform": "TRANSFORM_CARD",
                    "colorless": "RANDOM_COLORLESS",
                    "boss swap": "BOSS_SWAP",
                    "rare relic": "ONE_RARE_RELIC",
                }
                for key, option in neow_map.items():
                    if key in chosen.lower():
                        # Re-apply Neow choice with correct option
                        # Reset and reapply
                        rng_state = GameRNGState(seed)
                        rng_state.apply_neow_choice(option)
                        break

            # Update act
            if floor > 17 and act == 1:
                act = 2
                rng_state.transition_to_next_act()
            elif floor > 34 and act == 2:
                act = 3
                rng_state.transition_to_next_act()
            elif floor > 52 and act == 3:
                act = 4

            all_extractions.append(ext)

        print(f"    Found {len(extractions)} decisions")

    # Clean up
    extractor.cleanup()

    # Count mismatches
    mismatches = [e for e in all_extractions if e.get("mismatch", False)]

    # Save results
    output = {
        "video": video_path,
        "seed": seed,
        "extraction_timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "total_extractions": len(all_extractions),
        "total_mismatches": len(mismatches),
        "extractions": all_extractions,
    }

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(output, f, indent=2)

    print()
    print("=" * 60)
    print("EXTRACTION COMPLETE")
    print("=" * 60)
    print(f"Seed: {seed}")
    print(f"Total extractions: {len(all_extractions)}")
    print(f"Total mismatches: {len(mismatches)}")
    print(f"Output: {output_path}")

    # Summary by type
    by_type = {}
    for ext in all_extractions:
        t = ext.get("type", "unknown")
        by_type[t] = by_type.get(t, 0) + 1

    print("\nBy type:")
    for t, count in sorted(by_type.items()):
        print(f"  {t}: {count}")

    if mismatches:
        print(f"\nMismatches ({len(mismatches)}):")
        for m in mismatches:
            print(f"  Floor {m.get('floor', '?')}: Predicted {m.get('predicted_options', [])} vs Observed {m.get('cards_offered', [])}")

    return all_extractions, mismatches


if __name__ == "__main__":
    # Default paths for this specific task
    VIDEO_PATH = "/Users/jackswitzer/Desktop/SlayTheSpireRL/vod_data/merl/TO6u6As_lR4.mp4"
    SEED = "1V2ZJKI0"
    OUTPUT_PATH = "/Users/jackswitzer/Desktop/SlayTheSpireRL/vod/verify_ui/extractions.json"

    run_extraction(
        video_path=VIDEO_PATH,
        seed=SEED,
        output_path=OUTPUT_PATH,
        chunk_minutes=5,
        video_duration_minutes=55,
    )
