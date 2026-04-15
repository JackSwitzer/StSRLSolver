"""Append-only training run logging for manifests, events, metrics, and frontier reports."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Mapping

from .benchmarking import FrontierReport
from .contracts import BenchmarkReport
from .manifests import TrainingRunManifest


@dataclass(frozen=True)
class TrainingArtifacts:
    root: Path

    @property
    def manifest_path(self) -> Path:
        return self.root / "manifest.json"

    @property
    def events_path(self) -> Path:
        return self.root / "events.jsonl"

    @property
    def metrics_path(self) -> Path:
        return self.root / "metrics.jsonl"

    @property
    def frontier_report_path(self) -> Path:
        return self.root / "frontier_report.json"

    @property
    def benchmark_report_path(self) -> Path:
        return self.root / "benchmark_report.json"

    @property
    def frontier_markdown_path(self) -> Path:
        return self.root / "frontier_report.md"

    @property
    def frontier_groups_path(self) -> Path:
        return self.root / "frontier_groups.json"

    @property
    def episode_log_path(self) -> Path:
        return self.root / "episodes.jsonl"


class TrainingRunLogger:
    def __init__(self, artifacts: TrainingArtifacts):
        self.artifacts = artifacts
        self.artifacts.root.mkdir(parents=True, exist_ok=True)

    def write_manifest(self, manifest: TrainingRunManifest) -> None:
        self.artifacts.manifest_path.write_text(
            json.dumps(manifest.to_dict(), indent=2, sort_keys=True)
        )

    def append_event(self, event_type: str, **fields: Any) -> None:
        payload = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "event_type": event_type,
            **fields,
        }
        with self.artifacts.events_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(payload, sort_keys=True) + "\n")

    def append_metric(
        self,
        name: str,
        value: float,
        *,
        step: int,
        config: str,
        deck_family: str | None = None,
        remove_count: int | None = None,
        potion_set: tuple[str, ...] | None = None,
        enemy: str | None = None,
        corpus_slice: str | None = None,
        corpus_case: str | None = None,
        seed_source: str | None = None,
    ) -> None:
        payload = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "name": name,
            "value": value,
            "step": step,
            "config": config,
        }
        if deck_family is not None:
            payload["deck_family"] = deck_family
        if remove_count is not None:
            payload["remove_count"] = remove_count
        if potion_set is not None:
            payload["potion_set"] = list(potion_set)
        if enemy is not None:
            payload["enemy"] = enemy
        if corpus_slice is not None:
            payload["corpus_slice"] = corpus_slice
        if corpus_case is not None:
            payload["corpus_case"] = corpus_case
        if seed_source is not None:
            payload["seed_source"] = seed_source
        with self.artifacts.metrics_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(payload, sort_keys=True) + "\n")

    def append_episode(self, payload: Mapping[str, Any]) -> None:
        with self.artifacts.episode_log_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(dict(payload), sort_keys=True) + "\n")

    def write_frontier_report(self, report: FrontierReport) -> None:
        self.artifacts.frontier_report_path.write_text(report.to_json())
        self.artifacts.frontier_markdown_path.write_text(report.to_markdown())
        self.artifacts.frontier_groups_path.write_text(
            json.dumps(report.to_dict().get("groups", []), indent=2, sort_keys=True)
        )

    def write_benchmark_report(self, report: BenchmarkReport) -> None:
        self.artifacts.benchmark_report_path.write_text(
            json.dumps(asdict(report), indent=2, sort_keys=True)
        )
