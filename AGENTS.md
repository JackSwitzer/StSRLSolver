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
6. Flip the ledger row with the helper (never hand-edit the JSON): `scripts/ledger.sh flip <row-id> <branch>`, or after 2 failed real attempts `scripts/ledger.sh quarantine <row-id> <branch> DEV-NNN` + register entry per `docs/goal/GOAL.md` Edge-Case Policy. `scripts/ledger.sh status` shows counts + next rows.
7. Commit (`verify(<kind>/<Id>): <what changed>`), batch related items per commit sensibly. Repeat.

## Practical notes (read once — saves real tokens)

- **Ledger ids are Java class names or ID strings** (`relic/Pure Water` → class `PureWater`); the engine sometimes checks both spaced and unspaced forms (`"Violet Lotus" || "VioletLotus"`) — grep both.
- **An item's existing tests are scattered** across several files (wave files, `test_enemies.rs`, `test_cards_*.rs`); grep broadly for the id before writing a new test, and extend/replace in place.
- **Base-class ground truth** lives at `reference/extracted/methods/base/` (e.g. `AbstractMonster.java` `rollMove` = the one-aiRng-tick-per-turn contract). Check it before reasoning about RNG consumption.
- **RNG fidelity by layer**: in-combat streams (`ai`, `shuffle`, `cardRandom`…) must match Java tick-for-tick. Run-level generation (`RunEngine`) currently uses one shared stream, unlike Java's split streams — run-level tick-count parity is **not yet achievable**; verify run-level behavior semantically and do NOT quarantine rows over run-level tick counts (stream-splitting is a queued infra unit).
- **Ascension-dependent stats**: `create_enemy(id, hp, max_hp)` takes no ascension — patch at the spawn site in `run.rs::enter_specific_combat` (precedent: Sentry stagger, Cultist ritual). Do not re-plumb `create_enemy`.
- **The trace oracle is dormant until goldens land**: one smoke golden exists; `trace_diff.sh` only applies to rows a committed golden exercises. Smart source-derived tests are the per-row oracle — don't burn attempts on the trace step elsewhere.
- Exemplar commits to imitate: PR #157 (`verify(card/Eruption)`, `verify(monster/Cultist)`, `verify(relic/Pure Water)`).

**Escalation, never workaround**: if source reading is ambiguous and you need real-game evidence, write the desired action script to `data/traces/requests/<name>.json` and continue with other rows — a human mints goldens (`scripts/trace_java.sh` launches the actual game; agents never do).

## Verify commands

```bash
./scripts/test_engine_rs.sh test --lib          # green, count only goes up (2255+ as of 2026-07-10)
scripts/trace_diff.sh data/traces/scripts/<s>.json   # offline parity oracle (needs committed golden)
scripts/ledger.sh status                        # loop state
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
