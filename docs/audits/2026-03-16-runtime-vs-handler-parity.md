# Runtime vs Handler Parity (2026-03-16)

This companion note separates two different claims that were previously mixed
together:

- handler parity: isolated event/rest/reward helpers behave like Java
- runtime parity: `GameRunner` wires those helpers into the actual run loop

## Current Split

| Subsystem | Handler Status | Runtime Status | Notes |
|---|---|---|---|
| Event fights | Mostly implemented | Broken | Runtime ignores requested event encounter and starts generic hallway combat. |
| Post-event fight resolution | Partially modeled | Broken | Runtime leaves event state in `COMBAT_PENDING` and falls into generic rewards. |
| `?` room handling | Helper exists (`on_enter_question_room`) | Broken | Runtime never calls unknown-room resolution before `_enter_event()`. |
| Burning elite | Map flag exists | Broken | `has_emerald_key` never becomes `GameRunner.is_burning_elite`. |
| Campfire restrictions | `RestHandler` knows blocking relics | Broken | `GameRunner` still emits blocked upgrade actions. |
| Replay parity | JSONL parse exists | Broken | Replay path never advances beyond floor 0. |

## What To Trust Today

- Trust handler-level docs for isolated formulas and option generation.
- Do not trust handler-level closure as proof that the full runtime loop is
  parity-complete.
- Any gameplay-critical claim should be backed by a `GameRunner` integration
  test, not just a handler test.
