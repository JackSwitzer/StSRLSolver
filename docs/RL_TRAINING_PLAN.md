# Watcher RL Training Plan (5 Days, Mac mini M4 24GB)

This is a **doc-only** plan to bootstrap Watcher training using a MCTS‑distillation pipeline that fits on a Mac mini M4 (24GB). It assumes Watcher-only runs (no Prismatic Shard) and uses the current engine API.

## Goals
- Produce a Watcher policy that learns from MCTS‑guided trajectories.
- Track win rate and loss curves during training via a simple metrics file.
- Keep compute within CPU/MPS constraints and avoid unstable systems.

## Constraints and Assumptions
- **Watcher only** (no Prismatic Shard; Defect orbs not required).
- **Skip actions required** in rewards/potions/events for stable rollouts.
- **Violet Lotus** required for stance‑energy fidelity.
- Missing powers/relic triggers can bias training; see work-unit docs.

## Training Approach (MCTS Distillation)
1. **MCTS rollout policy** uses the engine to explore actions and select a strong action.
2. **Distillation dataset** is generated from MCTS decisions (state → action).
3. **Supervised training** on the dataset yields a fast policy.
4. Repeat: re‑run MCTS using the improved policy as a prior.

## Resource Budget (Mac mini M4)
- Parallel envs: 4–8 (CPU bound).
- MCTS rollouts: 16–64 per decision (start low).
- Policy model: small MLP (1–2 hidden layers, <1M params).
- Batch sizes: 128–512 (tune for RAM).

## 5‑Day Schedule
**Day 1: Data & telemetry scaffolding**
- Implement dataset format (JSONL or parquet).
- Add metrics writer (loss, win rate, avg floor, steps/sec).
- Run a small MCTS generation pass to validate pipeline.

**Day 2: Baseline MCTS distillation**
- Generate ~5–10k decisions using MCTS with small rollout budget.
- Train a supervised policy (cross‑entropy on action).
- Evaluate win rate on a small held‑out seed set.

**Day 3: Iterated improvement**
- Increase rollout budget gradually.
- Generate larger dataset (~25–50k decisions).
- Retrain; compare win rate + loss curves.

**Day 4: Curriculum + stabilization**
- Curriculum by act (Act 1 only → Act 2 → Act 3).
- Add simple action masking for known missing behaviors.

**Day 5: Evaluation + reporting**
- Fixed‑seed evaluation set for trend stability.
- Export metrics for website polling.

## Telemetry & Website Polling
Expose a simple local HTTP endpoint for polling (doc‑only plan):
- `GET /metrics/latest` → latest JSON snapshot
- `GET /metrics/history` → JSONL or JSON array of recent points

Minimal schema:
```json
{
  "step": 12345,
  "epoch": 7,
  "loss": 0.432,
  "win_rate": 0.18,
  "avg_floor": 12.7,
  "decisions_per_sec": 55.2,
  "timestamp": "2026-02-04T12:00:00Z"
}
```

The website can poll `/metrics/latest` every 10–30s and render loss/win‑rate curves. Persisting to `logs/training_metrics.jsonl` is still recommended for durability.

## Dependencies Needed (Future Implementation)
- Lightweight NN training (PyTorch + MPS if desired).
- Dataset I/O utilities.
- MCTS loop integrated with `GameRunner` action API.

## Known Risks
- Missing reward skip/claim actions will stall training loops.
- Incomplete relic/power triggers can skew value estimates.
- Event coverage gaps may bias rollouts; consider skipping events or forcing “leave”.

## References
- Engine API: `packages/engine/game.py`, `packages/engine/api.py`
- Work units: `docs/work_units/` (rewards, relics, powers, potions, events)
