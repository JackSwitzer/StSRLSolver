# reference/

`extracted/` (gitignored, regenerable): agent-ready distillation of the
decompiled Java — data tables (cards/monsters/relics/potions.json) and
per-item verbatim logic bodies (`methods/<kind>/<Class>.java`, `methods/index.json`).

Regenerate with `scripts/extract.sh` (also refreshes `docs/goal/ledger.json`,
preserving verification statuses). Requires `decompiled/java-src` —
regenerate that first with `scripts/decompile_java.sh` if missing.
