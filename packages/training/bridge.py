"""Thin bridge helpers for the Rust training contract session API."""

from __future__ import annotations

import json
from typing import Any

from .contracts import (
    CombatSnapshot,
    CombatTrainingState,
    RestrictionPolicy,
    TrainingSchemaVersions,
    parse_combat_snapshot as _parse_combat_snapshot,
    parse_combat_training_state as _parse_combat_training_state,
    parse_training_schema_versions as _parse_training_schema_versions,
)


def _policy_json(policy: RestrictionPolicy | None) -> str | None:
    if policy is None:
        return None
    return json.dumps(policy.to_dict())


def parse_training_schema_versions(payload: dict[str, Any]) -> TrainingSchemaVersions:
    return _parse_training_schema_versions(payload)


def parse_combat_training_state(payload: dict[str, Any]) -> CombatTrainingState:
    return _parse_combat_training_state(payload)


def parse_combat_snapshot(payload: dict[str, Any]) -> CombatSnapshot:
    return _parse_combat_snapshot(payload)


def load_training_schema_versions(engine_session: Any) -> TrainingSchemaVersions:
    return parse_training_schema_versions(engine_session.get_training_schema_versions())


def load_combat_training_state(
    engine_session: Any,
    policy: RestrictionPolicy | None = None,
) -> CombatTrainingState:
    payload = engine_session.get_combat_training_state(_policy_json(policy))
    return parse_combat_training_state(payload)


def load_combat_snapshot(engine_session: Any) -> CombatSnapshot:
    return parse_combat_snapshot(engine_session.get_combat_snapshot())
