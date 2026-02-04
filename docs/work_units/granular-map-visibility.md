# Map Visibility (Model-Facing)

## Scope summary
- Expose the full current‑act map so the agent can plan pathing.
- Ensure map action selection is deterministic and index‑based.

## Observation requirements
- `map` must be present in every observation (see `granular-observation.md`).
- `nodes`: all nodes in the current act, including room type and emerald key flag.
- `edges`: all directed edges connecting nodes, including boss edges.
- `available_paths`: ordered list of reachable nodes from current position.
- `visited_nodes`: list of `{act, x, y}` for path history.

## Room type encoding
- Use `RoomType` names (e.g., `MONSTER`, `ELITE`, `REST`, `SHOP`, `EVENT`, `TREASURE`, `BOSS`, `TRUE_VICTORY`).
- Unknown room content remains visible as `EVENT` (the `?` room).

## Action integration
- Map navigation uses `path_choice{node_index}`.
- `node_index` is the index into `map.available_paths` (ordering is stable).
- If the available path is a boss edge, `room_type` should be `BOSS` and `is_boss` true in the edge list.

## Acceptance criteria
- Agent can reconstruct the full map graph from observation alone.
- `available_paths` ordering is stable for identical state.
- Choosing `path_choice{node_index}` moves the agent to the corresponding node.
