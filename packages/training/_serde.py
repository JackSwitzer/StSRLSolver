"""Small JSON serialization helpers shared by the new training modules."""

from __future__ import annotations

from dataclasses import asdict, is_dataclass
from enum import Enum
import hashlib
import json
from pathlib import Path
from typing import Any, TypeAlias

JsonScalar: TypeAlias = None | bool | int | float | str
JsonValue: TypeAlias = JsonScalar | list["JsonValue"] | dict[str, "JsonValue"]


def to_jsonable(value: Any) -> JsonValue:
    """Convert common Python objects into stable JSON-compatible values."""
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if isinstance(value, Enum):
        return value.value
    if isinstance(value, Path):
        return str(value)
    if is_dataclass(value):
        return to_jsonable(asdict(value))
    if isinstance(value, dict):
        return {str(key): to_jsonable(item) for key, item in sorted(value.items(), key=lambda pair: str(pair[0]))}
    if isinstance(value, (list, tuple)):
        return [to_jsonable(item) for item in value]
    if isinstance(value, (set, frozenset)):
        return [to_jsonable(item) for item in sorted(value, key=repr)]
    raise TypeError(f"Value of type {type(value).__name__} is not JSON serializable")


def stable_json_dumps(value: Any, *, indent: int | None = None) -> str:
    """Dump JSON with stable ordering for hashing and deterministic tests."""
    jsonable = to_jsonable(value)
    if indent is None:
        return json.dumps(jsonable, sort_keys=True, ensure_ascii=True, separators=(",", ":"))
    return json.dumps(jsonable, sort_keys=True, ensure_ascii=True, indent=indent)


def json_sha256(value: Any) -> str:
    """Hash a JSON-serializable value using a stable encoding."""
    payload = stable_json_dumps(value).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()
