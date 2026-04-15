from packages.training.seed_suite import default_watcher_validation_seed_suite


def test_default_watcher_validation_seed_suite_is_balanced_for_easy_and_control_runs():
    seeds = default_watcher_validation_seed_suite()

    assert len(seeds) >= 8
    assert all(seed.character == "Watcher" for seed in seeds)
    assert all(seed.suggested_eval_ascension == 0 for seed in seeds)
    assert any("easy" in seed.tags for seed in seeds)
    assert any("minimalist-style" in seed.tags for seed in seeds)
    assert any("loss-control" in seed.tags for seed in seeds)
    assert all(seed.source_url.startswith("https://baalorlord.tv/runs/") for seed in seeds)
