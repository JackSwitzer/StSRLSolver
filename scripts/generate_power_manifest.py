#!/usr/bin/env python3
"""Generate deterministic Java-vs-Python power parity manifest."""

from __future__ import annotations

import argparse
import ast
import importlib
import importlib.util
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Dict, Iterable, List, Tuple


HOOK_METHODS = {
    "atDamageFinalReceive": r"atDamageFinalReceive\s*\(",
    "atDamageGive": r"atDamageGive\s*\(",
    "atDamageReceive": r"atDamageReceive\s*\(",
    "atEndOfRound": r"atEndOfRound\s*\(",
    "atEndOfTurn": r"atEndOfTurn\s*\(",
    "atEndOfTurnPreEndTurnCards": r"atEndOfTurnPreEndTurnCards\s*\(",
    "atStartOfTurn": r"atStartOfTurn\s*\(",
    "atStartOfTurnPostDraw": r"atStartOfTurnPostDraw\s*\(",
    "modifyBlock": r"modifyBlock\s*\(",
    "onAfterCardPlayed": r"onAfterCardPlayed\s*\(",
    "onAfterUseCard": r"onAfterUseCard\s*\(",
    "onApplyPower": r"onApplyPower\s*\(",
    "onAttack": r"onAttack\s*\(",
    "onAttacked": r"onAttacked\s*\(",
    "onAttackedToChangeDamage": r"onAttackedToChangeDamage\s*\(",
    "onCardDraw": r"onCardDraw\s*\(",
    "onChangeStance": r"onChangeStance\s*\(",
    "onDeath": r"onDeath\s*\(",
    "onEnergyRecharge": r"onEnergyRecharge\s*\(",
    "onExhaust": r"onExhaust\s*\(",
    "onScry": r"onScry\s*\(",
    "onUseCard": r"onUseCard\s*\(",
    "wasHPLost": r"wasHPLost\s*\(",
}


def normalize_power_id(power_id: str) -> str:
    """Canonical normalization for power IDs/class names."""
    text = "".join(ch for ch in power_id if ch.isalnum()).lower()
    if text.endswith("power"):
        text = text[:-5]
    return text


def split_camel_words(name: str) -> str:
    text = re.sub(r"Power$", "", name)
    text = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", text)
    return text.strip() or name


def load_powers_module(repo_root: Path):
    """Load packages.engine.content.powers with package-relative imports enabled."""
    repo_root_str = str(repo_root)
    if repo_root_str not in sys.path:
        sys.path.insert(0, repo_root_str)
    return importlib.import_module("packages.engine.content.powers")


def parse_java_powers(java_root: Path) -> Dict[str, Dict[str, Any]]:
    rows: Dict[str, Dict[str, Any]] = {}
    for path in sorted(java_root.rglob("*.java")):
        rel = str(path).replace("\\", "/")
        if "/deprecated/" in rel or path.name == "AbstractPower.java":
            continue

        text = path.read_text(encoding="utf-8", errors="ignore")
        class_name = path.stem

        hooks: List[str] = []
        for hook, method_pattern in HOOK_METHODS.items():
            pattern = rf"public\s+[^\n{{]*{method_pattern}"
            if re.search(pattern, text):
                hooks.append(hook)

        power_type_match = re.search(r"PowerType\.(BUFF|DEBUFF)", text)
        power_type = power_type_match.group(1) if power_type_match else "BUFF"

        is_turn_based = "isTurnBased = true" in text

        priority_match = re.search(r"priority\s*=\s*(-?\d+)", text)
        priority = int(priority_match.group(1)) if priority_match else None

        id_match = re.search(r"POWER_ID\s*=\s*\"([^\"]+)\"", text)
        if not id_match:
            id_match = re.search(r"this\.ID\s*=\s*\"([^\"]+)\"", text)
        java_id = id_match.group(1) if id_match else class_name

        rows[class_name] = {
            "java_class": class_name,
            "java_id": java_id,
            "java_hooks_overridden": sorted(hooks),
            "java_power_type": power_type,
            "java_is_turn_based": is_turn_based,
            "java_priority": priority,
            "java_source": rel,
        }

    return rows


def parse_registry_hooks(registry_path: Path) -> Dict[str, List[str]]:
    mod = ast.parse(registry_path.read_text(encoding="utf-8"))
    hooks_by_power: Dict[str, set[str]] = defaultdict(set)

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
            hook = None
            power = None
            if deco.args and isinstance(deco.args[0], ast.Constant) and isinstance(deco.args[0].value, str):
                hook = deco.args[0].value
            for kw in deco.keywords:
                if kw.arg == "power" and isinstance(kw.value, ast.Constant) and isinstance(kw.value.value, str):
                    power = kw.value.value
            if hook and power:
                hooks_by_power[power].add(hook)

    return {k: sorted(v) for k, v in sorted(hooks_by_power.items())}


def choose_canonical_by_norm(candidates: Iterable[str]) -> str:
    ranked = sorted(set(candidates), key=lambda x: (len(x), x.lower(), x))
    return ranked[0]


