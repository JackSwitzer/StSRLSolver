from packages.training.manifests import GitSnapshot, TrainingConfigSnapshot, TrainingRunManifest


def test_training_manifest_round_trips_with_stable_config_hash(tmp_path):
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
        config=config,
        tags=["combat-first"],
        notes=["smoke"],
    )

    destination = manifest.write_json(tmp_path / "manifest.json")
    loaded = TrainingRunManifest.from_dict(__import__("json").loads(destination.read_text()))

    assert destination.exists()
    assert loaded.run_id == "run_001"
    assert loaded.config.config_hash == config.config_hash
    assert loaded.git.branch == "codex/training-rebuild"
