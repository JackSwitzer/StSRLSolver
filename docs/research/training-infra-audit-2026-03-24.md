# Training Infrastructure Audit: 2026-03-24

## Scope
- Audit the current training runner, worker/inference plumbing, monitoring, checkpointing, and shutdown behavior.
- Focus on weekend-run stability and efficiency on an M4 Mac mini with ~24 GB unified memory, Cloud remote desktop overhead, and long unattended runs.
- Do not implement behavior changes in this PR; document what is happening, what is high confidence, and what to fix first.

## What I Did
- Read the training orchestration and runtime paths:
  - `packages/training/training_runner.py`
  - `packages/training/worker.py`
  - `packages/training/inference_server.py`
  - `packages/training/strategic_trainer.py`
  - `scripts/training.sh`
- Replayed a small real run with `python -m packages.training.training_runner`.
- Captured runtime artifacts from `logs/manual_probe/`:
  - `episodes.jsonl`
  - `recent_episodes.json`
  - `best_trajectories/*.npz`
  - `status.json`
- Ran the focused training/inference test suite:
  - `uv run pytest tests/training tests/test_shared_inference.py -q`
  - Result: `86 passed in 7.36s`

## Executive Summary
- The biggest problems are not raw RAM pressure. They are orchestration bugs and observability gaps.
- The trainer can overshoot the requested game budget by up to a full collect window.
- During collect, `status.json` becomes stale even while episodes continue to complete, which makes the dashboard misleading exactly when long runs need visibility.
- Shutdown is not responsive enough for unattended runs. A signal only flips a flag; the main loop can remain blocked inside batch collection, and worker crashes can leave the run wedged without a warm checkpoint.
- Worker recovery is fragile. Replacement workers consume from a one-shot slot queue; once a worker dies, the replacement can fail to acquire a slot and silently continue without inference.
- Two decision-quality bugs are likely wasting weekend compute:
  - “Strategic search” does not actually evaluate distinct actions.
  - The combat-action safety net described in the docstring is not implemented, and the probe logs show repeated zero-card turns with multiple playable cards still in hand.

## High-Confidence Findings

### 1. `--games` is not enforced inside the collect phase
- Code:
  - `packages/training/training_runner.py:796`
  - `packages/training/training_runner.py:818`
- The outer loop respects `sweep_games < n_games` and `self.total_games < self.max_games`, but the inner collect loop does not.
- Current behavior:
  - Once a collect phase starts, it keeps going until `TRAIN_COLLECT_GAMES` is reached, even if the requested run budget has already been met.
  - With the current default `TRAIN_COLLECT_GAMES = 100`, a small run can overshoot by nearly 100 games.
- Reproduction:
  - A real run launched with `--games 1 --workers 1 --batch 1` immediately entered `=== COLLECT phase: gathering 100 games ===`.
- Why this matters:
  - Weekend sweeps waste time on unplanned extra games.
  - Small smoke tests are not trustworthy.
  - Graceful shutdown becomes slower because the run commits to an oversized collect window.

### 2. `status.json` is stale throughout collect, so monitoring lies during the busiest phase
- Code:
  - `packages/training/training_runner.py:185`
  - `packages/training/training_runner.py:226`
  - `packages/training/training_runner.py:806`
  - `packages/training/training_runner.py:893`
  - `packages/training/training_runner.py:957`
- Current behavior:
  - `write_status()` is called at phase boundaries, not after each finished game.
  - `_record_game()` updates counters and recent episodes, but does not refresh `status.json`.
  - `recent_episodes.json` and `floor_curve.json` are only flushed every 10 games.
- Reproduction:
  - In the probe run, `logs/manual_probe/status.json` remained frozen at:
    - `total_games = 0`
    - `inference.total_requests = 0`
    - `sweep_phase = collecting`
  - At the same time, `episodes.jsonl` had already reached 18 completed games and `best_trajectories/` was being populated.
- Why this matters:
  - The dashboard looks idle even when training is making progress.
  - Stalls, worker failures, or throughput regressions are much harder to diagnose remotely.
  - Inference stats in the dashboard are snapshots from phase start, not live state.

### 3. Shutdown is not responsive enough for unattended runs
- Code:
  - `packages/training/training_runner.py:1173`
  - `scripts/training.sh:68`
  - `scripts/training.sh:71`
