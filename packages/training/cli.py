"""CLI for the canonical combat-first Rust-PUCT training stack."""

from __future__ import annotations

import argparse
import json
import socket
import subprocess
from collections import defaultdict
from dataclasses import asdict
from pathlib import Path
from statistics import fmean
from time import perf_counter
from typing import Any, Iterable

from .benchmarking import build_frontier_report
from .config import TrainingStackConfig
from .contracts import (
    BenchmarkReport,
    BenchmarkSliceResult,
    EpisodeLog,
    EpisodeStep,
    RestrictionPolicy,
)
from .inference_service import (
    CombatInferenceService,
    CombatPolicyValueTrainer,
    CombatSearchConfig,
    PolicyValueEpochSummary,
)
from .manifests import (
    GitSnapshot,
    OvernightSearchSnapshot,
    SearchBudgetSnapshot,
    TrainingConfigSnapshot,
    TrainingRunManifest,
    build_run_manifest,
)
from .run_logging import TrainingArtifacts, TrainingRunLogger
from .seed_imports import default_imported_act1_scripts
from .shared_memory import CombatPuctTargetExample
from .stage2_pipeline import (
    PuctCollectionRecord,
    build_seed_validation_report,
    frontier_points_from_records,
    load_snapshot_corpus,
    records_to_puct_targets,
    write_puct_collection,
    write_snapshot_corpus,
)
from .system_stats import SystemStatsSampler
from .value_targets import PHASE1_POTION_VOCAB


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Combat-first training rebuild tools")
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("print-corpus-plan", help="Print the current snapshot corpus plan")
    subparsers.add_parser("print-seed-suite", help="Print the reconstructed Act 1 validation seed suite")

    generate_corpus_parser = subparsers.add_parser(
        "generate-phase1-corpus",
        help="Generate the deterministic Watcher A0 snapshot corpus",
    )
    generate_corpus_parser.add_argument("--output-dir", required=True)
    generate_corpus_parser.add_argument("--target-cases", type=int, default=50_000)

    collect_targets_parser = subparsers.add_parser(
        "collect-puct-targets",
        help="Run Rust PUCT collection over a generated snapshot corpus",
    )
    collect_targets_parser.add_argument("--input", required=True)
    collect_targets_parser.add_argument("--output-dir", required=True)
    collect_targets_parser.add_argument("--collection-passes", type=int, default=3)

    train_checkpoint_parser = subparsers.add_parser(
        "train-puct-checkpoint",
        help="Train a policy/value checkpoint from PUCT target examples",
    )
    train_checkpoint_parser.add_argument("--input-dir", required=True)
    train_checkpoint_parser.add_argument("--output-dir", required=True)
    train_checkpoint_parser.add_argument("--epochs", type=int, default=4)
    train_checkpoint_parser.add_argument("--learning-rate", type=float, default=0.01)
    train_checkpoint_parser.add_argument("--top-k", type=int, default=8)
    train_checkpoint_parser.add_argument("--checkpoint")
    train_checkpoint_parser.add_argument("--no-update", action="store_true")

    validate_seed_parser = subparsers.add_parser(
        "validate-seed-suite",
        help="Validate the reconstructed Act 1 Watcher seed suite",
    )
    validate_seed_parser.add_argument("--output-dir", required=True)
    validate_seed_parser.add_argument("--checkpoint")

    puct_overnight_parser = subparsers.add_parser(
        "run-phase1-puct-overnight",
        help="Generate corpus, collect Rust PUCT targets, train, benchmark, and validate seeds",
    )
    puct_overnight_parser.add_argument("--output-dir", required=True)
    puct_overnight_parser.add_argument("--target-cases", type=int, default=50_000)
    puct_overnight_parser.add_argument("--collection-passes", type=int, default=3)
    puct_overnight_parser.add_argument("--epochs", type=int, default=4)
    puct_overnight_parser.add_argument("--learning-rate", type=float, default=0.01)
    puct_overnight_parser.add_argument("--top-k", type=int, default=8)
    puct_overnight_parser.add_argument("--seed", type=int, default=42)

    return parser


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


