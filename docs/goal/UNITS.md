# UNITS — Tier-1 Work Queue

System-level units in dependency order. Tier-2 (per-item burn-down) lives in `ledger.json` once U07 generates it. **env**: `sandbox` = any agent, offline; `game` = needs the real game, human-attended (not agent-loop work).

Update the Status column in place as work lands (`ready` → `in-progress` → `done`, or `blocked(reason)`).

| id | title | env | depends | status |
|----|-------|-----|---------|--------|
| U00 | Ship viz source from worktree | local | — | done (PR #150) |
| U01 | Clean room: archive legacy, fix stale docs | sandbox | — | done (PR #152) |
| U02 | Trace schema module (`trace.rs`) | sandbox | — | done (PR #151) |
| U03 | Sim-core boundaries + `check-arch` | sandbox | — | ready |
| U04 | TraceLab mod + `trace_java.sh` | game | U02 | done — B0 proven (PR on `claude/u04-tracelab`) |
| U05 | `trace_replay` bin + differ + `trace_diff.sh` | sandbox | U02, one U04 golden | done (PR #153) |
| U0X | Content extraction tooling (data tables + method index) | sandbox | — | in progress |
| U06 | Mint A0 corpus + oracle test | game | U04, U05 | ready (1 golden minted; expand corpus) |
| U07 | Ledger generator + coverage (`goal.sh`) | sandbox | U05, U0X | ready |
| U08 | Enemy roll parity vs decompiled getMove (see FINDINGS F1/F3) | sandbox | — | ready — RNG threading already done; verify/fix per-enemy logic + `ai` counter |
| U09 | Run-layer smalls (Neow blessing F4, rng_counters F2, potion-slot F5) | sandbox | — | ready |
| U10 | Enemy roll parity — Acts 2-4 Watcher path | sandbox | U08 | blocked(U08) |
| U11 | complex_hook burn-down (Watcher-relevant) | sandbox | U0X | ready |
| U12 | Acts 2-4 content burn-down (ledger-driven) | sandbox | U07, U10 | blocked |
| U13 | Viz ParityView | sandbox | U00, U05 | ready |
| U14 | A20 corpus (stretch) | game | DoD 1-5 at A0 | deferred |

**Oracle model (two-tier, per session decision):** per-unit work is verified by **offline smart tests** derived from `decompiled/java-src` (seed the relevant RNG, assert the sequence/values against the decompiled logic — cheap, no game) using the U0X method index; the **full-run trace corpus** (U06, `trace_diff.sh`) is the periodic end-to-end integration gate. See `docs/goal/BENCHMARKS.md` and `docs/goal/FINDINGS.md`. Note: existing hand-written parity tests may encode *wrong* expected values (see F1) — re-derive from source, not from the current tests.

## U00 — Ship viz source
Merge branch `claude/wonderful-tharp`'s `packages/viz/` src, `scripts/viz.sh`, `.claude/launch.json` to main via PR (safety commit already pushed on that branch). **Accept**: `packages/viz/src/sprites/index.tsx` on main; `bun install && bun run build` succeeds in `packages/viz`.

## U01 — Clean room
`git mv packages/engine packages/parity → archive/2026-07-python-engine/` after verifying nothing live imports them (`grep -r "packages.engine\|packages\.parity" --include="*.py" packages/training tests scripts`); banner superseded `docs/work_units/granular-*.md` + `python-to-rust-migration.md` if stale; rewrite `CLAUDE.md` "Active Branch Shape" (stale PR #132/#133 stack) to point at `AGENTS.md`/`docs/goal/`. **Accept**: `uv run pytest tests/training -q` green; no un-bannered doc references the Python engine as active.

## U02 — Trace schema
`packages/engine-rs/src/trace.rs`: serde structs for header/record/action/report per TOOLING T1, `v:1`, unit-tested round-trip + one committed hand-written fixture. **Accept**: `./scripts/test_engine_rs.sh test --lib trace_` green.

## U03 — Sim-core boundaries
Feature-gate PyO3 (`python` feature, default off for the bin path if currently unconditional); add `scripts/goal.sh check-arch` (core modules ban `obs|search|training_contract|pyo3` imports); fix any violations found (expect small). **Accept**: `check-arch` exits 0; full lib suite green with and without `--features python`.

## U04 — TraceLab mod (game)
Per TOOLING T2. Resurrect from git history, rename, extend to per-action records with ordered piles + 13 counters; scripted seeded launch; `scripts/trace_java.sh`. **Accept**: verified-seed Act-1 combat script produces a trace whose values match `docs/vault/seed-WATCHER-57554006466-full-prediction.md` spot-checks; same seed twice → byte-identical traces (determinism A/B).

## U05 — trace_replay + differ
Per TOOLING T3/T4. **Accept**: exits 0 on a combat the engine already matches; on a known §1.1 enemy-branch case reports `first_divergence` at the right action with the `rng.ai` counter delta; masks file respected (DEV-required); `test_trace_oracle.rs` scaffold runs committed goldens in the lib suite.

## U06 — Mint A0 corpus (game)
Per TOOLING T5: ~10 coverage-maximizing A0 seeds + golden run `1776347657` action-script reconstruction (source its decisions from the .run file + EVTracker logs where available). Commit goldens + scripts. **Accept**: every script has a golden; `test_trace_oracle.rs` covers all; divergences at this point are *expected* (they are U08+'s backlog) and each gets a tracking line in `parity-status.md`.

## U07 — Ledger + coverage
Per TOOLING T6. **Accept**: `goal.sh ledger` emits rows for all Watcher-reachable content with `java_ref`s; `goal.sh coverage` stamps corpus coverage; `goal.sh status` prints the dashboard; reachable-but-uncovered items listed (feeds corpus additions).

## U08 — Enemy AI RNG, Act 1 (audit §1.1)
Thread `aiRng` through Act 1 enemy intent rolls per `decompiled/java-src/.../monsters/**` (`getMove(int roll)`, `aiRng.random(99)` semantics, move-history constraints; `docs/vault/enemy-ai-patterns.md`). **Accept**: Act-1 corpus combats reach `match` through their combats (or masked DEV); `rng.ai` counters track Java exactly; lib suite green (existing deterministic-intent tests updated with Java citations).

## U09 — Run-layer smalls
Audit §1.4 (replay energy honors relics), §1.5 (Neow TEN_PERCENT_HP_LOSS), first-turn intent gating, other single-file audit items. One PR each or batched, cite audit § in commits. **Accept**: corresponding corpus fields go exact; suite green.

## U10 — Enemy AI RNG, Acts 2-4 Watcher path
Same as U08 for the Watcher-reachable Act 2-4 + Heart roster (Chosen, Byrds, BookOfStabbing, GremlinLeader, Reptomancer, Nemesis, Transient, Maw, Darklings, TimeEater, AwakenedOne, SpireShield/Spear, CorruptHeart, …). **Accept**: full-run corpus traces pass combat segments through Act 4 or carry DEV masks.

## U11 — complex_hook burn-down (Watcher-relevant)
From `docs/research/engine-rs-audits/COMPLEX_HOOK_AUDIT.md`: migrate Watcher-reachable fallback cards to real implementations (Lesson Learned first; Wish/Nightmare/Omniscience per effort cap → quarantine candidates). **Accept**: per-card lib tests + any corpus trace touching the card stays/goes green; ledger rows flip.

## U12 — Acts 2-4 content burn-down
Ledger-driven grind: every red row (relics/powers/events/potions on the Watcher path) → implement with Java citation → green via its oracle + corpus. This is the long "until done" tail; add corpus scripts if coverage gaps block greening. **Accept**: DoD item 2.

## U13 — Viz ParityView
Per TOOLING T7. **Accept**: opening the viz app on a real U06+ divergence report shows side-by-side state at the diverging action with sprites and highlighted fields.

## U14 — A20 corpus (stretch, deferred)
A20 scripts + goldens (ascension modifiers per `docs/vault/ascension-modifiers.md`); only after A0 DoD holds.
