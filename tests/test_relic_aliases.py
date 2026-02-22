"""Relic ID alias and inventory closure tests (REL-006)."""

import pytest

from packages.engine.content.relics import RelicTier, get_relic, get_relics_by_tier
from packages.engine.state.run import create_watcher_run


@pytest.mark.parametrize(
    ("alias_id", "canonical_id"),
    [
        ("Abacus", "TheAbacus"),
        ("Courier", "The Courier"),
        ("Wing Boots", "WingedGreaves"),
        ("Waffle", "Lee's Waffle"),
        ("White Beast", "White Beast Statue"),
        ("SneckoSkull", "Snake Skull"),
        ("SnakeRing", "Ring of the Snake"),
        ("PhilosopherStone", "Philosopher's Stone"),
    ],
)
def test_get_relic_resolves_java_style_alias_ids(alias_id: str, canonical_id: str) -> None:
    relic = get_relic(alias_id)
    assert relic.id == canonical_id


def test_toolbox_is_registered_as_shop_relic() -> None:
    toolbox = get_relic("Toolbox")
    assert toolbox.id == "Toolbox"
    assert toolbox.tier == RelicTier.SHOP

    shop_relic_ids = {relic.id for relic in get_relics_by_tier(RelicTier.SHOP)}
    assert "Toolbox" in shop_relic_ids


def test_run_state_add_relic_canonicalizes_aliases() -> None:
    run = create_watcher_run("ALIAS_CANON", ascension=0)
    run.add_relic("Courier")

    assert run.has_relic("Courier")
    assert run.has_relic("The Courier")
    assert run.get_relic("Courier") is not None
    assert run.get_relic("Courier").id == "The Courier"


def test_run_state_alias_canonicalization_preserves_pickup_effects() -> None:
    run = create_watcher_run("ALIAS_EFFECT", ascension=0)
    run.current_hp -= 10
    max_hp_before = run.max_hp

    run.add_relic("Waffle")

    assert run.has_relic("Lee's Waffle")
    assert run.max_hp == max_hp_before + 7
    assert run.current_hp == run.max_hp
