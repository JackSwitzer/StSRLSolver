"""Engine-backed canonical ID lookups for Stage 2 training data."""

from __future__ import annotations

from collections import defaultdict
from functools import lru_cache
from typing import Any

from .engine_module import load_engine_module


def _normalize_key(value: str) -> str:
    return "".join(ch.lower() for ch in value if ch.isalnum())


def _split_upgrade_suffix(value: str) -> tuple[str, bool]:
    if value.endswith("+"):
        return value[:-1], True
    return value, False


def _apply_upgrade_suffix(card_id: str, upgraded: bool, *, exact_ids: set[str]) -> str:
    if not upgraded:
        return card_id
    upgraded_id = f"{card_id}+"
    return upgraded_id if upgraded_id in exact_ids else card_id


@lru_cache(maxsize=1)
def training_entity_catalog() -> dict[str, list[dict[str, str]]]:
    module = load_engine_module()
    if not hasattr(module, "get_training_entity_catalog"):
        module = load_engine_module(force_rebuild=True, force_reload=True)
    payload = module.get_training_entity_catalog()
    return {
        domain: [dict(row) for row in rows]
        for domain, rows in dict(payload).items()
    }


@lru_cache(maxsize=1)
def _domain_lookup(domain: str) -> tuple[dict[str, tuple[str, ...]], set[str]]:
    rows = training_entity_catalog()[domain]
    lookup: dict[str, set[str]] = defaultdict(set)
    exact_ids: set[str] = set()
    for row in rows:
        entity_id = str(row["id"])
        entity_name = str(row["name"])
        exact_ids.add(entity_id)
        for raw_key in (
            entity_id,
            entity_name,
            entity_id.removesuffix("+"),
            entity_name.removesuffix("+"),
        ):
            normalized = _normalize_key(raw_key)
            if normalized:
                lookup[normalized].add(entity_id)
    return (
        {key: tuple(sorted(values)) for key, values in lookup.items()},
        exact_ids,
    )


def _resolve_unique_id(domain: str, value: str) -> str | None:
    base_value, upgraded = _split_upgrade_suffix(value)
    lookup, exact_ids = _domain_lookup(domain)
    matches = lookup.get(_normalize_key(base_value), ())
    if len(matches) != 1:
        return None
    return _apply_upgrade_suffix(matches[0], upgraded, exact_ids=exact_ids)


WATCHER_CARD_ALIASES: dict[str, str] = {
    "strike": "Strike",
    "strikew": "Strike",
    "strikewatcher": "Strike",
    "strikepurple": "Strike",
    "defend": "Defend",
    "defendw": "Defend",
    "defendwatcher": "Defend",
    "defendpurple": "Defend",
}


def canonicalize_watcher_card_id(value: str) -> str:
    base_value, upgraded = _split_upgrade_suffix(value)
    lookup, exact_ids = _domain_lookup("cards")
    normalized = _normalize_key(base_value)
    alias = WATCHER_CARD_ALIASES.get(normalized)
    if alias is not None:
        return _apply_upgrade_suffix(alias, upgraded, exact_ids=exact_ids)

    exact_match = _resolve_unique_id("cards", value)
    if exact_match is not None:
        return exact_match

    if base_value in exact_ids:
        return _apply_upgrade_suffix(base_value, upgraded, exact_ids=exact_ids)

    matches = lookup.get(normalized, ())
    if matches:
        return _apply_upgrade_suffix(matches[0], upgraded, exact_ids=exact_ids)
    raise KeyError(f"unknown Watcher card id/name: {value}")


def canonicalize_relic_id(value: str) -> str:
    exact_match = _resolve_unique_id("relics", value)
    if exact_match is not None:
        return exact_match
    raise KeyError(f"unknown relic id/name: {value}")


def canonicalize_potion_id(value: str) -> str:
    exact_match = _resolve_unique_id("potions", value)
    if exact_match is not None:
        return exact_match
    raise KeyError(f"unknown potion id/name: {value}")


def canonicalize_watcher_deck(values: tuple[str, ...] | list[str]) -> tuple[str, ...]:
    return tuple(canonicalize_watcher_card_id(value) for value in values)
