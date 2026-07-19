# Archived Engine Consumers

This directory preserves the pre-freeze observation, search, training-contract,
gameplay-session, and PyO3 adapters removed from the supported Rust simulator in
July 2026. They are historical reference only and are not semantic sources of
truth.

New consumers should build on the pure core interfaces:

- `GameAction` for every legal simulator input
- `StepOutcome` for acceptance, terminal state, the next decision, and events
- `CoreCheckpoint` for causal save, restore, hashing, and deterministic replay

Gameplay assertions formerly coupled to these adapters were moved to core tests
where they provided unique coverage. Observation layouts, action-ID codecs,
search policies, and Python bindings must be redesigned outside the faithful
simulation crate rather than reconnected from this archive.
