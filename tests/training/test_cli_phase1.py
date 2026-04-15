from __future__ import annotations

import json
from pathlib import Path

from packages.training.cli import main


def test_cli_select_frontier_writes_deterministic_choice(tmp_path: Path) -> None:
    frontier_path = tmp_path / "frontier.json"
    output_path = tmp_path / "selection.json"
    frontier_path.write_text(
        json.dumps(
            {
                "lines": [
                    {
                        "line_index": 2,
                        "action_prefix": [2],
                        "visits": 100,
                        "expanded_nodes": 80,
                        "elapsed_ms": 30,
                        "outcome": {
                            "solve_probability": 0.95,
                            "expected_hp_loss": 1.0,
                            "expected_turns": 3.0,
                            "potion_cost": 0.0,
                            "setup_value_delta": 0.0,
                            "persistent_scaling_delta": 0.0,
                        },
                    },
                    {
                        "line_index": 1,
                        "action_prefix": [1],
                        "visits": 90,
                        "expanded_nodes": 70,
                        "elapsed_ms": 28,
                        "outcome": {
                            "solve_probability": 0.99,
                            "expected_hp_loss": 4.0,
                            "expected_turns": 4.0,
                            "potion_cost": 0.0,
                            "setup_value_delta": 0.0,
                            "persistent_scaling_delta": 0.0,
                        },
                    },
                ]
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    exit_code = main(["select-frontier", "--input", str(frontier_path), "--output", str(output_path)])

    payload = json.loads(output_path.read_text(encoding="utf-8"))
    assert exit_code == 0
    assert payload["chosen_line_index"] == 1
    assert payload["ordered_line_indices"] == [1, 2]


def test_cli_smoke_overnight_writes_dataset_summary_and_checkpoint(tmp_path: Path) -> None:
    output_dir = tmp_path / "overnight"

    exit_code = main(
        [
            "smoke-overnight",
            "--output-dir",
            str(output_dir),
            "--requests",
            "6",
            "--epochs",
            "2",
            "--learning-rate",
            "0.2",
        ]
    )

    summary = json.loads((output_dir / "summary.json").read_text(encoding="utf-8"))
    dataset_lines = (output_dir / "dataset.jsonl").read_text(encoding="utf-8").strip().splitlines()
    checkpoint = json.loads((output_dir / "checkpoint.json").read_text(encoding="utf-8"))

    assert exit_code == 0
    assert len(dataset_lines) == 6
    assert len(summary["epochs"]) == 2
    assert "candidate_weights" in checkpoint
    assert (output_dir / "epoch_000_results.jsonl").exists()
    assert (output_dir / "epoch_001_results.jsonl").exists()


def test_cli_phase1_overnight_writes_monitor_artifacts(tmp_path: Path) -> None:
    output_dir = tmp_path / "phase1"

    exit_code = main(
        [
            "run-phase1-overnight",
            "--output-dir",
            str(output_dir),
            "--epochs",
            "1",
            "--target-requests",
            "12",
            "--backend",
            "linear",
        ]
    )

    manifest = json.loads((output_dir / "manifest.json").read_text(encoding="utf-8"))
    summary = json.loads((output_dir / "summary.json").read_text(encoding="utf-8"))
    frontier_report = json.loads((output_dir / "frontier_report.json").read_text(encoding="utf-8"))
    benchmark_report = json.loads((output_dir / "benchmark_report.json").read_text(encoding="utf-8"))
    metrics_lines = (output_dir / "metrics.jsonl").read_text(encoding="utf-8").strip().splitlines()
    event_lines = (output_dir / "events.jsonl").read_text(encoding="utf-8").strip().splitlines()
    episode_lines = (output_dir / "episodes.jsonl").read_text(encoding="utf-8").strip().splitlines()

    assert exit_code == 0
    assert manifest["run_id"] == "phase1-linear-42"
    assert summary["example_count"] == 12
    assert summary["slice_count"] == 3
    assert frontier_report["ranking"]
    assert benchmark_report["slices"]
    assert len(metrics_lines) >= 12
    assert len(event_lines) >= 4
    assert len(episode_lines) == 12


def test_cli_generate_phase1_corpus_writes_deterministic_inventory(tmp_path: Path) -> None:
    output_dir = tmp_path / "corpus"

    exit_code = main(
        [
            "generate-phase1-corpus",
            "--output-dir",
            str(output_dir),
            "--target-cases",
            "12",
        ]
    )

    corpus_lines = (output_dir / "corpus.jsonl").read_text(encoding="utf-8").strip().splitlines()
    summary = json.loads((output_dir / "corpus_summary.json").read_text(encoding="utf-8"))
    first = json.loads(corpus_lines[0])

    assert exit_code == 0
    assert len(corpus_lines) == 12
    assert summary["total_cases"] == 12
    assert summary["corpus_name"] == "watcher_a0_act1"
    assert first["corpus_id"].startswith("watcher_a0_act1::")
    assert first["corpus_group"].startswith(first["slice_name"])
    assert first["request"]["metadata"]["corpus_id"] == first["corpus_id"]


def test_cli_collect_puct_targets_writes_multi_pass_bundles_and_report(tmp_path: Path) -> None:
    corpus_dir = tmp_path / "corpus"
    collect_dir = tmp_path / "targets"
    main(["generate-phase1-corpus", "--output-dir", str(corpus_dir), "--target-cases", "12"])

    exit_code = main(
        [
            "collect-puct-targets",
            "--input",
            str(corpus_dir / "corpus.jsonl"),
            "--output-dir",
            str(collect_dir),
            "--collection-passes",
            "3",
        ]
    )

    summary = json.loads((collect_dir / "puct_targets_summary.json").read_text(encoding="utf-8"))
    report = json.loads((collect_dir / "puct_targets_report.json").read_text(encoding="utf-8"))
    combined_lines = (collect_dir / "puct_targets.jsonl").read_text(encoding="utf-8").strip().splitlines()

    assert exit_code == 0
    assert len(combined_lines) == 12
    assert summary["collection_passes"] == 3
    assert summary["pass_count"] == 3
    assert report["collection_passes"] == 3
    assert len(report["pass_summaries"]) == 3
    assert (collect_dir / "puct_targets_pass_000.jsonl").exists()
    assert (collect_dir / "puct_targets_pass_001.jsonl").exists()
    assert (collect_dir / "puct_targets_pass_002.jsonl").exists()


def test_cli_train_puct_checkpoint_consumes_collected_targets(tmp_path: Path) -> None:
    corpus_dir = tmp_path / "corpus"
    collect_dir = tmp_path / "targets"
    train_dir = tmp_path / "train"
    main(["generate-phase1-corpus", "--output-dir", str(corpus_dir), "--target-cases", "12"])
    main(
        [
            "collect-puct-targets",
            "--input",
            str(corpus_dir / "corpus.jsonl"),
            "--output-dir",
            str(collect_dir),
            "--collection-passes",
            "3",
        ]
    )

    exit_code = main(
        [
            "train-puct-checkpoint",
            "--input-dir",
            str(collect_dir),
            "--output-dir",
            str(train_dir),
            "--epochs",
            "1",
            "--backend",
            "linear",
        ]
    )

    summary = json.loads((train_dir / "puct_training_summary.json").read_text(encoding="utf-8"))
    checkpoint = json.loads((train_dir / "checkpoint.json").read_text(encoding="utf-8"))

    assert exit_code == 0
    assert summary["example_count"] == 12
    assert summary["epochs"]
    assert "candidate_weights" in checkpoint


def test_cli_validate_seed_suite_writes_report_for_fixed_three_seeds(tmp_path: Path) -> None:
    output_dir = tmp_path / "seed-validation"

    exit_code = main(
        [
            "validate-seed-suite",
            "--output-dir",
            str(output_dir),
        ]
    )

    report = json.loads((output_dir / "seed_validation_report.json").read_text(encoding="utf-8"))
    summary = json.loads((output_dir / "seed_validation_summary.json").read_text(encoding="utf-8"))

    assert exit_code == 0
    assert report["seed_count"] == 3
    assert summary["seed_count"] == 3
    assert report["issues"] == []
    assert report["labels"] == [
        "minimalist_remove",
        "lesson_learned_shell",
        "icecream_runic_pyramid",
    ]


def test_cli_run_phase1_puct_overnight_writes_all_artifacts(tmp_path: Path) -> None:
    output_dir = tmp_path / "phase1-puct"

    exit_code = main(
        [
            "run-phase1-puct-overnight",
            "--output-dir",
            str(output_dir),
            "--target-cases",
            "12",
            "--collection-passes",
            "3",
            "--epochs",
            "1",
            "--backend",
            "linear",
        ]
    )

    summary = json.loads((output_dir / "summary.json").read_text(encoding="utf-8"))
    manifest = json.loads((output_dir / "manifest.json").read_text(encoding="utf-8"))

    assert exit_code == 0
    assert summary["command"] == "run-phase1-puct-overnight"
    assert summary["corpus_summary"]["total_cases"] == 12
    assert summary["collection_summary"]["collection_passes"] == 3
    assert summary["seed_summary"]["seed_count"] == 3
    assert manifest["corpus_summary"]["total_cases"] == 12
    assert (output_dir / "corpus.jsonl").exists()
    assert (output_dir / "puct_targets.jsonl").exists()
    assert (output_dir / "seed_validation_report.json").exists()
