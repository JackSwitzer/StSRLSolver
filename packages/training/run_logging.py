"""Append-only training run logging for manifests, events, metrics, and frontier reports."""

from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .benchmarking import FrontierReport
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
    def frontier_markdown_path(self) -> Path:
        return self.root / "frontier_report.md"


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

    def append_metric(self, name: str, value: float, *, step: int, config: str) -> None:
        payload = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "name": name,
            "value": value,
            "step": step,
            "config": config,
        }
        with self.artifacts.metrics_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(payload, sort_keys=True) + "\n")

    def write_frontier_report(self, report: FrontierReport) -> None:
        self.artifacts.frontier_report_path.write_text(report.to_json())
        self.artifacts.frontier_markdown_path.write_text(report.to_markdown())
