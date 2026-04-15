from packages.training.benchmarking import (
    BenchmarkFrontierPoint,
    FrontierWeights,
    build_phase1_puct_collection_report,
    build_frontier_report,
    group_frontier_points,
    pareto_frontier,
)
from packages.training.corpus import build_phase1_requests, default_watcher_a0_act1_corpus_plan


def test_pareto_frontier_filters_dominated_points():
    points = [
        BenchmarkFrontierPoint("baseline", win_rate=0.20, avg_floor=12.0, throughput_gpm=20.0),
        BenchmarkFrontierPoint("stronger", win_rate=0.25, avg_floor=14.0, throughput_gpm=19.0),
        BenchmarkFrontierPoint("dominated", win_rate=0.10, avg_floor=9.0, throughput_gpm=12.0),
    ]

    frontier = pareto_frontier(points)

    assert [point.label for point in frontier] == ["baseline", "stronger"]


def test_frontier_report_ranks_renders_and_groups_by_corpus_axes():
    points = [
        BenchmarkFrontierPoint(
            "baseline",
            win_rate=0.20,
            avg_floor=12.0,
            throughput_gpm=20.0,
            deck_family="starter-vanilla",
            remove_count=0,
            potion_set=(),
            enemy="Jaw Worm",
        ),
        BenchmarkFrontierPoint(
            "combat-first",
            win_rate=0.28,
            avg_floor=15.5,
            throughput_gpm=16.0,
            deck_family="single-strike-remove",
            remove_count=1,
            potion_set=("Fire Potion",),
            enemy="Gremlin Nob",
        ),
        BenchmarkFrontierPoint(
            "combat-first-rerun",
            win_rate=0.24,
            avg_floor=14.5,
            throughput_gpm=17.0,
            deck_family="single-strike-remove",
            remove_count=1,
            potion_set=("Fire Potion",),
            enemy="Gremlin Nob",
        ),
    ]

    report = build_frontier_report(points, weights=FrontierWeights(win_rate=0.6, avg_floor=0.3, throughput_gpm=0.1))
    markdown = report.to_markdown()
    grouped = group_frontier_points(points)

    assert report.best_by_metric["win_rate"] == "combat-first"
    assert report.ranking[0] == "combat-first"
    assert "Benchmark Frontier Report" in markdown
    assert "| combat-first |" in markdown
    assert "## Benchmark Groups" in markdown
    assert len(grouped) == 2
    gremlin_group = next(group for group in grouped if group.key.enemy == "Gremlin Nob")
    assert gremlin_group.key.deck_family == "single-strike-remove"
    assert gremlin_group.key.remove_count == 1
    assert gremlin_group.key.potion_set == ("Fire Potion",)
    assert gremlin_group.count == 2


def test_phase1_puct_collection_report_groups_requests_across_passes():
    plan = default_watcher_a0_act1_corpus_plan()
    requests = build_phase1_requests(plan, target_requests=9)

    report = build_phase1_puct_collection_report(requests, collection_passes=3)

    assert report.corpus_name == "watcher_a0_act1"
    assert report.total_cases == 9
    assert report.collection_passes == 3
    assert len(report.pass_summaries) == 3
    assert report.pass_summaries[0].cases == 3
    assert report.pass_summaries[0].unique_groups >= 1
    assert report.to_dict()["collection_passes"] == 3