- Current behavior:
  - `SIGINT`/`SIGTERM` only sets `_shutdown_requested = True`.
  - The main thread can still remain blocked inside `_collect_batch()`, which calls `ar.get(timeout=3600)` per game.
  - If a worker dies mid-batch, the run can stay alive while no warm checkpoint is written.
- Reproduction:
  - Sending `Ctrl-C` during collect logged `Graceful shutdown requested (SIGINT), finishing current batch...`.
  - A worker then died inside `TurnSolver`.
  - The main process stayed alive and no `shutdown_checkpoint.pt` or `summary.json` was written.
  - I had to force-kill the run.
- Why this matters:
  - A long boss/MCTS batch can delay shutdown for a very long time.
  - Weekend resume safety is weaker than it appears.
  - The shell wrapper’s 30-second grace period is much shorter than the Python-side blocking window.

### 4. Worker replacement is fragile because slot ownership is one-shot
- Code:
  - `packages/training/inference_server.py:328`
  - `packages/training/worker.py:74`
  - `packages/training/worker.py:87`
- Current behavior:
  - `InferenceServer` preloads `slot_q` once with `[0..n_workers-1]`.
  - Each worker consumes a slot on init.
  - There is no path that returns a slot when a worker dies.
  - A replacement worker can therefore time out acquiring a slot and continue without inference.
- Reproduction:
  - After interrupting the run and letting the pool recover, the process emitted:
    - `Worker failed to acquire slot from slot_q — running without inference`
- Why this matters:
  - A single worker crash can permanently degrade the pool.
  - The fallback path in a no-inference worker is “first legal action” for strategic decisions, which silently poisons data quality.
  - This is a strong candidate for “unstable and inefficient over the weekend.”

### 5. “Strategic search” currently burns inference calls without evaluating distinct options
- Code:
  - `packages/training/worker.py:37`
- Current behavior:
  - `_strategic_search()` loops over `n_actions`, but each loop encodes the exact same `runner.run_state`.
  - It never applies or simulates action `i`.
  - The only thing that changes is the number of repeated calls.
- Consequence:
  - The value estimates do not correspond to candidate actions.
  - The search policy is derived from repeated evaluations of the same state.
  - This adds overhead without adding information.
- Why this matters:
  - Weekend runs that enable `strategic_search` are likely paying extra inference cost for almost no benefit.

### 6. The combat safety net described in the code is not actually implemented
- Code:
  - `packages/training/worker.py:135`
  - `packages/training/worker.py:155`
- Current behavior:
  - The docstring says: “if solver returns EndTurn but playable cards exist, prefer a card.”
  - The implementation immediately returns the solver result, including `EndTurn`.
  - The fallback that prefers cards only runs when the solver returns `None`.
- Runtime evidence:
  - Probe episodes show multiple fights with:
    - `cards_played = 0`
    - repeated `playable_unplayed = 5` or `6`
    - turns ending with full energy and a full playable hand
- Why this matters:
  - This wastes a huge amount of training data.
  - It can make early-floor failures look like learning instability when the immediate problem is action selection quality.

### 7. Best-checkpoint writes are global, not per-run
- Code:
  - `packages/training/strategic_trainer.py:81`
  - `packages/training/strategic_trainer.py:348`
- Current behavior:
  - `StrategicTrainer` writes “best” checkpoints to `logs/strategic_checkpoints/`, not the active run directory.
  - Periodic/shutdown/final checkpoints are written to `run_dir`, so checkpoint ownership is split across two locations.
- Why this matters:
  - Cross-run contamination is easy.
  - Different worktrees/runs can overwrite each other’s “latest” checkpoint.
  - Resume behavior becomes harder to reason about during a long experiment sequence.

### 8. Worker-status files are hardcoded to `logs/active/workers`
- Code:
  - `packages/training/worker.py:226`
- Current behavior:
  - Worker JSON files do not follow `run_dir`; they always write through `logs/active`.
- Why this matters:
  - Direct launches that use a custom `--run-dir` do not get local worker files.
  - Multiple concurrent runs would collide.
  - Observability depends on using the shell wrapper, not on the Python runner itself.

### 9. Cold-start distillation is repeated per config
- Code:
  - `packages/training/training_runner.py:728`
  - `packages/training/training_runner.py:733`
  - `packages/training/training_runner.py:738`
