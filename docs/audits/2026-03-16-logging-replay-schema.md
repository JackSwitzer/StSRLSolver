# Logging and Replay Schema Gaps (2026-03-16)

## Current State

The repo currently has multiple incompatible artifact shapes:

- `packages/training/episode_logger.py`
- `packages/training/overnight.py`
- `packages/server/training_runner.py`
- dashboard-facing summary files such as `recent_episodes.json`

These shapes are not mutually reconstructible.

## Required Canonical Fields

Every future persisted episode/run artifact should include:

- `run_id`
- `seed`
- `ascension`
- `character`
- config snapshot
- git SHA
- resume lineage
- `phase_trace`
- `decision_trace`
- `combat_trace`
- canonical action ids
- legal candidate ids or candidate descriptors
- reward payloads by source
- `event_id`
- potion identity / slot / target / outcome
- path options / chosen path
- timestamps

## Known Current Gaps

| ID | Gap |
|---|---|
| `LOG-001` | `ReplayRunner` does not replay real floor transitions. |
| `LOG-002A` | `EpisodeLog.to_dict()` omits phase/combat/decision traces. |
| `LOG-002B` | Overnight episode output omits canonical action ids and full candidate surfaces. |
| `LOG-002C` | Training-runner durable logs truncate combats, decisions, and deck diffs. |
| `LOG-003` | `training-status.sh` does not speak the current WebSocket protocol. |

## Archived Evidence Note

Archived March 11-12 artifacts under `~/Desktop/sts-archive/logs/runs` were used
as audit evidence only. The gap tests in this PR use checked-in synthetic
fixtures instead of depending on local machine paths.
