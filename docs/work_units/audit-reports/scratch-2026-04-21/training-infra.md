# PR #138 Training-Infra Audit — 2026-04-21

Read-only review of `scripts/training.sh`, `packages/training/cli.py`,
`packages/training/stage2_pipeline.py`. Branch `claude/parity-audit-fleet-2026-04-21`.

## Summary

- `scripts/training.sh` — **yellow** (6 findings: 1 P1, 5 P2)
- `packages/training/stage2_pipeline.py` — **yellow** (5 findings: 2 P1, 3 P2)
- `packages/training/cli.py` — **green** (1 P2)
- Cross-cutting — **yellow** (3 findings: 1 P1, 2 P2)
- Total: 15 findings (4 P1, 11 P2, 0 P0)

No P0s. Nothing here will corrupt data or silently launch a broken overnight.
The P1 items are UX/correctness cliffs that will bite once someone uses these
paths in anger.

---

## training.sh

### P1: `smoke` does not run `archive_active` but is called by `launch`
`scripts/training.sh:47-66` vs `scripts/training.sh:97-105`. `launch --with-smoke`
calls `archive_active` **before** invoking smoke via the recursive sub-shell
call (`$SCRIPT_DIR/training.sh smoke`, line 101). `archive_active` has already
moved `logs/active` aside and re-created it empty — correct order, but if the
user ever runs `training.sh smoke` **directly** (documented bring-up path per
`docs/work_units/combat-first-training-rebuild.md:162-170` passes `--output-dir
logs/active`), there is no guard against a live overnight still writing to
`logs/active`. The smoke subcommand trusts `--output-dir` is safe without any
pid/lock check of its own.

### P2: smoke directory collision at 1-second resolution
`scripts/training.sh:49`. `stamp="$(date -u +%Y%m%dT%H%M%SZ)"` has second
resolution; `logs/smoke/$stamp` has no `$$` fallback (archive_active does, line
34). Two smokes within the same second would reuse the same output dir.
Unlikely in interactive use, but trivially hit by a script doing two smoke
invocations back-to-back.

### P2: `uv run python` exit-code double-trap
`scripts/training.sh:58-65`. `set -e` plus `rc=$?` after an `uv run` that fails
— `set -e` may exit the case arm before `rc=$?` can read the status. Currently
works because the subcommand is the last statement of a multi-line command with
explicit trailing args, but if someone refactors they could easily break the rc
capture. Consider `|| rc=$?` or explicit `set +e`/`set -e` pairing.

### P2: pid_file is written non-atomically
`scripts/training.sh:110`. `printf '%s\n' "$pid" > "$pid_file"` truncates then
writes. If a reader (e.g. monitor UI) reads while the write is in-flight, it
sees empty. The new regex guard in `archive_active:24` is precisely the right
defense. Consider `printf ... > "$pid_file.tmp" && mv "$pid_file.tmp"
"$pid_file"` for atomicity.

### P2: `launch` assumes no existing launcher child
`scripts/training.sh:108-110`. `archive_active` bails if `logs/active/*.pid` is
running (good), but `launch` lets the user pass `--pid-file logs/other/...` —
in that case `archive_active` still only checks `logs/active/*.pid`. A
concurrent launch with a non-default `--pid-file` escapes the running-pid
check. Probably acceptable since non-default is a power-user path, but worth
noting.

### P2: no disk-space guard pre-launch
`scripts/training.sh:97-112`. User rule (MEMORY.md:
`feedback_disk_monitoring.md`) calls for disk checks before overnight runs.
Neither `launch` nor `--with-smoke` has one. Out-of-scope for this PR is a
judgement call, but the gap is visible in the new launch flow.

---

## cli.py / stage2_pipeline.py

### P1: `0.84` synthetic split is not surfaced
`packages/training/stage2_pipeline.py:442`. `synthetic_target = round(total_cases
* 0.84)`. Stage E's stated goal was surfacing hidden constants; this one —
governing the single most consequential corpus ratio (synthetic vs imported) —
stayed hardcoded. If someone runs with `--target-cases 24` (the smoke default),
`round(24 * 0.84) = 20` synthetic + 4 imported; at `--target-cases 8` it's 7+1;
at `--target-cases 3` it's 3+0. This is a silent knob.

### P1: collection-pass multiplier schedule is hardcoded
`packages/training/stage2_pipeline.py:556`. `multiplier = 1 if pass_index == 0
else 2 if pass_index == 1 else 4`. This baked-in `1/2/4` visit-scaling ramp
controls compute per pass and is not flagged through CLI or config. Given Stage
E's surface-constants mandate, this is a notable omission — a reviewer or
overnight operator has no way to dial it.

