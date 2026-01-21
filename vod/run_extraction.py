#!/usr/bin/env python3
"""
VOD Extraction Script - Extract decisions from Slay the Spire VOD

Uses the NEW google.genai package (not deprecated google.generativeai).

Extracts: neow choice, path choices, card rewards, shop visits, rest sites, events, boss relics
Includes RNG predictions and mismatch detection with PROPER state tracking.
"""

import json
import os
import sys
import time
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Any, Set, Tuple
from copy import deepcopy

from dotenv import load_dotenv
load_dotenv()

# Import google.genai (the new package)
from google import genai
from google.genai import types

# Add parent to path for core imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.state.game_rng import GameRNGState, RNGStream
from core.generation.rewards import generate_card_rewards, RewardState


# =============================================================================
# RNG STATE TRACKER - Tracks exact RNG consumption
# =============================================================================

@dataclass
class RNGStateTracker:
    """
    Tracks RNG state with exact counter updates.

    Instead of estimated advances, this tracks actual RNG consumption
    by measuring counter before/after operations.
    """
    seed: str
    game_state: GameRNGState = field(init=False)
    reward_state: RewardState = field(default_factory=RewardState)

    # Current game position
    floor: int = 0
    act: int = 1

    # Debug tracking
    rng_history: List[Dict] = field(default_factory=list)

    def __post_init__(self):
        self.game_state = GameRNGState(self.seed)

    def get_card_counter(self) -> int:
        """Get current cardRng counter."""
        return self.game_state.get_counter(RNGStream.CARD)

    def apply_neow(self, option: str, boss_relic: str = None):
        """Apply Neow choice with proper RNG tracking."""
        counter_before = self.get_card_counter()
        self.game_state.apply_neow_choice(option, boss_relic)
        counter_after = self.get_card_counter()

        self.rng_history.append({
            "event": "neow",
            "option": option,
            "card_rng_consumed": counter_after - counter_before,
            "counter_after": counter_after,
        })

    def predict_card_reward(
        self,
        room_type: str = "normal",
        num_cards: int = 3,
    ) -> Tuple[List[str], int]:
        """
        Predict the next card reward WITHOUT advancing state.

        Returns: (card_names, rng_calls_consumed)
        """
        # Create a temporary RNG at current counter position
        card_rng = self.game_state.get_rng(RNGStream.CARD)
        counter_before = card_rng.counter

        # Generate cards (this advances the temporary RNG's counter)
        cards = generate_card_rewards(
            rng=card_rng,
            reward_state=deepcopy(self.reward_state),  # Don't modify actual state
            act=self.act,
            player_class="WATCHER",
            ascension=20,
            room_type=room_type,
            num_cards=num_cards,
        )

        counter_after = card_rng.counter
        rng_consumed = counter_after - counter_before

        return [c.name for c in cards], rng_consumed

    def consume_card_reward(
        self,
        room_type: str = "normal",
        num_cards: int = 3,
    ) -> List[str]:
        """
        Generate card reward AND advance RNG state properly.

        Returns: card_names
        """
        # Get RNG and track counter
        card_rng = self.game_state.get_rng(RNGStream.CARD)
        counter_before = card_rng.counter

        # Generate cards
        cards = generate_card_rewards(
            rng=card_rng,
            reward_state=self.reward_state,  # Modify actual state (blizzard)
            act=self.act,
            player_class="WATCHER",
            ascension=20,
            room_type=room_type,
            num_cards=num_cards,
        )

        # Advance the game state by exact amount consumed
        counter_after = card_rng.counter
        rng_consumed = counter_after - counter_before
        self.game_state.set_counter(RNGStream.CARD, counter_after)

        self.rng_history.append({
            "event": "card_reward",
            "floor": self.floor,
            "room_type": room_type,
            "card_rng_consumed": rng_consumed,
            "counter_after": counter_after,
            "cards": [c.name for c in cards],
        })

        return [c.name for c in cards]

    def apply_shop(self):
        """Apply shop visit RNG consumption."""
        counter_before = self.get_card_counter()
        self.game_state.apply_shop()
        counter_after = self.get_card_counter()

        self.rng_history.append({
            "event": "shop",
            "floor": self.floor,
            "card_rng_consumed": counter_after - counter_before,
            "counter_after": counter_after,
        })

    def apply_event(self, event_name: str = None):
        """Apply event RNG consumption."""
        counter_before = self.get_card_counter()
        self.game_state.apply_event(event_name)
        counter_after = self.get_card_counter()

        self.rng_history.append({
            "event": "event",
            "event_name": event_name,
            "floor": self.floor,
            "card_rng_consumed": counter_after - counter_before,
            "counter_after": counter_after,
        })

    def enter_floor(self, floor_num: int):
        """Enter a new floor."""
        self.floor = floor_num
        self.game_state.enter_floor(floor_num)

    def transition_act(self):
        """Transition to next act with cardRng snapping."""
        counter_before = self.get_card_counter()
        self.game_state.transition_to_next_act()
        self.act = self.game_state.act_num
        counter_after = self.get_card_counter()

        self.rng_history.append({
            "event": "act_transition",
            "new_act": self.act,
            "counter_before": counter_before,
            "counter_after": counter_after,
            "snapped": counter_after != counter_before,
        })

    def reset_with_neow(self, option: str, boss_relic: str = None):
        """Reset state and apply Neow choice (for when Neow detected late)."""
        self.game_state = GameRNGState(self.seed)
        self.reward_state = RewardState()
        self.floor = 0
        self.act = 1
        self.rng_history = []
        self.apply_neow(option, boss_relic)