def map_java_to_python(
    java_rows: Dict[str, Dict[str, Any]],
    power_data: Dict[str, Dict[str, Any]],
    aliases: Dict[str, str],
    registry_hooks: Dict[str, List[str]],
) -> List[Dict[str, Any]]:
    ids_by_norm: Dict[str, List[str]] = defaultdict(list)
    for power_id in power_data:
        ids_by_norm[normalize_power_id(power_id)].append(power_id)

    alias_to_target = dict(aliases)

    rows: List[Dict[str, Any]] = []

    for class_name in sorted(java_rows):
        row = dict(java_rows[class_name])
        java_id = row["java_id"]
        python_power_id = None
        used_alias = None

        # 1) direct exact key
        if java_id in power_data:
            python_power_id = java_id
        elif class_name in power_data:
            python_power_id = class_name

        # 2) explicit alias map
        if python_power_id is None:
            for alias_key in (java_id, class_name, re.sub(r"Power$", "", class_name)):
                if alias_key in alias_to_target:
                    candidate = alias_to_target[alias_key]
                    if candidate in power_data:
                        python_power_id = candidate
                        used_alias = alias_key
                        break

        # 3) normalized match
        if python_power_id is None:
            for probe in (java_id, class_name, re.sub(r"Power$", "", class_name), split_camel_words(class_name)):
                norm = normalize_power_id(probe)
                if norm in ids_by_norm:
                    python_power_id = choose_canonical_by_norm(ids_by_norm[norm])
                    break

        registry_for_power: List[str] = []
        if python_power_id is not None:
            target_norm = normalize_power_id(python_power_id)
            registry_set = set()
            for reg_power, hooks in registry_hooks.items():
                if normalize_power_id(reg_power) == target_norm:
                    registry_set.update(hooks)
            registry_for_power = sorted(registry_set)

        if python_power_id is None:
            status = "missing"
        else:
            java_norm = normalize_power_id(class_name)
            py_norm = normalize_power_id(python_power_id)
            status = "exact" if java_norm == py_norm else "alias"

        row.update(
            {
                "python_power_id": python_power_id,
                "python_registry_hooks": registry_for_power,
                "status": status,
                "mapping_via": used_alias,
            }
        )
        rows.append(row)

    return rows


def write_manifest_json(path: Path, rows: List[Dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(rows, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_manifest_markdown(path: Path, rows: List[Dict[str, Any]]) -> None:
    counts = Counter(row["status"] for row in rows)
    missing = [r for r in rows if r["status"] == "missing"]
    lines = [
        "# Power Manifest",
        "",
        "Deterministic Java-vs-Python manifest for power inventory and hook coverage.",
        "",
        f"- Java classes: `{len(rows)}`",
        f"- `exact`: `{counts.get('exact', 0)}`",
        f"- `alias`: `{counts.get('alias', 0)}`",
        f"- `missing`: `{counts.get('missing', 0)}`",
        "",
        "## Missing Java Classes",
        "",
    ]
    if missing:
        for row in missing:
            lines.append(f"- `{row['java_class']}` (hooks: {', '.join(row['java_hooks_overridden']) or 'none'})")
    else:
        lines.append("- None")

    lines.extend(
        [
            "",
            "## Rows",
            "",
            "| java_class | java_hooks_overridden | python_power_id | python_registry_hooks | status |",
            "|---|---|---|---|---|",
        ]
    )

    for row in rows:
        java_hooks = ", ".join(row["java_hooks_overridden"]) or "-"
        py_hooks = ", ".join(row["python_registry_hooks"]) or "-"
        py_id = row["python_power_id"] or "-"
        lines.append(
            f"| `{row['java_class']}` | `{java_hooks}` | `{py_id}` | `{py_hooks}` | `{row['status']}` |"
        )

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--java-root",
        type=Path,
        default=Path("/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers"),
        help="Path to decompiled Java powers root",
    )
    parser.add_argument(
        "--manifest-json",
        type=Path,
        default=repo_root / "docs/audits/2026-02-22-full-game-parity/traceability/power-manifest.json",
        help="Output JSON manifest path",
    )
    parser.add_argument(
        "--manifest-md",
        type=Path,
        default=repo_root / "docs/audits/2026-02-22-full-game-parity/traceability/power-manifest.md",
        help="Output markdown summary path",
    )
    args = parser.parse_args()

    powers_module = load_powers_module(repo_root)

    power_data: Dict[str, Dict[str, Any]] = getattr(powers_module, "POWER_DATA")
    aliases: Dict[str, str] = getattr(powers_module, "POWER_ID_ALIASES")

    java_rows = parse_java_powers(args.java_root)
    registry_hooks = parse_registry_hooks(repo_root / "packages/engine/registry/powers.py")

    rows = map_java_to_python(java_rows, power_data, aliases, registry_hooks)

    write_manifest_json(args.manifest_json, rows)
    write_manifest_markdown(args.manifest_md, rows)

    counts = Counter(row["status"] for row in rows)
    print(f"Generated {len(rows)} power rows")
    print(f"exact={counts.get('exact', 0)} alias={counts.get('alias', 0)} missing={counts.get('missing', 0)}")
    print(f"JSON: {args.manifest_json}")
    print(f"MD:   {args.manifest_md}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
