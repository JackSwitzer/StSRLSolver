import json

from packages.training.benchmarking import BenchmarkFrontierPoint, build_frontier_report
from packages.training.manifests import TrainingRunManifest
from packages.training.run_logging import TrainingArtifacts, TrainingRunLogger


def test_run_logger_writes_manifest_events_metrics_and_frontier(tmp_path):
    artifacts = TrainingArtifacts(tmp_path / "run")
    logger = TrainingRunLogger(artifacts)

    manifest = TrainingRunManifest.create(run_id="run_002")
    logger.write_manifest(manifest)
    logger.append_event("phase_change", phase="collect")
    logger.append_metric("games_per_min", 42.5, step=7, config="baseline_control")
    report = build_frontier_report(
        [
            BenchmarkFrontierPoint("baseline", win_rate=0.2, avg_floor=12.0, throughput_gpm=18.0),
            BenchmarkFrontierPoint("combat-first", win_rate=0.25, avg_floor=13.5, throughput_gpm=16.5),
        ]
    )
    logger.write_frontier_report(report)

    assert artifacts.manifest_path.exists()
    assert artifacts.events_path.exists()
    assert artifacts.metrics_path.exists()
    assert artifacts.frontier_report_path.exists()
    assert artifacts.frontier_markdown_path.exists()

    event = json.loads(artifacts.events_path.read_text().strip().splitlines()[0])
    metric = json.loads(artifacts.metrics_path.read_text().strip().splitlines()[0])
    frontier = json.loads(artifacts.frontier_report_path.read_text())

    assert event["event_type"] == "phase_change"
    assert event["phase"] == "collect"
    assert metric["name"] == "games_per_min"
    assert metric["step"] == 7
    assert frontier["ranking"][0] == "combat-first"
