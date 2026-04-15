from packages.training.benchmarking import (
    BenchmarkFrontierPoint,
    FrontierWeights,
    build_frontier_report,
    pareto_frontier,
)


def test_pareto_frontier_filters_dominated_points():
    points = [
        BenchmarkFrontierPoint("baseline", win_rate=0.20, avg_floor=12.0, throughput_gpm=20.0),
        BenchmarkFrontierPoint("stronger", win_rate=0.25, avg_floor=14.0, throughput_gpm=19.0),
        BenchmarkFrontierPoint("dominated", win_rate=0.10, avg_floor=9.0, throughput_gpm=12.0),
    ]

    frontier = pareto_frontier(points)

    assert [point.label for point in frontier] == ["baseline", "stronger"]


def test_frontier_report_ranks_and_renders_markdown():
    points = [
        BenchmarkFrontierPoint("baseline", win_rate=0.20, avg_floor=12.0, throughput_gpm=20.0),
        BenchmarkFrontierPoint("combat-first", win_rate=0.28, avg_floor=15.5, throughput_gpm=16.0),
        BenchmarkFrontierPoint("fast", win_rate=0.19, avg_floor=11.0, throughput_gpm=28.0),
    ]

    report = build_frontier_report(points, weights=FrontierWeights(win_rate=0.6, avg_floor=0.3, throughput_gpm=0.1))
    markdown = report.to_markdown()

    assert report.best_by_metric["win_rate"] == "combat-first"
    assert report.ranking[0] == "combat-first"
    assert "Benchmark Frontier Report" in markdown
    assert "| combat-first |" in markdown
