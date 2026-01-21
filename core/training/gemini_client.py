"""Native Google Gemini client for video analysis.

This client uses Google's native API for direct YouTube video analysis,
which is more capable than OpenRouter for multimodal content.

Resolution Strategy:
- LOW (66 tokens/frame): Full video scanning, initial pass
- DEFAULT (258 tokens/frame): Card rewards, shop screens, key decisions
- HIGH: Individual card/potion analysis during combat solving
"""

import os
import json
from pathlib import Path
from typing import Optional, Any, Union
from dataclasses import dataclass
from enum import Enum


class Resolution(str, Enum):
    """Media resolution levels for Gemini analysis."""
    LOW = "low"          # 66 tokens/frame - for scanning
    DEFAULT = "medium"   # 258 tokens/frame - for decisions
    # HIGH not directly supported, use cropped images instead


@dataclass
class GeminiResponse:
    """Response from Gemini API."""
    content: str
    model: str
    usage: dict
    raw_response: Any = None


class GeminiClient:
    """Native Google Gemini client for video and multimodal analysis.

    Requires: pip install google-generativeai
    Set GOOGLE_API_KEY environment variable.
    """

    # Model identifiers
    FLASH = "gemini-2.0-flash"
    FLASH_LITE = "gemini-2.0-flash-lite"
    PRO = "gemini-2.5-pro"

    def __init__(self, api_key: Optional[str] = None):
        """Initialize with API key from env or parameter."""
        self.api_key = api_key or os.environ.get("GOOGLE_API_KEY")
        if not self.api_key:
            raise ValueError(
                "Google API key required. Set GOOGLE_API_KEY env var "
                "or pass api_key parameter. Get one at: "
                "https://aistudio.google.com/apikey"
            )

        import google.generativeai as genai
        genai.configure(api_key=self.api_key)
        self.genai = genai
        self._model_cache = {}

    def _get_model(self, model_name: str):
        """Get or create a model instance."""
        if model_name not in self._model_cache:
            self._model_cache[model_name] = self.genai.GenerativeModel(model_name)
        return self._model_cache[model_name]

    def analyze_youtube_video(
        self,
        video_url: str,
        prompt: str,
        model: str = FLASH,
        low_resolution: bool = True,
    ) -> GeminiResponse:
        """Analyze a YouTube video directly by URL.

        Args:
            video_url: YouTube video URL (must be public)
            prompt: Analysis prompt
            model: Model to use
            low_resolution: Use low media resolution (66 vs 258 tokens/frame)

        Returns:
            GeminiResponse with analysis
        """
        gen_model = self._get_model(model)

        # Build config
        generation_config = {}
        if low_resolution:
            generation_config["media_resolution"] = "low"

        response = gen_model.generate_content(
            [video_url, prompt],
            generation_config=generation_config if generation_config else None,
        )

        return GeminiResponse(
            content=response.text,
            model=model,
            usage={
                "prompt_tokens": getattr(response.usage_metadata, "prompt_token_count", 0),
                "completion_tokens": getattr(response.usage_metadata, "candidates_token_count", 0),
            },
            raw_response=response,
        )

    def analyze_video_file(
        self,
        video_path: Union[str, Path],
        prompt: str,
        model: str = FLASH,
        low_resolution: bool = True,
    ) -> GeminiResponse:
        """Analyze a local video file.

        Args:
            video_path: Path to video file
            prompt: Analysis prompt
            model: Model to use
            low_resolution: Use low media resolution

        Returns:
            GeminiResponse with analysis
        """
        # Upload file
        video_file = self.genai.upload_file(str(video_path))

        # Wait for processing
        import time
        while video_file.state.name == "PROCESSING":
            time.sleep(2)
            video_file = self.genai.get_file(video_file.name)

        if video_file.state.name == "FAILED":
            raise RuntimeError(f"Video processing failed: {video_file.state.name}")

        gen_model = self._get_model(model)

        generation_config = {}
        if low_resolution:
            generation_config["media_resolution"] = "low"

        response = gen_model.generate_content(
            [video_file, prompt],
            generation_config=generation_config if generation_config else None,
        )

        return GeminiResponse(
            content=response.text,
            model=model,
            usage={
                "prompt_tokens": getattr(response.usage_metadata, "prompt_token_count", 0),
                "completion_tokens": getattr(response.usage_metadata, "candidates_token_count", 0),
            },
            raw_response=response,
        )

    def analyze_image(
        self,
        image_path: Union[str, Path],
        prompt: str,
        model: str = FLASH,
    ) -> GeminiResponse:
        """Analyze an image file.

        Args:
            image_path: Path to image
            prompt: Analysis prompt
            model: Model to use

        Returns:
            GeminiResponse with analysis
        """
        import PIL.Image

        img = PIL.Image.open(image_path)
        gen_model = self._get_model(model)

        response = gen_model.generate_content([img, prompt])

        return GeminiResponse(
            content=response.text,
            model=model,
            usage={
                "prompt_tokens": getattr(response.usage_metadata, "prompt_token_count", 0),
                "completion_tokens": getattr(response.usage_metadata, "candidates_token_count", 0),
            },
            raw_response=response,
        )

    def complete(
        self,
        prompt: str,
        model: str = FLASH,
        system: Optional[str] = None,
        json_mode: bool = False,
    ) -> GeminiResponse:
        """Text completion.

        Args:
            prompt: User prompt
            model: Model to use
            system: System instruction
            json_mode: Request JSON output

        Returns:
            GeminiResponse
        """
        gen_model = self._get_model(model)

        generation_config = {}
        if json_mode:
            generation_config["response_mime_type"] = "application/json"

        if system:
            gen_model = self.genai.GenerativeModel(
                model,
                system_instruction=system,
            )

        response = gen_model.generate_content(
            prompt,
            generation_config=generation_config if generation_config else None,
        )

        return GeminiResponse(
            content=response.text,
            model=model,
            usage={
                "prompt_tokens": getattr(response.usage_metadata, "prompt_token_count", 0),
                "completion_tokens": getattr(response.usage_metadata, "candidates_token_count", 0),
            },
            raw_response=response,
        )

    def extract_json(self, response: GeminiResponse) -> Any:
        """Extract JSON from response."""
        content = response.content.strip()
        if content.startswith("```"):
            lines = content.split("\n")
            content = "\n".join(lines[1:-1])
        return json.loads(content)


