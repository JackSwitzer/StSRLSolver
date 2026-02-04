# Phase Flow (State Machine)

## Scope summary
- Define allowed phase transitions and required actions.
- Prevent illegal phase changes.

## Phases
- `NEOW`, `MAP_NAVIGATION`, `EVENT`, `COMBAT`, `COMBAT_REWARDS`, `SHOP`, `REST`, `TREASURE`, `BOSS_REWARDS`, `RUN_COMPLETE`

## Phase actions (model-facing)
- `MAP_NAVIGATION` uses `path_choice{node_index}`.

## Allowed transitions
- `NEOW` → `MAP_NAVIGATION`
- `MAP_NAVIGATION` → `COMBAT` | `EVENT` | `SHOP` | `REST` | `TREASURE` | `BOSS_REWARDS`
- `COMBAT` → `COMBAT_REWARDS` | `RUN_COMPLETE`
- `COMBAT_REWARDS` → `MAP_NAVIGATION`
- `EVENT` → `MAP_NAVIGATION` | `COMBAT` (event-triggered fights)
- `SHOP` → `MAP_NAVIGATION`
- `REST` → `MAP_NAVIGATION`
- `TREASURE` → `MAP_NAVIGATION`
- `BOSS_REWARDS` → `MAP_NAVIGATION` | `RUN_COMPLETE`

## Acceptance criteria
- Each phase exposes only its allowed actions.
- Transitions are only those listed above.
- Any illegal transition returns a structured error and preserves state.
