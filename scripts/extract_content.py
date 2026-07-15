#!/usr/bin/env python3
"""Distill decompiled/java-src into structured, agent-ready artifacts.

Outputs (all under reference/extracted/, gitignored like decompiled/):
  cards.json / monsters.json / relics.json / potions.json  -- data tables
  methods/<kind>/<Id>.java                                 -- verbatim logic bodies
  methods/index.json, _manifest.json

Optionally (--ledger PATH) seeds or refreshes the committed verification ledger:
new content items get status "unverified" while the completed statuses of the
existing 667 rows are preserved by ID. See docs/goal/GOAL.md.

Run via scripts/extract.sh (uv). Stdlib only; regex heuristics tuned to CFR
0.152 output — parse failures are recorded, never fatal.
"""

import argparse
import datetime
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SRC = REPO / "decompiled/java-src/com/megacrit/cardcrawl"
OUT = REPO / "reference/extracted"

CARD_DIRS = ["cards/red", "cards/green", "cards/blue", "cards/purple",
             "cards/colorless", "cards/curses", "cards/status", "cards/tempCards"]
MONSTER_DIRS = ["monsters/exordium", "monsters/city", "monsters/beyond", "monsters/ending"]

ID_RE = re.compile(r'public static final String ID = "([^"]+)";')


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def rel(path: Path) -> str:
    return str(path.relative_to(REPO))


