from packages.training.corpus import default_watcher_a0_act1_corpus_plan


def test_default_corpus_plan_contains_required_slices():
    plan = default_watcher_a0_act1_corpus_plan()
    slice_names = {slice_plan.name for slice_plan in plan.slices}
    assert plan.character == "Watcher"
    assert plan.ascension == 0
    assert {"curated-core", "opening-hand-buckets", "frontier-harvest-hard"} <= slice_names
