"""Audit tests for POW-001 manifest inventory coverage."""

from __future__ import annotations

import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = (
    REPO_ROOT
    / "docs/audits/2026-02-22-full-game-parity/traceability/power-manifest.json"
)


def _load_manifest() -> list[dict]:
    assert MANIFEST_PATH.exists(), f"Missing manifest: {MANIFEST_PATH}"
    rows = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    assert isinstance(rows, list)
    return rows


def test_manifest_rows_cover_all_java_classes() -> None:
    rows = _load_manifest()
    assert len(rows) == 149

    classes = [row["java_class"] for row in rows]
    assert len(classes) == len(set(classes)), "java_class values must be unique"


def test_manifest_has_no_missing_status_rows() -> None:
    rows = _load_manifest()
    statuses = {row["status"] for row in rows}
    assert statuses.issubset({"exact", "alias", "missing"})

    missing = [row["java_class"] for row in rows if row["status"] == "missing"]
    assert not missing, f"Manifest still has missing power mappings: {missing}"
