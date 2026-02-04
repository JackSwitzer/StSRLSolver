# Potion Behavior Completion - Work Units

## Scope summary
- Complete potion effects/usage to match Java behavior (combat + out of combat).
- Add missing choice flows, RNG parity, and auto-trigger/restrictions.
- Keep potion data and drop prediction logic as-is.

## Missing/partial behaviors
- Discovery potions (Attack/Skill/Power/Colorless): choose 1 of 3, use card RNG, Sacred Bark adds 2 copies, cost 0 this turn.
- Distilled Chaos: should play top N cards (3/6) for free with proper targeting/triggers; not just draw.
- Liquid Memories: choose from discard (1/2), set cost 0 for turn, handle empty pile/hand limit.
- Entropic Brew: use class potion pool + potionRng, respect Sozu, fill empty slots only, usable out of combat.
- Fairy Potion: auto-trigger on death, heal to % max HP, consume potion, Sacred Bark doubles; not manual use.
- Gambler's Brew / Elixir / Stance Potion: choice-driven discard/exhaust/stance selection.
- Snecko Oil: randomize hand costs via cardRandomRng; affect cost-for-turn only.
- Smoke Bomb: cannot use vs bosses or BackAttack; set escape and suppress rewards.
- Potion targeting/actions: hard-coded targets are wrong/incomplete; use potion metadata; onUsePotion relic hooks not fired.

## Task batches (unit-sized)
1) Choice/selection infrastructure
- Add pending choice state + resolver for potion selections (discard/exhaust/stance/discovery).
Acceptance: potion use can pause for choice and resolves deterministically; no effect applied before resolution.

2) Discovery potions
- Implement choose-1-of-3 pools + RNG selection; add chosen card(s) at cost 0 this turn; Sacred Bark adds 2 copies.
Acceptance: 3 unique options offered, selection adds correct card(s), RNG stream advances reproducibly.

3) Distilled Chaos auto-play
- Play top N cards via `EffectExecutor` for free; handle enemy targets using random valid enemy.
Acceptance: N cards removed from draw pile, on-play triggers fire, target selection works when needed.

4) Discard/exhaust/return utilities
- Gambler's Brew discard selection then draw same count.
- Elixir exhaust selection.
- Liquid Memories pick discard card(s) and set cost 0.
Acceptance: respects hand limit, empty discard, Sacred Bark doubles Liquid Memories.

5) Auto-trigger + restrictions + RNG parity
- Fairy Potion auto-trigger on death and consume potion; apply max HP % heal.
- Smoke Bomb gating (boss/BackAttack) and escape state effects.
- Entropic Brew uses potionRng + class pool; Sozu check; out-of-combat use.
- Snecko Oil uses cardRandomRng and cost-for-turn only.
Acceptance: each restriction enforced, RNG is deterministic (no `random` module), correct effects applied.

6) Targeting + relic hooks
- Use `PotionTargetType` for action generation and targeting; fix thrown/AoE cases.
- Trigger onUsePotion relic hooks (Toy Ornithopter).
Acceptance: available potion actions reflect correct targets; relics react to potion use.

## Files to touch
- `packages/engine/registry/potions.py`
- `packages/engine/registry/__init__.py`
- `packages/engine/handlers/combat.py`
- `packages/engine/state/combat.py`
- `packages/engine/effects/executor.py`
- `packages/engine/content/potions.py`
- `packages/engine/state/game_rng.py` and/or `packages/engine/state/rng.py`
- `tests/test_potion_effects_full.py`
- `tests/test_potion_registry.py`
- `tests/test_potion_sacred_bark.py`
