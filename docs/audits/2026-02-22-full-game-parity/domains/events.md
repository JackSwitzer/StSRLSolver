# Events Domain Audit

## Status
- Core handlers/choice generators mostly present.
- Choice-rich event action-surface parity is still incomplete at `GameRunner` boundary.

## Confirmed gaps
- [ ] Event choice flows requiring card selection are not fully model-traversable.
- [ ] `event_choice` path does not yet pass selected card index to handlers.
- [ ] Alias/inventory closure and canonical ID policy still needs hardening.

## Java reference focus
- event classes under `com/megacrit/cardcrawl/events/**`
- particularly flows with card removal/upgrade/transform/selection and multi-phase transitions.

## Feature IDs
- `EVT-001` selection follow-up actions
- `EVT-002` wire event selection params through action handling
- `EVT-003` deterministic multi-phase coverage
- `EVT-004` alias/inventory normalization
