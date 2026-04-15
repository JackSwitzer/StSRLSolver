from packages.training.corpus import (
    PHASE1_CORPUS_TARGET_CASES,
    build_phase1_corpus_requests,
    build_phase1_requests,
    default_watcher_a0_act1_corpus_plan,
    summarize_phase1_corpus_requests,
)


def test_default_corpus_plan_contains_required_slices_and_families():
    plan = default_watcher_a0_act1_corpus_plan()
    slice_names = {slice_plan.name for slice_plan in plan.slices}
    family_names = {family.family for family in plan.families}

    assert plan.character == "Watcher"
    assert plan.ascension == 0
    assert {"curated-core", "opening-hand-buckets", "frontier-harvest-hard"} <= slice_names
    assert {"starter-vanilla", "single-strike-remove", "calm-setup-upgrade"} <= family_names


def test_corpus_cases_carry_structured_deck_seed_and_neow_provenance():
    plan = default_watcher_a0_act1_corpus_plan()
    cases = [case for slice_plan in plan.slices for case in slice_plan.cases]

    assert cases
    assert all(case.deck.family for case in cases)
    assert all(case.seed_provenance.source == "easy_seed_placeholder" for case in cases)
    assert all(case.neow_provenance.policy == "always_four_choices" for case in cases)
    assert all("synthetic" in case.tags for case in cases)

    remove_case = next(case for case in cases if case.case_id == "gremlin-nob-fire-potion")
    assert remove_case.deck_family == "single-strike-remove"
    assert remove_case.remove_count == 1
    assert remove_case.potion_set == ("Fire Potion",)


def test_phase1_request_builder_preserves_corpus_axes_in_request_metadata():
    plan = default_watcher_a0_act1_corpus_plan()
    prepared = build_phase1_requests(plan, target_requests=10)

    assert len(prepared) == 10
    first = prepared[0]
    assert first.request.metadata["corpus_slice"] == first.slice_name
    assert first.request.metadata["corpus_case"] == first.case.case_id
    assert first.request.metadata["deck_family"] == first.case.deck_family
    assert first.request.metadata["remove_count"] == first.case.remove_count
    assert first.request.metadata["opening_hand_bucket"] == first.opening_hand_bucket
    assert first.request.metadata["corpus_index"] == first.corpus_index
    assert first.request.metadata["corpus_id"] == first.corpus_id
    assert first.corpus_id.startswith("watcher_a0_act1::")
    assert first.corpus_group.startswith(first.slice_name)
    assert first.request.candidates[0].legal is True

    round_trip = type(first).from_dict(first.to_dict())
    assert round_trip.corpus_id == first.corpus_id
    assert round_trip.corpus_group == first.corpus_group
    assert round_trip.request.request_id == first.request.request_id


def test_phase1_corpus_requests_default_to_fifty_thousand_deterministic_cases():
    plan = default_watcher_a0_act1_corpus_plan()
    prepared = build_phase1_corpus_requests(plan, total_cases=12)
    summary = summarize_phase1_corpus_requests(prepared)

    assert PHASE1_CORPUS_TARGET_CASES == 50_000
    assert len(prepared) == 12
    assert prepared[0].corpus_index == 0
    assert prepared[0].collection_pass == 0
    assert prepared[0].corpus_id.endswith("::00000")
    assert summary["total_cases"] == 12
    assert summary["corpus_name"] == "watcher_a0_act1"
    assert summary["unique_corpus_groups"] > 1
