"""Stable manifest helpers for rebuilt training runs."""

from __future__ import annotations

import hashlib
import json
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Mapping

from .contracts import RestrictionPolicy, RunManifest


def _stable_json_hash(values: Mapping[str, Any]) -> str:
    payload = json.dumps(values, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:16]


@dataclass(frozen=True)
class GitSnapshot:
    commit_sha: str = "unknown"
    branch: str = "unknown"
    dirty: bool = False


@dataclass(frozen=True)
class TrainingConfigSnapshot:
    values: Mapping[str, Any]
    config_hash: str

    @classmethod
    def from_values(cls, values: Mapping[str, Any]) -> "TrainingConfigSnapshot":
        canonical = json.loads(json.dumps(values, sort_keys=True))
        return cls(values=canonical, config_hash=_stable_json_hash(canonical))


@dataclass(frozen=True)
class SearchBudgetSnapshot:
    frontier_width: int | None = None
    node_budget: int | None = None
    rollout_budget: int | None = None
    time_limit_ms: int | None = None


@dataclass(frozen=True)
class OvernightSearchSnapshot:
    sweep_config: str | None = None
    search_policy: str | None = None
    planned_games: int | None = None
    worker_count: int | None = None
    corpus_name: str | None = None
    corpus_slices: tuple[str, ...] = ()
    benchmark_groups: tuple[str, ...] = ()
    easy_seed_bucket: str | None = None
    easy_seed_target_count: int = 0
    neow_policy: str | None = None
    budget: SearchBudgetSnapshot = field(default_factory=SearchBudgetSnapshot)


@dataclass(frozen=True)
class TrainingRunManifest:
    run_id: str
    created_at: str
    git: GitSnapshot
    config: TrainingConfigSnapshot
    engine_git: GitSnapshot | None = None
    host: str | None = None
    worktree: str | None = None
    sweep_config: str | None = None
    overnight_search: OvernightSearchSnapshot | None = None
    tags: tuple[str, ...] = ()
    notes: tuple[str, ...] = ()

    @classmethod
    def create(
        cls,
        *,
        run_id: str,
        git: GitSnapshot | None = None,
        config: TrainingConfigSnapshot | None = None,
        engine_git: GitSnapshot | None = None,
        host: str | None = None,
        worktree: str | None = None,
        sweep_config: str | None = None,
        overnight_search: OvernightSearchSnapshot | None = None,
        tags: list[str] | tuple[str, ...] = (),
        notes: list[str] | tuple[str, ...] = (),
    ) -> "TrainingRunManifest":
        return cls(
            run_id=run_id,
            created_at=datetime.now(timezone.utc).isoformat(),
            git=git or GitSnapshot(),
            config=config or TrainingConfigSnapshot.from_values({}),
            engine_git=engine_git,
            host=host,
            worktree=worktree,
            sweep_config=sweep_config,
            overnight_search=overnight_search,
            tags=tuple(tags),
            notes=tuple(notes),
        )

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "TrainingRunManifest":
        overnight_payload = payload.get("overnight_search")
        return cls(
            run_id=payload["run_id"],
            created_at=payload["created_at"],
            git=GitSnapshot(**payload["git"]),
            config=TrainingConfigSnapshot(
                values=payload["config"]["values"],
                config_hash=payload["config"]["config_hash"],
            ),
            engine_git=GitSnapshot(**payload["engine_git"]) if payload.get("engine_git") else None,
            host=payload.get("host"),
            worktree=payload.get("worktree"),
            sweep_config=payload.get("sweep_config"),
            overnight_search=(
                OvernightSearchSnapshot(
                    sweep_config=overnight_payload.get("sweep_config"),
                    search_policy=overnight_payload.get("search_policy"),
                    planned_games=overnight_payload.get("planned_games"),
                    worker_count=overnight_payload.get("worker_count"),
                    corpus_name=overnight_payload.get("corpus_name"),
                    corpus_slices=tuple(overnight_payload.get("corpus_slices", ())),
                    benchmark_groups=tuple(overnight_payload.get("benchmark_groups", ())),
                    easy_seed_bucket=overnight_payload.get("easy_seed_bucket"),
                    easy_seed_target_count=overnight_payload.get("easy_seed_target_count", 0),
                    neow_policy=overnight_payload.get("neow_policy"),
                    budget=SearchBudgetSnapshot(**overnight_payload.get("budget", {})),
                )
                if overnight_payload
                else None
            ),
            tags=tuple(payload.get("tags", ())),
            notes=tuple(payload.get("notes", ())),
        )

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)

    def write_json(self, destination: Path) -> Path:
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(json.dumps(self.to_dict(), indent=2, sort_keys=True))
        return destination


def build_run_manifest(
    *,
    model_version: str,
    benchmark_config: str,
    seed: int,
    restriction_policy: RestrictionPolicy,
    combat_observation_schema_version: int,
    action_candidate_schema_version: int,
    gameplay_export_schema_version: int,
    replay_event_trace_schema_version: int,
) -> RunManifest:
    return RunManifest(
        git_sha="unknown",
        git_dirty=False,
        combat_observation_schema_version=combat_observation_schema_version,
        action_candidate_schema_version=action_candidate_schema_version,
        gameplay_export_schema_version=gameplay_export_schema_version,
        replay_event_trace_schema_version=replay_event_trace_schema_version,
        model_version=model_version,
        benchmark_config=benchmark_config,
        seed=seed,
        restriction_policy=restriction_policy,
        hardware="unknown",
        runtime="training-runtime",
    )