def _load_puct_target_examples(input_dir: Path) -> list[CombatPuctTargetExample]:
    collection = input_dir / "puct_collection.jsonl"
    if collection.exists():
        records = [PuctCollectionRecord.from_dict(row) for row in _iter_jsonl(collection)]
        return records_to_puct_targets(records)

    combined = input_dir / "puct_targets.jsonl"
    if combined.exists():
        return [CombatPuctTargetExample.from_dict(row) for row in _iter_jsonl(combined)]

    if input_dir.is_file():
        return [CombatPuctTargetExample.from_dict(row) for row in _iter_jsonl(input_dir)]
    raise FileNotFoundError(f"no PUCT targets found in {input_dir}")


def _write_puct_target_examples(path: Path, examples: Iterable[CombatPuctTargetExample]) -> None:
    _write_jsonl(path, (example.to_dict() for example in examples))


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
    return GitSnapshot(commit_sha=commit_sha, branch="codex/universal-gameplay-runtime", dirty=False)


def _runtime_contract_manifest(*, seed: int):
    return build_run_manifest(
        model_version="phase1-policy-value-mlx",
        benchmark_config="watcher_a0_act1_snapshot",
        seed=seed,
        restriction_policy=RestrictionPolicy(),
        combat_observation_schema_version=1,
        action_candidate_schema_version=1,
        gameplay_export_schema_version=1,
        replay_event_trace_schema_version=1,
    )


def _print_corpus_plan() -> int:
    plan = {
        "corpus_name": "watcher_a0_act1_snapshot",
        "target_cases": 50_000,
        "source_mix": {"synthetic": 42_000, "imported_seed": 8_000},
        "synthetic_backbone": [
            "starting_only",
            "starting_only_remove_heavy",
            "starting_only_upgrade_heavy",
            "starting_only_stance_shell",
            "starting_only_retain_control",
        ],
        "minority_ablations": [
            "extra_relic_ablation_akabeko",
            "extra_relic_ablation_frozen_eye",
            "extra_relic_ablation_pocketwatch",
            "extra_relic_ablation_ice_cream",
        ],
        "canonical_starting_relic": "PureWater",
        "backend_policy": "mlx_only",
        "potion_vocab": list(PHASE1_POTION_VOCAB),
    }
    print(json.dumps(plan, indent=2, sort_keys=True))
    return 0


def _print_seed_suite() -> int:
    scripts = default_imported_act1_scripts()
    payload = {
        "suite_name": "watcher_validation_suite",
        "seed_count": len(scripts),
        "seeds": [
            {
                "label": script.label,
                "seed": script.seed,
                "source_url": script.source_url,
                "source_ascension": script.source_ascension,
                "eval_ascension": script.eval_ascension,
                "status": "reconstructed_validated" if script.exact_available else "metadata_only",
                "neow_choice": script.neow_choice,
                "floors": len(script.floors),
                "exact_issue": script.exact_issue,
            }
            for script in scripts
        ],
    }
    print(json.dumps(payload, indent=2, sort_keys=True))
    return 0


def _build_model(checkpoint: Path | None):
    from .combat_model import MLXCombatModel

    return MLXCombatModel(checkpoint_path=(str(checkpoint) if checkpoint else None))


def _benchmark_report_from_records(records: list[PuctCollectionRecord], manifest) -> BenchmarkReport:
    buckets: dict[str, list[PuctCollectionRecord]] = defaultdict(list)
    for record in records:
        buckets[record.case.slice_name].append(record)

    slices = []
    for slice_name, slice_records in sorted(buckets.items()):
        solve_rates = [record.puct_result.root_outcome.solve_probability for record in slice_records]
        hp_losses = [record.puct_result.root_outcome.expected_hp_loss for record in slice_records]
        turns = [record.puct_result.root_outcome.expected_turns for record in slice_records]
        elapsed = [float(record.puct_result.elapsed_ms) for record in slice_records]
        frontier_widths = [float(len(record.puct_result.frontier)) for record in slice_records]
        slices.append(
            BenchmarkSliceResult(
                slice_name=slice_name,
                cases=len(slice_records),
                solve_rate=fmean(solve_rates) if solve_rates else 0.0,
                expected_hp_loss=fmean(hp_losses) if hp_losses else 0.0,
                expected_turns=fmean(turns) if turns else 0.0,
                oracle_top_k_agreement=fmean(frontier_widths) / 8.0 if frontier_widths else 0.0,
                p95_elapsed_ms=max(elapsed) if elapsed else 0.0,
                p95_rss_gb=0.0,
            )
        )
    return BenchmarkReport(manifest=manifest, slices=tuple(slices))


