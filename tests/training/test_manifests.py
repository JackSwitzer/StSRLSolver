from packages.training.manifests import (
    GitSnapshot,
    OvernightSearchSnapshot,
    SearchBudgetSnapshot,
    TrainingConfigSnapshot,
    TrainingRunManifest,
)


def test_training_manifest_round_trips_with_stable_config_hash_and_overnight_fields(tmp_path):
    config = TrainingConfigSnapshot.from_values(
        {
            "algorithm": "ppo",
            "batch_size": 256,
            "restrictions": {"allow_card_rewards": False},
        }
    )
    manifest = TrainingRunManifest.create(
        run_id="run_001",
        git=GitSnapshot(commit_sha="abc123", branch="codex/training-rebuild", dirty=True),
        engine_git=GitSnapshot(commit_sha="def456", branch="codex/universal-gameplay-runtime", dirty=False),
        config=config,
        host="trainer-01",
        worktree="/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild",
        sweep_config="baseline_control",
        overnight_search=OvernightSearchSnapshot(
            sweep_config="baseline_control",
            search_policy="beam-search",
            planned_games=4000,
            worker_count=4,
            corpus_name="watcher_a0_act1",
            corpus_slices=("curated-core", "frontier-harvest-hard"),
            benchmark_groups=("single-strike-remove|1|Fire Potion|Gremlin Nob",),
            easy_seed_bucket="watcher_a0_act1_easy_seed_pool",
            easy_seed_target_count=64,
            neow_policy="always_four_choices",
            budget=SearchBudgetSnapshot(frontier_width=32, node_budget=12000, rollout_budget=0, time_limit_ms=250),
        ),
        tags=["combat-first"],
        notes=["smoke"],
    )

    destination = manifest.write_json(tmp_path / "manifest.json")
    loaded = TrainingRunManifest.from_dict(__import__("json").loads(destination.read_text()))

    assert destination.exists()
    assert loaded.run_id == "run_001"
    assert loaded.config.config_hash == config.config_hash
    assert loaded.git.branch == "codex/training-rebuild"
    assert loaded.engine_git is not None
    assert loaded.engine_git.branch == "codex/universal-gameplay-runtime"
    assert loaded.sweep_config == "baseline_control"
    assert loaded.overnight_search is not None
    assert loaded.overnight_search.corpus_slices == ("curated-core", "frontier-harvest-hard")
    assert loaded.overnight_search.easy_seed_target_count == 64
