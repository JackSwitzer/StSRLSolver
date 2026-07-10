# Audit 7 — combat_hooks.rs integrity post-Cycle 4/5

**Status:** CLEAN
**Test count:** 12 pass / 0 fail (5 combat_hooks_integration + 5 debuff_timing_parity indirect + 2 intent)

## Double-apply audit

Only 3 debuff handlers in `combat_hooks.rs` use the `apply_debuff_from_enemy`
path (Weak/Vuln/Frail). Each fires exactly once with no subsequent
`add_status`/`set_status` on the same status in the same arm:

- `mfx::WEAK` (line 287-289): single `apply_debuff_from_enemy` call -> OK
- `mfx::VULNERABLE` (line 290-292): single `apply_debuff_from_enemy` call -> OK
- `mfx::FRAIL` (line 293-295): single `apply_debuff_from_enemy` call -> OK

Non-debuff enemy effects (STRENGTH, RITUAL, ENTANGLE, SIPHON_STR/DEX,
CONSTRICT, HEX, BURN, WOUND, etc. lines 296-483) correctly use direct
`add_status`/`set_status` — no justApplied semantics apply to these and no
duplicate application occurs. Each arm makes exactly one mutation.

## WrithingMass arm (lines 574-586)

- Guarded on `enemy.hp > 0` (line 581): OK
- Calls `enemies::writhing_mass_reactive_reroll(&mut enemy)` on index
  `enemy_idx` only (line 582-584): OK
- No mutation of other enemy indices in the arm: OK
- Matches on `"WrithingMass"` string literal consistent with `create_enemy`
  id: OK

## sentry_fix_first_moves (engine.rs:246)

- Called in `start_combat` AFTER `self.state.enemies` is populated by caller
  (run.rs:1250 passes pre-built enemy vector into engine state before
  `start_combat`): OK
- Only mutates entries where `enemy.id == "Sentry"` and
  `move_history.is_empty()` (act1.rs:426-434): non-Sentry move_ids untouched,
  OK
- `iter_mut().enumerate()` over slice — empty slice is a no-op, 3-Sentry
  slice indexes 0/1/2 correctly alternate BEAM/BOLT/BEAM: no panic risk, OK

## New-PowerDef dispatch path

- DEF_EQUILIBRIUM registered at: `powers/defs/turn_end.rs:194` (declaration),
  registered in both `ALL_POWER_DEFS` (`powers/defs/mod.rs:65`) and
  `RUNTIME_PLAYER_POWER_DEFS` (`powers/defs/mod.rs:135`)
- DEF_FASTING registered at: `powers/defs/turn_start.rs:544` (declaration),
  registered in both `ALL_POWER_DEFS` (`powers/defs/mod.rs:56`) and
  `RUNTIME_PLAYER_POWER_DEFS` (`powers/defs/mod.rs:128`)
- Path: **runtime** — both fire through `effects::runtime` via the PowerDef
  system (Equilibrium has declarative `TurnEnd` effect; Fasting has
  `complex_hook = hook_fasting` at turn start). Neither touches
  `combat_hooks.rs` dispatch, which is correct (these are player-turn
  powers, not enemy intents or boss-damage hooks).

## New damage entry-points — event emission

- `apply_damage_to_player` (engine.rs:2706): routes through
  `damage::calculate_incoming_damage` + `player_lose_hp`.
  Emits: **`OnPlayerHpLoss`** (via `player_lose_hp` line 2345), and
  `player_lose_hp` fires owner-aware dispatch so Centennial Puzzle,
  Self-Forming Clay, Runic Cube, Red Skull react correctly.
- `apply_hp_loss_to_player` (engine.rs:2742): routes through
  `apply_hp_loss` + `player_lose_hp`.
  Emits: **`OnPlayerHpLoss`** (same path).
- `apply_hp_loss_to_enemy` (engine.rs:2761): routes through
  `record_enemy_hp_damage`, which calls `combat_hooks::on_enemy_damaged`
  (boss hooks: Guardian/Lagavulin/SlimeBoss/AwakenedOne/Champ/WrithingMass
  + Angry). No cross-entity Thorns event — that's correct since HP_LOSS
  damage to an enemy is not an "attack" in Java semantics.

Note: Thorns/Flame Barrier are inline per-hit in `execute_enemy_move`
(lines 240-264), which is correct — those trigger only on enemy attacks
on the player, not on general damage entry-points.

## Recommendation

**Ship.** No double-apply regressions. WrithingMass arm is well-guarded.
`sentry_fix_first_moves` is correctly sequenced and bounds-safe. PowerDef
registrations for Equilibrium and Fasting are complete in both ALL and
RUNTIME tables. Damage entry-points emit the correct events via the
standard `player_lose_hp` / `record_enemy_hp_damage` funnel. All targeted
tests pass.