def _episode_payload(record: PuctCollectionRecord, manifest) -> dict[str, Any]:
    action_id = record.puct_result.chosen_action_id
    if action_id is None and record.puct_result.root_action_ids:
        action_id = int(record.puct_result.root_action_ids[0])
    frontier_payload = {
        "capacity": 8,
        "lines": [
            {
                "line_index": line.line_index,
                "action_prefix": list(line.action_prefix),
                "visits": line.visits,
                "expanded_nodes": line.expanded_nodes,
                "elapsed_ms": line.elapsed_ms,
                "outcome": {
                    "solve_probability": line.outcome.solve_probability,
                    "expected_hp_loss": line.outcome.expected_hp_loss,
                    "expected_turns": line.outcome.expected_turns,
                    "potion_cost": line.outcome.potion_cost,
                    "setup_value_delta": line.outcome.setup_value_delta,
                    "persistent_scaling_delta": line.outcome.persistent_scaling_delta,
                },
            }
            for line in record.puct_result.frontier
        ],
    }
    value_payload = {
        "solve_probability": record.puct_result.root_outcome.solve_probability,
        "expected_hp_loss": record.puct_result.root_outcome.expected_hp_loss,
        "expected_turns": record.puct_result.root_outcome.expected_turns,
        "potion_cost": record.puct_result.root_outcome.potion_cost,
        "setup_value_delta": record.puct_result.root_outcome.setup_value_delta,
        "persistent_scaling_delta": record.puct_result.root_outcome.persistent_scaling_delta,
    }
    episode = EpisodeLog(
        manifest=manifest,
        steps=(
            EpisodeStep(
                step_index=0,
                action_id=int(action_id or 0),
                reward_delta=1.0 - float(record.puct_result.root_outcome.expected_hp_loss) / 25.0,
                done=False,
                search_frontier=None,
                value=None,
            ),
        ),
    )
    payload = asdict(episode)
    payload["steps"][0]["search_frontier"] = frontier_payload
    payload["steps"][0]["value"] = value_payload
    return payload


def _train_examples(
    examples: list[CombatPuctTargetExample],
    *,
    output_dir: Path,
    epochs: int,
    learning_rate: float,
    top_k: int,
    checkpoint: Path | None = None,
    update: bool = True,
    logger: TrainingRunLogger | None = None,
    system_sampler: SystemStatsSampler | None = None,
) -> dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    model = _build_model(checkpoint)
    service = CombatInferenceService.build(model=model, config=CombatSearchConfig(top_k=top_k))
    trainer = CombatPolicyValueTrainer(service=service, learning_rate=learning_rate)
    training_started = perf_counter()

    def _on_epoch_complete(summary: PolicyValueEpochSummary) -> None:
        if logger is not None:
            logger.append_event(
                "training_epoch_complete",
                epoch_index=summary.epoch_index,
                example_count=summary.example_count,
                policy_loss=summary.policy_loss,
                value_loss=summary.value_loss,
                throughput_examples_per_sec=summary.throughput_examples_per_sec,
                backend_loaded=model.loaded_backend,
            )
        if logger is not None and system_sampler is not None:
            logger.append_system_stats(
                system_sampler.sample(
                    phase="training_epoch_complete",
                    step=summary.epoch_index,
                    note=f"policy_loss={summary.policy_loss:.6f}, value_loss={summary.value_loss:.6f}",
                )
            )

    summaries = trainer.run(
        examples,
        epochs=epochs,
        update=update,
        on_epoch_complete=_on_epoch_complete,
    )
    checkpoint_path = output_dir / "checkpoint.json"
    model.save_checkpoint(checkpoint_path)

    payload = {
        "example_count": len(examples),
        "epochs": [summary.to_dict() for summary in summaries],
        "learning_rate": learning_rate,
        "top_k": top_k,
        "backend_requested": "mlx",
        "backend_loaded": model.loaded_backend,
        "training_wall_seconds": perf_counter() - training_started,
        "final_checkpoint": str(checkpoint_path),
        "policy_loss": summaries[-1].policy_loss if summaries else 0.0,
        "value_loss": summaries[-1].value_loss if summaries else 0.0,
    }
    _write_json(output_dir / "puct_training_summary.json", payload)
    return payload


