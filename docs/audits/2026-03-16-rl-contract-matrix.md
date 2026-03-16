# RL Contract Matrix (2026-03-16)

Canonical target surface:

- `GameRunner.get_observation(profile="human")`
- `GameRunner.get_available_action_dicts()`
- `GameRunner.take_action_dict()`

## Matrix

| Surface | Engine Canonical | `overnight.py` | `gym_env.py` | `training_runner.py` |
|---|---|---|---|---|
| Observation payload | Phase-aware dict with schema markers | Custom encoded run vector | `ObservationEncoder` array | Mixed live summaries for UI |
| Action semantics | Semantic action dicts with ids/params | Raw `get_available_actions()` slots | ActionSpace over action dict ids | Mostly heuristic planner choices |
| Candidate detail | Explicit | Missing | Present in `available_actions`, not in observation | Mostly dropped from durable logs |
| Reward contract | Engine result + full phase state | Shaped heuristic mix + PBRS | Wrapper reward | No learning reward, dashboard only |
| Replayability | High if action dicts persisted | Low | Medium inside env episode only | Low, truncated summaries |
| Artifact schema | Schema markers on observation | Non-canonical JSONL | None beyond rollout log | Non-canonical per-agent JSONL |

## Main Contract Gaps

1. Strategic inference does not receive semantic candidate actions.
2. Strategic action heads assume a fixed `256`-wide action space.
3. Training artifacts do not persist the canonical action ids needed to replay
   `take_action_dict()` decisions.
4. `training_runner.py` is rollout/visualization infrastructure, not the
   canonical RL loop.
