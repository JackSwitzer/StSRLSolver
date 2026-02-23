"""Audit tests for POW-002 runtime hook dispatch coverage."""

from __future__ import annotations

import ast
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = REPO_ROOT / "packages/engine/registry/powers.py"
RUNTIME_FILES = [
    REPO_ROOT / "packages/engine/combat_engine.py",
    REPO_ROOT / "packages/engine/handlers/combat.py",
]


def _parse_registry_hooks() -> set[str]:
    mod = ast.parse(REGISTRY_PATH.read_text(encoding="utf-8"))
    hooks: set[str] = set()
    for node in ast.walk(mod):
        if not isinstance(node, ast.FunctionDef):
            continue
        for deco in node.decorator_list:
            if not (
                isinstance(deco, ast.Call)
                and isinstance(deco.func, ast.Name)
                and deco.func.id == "power_trigger"
            ):
                continue
            if deco.args and isinstance(deco.args[0], ast.Constant):
                hooks.add(str(deco.args[0].value))
    return hooks


def _parse_runtime_trigger_hooks() -> set[str]:
    hooks: set[str] = set()
    pattern = re.compile(r'execute_power_triggers\(\s*"([^"]+)"')
    for path in RUNTIME_FILES:
        text = path.read_text(encoding="utf-8")
        hooks.update(pattern.findall(text))
    return hooks


def test_high_priority_power_hooks_are_runtime_dispatched() -> None:
    registry_hooks = _parse_registry_hooks()
    runtime_hooks = _parse_runtime_trigger_hooks()

    required_hooks = {
        "atStartOfTurnPostDraw",
        "onCardDraw",
        "onApplyPower",
        "onScry",
        "onAttackedToChangeDamage",
    }

    for hook in required_hooks:
        assert hook in registry_hooks, f"{hook} not registered in power registry"
        assert hook in runtime_hooks, f"{hook} not dispatched by runtime"
