# Ultra-Granular Work Units: Rewards

## Canonical status (2026-02-22)
- [x] `RWD-001` Route runner reward action emission through `RewardHandler.get_available_actions`.
- [x] `RWD-002` Route runner reward claim/skip execution through `RewardHandler.handle_action`.
- [x] `RWD-003` Enforce proceed gating for unresolved mandatory rewards.
- [x] `RWD-004` Support indexed relic claims for Black Star second relic rewards.

## Current behavior guarantees
- Reward action dicts are emitted from canonical handler actions.
- Reward claim/skip execution returns structured success/error payloads.
- `proceed_from_rewards` is blocked until all mandatory rewards are resolved.
- `claim_relic` supports indexed reward slots (`relic_reward_index`), including `second_relic`.
- Selection-required relic reward flows expose explicit `select_cards` follow-up actions.

## Remaining reward-adjacent follow-ups
- Combat statistics currently pass placeholder `enemies_killed=1` into reward generation:
  - `packages/engine/game.py:3285`
- Cross-domain parity dependencies remain in powers/orbs/cards tracks, not in core reward action plumbing.

## Source of truth
- Domain audit: `docs/audits/2026-02-22-full-game-parity/domains/rewards.md`
- Campaign baseline and next steps: `docs/audits/2026-02-22-full-game-parity/GROUND_TRUTH.md`
