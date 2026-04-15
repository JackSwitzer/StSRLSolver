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
