# Relics Domain Audit

## Status
- Many behavior fixes are in place (eggs, chest counters, rest-site interactions, on-use triggers).
- `REL-003`, `REL-004`, and `REL-008` are complete: Orrery/Bottled/Dolly now use explicit follow-up selection actions in shop/reward action flow.
- `REL-005` and `REL-006` are complete: deterministic selection IDs are locked and relic alias/inventory closure now resolves Java-style IDs with shared canonical lookup.
- Highest-priority remaining relic work is `REL-007` ordering edge cases.

## Confirmed open gaps
- [ ] `REL-007` residual ordering edge cases in reward/chest transitions.

## REL-006 closure summary
- Java reference check:
  - `com/megacrit/cardcrawl/relics/Toolbox.java` confirms `ID = "Toolbox"` and `RelicTier.SHOP`.
  - Java relic ID inventory currently has `186` IDs in decompiled snapshot (includes test relic IDs: `Test 1/3/4/5/6`).
  - Python content inventory is now `181` IDs, with `Toolbox` added.
- Implemented scope:
  - Added missing `Toolbox` content entry with `SHOP` tier and Java-consistent trigger text.
  - Added canonical alias resolver in content layer for Java/class-name IDs.
  - Routed run-state relic lookup/add paths through the same resolver to avoid split alias logic.
- Regression tests added:
  - `tests/test_relic_aliases.py::test_toolbox_is_registered_as_shop_relic`
  - `tests/test_relic_aliases.py::test_get_relic_resolves_java_style_alias_ids`
  - `tests/test_relic_aliases.py::test_run_state_add_relic_canonicalizes_aliases`
  - `tests/test_relic_aliases.py::test_run_state_alias_canonicalization_preserves_pickup_effects`

## Completed in this batch
- [x] `REL-003` Orrery explicit selection actions.
  - Shop/reward acquisition returns `requires_selection` + `candidate_actions`.
  - Follow-up `select_cards` roundtrip applies one choice per generated offer bundle.
  - Runtime relic application consumes selected indices through `selection_card_indices`.
  - Tests: `tests/test_agent_api.py` Orrery selection flow tests.
- [x] `REL-004` Bottled relic explicit selection actions.
  - Shop/reward acquisition returns `requires_selection` + candidate card picks.
  - Follow-up `select_cards` roundtrip assigns bottled card IDs from selected deck indices.
  - Runtime relic application consumes selected indices through `selection_card_indices`.
  - Tests: `tests/test_agent_api.py` bottled selection flow tests.
- [x] `REL-008` Dolly's Mirror explicit selection actions.
  - Shop/reward acquisition returns `requires_selection` + deck-card candidate picks.
  - Follow-up `select_cards` roundtrip duplicates selected card with upgrade/misc preservation.
  - Runtime relic application consumes selected indices through `selection_card_indices`.
  - Tests: `tests/test_agent_api.py` Dolly selection flow tests.
- [x] `REL-005` Deterministic selection ID + validation hardening.
  - Added deterministic-ID regression tests for Orrery candidate action sets.
  - Added validation rejection tests for invalid Orrery/bottled selection payloads.
  - Tests: `tests/test_agent_api.py` selection determinism/validation tests.
- [x] `REL-006` Relic alias normalization + Java ID closure (`Toolbox`).
  - Added `Toolbox` to content registry as a `SHOP` relic.
  - Added canonical `resolve_relic_id` lookup and Java-style alias map in content layer.
  - Updated `RunState` relic add/lookup normalization to use the shared content resolver.
  - Updated stale Dolly pickup tests to assert true pickup behavior under alias canonicalization.
  - Tests: `tests/test_relic_aliases.py`, `tests/test_relic_pickup.py`.

## Java references
- `com/megacrit/cardcrawl/relics/Orrery.java`
- `com/megacrit/cardcrawl/relics/BottledFlame.java`
- `com/megacrit/cardcrawl/relics/BottledLightning.java`
- `com/megacrit/cardcrawl/relics/BottledTornado.java`
- `com/megacrit/cardcrawl/relics/DollysMirror.java`
- `com/megacrit/cardcrawl/relics/Toolbox.java`

## Python implementation touchpoints
- `packages/engine/state/run.py` (`_on_relic_obtained`, `add_relic`)
- `packages/engine/game.py` (pending selection context, action dict interception)
- `packages/engine/registry/relics.py` (battle triggers and orb-linked TODOs)

## Next commit order
1. `REL-007`
2. `EVT-001`