# =============================================================================
# NEOW OPTION MAPPING
# =============================================================================

NEOW_TEXT_TO_OPTION = {
    # Bonuses (match partial text)
    "100 gold": "HUNDRED_GOLD",
    "receive 100 gold": "HUNDRED_GOLD",
    "upgrade a card": "UPGRADE_CARD",
    "upgrade": "UPGRADE_CARD",
    "max hp": "TEN_PERCENT_HP_BONUS",
    "common relic": "RANDOM_COMMON_RELIC",
    "random rare card": "ONE_RANDOM_RARE_CARD",
    "obtain a random rare card": "ONE_RANDOM_RARE_CARD",
    "remove card": "REMOVE_CARD",
    "remove a card": "REMOVE_CARD",
    "transform": "TRANSFORM_CARD",
    "transform a card": "TRANSFORM_CARD",
    "colorless card": "RANDOM_COLORLESS",
    "random colorless": "RANDOM_COLORLESS",
    "boss relic": "BOSS_SWAP",
    "boss swap": "BOSS_SWAP",
    "swap": "BOSS_SWAP",
    "rare relic": "ONE_RARE_RELIC",
    "250 gold": "TWO_FIFTY_GOLD",
    "receive 250 gold": "TWO_FIFTY_GOLD",
    "three rare cards": "THREE_RARE_CARDS",
    "choose a rare card": "THREE_RARE_CARDS",
    "enemies in your first three": "THREE_ENEMY_KILL",
    "three enemy kill": "THREE_ENEMY_KILL",
    "three cards": "THREE_CARDS",
    "choose a card": "THREE_CARDS",
}


def detect_neow_option(text: str) -> Optional[str]:
    """Detect Neow option from extraction text."""
    text_lower = text.lower()
    for pattern, option in NEOW_TEXT_TO_OPTION.items():
        if pattern in text_lower:
            return option
    return None


# =============================================================================
# VOD EXTRACTOR
# =============================================================================

