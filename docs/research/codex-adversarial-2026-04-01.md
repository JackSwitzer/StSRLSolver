# Codex Adversarial Review — 2026-04-01 (post-parity suite)

## State: 1686 passed, 0 failed, 1 ignored

## Findings (10 total: 2 critical, 6 high, 2 medium)

### Critical
1. **Power cards are inert after play** — install_power is a short allowlist. Barricade, Demon Form, Noxious Fumes, A Thousand Cuts, Infinite Blades, Well-Laid Plans, Corruption all no-ops.
2. **Double Tap, Burst, Echo Form not wired** — Java replays cards, Rust ignores them.

### High  
3. **After Image ordering** — Rust fires before card effects; Java queues it after card's own actions in the action queue (subtle ordering difference).
4. **Delayed-turn tags dead** — next_turn_block, next_turn_energy, draw_next_turn, retain_hand, no_draw never consumed at turn start/end.
5. **Enemy move effects dropped** — narrow whitelist, many effects (Hex, Wounds, Constricted, debuff removal, Heart buffs, upgraded Burns, draw reduction) never consumed. Key mismatch: burn_upgrade vs burn+.
6. **Relic triggers write unused flags** — Bag of Preparation, Ring of the Snake, Pocketwatch, Damaru, Ink Bottle, Mummified Hand effects never consumed.
7. **Orange Pellets hardcoded debuff subset** — should clear ALL debuffs, not just 5.
8. **Run simulation far from Java** — no Neow, Watcher-only, ends after first boss, placeholder rewards.

### Medium
9. **Status key mismatches** — NoDraw vs "No Draw", LikeWater vs LikeWaterPower, Omega vs OmegaPower.
10. **TODO cards still approximations** — Conjure Blade, Omniscience, Wish.
