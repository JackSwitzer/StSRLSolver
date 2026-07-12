#!/usr/bin/env bash
# Preflight-validate every @SpirePatch in packages/harness-java against the
# decompiled game source, so ModTheSpire patcher failures are caught at build
# time instead of one-by-one at game launch. Rules learned the hard way:
#   - MTS cannot Prefix/Postfix ABSTRACT methods (ParamInfo NPE)
#   - static targets cannot take __instance
#   - Postfix's __result must be the FIRST parameter
#   - target class/method must exist with that name
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
if [[ ! -e "$REPO/decompiled/java-src" ]]; then
  MAIN_CHECKOUT="$(cd "$(git rev-parse --git-common-dir)/.." && pwd)"
  if [[ "$MAIN_CHECKOUT" != "$REPO" && -d "$MAIN_CHECKOUT/decompiled/java-src" ]]; then
    ln -sfn "$MAIN_CHECKOUT/decompiled" "$REPO/decompiled"
  fi
fi
python3 - "$REPO" << 'PY'
import re, sys
from pathlib import Path

repo = Path(sys.argv[1])
src_root = repo / "decompiled/java-src"
errors, checked = [], 0

def resolve(clz, imports):
    if "." in clz:
        return clz
    return imports.get(clz)

for jf in (repo / "packages/harness-java/src/main/java").rglob("*.java"):
    text = jf.read_text(errors="replace")
    imports = {}
    for m in re.finditer(r'^import\s+([\w.]+)\.(\w+);', text, re.M):
        imports[m.group(2)] = f"{m.group(1)}.{m.group(2)}"
    for m in re.finditer(
            r'@SpirePatch\(\s*clz\s*=\s*([\w.]+)\.class\s*,\s*method\s*=\s*("(\w+)"|SpirePatch\.CONSTRUCTOR)[\s\S]*?class\s+(\w+)\s*\{([\s\S]*?)\n    \}',
            text):
        clz_token, _, method, patch_cls, body = m.groups()
        checked += 1
        fqn = resolve(clz_token, imports)
        loc = f"{jf.name}:{patch_cls}"
        if not fqn:
            errors.append(f"{loc}: cannot resolve class {clz_token} (missing import?)")
            continue
        tf = src_root / (fqn.replace(".", "/") + ".java")
        if not tf.is_file():
            errors.append(f"{loc}: no decompiled source for {fqn}")
            continue
        tsrc = tf.read_text(errors="replace")
        if method is None:  # CONSTRUCTOR
            if not re.search(r'(public|protected)\s+' + fqn.split(".")[-1] + r'\s*\(', tsrc):
                errors.append(f"{loc}: no visible constructor on {fqn}")
            continue
        decls = re.findall(
            r'^\s*((?:public|protected|private)?[\w\s]*?)\b[\w<>\[\], ]+\s+' + method + r'\s*\(([^)]*)\)\s*[{;]',
            tsrc, re.M)
        if not decls:
            errors.append(f"{loc}: method {fqn}.{method} not found in decompiled source")
            continue
        mods = " ".join(d[0] for d in decls)
        is_abstract = "abstract" in mods
        is_static = "static" in mods
        for pm in re.finditer(r'(?:public\s+static\s+\w[\w<>\[\], .]*)\s+(Prefix|Postfix|Insert)\s*\(([^)]*)\)', body):
            kind, params = pm.groups()
            plist = [p.strip() for p in params.split(",") if p.strip()]
            names = [p.split()[-1] for p in plist]
            if is_abstract and (plist or kind != "Postfix"):
                errors.append(f"{loc}: {fqn}.{method} is ABSTRACT — MTS cannot patch it; hook a concrete caller")
            if is_static and "__instance" in names:
                errors.append(f"{loc}: {fqn}.{method} is static — remove __instance param")
            if kind == "Postfix" and "__result" in names and names[0] != "__result":
                errors.append(f"{loc}: Postfix __result must be the FIRST parameter")

print(f"[check_patches] {checked} @SpirePatch targets checked")
if errors:
    print("\n".join("ERROR " + e for e in errors))
    sys.exit(1)
print("[check_patches] all patches statically valid")
PY
