# Potions Domain Audit

## Status
- High-priority potion parity slice is implemented and test-covered.
- Selection-required potion flows are explicit in agent action dict interface.
- Remaining risk is mostly audit traceability completeness and global RNG normalization.

## Confirmed implemented parity slice
- Discovery-family offer and selection behavior:
  - `AttackPotion`, `SkillPotion`, `PowerPotion`, `ColorlessPotion`
- Selection potions:
  - `LiquidMemories`, `GamblersBrew`, `ElixirPotion`, `StancePotion`
- Runtime behavior:
  - `DistilledChaos` top-card play flow
  - `EntropicBrew` fill-empty-slots flow
  - `SneckoOil` random-cost rewrite path
  - `SmokeBomb` combat escape invariants
- Compatibility behavior:
  - Sacred Bark interactions and deterministic action-surface support

## Action-surface guarantees
- `get_available_action_dicts()` marks selection-required potions with `requires` metadata.
- `take_action_dict` returns explicit selection candidates when missing required params.
- Follow-up selection uses explicit `select_cards` / `select_stance` actions.

## RNG notes
- Potion behavior uses dedicated stream ownership where implemented (`potion_rng`, `card_rng`, `card_random_rng`).
- Remaining campaign risk: engine-wide direct Python `random` callsites still exist and should be normalized (`CONS-001`).

## Remaining tasks
- [ ] Add full potion inventory traceability rows to `gap-manifest.md` with exact Java refs.
- [ ] Restore/attach local Java potion class inventory snapshot for fully auditable class-level parity.
- [ ] Complete global RNG normalization pass across all domains.

## Python touchpoints
- `packages/engine/registry/potions.py`
- `packages/engine/content/potions.py`
- `packages/engine/game.py`

## Test evidence
- `tests/test_potion_effects_full.py`
- `tests/test_potion_rng_streams.py`
- `tests/test_potion_runtime_dedup.py`
- `tests/test_potion_sacred_bark.py`