### P2: `ROOM_KIND_CORPUS_WEIGHTS` has no normalization contract
`packages/training/stage2_pipeline.py:55-59,315-322`. Weights are raw
multipliers (`hallway:1, elite:2, boss:3`). Fine today, but
`_weighted_encounter_pool` uses `max(1, weight)` so a zero weight silently
becomes 1. If a user tried to drop hallways by setting weight 0, they'd get the
opposite. Prefer an explicit `if weight <= 0: continue` or at least a comment.

### P2: `_ENCOUNTER_POOL_SHUFFLE_SEED = 20260421` is a magic date
`packages/training/stage2_pipeline.py:312`. Deterministic-seed literal with no
comment on why changing it would change corpus composition. Given the rest of
the file is careful about provenance, this one literal in a module constant
feels isolated. The preceding comment explains *why shuffle*, not *why this
seed*.

### P2: `buckets = 8` is a hidden knob
`packages/training/stage2_pipeline.py:329`. Per-case opening-hand variance
multiplier. Not surfaced, not commented. A reviewer cannot tell whether it's
load-bearing or arbitrary.

### P2: smoke path is the full overnight command
`scripts/training.sh:53-58` passes `run-phase1-puct-overnight` with small
values. Good — smoke exercises the same code path as full training. No
short-circuit that could hide real bugs. (Flagging as confirmed-good, not
concern.) However `--target-cases 24 --collection-passes 1 --epochs 1` are
baked and not overridable; a user wanting an even smaller smoke (e.g. single
case) has no path.

---

## Cross-cutting consistency

### P1: `COLLECTION_WORKER_COUNT = 1` reported as truth, but schedule suggests multi-worker was intended
`packages/training/stage2_pipeline.py:45` / `cli.py:701`. The change from
`worker_count=12` to `worker_count=COLLECTION_WORKER_COUNT (=1)` is correct
(collection runs single-process), but the comment at stage2_pipeline.py:43-45
says "Collection runs single-process against the Rust engine" as fact. Check
whether `packages/engine-rs` supports parallel PyO3 calls — if yes, this
constant is a permanent throughput ceiling that should be a config, not a
literal. If no, the comment is fine but worth linking to the engine constraint
that forces it.

### P2: `uv` usage is consistent, `bun` is n/a
`scripts/training.sh:53,108,116` — all Python entry points go through `uv run
python`. No `pip`, `python3`, `poetry`. No JS/TS touched by this PR. User's
global rule holds.

### P2: `--with-smoke` and `archive` subcommands are not documented
`docs/CLAUDE-training.md:44-73` and `docs/work_units/combat-first-training-
rebuild.md:159-180` still show only the pre-PR command surface. The new
`archive`, `smoke`, and `--with-smoke` flows are only discoverable by reading
`training.sh`. User rule: `feedback_pretrain_verification.md` (smoke before
long runs) — the tooling now supports it but nothing in docs points people at
`--with-smoke`.

---

## Would-review-but-out-of-scope

- `packages/engine-rs/src/enemies/act3.rs:82-86` — Spiker roll-time fix in
  commit `d16cdba2` is correct (removes double-apply), but the test suite only
  covers roll-time state; no integration test asserts post-execute THORNS ==
  5 after combat_hooks fires. The comment at `test_enemy_ai.rs:686-689`
  admits this. Follow-up worth tracking.
- `packages/training/stage2_pipeline.py:302`. `mutated["player_hp"] = max(12,
  int(mutated["player_hp"]) - (bucket_index % 4))`. Floor of 12 HP is hardcoded
  and unexplained — why 12?
- `packages/training/stage2_pipeline.py:414-416`. Imported-case branch
  overwrites `snapshot["player_hp"]` and `player_max_hp` after
  `_mutate_snapshot_for_bucket` — subtle: the `max(12, ...)` cap above applies
  to synthetic cases only. If imported HP drops below 12, the cap doesn't
  trigger. Probably intentional (imported values are canonical) but
  undocumented.
- `scripts/training.sh:101` — recursive self-call via `$SCRIPT_DIR/training.sh
  smoke`. If the script is moved or symlinked mid-run, the sub-shell could
  resolve differently from the parent. `${0:A:h}` on zsh uses realpath-style
  resolution so it should work, but this is a pattern that's easy to break
  later.
- Smoke output dir is `logs/smoke/$stamp`, which is not covered by
  `archive_active`. Over time `logs/smoke/` will accumulate. Flag for a future
  cleanup subcommand (`training.sh clean-smoke` or similar).
