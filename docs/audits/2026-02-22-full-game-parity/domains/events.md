# Events Domain Audit

## Status
- Event coverage is structurally complete in the handler layer.
- Event action-surface now supports explicit card-selection follow-up for single-card event choices.
- Deterministic multi-phase runner transition coverage is now locked by explicit action-surface tests.
- Remaining event gap is alias/inventory hardening.

## Confirmed facts
- [x] Event definitions registered: 51
- [x] Event choice generators registered: 51
- [x] Event handlers registered: 51

## Confirmed open gaps
- [x] `EVT-001` card-required event choices expose explicit follow-up selection actions.
- [x] `EVT-002` selected card indices are passed to event execution (`card_idx` no longer forced to `None`).
- [x] `EVT-003` deterministic multi-phase transitions now have explicit action-surface tests.
- [ ] `EVT-004` alias mapping should be hardened and tested against Java class names.

## EVT-001 / EVT-002 implementation result
- `take_action_dict({"type":"event_choice"})` now performs selection detection via copied-state preview and returns:
  - `requires_selection: true`
  - deterministic `candidate_actions` (`select_cards`)
  - no live-state mutation on the first call.
- Event follow-up selection now routes through pending selection context and validates selected indices before dispatch.
- `_handle_event_action` now forwards selected card index to `EventHandler.execute_choice(..., card_idx=...)` and clears pending event selection state.
- `get_available_action_dicts()` marks event choices with `requires=["card_indices"]` when selection is required.

## Tests added in this slice
- `tests/test_agent_api.py::TestActionExecution::test_event_choice_requires_selection_without_state_mutation`
  - validates selection-required response and no first-call mutation.
- `tests/test_agent_api.py::TestActionExecution::test_event_choice_selection_roundtrip_uses_selected_card_index`
  - validates selected non-default index is the card actually removed.
- `tests/test_agent_api.py::TestActionExecution::test_event_multiphase_golden_idol_keeps_event_phase_and_updates_choices`
  - validates phase continuity (`GamePhase.EVENT`) and explicit secondary actions.
- `tests/test_agent_api.py::TestActionExecution::test_event_multiphase_golden_idol_followup_action_ids_are_deterministic`
  - validates deterministic follow-up action IDs across equivalent multi-phase states.
- Full suite after change: `4642 passed, 5 skipped, 0 failed`.

## Java references
- `com/megacrit/cardcrawl/events/exordium/**`
- `com/megacrit/cardcrawl/events/city/**`
- `com/megacrit/cardcrawl/events/beyond/**`
- `com/megacrit/cardcrawl/events/shrines/**`

## Python touchpoints
- `packages/engine/handlers/event_handler.py`
- `packages/engine/game.py` (`get_available_action_dicts`, `take_action_dict`, `_apply_pending_selection`, `_handle_event_action`)

## Next commit order
1. `EVT-004`