def _generate_phase1_corpus(*, output_dir: Path, target_cases: int) -> dict[str, Any]:
    from .engine_module import build_engine_extension

    build_engine_extension()
    summary = write_snapshot_corpus(output_dir, total_cases=target_cases)
    _write_json(output_dir / "corpus_summary.json", {"command": "generate-phase1-corpus", **summary})
    return summary


def _collect_puct_targets(
    *,
    input_path: Path,
    output_dir: Path,
    collection_passes: int,
) -> dict[str, Any]:
    from .engine_module import build_engine_extension

    build_engine_extension()
    cases = load_snapshot_corpus(input_path)
    records = list(
        write_puct_collection(
            output_dir,
            cases=cases,
            collection_passes=collection_passes,
        )
    )
    examples = records_to_puct_targets(records)
    _write_puct_target_examples(output_dir / "puct_targets.jsonl", examples)
    summary = {
        "corpus_name": "watcher_a0_act1_snapshot",
        "total_cases": len(cases),
        "collection_passes": collection_passes,
        "record_count": len(records),
        "target_count": len(examples),
        "combined_targets": str(output_dir / "puct_targets.jsonl"),
        "collection_records": str(output_dir / "puct_collection.jsonl"),
        "backend_requested": "mlx",
        "backend_loaded": "mlx",
    }
    _write_json(output_dir / "puct_targets_summary.json", {"command": "collect-puct-targets", **summary})
    return summary


