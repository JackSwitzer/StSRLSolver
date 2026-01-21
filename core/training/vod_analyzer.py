"""Full VOD analysis pipeline combining transcripts, frames, and LLM analysis."""

import json
import subprocess
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, field

from .youtube_client import YouTubeClient, VideoTranscript
from .decision_extractor import DecisionExtractor, GameRun
from .llm_client import OpenRouterClient


@dataclass
class FrameCapture:
    """A captured frame from video."""
    timestamp_seconds: float
    path: Path
    description: str = ""


@dataclass
class AnalyzedVOD:
    """Complete analysis of a VOD."""
    video_id: str
    streamer: str
    transcript: Optional[VideoTranscript]
    game_run: Optional[GameRun]
    frames: list[FrameCapture] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "video_id": self.video_id,
            "streamer": self.streamer,
            "transcript": self.transcript.to_dict() if self.transcript else None,
            "game_run": self.game_run.to_dict() if self.game_run else None,
            "frames": [
                {"timestamp": f.timestamp_seconds, "path": str(f.path), "description": f.description}
                for f in self.frames
            ],
        }


class VODAnalyzer:
    """Analyzes Slay the Spire VODs for training data extraction."""

    # Key decision timestamps to capture (seconds between checks)
    FRAME_INTERVAL = 30  # Capture every 30 seconds for decision detection

    def __init__(
        self,
        output_dir: Path = Path("data/vod_analysis"),
        cache_dir: Path = Path("data/transcripts"),
        llm_model: str = OpenRouterClient.GEMINI_PRO,
    ):
        self.output_dir = output_dir
        self.output_dir.mkdir(parents=True, exist_ok=True)

        self.youtube = YouTubeClient(cache_dir)
        self.extractor = DecisionExtractor(model=llm_model)

    def analyze_video(
        self,
        video_id: str,
        streamer: str,
        extract_frames: bool = False,
        frame_timestamps: Optional[list[float]] = None,
    ) -> AnalyzedVOD:
        """Analyze a single VOD.

        Args:
            video_id: YouTube video ID
            streamer: Streamer name
            extract_frames: Whether to extract video frames
            frame_timestamps: Specific timestamps to capture (seconds)

        Returns:
            AnalyzedVOD with all extracted data
        """
        print(f"Analyzing VOD: {video_id} from {streamer}")

        # Get transcript
        transcript = self.youtube.get_transcript(video_id)
        if not transcript:
            print(f"  No transcript available for {video_id}")
            return AnalyzedVOD(video_id, streamer, None, None)

        print(f"  Transcript: {len(transcript.segments)} segments")

        # Extract decisions from transcript
        game_run = self.extractor.extract_from_transcript(transcript, streamer)

        # Extract frames if requested
        frames = []
        if extract_frames:
            frames = self._extract_frames(video_id, frame_timestamps or [])

        result = AnalyzedVOD(
            video_id=video_id,
            streamer=streamer,
            transcript=transcript,
            game_run=game_run,
            frames=frames,
        )

        # Save results
        self._save_analysis(result)

        return result

    def analyze_streamer_videos(
        self,
        streamer: str,
        video_ids: Optional[list[str]] = None,
        max_videos: int = 10,
    ) -> list[AnalyzedVOD]:
        """Analyze multiple videos from a streamer.

        Args:
            streamer: Streamer name
            video_ids: Specific video IDs (or use known list)
            max_videos: Maximum videos to process

        Returns:
            List of analyzed VODs
        """
        if video_ids is None:
            video_ids = self.youtube.list_available_videos(streamer)

        video_ids = video_ids[:max_videos]
        print(f"Analyzing {len(video_ids)} videos from {streamer}")

        results = []
        for vid in video_ids:
            try:
                result = self.analyze_video(vid, streamer)
                results.append(result)
            except Exception as e:
                print(f"Error analyzing {vid}: {e}")
                continue

        return results

    def _extract_frames(
        self,
        video_id: str,
        timestamps: list[float],
    ) -> list[FrameCapture]:
        """Extract frames at specific timestamps using yt-dlp + ffmpeg.

        Note: Requires yt-dlp and ffmpeg installed.
        """
        frames = []
        url = f"https://www.youtube.com/watch?v={video_id}"
        frame_dir = self.output_dir / "frames" / video_id
        frame_dir.mkdir(parents=True, exist_ok=True)

        for ts in timestamps:
            output_path = frame_dir / f"frame_{int(ts)}.jpg"

            try:
                # Use yt-dlp to get video URL, then ffmpeg to extract frame
                # This is a simplified version - in practice might need caching
                cmd = [
                    "yt-dlp",
                    "-f", "best[height<=720]",  # Medium quality for speed
                    "--get-url",
                    url,
                ]
                result = subprocess.run(cmd, capture_output=True, text=True)
                if result.returncode != 0:
                    print(f"    Failed to get video URL: {result.stderr}")
                    continue

                video_url = result.stdout.strip()

                # Extract frame with ffmpeg
                cmd = [
                    "ffmpeg",
                    "-ss", str(ts),
                    "-i", video_url,
                    "-frames:v", "1",
                    "-y",  # Overwrite
                    str(output_path),
                ]
                result = subprocess.run(cmd, capture_output=True)
                if result.returncode == 0 and output_path.exists():
                    frames.append(FrameCapture(ts, output_path))
                    print(f"    Captured frame at {ts}s")

            except Exception as e:
                print(f"    Error extracting frame at {ts}: {e}")
                continue

        return frames

    def _save_analysis(self, analysis: AnalyzedVOD):
        """Save analysis results to JSON."""
        output_path = self.output_dir / f"{analysis.video_id}_analysis.json"
        with open(output_path, "w") as f:
            json.dump(analysis.to_dict(), f, indent=2)
        print(f"  Saved to {output_path}")

    def aggregate_training_data(
        self,
        analyses: list[AnalyzedVOD],
        output_path: Optional[Path] = None,
    ) -> dict:
        """Aggregate decisions from multiple VODs into training format.

        Args:
            analyses: List of analyzed VODs
            output_path: Path to save aggregated data

        Returns:
            Aggregated training data dict
        """
        training_data = {
            "card_rewards": [],
            "path_choices": [],
            "rest_decisions": [],
            "shop_decisions": [],
            "combat_lines": [],
            "neow_choices": [],
            "event_choices": [],
        }

        for analysis in analyses:
            if not analysis.game_run:
                continue

            for decision in analysis.game_run.decisions:
                entry = {
                    "video_id": analysis.video_id,
                    "streamer": analysis.streamer,
                    "floor": decision.floor,
                    "options": decision.options,
                    "chosen": decision.chosen,
                    "reasoning": decision.reasoning,
                }

                if decision.type.value == "card_reward":
                    training_data["card_rewards"].append(entry)
                elif decision.type.value == "path":
                    training_data["path_choices"].append(entry)
                elif decision.type.value == "rest_site":
                    training_data["rest_decisions"].append(entry)
                elif decision.type.value == "shop":
                    training_data["shop_decisions"].append(entry)
                elif decision.type.value == "neow":
                    training_data["neow_choices"].append(entry)
                elif decision.type.value == "event":
                    training_data["event_choices"].append(entry)

            for combat in analysis.game_run.combat_lines:
                training_data["combat_lines"].append({
                    "video_id": analysis.video_id,
                    "streamer": analysis.streamer,
                    "floor": combat.floor,
                    "turn": combat.turn,
                    "actions": combat.actions,
                    "reasoning": combat.reasoning,
                })

        # Summary
        print("\nTraining data summary:")
        for key, items in training_data.items():
            print(f"  {key}: {len(items)} entries")

        if output_path:
            with open(output_path, "w") as f:
                json.dump(training_data, f, indent=2)
            print(f"\nSaved to {output_path}")

        return training_data

    def close(self):
        """Clean up resources."""
        self.extractor.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
