# Potions Domain Audit

## Status
- Baseline: strong parity coverage and action-surface support.
- Remaining work: regression lock and manifest synchronization.

## What is complete
- Selection potion action roundtrip behavior.
- RNG stream assertions for key potion paths.
- Sacred Bark and Fairy invariants covered.

## Remaining tasks
- [ ] Map all potion rows into canonical gap manifest as `exact` with references.
- [ ] Add parity-campaign traceability assertions for potion inventory completeness.
- [ ] Keep runtime/registry semantics synchronized in tests.
