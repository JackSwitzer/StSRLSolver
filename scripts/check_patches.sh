#!/usr/bin/env bash
# check_patches.sh — pre-flight validation of TraceLab SpirePatch signatures
# against ModTheSpire 3.30.3 parameter rules, WITHOUT launching the game.
#
# The game jar ships with no LocalVariableTable or MethodParameters, so MTS
# resolves each non-special Prefix/Postfix parameter POSITIONALLY BY TYPE
# against the target method. Rules enforced here, up front:
#   - special names (__instance, __args, ___field) are always legal;
#   - any other double-underscore name (e.g. __result) is NOT a special and
#     fails at inject time with "Illegal patch parameter: Cannot determine
#     name" — the exact crash this script exists to prevent;
#   - remaining params must match the target method's parameter types in
#     order (erased simple names), and the target method must exist.
#
# Usage: scripts/check_patches.sh    (record_play.sh runs it before launch)
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_DIR="${REPO}/packages/harness-java/src/main/java/tracelab/patches"
GAME_JAR="$HOME/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/desktop-1.0.jar"
JAVAP="${TRACELAB_JAVA_HOME:-/opt/homebrew/opt/openjdk@11}/bin/javap"

[[ -f "${GAME_JAR}" ]] || { echo "game jar missing: ${GAME_JAR}" >&2; exit 2; }
fail=0
checked=0

# Target method parameter types (erased simple names, one per line, in
# order) parsed from the JVM descriptor via javap -s. Empty output + status 3
# means the method was not found on the class at all.
target_param_types() { # fqcn method
  local desc
  desc=$("${JAVAP}" -p -s -classpath "${GAME_JAR}" "$1" 2>/dev/null | awk -v m="$2" '
    found { print; exit }
    $0 ~ ("[ .]" m "\\(") || (m == "<init>" && $0 ~ /^ *(public|protected|private)? *[A-Za-z0-9.$]+\(/ && $0 !~ / (static|void) /) { found=1 }
  ' | grep -o 'descriptor: .*' | sed 's/descriptor: //') || true
  if [[ -z "${desc}" ]]; then
    return 3
  fi
  echo "${desc}" | sed 's/^(//; s/).*//' | awk '{
    s = $0
    while (length(s) > 0) {
      arr = ""
      while (substr(s, 1, 1) == "[") { arr = arr "[]"; s = substr(s, 2) }
      c = substr(s, 1, 1)
      if (c == "L") {
        i = index(s, ";")
        t = substr(s, 2, i - 2); s = substr(s, i + 1)
        n = split(t, parts, "/"); t = parts[n]
        sub(/.*\$/, "", t)
      } else {
        s = substr(s, 2)
        t = (c=="Z") ? "boolean" : (c=="I") ? "int" : (c=="J") ? "long" : \
            (c=="F") ? "float" : (c=="D") ? "double" : (c=="B") ? "byte" : \
            (c=="S") ? "short" : (c=="C") ? "char" : "?"
      }
      print t arr
    }
  }'
}

# Resolve a simple class name to FQCN via the source file's imports.
resolve_fqcn() { # srcfile SimpleName
  local imp
  imp=$(grep -E "^import .*[.]$2;" "$1" | head -1 | sed 's/^import //; s/;//')
  if [[ -n "${imp}" ]]; then echo "${imp}"; else echo "$2"; fi
}

for src in "${SRC_DIR}"/*.java; do
  # Emit tuples: TARGET|method  then  HOOK|name|params  from the source.
  # awk tracks the most recent @SpirePatch annotation (possibly multi-line).
  while IFS='|' read -r kind a b; do
    case "${kind}" in
      TARGET)
        cur_cls="${a}"; cur_mth="${b}" ;;
      HOOK)
        [[ -n "${cur_cls:-}" ]] || continue
        hook_name="${a}"; params="${b}"
        checked=$((checked + 1))
        fqcn=$(resolve_fqcn "${src}" "${cur_cls}")
        [[ "${cur_mth}" == "SpirePatch.CONSTRUCTOR" ]] && cur_mth="<init>"
        if ! ttypes=$(target_param_types "${fqcn}" "${cur_mth}"); then
          echo "FAIL ${cur_cls}.${hook_name}: target method ${fqcn}.${cur_mth} not found in game jar" >&2
          fail=1; continue
        fi
        ti=0
        bad=0
        oldIFS="${IFS}"; IFS=','
        for p in ${params}; do
          IFS="${oldIFS}"
          pname=$(echo "${p}" | awk '{print $NF}')
          ptype=$(echo "${p}" | awk '{print $(NF-1)}' | sed 's/.*[.]//')
          [[ -n "${pname}" ]] || continue
          case "${pname}" in
            __instance|__args) continue ;;
            ___*) continue ;;
            __*)
              echo "FAIL ${cur_cls}.${hook_name}: '${pname}' is not an MTS special parameter name (only __instance/__args/___field)" >&2
              fail=1; bad=1; continue ;;
          esac
          ttype=$(echo "${ttypes}" | sed -n "$((ti + 1))p")
          if [[ -z "${ttype}" ]]; then
            echo "FAIL ${cur_cls}.${hook_name}: param '${ptype} ${pname}' exceeds ${fqcn}.${cur_mth} arity ($(echo "${ttypes}" | grep -c . ) params)" >&2
            fail=1; bad=1
          elif [[ "${ptype}" != "${ttype}" ]]; then
            echo "FAIL ${cur_cls}.${hook_name}: param ${ti} type '${ptype}' != target type '${ttype}' in ${fqcn}.${cur_mth}" >&2
            fail=1; bad=1
          fi
          ti=$((ti + 1))
          IFS=','
        done
        IFS="${oldIFS}"
        [[ "${bad}" == "0" ]] && echo "ok   ${cur_cls}.${hook_name} -> ${fqcn}.${cur_mth}" ;;
    esac
  done < <(awk '
    /@SpirePatch\(/ { ann = $0; while (ann !~ /\)/ && (getline line) > 0) ann = ann line }
    /@SpirePatch\(/ {
      cls = ann; sub(/.*clz *= */, "", cls); sub(/\.class.*/, "", cls)
      mth = ann
      if (mth ~ /SpirePatch\.CONSTRUCTOR/) { mth = "SpirePatch.CONSTRUCTOR" }
      else { sub(/.*method *= *"/, "", mth); sub(/".*/, "", mth) }
      print "TARGET|" cls "|" mth
      next
    }
    /static .*(Prefix|Postfix) *\(/ {
      sig = $0; while (sig !~ /\)/ && (getline line) > 0) sig = sig line
      name = (sig ~ /Prefix/) ? "Prefix" : "Postfix"
      par = sig; sub(/[^(]*\(/, "", par); sub(/\).*/, "", par)
      gsub(/\t/, " ", par)
      print "HOOK|" name "|" par
    }
  ' "${src}")
done

echo "checked ${checked} hook(s)"
if [[ "${fail}" != "0" ]]; then
  echo "check_patches: FAILURES — fix before launching the game" >&2
  exit 1
fi
echo "check_patches: all patch signatures conform to MTS parameter rules"
