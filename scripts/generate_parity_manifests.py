#!/usr/bin/env python3
"""Generate deterministic Java-vs-Python inventory and parity manifests."""

from __future__ import annotations

import argparse
import ast
import importlib
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Callable, Dict, Iterable, List, Optional, Sequence, Tuple

CARD_SUBDIRS = {"red", "green", "blue", "purple", "colorless", "curses", "status"}
CARD_EXCLUDED_DIRS = {"deprecated", "optionCards", "tempCards"}
EVENT_EXCLUDED_CLASSES = {
    "AbstractEvent",
    "AbstractImageEvent",
    "GenericEventDialog",
    "RoomEventDialog",
    "AddEventParams",
}
POTION_EXCLUDED_CLASSES = {"AbstractPotion"}
POWER_EXCLUDED_CLASSES = {"AbstractPower"}
POWER_HOOK_METHODS = {
    "atDamageFinalGive": r"atDamageFinalGive\s*\(",
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


def normalize_id(value: str, *, strip_power_suffix: bool = False) -> str:
    token = "".join(ch for ch in value if ch.isalnum()).lower()
    if strip_power_suffix and token.endswith("power"):
        token = token[:-5]
    return token


def split_camel(value: str) -> str:
    return re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", value).strip()


def choose_canonical(candidates: Iterable[str]) -> str:
    ranked = sorted(set(candidates), key=lambda x: (len(x), x.lower(), x))
    return ranked[0]


def load_python_modules(repo_root: Path) -> Dict[str, Any]:
    repo_root_str = str(repo_root)
    if repo_root_str not in sys.path:
        sys.path.insert(0, repo_root_str)

    return {
        "cards": importlib.import_module("packages.engine.content.cards"),
        "relics": importlib.import_module("packages.engine.content.relics"),
        "potions": importlib.import_module("packages.engine.content.potions"),
        "powers": importlib.import_module("packages.engine.content.powers"),
        "events": importlib.import_module("packages.engine.handlers.event_handler"),
    }


def _java_source(path: Path, java_root: Path) -> str:
    return str(path.relative_to(java_root)).replace("\\", "/")


def parse_java_cards(java_root: Path) -> Dict[str, Any]:
    cards_root = java_root / "cards"
    if not cards_root.exists():
        return {"source_missing": True, "items": []}

    items: List[Dict[str, str]] = []
    for path in sorted(cards_root.rglob("*.java")):
        rel = path.relative_to(cards_root)
        parts = rel.parts
        if not parts:
            continue
        if any(part in CARD_EXCLUDED_DIRS for part in parts):
            continue
        if parts[0] not in CARD_SUBDIRS:
            continue

        class_name = path.stem
        if class_name == "AbstractCard":
            continue

        items.append(
            {
                "java_id": class_name,
                "java_class": class_name,
                "java_source": _java_source(path, java_root),
            }
        )

    return {"source_missing": False, "items": items}


def parse_java_relics(java_root: Path) -> Dict[str, Any]:
    relics_root = java_root / "relics"
    if not relics_root.exists():
        return {"source_missing": True, "items": []}

    items: List[Dict[str, str]] = []
    for path in sorted(relics_root.rglob("*.java")):
        class_name = path.stem
        if class_name == "AbstractRelic" or class_name.startswith("Test"):
            continue
        if class_name.startswith("DEPRECATED"):
            continue
        if class_name == "DerpRock":
            continue
        items.append(
            {
                "java_id": class_name,
                "java_class": class_name,
                "java_source": _java_source(path, java_root),
            }
        )
    return {"source_missing": False, "items": items}


def parse_java_events(java_root: Path) -> Dict[str, Any]:
    events_root = java_root / "events"
    if not events_root.exists():
        return {"source_missing": True, "items": []}

    items: List[Dict[str, str]] = []
    for path in sorted(events_root.rglob("*.java")):
        class_name = path.stem
        if class_name in EVENT_EXCLUDED_CLASSES:
            continue

        items.append(
            {
                "java_id": class_name,
                "java_class": class_name,
                "java_source": _java_source(path, java_root),
            }
        )
    return {"source_missing": False, "items": items}


def parse_java_potions(java_root: Path) -> Dict[str, Any]:
    potions_root = java_root / "potions"
    if not potions_root.exists():
        return {"source_missing": True, "items": []}

    items: List[Dict[str, str]] = []
    for path in sorted(potions_root.rglob("*.java")):
        class_name = path.stem
        if class_name in POTION_EXCLUDED_CLASSES:
            continue
        items.append(
            {
                "java_id": class_name,
                "java_class": class_name,
                "java_source": _java_source(path, java_root),
            }
        )
    return {"source_missing": False, "items": items}


def parse_java_powers(java_root: Path) -> Dict[str, Any]:
    powers_root = java_root / "powers"
    if not powers_root.exists():
        return {"source_missing": True, "items": []}

    items: List[Dict[str, Any]] = []
    for path in sorted(powers_root.rglob("*.java")):
        rel = _java_source(path, java_root)
        if "/deprecated/" in rel:
            continue
        class_name = path.stem
        if class_name in POWER_EXCLUDED_CLASSES:
            continue

        text = path.read_text(encoding="utf-8", errors="ignore")
        hooks: List[str] = []
        for hook_name, method_pattern in POWER_HOOK_METHODS.items():
            pattern = rf"public\s+[^\n{{]*{method_pattern}"
            if re.search(pattern, text):
                hooks.append(hook_name)

        id_match = re.search(r"POWER_ID\s*=\s*\"([^\"]+)\"", text)
        if not id_match:
            id_match = re.search(r"this\.ID\s*=\s*\"([^\"]+)\"", text)
        java_id = id_match.group(1) if id_match else class_name

        items.append(
            {
                "java_id": java_id,
                "java_class": class_name,
                "java_hooks_overridden": sorted(hooks),
                "java_source": rel,
            }
        )
    return {"source_missing": False, "items": items}


def parse_java_inventory(java_root: Path) -> Dict[str, Any]:
    return {
        "cards": parse_java_cards(java_root),
        "relics": parse_java_relics(java_root),
        "events": parse_java_events(java_root),
        "powers": parse_java_powers(java_root),
        "potions": parse_java_potions(java_root),
    }


def parse_python_inventory(modules: Dict[str, Any]) -> Dict[str, Any]:
    event_mod = modules["events"]
    event_ids = sorted(
        set(event_mod.ACT1_EVENTS)
        | set(event_mod.ACT2_EVENTS)
        | set(event_mod.ACT3_EVENTS)
        | set(event_mod.SHRINE_EVENTS)
        | set(event_mod.SPECIAL_ONE_TIME_EVENTS)
    )

    return {
        "cards": {
            "ids": sorted(modules["cards"].ALL_CARDS.keys()),
            "aliases": dict(sorted(getattr(modules["cards"], "CARD_ID_ALIASES", {}).items())),
        },
        "relics": {
            "ids": sorted(modules["relics"].ALL_RELICS.keys()),
        },
        "events": {
            "ids": event_ids,
            "choice_generators": sorted(event_mod.EVENT_CHOICE_GENERATORS.keys()),
            "handlers": sorted(event_mod.EVENT_HANDLERS.keys()),
            "aliases": dict(sorted(event_mod.EVENT_ID_ALIASES.items())),
        },
        "powers": {
            "ids": sorted(modules["powers"].POWER_DATA.keys()),
            "aliases": dict(sorted(modules["powers"].POWER_ID_ALIASES.items())),
        },
        "potions": {
            "ids": sorted(modules["potions"].ALL_POTIONS.keys()),
        },
    }


def map_domain_ids(
    *,
    java_rows: Sequence[Dict[str, Any]],
    python_ids: Sequence[str],
    java_id_key: str = "java_id",
    resolver: Optional[Callable[[str], Optional[str]]] = None,
    extra_candidates: Optional[Callable[[str], Sequence[str]]] = None,
    strip_power_suffix: bool = False,
) -> List[Dict[str, Any]]:
    py_set = set(python_ids)
    py_norm: Dict[str, List[str]] = defaultdict(list)
    for py_id in python_ids:
        py_norm[normalize_id(py_id, strip_power_suffix=strip_power_suffix)].append(py_id)

    rows: List[Dict[str, Any]] = []
    for src in sorted(java_rows, key=lambda row: str(row.get(java_id_key, ""))):
        java_id = str(src.get(java_id_key, ""))
        python_id: Optional[str] = None
        via = ""
        status = "missing"

        if java_id in py_set:
            python_id = java_id
            status = "exact"
            via = "exact"
        else:
            candidates = [java_id]
            if extra_candidates:
                candidates.extend(extra_candidates(java_id))

            for candidate in candidates:
                if candidate in py_set:
                    python_id = candidate
                    via = f"candidate:{candidate}"
                    break

                if resolver:
                    resolved = resolver(candidate)
                    if resolved and resolved in py_set:
                        python_id = resolved
                        via = f"resolver:{candidate}"
                        break

            if python_id is None:
                for candidate in candidates:
                    token = normalize_id(candidate, strip_power_suffix=strip_power_suffix)
                    if token in py_norm:
                        python_id = choose_canonical(py_norm[token])
                        via = f"normalized:{candidate}"
                        break

            if python_id is not None:
                java_norm = normalize_id(java_id, strip_power_suffix=strip_power_suffix)
                py_norm_id = normalize_id(python_id, strip_power_suffix=strip_power_suffix)
                status = "exact" if java_norm == py_norm_id and via == "exact" else "alias"

        rows.append(
            {
                **src,
                "python_id": python_id,
                "status": status,
                "mapping_via": via,
            }
        )

    return rows


def _event_candidates(java_id: str) -> Sequence[str]:
    stripped = re.sub(r"Event$", "", java_id)
    return [
        stripped,
        split_camel(stripped),
        stripped.replace("_", " "),
    ]


def _power_candidates(java_id: str) -> Sequence[str]:
    return [
        java_id,
        re.sub(r"Power$", "", java_id),
        split_camel(java_id),
    ]


def _card_candidates(java_id: str) -> Sequence[str]:
    return [
        java_id,
        java_id.replace(" ", ""),
        split_camel(java_id).replace(" ", ""),
    ]


def _relic_candidates(java_id: str) -> Sequence[str]:
    candidates = [java_id, split_camel(java_id)]
    if java_id == "ChampionsBelt":
        candidates.append("Champion Belt")
    return candidates


def summarize_domain(rows: Sequence[Dict[str, Any]], *, source_missing: bool) -> Dict[str, Any]:
    counter = Counter(row["status"] for row in rows)
    missing = [row["java_id"] for row in rows if row["status"] == "missing"]
    alias = [
        {
            "java_id": row["java_id"],
            "python_id": row["python_id"],
            "via": row["mapping_via"],
        }
        for row in rows
        if row["status"] == "alias"
    ]
    return {
        "source_missing": source_missing,
        "java_count": len(rows),
        "exact": counter.get("exact", 0),
        "alias": counter.get("alias", 0),
        "missing": counter.get("missing", 0),
        "missing_java_ids": sorted(missing),
        "alias_examples": sorted(alias, key=lambda r: (r["java_id"], str(r["python_id"]))),
    }


def parse_registry_power_hooks(registry_path: Path) -> Dict[str, Any]:
    tree = ast.parse(registry_path.read_text(encoding="utf-8"))
    hooks: Dict[str, List[str]] = defaultdict(list)

    for node in ast.walk(tree):
        if not isinstance(node, ast.FunctionDef):
            continue

        for deco in node.decorator_list:
            if not (
                isinstance(deco, ast.Call)
                and isinstance(deco.func, ast.Name)
                and deco.func.id == "power_trigger"
                and deco.args
                and isinstance(deco.args[0], ast.Constant)
                and isinstance(deco.args[0].value, str)
            ):
                continue

            hook_name = deco.args[0].value
            power_name = None
            for kw in deco.keywords:
                if kw.arg == "power" and isinstance(kw.value, ast.Constant) and isinstance(kw.value.value, str):
                    power_name = kw.value.value
                    break
            if power_name:
                hooks[hook_name].append(power_name)

    return {
        "hooks": {key: sorted(values) for key, values in sorted(hooks.items())},
        "hook_names": sorted(hooks.keys()),
    }


def parse_runtime_power_dispatch_hooks(paths: Sequence[Path]) -> List[str]:
    found: set[str] = set()
    pattern = re.compile(r'execute_power_triggers\("([^\"]+)"')
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for hook in pattern.findall(text):
            found.add(hook)
    return sorted(found)


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_java_inventory_md(path: Path, java_inventory: Dict[str, Any], python_inventory: Dict[str, Any], diff: Dict[str, Any], java_root: Path) -> None:
    lines = [
        "# Java Inventory",
        "",
        "Repository-local Java source root used for this audit:",
        f"- `{java_root}`",
        "",
        "This file is script-generated by `scripts/generate_parity_manifests.py`.",
        "",
        "## Snapshot counts",
        "",
        "| domain | java count | python count | exact | alias | missing | notes |",
        "|---|---:|---:|---:|---:|---:|---|",
    ]

    for domain in ("cards", "relics", "events", "powers", "potions"):
        java_entry = java_inventory[domain]
        py_count = len(python_inventory[domain]["ids"]) if domain in python_inventory else 0
        summary = diff[domain]["summary"]
        note = "local Java source missing" if java_entry["source_missing"] else "-"
        lines.append(
            f"| {domain} | {summary['java_count']} | {py_count} | {summary['exact']} | {summary['alias']} | {summary['missing']} | {note} |"
        )

    lines.extend([
        "",
        "## Missing rows by domain",
        "",
    ])

    for domain in ("cards", "relics", "events", "powers", "potions"):
        summary = diff[domain]["summary"]
        lines.append(f"### {domain}")
        if summary["source_missing"]:
            lines.append("- Java source unavailable in local decompile snapshot.")
        elif summary["missing_java_ids"]:
            for item in summary["missing_java_ids"]:
                lines.append(f"- `{item}`")
        else:
            lines.append("- None")
        lines.append("")

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_python_coverage_md(path: Path, python_inventory: Dict[str, Any], diff: Dict[str, Any], hook_report: Dict[str, Any]) -> None:
    lines = [
        "# Python Coverage Inventory",
        "",
        "This file is script-generated by `scripts/generate_parity_manifests.py`.",
        "",
        "## Snapshot",
        "",
        "| domain | python inventory count | java exact | java alias | java missing |",
        "|---|---:|---:|---:|---:|",
    ]

    for domain in ("cards", "relics", "events", "powers", "potions"):
        py_count = len(python_inventory[domain]["ids"])
        summary = diff[domain]["summary"]
        lines.append(
            f"| {domain} | {py_count} | {summary['exact']} | {summary['alias']} | {summary['missing']} |"
        )

    lines.extend([
        "",
        "## Event infrastructure",
        "",
        f"- Event definitions: `{len(python_inventory['events']['ids'])}`",
        f"- Event choice generators: `{len(python_inventory['events']['choice_generators'])}`",
        f"- Event handlers: `{len(python_inventory['events']['handlers'])}`",
        "",
        "## Power hook dispatch coverage",
        "",
        f"- Registry hooks: `{len(hook_report['registry_hook_names'])}`",
        f"- Runtime-dispatched hooks: `{len(hook_report['runtime_hook_names'])}`",
        f"- Registered but not dispatched: `{len(hook_report['registered_not_dispatched'])}`",
    ])

    if hook_report["registered_not_dispatched"]:
        lines.append("")
        for hook in hook_report["registered_not_dispatched"]:
            lines.append(f"- `{hook}`")

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--java-root",
        type=Path,
        default=Path("/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl"),
        help="Path to decompiled Java cardcrawl root",
    )
    parser.add_argument(
        "--traceability-dir",
        type=Path,
        default=repo_root / "docs/audits/2026-02-22-full-game-parity/traceability",
        help="Output directory for inventory manifests",
    )
    args = parser.parse_args()

    modules = load_python_modules(repo_root)
    java_inventory = parse_java_inventory(args.java_root)
    python_inventory = parse_python_inventory(modules)

    cards_rows = map_domain_ids(
        java_rows=java_inventory["cards"]["items"],
        python_ids=python_inventory["cards"]["ids"],
        resolver=lambda value: getattr(modules["cards"], "resolve_card_id")(value),
        extra_candidates=_card_candidates,
    )
    relics_rows = map_domain_ids(
        java_rows=java_inventory["relics"]["items"],
        python_ids=python_inventory["relics"]["ids"],
        resolver=lambda value: getattr(modules["relics"], "resolve_relic_id")(value),
        extra_candidates=_relic_candidates,
    )

    event_aliases = python_inventory["events"]["aliases"]

    def event_resolver(value: str) -> Optional[str]:
        lower = value.lower()
        if lower in event_aliases:
            return event_aliases[lower]
        return None

    events_rows = map_domain_ids(
        java_rows=java_inventory["events"]["items"],
        python_ids=python_inventory["events"]["ids"],
        resolver=event_resolver,
        extra_candidates=_event_candidates,
    )
    powers_rows = map_domain_ids(
        java_rows=java_inventory["powers"]["items"],
        python_ids=python_inventory["powers"]["ids"],
        resolver=lambda value: getattr(modules["powers"], "resolve_power_id")(value),
        extra_candidates=_power_candidates,
        strip_power_suffix=True,
    )
    potions_rows = map_domain_ids(
        java_rows=java_inventory["potions"]["items"],
        python_ids=python_inventory["potions"]["ids"],
        extra_candidates=lambda value: [value, split_camel(value)],
    )

    parity_diff = {
        "cards": {
            "summary": summarize_domain(
                cards_rows,
                source_missing=java_inventory["cards"]["source_missing"],
            ),
            "rows": cards_rows,
        },
        "relics": {
            "summary": summarize_domain(
                relics_rows,
                source_missing=java_inventory["relics"]["source_missing"],
            ),
            "rows": relics_rows,
        },
        "events": {
            "summary": summarize_domain(
                events_rows,
                source_missing=java_inventory["events"]["source_missing"],
            ),
            "rows": events_rows,
        },
        "powers": {
            "summary": summarize_domain(
                powers_rows,
                source_missing=java_inventory["powers"]["source_missing"],
            ),
            "rows": powers_rows,
        },
        "potions": {
            "summary": summarize_domain(
                potions_rows,
                source_missing=java_inventory["potions"]["source_missing"],
            ),
            "rows": potions_rows,
        },
    }

    registry = parse_registry_power_hooks(repo_root / "packages/engine/registry/powers.py")
    runtime_hooks = parse_runtime_power_dispatch_hooks(
        [
            repo_root / "packages/engine/combat_engine.py",
            repo_root / "packages/engine/handlers/combat.py",
        ]
    )
    hook_report = {
        "registry_hook_names": registry["hook_names"],
        "runtime_hook_names": runtime_hooks,
        "registered_not_dispatched": sorted(set(registry["hook_names"]) - set(runtime_hooks)),
        "dispatched_not_registered": sorted(set(runtime_hooks) - set(registry["hook_names"])),
        "hooks_by_name": registry["hooks"],
    }

    trace_dir = args.traceability_dir
    write_json(trace_dir / "java-inventory.json", java_inventory)
    write_json(trace_dir / "python-inventory.json", python_inventory)
    write_json(trace_dir / "parity-diff.json", parity_diff)
    write_json(trace_dir / "power-hook-coverage.json", hook_report)

    write_java_inventory_md(
        trace_dir / "java-inventory.md",
        java_inventory,
        python_inventory,
        parity_diff,
        args.java_root,
    )
    write_python_coverage_md(
        trace_dir / "python-coverage.md",
        python_inventory,
        parity_diff,
        hook_report,
    )

    print("Generated parity manifests:")
    print(f"- {trace_dir / 'java-inventory.json'}")
    print(f"- {trace_dir / 'python-inventory.json'}")
    print(f"- {trace_dir / 'parity-diff.json'}")
    print(f"- {trace_dir / 'power-hook-coverage.json'}")
    print(f"- {trace_dir / 'java-inventory.md'}")
    print(f"- {trace_dir / 'python-coverage.md'}")

    for domain in ("cards", "relics", "events", "powers", "potions"):
        summary = parity_diff[domain]["summary"]
        print(
            f"{domain}: java={summary['java_count']} exact={summary['exact']} alias={summary['alias']} missing={summary['missing']}"
        )

    print(
        "power hooks: "
        f"registry={len(hook_report['registry_hook_names'])} "
        f"runtime={len(hook_report['runtime_hook_names'])} "
        f"undispatched={len(hook_report['registered_not_dispatched'])}"
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
