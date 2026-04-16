from __future__ import annotations

import json
from pathlib import Path
import pytest

from packages.training.cli import main


def test_cli_no_longer_accepts_backend_flag(tmp_path: Path) -> None:
    corpus_dir = tmp_path / "corpus"
    assert main(["generate-phase1-corpus", "--output-dir", str(corpus_dir), "--target-cases", "4"]) == 0
    with pytest.raises(SystemExit):
        main(
            [
                "collect-puct-targets",
                "--input",
                str(corpus_dir / "corpus.jsonl"),
                "--output-dir",
                str(tmp_path / "targets"),
                "--backend",
                "mlx",
            ]
        )


def test_cli_generate_collect_train_validate_pipeline(tmp_path: Path) -> None:
    corpus_dir = tmp_path / "corpus"
    collect_dir = tmp_path / "targets"
    train_dir = tmp_path / "train"
    validate_dir = tmp_path / "validate"

    assert main(["generate-phase1-corpus", "--output-dir", str(corpus_dir), "--target-cases", "12"]) == 0
    assert main(
        [
            "collect-puct-targets",
            "--input",
            str(corpus_dir / "corpus.jsonl"),
            "--output-dir",
            str(collect_dir),
            "--collection-passes",
            "2",
        ]
    ) == 0
    assert main(
        [
            "train-puct-checkpoint",
            "--input-dir",
            str(collect_dir),
            "--output-dir",
            str(train_dir),
            "--epochs",
            "1",
        ]
    ) == 0
    assert main(
        [
            "validate-seed-suite",
            "--output-dir",
            str(validate_dir),
            "--checkpoint",
            str(train_dir / "checkpoint.json"),
        ]
    ) == 0

    corpus_summary = json.loads((corpus_dir / "corpus_summary.json").read_text(encoding="utf-8"))
    collect_summary = json.loads((collect_dir / "puct_targets_summary.json").read_text(encoding="utf-8"))
    train_summary = json.loads((train_dir / "puct_training_summary.json").read_text(encoding="utf-8"))
    validate_summary = json.loads((validate_dir / "seed_validation_summary.json").read_text(encoding="utf-8"))
    target_lines = (collect_dir / "puct_targets.jsonl").read_text(encoding="utf-8").strip().splitlines()

    assert corpus_summary["total_cases"] == 12
    assert corpus_summary["relic_profile_counts"]["starting_only"] >= 1
    assert collect_summary["record_count"] >= 12
    assert target_lines
    assert train_summary["example_count"] >= 12
    assert Path(train_summary["final_checkpoint"]).exists()
    assert validate_summary["seed_count"] == 3
    assert validate_summary["required_seed_count"] == 2


def test_cli_phase1_puct_overnight_writes_monitor_artifacts(tmp_path: Path) -> None:
    output_dir = tmp_path / "phase1"

    exit_code = main(
        [
            "run-phase1-puct-overnight",
            "--output-dir",
            str(output_dir),
            "--epochs",
            "1",
            "--target-cases",
            "12",
        ]
    )

    manifest = json.loads((output_dir / "manifest.json").read_text(encoding="utf-8"))
    summary = json.loads((output_dir / "summary.json").read_text(encoding="utf-8"))
    frontier_report = json.loads((output_dir / "frontier_report.json").read_text(encoding="utf-8"))
    benchmark_report = json.loads((output_dir / "benchmark_report.json").read_text(encoding="utf-8"))
    metrics_lines = (output_dir / "metrics.jsonl").read_text(encoding="utf-8").strip().splitlines()
    event_lines = (output_dir / "events.jsonl").read_text(encoding="utf-8").strip().splitlines()
    episode_lines = (output_dir / "episodes.jsonl").read_text(encoding="utf-8").strip().splitlines()
    system_stat_lines = (output_dir / "system_stats.jsonl").read_text(encoding="utf-8").strip().splitlines()
    seed_report = json.loads((output_dir / "seed_validation_report.json").read_text(encoding="utf-8"))

    assert exit_code == 0
    assert manifest["run_id"] == "phase1-puct-mlx-42"
    assert summary["training_summary"]["example_count"] >= 12
    assert summary["backend_requested"] == "mlx"
    assert summary["backend_loaded_collection"] == "mlx"
    assert summary["backend_loaded_training"] == "mlx"
    assert frontier_report["ranking"]
    assert benchmark_report["slices"]
    assert len(metrics_lines) >= 12
    assert len(event_lines) >= 4
    assert len(episode_lines) == len(metrics_lines) // 2
    assert len(system_stat_lines) >= 4
    assert seed_report["seed_count"] == 3
    assert seed_report["required_seed_count"] == 2
    assert seed_report["backend_loaded"] == "mlx"
    assert summary["timings"]["collection_wall_seconds"] >= 0.0
