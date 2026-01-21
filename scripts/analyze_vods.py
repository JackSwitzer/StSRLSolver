#!/usr/bin/env python3
"""CLI for analyzing Slay the Spire VODs for training data extraction.

Usage:
    # Analyze a single video with transcript
    uv run python scripts/analyze_vods.py --video NT4GitAXwxs --streamer merl61

    # Analyze YouTube video directly with Gemini (requires GOOGLE_API_KEY)
    uv run python scripts/analyze_vods.py --video-url "https://youtube.com/watch?v=..." --gemini

    # Analyze all known videos for a streamer
    uv run python scripts/analyze_vods.py --streamer merl61 --max-videos 5

    # Process Baalorlord Spirelogs data
    uv run python scripts/analyze_vods.py --baalorlord

    # Test with a transcript chunk
    uv run python scripts/analyze_vods.py --test-chunk

    # List available streamers and videos
    uv run python scripts/analyze_vods.py --list

Requirements:
    - OPENROUTER_API_KEY or GOOGLE_API_KEY environment variable
    - uv pip install -e ".[vod]"
"""

import argparse
import json
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

# Auto-load environment variables
from core.training.env import load_env, ensure_api_keys
load_env()


def test_llm_connection():
    """Test that OpenRouter connection works."""
    from core.training.llm_client import OpenRouterClient, get_client

    print("Testing OpenRouter connection...")
    try:
        with get_client() as client:
            response = client.complete(
                "Say 'Hello' and nothing else.",
                model=OpenRouterClient.GEMINI_PRO,
                max_tokens=10,
            )
            print(f"  Response: {response.content}")
            print(f"  Model: {response.model}")
            print("  Connection OK!")
            return True
    except Exception as e:
        print(f"  Error: {e}")
        return False


def test_gemini_connection():
    """Test native Gemini connection."""
    from core.training.gemini_client import GeminiClient

    print("Testing native Gemini connection...")
    try:
        client = GeminiClient()
        response = client.complete("Say 'Hello' and nothing else.")
        print(f"  Response: {response.content}")
        print(f"  Model: {response.model}")
        print("  Connection OK!")
        return True
    except Exception as e:
        print(f"  Error: {e}")
        return False


def test_chunk_extraction():
    """Test extraction on a sample transcript chunk."""
    from core.training.decision_extractor import DecisionExtractor

    sample_text = """
    um i like two late elites here late elites and a lot of late combats
    that's not the best but it's okay we also have potential to skip an elite for a shop
    if we want to do that um i think that boss swap is obviously generally speaking
    quite strong on watcher i think that transform is okay i like common relic as well
    i think transforming a defend is probably what we're going to end up going with here

    okay so eruption tantrum and vigilance this is a really solid card reward
    tantrum is amazing for watcher because it gives you wrath entry and draws a card
    i'm going to take tantrum here over eruption since we already have one

    floor 3 now and we got cut through fate third eye and battle hymn
    cut through fate is insane card draw plus scry so definitely taking that
    """

    print("\nTesting decision extraction on sample transcript...")
    print("-" * 60)

    with DecisionExtractor() as extractor:
        run = extractor.extract_from_text(sample_text, "test")

        print(f"\nExtracted {len(run.decisions)} decisions:")
        for d in run.decisions:
            print(f"  [{d.type.value}] Floor {d.floor}: {d.chosen}")
            print(f"    Options: {d.options}")
            print(f"    Reasoning: {d.reasoning[:100]}...")

        print(f"\nExtracted {len(run.combat_lines)} combat lines:")
        for c in run.combat_lines:
            print(f"  Floor {c.floor} Turn {c.turn}: {' -> '.join(c.actions)}")


def list_available():
    """List available streamers and videos."""
    from core.training.youtube_client import YouTubeClient

    client = YouTubeClient()

    print("\nAvailable streamers and videos:")
    print("-" * 60)

    for streamer in ["merl61", "lifecoach", "baalorlord"]:
        videos = client.list_available_videos(streamer)
        print(f"\n{streamer}:")
        if videos:
            for vid in videos:
                print(f"  - {vid} (https://youtube.com/watch?v={vid})")
        else:
            print("  (no videos configured)")


def analyze_video(video_id: str, streamer: str, extract_frames: bool = False):
    """Analyze a single video via transcript."""
    from core.training.vod_analyzer import VODAnalyzer

    print(f"\nAnalyzing video: {video_id}")
    print(f"Streamer: {streamer}")
    print("-" * 60)

    with VODAnalyzer() as analyzer:
        result = analyzer.analyze_video(
            video_id,
            streamer,
            extract_frames=extract_frames,
        )

        if result.game_run:
            print(f"\nResults:")
            print(f"  Decisions extracted: {len(result.game_run.decisions)}")
            print(f"  Combat lines: {len(result.game_run.combat_lines)}")

            from collections import Counter
            types = Counter(d.type.value for d in result.game_run.decisions)
            print(f"\n  Decision types:")
            for t, count in types.most_common():
                print(f"    {t}: {count}")
        else:
            print("  No decisions extracted")


