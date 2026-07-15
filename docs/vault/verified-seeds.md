# Verified trace seeds

This inventory lists only seeds represented by committed Java goldens.
“Verified” means the checked-in JSONL records were emitted by TraceLab for the
listed script; it does not mean a full run, a win, or full engine parity.

| Seed | Character | Ascension | Script | Golden | Scope |
|---:|---|---:|---|---|---|
| 57554006466 | WATCHER | 0 | data/traces/scripts/smoke-neow-floor1.json | data/traces/java/smoke-neow-floor1.jsonl | Neow choice, first path, Jaw Worm entry, three end turns |

The current corpus contains one script and one seven-line golden: a header,
five action records, and an end marker. The action path stops during the first
combat.

The detailed, source-bound spot checks for this seed are in
docs/vault/seed-WATCHER-57554006466-full-prediction.md. The historical filename
is retained because canonical goal documents link to it; its contents are now
limited to the committed smoke trace.

## Adding a seed

1. Add a deterministic script under data/traces/scripts/.
2. Have a human mint the JSONL with scripts/trace_java.sh.
3. Repeat the mint and compare bytes before committing the golden.
4. Record only facts visible in the committed golden.
5. Add offline Rust replay coverage through scripts/trace_diff.sh.

Uncommitted run logs, handwritten predictions, and old Python-engine outputs do
not qualify for this inventory.
