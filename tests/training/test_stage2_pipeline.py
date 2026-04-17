from __future__ import annotations

from packages.training.seed_imports import build_imported_combat_cases
from packages.training.stage2_pipeline import (
    build_phase2_snapshot_corpus,
    collect_rust_puct_records,
)


def test_imported_act1_cases_reconstruct_watcher_combat_entries() -> None:
    cases = build_imported_combat_cases()

    assert len(cases) == 20
    assert {case.seed_label for case in cases} == {
        "minimalist_remove",
        "lesson_learned_shell",
    }
    assert cases[0].deck[:5] == (
        "Strike_P",
        "Strike_P",
        "Strike_P",
        "Strike_P",
        "Defend_P",
    )
    assert cases[0].relics == ("PureWater",)
    assert "TalkToTheHand" in cases[-1].deck


def test_phase2_snapshot_corpus_is_deterministic_and_mixed_source() -> None:
    first = build_phase2_snapshot_corpus(total_cases=12)
    second = build_phase2_snapshot_corpus(total_cases=12)

    assert tuple(case.case_id for case in first) == tuple(case.case_id for case in second)
    assert {case.source_kind for case in first} == {"synthetic", "imported_seed"}
    assert any(case.relic_profile == "starting_only" for case in first)
    assert first[0].snapshot["player_hp"] >= 12
    assert first[0].snapshot["hand"]
    assert any(case.seed_label == "minimalist_remove" for case in first)


def test_phase2_collection_runs_real_rust_puct_over_snapshots() -> None:
    cases = build_phase2_snapshot_corpus(total_cases=2)
    records = collect_rust_puct_records(
        cases=cases,
        collection_passes=1,
    )

    assert len(records) == 2
    for record in records:
        assert record.puct_result.root_total_visits >= 1
        assert record.puct_result.root_action_ids
        assert record.puct_result.stop_reason.value in {
            "Converged",
            "HardVisitCap",
            "TimeCap",
            "TerminalRoot",
            "NoLegalActions",
        }
