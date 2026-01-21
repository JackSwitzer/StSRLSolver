"""Extract frames from videos at specific timestamps using ffmpeg/yt-dlp."""

import subprocess
import json
from pathlib import Path
from typing import Optional, Union
from dataclasses import dataclass


@dataclass
class ExtractedFrame:
    """Information about an extracted frame."""
    path: Path
    timestamp: float
    video_id: str


class FrameExtractor:
    """Extract frames from YouTube videos or local files."""

    def __init__(self, output_dir: Path = Path("data/frames")):
        """Initialize frame extractor.

        Args:
            output_dir: Directory to save extracted frames
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        # Check for required tools
        self._check_ffmpeg()

    def _check_ffmpeg(self):
        """Verify ffmpeg is available."""
        try:
            subprocess.run(["ffmpeg", "-version"], capture_output=True, check=True)
        except FileNotFoundError:
            raise RuntimeError(
                "ffmpeg not found. Install with: brew install ffmpeg (macOS) "
                "or apt install ffmpeg (Linux)"
            )

    def extract_frame_from_video(
        self,
        video_path: Union[str, Path],
        timestamp: float,
        output_name: Optional[str] = None,
    ) -> ExtractedFrame:
        """Extract a single frame from a local video file.

        Args:
            video_path: Path to video file
            timestamp: Time in seconds
            output_name: Optional output filename (without extension)

        Returns:
            ExtractedFrame with path to saved frame
        """
        video_path = Path(video_path)
        video_id = video_path.stem

        if output_name is None:
            output_name = f"{video_id}_frame_{int(timestamp)}"

        output_path = self.output_dir / f"{output_name}.png"

        # Format timestamp
        hours = int(timestamp // 3600)
        minutes = int((timestamp % 3600) // 60)
        seconds = timestamp % 60
        time_str = f"{hours:02d}:{minutes:02d}:{seconds:06.3f}"

        cmd = [
            "ffmpeg", "-y",
            "-ss", time_str,
            "-i", str(video_path),
            "-frames:v", "1",
            "-q:v", "2",
            str(output_path),
        ]

        result = subprocess.run(cmd, capture_output=True)
        if result.returncode != 0:
            raise RuntimeError(f"ffmpeg failed: {result.stderr.decode()}")

        return ExtractedFrame(
            path=output_path,
            timestamp=timestamp,
            video_id=video_id,
        )

    def extract_frames_at_interval(
        self,
        video_path: Union[str, Path],
        interval: int = 30,
        max_frames: Optional[int] = None,
    ) -> list[ExtractedFrame]:
        """Extract frames at regular intervals.

        Args:
            video_path: Path to video file
            interval: Seconds between frames
            max_frames: Maximum number of frames to extract

        Returns:
            List of ExtractedFrame objects
        """
        video_path = Path(video_path)
        video_id = video_path.stem

        # Get video duration
        duration = self._get_video_duration(video_path)
        if duration is None:
            raise RuntimeError(f"Could not determine duration for {video_path}")

        # Calculate timestamps
        timestamps = list(range(0, int(duration), interval))
        if max_frames:
            timestamps = timestamps[:max_frames]

        frames = []
        for ts in timestamps:
            try:
                frame = self.extract_frame_from_video(
                    video_path,
                    float(ts),
                    output_name=f"{video_id}_frame_{ts:06d}",
                )
                frames.append(frame)
                print(f"  Extracted frame at {ts}s")
            except Exception as e:
                print(f"  Failed to extract frame at {ts}s: {e}")

        return frames

    def extract_frames_at_timestamps(
        self,
        video_path: Union[str, Path],
        timestamps: list[float],
    ) -> list[ExtractedFrame]:
        """Extract frames at specific timestamps.

        Args:
            video_path: Path to video file
            timestamps: List of timestamps in seconds

        Returns:
            List of ExtractedFrame objects
        """
        video_path = Path(video_path)
        video_id = video_path.stem

        frames = []
        for ts in timestamps:
            try:
                frame = self.extract_frame_from_video(
                    video_path,
                    ts,
                    output_name=f"{video_id}_frame_{int(ts):06d}",
                )
                frames.append(frame)
            except Exception as e:
                print(f"  Failed at {ts}s: {e}")

        return frames

    def download_and_extract_frames(
        self,
        video_url: str,
        timestamps: list[float],
        video_id: Optional[str] = None,
    ) -> list[ExtractedFrame]:
        """Download YouTube video segment and extract frames.

        This downloads only the necessary segments, not the full video.

        Args:
            video_url: YouTube URL
            timestamps: List of timestamps to capture
            video_id: Optional video ID for naming

        Returns:
            List of ExtractedFrame objects
        """
        if video_id is None:
            # Extract from URL
            import re
            match = re.search(r"(?:v=|/)([a-zA-Z0-9_-]{11})", video_url)
            video_id = match.group(1) if match else "unknown"

        frames = []
        frame_dir = self.output_dir / video_id
        frame_dir.mkdir(exist_ok=True)

        for ts in timestamps:
            output_path = frame_dir / f"frame_{int(ts):06d}.png"

            try:
                # Download just a few seconds around the timestamp
                start = max(0, ts - 1)
                end = ts + 1

                # Download segment
                segment_path = frame_dir / f"segment_{int(ts)}.mp4"
                dl_cmd = [
                    "yt-dlp",
                    "-f", "bestvideo[height<=720]",
                    "--download-sections", f"*{start}-{end}",
                    "-o", str(segment_path),
                    video_url,
                ]
                subprocess.run(dl_cmd, capture_output=True, check=True)

                # Extract frame at 1 second (middle of segment)
                ff_cmd = [
                    "ffmpeg", "-y",
                    "-ss", "1",
                    "-i", str(segment_path),
                    "-frames:v", "1",
                    "-q:v", "2",
                    str(output_path),
                ]
                subprocess.run(ff_cmd, capture_output=True, check=True)

                # Clean up segment
                segment_path.unlink(missing_ok=True)

                frames.append(ExtractedFrame(
                    path=output_path,
                    timestamp=ts,
                    video_id=video_id,
                ))
                print(f"  Extracted frame at {ts}s")

            except Exception as e:
                print(f"  Failed at {ts}s: {e}")

        return frames

    def _get_video_duration(self, video_path: Path) -> Optional[float]:
        """Get video duration using ffprobe."""
        cmd = [
            "ffprobe",
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "json",
            str(video_path),
        ]

        try:
            result = subprocess.run(cmd, capture_output=True, check=True)
            data = json.loads(result.stdout)
            return float(data["format"]["duration"])
        except Exception:
            return None


def download_youtube_video(
    url: str,
    output_dir: Path = Path("data/videos"),
    max_height: int = 720,
) -> Path:
    """Download a YouTube video.

    Args:
        url: YouTube URL
        output_dir: Directory to save video
        max_height: Maximum video height

    Returns:
        Path to downloaded video
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Extract video ID
    import re
    match = re.search(r"(?:v=|/)([a-zA-Z0-9_-]{11})", url)
    video_id = match.group(1) if match else "video"

    output_path = output_dir / f"{video_id}.mp4"

    cmd = [
        "yt-dlp",
        "-f", f"bestvideo[height<={max_height}]+bestaudio/best[height<={max_height}]",
        "-o", str(output_path),
        "--merge-output-format", "mp4",
        url,
    ]

    print(f"Downloading {url}...")
    subprocess.run(cmd, check=True)
    print(f"Saved to {output_path}")

    return output_path
