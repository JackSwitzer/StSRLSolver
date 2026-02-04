# Agent Interface (Model-Facing, No UI)

## Scope summary
- Define the minimum API surface for a non-player agent to run the game.
- Ensure actions are deterministic, parameterized, and never require UI.
- Establish error handling and “missing params” resolution behavior.

## Required API surface
- `GameRunner.get_available_action_dicts() -> List[ActionDict]` (JSON‑first)
- `GameRunner.take_action_dict(action: ActionDict) -> ActionResult`
- `GameRunner.get_available_actions() -> List[GameAction]` (legacy dataclass API; still supported)
- `GameRunner.take_action(action: GameAction | ActionDict) -> ActionResult`
- `GameRunner.get_observation() -> ObservationDict`

## Action handling rules
- Actions follow `docs/work_units/granular-actions.md`.
- Actions are JSON‑serializable dicts (`id`, `type`, `label`, `params`, `phase`).
- Action IDs must be deterministic for identical state + phase.
- Action lists must be **non-empty** in all phases (no dead-ends).
- If required params are missing, `take_action` returns a structured list of follow‑up actions (no state mutation).
- Invalid action or params must return `{success: false, error: ...}` without state mutation.

## Phase behavior
- Each phase exposes only actions valid for that phase.
- `take_action` must advance to the next valid phase or remain in phase if more input is required.

## Observation API rules
- `get_observation()` must include `map` visibility per `granular-map-visibility.md`.
- Observation schema matches `granular-observation.md` (stable fields by phase).

## Acceptance criteria
- For every phase, `get_available_actions()` returns at least one valid action.
- Replaying the same action list on identical state yields identical ordering and IDs.
- `take_action` with missing params returns follow‑up action options and does not alter state.
- Invalid actions do not mutate state and return a structured error.
- `get_observation()` includes map visibility and can drive `path_choice` selection.
