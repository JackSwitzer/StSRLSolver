# Java RNG Stream Ownership Spec

Last updated: 2026-02-24
Feature ID: `RNG-SPEC-001`

## Objective
Define one canonical randomness model for parity-critical runtime behavior.

## Core rules
- Java RNG semantics (`packages/engine/state/rng.py`) are the source of truth.
- Runtime-critical logic must not use Python `random.*` directly.
- Every random draw must be attributable to an owned stream.

## Stream ownership (minimum contract)
- `monster_rng`: encounter-level randomness for enemy selection/ordering.
- `ai_rng`: enemy intent and behavior variation.
- `monster_hp_rng`: enemy HP rolls.
- `event_rng`: event random outcomes.
- `card_rng`: card reward generation and card-pool random picks.
- `card_random_rng`: card effects using card-random stream semantics.
- `relic_rng`: relic generation and relic random picks.
- `potion_rng`: potion generation and random potion outcomes.
- `treasure_rng`: chest/reward randomization.
- `merchant_rng`: shop inventory/pricing randomization.
- `shuffle_rng`: pile shuffle operations.
- `misc_rng`: misc parity-critical random operations where Java uses misc stream.
- `neow_rng`: Neow reward randomization.

## Migration policy
- Phase 1: parity-critical runtime modules first:
  - `packages/engine/registry/relics.py`
  - `packages/engine/registry/potions.py`
  - `packages/engine/effects/defect_cards.py`
  - `packages/engine/game.py` (runtime paths only)
- Phase 2: generation/simulation support modules.

## Determinism policy
- Same seed + same legal action sequence must produce identical trajectories.
- Tests must lock both result equality and stream advancement expectations where practical.

## Allowed exceptions
- Non-parity tooling paths may temporarily use local RNG if isolated from runtime state evolution.
- Exceptions must be documented and covered by non-runtime guardrails.