def _validate_seed_suite(
    *,
    output_dir: Path,
    checkpoint: Path | None = None,
) -> dict[str, Any]:
    from .engine_module import build_engine_extension

    build_engine_extension()
    report = build_seed_validation_report(
        checkpoint=str(checkpoint) if checkpoint else "untrained",
    )
    _write_json(output_dir / "seed_validation_report.json", report)
    (output_dir / "seed_validation_report.md").write_text(
        json.dumps(report, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    summary = {
        "suite_name": report["suite_name"],
        "seed_count": report["seed_count"],
        "validated_seeds": report["validated_seeds"],
        "failed_seeds": report["failed_seeds"],
        "required_seed_count": report["required_seed_count"],
        "metadata_only_count": report["metadata_only_count"],
    }
    _write_json(output_dir / "seed_validation_summary.json", summary)
    return summary


def _run_phase1_puct_overnight(
    *,
    output_dir: Path,
    target_cases: int,
    collection_passes: int,
    epochs: int,
    learning_rate: float,
    top_k: int,
    seed: int,
) -> dict[str, Any]:
    from .engine_module import build_engine_extension

    build_engine_extension()
    output_dir.mkdir(parents=True, exist_ok=True)
    logger = TrainingRunLogger(TrainingArtifacts(output_dir))
    system_sampler = SystemStatsSampler()
    logger.append_system_stats(system_sampler.sample(phase="startup", note="phase1-puct-overnight"))

    corpus_started = perf_counter()
    corpus_summary = write_snapshot_corpus(output_dir, total_cases=target_cases)
    cases = load_snapshot_corpus(output_dir)
    corpus_wall_seconds = perf_counter() - corpus_started
    logger.append_event(
        "corpus_generated",
        total_cases=len(cases),
        target_cases=target_cases,
        backend_loaded="mlx",
        wall_seconds=corpus_wall_seconds,
    )
    logger.append_system_stats(
        system_sampler.sample(
            phase="corpus_generated",
            step=len(cases),
            note=f"wall_seconds={corpus_wall_seconds:.3f}",
        )
    )

    def _on_record(record: PuctCollectionRecord, record_index: int) -> None:
        if record_index == 1 or record_index % 25 == 0:
            logger.append_event(
                "collection_progress",
                record_index=record_index,
                collection_pass=record.collection_pass,
                case_id=record.case.case_id,
                deck_family=record.case.deck_family,
                enemy=record.case.enemy,
                stop_reason=record.puct_result.stop_reason.value,
                root_total_visits=record.puct_result.root_total_visits,
                frontier_width=len(record.puct_result.frontier),
                elapsed_ms=record.puct_result.elapsed_ms,
                backend_loaded="mlx",
            )
            logger.append_system_stats(
                system_sampler.sample(
                    phase="collection_progress",
                    step=record_index,
                    note=f"pass={record.collection_pass}, case={record.case.case_id}",
                )
            )

    collection_started = perf_counter()
    records = list(
        write_puct_collection(
            output_dir,
            cases=cases,
            collection_passes=collection_passes,
            on_record=_on_record,
        )
    )
    collection_wall_seconds = perf_counter() - collection_started
    targets = records_to_puct_targets(records)
    _write_puct_target_examples(output_dir / "puct_targets.jsonl", targets)
    logger.append_event(
        "puct_collection_complete",
        record_count=len(records),
        collection_passes=collection_passes,
        backend_loaded="mlx",
        wall_seconds=collection_wall_seconds,
    )
    logger.append_system_stats(
        system_sampler.sample(
            phase="puct_collection_complete",
            step=len(records),
            note=f"wall_seconds={collection_wall_seconds:.3f}",
        )
    )

    training_started = perf_counter()
    training_summary = _train_examples(
        targets,
        output_dir=output_dir,
        epochs=epochs,
        learning_rate=learning_rate,
        top_k=top_k,
        update=True,
        logger=logger,
        system_sampler=system_sampler,
    )
    training_wall_seconds = perf_counter() - training_started
    checkpoint_path = Path(training_summary["final_checkpoint"])
    logger.append_event(
        "training_complete",
        epochs=epochs,
        checkpoint=str(checkpoint_path),
        backend_loaded=training_summary["backend_loaded"],
        wall_seconds=training_wall_seconds,
        example_count=training_summary["example_count"],
    )
    logger.append_system_stats(
        system_sampler.sample(
            phase="training_complete",
            step=training_summary["example_count"],
            note=f"wall_seconds={training_wall_seconds:.3f}",
        )
    )

    seed_validation_started = perf_counter()
    seed_summary = _validate_seed_suite(output_dir=output_dir, checkpoint=checkpoint_path)
    seed_validation_wall_seconds = perf_counter() - seed_validation_started
    logger.append_event(
        "seed_validation_complete",
        validated_seeds=seed_summary["validated_seeds"],
        required_seed_count=seed_summary["required_seed_count"],
        metadata_only_count=seed_summary["metadata_only_count"],
        backend_loaded="mlx",
        wall_seconds=seed_validation_wall_seconds,
    )
    logger.append_system_stats(
        system_sampler.sample(
            phase="seed_validation_complete",
            step=seed_summary["validated_seeds"],
            note=f"wall_seconds={seed_validation_wall_seconds:.3f}",
        )
    )

    runtime_manifest = _runtime_contract_manifest(seed=seed)
    training_manifest = TrainingRunManifest.create(
        run_id=f"phase1-puct-mlx-{seed}",
        git=_capture_git_snapshot(),
        config=TrainingConfigSnapshot.from_values(
            {
                "target_cases": target_cases,
                "collection_passes": collection_passes,
                "epochs": epochs,
                "learning_rate": learning_rate,
                "top_k": top_k,
                "backend_policy": "mlx_only",
                "backend_requested": "mlx",
                "backend_loaded_collection": "mlx",
                "backend_loaded_training": training_summary["backend_loaded"],
                "potion_vocab": list(PHASE1_POTION_VOCAB),
            }
        ),
        engine_git=_capture_engine_git_snapshot(),
        host=socket.gethostname(),
        worktree=str(_repo_root()),
        overnight_search=OvernightSearchSnapshot(
            sweep_config="phase1_puct",
            search_policy="rust_puct_policy_value",
            planned_games=target_cases,
            worker_count=12,
            corpus_name="watcher_a0_act1_snapshot",
            corpus_slices=tuple(sorted({case.slice_name for case in cases})),
            benchmark_groups=tuple(sorted({case.deck_family for case in cases})),
            easy_seed_bucket="watcher_validation_suite",
            easy_seed_target_count=3,
            neow_policy="reconstructed_act1_scripts",
            budget=SearchBudgetSnapshot(frontier_width=8),
        ),
        tags=("phase1", "puct", "policy-value"),
        notes=("Canonical overnight path: snapshot corpus -> Rust PUCT -> policy/value training.",),
    )
    logger.write_manifest(training_manifest)

    for index, record in enumerate(records):
        logger.append_metric(
            "root_visits",
            float(record.puct_result.root_total_visits),
            step=index,
            config="phase1_puct",
            deck_family=record.case.deck_family,
            remove_count=record.case.remove_count,
            potion_set=record.case.potion_set,
            enemy=record.case.enemy,
            corpus_slice=record.case.slice_name,
            corpus_case=record.case.case_id,
            seed_source=record.case.seed_label,
        )
        logger.append_metric(
            "solve_probability",
            float(record.puct_result.root_outcome.solve_probability),
            step=index,
            config="phase1_puct",
            deck_family=record.case.deck_family,
            remove_count=record.case.remove_count,
            potion_set=record.case.potion_set,
            enemy=record.case.enemy,
            corpus_slice=record.case.slice_name,
            corpus_case=record.case.case_id,
            seed_source=record.case.seed_label,
        )
        logger.append_episode(_episode_payload(record, runtime_manifest))

    frontier_report = build_frontier_report(frontier_points_from_records(records))
    logger.write_frontier_report(frontier_report)
    logger.write_benchmark_report(_benchmark_report_from_records(records, runtime_manifest))

    summary = {
        "command": "run-phase1-puct-overnight",
        "target_cases": target_cases,
        "collection_passes": collection_passes,
        "epochs": epochs,
        "backend_requested": "mlx",
        "backend_loaded_collection": "mlx",
        "backend_loaded_training": training_summary["backend_loaded"],
        "learning_rate": learning_rate,
        "top_k": top_k,
        "seed": seed,
        "timings": {
            "corpus_wall_seconds": corpus_wall_seconds,
            "collection_wall_seconds": collection_wall_seconds,
            "training_wall_seconds": training_wall_seconds,
            "seed_validation_wall_seconds": seed_validation_wall_seconds,
        },
        "corpus_summary": corpus_summary,
        "collection_summary": {
            "record_count": len(records),
            "target_count": len(targets),
        },
        "training_summary": training_summary,
        "seed_summary": seed_summary,
    }
    _write_json(output_dir / "summary.json", summary)
    return summary


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "print-corpus-plan":
        return _print_corpus_plan()
    if args.command == "print-seed-suite":
        return _print_seed_suite()
    if args.command == "generate-phase1-corpus":
        _generate_phase1_corpus(output_dir=Path(args.output_dir), target_cases=args.target_cases)
        return 0
    if args.command == "collect-puct-targets":
        _collect_puct_targets(
            input_path=Path(args.input),
            output_dir=Path(args.output_dir),
            collection_passes=args.collection_passes,
        )
        return 0
    if args.command == "train-puct-checkpoint":
        examples = _load_puct_target_examples(Path(args.input_dir))
        _train_examples(
            examples,
            output_dir=Path(args.output_dir),
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            top_k=args.top_k,
            checkpoint=(None if args.checkpoint is None else Path(args.checkpoint)),
            update=not args.no_update,
        )
        return 0
    if args.command == "validate-seed-suite":
        _validate_seed_suite(
            output_dir=Path(args.output_dir),
            checkpoint=(None if args.checkpoint is None else Path(args.checkpoint)),
        )
        return 0
    if args.command == "run-phase1-puct-overnight":
        _run_phase1_puct_overnight(
            output_dir=Path(args.output_dir),
            target_cases=args.target_cases,
            collection_passes=args.collection_passes,
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            top_k=args.top_k,
            seed=args.seed,
        )
        return 0

    parser.error(f"unsupported command: {args.command}")
    return 2
