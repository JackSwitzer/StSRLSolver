from packages.training.seed_suite import default_watcher_validation_seed_suite, validate_watcher_validation_seed_suite


def test_default_watcher_validation_seed_suite_is_fixed_to_three_validation_seeds():
    seeds = default_watcher_validation_seed_suite()
    report = validate_watcher_validation_seed_suite(seeds)

    assert len(seeds) == 3
    assert all(seed.character == "Watcher" for seed in seeds)
    assert all(seed.suggested_eval_ascension == 0 for seed in seeds)
    assert {seed.seed for seed in seeds} == {
        "4AWM3ECVQDEWJ",
        "4VM6JKC3KR3TD",
        "1TPMUARFP690B",
    }
    assert {seed.source.value for seed in seeds} == {"Baalorlord", "Steam"}
    assert report.issues == ()
    assert report.to_dict()["seed_count"] == 3
    assert report.to_dict()["all_watcher"] is True
