"""Append-only JSONL episode logging for the rebuilt training stack."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Mapping

from .contracts import EpisodeLog


@dataclass(frozen=True)
class EpisodeProvenance:
    corpus_slice: str | None = None
    corpus_case: str | None = None
    deck_family: str | None = None
    remove_count: int | None = None
    potion_set: tuple[str, ...] = ()
    enemy: str | None = None
    seed_source: str | None = None
    neow_source: str | None = None


@dataclass(frozen=True)
class LoggedEpisode:
    episode: EpisodeLog
    provenance: EpisodeProvenance | None = None
    notes: tuple[str, ...] = ()


class EpisodeLogger:
    def __init__(self, path: Path):
        self.path = path
        self.path.parent.mkdir(parents=True, exist_ok=True)

    def append(self, episode: EpisodeLog | LoggedEpisode) -> None:
        payload = (
            asdict(episode)
            if isinstance(episode, LoggedEpisode)
            else asdict(episode)
        )
        with self.path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(payload, sort_keys=True) + "\n")

    def append_payload(self, payload: Mapping[str, Any]) -> None:
        with self.path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(dict(payload), sort_keys=True) + "\n")
