# AGENTS.md — Read This First

Clean-room **Rust simulator of Slay the Spire** (`packages/engine-rs`) for RL
training, checked against the decompiled game. The source-verification sweep for
the ledger's 667 cards, monsters, relics, and potions is complete. That content
milestone does not imply full-run or systems parity; use the current scoped task
and `docs/goal/` to determine the active mission.

## The contract (binding)

1. **Ground truth is the decompiled game**, nothing else.
   `decompiled/java-src/com/megacrit/cardcrawl/` is local-only and gitignored;
   regenerate it with `scripts/decompile_java.sh`. Agent-ready extracts live in
   `reference/extracted/`, including per-item logic under
   `reference/extracted/methods/` (regenerate with `scripts/extract.sh`).
2. **Verification remains source-derived.** Existing Rust and tests are evidence,
   not ground truth. If a test contradicts Java, replace it with a source-cited
   test; never remove coverage without a source-derived replacement.
3. **The ledger records the completed content sweep.**
   `docs/goal/ledger.json` contains 667 verified rows (370 cards, 68 monsters,
   186 relics, and 43 potions). It does not cover every run-level system, event,
   map rule, Neow path, power interaction, or observation/training boundary.
4. **System gaps are tracked separately.** Read
   `docs/work_units/audit-reports/engine-deep-audit.md` and
   `docs/work_units/sim-completion-map.md` when present; known baseline gaps stay
   in `docs/goal/FINDINGS.md`.

## Source-derived correction workflow

This is the completed sweep's existing contract for a row that is reopened or
newly extracted. It does not redefine what `/goal` should select next; the
post-sweep replacement prompt in EDA-035 requires human approval.

When evidence reopens a verified content row or exposes a system-level parity
gap:

1. Read the relevant extracted method and the full Java source when needed.
2. Compare the Rust implementation and all tests that mention the behavior.
3. Match values, ordering, and RNG consumption exactly. Every Java RNG draw
   matters to the trace counters.
4. Add or replace a source-derived test. Cite the Java file in the test or the
   implementation comment.
5. Run `./scripts/test_engine_rs.sh test --lib`. When a committed golden covers
   the behavior, also run `scripts/trace_diff.sh <script.json>`.
6. Update the appropriate findings/register document. Use `scripts/ledger.sh`
   only when a ledger row's state actually changes; never hand-edit the JSON.
7. Commit a focused change using the repository's conventional commit style.

## Practical notes

- Ledger IDs are Java class names or game ID strings. The engine sometimes
  accepts both spaced and unspaced relic IDs, so search for both forms.
- Tests are distributed across many files. Search broadly before adding new
  coverage, and extend the closest source-derived test when practical.
- Base-class behavior lives under `reference/extracted/methods/base/`; consult it
  before reasoning about shared mechanics or RNG.
- In-combat RNG streams must match Java tick-for-tick. Run-level generation still
  has known stream-model gaps; do not hide those differences behind content-row
  assertions.
- `create_enemy(id, hp, max_hp)` does not take ascension. Ascension-specific spawn
  adjustments belong at the run combat-entry site unless a scoped design change
  explicitly replaces that architecture.
- The committed trace corpus is currently small. Source-derived unit tests remain
  necessary where no golden exercises a behavior.

If source reading is ambiguous and real-game evidence is required, add the
desired action script under `data/traces/requests/` and continue with other work.
A human runs `scripts/trace_java.sh`; agents never launch the game.

## Verify commands

```bash
./scripts/test_engine_rs.sh test --lib          # 2,883 pass + 11 ignored audit repros (2026-07-15)
scripts/trace_diff.sh data/traces/scripts/<s>.json
scripts/ledger.sh status                        # 667 verified, 0 unverified
uv run pytest tests/training -q                 # only for authorized training changes
```

## Hard rules

- Never modify `data/traces/java/`, `decompiled/`, `packages/training/`, `logs/`,
  or `runs/` during engine parity work. Never use `rm -rf`; archive uncertain
  material under `archive/`.
- Never launch the game (`ModTheSpire`, `trace_java.sh`, or `play.sh`).
- Never mark content verified without source evidence and a source-derived test;
  never mask a trace diff without a registered `DEV-` exception.
- Use bash plus `jq` for scripts, Rust for `packages/engine-rs`, Python through
  `uv`, and JavaScript through `bun`. Do not add Python infrastructure as a
  shortcut for engine work.
- Use `codex/` (or `claude/`) branches, open PRs to `main`, and never commit
  directly to `main`.
- Preserve unrelated worktree changes and obey any narrower restrictions in the
  active task.

## Context map

- `docs/goal/GOAL.md` — target invariants and definition of done
- `docs/goal/FINDINGS.md` — established system-level gaps
- `docs/goal/BENCHMARKS.md` — benchmark ladder
- `docs/goal/TOOLING.md` — trace schema and oracle contracts
- `docs/goal/UNITS.md` — canonical infrastructure queue
- `docs/work_units/audit-reports/engine-deep-audit.md` — ranked post-sweep audit register
- `docs/work_units/sim-completion-map.md` — remaining-layer gap map
- `docs/vault/` — mechanics notes, seed work, and human launch procedures

## Local reference setup

```bash
scripts/decompile_java.sh   # only when local decompiled sources are absent/stale
scripts/extract.sh
./scripts/test_engine_rs.sh test --lib
```
