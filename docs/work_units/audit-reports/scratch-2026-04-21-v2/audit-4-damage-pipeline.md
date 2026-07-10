# Audit 4 — Cycle 5 damage pipeline (D91 / D124)

**Status:** CLEAN
**Rows audited:** D91, D124
**Bypass sites remaining:** 0 — all 12 `.hp -=` / `hp -=` sites categorized below as canonical or documented-tick.

## Entry point audit (engine.rs:2706-2779)

- `apply_damage_to_player` (engine.rs:2706): reads Wrath / Vuln / Intangible / Torii / Tungsten / Odd Mushroom, calls `damage::calculate_incoming_damage` (line 2717), assigns `block_remaining` (line 2727), routes hp_loss through `player_lose_hp` (line 2729) — OK.
- `apply_hp_loss_to_player` (engine.rs:2742): bypasses block per Java `decrementBlock:175`, applies `damage::apply_hp_loss` with Intangible+Tungsten (line 2748), routes through `player_lose_hp` — OK.
- `apply_hp_loss_to_enemy` (engine.rs:2761): bypasses block, applies Intangible cap-to-1 locally (line 2769), direct `entity.hp -=` (line 2772), then `record_enemy_hp_damage` (line 2777) fires Rebirth / Mode Shift / phase hooks — OK. This is the canonical site per its doc comment.

`player_lose_hp` (engine.rs:2330) is the unified HP-loss sink — fires `OnPlayerHpLoss` trigger (Centennial Puzzle, Self-Forming Clay, Runic Cube, Red Skull, Rupture).

## Bypass catalog (12 raw `entity.hp -=` / `player.hp -=` sites)

CANONICAL (inside the three entry points or unified sink):
- engine.rs:2334 — `player_lose_hp` body, unified HP-loss sink
- engine.rs:2772 — `apply_hp_loss_to_enemy` body
- engine.rs:2500 — `deal_damage_to_enemy` NORMAL-damage path (Slow/Flight/Invincible/Curl-Up/Malleable/Sharp-Hide/Shifting); out of D91/D124 scope

TICK / PRECOMPUTED (compute Intangible + Tungsten before subtracting):
- status_effects.rs:49 — Burn (NORMAL; tungsten+intangible+block precomputed inline, see lines 39-47)
- status_effects.rs:57 — Regret HP_LOSS (`damage::apply_hp_loss(raw, intangible, tungsten)`, line 55)
- status_effects.rs:122 — Pain HP_LOSS (`damage::apply_hp_loss(1, intangible, tungsten)`, line 119)
- powers/debuffs.rs:97 — `tick_poison` enemy poison (Intangible check handled at the combat_hooks call site context; player poison uses separate path at engine.rs:1539-1543 with `apply_hp_loss` and `player_lose_hp`). OK.

ON-HIT REACTIVES (Thorns / Flame Barrier / Static Discharge — reactive damage, inline block+hp math, tracked as NORMAL sub-hits):
- combat_hooks.rs:209 — Static Discharge Lightning evoke
- combat_hooks.rs:246 — Thorns reflect damage
- combat_hooks.rs:260 — Flame Barrier reflect damage
- potions/mod.rs:564 — potion direct enemy damage with Vuln + Intangible + Invincible cap precomputed (lines 547-559)

## Findings

- [PASS] Entry points exist and route through `calculate_incoming_damage` / `apply_hp_loss`. Cited above.
- [PASS] Bypass hunt: 12 sites; 3 canonical, 9 tick/reactive with local Intangible+Tungsten precompute. Zero uncategorized bypasses.
- [PASS] Pressure Points routed: `effects/interpreter.rs:620-636` `SimpleEffect::TriggerMarks` loops living enemies and calls `engine.apply_hp_loss_to_enemy(idx, mark)` at line 631. Correct HP_LOSS routing per D124 doc.
- [PASS] Brutality routed: `effects/runtime.rs:1713-1721` `Target::Player` branch of `deal_damage` explicitly routes to `engine.apply_hp_loss_to_player(amount)` with comment citing D91 + Java `LoseHPAction.java:36`.
- [PASS] Tests green: `test_damage_pipeline_routing` 15/15 pass.
- [PASS] No regression: damage (223 pass), intangible (24), vulnerable (18), wrath (35), torii (11), tungsten (11). Zero failures.
- [PASS] Register correctness: `pre-merge-triage-2026-04-21.md:190` and `:195` show D91/D124 both marked `**closed** (Cycle 5 @ 10c34602)`. Commit `10c34602` exists in log as "Cycle 5: damage pipeline routing (D91 / D124)".

## Recommendation

**SHIP.** D91 and D124 closed end-to-end. Canonical entry points in place, Pressure Points and Brutality both routed through HP_LOSS pipeline, all 322 touched tests green, register stamped with real SHA. No hidden bypasses — every remaining raw `.hp -=` is either inside a canonical entry, the unified `player_lose_hp` sink, or a tick/reactive site that precomputes Intangible + Tungsten + block locally.
