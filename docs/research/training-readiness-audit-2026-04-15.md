# Training Readiness Audit

Last updated: 2026-04-15  
Branch: `codex/universal-gameplay-runtime`

This document captures the current training-facing discrepancies after the final parity audit.

## Summary

The Rust engine is strong enough to train against, but the Python/training layer is not yet fully aligned with the live RL surface.

Current training-readiness blockers:

1. `Neow` is modeled in Rust, but training still skips it by default.
2. The observation contract is split between Rust run observations and Python training observations.
3. There is no first-class train-time restriction layer for curriculum rules like `no card rewards`.
4. Run provenance is not complete enough yet for serious overnight experiment comparison.

## Confirmed Rust vs Python Discrepancies

### 1. Neow exposure

- Rust parity models Neow as a real start-of-run decision surface.
- Python training still defaults to `skip_neow=True`.

Relevant files:

- [packages/engine-rs/src/decision.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/decision.rs:1)
- [packages/engine/game.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/game.py:382)
- [packages/training/gym_env.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/training/gym_env.py:100)

Why it matters:

- if bots are supposed to learn the full run-opening distribution, training should not silently start after Neow
- if we intentionally skip Neow in a curriculum, that should be a named training policy, not a hidden default

### 2. Observation split

- Rust canonical observation exposes a `480`-dimensional run observation with a `260` state slice plus `220` action slice.
- Python training still uses the compact `ObservationEncoder` and expects `260`-dimensional observations.

Relevant files:

- [packages/engine-rs/src/obs.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/obs.rs:20)
- [packages/engine/rl_observations.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/rl_observations.py:1)
- [packages/training/inference_server.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/training/inference_server.py:890)

Why it matters:

- this is fine if deliberate, but it must be versioned and documented so training metrics are interpretable

### 3. Action restriction abstraction is missing

Base legality is currently engine-owned, which is correct. What is missing is a train-time overlay for curriculum or experiment rules.

Examples the user explicitly wants:

- no card rewards
- restricted Neow choices
- no boss relic picks
- phase-specific evaluation rules

Recommended abstraction:

- `ActionRestriction`
- `CurriculumPolicy`

Where it should live:

- above engine legality
- at the training/evaluation boundary where actions are enumerated and masked

Suggested plumbing points:

- [packages/engine/game.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/game.py:2176)
- [packages/engine/rl_masks.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/rl_masks.py:139)
- [packages/training/gym_env.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/training/gym_env.py:220)

This should not be baked into Rust legality. Rust should keep modeling the real game plus documented intentional deviations.

### 4. Provenance and manifests

The current training stack logs status and metrics, but it still lacks a canonical run manifest with:

- git SHA
- branch
- dirty state
- engine schema version
- observation schema version
- training config hash

Relevant files:

- [packages/training/training_runner.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/training/training_runner.py:211)
- [scripts/push_metrics.py](/Users/jackswitzer/Desktop/SlayTheSpireRL/scripts/push_metrics.py:34)

## Current Good Abstractions

These are the abstractions that already look worth keeping:

- Rust canonical legality in `get_legal_actions()`
- Rust canonical run/event/reward state machine
- PyO3 wrapper for direct Rust access
- deterministic Rust search harnesses
- Python-side `StsEnv` / `GameRunner` boundary for experiment orchestration

## Recommended Next Training Plan

1. Add a canonical run manifest.
2. Add a training-side `ActionRestriction` / `CurriculumPolicy` layer.
3. Decide whether training should expose Neow by default.
4. Version the observation contract explicitly.
5. Decide whether long-term search should stay Python-native or move closer to Rust.
