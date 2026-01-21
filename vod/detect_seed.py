#!/usr/bin/env python3
"""
Seed Detection Script for Slay the Spire VODs

Multi-point approach: checks beginning, multiple mid-points, and end of video
to reliably detect the seed.

Usage:
    python -m vod.detect_seed path/to/video.mp4
    python -m vod.detect_seed path/to/video.mp4 --model gemini-3-pro-preview
"""

import argparse
import os
import sys
import time
from dataclasses import dataclass
from typing import Optional, List
from collections import Counter

from dotenv import load_dotenv
load_dotenv()

from google import genai
from google.genai import types


@dataclass
class SeedDetection:
    """A single seed detection result."""
    seed: str
    timestamp: str
    location: str
    confidence: str  # high/medium/low


@dataclass
class SeedDetectionResult:
    """Combined result from multi-point seed detection."""
    detected_seed: Optional[str]
    all_detections: List[SeedDetection]
    confidence: float
    raw_responses: List[str]


class SeedDetector:
    """
    Multi-point seed detection for Slay the Spire VODs.

    Checks multiple timestamps throughout the video to reliably
    detect the seed, even if it's only briefly visible.
    """

    # Timestamps to check (MM:SS format)
    CHECK_POINTS = [
        ("00:00", "00:30", "beginning - character select/Neow"),
        ("05:00", "05:30", "early game - might pause"),
        ("15:00", "15:30", "mid act 1"),
        ("30:00", "30:30", "mid game"),
        ("45:00", "45:30", "late game"),
        ("50:00", "55:00", "end of run - victory/defeat screen"),
    ]

    SEED_PROMPT = '''Look at this video segment for the SEED.

The seed is displayed as "Seed: XXXXX" where XXXXX is alphanumeric.

Check:
1. Top-left or top-right corner during gameplay
2. Pause menu (if player presses ESC)
3. End of run screen (victory/defeat)
4. Character select screen

Seeds are 4-15 alphanumeric characters. Examples: 1V2ZJKI0, ABC123DEF, 5F8M2N

If you see a seed, report:
1. SEED: [exact characters]
2. TIMESTAMP: [when visible]
3. LOCATION: [where on screen]

If no seed visible in this segment, say "NO SEED VISIBLE".
Be careful: 0/O, 1/I/l, 5/S can look similar.'''

    def __init__(self, model: str = "gemini-3-flash-preview"):
        self.model = model
        self.client = genai.Client(api_key=os.environ.get("GOOGLE_API_KEY"))
        self.video_file = None

    def upload_video(self, video_path: str, timeout: int = 300) -> None:
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

    def check_segment(self, start: str, end: str, description: str) -> Optional[SeedDetection]:
        """Check a single video segment for the seed."""
        if not self.video_file:
            raise RuntimeError("Video not uploaded")

        prompt = f"{self.SEED_PROMPT}\n\nVideo segment: {start} to {end} ({description})"

        try:
            response = self.client.models.generate_content(
                model=self.model,
                contents=[self.video_file, prompt],
                config=types.GenerateContentConfig(
                    temperature=0.0,
                    max_output_tokens=500,
                ),
            )

            text = response.text.upper()

            # Parse response for seed
            if "SEED:" in text:
                # Extract seed value
                for line in text.split("\n"):
                    if "SEED:" in line and "NO SEED" not in line:
                        seed = line.split("SEED:")[1].strip()
                        # Clean up: remove quotes, brackets, etc.
                        seed = seed.split()[0].strip("\"'[](),.")
                        if seed and len(seed) >= 4:
                            return SeedDetection(
                                seed=seed,
                                timestamp=start,
                                location=description,
                                confidence="high" if len(seed) > 6 else "medium"
                            )

            return None

        except Exception as e:
            print(f"  Error checking {start}-{end}: {e}")
            return None

    def detect(self, video_path: str) -> SeedDetectionResult:
        """
        Run multi-point seed detection on a video.

        Returns the most likely seed based on multiple detections.
        """
        self.upload_video(video_path)

        detections: List[SeedDetection] = []
        raw_responses: List[str] = []

        print(f"\nChecking {len(self.CHECK_POINTS)} video segments...")

        for start, end, desc in self.CHECK_POINTS:
            print(f"  [{start}-{end}] {desc}...")
            detection = self.check_segment(start, end, desc)

            if detection:
                print(f"    Found: {detection.seed}")
                detections.append(detection)
            else:
                print(f"    No seed visible")

        # Determine most likely seed (majority vote)
        if detections:
            seed_counts = Counter(d.seed for d in detections)
            most_common_seed, count = seed_counts.most_common(1)[0]
            confidence = count / len(detections)

            return SeedDetectionResult(
                detected_seed=most_common_seed,
                all_detections=detections,
                confidence=confidence,
                raw_responses=raw_responses,
            )

        return SeedDetectionResult(
            detected_seed=None,
            all_detections=[],
            confidence=0.0,
            raw_responses=raw_responses,
        )

    def cleanup(self):
        """Delete uploaded video file."""
        if self.video_file:
            try:
                self.client.files.delete(name=self.video_file.name)
            except:
                pass


def detect_seed(video_path: str, model: str = "gemini-3-flash-preview") -> SeedDetectionResult:
    """
    Convenience function to detect seed from a video.

    Args:
        video_path: Path to video file
        model: Gemini model to use

    Returns:
        SeedDetectionResult with detected seed and confidence
    """
    detector = SeedDetector(model=model)
    try:
        return detector.detect(video_path)
    finally:
        detector.cleanup()


def main():
    parser = argparse.ArgumentParser(description="Detect seed from Slay the Spire VOD")
    parser.add_argument("video", help="Path to video file")
    parser.add_argument("--model", default="gemini-3-flash-preview",
                        help="Gemini model (default: gemini-3-flash-preview)")

    args = parser.parse_args()

    if not os.path.exists(args.video):
        print(f"Error: Video not found: {args.video}")
        sys.exit(1)

    result = detect_seed(args.video, args.model)

    print("\n" + "=" * 50)
    print("SEED DETECTION RESULTS")
    print("=" * 50)

    if result.detected_seed:
        print(f"Detected Seed: {result.detected_seed}")
        print(f"Confidence: {result.confidence:.0%}")
        print(f"Detections: {len(result.all_detections)}")

        if len(result.all_detections) > 1:
            print("\nAll detections:")
            for d in result.all_detections:
                print(f"  - {d.seed} at {d.timestamp} ({d.location})")
    else:
        print("No seed detected")
        print("Try checking the video manually for pause screens or end-of-run screen")

    return result


if __name__ == "__main__":
    main()
