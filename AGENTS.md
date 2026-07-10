# AGENTS.md — Read This First

Clean-room **Rust simulator of Slay the Spire** (`packages/engine-rs`) for RL training, verified against the real game. Active mission: **Watcher full-run parity**.

## The contract (binding)

1. **Ground truth is the decompiled game**, nothing else. `decompiled/java-src/com/megacrit/cardcrawl/` (local-only, gitignored; regenerate with `scripts/decompile_java.sh`). Agent-ready distillation: `reference/extracted/` — data tables + per-item verbatim logic at `reference/extracted/methods/<kind>/<Class>.java` (regenerate with `scripts/extract.sh`).
2. **The existing engine is an unverified draft.** It is ~mostly right, but nothing counts as done until checked against source. **Existing tests are NOT gospel** — some assert wrong values copied from misread source (proven: `test_ai_rng_parity.rs` vs JawWorm `getMove`, see `docs/goal/FINDINGS.md` F1). You may rewrite any test **if and only if** you cite the decompiled file/lines that contradict it. Never delete coverage without a source-derived replacement.
3. **The loop is the ledger.** `docs/goal/ledger.json` — one row per content item (667: cards, monsters, relics, potions), all starting `unverified`. Work = pick rows, verify, flip. Done = no `unverified` rows on the Watcher-reachable set (plus `docs/goal/GOAL.md` Definition of Done).

## The verification loop (what "/goal" means)

For the next `unverified` row (prefer: Watcher cards → Act 1 monsters → Watcher-reachable relics/potions → the rest):

1. Read the item's logic at its `methods_ref` (and the full source file if needed).
2. Compare the Rust implementation (grep `packages/engine-rs/src/` for the item id).
3. Confirm or fix the Rust — cite the Java file in a comment. Match RNG consumption exactly (every `aiRng.randomBoolean(p)` etc. consumes a counter tick; the trace oracle diffs counters).
4. Replace/extend the item's test with a **source-derived** one (expected values re-derived from the Java you just read, not from existing tests or engine output).
5. Run `./scripts/test_engine_rs.sh test --lib` (must stay green) and, when a relevant golden exists, `./scripts/trace_diff.sh data/traces/scripts/<script>.json`.
6. Flip the ledger row: `status: "verified"`, `verified_by: "<branch/commit>"`. Genuinely blocked after 2 real attempts → `status: "quarantined"`, `dev: "DEV-NNN"` + register entry per `docs/goal/GOAL.md` Edge-Case Policy.
7. Commit (`verify(<kind>/<Id>): <what changed>`), batch related items per commit sensibly. Repeat.

**Escalation, never workaround**: if source reading is ambiguous and you need real-game evidence, write the desired action script to `data/traces/requests/<name>.json` and continue with other rows — a human mints goldens (`scripts/trace_java.sh` launches the actual game; agents never do).

## Verify commands

```bash
./scripts/test_engine_rs.sh test --lib          # 2251+, only goes up
scripts/trace_diff.sh data/traces/scripts/<s>.json   # offline parity oracle (needs committed golden)
uv run pytest tests/training -q                 # only if you touched anything near training (don't)
```

## Hard rules (violations = bounced work)

- Never modify: `data/traces/java/` (goldens), `decompiled/`, `packages/training/`, `logs/`, `runs/`. Never `rm -rf` — archive to `archive/`.
- Never launch the game (`ModTheSpire`, `trace_java.sh`, `play.sh`).
- Ledger discipline: never flip to `verified` without a source citation + test; never mask a trace diff without a `DEV-` register entry.
- Tooling: bash in `scripts/` + `jq`; Rust in `packages/engine-rs`; Python via `uv`, JS via `bun`. No new Python infra files, no docs beyond goal/register files.
- Branches `codex/verify-<slug>` (or `claude/`), PR to `main`, conventional commits. Never commit to `main`.

## Context map

`docs/goal/GOAL.md` (end state, invariants, quarantine policy) · `docs/goal/FINDINGS.md` (known gaps F1-F7 — good first targets) · `docs/goal/BENCHMARKS.md` (B0-B5 ladder; B0 done) · `docs/goal/TOOLING.md` (trace schema/oracle contracts) · `docs/goal/UNITS.md` (infra work queue) · `docs/vault/` (mechanics notes: RNG streams, seeds, launch procedures).

## Running the loop (human setup)

```bash
git worktree add ../SlayTheSpireRL-goal -b codex/verify-sweep origin/main
cd ../SlayTheSpireRL-goal && scripts/extract.sh   # materialize reference/extracted locally
codex   # then: /goal
```
