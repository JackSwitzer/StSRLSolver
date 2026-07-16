# Parity Quarantine Register

This register is the source of truth for deliberate, scoped `DEV-NNN` trace masks. It does not carry forward the April `D1`–`D87` claims: those pre-sweep notes were neither a verified backlog nor proof of current behavior. Revalidated engine findings belong in the [deep-audit register](audit-reports/engine-deep-audit.md), and remaining system layers are tracked in the [simulator completion map](sim-completion-map.md).

## Current state

- Content ledger: **667 verified** (370 cards, 68 monsters, 186 relics, 43 potions)
- Quarantined ledger rows: **0**
- Active trace masks: **0** (`docs/goal/masks.json` is `[]`)
- Active `DEV-NNN` entries: **none**

Content verification does not establish full-run, event, map, Neow, power-interaction, RNG-stream, or observation-boundary parity. Record those gaps in the deep-audit register until a concrete trace mismatch must be quarantined.

## Quarantine workflow

Add a `DEV-NNN` entry only after two source-guided implementation attempts fail and the scoped trace mismatch blocks otherwise useful verification. Every entry must have a matching object in `docs/goal/masks.json`; masks must identify the narrowest exact path, item, and seed/floor scope available. Remove both the entry and mask when the mismatch is fixed.

Never use a quarantine to suppress an unexplained diff, a missing test, or a run-level RNG limitation that the current task can simply document.

## Entry template

### DEV-NNN — Short title

- **Ledger row:** `kind/Item` or `system-only`
- **Trace/script:** `data/traces/scripts/<name>.json`
- **Masked path and scope:** `<exact post-state path>` / `<seed@floor or item>`
- **Java evidence:** `decompiled/java-src/.../Class.java:<lines>`
- **Rust evidence:** `packages/engine-rs/src/...:<lines>`
- **Attempts:** `<commit 1>`, `<commit 2>`
- **Observed mismatch:** concise expected-versus-actual values, including RNG counters
- **Suspected cause:** concise hypothesis
- **Exit condition:** the exact result that permits deleting this entry and its mask
