"""Thin bridge helpers for the Rust training contract session API."""

from __future__ import annotations

import json
from typing import Any

from .contracts import (
    CombatPuctConfig,
    CombatPuctResult,
    CombatSnapshot,
    CombatTrainingState,
    RestrictionPolicy,
    TrainingSchemaVersions,
    parse_combat_puct_result as _parse_combat_puct_result,
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


def parse_combat_puct_result(payload: dict[str, Any]) -> CombatPuctResult:
    return _parse_combat_puct_result(payload)


def load_training_schema_versions(engine_session: Any) -> TrainingSchemaVersions:
    return parse_training_schema_versions(engine_session.get_training_schema_versions())


def load_combat_training_state(
    engine_session: Any,
    policy: RestrictionPolicy | None = None,
) -> CombatTrainingState:
    policy_json = _policy_json(policy)
    try:
        if policy_json is None:
            payload = engine_session.get_combat_training_state()
        else:
            payload = engine_session.get_combat_training_state(policy_json)
    except TypeError:
        payload = engine_session.get_combat_training_state()
    return parse_combat_training_state(payload)


def load_combat_snapshot(engine_session: Any) -> CombatSnapshot:
    return parse_combat_snapshot(engine_session.get_combat_snapshot())


def run_combat_puct(
    engine_session: Any,
    evaluator,
    config: CombatPuctConfig | None = None,
) -> CombatPuctResult:
    config_json = None if config is None else json.dumps(config.to_dict())
    payload = engine_session.run_combat_puct(evaluator, config_json)
    return parse_combat_puct_result(payload)
