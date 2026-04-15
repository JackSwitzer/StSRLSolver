from __future__ import annotations

from packages import training


def test_training_package_exports_skeleton_surface() -> None:
    assert hasattr(training, "CombatInferenceService")
    assert hasattr(training, "CombatStateSummary")
    assert hasattr(training, "LegalCombatCandidate")
    assert hasattr(training, "TrainingConfig")

