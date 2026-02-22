# Events Domain Audit

## Status
- Event coverage is structurally complete in the handler layer.
- Action-surface completeness is still incomplete at the runner/action API boundary.

## Confirmed facts
- [x] Event definitions registered: 51
- [x] Event choice generators registered: 51
- [x] Event handlers registered: 51

## Confirmed open gaps
- [ ] `EVT-001` card-required event choices do not expose explicit follow-up selection actions.
- [ ] `EVT-002` selected card indices are not passed to event execution (`card_idx` forced to `None`).
- [ ] `EVT-003` deterministic multi-phase transitions need explicit action-surface tests.
- [ ] `EVT-004` alias mapping should be hardened and tested against Java class names.

## Java references
- `com/megacrit/cardcrawl/events/exordium/**`
- `com/megacrit/cardcrawl/events/city/**`
- `com/megacrit/cardcrawl/events/beyond/**`
- `com/megacrit/cardcrawl/events/shrines/**`

## Python touchpoints
- `packages/engine/handlers/event_handler.py`
- `packages/engine/game.py` (`_handle_event_action`, `take_action_dict`)

## Next commit order
1. `EVT-001`
2. `EVT-002`
3. `EVT-003`
4. `EVT-004`
