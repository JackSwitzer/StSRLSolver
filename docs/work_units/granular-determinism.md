# Determinism & RNG Spec

## Scope summary
- Define RNG stream usage and counter sync rules.
- Guarantee identical outcomes for identical seed + actions.

## RNG rules
- **No use of Python `random`** in game logic.
- All randomness must use `state.rng.Random` streams.
- Stream ownership:
  - `card` for card rewards and card‑generation effects.
  - `potion` for potion drops and potion‑generation effects.
  - `relic` for relic pools and relic rolls.
  - `event`/`misc` for event outcome rolls.
  - `merchant` for shop prices/tier rolls.
  - `ai` for enemy intent selection.
  - `shuffle` for deck shuffles.
  - `map` for map generation (seed + act offset only).

## Counter synchronization
- After each phase transition, sync RNG counters into `RunState` (or `GameRunner`).
- `save/load` must restore counters exactly.

## Acceptance criteria
- Two runs with same seed and action sequence yield identical observations and rewards.
- `save` → `load` → next RNG output matches a continuous run.
- Any new randomness explicitly declares its stream usage in code comments/tests.
