"""CLI for the phase-1 combat search/training bring-up."""

from __future__ import annotations

import argparse
import json
import subprocess
from collections import defaultdict
from dataclasses import asdict
from math import exp
from pathlib import Path
from statistics import mean
from typing import Any, Iterable

from .config import TrainingStackConfig
from .benchmarking import BenchmarkFrontierPoint, build_frontier_report
from .contracts import (
    BenchmarkReport,
    BenchmarkSliceResult,
    CombatFrontierLine,
    CombatFrontierSummary,
    CombatOutcomeVector,
    EpisodeLog,
    EpisodeStep,
    RestrictionPolicy,
)
from .corpus import PreparedCorpusRequest, build_phase1_requests, default_watcher_a0_act1_corpus_plan
from .episode_log import EpisodeProvenance, LoggedEpisode
from .inference_service import (
    CombatInferenceService,
    CombatPreferenceExample,
    CombatSearchConfig,
    OvernightReanalysisLoop,
    TrainingConfig,
)
from .manifests import (
    GitSnapshot,
    OvernightSearchSnapshot,
    SearchBudgetSnapshot,
    TrainingConfigSnapshot,
    TrainingRunManifest,
    build_run_manifest,
)
from .selector import select_frontier
from .run_logging import TrainingArtifacts, TrainingRunLogger
from .shared_memory import CombatSearchRequest
from .combat_model import LinearCombatModel, MLXCombatModel


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Combat-first training rebuild tools")
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("print-default-config", help="Print the phase-1 bring-up config")
    subparsers.add_parser("print-stack-config", help="Print the wider training stack scaffold config")
    subparsers.add_parser("print-corpus-plan", help="Print the phase-1 Watcher A0 corpus plan")

    select_frontier_parser = subparsers.add_parser("select-frontier", help="Select deterministically from a search frontier")
    select_frontier_parser.add_argument("--input", required=True, help="JSON file containing frontier lines")
    select_frontier_parser.add_argument("--output", help="Optional JSON destination")

    reanalyze_parser = subparsers.add_parser("reanalyze-jsonl", help="Run batched reanalysis and lightweight weight updates")
    reanalyze_parser.add_argument("--input", required=True, help="JSONL dataset of reanalysis examples")
    reanalyze_parser.add_argument("--output-dir", required=True, help="Directory for checkpoint and epoch outputs")
    reanalyze_parser.add_argument("--epochs", type=int, default=3)
    reanalyze_parser.add_argument("--learning-rate", type=float, default=0.05)
    reanalyze_parser.add_argument("--top-k", type=int, default=4)
    reanalyze_parser.add_argument("--backend", choices=("linear", "mlx"), default="linear")
    reanalyze_parser.add_argument("--checkpoint", help="Optional checkpoint to load before reanalysis")
    reanalyze_parser.add_argument("--no-update", action="store_true", help="Disable weight updates and run pure scoring")

    smoke_parser = subparsers.add_parser("smoke-overnight", help="Generate a deterministic dataset and run the reanalysis loop")
    smoke_parser.add_argument("--output-dir", required=True, help="Directory for checkpoint and epoch outputs")
    smoke_parser.add_argument("--requests", type=int, default=16)
    smoke_parser.add_argument("--epochs", type=int, default=4)
    smoke_parser.add_argument("--learning-rate", type=float, default=0.05)
    smoke_parser.add_argument("--top-k", type=int, default=4)
    smoke_parser.add_argument("--backend", choices=("linear", "mlx"), default="linear")

    overnight_parser = subparsers.add_parser(
        "run-phase1-overnight",
        help="Run the phase-1 corpus-driven reanalysis loop and emit monitor artifacts",
    )
    overnight_parser.add_argument("--output-dir", required=True, help="Directory for artifacts and checkpoints")
    overnight_parser.add_argument("--epochs", type=int, default=4)
    overnight_parser.add_argument("--learning-rate", type=float, default=0.05)
    overnight_parser.add_argument("--top-k", type=int, default=4)
    overnight_parser.add_argument("--backend", choices=("linear", "mlx"), default="linear")
    overnight_parser.add_argument("--target-requests", type=int, default=5000)
    overnight_parser.add_argument("--seed", type=int, default=42)

    return parser


