# BENCHMARKS — The Measurable Ladder

Six levels, B0→B5. Each level is a command (or short command set) that exits 0, so "where are we" is never a judgment call. Levels B1+ are fully offline — any agent can run them; only B0 minting needs the real game (human-attended). `docs/goal/GOAL.md` Definition of Done = B3+B4+B5 held simultaneously.

**Current level: B0 achieved, B1 in progress** (2026-07-06 — full seeded run Neow→map→JawWorm combat→END_TURN traces byte-identically across runs; `trace_diff.sh` produces honest divergence reports. First real gaps catalogued in `docs/goal/FINDINGS.md`: enemy roll logic F1, rng-counter exposure F2, Neow mapping F4. Next: fix F2 so combat RNG parity is visible, then work F1 to reach B1.)

## B0 — Oracle online (infra proof, game-side)

```bash
scripts/trace_java.sh data/traces/scripts/smoke-neow-floor1.json /tmp/t1.jsonl   # exits 0, end status script_exhausted|max_floor
scripts/trace_java.sh data/traces/scripts/smoke-neow-floor1.json /tmp/t2.jsonl  # rerun
diff /tmp/t1.jsonl /tmp/t2.jsonl                                                 # byte-identical (determinism A/B)
scripts/trace_diff.sh data/traces/scripts/smoke-neow-floor1.json                 # runs end-to-end, any verdict
```

Pass = a scripted seeded run (Neow → path → combat turns) produces per-action records with all 13 RNG counters, twice identically, and the Rust differ consumes it without error.

## B1 — Honest first divergence (diagnosis quality)

A JawWorm (or other verified-seed Act 1) combat script diffs to either `match` or a `first_divergence` that is a *real known parity gap* — expected: `post.enemies[0].intent.*` / `post.rng.ai` per audit §1.1 — not schema or infra noise. Pass = the report's divergence path is explainable against `decompiled/java-src`. This is the gate for pointing Codex at U08: the loop's error signal is trustworthy.

## B2 — Act 1 exact

Every corpus script matches through its Act 1 segment (floors 1-16 or script end), or carries a `DEV-` mask. Metric: `N/11 scripts`, command: `for s in data/traces/scripts/*.json; do scripts/trace_diff.sh "$s" || exit 1; done` with Act-1-scoped scripts.

## B3 — Full-run exact (DoD 1 + 5)

11/11 corpus scripts (10 seeds + golden run `1776347657`) match Neow→Heart end-to-end including all 13 RNG counters. Constraints: masks in `docs/goal/masks.json` < 15, every mask references a `DEV-` register entry, quarantine list triaged.

## B4 — Coverage complete (DoD 2)

Zero `unverified` ledger rows on the Watcher-reachable set: every Watcher-reachable card, relic, potion, enemy, event, boss is `verified` or `quarantined` (DEV-documented). Command today: `jq '[.rows[] | select(.status=="unverified")] | length' docs/goal/ledger.json` (scoped to Watcher-reachable once U07's `goal.sh coverage` exists). If reachable content is never exercised by the corpus, the fix is a new corpus script, not a status flip.

## B5 — Regression frozen (DoD 3 + 4)

The whole ladder is enforced inside the offline suite, so it can never silently regress:

```bash
./scripts/test_engine_rs.sh test --lib     # 2251+ green, includes test_trace_oracle over all committed goldens
scripts/goal.sh check-arch                 # sim-core dependency direction holds (future tool — U03/U07)
uv run pytest tests/training -q            # training stack untouched
```

Already partially live: `test_trace_oracle.rs` runs synthetic fixtures today and auto-picks up real goldens as they land in `data/traces/java/`.

## Who runs what

| Level | Env | Runner |
|---|---|---|
| B0 mint | real game, window pops | Claude + Jack, attended |
| B1-B5 | offline, cargo + bash | any agent (Codex `/goal`, Claude, CI) |

## Status snapshot (2026-07-10)

| Unit | State | Where |
|---|---|---|
| Spec + AGENTS.md | merged | PR #149 |
| U00 viz ship | merged | PR #150 |
| U01 clean room | merged | PR #152 |
| U02 trace schema | merged | PR #151 |
| U05 replay + differ | merged | main (exit codes 0/1/2/3 verified) |
| U04 TraceLab | merged — B0 proven | PR #155 |
| U0X extraction + ledger | merged | main (667-row ledger, `scripts/extract.sh`) |
| U06 corpus | 1 golden minted (smoke-neow-floor1); expand | data/traces/ |
| U07 goal.sh tooling | ready | — |
| U08-U12 parity grind | verification sweep open (AGENTS.md loop) | ledger.json |
| U13 ParityView | ready | — |
| U14 A20 | deferred | — |
