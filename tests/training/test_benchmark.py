from packages.training.benchmark import BenchmarkConfig, frontier_score
from packages.training.contracts import BenchmarkReport, BenchmarkSliceResult


def test_frontier_score_rewards_stronger_solve_rate_and_oracle_agreement():
    report = BenchmarkReport(
        manifest=None,
        slices=(
            BenchmarkSliceResult(
                slice_name="core",
                cases=10,
                solve_rate=0.9,
                expected_hp_loss=5.0,
                expected_turns=4.0,
                oracle_top_k_agreement=0.8,
                p95_elapsed_ms=120.0,
                p95_rss_gb=8.0,
            ),
        ),
    )
    score = frontier_score(report, BenchmarkConfig())
    assert score > 0.0