def analyze_video_with_gemini(video_url: str):
    """Analyze YouTube video directly using Gemini's video understanding."""
    from core.training.gemini_client import GeminiClient, STS_VIDEO_ANALYSIS_PROMPT

    print(f"\nAnalyzing video with Gemini: {video_url}")
    print("-" * 60)

    client = GeminiClient()

    print("Sending video to Gemini (this may take a minute)...")
    response = client.analyze_youtube_video(
        video_url,
        STS_VIDEO_ANALYSIS_PROMPT,
        model=GeminiClient.FLASH,
        low_resolution=True,
    )

    print(f"\nTokens used: {response.usage}")
    print(f"\nAnalysis:")
    print(response.content[:2000])

    # Save full response
    output_path = Path("data/vod_analysis/gemini_analysis.json")
    output_path.parent.mkdir(parents=True, exist_ok=True)

    try:
        data = client.extract_json(response)
        with open(output_path, "w") as f:
            json.dump(data, f, indent=2)
        print(f"\nStructured data saved to {output_path}")
    except json.JSONDecodeError:
        with open(output_path.with_suffix(".txt"), "w") as f:
            f.write(response.content)
        print(f"\nRaw response saved to {output_path.with_suffix('.txt')}")


def analyze_streamer(streamer: str, max_videos: int = 5):
    """Analyze all videos for a streamer."""
    from core.training.vod_analyzer import VODAnalyzer

    print(f"\nAnalyzing videos for: {streamer}")
    print(f"Max videos: {max_videos}")
    print("-" * 60)

    with VODAnalyzer() as analyzer:
        results = analyzer.analyze_streamer_videos(streamer, max_videos=max_videos)

        output_path = Path(f"data/{streamer}_training_data.json")
        analyzer.aggregate_training_data(results, output_path)


def process_baalorlord_data():
    """Process Baalorlord's Spirelogs data."""
    from core.training.spirelogs_parser import SpirelogsParser, load_baalorlord_data

    print("\nProcessing Baalorlord Spirelogs data...")
    print("-" * 60)

    data_dir = Path("data/baalorlord/raw")
    if not data_dir.exists():
        print(f"Error: {data_dir} not found")
        return

    data = load_baalorlord_data(data_dir)

    print(f"\nData summary:")
    print(f"  Card rewards: {len(data['card_rewards'])}")
    print(f"  Rest decisions: {len(data['rest_decisions'])}")
    print(f"  Neow choices: {len(data['neow_choices'])}")

    # Compute card pick rates
    parser = SpirelogsParser(data_dir)
    runs = parser.parse_all_runs()
    stats = parser.compute_card_stats(runs)

    # Sort by pick rate
    sorted_cards = sorted(
        [(k, v) for k, v in stats.items() if v["offered"] >= 3],
        key=lambda x: x[1]["pick_rate"],
        reverse=True,
    )

    print(f"\nTop 20 most picked cards (min 3 offers):")
    for card, s in sorted_cards[:20]:
        print(f"  {card}: {s['pick_rate']:.1%} ({s['picked']}/{s['offered']})")

    print(f"\nBottom 10 (most skipped):")
    for card, s in sorted_cards[-10:]:
        print(f"  {card}: {s['pick_rate']:.1%} ({s['picked']}/{s['offered']})")

    # Save aggregated data
    output_path = Path("data/baalorlord_aggregated.json")
    with open(output_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"\nSaved aggregated data to {output_path}")

    # Save card stats
    stats_path = Path("data/baalorlord_card_stats.json")
    with open(stats_path, "w") as f:
        json.dump(stats, f, indent=2)
    print(f"Saved card stats to {stats_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Analyze Slay the Spire VODs for training data"
    )
    parser.add_argument("--video", "-v", help="YouTube video ID to analyze")
    parser.add_argument("--video-url", help="YouTube video URL (for Gemini analysis)")
    parser.add_argument("--streamer", "-s", help="Streamer name")
    parser.add_argument("--max-videos", "-m", type=int, default=5)
    parser.add_argument("--extract-frames", "-f", action="store_true")
    parser.add_argument("--gemini", action="store_true", help="Use native Gemini")
    parser.add_argument("--baalorlord", action="store_true", help="Process Baalorlord data")
    parser.add_argument("--test-connection", action="store_true")
    parser.add_argument("--test-chunk", action="store_true")
    parser.add_argument("--list", "-l", action="store_true")

    args = parser.parse_args()

    # Check for required API keys based on mode
    has_openrouter = bool(os.environ.get("OPENROUTER_API_KEY"))
    has_google = bool(os.environ.get("GOOGLE_API_KEY"))

    if args.gemini or args.video_url:
        if not has_google:
            print("Error: GOOGLE_API_KEY required for Gemini analysis")
            print("Get one at: https://aistudio.google.com/apikey")
            sys.exit(1)
    elif not args.baalorlord and not args.list:
        if not has_openrouter:
            print("Warning: OPENROUTER_API_KEY not set")
            print("Get one at: https://openrouter.ai/keys")

    if args.test_connection:
        if has_openrouter:
            test_llm_connection()
        if has_google:
            test_gemini_connection()
    elif args.test_chunk:
        test_chunk_extraction()
    elif args.list:
        list_available()
    elif args.baalorlord:
        process_baalorlord_data()
    elif args.video_url:
        analyze_video_with_gemini(args.video_url)
    elif args.video:
        if not args.streamer:
            print("Error: --streamer required with --video")
            sys.exit(1)
        analyze_video(args.video, args.streamer, args.extract_frames)
    elif args.streamer:
        analyze_streamer(args.streamer, args.max_videos)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