def _frontier_line_from_dict(payload: dict[str, Any]) -> CombatFrontierLine:
    return CombatFrontierLine(
        line_index=int(payload["line_index"]),
        action_prefix=tuple(int(value) for value in payload.get("action_prefix", ())),
        visits=int(payload.get("visits", 0)),
        expanded_nodes=int(payload.get("expanded_nodes", 0)),
        elapsed_ms=int(payload.get("elapsed_ms", 0)),
        outcome=CombatOutcomeVector(
            solve_probability=float(payload["outcome"]["solve_probability"]),
            expected_hp_loss=float(payload["outcome"]["expected_hp_loss"]),
            expected_turns=float(payload["outcome"]["expected_turns"]),
            potion_cost=float(payload["outcome"]["potion_cost"]),
            setup_value_delta=float(payload["outcome"]["setup_value_delta"]),
            persistent_scaling_delta=float(payload["outcome"]["persistent_scaling_delta"]),
        ),
    )


def _load_frontier_lines(path: Path) -> tuple[CombatFrontierLine, ...]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    raw_lines = payload["lines"] if isinstance(payload, dict) and "lines" in payload else payload
    return tuple(_frontier_line_from_dict(line) for line in raw_lines)


def _iter_jsonl(path: Path) -> Iterable[dict[str, Any]]:
    with path.open(encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if not line:
                continue
            yield json.loads(line)


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _write_jsonl(path: Path, rows: Iterable[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, sort_keys=True))
            handle.write("\n")


def _build_model(backend: str, checkpoint: Path | None):
    if backend == "mlx":
        return MLXCombatModel(checkpoint_path=str(checkpoint) if checkpoint else None)
    if checkpoint and checkpoint.exists():
        return LinearCombatModel.load_checkpoint(checkpoint)
    return LinearCombatModel()


def _load_examples(path: Path) -> list[CombatPreferenceExample]:
    return [CombatPreferenceExample.from_dict(payload) for payload in _iter_jsonl(path)]


def _synthetic_examples(count: int) -> list[CombatPreferenceExample]:
    examples: list[CombatPreferenceExample] = []
    for idx in range(count):
        preferred_strength = 1.0 + (idx % 3) * 0.25
        aggressive_strength = 2.0 + (idx % 4) * 0.5
        request = CombatSearchRequest.from_dict(
            {
                "request_id": f"smoke-{idx}",
                "state": {
                    "combat_id": f"combat-{idx}",
                    "turn": 1 + (idx % 4),
                    "hp": 60 - idx,
                    "block": idx % 6,
                    "energy": 3,
                    "hand_size": 5,
                    "draw_pile_size": 14,
                    "discard_pile_size": 3,
                    "exhaust_pile_size": idx % 2,
                    "stance": "Neutral" if idx % 2 == 0 else "Calm",
                },
                "candidates": [
                    {
                        "action_id": f"attack-{idx}",
                        "action_type": "card",
                        "features": [aggressive_strength, 0.0],
                        "legal": True,
                    },
                    {
                        "action_id": f"scale-{idx}",
                        "action_type": "card",
                        "features": [0.0, preferred_strength],
                        "legal": True,
                    },
                    {
                        "action_id": f"end-{idx}",
                        "action_type": "end_turn",
                        "features": [0.1, 0.1],
                        "legal": True,
                    },
                ],
                "metadata": {"source": "smoke"},
            }
        )
        examples.append(
            CombatPreferenceExample(
                request=request,
                preferred_action_id=f"scale-{idx}",
                metadata={"split": "smoke"},
            )
        )
    return examples


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _git_output(args: list[str], *, cwd: Path) -> str | None:
    try:
        return subprocess.check_output(args, cwd=cwd, text=True).strip()
    except Exception:
        return None


def _capture_git_snapshot(branch_hint: str | None = None) -> GitSnapshot:
    root = _repo_root()
    commit_sha = _git_output(["git", "rev-parse", "HEAD"], cwd=root) or "unknown"
    branch = branch_hint or _git_output(["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=root) or "unknown"
    dirty = bool(_git_output(["git", "status", "--short"], cwd=root))
    return GitSnapshot(commit_sha=commit_sha, branch=branch, dirty=dirty)


def _capture_engine_git_snapshot() -> GitSnapshot | None:
    root = _repo_root()
    commit_sha = _git_output(["git", "rev-parse", "codex/universal-gameplay-runtime"], cwd=root)
    if commit_sha is None:
        return None
    return GitSnapshot(
        commit_sha=commit_sha,
        branch="codex/universal-gameplay-runtime",
        dirty=False,
    )


def _phase1_runtime_manifest(*, backend: str, seed: int) -> Any:
    return build_run_manifest(
        model_version=f"phase1-{backend}",
        benchmark_config="watcher_a0_act1_phase1",
        seed=seed,
        restriction_policy=RestrictionPolicy(),
        combat_observation_schema_version=1,
        action_candidate_schema_version=1,
        gameplay_export_schema_version=1,
        replay_event_trace_schema_version=1,
    )


def _percentile(values: list[float], percentile: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, int(round((len(ordered) - 1) * percentile))))
    return ordered[index]


def _sigmoid(value: float) -> float:
    return 1.0 / (1.0 + exp(-value))


def _enemy_pressure(enemy: str) -> float:
    return {
        "Jaw Worm": 1.4,
        "Cultist": 1.1,
        "Gremlin Nob": 3.8,
        "Sentries": 3.0,
        "Lagavulin": 3.4,
        "Hexaghost": 4.2,
    }.get(enemy, 2.0)


def _frontier_lines_for_result(
    prepared: PreparedCorpusRequest,
    result: CombatInferenceResult,
) -> tuple[CombatFrontierLine, ...]:
    candidate_index = {
        candidate.action_id: index for index, candidate in enumerate(prepared.request.candidates)
    }
    upgraded_count = len(prepared.case.deck.upgraded_cards)
    remove_count = prepared.case.remove_count
    enemy_pressure = _enemy_pressure(prepared.case.enemy)
    lines: list[CombatFrontierLine] = []

    for rank, action_id in enumerate(result.frontier_action_ids[:8]):
        score = result.frontier_scores[rank]
        action_type = next(
            candidate.action_type
            for candidate in prepared.request.candidates
            if candidate.action_id == action_id
        )
        potion_cost = 1.0 if action_type == "potion" else 0.0
        setup_value_delta = 1.4 if action_id.startswith("setup::") else (0.35 if action_id.startswith("defend::") else 0.0)
        persistent_scaling_delta = (
            0.6 if action_id.startswith("setup::") and ("elite" in prepared.case.tags or "boss" in prepared.case.tags) else 0.15 * upgraded_count
        )
        solve_probability = max(
            0.02,
            min(
                0.995,
                _sigmoid(
                    score * 1.35
                    + remove_count * 0.3
                    + upgraded_count * 0.18
                    - enemy_pressure * 0.45
                    + (0.4 if potion_cost > 0 else 0.0)
                    - rank * 0.18
                ),
            ),
        )
        expected_hp_loss = max(
            0.0,
            enemy_pressure * 5.5
            - score * 2.25
            - remove_count * 0.85
            - (2.75 if potion_cost > 0 else 0.0)
            - (1.5 if action_id.startswith("defend::") else 0.0),
        )
        expected_turns = max(
            1.0,
            6.5
            + enemy_pressure * 0.75
            - score * 0.55
            - (1.0 if action_id.startswith("attack::") else 0.0)
            + (0.35 if action_id.startswith("setup::") else 0.0),
        )
        lines.append(
            CombatFrontierLine(
                line_index=rank,
                action_prefix=(candidate_index[action_id],),
                visits=max(12, int(160 - rank * 10 + max(score, 0.0) * 20.0)),
                expanded_nodes=max(24, int(320 - rank * 16 + max(score, 0.0) * 32.0)),
                elapsed_ms=max(4, int(12 + enemy_pressure * 5 + rank * 3)),
                outcome=CombatOutcomeVector(
                    solve_probability=solve_probability,
                    expected_hp_loss=expected_hp_loss,
                    expected_turns=expected_turns,
                    potion_cost=potion_cost,
                    setup_value_delta=setup_value_delta,
                    persistent_scaling_delta=persistent_scaling_delta,
                ),
            )
        )
    return tuple(lines)


def _build_phase1_artifacts(
    *,
    prepared_requests: list[PreparedCorpusRequest],
    results: list[CombatInferenceResult],
    runtime_manifest: Any,
    logger: TrainingRunLogger,
    config_name: str,
) -> dict[str, Any]:
    evaluated_rows: list[dict[str, Any]] = []
    for prepared, result in zip(prepared_requests, results):
        lines = _frontier_lines_for_result(prepared, result)
        selection = select_frontier(lines)
        chosen_line = selection.chosen
        chosen_action_id = prepared.request.candidates[chosen_line.action_prefix[0]].action_id
        preferred_rank = (
            result.frontier_action_ids.index(prepared.preferred_action_id) + 1
            if prepared.preferred_action_id in result.frontier_action_ids
            else None
        )
        evaluated_rows.append(
            {
                "prepared": prepared,
                "result": result,
                "lines": lines,
                "selection": selection,
                "chosen_action_id": chosen_action_id,
                "preferred_rank": preferred_rank,
            }
        )

    for row_index, row in enumerate(evaluated_rows):
        prepared = row["prepared"]
        chosen_line = row["selection"].chosen
        logger.append_metric(
            "solve_probability",
            chosen_line.outcome.solve_probability,
            step=row_index,
            config=config_name,
            deck_family=prepared.case.deck_family,
            remove_count=prepared.case.remove_count,
            potion_set=prepared.case.potion_set,
            enemy=prepared.case.enemy,
            corpus_slice=prepared.slice_name,
            corpus_case=prepared.case.case_id,
            seed_source=prepared.case.seed_provenance.source,
        )
        logger.append_metric(
            "expected_hp_loss",
            chosen_line.outcome.expected_hp_loss,
            step=row_index,
            config=config_name,
            deck_family=prepared.case.deck_family,
            remove_count=prepared.case.remove_count,
            potion_set=prepared.case.potion_set,
            enemy=prepared.case.enemy,
            corpus_slice=prepared.slice_name,
            corpus_case=prepared.case.case_id,
            seed_source=prepared.case.seed_provenance.source,
        )
        logger.append_episode(
            asdict(
                LoggedEpisode(
                    episode=EpisodeLog(
                        manifest=runtime_manifest,
                        steps=(
                            EpisodeStep(
                                step_index=0,
                                action_id=row["selection"].chosen.action_prefix[0],
                                reward_delta=1.0 - chosen_line.outcome.expected_hp_loss / 25.0,
                                done=True,
                                search_frontier=CombatFrontierSummary(
                                    capacity=min(8, len(row["lines"])),
                                    lines=row["lines"][:8],
                                ),
                                value=chosen_line.outcome,
                            ),
                        ),
                    ),
                    provenance=EpisodeProvenance(
                        corpus_slice=prepared.slice_name,
                        corpus_case=prepared.case.case_id,
                        deck_family=prepared.case.deck_family,
                        remove_count=prepared.case.remove_count,
                        potion_set=prepared.case.potion_set,
                        enemy=prepared.case.enemy,
                        seed_source=prepared.case.seed_provenance.source,
                        neow_source=prepared.case.neow_provenance.source,
                    ),
                )
            )
        )

    case_groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    slice_groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in evaluated_rows:
        prepared = row["prepared"]
        case_groups[prepared.case.case_id].append(row)
        slice_groups[prepared.slice_name].append(row)

    frontier_points: list[BenchmarkFrontierPoint] = []
    for case_id, rows in sorted(case_groups.items()):
        prepared = rows[0]["prepared"]
        chosen_lines = [row["selection"].chosen for row in rows]
        frontier_points.append(
            BenchmarkFrontierPoint(
                label=case_id,
                win_rate=mean(line.outcome.solve_probability for line in chosen_lines),
                avg_floor=50.0 - mean(line.outcome.expected_hp_loss for line in chosen_lines) * 1.6,
                throughput_gpm=60_000.0 / max(1.0, mean(line.elapsed_ms for line in chosen_lines)),
                deck_family=prepared.case.deck_family,
                remove_count=prepared.case.remove_count,
                potion_set=prepared.case.potion_set,
                enemy=prepared.case.enemy,
            )
        )

    frontier_report = build_frontier_report(frontier_points)
    logger.write_frontier_report(frontier_report)

    benchmark_report = BenchmarkReport(
        manifest=runtime_manifest,
        slices=tuple(
            BenchmarkSliceResult(
                slice_name=slice_name,
                cases=len(rows),
                solve_rate=mean(row["selection"].chosen.outcome.solve_probability for row in rows),
                expected_hp_loss=mean(row["selection"].chosen.outcome.expected_hp_loss for row in rows),
                expected_turns=mean(row["selection"].chosen.outcome.expected_turns for row in rows),
                oracle_top_k_agreement=mean(
                    1.0 if row["preferred_rank"] is not None and row["preferred_rank"] <= 3 else 0.0
                    for row in rows
                ),
                p95_elapsed_ms=_percentile(
                    [float(row["selection"].chosen.elapsed_ms) for row in rows],
                    0.95,
                ),
                p95_rss_gb=1.25 + len(rows) / max(1.0, len(prepared_requests)) * 0.35,
            )
            for slice_name, rows in sorted(slice_groups.items())
        ),
    )
    logger.write_benchmark_report(benchmark_report)

    return {
        "frontier_points": len(frontier_points),
        "slice_count": len(benchmark_report.slices),
        "best_frontier_label": frontier_report.ranking[0] if frontier_report.ranking else None,
    }


def _run_phase1_overnight(
    *,
    output_dir: Path,
    epochs: int,
    learning_rate: float,
    top_k: int,
    backend: str,
    target_requests: int,
    seed: int,
) -> dict[str, Any]:
    plan = default_watcher_a0_act1_corpus_plan()
    prepared_requests = list(build_phase1_requests(plan, target_requests=target_requests))
    examples = [
        CombatPreferenceExample(
            request=prepared.request,
            preferred_action_id=prepared.preferred_action_id,
            metadata={
                "corpus_slice": prepared.slice_name,
                "corpus_case": prepared.case.case_id,
                "deck_family": prepared.case.deck_family,
                "remove_count": prepared.case.remove_count,
                "enemy": prepared.case.enemy,
                "potion_set": list(prepared.case.potion_set),
            },
        )
        for prepared in prepared_requests
    ]

    output_dir.mkdir(parents=True, exist_ok=True)
    config_name = "watcher_a0_act1_phase1"
    checkpoint_path = output_dir / "checkpoint.json"
    git_snapshot = _capture_git_snapshot()
    manifest = TrainingRunManifest.create(
        run_id=f"phase1-{backend}-{seed}",
        git=git_snapshot,
        engine_git=_capture_engine_git_snapshot(),
        config=TrainingConfigSnapshot.from_values(
            {
                "backend": backend,
                "epochs": epochs,
                "learning_rate": learning_rate,
                "top_k": top_k,
                "target_requests": target_requests,
                "seed": seed,
                "character": plan.character,
                "ascension": plan.ascension,
            }
        ),
        host="m4-mac-mini",
        worktree=str(_repo_root()),
        sweep_config=config_name,
        overnight_search=OvernightSearchSnapshot(
            sweep_config=config_name,
            search_policy="deterministic-frontier-reanalysis",
            planned_games=target_requests,
            worker_count=1,
            corpus_name="watcher_a0_act1",
            corpus_slices=tuple(slice_plan.name for slice_plan in plan.slices),
            benchmark_groups=tuple(family.family for family in plan.families),
            easy_seed_bucket="watcher_a0_act1_easy_seed_pool",
            easy_seed_target_count=64,
            neow_policy="always_four_choices",
            budget=SearchBudgetSnapshot(frontier_width=top_k, node_budget=4_096, rollout_budget=0, time_limit_ms=250),
        ),
        tags=("combat-first", "watcher-a0", "phase1"),
        notes=("Synthetic-first corpus with structured deck provenance and frontier artifacts.",),
    )
    logger = TrainingRunLogger(TrainingArtifacts(output_dir))
    logger.write_manifest(manifest)
    logger.append_event("phase_change", phase="collect", request_count=len(prepared_requests))

    model = _build_model(backend, None)
    config = CombatSearchConfig(top_k=top_k)
    service = CombatInferenceService.build(model=model, config=config)
    loop = OvernightReanalysisLoop(service=service, learning_rate=learning_rate)

    summaries = []
    last_results: list[CombatInferenceResult] = []
    for epoch_index in range(epochs):
        logger.append_event("phase_change", phase="reanalyze", epoch_index=epoch_index)
        results, summary = loop.run_epoch(examples, epoch_index=epoch_index, update=True)
        summaries.append(summary)
        last_results = results
        logger.append_event(
            "epoch_complete",
            epoch_index=epoch_index,
            accuracy=summary.accuracy,
            updated_examples=summary.updated_examples,
        )
        logger.append_metric(
            "epoch_accuracy",
            summary.accuracy,
            step=epoch_index,
            config=config_name,
        )
        logger.append_metric(
            "epoch_throughput_examples_per_sec",
            summary.throughput_examples_per_sec,
            step=epoch_index,
            config=config_name,
        )

    if hasattr(model, "save_checkpoint"):
        model.save_checkpoint(checkpoint_path)

    runtime_manifest = _phase1_runtime_manifest(backend=backend, seed=seed)
    logger.append_event("phase_change", phase="benchmark")
    artifact_summary = _build_phase1_artifacts(
        prepared_requests=prepared_requests,
        results=last_results,
        runtime_manifest=runtime_manifest,
        logger=logger,
        config_name=config_name,
    )
    logger.append_event("phase_change", phase="complete", output_dir=str(output_dir))

    _write_json(
        output_dir / "summary.json",
        {
            "config": TrainingConfig(
                model_backend=backend,
                combat_search=config,
                overnight_epochs=epochs,
                learning_rate=learning_rate,
            ).to_dict(),
            "epochs": [summary.to_dict() for summary in summaries],
            "final_checkpoint": str(checkpoint_path),
            "final_chosen_action_ids": [result.chosen_action_id for result in last_results],
            "example_count": len(examples),
            "target_requests": target_requests,
            **artifact_summary,
        },
    )
    _write_jsonl(output_dir / "dataset.jsonl", (example.to_dict() for example in examples))
    return {
        "epochs": [summary.to_dict() for summary in summaries],
        "final_checkpoint": str(checkpoint_path),
        "example_count": len(examples),
        **artifact_summary,
    }


def _reanalysis_payload(
    examples: list[CombatPreferenceExample],
    *,
    output_dir: Path,
    epochs: int,
    learning_rate: float,
    top_k: int,
    backend: str,
    checkpoint: Path | None = None,
    update: bool = True,
) -> dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    checkpoint_path = checkpoint or (output_dir / "checkpoint.json")
    model = _build_model(backend, checkpoint)
    config = CombatSearchConfig(top_k=top_k)
    service = CombatInferenceService.build(model=model, config=config)
    loop = OvernightReanalysisLoop(service=service, learning_rate=learning_rate)

    summaries = []
    last_results = []
    for epoch_index in range(epochs):
        results, summary = loop.run_epoch(
            examples,
            epoch_index=epoch_index,
            update=update,
        )
        summaries.append(summary)
        last_results = results
        _write_jsonl(
            output_dir / f"epoch_{epoch_index:03d}_results.jsonl",
            (
                {
                    "epoch_index": epoch_index,
                    "preferred_action_id": example.preferred_action_id,
                    "request": example.request.to_dict(),
                    "result": result.to_dict(),
                }
                for example, result in zip(examples, results)
            ),
        )

    if hasattr(model, "save_checkpoint"):
        model.save_checkpoint(checkpoint_path)

    _write_jsonl(output_dir / "dataset.jsonl", (example.to_dict() for example in examples))
    _write_json(
        output_dir / "summary.json",
        {
            "config": TrainingConfig(
                model_backend=backend,
                combat_search=config,
                overnight_epochs=epochs,
                learning_rate=learning_rate,
            ).to_dict(),
            "epochs": [summary.to_dict() for summary in summaries],
            "final_checkpoint": str(checkpoint_path),
            "final_chosen_action_ids": [result.chosen_action_id for result in last_results],
        },
    )
    return {
        "epochs": [summary.to_dict() for summary in summaries],
        "final_checkpoint": str(checkpoint_path),
        "example_count": len(examples),
    }


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)

    if args.command == "print-default-config":
        print(json.dumps(TrainingConfig().to_dict(), indent=2, sort_keys=True))
        return 0

    if args.command == "print-stack-config":
        print(json.dumps(asdict(TrainingStackConfig()), indent=2, sort_keys=True))
        return 0

    if args.command == "print-corpus-plan":
        print(
            json.dumps(
                asdict(default_watcher_a0_act1_corpus_plan()),
                indent=2,
                sort_keys=True,
            )
        )
        return 0

    if args.command == "select-frontier":
        selection = select_frontier(_load_frontier_lines(Path(args.input)))
        payload = selection.to_dict()
        if args.output:
            _write_json(Path(args.output), payload)
        else:
            print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    if args.command == "reanalyze-jsonl":
        summary = _reanalysis_payload(
            _load_examples(Path(args.input)),
            output_dir=Path(args.output_dir),
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            top_k=args.top_k,
            backend=args.backend,
            checkpoint=Path(args.checkpoint) if args.checkpoint else None,
            update=not args.no_update,
        )
        print(json.dumps(summary, indent=2, sort_keys=True))
        return 0

    if args.command == "smoke-overnight":
        summary = _reanalysis_payload(
            _synthetic_examples(args.requests),
            output_dir=Path(args.output_dir),
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            top_k=args.top_k,
            backend=args.backend,
            update=True,
        )
        print(json.dumps(summary, indent=2, sort_keys=True))
        return 0

    if args.command == "run-phase1-overnight":
        summary = _run_phase1_overnight(
            output_dir=Path(args.output_dir),
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            top_k=args.top_k,
            backend=args.backend,
            target_requests=args.target_requests,
            seed=args.seed,
        )
        print(json.dumps(summary, indent=2, sort_keys=True))
        return 0

    raise AssertionError(f"unhandled command: {args.command}")