- Current behavior:
  - On a cold start, each config in the sweep creates a fresh `StrategicTrainer`.
  - That trainer re-runs BC pretrain / pretrain / deep distillation unless a warm checkpoint is loaded.
- Probe evidence:
  - Even a tiny `--games 1` run logged repeated “Deep distillation: no data to distill from, skipping” for each config before real collection began.
- Why this matters:
  - If `best_trajectories/` exists, startup cost multiplies by the number of configs.
  - Weekend sweeps can spend a surprising amount of time re-doing the same bootstrap work.

## Runtime Notes From The Probe
- Real run path used:
  - `uv run python -m packages.training.training_runner --games 1 --workers 1 --batch 1 --run-dir logs/manual_probe`
- Observed memory footprint during collect:
  - Main process RSS: about `424 MB`
  - Worker process RSS: about `44-63 MB`
- Interpretation:
  - On its face, the current architecture does not look primarily RAM-bound on a 24 GB Mac mini.
  - CPU time and orchestration behavior look like the bigger weekend risks.
- Observed throughput:
  - The single-worker probe accumulated 18 completed episodes in under a minute.
  - This suggests the core runner can make forward progress, but the visibility and control surfaces are not keeping up.

## What Looks Good
- The focused training/inference unit tests are healthy.
- Shared-memory inference on Apple Silicon is present and tested.
- Run artifacts like `episodes.jsonl`, `best_trajectories/`, and warm checkpoints are conceptually good building blocks.
- The project already has enough telemetry in episode logs to reconstruct real failure modes after the fact.

## What Is Not Well Covered By Tests
- No test coverage found for:
  - budget enforcement inside the phased collect loop
  - signal-driven shutdown during collect
  - worker crash / worker replacement / slot reuse
  - `strategic_search` semantics
  - status freshness during long collect phases
  - per-run checkpoint isolation
  - combat “end turn with playable cards” guard

## Recommended Fix Order

### Priority 0: Make weekend runs safe to stop and observe
1. Enforce `n_games` and `max_games` inside the inner collect loop.
2. Refresh `status.json` after each completed game or after each completed batch.
3. Make `_collect_batch()` poll results incrementally instead of blocking for up to 3600s per item.
4. Ensure signal handling cancels or drains outstanding jobs and always writes a warm checkpoint if possible.

### Priority 1: Make worker recovery trustworthy
1. Replace one-shot `slot_q` ownership with deterministic slot assignment or explicit slot return on worker exit.
2. Treat “worker started without inference” as a hard failure, not a silent fallback.
3. Write worker status under `run_dir / "workers"` and optionally mirror through `logs/active`.

### Priority 2: Stop wasting weekend compute on broken search paths
1. Fix `_strategic_search()` so it evaluates actual candidate outcomes, or disable it until it does.
2. Implement the combat safety net promised by `_pick_combat_action()`.
3. Add counters to `status.json` for:
   - solver calls
   - zero-card turns
   - inference timeouts
   - no-slot workers
   - collect-phase completed games

### Priority 3: Clean up experiment bookkeeping
1. Keep all checkpoints under the active `run_dir`.
2. Distill once per run, not once per config, unless explicitly requested.
3. Separate “small smoke test” defaults from “weekend sweep” defaults so tiny runs do not inherit a 100-game collect window.

## Recommended Weekend Run Settings Before The Next Implementation PR
- Treat the current system as “debuggable but not unattended-safe.”
- Prefer shorter bounded runs with manual inspection until Priority 0 and Priority 1 are fixed.
- If you do another overnight run before fixes:
  - keep workers modest rather than maxing them immediately
  - watch `episodes.jsonl` and `recent_episodes.json`, not just `status.json`
  - do not assume `--games` is honored mid-collect
  - assume a signal may not produce a quick warm checkpoint

## Proposed Follow-Up PRs
1. `training-runner-safety`: budget enforcement, incremental status updates, shutdown responsiveness
2. `worker-slot-recovery`: deterministic slot assignment and hard-fail inference setup
3. `decision-quality-fixes`: real strategic-search evaluation plus combat end-turn guard
4. `run-bookkeeping-cleanup`: per-run checkpoints and distillation-once semantics

## Bottom Line
- The weekend instability is most plausibly coming from control-plane issues, not lack of raw machine capacity.
- The best next move is not a bigger sweep. It is a safety/visibility pass on the runner so long runs can be trusted, stopped, resumed, and interpreted.