# Decision extraction prompts for STS VOD analysis
STS_VIDEO_ANALYSIS_PROMPT = """Analyze this Slay the Spire gameplay video and extract ALL game decisions.

Focus on Watcher character decisions:
1. Neow bonus selection (start of run)
2. Card reward choices (after combats)
3. Path/map choices
4. Rest site decisions (rest vs upgrade vs dig)
5. Shop purchases and removes
6. Event choices
7. Boss relic selection

For each decision, note:
- Timestamp (MM:SS format)
- Decision type
- Options available (if visible/mentioned)
- Choice made
- Any reasoning given by the player

Also extract combat sequences:
- Notable card plays
- Stance changes (Calm, Wrath, Divinity)
- Turn sequences for difficult fights

Output as JSON:
{
  "decisions": [
    {
      "timestamp": "12:34",
      "type": "card_reward",
      "floor": 5,
      "options": ["Tantrum", "Eruption", "Crescendo"],
      "chosen": "Tantrum",
      "reasoning": "player mentioned wanting more wrath entry"
    }
  ],
  "combat_highlights": [
    {
      "timestamp": "15:20",
      "floor": 8,
      "enemy": "Gremlin Nob",
      "notable_plays": ["Wrath -> Eruption -> Calm exit"],
      "outcome": "kill with 45 HP remaining"
    }
  ],
  "run_outcome": "win/loss",
  "final_floor": 57
}"""


STS_FRAME_ANALYSIS_PROMPT = """Analyze this Slay the Spire screenshot and extract the game state.

Identify:
1. Screen type (combat, card_reward, shop, map, rest_site, event, boss_relic)
2. If card_reward: List all cards shown with names
3. If shop: List items for sale with prices
4. If combat: Player HP, enemy HP, cards in hand, energy
5. If map: Current floor, available paths
6. Any relics visible
7. Gold amount
8. Potion slots

Output as JSON:
{
  "screen_type": "card_reward",
  "floor": 5,
  "data": {
    "cards": [
      {"name": "Tantrum", "rarity": "common", "upgraded": false},
      {"name": "Eruption", "rarity": "starter", "upgraded": true}
    ]
  },
  "player_state": {
    "hp": 65,
    "max_hp": 72,
    "gold": 150,
    "relics": ["Burning Blood", "Vajra"]
  }
}"""
