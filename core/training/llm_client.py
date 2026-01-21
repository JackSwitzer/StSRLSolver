"""OpenRouter LLM client for Gemini and other models."""

import os
import json
import httpx
from typing import Optional, Any
from dataclasses import dataclass


@dataclass
class LLMResponse:
    """Response from LLM."""
    content: str
    model: str
    usage: dict[str, int]
    raw_response: dict


class OpenRouterClient:
    """Client for OpenRouter API supporting Gemini and other models."""

    BASE_URL = "https://openrouter.ai/api/v1"

    # Model identifiers
    GEMINI_PRO = "google/gemini-2.0-flash-001"  # Fast, cheap
    GEMINI_FLASH = "google/gemini-2.0-flash-thinking-exp-1219"  # Thinking model
    CLAUDE_SONNET = "anthropic/claude-3.5-sonnet"

    def __init__(self, api_key: Optional[str] = None):
        """Initialize client with API key from env or parameter."""
        self.api_key = (
            api_key or
            os.environ.get("OPENROUTER_API_KEY") or
            os.environ.get("OPEN_ROUTER_API_KEY")
        )
        if not self.api_key:
            raise ValueError(
                "OpenRouter API key required. Set OPENROUTER_API_KEY env var "
                "or pass api_key parameter."
            )

        self.client = httpx.Client(
            base_url=self.BASE_URL,
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "HTTP-Referer": "https://github.com/stsrl",
                "X-Title": "STS RL Training",
                "Content-Type": "application/json",
            },
            timeout=120.0,
        )

    def complete(
        self,
        prompt: str,
        model: str = GEMINI_PRO,
        system: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.3,
        json_mode: bool = False,
    ) -> LLMResponse:
        """Send completion request to OpenRouter.

        Args:
            prompt: User prompt
            model: Model identifier
            system: Optional system prompt
            max_tokens: Max response tokens
            temperature: Sampling temperature
            json_mode: Request JSON output

        Returns:
            LLMResponse with content and metadata
        """
        messages = []
        if system:
            messages.append({"role": "system", "content": system})
        messages.append({"role": "user", "content": prompt})

        payload = {
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
        }

        if json_mode:
            payload["response_format"] = {"type": "json_object"}

        response = self.client.post("/chat/completions", json=payload)
        response.raise_for_status()

        data = response.json()

        return LLMResponse(
            content=data["choices"][0]["message"]["content"],
            model=data.get("model", model),
            usage=data.get("usage", {}),
            raw_response=data,
        )

    def extract_json(self, response: LLMResponse) -> Any:
        """Extract JSON from response, handling markdown code blocks."""
        content = response.content.strip()

        # Handle markdown code blocks
        if content.startswith("```"):
            lines = content.split("\n")
            # Remove first and last lines (```json and ```)
            content = "\n".join(lines[1:-1])

        return json.loads(content)

    def close(self):
        """Close the HTTP client."""
        self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


# Convenience function
def get_client(api_key: Optional[str] = None) -> OpenRouterClient:
    """Get OpenRouter client instance."""
    return OpenRouterClient(api_key)