class VODExtractor:
    """Extract all decisions from a Slay the Spire VOD using Gemini."""

    def __init__(self, model: str = "gemini-3-flash-preview"):
        self.model = model
        api_key = os.environ.get("GOOGLE_API_KEY")
        if not api_key:
            raise ValueError("GOOGLE_API_KEY environment variable not set")
        self.client = genai.Client(api_key=api_key)
        self.video_file = None

    def upload_video(self, video_path: str, timeout: int = 1800):
        """Upload video to Gemini for processing (30 min timeout for large videos)."""
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
        card_rng_counter: int = 0,
    ) -> List[Dict]:
        """Extract decisions from a video chunk."""

        if not self.video_file:
            raise RuntimeError("Video not uploaded - call upload_video first")

        predictions_text = ""
        if predicted_cards:
            predictions_text = f"""

PREDICTED NEXT CARD REWARD (from RNG simulation):
Cards: {predicted_cards}
cardRng counter: {card_rng_counter}

If you see a card reward, report the EXACT cards shown and compare to prediction."""

        prompt = f"""Analyze this Slay the Spire video segment from {start_time} to {end_time}.

You are extracting game decisions for a Watcher run. Return a JSON array of decision objects.

CRITICAL RULES:
1. Only report decisions you can CLEARLY see happen
2. Use EXACT card/relic names as shown in the game (spelling matters!)
3. For card rewards: report ALL cards offered in order, and which was chosen
4. Report events IN CHRONOLOGICAL ORDER within the segment
5. Track floor numbers carefully - they increment as the player progresses

Current game state:
- Floor: {floor}
- Act: {act}
{predictions_text}

Return a JSON array with these object types:

{{"timestamp": "MM:SS", "type": "neow_choice", "floor": 0, "chosen_bonus": "exact bonus text", "drawback": "exact drawback text or none"}}

{{"timestamp": "MM:SS", "type": "path_choice", "floor": N, "room_type": "monster|elite|rest|shop|event|chest|boss"}}

{{"timestamp": "MM:SS", "type": "card_reward", "floor": N, "cards_offered": ["Card1", "Card2", "Card3"], "chosen_card": "CardName or SKIP"}}

{{"timestamp": "MM:SS", "type": "shop_visit", "floor": N, "cards_bought": [], "relics_bought": [], "card_removed": "CardName or null"}}

{{"timestamp": "MM:SS", "type": "rest_site", "floor": N, "action": "rest|smith|dig|recall|toke|lift", "card_upgraded": "CardName or null"}}

{{"timestamp": "MM:SS", "type": "event_choice", "floor": N, "event_name": "Event Name", "choice_made": "choice text"}}

{{"timestamp": "MM:SS", "type": "boss_relic", "floor": N, "options": ["Relic1", "Relic2", "Relic3"], "chosen_relic": "RelicName"}}

{{"timestamp": "MM:SS", "type": "combat_end", "floor": N, "enemy": "EnemyName"}}

ONLY return the JSON array, no other text. Return [] if no decisions visible."""

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


# =============================================================================
# CARD NAME NORMALIZATION
# =============================================================================

def normalize_card_name(name: str) -> str:
    """Normalize card name for comparison."""
    name = name.replace("+", "").strip()
    # Common OCR/transcription errors
    name = name.replace("Judgement", "Judgment")  # Alternate spelling
    return name.lower()


def check_card_match(predicted: List[str], observed: List[str]) -> Tuple[bool, float]:
    """
    Check if predicted cards match observed cards.

    Returns: (is_match, match_score)
    - is_match: True if all cards match
    - match_score: 0.0-1.0 indicating how many matched
    """
    if not predicted or not observed:
        return False, 0.0

    pred_normalized = [normalize_card_name(c) for c in predicted]
    obs_normalized = [normalize_card_name(c) for c in observed]

    # Check exact order match
    if len(pred_normalized) == len(obs_normalized):
        if all(p == o for p, o in zip(pred_normalized, obs_normalized)):
            return True, 1.0

    # Check set match (order doesn't matter)
    pred_set = set(pred_normalized)
    obs_set = set(obs_normalized)

    if pred_set == obs_set:
        return True, 1.0

    # Partial match
    intersection = pred_set & obs_set
    union = pred_set | obs_set
    score = len(intersection) / len(union) if union else 0.0

    return score >= 0.5, score  # Consider 50%+ overlap a partial match


