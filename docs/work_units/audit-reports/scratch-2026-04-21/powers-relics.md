# Powers + Relics Trigger-Hook Audit (PR #138 pre-merge)

Scope: WHEN each power / relic fires — wrong-phase, wrong-frequency, stack
semantics, expiry, conditional triggers, power+relic interaction, on-attack vs
on-attacked polarity. Damage pipeline numbers are out of scope (separate audit).

Read-only. Findings only — no fixes. P0 = combat-incorrect, P1 = edge/rare, P2 = cosmetic/coverage gap.

Cross-ref shorthand:
- `P#` = entry in `docs/work_units/audit-reports/powers-buffs-debuffs.md`
- `R#` = entry in `docs/work_units/audit-reports/relics.md`
- `D###` = entry in `docs/work_units/parity-deviations-register.md`

---

## 1. Summary (P0 band)

| # | Power / Relic          | Severity | One-liner                                                                                           |
|---|------------------------|----------|-----------------------------------------------------------------------------------------------------|
| 1 | Flight                 | P1-NEW   | Rust decrements per-hit but never restores `storedAmount` at start-of-turn; Byrd/Darkling permanently degrade across turns. |
| 2 | Buffer                 | P1       | Rust decrements Buffer on *every* enemy hit regardless of incoming damage; Java guards `damageAmount > 0`.  (= existing P11) |
| 3 | Poison (player-owned)  | P0       | Rust ticks at end of player turn; Java `atStartOfTurn`.  (= D55 / D79, tracked)                     |
| 4 | Deva Form              | P0       | Rust pre-increments before granting energy; Java grants current then increments.  (= D123)          |
| 5 | Malleable              | P1       | No reset at start of enemy turn; escalation leaks across turns.  (= existing P1 in powers audit)    |
| 6 | Thorns (player)        | P1       | `combat_hooks.rs` applies Thorns to all hits — missing Java HP_LOSS/THORNS type exclusion (= P4)     |
| 7 | Sadistic Nature        | P1       | Rust triggers on every applied debuff; Java excludes self-inflicted / internal tags (= P3)          |
| 8 | Flame Barrier timing   | P2-NOTE  | Rust removes at next player `process_start_of_turn`; matches Java owner-atStartOfTurn. Contrary to P18 claim — P18 needs re-examination. |

Novel (not in Stage B / register): #1 Flight.
All others already tracked — re-listed here to confirm scope of PR #138's impact.

---

## 2. Power trigger deviations

### P0 band (combat-incorrect)

#### Player poison fires at end-of-player-turn (re-confirmation of D55/D79)
- Rust: `engine.rs:1529-1541` runs `tick_poison(&mut state.player)` inside the
  player-turn close-out sequence (TurnEnd → orb passives → TurnEndPostOrbs →
  poison tick). Also enemy poison tick in `combat_hooks.rs:16-115` fires at start
  of *enemy turn* (this part is correct).
- Java: `PoisonPower.atStartOfTurn` — fires at the owner's own start-of-turn.
  For the *player* that means after the enemy turn block, not before it.
- Effect: Rust applies 1 extra tick of damage on Turn 1 (because `process_end_of_turn`
  runs before first enemy turn even begins), and off-by-one across future turns.
- Register: D55 (consolidated D79). Already tracked, flagged again because PR #138
  does not address it and it directly interacts with Catalyst / Noxious Fumes tests.

#### Deva Form grants pre-incremented energy (= D123)
- Rust: `powers/defs/turn_start.rs` `apply_deva_form` increments DEVA_FORM stack
  before returning energy amount; result is "grant stack+1 energy then +1".
- Java: `DevaFormPower.atStartOfTurn` grants `amount` energy first (via
  `GainEnergyAction(this.amount)`), then stacks Strength by 1 using `ApplyPower`
  queued after.
- Impact: First-turn Deva yields 2 energy in Rust vs 1 in Java. Downstream
  strategic tests fail.
- Register: D123. Tracked.

### P1 band (edge/rare)

#### Flight: no atStartOfTurn restoration of storedAmount [NOVEL]
- Rust: `engine.rs:2479-2485` decrements `sid::FLIGHT` on every hit landed on
  the enemy. No corresponding call in `start_player_turn` or in any enemy
  TurnStart hook restores the original value. `sid::FLIGHT` is a raw counter —
  no shadow `stored_amount` concept exists.
- Java: `decompiled/java-src/com/megacrit/cardcrawl/powers/FlightPower.java:21-32`
  stores `this.storedAmount = amount` in the constructor, and
  `FlightPower.java:44-48` `atStartOfTurn { this.amount = this.storedAmount; }`.
  So the halve-damage counter refreshes to full every enemy turn.
- Impact: After Turn 1, Byrd / BanditPointy / any Flight enemy loses damage
  mitigation they should retain across the fight. Against Byrds this is a ~2x
  damage upside for the player — not combat-neutral.