def find_method(text: str, name_pattern: str):
    """Return verbatim body of first method whose name matches, via brace counting."""
    sig = re.compile(
        r'^\s*(?:public|protected|private)?\s*(?:static\s+)?[\w<>\[\], .]*?\b('
        + name_pattern + r')\s*\([^;{]*\)\s*\{', re.MULTILINE)
    m = sig.search(text)
    if not m:
        return None, None
    start = m.start()
    depth = 0
    for i in range(text.index("{", m.start()), len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return m.group(1), text[start:i + 1]
    return m.group(1), None


def extract_methods(text: str, patterns):
    out = {}
    for pat in patterns:
        pos = 0
        remaining = text
        while True:
            name, body = find_method(remaining, pat)
            if not name or not body:
                break
            out.setdefault(name, body)
            idx = remaining.find(body)
            remaining = remaining[idx + len(body):] if idx >= 0 else ""
            if len(out) > 40:
                break
            pos += 1
            if pos > 40:
                break
    return out


def first_int(args: str):
    for tok in re.findall(r'(?<![\w."])-?\d+(?![\w."])', args):
        return int(tok)
    return None


def parse_card(path: Path):
    text = read(path)
    cls = path.stem
    rec = {"id": None, "class": cls, "file": rel(path)}
    m = ID_RE.search(text)
    rec["id"] = m.group(1) if m else cls
    sup = re.search(r'super\((.*?)\);', text, re.DOTALL)
    if sup:
        args = sup.group(1)
        rec["cost"] = first_int(args)
        for field, pat in [("type", r'CardType\.(\w+)'), ("color", r'CardColor\.(\w+)'),
                           ("rarity", r'CardRarity\.(\w+)'), ("target", r'CardTarget\.(\w+)')]:
            em = re.search(pat, args)
            rec[field] = em.group(1) if em else None
    for field, pat in [("base_damage", r'this\.baseDamage = (-?\d+)'),
                       ("base_block", r'this\.baseBlock = (-?\d+)'),
                       ("base_magic", r'this\.baseMagicNumber = (-?\d+)')]:
        em = re.search(pat, text)
        rec[field] = int(em.group(1)) if em else None
    for flag, pat in [("exhausts", r'this\.exhaust = true'), ("ethereal", r'this\.isEthereal = true'),
                      ("innate", r'this\.isInnate = true'), ("retain", r'this\.selfRetain = true')]:
        rec[flag] = bool(re.search(pat, text))
    _, upg = find_method(text, "upgrade")
    if upg:
        for field, pat in [("upgrade_damage", r'upgradeDamage\((-?\d+)\)'),
                           ("upgrade_block", r'upgradeBlock\((-?\d+)\)'),
                           ("upgrade_magic", r'upgradeMagicNumber\((-?\d+)\)'),
                           ("upgrade_cost", r'upgradeBaseCost\((-?\d+)\)')]:
            em = re.search(pat, upg)
            rec[field] = int(em.group(1)) if em else None
    methods = extract_methods(text, [r'use', r'upgrade', r'canUse', r'trigger[A-Z]\w*',
                                     re.escape(cls)])
    return rec, methods


def parse_monster(path: Path):
    text = read(path)
    cls = path.stem
    rec = {"id": None, "class": cls, "file": rel(path)}
    m = ID_RE.search(text)
    rec["id"] = m.group(1) if m else cls
    rec["hp_settings"] = re.findall(r'this\.setHp\((\d+)(?:,\s*(\d+))?\)', text)
    rec["ascension_gates"] = sorted({int(a) for a in
                                     re.findall(r'ascensionLevel >= (\d+)', text)})
    rec["move_constants"] = {name: int(v) for name, v in
                             re.findall(r'private static final byte (\w+) = (-?\d+);', text)}
    rec["damage_values"] = [int(d) for d in re.findall(r'new DamageInfo\([^,]+,\s*(\d+)', text)]
    rec["has_getMove"] = "void getMove(" in text
    rec["uses_aiRng"] = "aiRng" in text
    rec["first_move_forced"] = "firstMove" in text
    methods = extract_methods(text, [r'getMove', r'takeTurn', r'usePreBattleAction',
                                     r'changeState', r'damage', re.escape(cls)])
    return rec, methods


def parse_relic(path: Path):
    text = read(path)
    cls = path.stem
    rec = {"id": None, "class": cls, "file": rel(path)}
    m = ID_RE.search(text)
    if not m:
        m = re.search(r'super\("([^"]+)"', text)
    rec["id"] = m.group(1) if m else cls
    em = re.search(r'RelicTier\.(\w+)', text)
    rec["tier"] = em.group(1) if em else None
    em = re.search(r'this\.counter = (-?\d+)', text)
    rec["counter_init"] = int(em.group(1)) if em else None
    methods = extract_methods(text, [r'(?:on|at|when)[A-Z]\w*', r'getUpdatedDescription',
                                     re.escape(cls)])
    return rec, methods


def parse_potion(path: Path):
    text = read(path)
    cls = path.stem
    rec = {"id": None, "class": cls, "file": rel(path)}
    m = ID_RE.search(text)
    if not m:
        m = re.search(r'super\([^,]*,\s*"([^"]+)"', text)
    rec["id"] = m.group(1) if m else cls
    em = re.search(r'PotionRarity\.(\w+)', text)
    rec["rarity"] = em.group(1) if em else None
    em = re.search(r'getPotency[^{]*\{[^}]*?return (\d+);', text, re.DOTALL)
    rec["potency"] = int(em.group(1)) if em else None
    methods = extract_methods(text, [r'use', r'getPotency', re.escape(cls)])
    return rec, methods


def walk(dirs_or_dir, parser, kind, failures):
    records, method_index = [], {}
    dirs = dirs_or_dir if isinstance(dirs_or_dir, list) else [dirs_or_dir]
    for d in dirs:
        base = SRC / d
        if not base.is_dir():
            continue
        for path in sorted(base.glob("*.java")):
            if path.stem.startswith("Abstract") or path.stem.endswith("Strings"):
                continue
            try:
                rec, methods = parser(path)
            except Exception as exc:  # record, never die
                failures.append({"file": rel(path), "error": str(exc)})
                continue
            rec["pool_dir"] = d
            records.append(rec)
            if methods:
                mdir = OUT / "methods" / kind
                mdir.mkdir(parents=True, exist_ok=True)
                mfile = mdir / f"{rec['class']}.java"
                header = f"// extracted from {rec['file']} — ground truth for {kind}/{rec['id']}\n"
                mfile.write_text(header + "\n\n".join(methods.values()), encoding="utf-8")
                method_index[f"{kind}/{rec['id']}"] = {
                    "file": rel(mfile), "methods": sorted(methods.keys()),
                    "source": rec["file"]}
    return records, method_index


BASE_CLASSES = {
    "monsters/AbstractMonster.java": [r'rollMove', r'setMove', r'lastMove', r'lastTwoMoves',
                                      r'createIntent', r'getIntentDmg'],
    "cards/AbstractCard.java": [r'calculateCardDamage', r'applyPowers', r'upgradeDamage',
                                r'upgradeBlock', r'upgradeMagicNumber', r'upgradeBaseCost'],
    "relics/AbstractRelic.java": [r'flash', r'counterUp', r'setCounter'],
    "characters/AbstractPlayer.java": [r'useCard', r'draw'],
}


def extract_base_classes():
    """Shared superclass logic every per-item verification needs (e.g.
    AbstractMonster.rollMove is where the one-aiRng-tick-per-turn contract
    lives). Emitted to methods/base/."""
    out = {}
    for rel_path, patterns in BASE_CLASSES.items():
        path = SRC / rel_path
        if not path.is_file():
            continue
        methods = extract_methods(read(path), patterns)
        if not methods:
            continue
        bdir = OUT / "methods" / "base"
        bdir.mkdir(parents=True, exist_ok=True)
        bfile = bdir / path.name
        header = f"// extracted from {rel(path)} — shared base-class ground truth\n"
        bfile.write_text(header + "\n\n".join(methods.values()), encoding="utf-8")
        out[f"base/{path.stem}"] = {"file": rel(bfile), "methods": sorted(methods.keys()),
                                    "source": rel(path)}
    return out


def flip_ledger_row(ledger_path: Path, row_id: str, status: str, by: str, dev: str):
    """Flip one ledger row in place, recomputing status_counts and keeping the
    indent=1 formatting stable. Bash-facing entry point: scripts/ledger.sh."""
    data = json.loads(ledger_path.read_text())
    match = [r for r in data["rows"] if r["id"] == row_id]
    if not match:
        near = [r["id"] for r in data["rows"]
                if row_id.split("/")[-1].lower() in r["id"].lower()][:5]
        sys.exit(f"no ledger row '{row_id}'" + (f"; close matches: {near}" if near else ""))
    row = match[0]
    if status == "quarantined" and not (dev or "").startswith("DEV-"):
        sys.exit("quarantined requires --dev DEV-NNN (see GOAL.md Edge-Case Policy)")
    row["status"] = status
    row["verified_by"] = by
    row["dev"] = dev
    counts = {}
    for r in data["rows"]:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    data["status_counts"] = counts
    data["updated"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    ledger_path.write_text(json.dumps(data, indent=1) + "\n")
    print(f"{row_id} -> {status} (by {by}) counts={counts}")


def seed_ledger(ledger_path: Path, tables):
    existing = {}
    existing_rows = None
    if ledger_path.exists():
        existing_rows = json.loads(ledger_path.read_text())["rows"]
        for row in existing_rows:
            existing[row["id"]] = row
    rows = []
    for kind, records in tables.items():
        for rec in records:
            rid = f"{kind}/{rec['id']}"
            if rid in existing:
                rows.append(existing[rid])
                continue
            rows.append({"id": rid, "kind": kind, "class": rec["class"],
                         "java_ref": rec["file"],
                         "methods_ref": f"reference/extracted/methods/{kind}/{rec['class']}.java",
                         "status": "unverified", "verified_by": None, "dev": None})
    rows.sort(key=lambda r: r["id"])
    counts = {}
    for row in rows:
        counts[row["status"]] = counts.get(row["status"], 0) + 1
    if rows == existing_rows:
        # Nothing changed — don't rewrite (the timestamp bump would dirty the
        # committed ledger on every fresh-worktree scripts/extract.sh run).
        return counts, len(rows)
    ledger_path.write_text(json.dumps(
        {"v": 1, "updated": datetime.datetime.now(datetime.timezone.utc).isoformat(),
         "status_counts": counts, "rows": rows}, indent=1) + "\n")
    return counts, len(rows)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ledger", type=Path, default=None,
                    help="also seed/refresh the committed verification ledger (merge-preserving)")
    ap.add_argument("--flip", metavar="ROW_ID",
                    help="flip one ledger row and exit (use with --ledger, --by)")
    ap.add_argument("--status", default="verified",
                    choices=["verified", "unverified", "quarantined"])
    ap.add_argument("--by", default=None, help="verified_by value (branch/commit)")
    ap.add_argument("--dev", default=None, help="DEV-NNN register id (quarantine only)")
    args = ap.parse_args()

    if args.flip:
        if not args.ledger or not args.by:
            sys.exit("--flip requires --ledger and --by")
        flip_ledger_row(args.ledger, args.flip, args.status, args.by, args.dev)
        return

    if not SRC.is_dir():
        sys.exit(f"decompiled source missing at {SRC} — run scripts/decompile_java.sh first")
    OUT.mkdir(parents=True, exist_ok=True)

    failures = []
    index = {}
    tables = {}
    for kind, dirs, parser in [("card", CARD_DIRS, parse_card),
                               ("monster", MONSTER_DIRS, parse_monster),
                               ("relic", "relics", parse_relic),
                               ("potion", "potions", parse_potion)]:
        records, mindex = walk(dirs, parser, kind, failures)
        tables[kind] = records
        index.update(mindex)
        (OUT / f"{kind}s.json").write_text(json.dumps(records, indent=1) + "\n")

    index.update(extract_base_classes())
    (OUT / "methods" / "index.json").write_text(json.dumps(index, indent=1) + "\n")
    summary = {"generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
               "counts": {k: len(v) for k, v in tables.items()},
               "method_files": len(index), "parse_failures": failures}
    (OUT / "_manifest.json").write_text(json.dumps(summary, indent=1) + "\n")

    print(json.dumps({k: len(v) for k, v in tables.items()}, indent=None),
          f"method_files={len(index)}", f"failures={len(failures)}")

    if args.ledger:
        counts, total = seed_ledger(args.ledger, tables)
        print(f"ledger: {total} rows -> {args.ledger} status_counts={counts}")


if __name__ == "__main__":
    main()