# =============================================================================
# MAIN EXTRACTION
# =============================================================================

def run_extraction(
    video_path: str,
    seed: str,
    output_path: str,
    chunk_minutes: int = 5,
    video_duration_minutes: int = 60,
):
    """
    Run full extraction on a video with proper RNG tracking.

    Key improvements:
    1. Track exact RNG consumption (not estimates)
    2. Generate prediction for each card reward individually
    3. Apply Neow choice as soon as detected
    4. Handle shop/event RNG consumption properly
    """

    print(f"Starting extraction with proper RNG tracking")
    print(f"  Video: {video_path}")
    print(f"  Seed: {seed}")
    print(f"  Output: {output_path}")
    print(f"  Chunks: {chunk_minutes}min, Duration: {video_duration_minutes}min")
    print()

    # Initialize extractor
    extractor = VODExtractor()

    # Upload video (extended timeout)
    extractor.upload_video(video_path, timeout=1800)

    # Initialize RNG tracker
    rng_tracker = RNGStateTracker(seed)
    neow_detected = False

    all_extractions = []
    processed_timestamps = set()  # Avoid duplicate processing

    # Process in chunks
    total_chunks = (video_duration_minutes + chunk_minutes - 1) // chunk_minutes
    print(f"Processing {total_chunks} chunks...")
    print()

    for chunk_idx in range(total_chunks):
        start_min = chunk_idx * chunk_minutes
        end_min = min((chunk_idx + 1) * chunk_minutes, video_duration_minutes)
        start_time = f"{start_min:02d}:00"
        end_time = f"{end_min:02d}:00"

        # Get prediction for NEXT card reward at current state
        predicted_cards, _ = rng_tracker.predict_card_reward()
        card_counter = rng_tracker.get_card_counter()

        print(f"[{chunk_idx+1}/{total_chunks}] {start_time} - {end_time}")
        print(f"  cardRng counter: {card_counter}")
        print(f"  Next prediction: {predicted_cards}")

        # Extract chunk
        extractions = extractor.extract_chunk(
            start_time, end_time,
            floor=rng_tracker.floor,
            act=rng_tracker.act,
            predicted_cards=predicted_cards,
            card_rng_counter=card_counter,
        )

        # Process extractions in order
        for ext in extractions:
            ext_type = ext.get("type", "")
            timestamp = ext.get("timestamp", "")
            ext_floor = ext.get("floor", rng_tracker.floor)

            # Skip duplicates
            ext_key = f"{ext_type}:{timestamp}:{ext_floor}"
            if ext_key in processed_timestamps:
                continue
            processed_timestamps.add(ext_key)

            # Update floor if needed
            if ext_floor > rng_tracker.floor:
                rng_tracker.enter_floor(ext_floor)

            # Handle Neow choice - MUST be processed first
            if ext_type == "neow_choice" and not neow_detected:
                chosen = ext.get("chosen_bonus", "")
                option = detect_neow_option(chosen)
                if option:
                    print(f"  NEOW detected: {chosen} -> {option}")
                    rng_tracker.reset_with_neow(option)
                    neow_detected = True
                    # Re-predict with correct Neow
                    predicted_cards, _ = rng_tracker.predict_card_reward()
                    print(f"  New prediction after Neow: {predicted_cards}")

            # Handle shop (BEFORE checking card_reward because shop enters affect cardRng)
            elif ext_type == "shop_visit":
                rng_tracker.apply_shop()
                print(f"  Shop visit at floor {ext_floor}, counter now: {rng_tracker.get_card_counter()}")

            # Handle event
            elif ext_type == "event_choice":
                event_name = ext.get("event_name", "")
                rng_tracker.apply_event(event_name)
                print(f"  Event '{event_name}' at floor {ext_floor}")

            # Handle card reward - THE KEY PART
            elif ext_type == "card_reward":
                observed = ext.get("cards_offered", [])

                # Generate fresh prediction at current state
                predicted, rng_consumed = rng_tracker.predict_card_reward()

                # Check match
                is_match, score = check_card_match(predicted, observed)
                ext["predicted_options"] = predicted
                ext["match_score"] = score
                ext["mismatch"] = not is_match
                ext["rng_counter_before"] = rng_tracker.get_card_counter()

                if is_match:
                    print(f"  ✓ Card reward MATCH at floor {ext_floor}: {observed}")
                else:
                    print(f"  ✗ MISMATCH at floor {ext_floor}!")
                    print(f"    Predicted: {predicted}")
                    print(f"    Observed:  {observed}")
                    print(f"    Counter:   {rng_tracker.get_card_counter()}")

                # CONSUME the card reward RNG (advance state)
                rng_tracker.consume_card_reward()
                ext["rng_counter_after"] = rng_tracker.get_card_counter()

            # Handle path choice (for room type tracking)
            elif ext_type == "path_choice":
                room_type = ext.get("room_type", "")
                # Note: actual RNG consumption happens when entering the room
                # Shop/event handled above, combat card reward handled above

            # Handle act transition
            if ext_floor == 17 and rng_tracker.act == 1:
                rng_tracker.transition_act()
                print(f"  ACT TRANSITION to Act 2, counter snapped to: {rng_tracker.get_card_counter()}")
            elif ext_floor == 34 and rng_tracker.act == 2:
                rng_tracker.transition_act()
                print(f"  ACT TRANSITION to Act 3, counter snapped to: {rng_tracker.get_card_counter()}")
            elif ext_floor == 52 and rng_tracker.act == 3:
                rng_tracker.transition_act()
                print(f"  ACT TRANSITION to Act 4, counter snapped to: {rng_tracker.get_card_counter()}")

            all_extractions.append(ext)

        print(f"  Found {len(extractions)} decisions")
        print()

    # Clean up
    extractor.cleanup()

    # Calculate results
    card_rewards = [e for e in all_extractions if e.get("type") == "card_reward"]
    mismatches = [e for e in card_rewards if e.get("mismatch", False)]
    matches = [e for e in card_rewards if not e.get("mismatch", True)]

    match_rate = len(matches) / len(card_rewards) * 100 if card_rewards else 0

    # Save results
    output = {
        "video": video_path,
        "seed": seed,
        "extraction_timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "total_extractions": len(all_extractions),
        "card_rewards": len(card_rewards),
        "matches": len(matches),
        "mismatches": len(mismatches),
        "match_rate": f"{match_rate:.1f}%",
        "rng_history": rng_tracker.rng_history,
        "extractions": all_extractions,
    }

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(output, f, indent=2)

    print()
    print("=" * 60)
    print("EXTRACTION COMPLETE")
    print("=" * 60)
    print(f"Seed: {seed}")
    print(f"Total extractions: {len(all_extractions)}")
    print(f"Card rewards: {len(card_rewards)}")
    print(f"  Matches: {len(matches)}")
    print(f"  Mismatches: {len(mismatches)}")
    print(f"  Match rate: {match_rate:.1f}%")
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
        for m in mismatches[:10]:  # Show first 10
            print(f"  Floor {m.get('floor', '?')}: Predicted {m.get('predicted_options', [])} vs Observed {m.get('cards_offered', [])}")
        if len(mismatches) > 10:
            print(f"  ... and {len(mismatches) - 10} more")

    return all_extractions, mismatches


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Extract VOD with RNG verification")
    parser.add_argument("--video", default="vod_data/merl/run7_first_hour.mkv", help="Video path")
    parser.add_argument("--seed", default="227QYN385T72G", help="Seed string")
    parser.add_argument("--output", default="vod_data/merl/run7_extraction.json", help="Output path")
    parser.add_argument("--duration", type=int, default=60, help="Video duration in minutes")
    parser.add_argument("--chunk", type=int, default=5, help="Chunk size in minutes")

    args = parser.parse_args()

    run_extraction(
        video_path=args.video,
        seed=args.seed,
        output_path=args.output,
        chunk_minutes=args.chunk,
        video_duration_minutes=args.duration,
    )
