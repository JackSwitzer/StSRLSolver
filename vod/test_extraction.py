#!/usr/bin/env python3
"""
Test script for VOD extraction on Merl video.

Usage:
    # Mock test (no API calls)
    python -m vod.test_extraction --mock

    # Real extraction with dynamic chunking
    python -m vod.test_extraction

    # Static chunking (faster)
    python -m vod.test_extraction --static

    # Use Pro model for scan pass
    python -m vod.test_extraction --scan-model gemini-1.5-pro
"""

import argparse
import json
import os
import sys

# Add project root to path
project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, project_root)

# Load .env from project root
from dotenv import load_dotenv
load_dotenv(os.path.join(project_root, ".env"))


def test_mock_extraction():
    """Test extraction flow with mock extractor."""
    from vod.state import VODRunState
    from vod.handlers import ToolHandler
    from vod.voting import VotingEngine
    from vod.chunker import VideoChunker, DynamicChunker, DetectedEvent, ScanResult

    print("Testing mock extraction flow...")

    # Create mock scan result
    mock_events = [
        DetectedEvent("neow", 0, "00:10", "00:25"),
        DetectedEvent("combat", 1, "00:30", "02:00", enemy="Jaw Worm"),
        DetectedEvent("combat", 2, "02:30", "04:00", enemy="2 Louses"),
        DetectedEvent("shop", 3, "04:30", "05:00"),
        DetectedEvent("combat", 4, "05:30", "07:00", enemy="Gremlin Gang"),
        DetectedEvent("rest", 5, "07:30", "08:00"),
        DetectedEvent("elite", 6, "08:30", "11:00", enemy="Lagavulin"),
        DetectedEvent("combat", 7, "11:30", "13:00", enemy="Cultist"),
        DetectedEvent("boss", 16, "18:00", "22:00", enemy="Slime Boss"),
        DetectedEvent("boss_relic", 16, "22:00", "22:30"),
    ]

    scan_result = ScanResult(
        events=mock_events,
        final_floor=16,
        victory=False,
        heart_kill=False,
        total_duration="22:30",
        seed="ABC123",
    )

    # Create dynamic chunks
    chunker = DynamicChunker()
    chunks = chunker.create_chunks(scan_result)

    print(f"\nCreated {len(chunks)} chunks from scan:")
    for i, chunk in enumerate(chunks):
        print(f"  {i+1}. {chunk}")
        if chunk.events:
            for e in chunk.events:
                print(f"      - {e.timestamp_start}: {e.event_type} (floor {e.floor})")

    # Test state management
    state = VODRunState.create(
        video_id="test_video",
        seed="ABC123",
    )
    handler = ToolHandler(state)

    # Simulate tool calls
    handler.handle("neow", {"chosen": "Choose a Rare Card", "drawback": "Lose all Gold"})
    handler.handle("path", {"floor": 1, "chosen": "monster"})
    handler.handle("combat_start", {"floor": 1, "enemy": "Jaw Worm"})
    handler.handle("combat_turn", {"turn": 1, "cards": ["Eruption", "Strike", "Vigilance"]})
    handler.handle("combat_end", {"hp": 65, "gold_gained": 15})
    handler.handle("card_reward", {"floor": 1, "options": ["Crescendo", "Tantrum", "Evaluate"], "chosen": "Tantrum"})

    print(f"\nState after mock calls:")
    print(f"  Floor: {state.floor}")
    print(f"  HP: {state.hp}/{state.max_hp}")
    print(f"  Deck size: {len(state.deck)}")
    print(f"  Decisions logged: {len(state.decisions)}")

    # Test voting
    from vod.state import DecisionLog, DecisionType

    pass1 = [
        DecisionLog(DecisionType.CARD_REWARD, 1, "01:05",
                   {"options": ["A", "B", "C"], "chosen": "A"}, pass_number=1),
    ]
    pass2 = [
        DecisionLog(DecisionType.CARD_REWARD, 1, "01:05",
                   {"options": ["A", "B", "C"], "chosen": "A"}, pass_number=2),
    ]
    pass3 = [
        DecisionLog(DecisionType.CARD_REWARD, 1, "01:05",
                   {"options": ["A", "B", "C"], "chosen": "B"}, pass_number=3),
    ]

    engine = VotingEngine()
    engine.add_pass(pass1)
    engine.add_pass(pass2)
    engine.add_pass(pass3)
    result = engine.vote()

    print(f"\nVoting test (3 passes, 2 agree on 'A'):")
    print(f"  Winner: {result.decisions[0].data.get('chosen')}")
    print(f"  Confidence: {result.decisions[0].confidence:.1%}")

    print("\nMock extraction test PASSED")
    return True


def test_real_extraction(
    video_path: str,
    chunking: str = "dynamic",
    scan_model: str = "gemini-3-flash-preview",
    passes: int = 1,
    transcript_path: str = None,
    limit_chunks: int = None,
):
    """Test real extraction on video file."""
    from vod.orchestrator import extract_vod, print_extraction_summary

    print(f"Testing real extraction on: {video_path}")
    print(f"  Chunking: {chunking}")
    print(f"  Scan model: {scan_model}")
    print(f"  Passes: {passes}")
    if transcript_path:
        print(f"  Transcript: {transcript_path}")
    if limit_chunks:
        print(f"  Limit: first {limit_chunks} chunks")

    # Check if video exists
    if not os.path.exists(video_path):
        print(f"ERROR: Video not found: {video_path}")
        return False

    # Check for API key
    if not os.environ.get("GOOGLE_API_KEY"):
        print("ERROR: GOOGLE_API_KEY environment variable not set")
        return False

    # For micro chunking, check transcript
    if chunking == "micro" and not transcript_path:
        print("ERROR: --transcript required for micro chunking")
        return False

    result = extract_vod(
        video_path=video_path,
        chunking=chunking,
        scan_model=scan_model,
        passes=passes,
        output_dir="data/vod_extractions",
        transcript_path=transcript_path,
    )

    print_extraction_summary(result)

    print(f"\nResults saved to: data/vod_extractions/{result.video_id}_extraction.json")
    return True


def main():
    parser = argparse.ArgumentParser(description="Test VOD extraction")
    parser.add_argument("--mock", action="store_true", help="Run mock test (no API calls)")
    parser.add_argument("--static", action="store_true", help="Use static chunking")
    parser.add_argument("--micro", action="store_true", help="Use micro chunking (requires --transcript)")
    parser.add_argument("--transcript", type=str,
                       help="Path to transcript JSON (required for micro chunking)")
    parser.add_argument("--scan-model", default="gemini-3-flash-preview",
                       help="Model for scan pass (e.g., gemini-1.5-pro)")
    parser.add_argument("--passes", type=int, default=1,
                       help="Number of extraction passes per chunk")
    parser.add_argument("--video", type=str,
                       default="/Users/jackswitzer/Desktop/SlayTheSpireRL/vod_data/test/test_360p.mp4",
                       help="Path to video file")
    parser.add_argument("--limit", type=int, default=None,
                       help="Limit to first N chunks (for testing)")

    args = parser.parse_args()

    if args.mock:
        success = test_mock_extraction()
    else:
        if args.micro:
            chunking = "micro"
        elif args.static:
            chunking = "static"
        else:
            chunking = "dynamic"

        success = test_real_extraction(
            video_path=args.video,
            chunking=chunking,
            scan_model=args.scan_model,
            passes=args.passes,
            transcript_path=args.transcript,
            limit_chunks=args.limit,
        )

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