- Also: the Java `onAttacked` reduction guards `info.type != HP_LOSS && info.type
  != THORNS && damageAmount > 0 && willLive`. Rust decrements on *every* hit
  including thorns-style retaliation and lethal blows. Compounds the drift.
- Not present in Stage B powers audit nor in parity-deviations-register.
- Register action needed: new entry (call it D162+) — or reuse an existing
  pattern ID if PR #138 touches this file.

#### Buffer: no damageAmount > 0 guard (= P11)
- Rust: `combat_hooks.rs:160-166`
  ```
  let buffer = engine.state.player.status(sid::BUFFER);
  if buffer > 0 {
      engine.state.player.set_status(sid::BUFFER, buffer - 1);
      continue;
  }
  ```
  Unconditional — any hit at all triggers decrement.
- Java: `BufferPower.java:40-46` `onAttackedToChangeDamage(info, damageAmount)`
  guards `if (damageAmount > 0) { ... ReducePowerAction ... }`.
- Impact: 0-damage attacks (Debuff-only moves, dagger-spray-into-intangible,
  0-strength scaling cases) eat a Buffer stack.
- Audit: already captured as P11 in `powers-buffs-debuffs.md`.

#### Malleable: no start-of-enemy-turn reset (= existing Stage B finding)
- Rust: `engine.rs:2520-2527` increments MALLEABLE by 1 for every hit taken by
  an enemy with MALLEABLE > 0 but no reset. Once Hexaghost bumps MALLEABLE to 7
  in fight 1, a second encounter (or the next enemy turn) still shows 7.
- Java: `MalleablePower.atStartOfTurn` sets `this.amount = this.base`.
- Stage B: listed.
- Mentioned here to confirm PR #138 does not touch it.

#### Thorns: missing type filter for HP_LOSS / THORNS (= P4)
- Rust: `combat_hooks.rs:239-251` deals thorns damage per enemy hit, no
  `damageAmount > 0` guard and no damage-type filter.
- Java: `ThornsPower.onAttacked` excludes `HP_LOSS` and `THORNS` damage types
  and wraps in `damageAmount > 0`.
- Rust partially saved by the call-site being the NORMAL damage path; but player
  poison tick counts as HP_LOSS through a separate code path so this case does
  not actually leak. Still, the missing explicit guard is a foot-gun for future
  refactors.

#### Sadistic Nature: triggers on every applied debuff (= P3)
- Rust: `powers/debuffs.rs:215-227` `apply_debuff_with_sadistic` — single gate
  is "was debuff applied". No damage-type / source filters.
- Java: `SadisticNaturePower.onApplyPower` excludes internal-tag debuffs
  (Shackled, Gone, etc.) and requires `power.owner == info.owner`.
- P3 in powers audit.

### P2 band (cosmetic / coverage)

#### Flame Barrier end-of-turn timing — possibly correct, contradicts P18
- Rust: `buffs.rs:628` calls `remove_flame_barrier(entity)` inside
  `process_start_of_turn`. So Flame Barrier's +block/+retaliate effect persists
  through the whole enemy turn and is cleared at the player's *next* start-of-turn.
- Java: `FlameBarrierPower` declares owner = player and uses `atStartOfTurn` to
  schedule `RemoveSpecificPowerAction`. On the player, that fires at the
  player's next atStartOfTurn, i.e. same phase Rust picks.
- Stage B P18 suggests this is incorrect. Re-examine: I believe P18 is mis-filed
  or the symptom is different (stored amount vs removal timing). PR #138 should
  not be blocked on this — downgrade or rewrite P18.

---

## 3. Relic trigger deviations

All relics below verified by reading the EntityDef registry
(`packages/engine-rs/src/relics/defs/*.rs`) + runtime dispatch hooks in
`combat_hooks.rs` and `engine.rs::start_player_turn / end_turn`.

### Clean in this pass (no new findings)

| Relic               | Trigger phases cross-checked                                        | Verdict       |
|---------------------|---------------------------------------------------------------------|---------------|
| Kunai               | OnAttackPlayed counter, TurnStart reset                             | correct       |
| Shuriken            | OnAttackPlayed counter, TurnStart reset                             | correct       |
| Ornamental Fan      | OnAttackPlayed counter, TurnStart reset                             | correct       |
| Pen Nib             | OnAttackPlayed counter (10), no reset (correct — persistent)        | correct       |
| Horn Cleat          | TurnStart effect-state countdown (turn 2 block)                     | correct       |
| Captain's Wheel     | TurnStart effect-state countdown (turn 3 block)                     | correct       |
| Velvet Choker       | OnAnyCardPlayed counter, TurnStart reset                            | trigger-phase correct — see R2 for separate +energy bug |
| Vajra               | CombatStart strength                                                | correct       |
| Pure Water          | CombatStart / TurnStart (Miracle draw)                              | correct       |

