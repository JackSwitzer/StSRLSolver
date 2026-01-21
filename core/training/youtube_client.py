"""YouTube video search and transcript fetching."""

import re
import json
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, field
from youtube_transcript_api import YouTubeTranscriptApi


@dataclass
class VideoInfo:
    """Information about a YouTube video."""
    video_id: str
    title: str = ""
    channel: str = ""
    duration_seconds: int = 0
    url: str = field(init=False)

    def __post_init__(self):
        self.url = f"https://www.youtube.com/watch?v={self.video_id}"


@dataclass
class TranscriptSegment:
    """A segment of video transcript."""
    text: str
    start: float
    duration: float

    @property
    def end(self) -> float:
        return self.start + self.duration


@dataclass
class VideoTranscript:
    """Full video transcript with segments."""
    video_id: str
    segments: list[TranscriptSegment]
    language: str = "en"

    @property
    def full_text(self) -> str:
        """Get full transcript as single string."""
        return " ".join(s.text for s in self.segments)

    def get_text_at_time(self, time_seconds: float, window: float = 30.0) -> str:
        """Get transcript text around a specific timestamp."""
        relevant = [
            s for s in self.segments
            if s.start >= time_seconds - window and s.start <= time_seconds + window
        ]
        return " ".join(s.text for s in relevant)

    def to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        return {
            "video_id": self.video_id,
            "language": self.language,
            "full_transcript": [
                {"text": s.text, "start": s.start, "duration": s.duration}
                for s in self.segments
            ],
        }

    @classmethod
    def from_dict(cls, data: dict) -> "VideoTranscript":
        """Create from dictionary."""
        segments = [
            TranscriptSegment(
                text=s["text"],
                start=s["start"],
                duration=s["duration"],
            )
            for s in data["full_transcript"]
        ]
        return cls(
            video_id=data["video_id"],
            segments=segments,
            language=data.get("language", "en"),
        )


class YouTubeClient:
    """Client for fetching YouTube video transcripts."""

    # Known Watcher streamers and their channel info
    STREAMERS = {
        "merl61": {
            "channel_id": "UC0yX5O-Vy0e-f7qQgJ5F8Gg",
            "search_terms": ["watcher a20 streak", "slay the spire watcher"],
        },
        "lifecoach": {
            "channel_id": "UCfc5RsHXuFp_DLgJLdSjCsQ",
            "search_terms": ["watcher a20", "slay the spire watcher streak"],
        },
        "baalorlord": {
            "channel_id": "UCvb0wUcKKSJKvGhwPR7KxKg",
            "search_terms": ["watcher a20", "slay the spire watcher"],
        },
    }

    def __init__(self, cache_dir: Optional[Path] = None):
        """Initialize YouTube client.

        Args:
            cache_dir: Directory to cache transcripts
        """
        self.cache_dir = cache_dir or Path("data/transcripts")
        self.cache_dir.mkdir(parents=True, exist_ok=True)

    def get_transcript(
        self,
        video_id: str,
        languages: list[str] = ["en"],
        use_cache: bool = True,
    ) -> Optional[VideoTranscript]:
        """Fetch transcript for a video.

        Args:
            video_id: YouTube video ID
            languages: Preferred languages in order
            use_cache: Whether to use cached transcripts

        Returns:
            VideoTranscript or None if unavailable
        """
        # Check cache first
        cache_path = self.cache_dir / f"{video_id}_transcript.json"
        if use_cache and cache_path.exists():
            with open(cache_path) as f:
                data = json.load(f)
                return VideoTranscript.from_dict(data)

        try:
            # Fetch from YouTube (new API v1.0+)
            ytt = YouTubeTranscriptApi()
            transcript_list = ytt.fetch(video_id, languages=languages)

            segments = [
                TranscriptSegment(
                    text=item.text,
                    start=item.start,
                    duration=item.duration,
                )
                for item in transcript_list
            ]

            transcript = VideoTranscript(
                video_id=video_id,
                segments=segments,
                language=languages[0] if languages else "en",
            )

            # Cache the result
            with open(cache_path, "w") as f:
                json.dump(transcript.to_dict(), f, indent=2)

            return transcript

        except Exception as e:
            print(f"Failed to fetch transcript for {video_id}: {e}")
            return None

    def extract_video_id(self, url: str) -> Optional[str]:
        """Extract video ID from YouTube URL."""
        patterns = [
            r"(?:v=|/v/|youtu\.be/)([a-zA-Z0-9_-]{11})",
            r"^([a-zA-Z0-9_-]{11})$",  # Just the ID
        ]
        for pattern in patterns:
            match = re.search(pattern, url)
            if match:
                return match.group(1)
        return None

    def list_available_videos(self, streamer: str) -> list[str]:
        """List known video IDs for a streamer.

        Note: This returns manually curated video IDs.
        For full channel scraping, use yt-dlp.
        """
        # Curated list of high-quality Watcher runs
        known_videos = {
            "merl61": [
                "NT4GitAXwxs",  # Watcher streak game
                "q3l_tN-0oac",  # Long watcher run (4.8hr)
                "TO6u6As_lR4",  # Another watcher game
            ],
            "jorbs": [
                "o0zyhZLfdmw",  # Watcher A20 recommended
                "_1-Qgs1R7E0",  # Watcher A20 recommended
            ],
            "lifecoach": [
                # Primarily Twitch - check twitch.tv/lifecoach1981
            ],
            "baalorlord": [
                # Check baalorlord.tv/youtube for current uploads
            ],
        }
        return known_videos.get(streamer.lower(), [])

    def batch_fetch_transcripts(
        self,
        video_ids: list[str],
        streamer: str = "unknown",
    ) -> dict[str, VideoTranscript]:
        """Fetch transcripts for multiple videos.

        Args:
            video_ids: List of video IDs
            streamer: Streamer name for metadata

        Returns:
            Dict mapping video_id to transcript
        """
        results = {}
        for vid in video_ids:
            print(f"Fetching transcript for {vid}...")
            transcript = self.get_transcript(vid)
            if transcript:
                results[vid] = transcript
        return results
