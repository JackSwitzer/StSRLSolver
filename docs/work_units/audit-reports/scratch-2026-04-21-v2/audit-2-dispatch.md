# Audit 2 — D70 / D89 / D111 dispatch closure

**Status:** CLEAN
**Rows audited:** D70, D89, D111
**Rust test count:** 2327 / 0 / 10 ignored (matches expected)

## Findings

1. **Full engine test suite** — CLEAN
   `./scripts/test_engine_rs.sh test --lib -- --nocapture` → `test result: ok. 2327 passed; 0 failed; 10 ignored; 0 measured; 0 filtered out; finished in 44.14s`. No regressions.

2. **Dead helpers untouched (intentional)** — CLEAN
   Helper functions still exist in `packages/engine-rs/src/powers/buffs.rs`:
   - `process_start_of_turn` at line 620
   - `process_end_of_turn` at line 704
   - `process_end_of_round` at line 752
   Re-exported from `packages/engine-rs/src/powers/mod.rs:87-89`.
   `rg process_start_of_turn|process_end_of_turn|process_end_of_round packages/engine-rs/src/engine.rs` → **zero matches**. Only referenced from `powers/buffs.rs` test module, `tests/test_cards_defect.rs`, `tests/test_powers_dispatch_wired.rs` doc/comments. Kept for future D83 work per subagent report — matches expectation.

3. **D70 Equilibrium** — CLEAN
   `packages/engine-rs/src/powers/defs/turn_end.rs:187-201`:
   ```
   static EQUILIBRIUM_TRIGGERS: ... = [TriggeredEffect {
       trigger: Trigger::TurnEnd,
       condition: Always,
       effects: &EQUILIBRIUM_EFFECTS,  // AddStatus(Player, EQUILIBRIUM, Fixed(-1))
       ...
   }];
   pub static DEF_EQUILIBRIUM: EntityDef = { triggers: &EQUILIBRIUM_TRIGGERS, status_guard: Some(sid::EQUILIBRIUM), ... };
   ```
   Effect at line 181-185: `AddStatus(Target::Player, sid::EQUILIBRIUM, AmountSource::Fixed(-1))`. Matches spec.

4. **D89 Fasting** — CLEAN
   `packages/engine-rs/src/powers/defs/turn_start.rs:525-551`:
   - `FASTING_TRIGGERS` (line 525) uses `Trigger::TurnStart`
   - `hook_fasting` (line 532-542): `engine.state.energy = (engine.state.energy - fasting).max(0)` — drains player energy equal to FASTING stacks
   - `DEF_FASTING` (line 544): `complex_hook: Some(hook_fasting)`, `status_guard: Some(sid::FASTING)`. Matches spec.

5. **D111 post-draw timing** — CLEAN
   All three powers on `Trigger::TurnStartPostDraw` in `packages/engine-rs/src/powers/defs/turn_start.rs`:
   - `DEF_DEMON_FORM` trigger at line 26 (def at line 32)
   - `DEF_NOXIOUS_FUMES` trigger at line 53 (def at line 59)
   - `DEF_BRUTALITY` trigger at line 83 (def at line 89)
   Guarded by `test_post_draw_turn_start_defs_have_correct_trigger` (line 600-609). `Berserk`/`InfiniteBlades`/etc. correctly remain on `TurnStart` (line 107, guarded by `test_pre_draw_turn_start_defs_have_correct_trigger` line 583-597).

6. **Integration tests green** — CLEAN
   `./scripts/test_engine_rs.sh test --lib test_powers_dispatch_wired` → `6 passed; 0 failed; 0 ignored`:
   - `regeneration_heals_at_end_of_turn`
   - `metallicize_block_applies_at_end_of_turn`
   - `combust_damage_fires_at_end_of_turn`
   - `equilibrium_decrements_at_end_of_turn`
   - `fasting_drains_energy_at_player_start_of_turn`
   - `demon_form_grants_strength_at_start_of_turn`

7. **Register correctness** — CLEAN
   `docs/work_units/parity-deviations-register.md`:
   - D70 (line 133): **closed**, cites Cycle 3 / `f74dbaa1`, PowerDef path + test name
   - D89 (line 152): **closed**, same SHA + test name
   - D111 (line 174): **closed**, same SHA, documents DrawCardNextTurn (D83) carve-out as still open

## Recommendation

**ship** — all closure evidence verified, no regressions, 2327/0/10 unchanged.