### Already tracked (re-listed)

- R1 **Holy Water** — Miracle generation timing.  Tracked.
- R2 **Velvet Choker** — separate energy-grant arithmetic bug.  Tracked.
- R3 **Philosopher's Stone** — enemy strength buff phase.  Tracked.
- R4 **Runic Dome** — blocks card intent visibility + triggers UI hooks in wrong place.  Tracked.
- R5 **Damaru** — Mantra timing on combat entry.  Tracked.

No novel relic-trigger findings surfaced in this pass.

---

## 4. Hook coverage gaps

| Java hook              | Rust Trigger equivalent            | Gap                                                                                                        |
|------------------------|------------------------------------|------------------------------------------------------------------------------------------------------------|
| `atEndOfTurnPreEndTurnCards` | (none)                         | Rust merges into `TurnEnd`. Affects cards using `EndTurnCards` semantics — no current power uses it in the card corpus, so coverage is latent. P2. |
| `onSpecificTrigger`    | (none — bespoke per power)         | Each power implements its own trigger call path instead of a generic bus. Acceptable architecturally; flag if a future power needs cross-power dispatch. |
| `atEnergyGain`         | (none)                             | Used by a couple of Watcher / Defect powers. Verify no WIP uses of this. P2. |
| `atDamageFinalReceive` | Inlined into `damage::calculate_incoming_damage` | Not a gap, but note: any new mitigation power must add itself to the damage calc call site, not to a trigger registry. Architectural drift risk. |
| `wasHPLost`            | (none)                             | Used by some relics (Meat on the Bone, etc.). Verify existing handling is in `player_lose_hp` call sites. P2. |
| Enemy `justApplied` on non-Weak/Vuln/Frail debuffs | only 3 statuses wired | `debuffs.rs:63-73` `just_applied_flag_for` whitelists Weak/Vuln/Frail. Java's `justApplied` is generic on `AbstractPower`. If any enemy ever applies Poison/Lock-On/etc. with end-of-round semantics, Rust will decrement one tick early. Currently no enemy does this in the corpus, so latent. P2. |

---

## 5. Untested power/relic interactions

High-risk pairings currently lacking a dedicated parity test:

1. **Flight + Thorns + Torii** — Flight halves, Torii clamps: order matters. No test covers Byrd hitting a Thorns-Torii player.
2. **Buffer + Intangible** — both negate; Buffer should *not* consume against a hit that Intangible already capped to 1. Needs test.
3. **Malleable + Curl Up (shared enemy)** — Awakened One Phase 2 has both at times; Rust decrements Curl Up once on first hit, then Malleable escalates; Java order differs. No parity test.
4. **Sadistic Nature + Envenom + applied-by-poison-on-hit** — chained debuff-on-debuff trigger stacking. Needs test.
5. **Buffer + Invincible (Heart)** — Heart Phase 1 has Invincible cap on damage taken. Buffer consumption should happen *before* invincible cap calc. Verify ordering in `combat_hooks.rs:160-167` vs `apply_invincible_cap_tracked`.
6. **Flight decrement + lethal-hit exemption** — Java explicitly guards `willLive`. Rust decrements on lethal; irrelevant when enemy dies, but matters if a Feed/exhaust-resurrect mechanic revives. Very edge.
7. **Deva Form + Energized potion on Turn 1** — stacking order of "grant N then increment" vs Rust's "increment then grant N+1" compounds any extra-energy source.
8. **Static Discharge + Plated Armor + Thorns** — all three proc on same unblocked hit; Rust orders Plated Armor → Static Discharge → Thorns/Flame Barrier. Java orders differently via priority system. No test verifies ordering.

---

## 6. Recommended register deltas (informational, not in-scope)

- **NEW**: Flight atStartOfTurn reset — P1 — affects Byrds/BanditPointy/
  Darkling. Single-line fix: EntityDef TriggeredEffect on TurnStart for
  enemies with FLIGHT status to set to storedAmount (requires adding
  `FLIGHT_STORED` sibling status or an effect-state slot). NOT in PR #138
  scope.
- **Re-examine**: P18 (Flame Barrier timing) — likely mis-filed; verify.
- **Confirmation**: PR #138 does not fix any of the above. It touches
  [verify — outside audit scope].

---

## 7. Overall posture for PR #138

This audit finds **1 novel P1 issue** (Flight no atStartOfTurn reset) and
**0 novel P0 issues**. All other deviations in the power/relic trigger
phase space are already tracked in Stage B reports or the deviation register.
PR #138's merge readiness on this axis depends on whether its scope includes
any of P3/P4/P11/P18/R1-R5 — which is outside this sub-audit to determine.

File:line citations above; verify if merging this audit into an actionable
ticket.
