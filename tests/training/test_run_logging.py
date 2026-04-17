import json

from packages.training.contracts import BenchmarkReport, BenchmarkSliceResult, EpisodeLog, EpisodeStep
from packages.training.benchmarking import BenchmarkFrontierPoint, build_frontier_report
from packages.training.episode_log import EpisodeLogger, EpisodeProvenance, LoggedEpisode
from packages.training.manifests import TrainingRunManifest
from packages.training.run_logging import TrainingArtifacts, TrainingRunLogger
from packages.training.system_stats import SystemStatsSnapshot


def test_run_logger_writes_manifest_events_metrics_frontier_and_episode_artifacts(tmp_path):
    artifacts = TrainingArtifacts(tmp_path / "run")
    logger = TrainingRunLogger(artifacts)

    manifest = TrainingRunManifest.create(run_id="run_002", sweep_config="baseline_control")
    logger.write_manifest(manifest)
    logger.append_event("phase_change", phase="collect")
    logger.append_system_stats(
        SystemStatsSnapshot(
            timestamp="2026-04-16T00:00:00Z",
            phase="collect",
            step=1,
            process_pid=123,
            process_cpu_percent=88.0,
            process_rss_gb=1.5,
            process_memory_percent=6.0,
            host_cpu_percent=71.0,
            host_memory_used_gb=11.5,
            host_memory_total_gb=24.0,
            host_memory_percent=48.0,
            host_swap_used_gb=0.0,
            host_swap_percent=0.0,
            gpu_percent=None,
            gpu_sampler="powermetrics",
            gpu_status="powermetrics requires superuser",
            note="collection progress",
        )
    )
    logger.append_metric(
        "games_per_min",
        42.5,
        step=7,
        config="baseline_control",
        deck_family="single-strike-remove",
        remove_count=1,
        potion_set=("Fire Potion",),
        enemy="Gremlin Nob",
        corpus_slice="curated-core",
        corpus_case="gremlin-nob-fire-potion",
        seed_source="easy_seed_placeholder",
    )
    logger.append_episode(
        {
            "episode_id": "ep-1",
            "corpus_case": "gremlin-nob-fire-potion",
            "deck_family": "single-strike-remove",
        }
    )
    report = build_frontier_report(
        [
            BenchmarkFrontierPoint(
                "baseline",
                win_rate=0.2,
                avg_floor=12.0,
                throughput_gpm=18.0,
                deck_family="starter-vanilla",
                remove_count=0,
                enemy="Jaw Worm",
            ),
            BenchmarkFrontierPoint(
                "combat-first",
                win_rate=0.25,
                avg_floor=13.5,
                throughput_gpm=16.5,
                deck_family="single-strike-remove",
                remove_count=1,
                potion_set=("Fire Potion",),
                enemy="Gremlin Nob",
            ),
        ]
    )
    logger.write_frontier_report(report)
    logger.write_benchmark_report(
        BenchmarkReport(
            manifest=None,
            slices=(
                BenchmarkSliceResult(
                    slice_name="curated-core",
                    cases=2,
                    solve_rate=0.5,
                    expected_hp_loss=6.0,
                    expected_turns=4.0,
                    oracle_top_k_agreement=0.5,
                    p95_elapsed_ms=22.0,
                    p95_rss_gb=1.4,
                ),
            ),
        )
    )

    assert artifacts.manifest_path.exists()
    assert artifacts.events_path.exists()
    assert artifacts.metrics_path.exists()
    assert artifacts.system_stats_path.exists()
    assert artifacts.frontier_report_path.exists()
    assert artifacts.benchmark_report_path.exists()
    assert artifacts.frontier_markdown_path.exists()
    assert artifacts.frontier_groups_path.exists()
    assert artifacts.episode_log_path.exists()

    event = json.loads(artifacts.events_path.read_text().strip().splitlines()[0])
    metric = json.loads(artifacts.metrics_path.read_text().strip().splitlines()[0])
    system_stat = json.loads(artifacts.system_stats_path.read_text().strip().splitlines()[0])
    frontier = json.loads(artifacts.frontier_report_path.read_text())
    benchmark = json.loads(artifacts.benchmark_report_path.read_text())
    groups = json.loads(artifacts.frontier_groups_path.read_text())
    episode = json.loads(artifacts.episode_log_path.read_text().strip().splitlines()[0])

    assert event["event_type"] == "phase_change"
    assert event["phase"] == "collect"
    assert metric["name"] == "games_per_min"
    assert metric["step"] == 7
    assert metric["deck_family"] == "single-strike-remove"
    assert metric["remove_count"] == 1
    assert metric["potion_set"] == ["Fire Potion"]
    assert system_stat["phase"] == "collect"
    assert system_stat["gpu_status"] == "powermetrics requires superuser"
    assert frontier["ranking"][0] == "combat-first"
    assert benchmark["slices"][0]["slice_name"] == "curated-core"
    assert any(group["key"]["enemy"] == "Gremlin Nob" for group in groups)
    assert episode["corpus_case"] == "gremlin-nob-fire-potion"


def test_episode_logger_writes_episode_provenance_wrapper(tmp_path):
    logger = EpisodeLogger(tmp_path / "episodes.jsonl")
    logger.append(
        LoggedEpisode(
            episode=EpisodeLog(
                manifest=None,
                steps=(EpisodeStep(step_index=0, action_id=5, reward_delta=0.0, done=False),),
            ),
            provenance=EpisodeProvenance(
                corpus_slice="frontier-harvest-hard",
                corpus_case="lagavulin-hard-remove",
                deck_family="single-strike-remove",
                remove_count=1,
                potion_set=("Fear Potion",),
                enemy="Lagavulin",
                seed_source="easy_seed_placeholder",
                neow_source="neow_placeholder",
            ),
        )
    )

    payload = json.loads((tmp_path / "episodes.jsonl").read_text().strip())

    assert payload["provenance"]["deck_family"] == "single-strike-remove"
    assert payload["provenance"]["potion_set"] == ["Fear Potion"]
    assert payload["episode"]["steps"][0]["action_id"] == 5
