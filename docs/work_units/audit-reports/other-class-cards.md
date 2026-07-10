# Parity audit: Non-Watcher class cards (Ironclad, Silent, Defect, Colorless, Curse, Status)

**Date:** 2026-04-21
**Auditor:** Opus 4.7 (subagent)
**Scope:** Java `decompiled/java-src/com/megacrit/cardcrawl/cards/{red,green,blue,colorless,curses,status}/` vs Rust `packages/engine-rs/src/cards/{ironclad,silent,defect,colorless,curses,status}.rs`
**Cards counted:** 75 red (Java) -> 73 Rust (unified Strike/Defend), 75 green -> 74 Rust (unified + intentional drops), 76 blue -> 73 Rust (unified + intentional drops), 39 colorless -> 39 Rust, 14 curses -> 14 Rust, 5 status -> 5 Rust
**Deviations found:** 14

## Summary

Non-Watcher card coverage is effectively complete: every ID that Java's `CardLibrary.initialize()` actually loads is registered in Rust. The intentional Strike_{R,G,B,P}/Defend_{R,G,B,P} collapse to single "Strike"/"Defend" is isolated in `packages/engine-rs/src/cards/starters.rs` and only bites if the training pipeline ever needs per-class starters. Base damage/block/magic numbers for the sampled Ironclad, Silent, Defect, and Colorless cards match Java exactly, and the upgrade deltas I spot-checked (Searing Blow +4 on first upgrade, Rampage magic 5/+3 upgraded, Sentinel 5 block+2 energy/8+3, Biased Cognition +4 focus/-1 loss, Echo Form retain 1 copy, Bouncing Flask cardRandomRng-free, Demon Form, Corruption, Feel No Pain all align). The deviations fall in four buckets: (1) ID-string drift — `Crippling Cloud` Rust id vs `Crippling Poison` Java ID, `Daze` vs `Dazed`, `AscendersBane`/`CurseOfTheBell`/... lack the space Java canonically uses; (2) Defect orb damage routes through the NORMAL damage pipeline in `engine.rs:2471-2543`, which triggers Curl-Up/Malleable/Sharp Hide on Lightning/Dark damage that Java marks `DamageType.THORNS`; (3) Searing Blow only supports one upgrade and hardcodes +4 — Java's `4 + timesUpgraded` quadratic stack is not implemented; (4) two curse hooks are missing — Parasite's `onRemoveFromMasterDeck` (max HP -3 when removed from master deck) is unimplemented, and Pride's draw-pile copy-spawn uses the wrong trigger surface (`EndTurnInHand` fires on cards *still in hand*, but Pride `exhaust:true+innate` is never in hand at end of turn, so the copy never spawns). Status cards (Burn, Wound, Slimed, Void, Dazed) are otherwise faithful — Burn+ magic-4 scaling re-verified (closes D18), Void ethereal+LoseEnergy-on-draw wired, Wound unplayable. D58 (Static Discharge THORNS) is a specific manifestation of deviation OC2 below.

### Top 3 cross-class gaps (ranked by impact)

1. **Orb damage is NORMAL, not THORNS (OC2)** — every Lightning passive, Lightning evoke, and Dark evoke triggers Curl-Up, Malleable, and possibly Sharp Hide that Java's THORNS guard skips. Measurable Defect parity gap against any Louse/Slaver/Gremlin Nob/Guardian combat and corrupts the training signal for Defect runs.
2. **Searing Blow multi-upgrade formula missing (OC9)** — Armaments+/Apotheosis/Legendary Head/Dominate Potion can all stack multiple Searing Blow upgrades; Rust caps at +4 total regardless of times upgraded. Ironclad high-reward plays undervalued in training.
3. **Parasite `onRemoveFromMasterDeck` not wired (OC10)** — Removing Parasite through any event/campfire/shop yields no max-HP penalty. Silent/colorless/curse remove loops do not pay the 3 HP cost; economy is off any time Parasite is removed. Low frequency but structurally missing hook.

## Coverage matrix

| Class | Java count | Rust registrations | Unified/intentional drops | Real gaps |
|---|---|---|---|---|
| Red (Ironclad) | 75 (`red/*.java`) | 73 (`ironclad/*.rs` + unified `starters.rs`) | `Strike_Red`+`Defend_Red` collapsed | 0 missing IDs |
| Green (Silent) | 75 (`green/*.java`) | 74 (`silent/*.rs` + unified starters) | `Strike_Green`+`Defend_Green` collapsed; `Shiv` is a token registered in `silent/shiv.rs` (not a starter deck card) | 0 missing IDs (Crippling Poison renamed to Crippling Cloud — see OC1) |
| Blue (Defect) | 76 (`blue/*.java`) | 73 (`defect/*.rs` + unified starters) | `Strike_Blue`+`Defend_Blue` collapsed; `Impulse` intentionally removed (Java declares class but never registers in CardLibrary — see `content/generated-cards.txt:2060`) | 0 gameplay gaps |
| Colorless | 39 (`colorless/*.java`) | 39 (`colorless/*.rs`) | — | 0 missing IDs |
| Curses | 14 (`curses/*.java`) | 14 (`cards/curses.rs`) | — | Parasite + Pride hooks missing (OC10, OC11) |
| Status | 5 (`status/*.java`) | 5 (`cards/status.rs`) | — | Dazed id drift (OC3); Burn+ magic re-verified clean (closes D18) |

## Deviations

### OC1 — Crippling Cloud vs Crippling Poison (card ID drift)
- **Severity:** bug
- **Java:** `cards/green/CripplingPoison.java:14` — `public static final String ID = "Crippling Poison";`
- **Rust:** `packages/engine-rs/src/cards/silent/crippling_cloud.rs:6` — `id: "Crippling Cloud"`
- **Delta:** The canonical registry key differs. Any Java-facing data (corpus replays, save files, event referenceable by ID, decklist export) will look up `"Crippling Poison"` and miss the Rust registration. The name and stats are faithful (cost 2, 4 poison + 2 weak to all enemies, exhaust; +3 magic and +1 weak on upgrade), so gameplay is correct once the player owns the card. Impact: any persistence layer that round-trips the ID is broken; any Java→Rust test harness that imports Java IDs will need a translation shim.

### OC2 — Lightning/Dark orb damage treated as NORMAL instead of THORNS
- **Severity:** bug
- **Java:** `orbs/Lightning.java:65` (`DamageInfo(..., DamageInfo.DamageType.THORNS)` on evoke) + `orbs/Lightning.java:79` (passive also THORNS) + `orbs/Dark.java:57` (evoke THORNS); `powers/CurlUpPower.java:onAttacked` (`info.type == NORMAL` guard) + `powers/MalleablePower.java:onAttacked` (`info.type == NORMAL` guard)
- **Rust:** `packages/engine-rs/src/engine.rs:2726-2745` (`apply_evoke_effect`) and `engine.rs:2757-2768` (`apply_passive_effect`) both route Lightning and Dark damage through `deal_damage_to_enemy`. That path unconditionally fires Curl-Up (`engine.rs:2511-2517`), Malleable (`engine.rs:2521-2527`), Sharp Hide (`engine.rs:2530-2533`), and Shifting (`engine.rs:2536-2538`).
- **Delta:** Lightning passive/evoke and Dark evoke incorrectly pop Curl-Up charges (first hit on any Louse, Sentry, Mugger), stack Malleable (Guardian defensive mode), and deal Sharp Hide retaliation (Guardian). Shifting reaction is correct (Java Shifting fires on any damage). The existing comment at `engine.rs:2500-2506` explicitly claims orb damage does not reach this path, but it does. D58 (Static Discharge THORNS) is a specific manifestation; the bug is broader and covers every orb evoke/passive.

### OC3 — Dazed id drift ("Daze" vs "Dazed")
- **Severity:** bug
- **Java:** `cards/status/Dazed.java` — `public static final String ID = "Dazed";`
- **Rust:** `packages/engine-rs/src/cards/status.rs:18` — `id: "Daze"` / `name: "Daze"`
- **Delta:** Same issue as OC1. Already registered as D71/D85 in the deviation register; included here because it's a Status-card parity gap that external consumers (Java replay, ID-keyed saves) will notice. Gameplay is otherwise correct: ethereal set on line `runtime_meta.rs:144`, unplayable through `cost == -2`.

### OC4 — Searing Blow upgrade formula truncated to single-upgrade flat +4
- **Severity:** bug
- **Java:** `cards/red/SearingBlow.java:48-53` — `upgradeDamage(4 + this.timesUpgraded); ++this.timesUpgraded;` (infinitely upgradeable; upgrade N adds `4+N-1` = scaling 4, 5, 6, 7…). Base 12. Total after N upgrades = `12 + sum_{k=0..N-1}(4+k) = 12 + 4N + N(N-1)/2`.
- **Rust:** `packages/engine-rs/src/effects/hooks_damage.rs:89-95` (`hook_searing_blow`) — returns `+4` if `card_inst.flags & 0x04 != 0`, else `+0`. `cards/ironclad/searing_blow.rs` registers `Searing Blow` (dmg 12) and `Searing Blow+` (dmg 16) as the only two upgrade variants.
- **Delta:** Rust caps Searing Blow at `+1` upgrade with a flat +4. Armaments+/Apotheosis/Legendary Head/Dominate Potion/Smith/Duplicator chained upgrades are all undervalued. E.g., Java 3x-upgraded Searing Blow = `12+4+5+6 = 27`; Rust reads 16 regardless of upgrade count above 1.

### OC5 — Pride copy-to-draw-pile never fires
- **Severity:** bug
- **Java:** `cards/curses/Pride.java:24-27` — `triggerOnEndOfTurnForPlayingCard` (fires when the card was PLAYED this turn) schedules `MakeTempCardInDrawPileAction(this.makeStatEquivalentCopy(), 1, false, true)`
- **Rust:** `packages/engine-rs/src/cards/runtime_meta.rs:84-85` + `207` wires Pride to `EndTurnInHand(EndTurnHandRule::AddCopy)`. The dispatch in `status_effects.rs:67-69` only iterates over the current hand at end of turn: `for card_inst in &hand { ... if EndTurnInHand(AddCopy) => state.draw_pile.push(*card_inst); }`.
- **Delta:** Pride has `exhaust: true, innate: true` (curses.rs:87-92, runtime_meta.rs:110). It is drawn on turn 1, played, then exhausted — so it is never in hand at end of turn. The AddCopy trigger never fires; Pride never replicates. In Java this is the defining feature of the curse (keeps respawning via draw pile). Rust behaves as if Pride were a single-time innate curse that exhausts on play.

### OC6 — Parasite onRemoveFromMasterDeck not wired (max HP -3)
- **Severity:** bug
- **Java:** `cards/curses/Parasite.java:25-28` — `onRemoveFromMasterDeck` calls `AbstractDungeon.player.decreaseMaxHealth(3)` on removal.
- **Rust:** `packages/engine-rs/src/cards/curses.rs:80-85` registers Parasite with no effect_data/complex_hook. Grep for Parasite across the engine (`run.rs`, `events/exordium.rs`, `enemies/act3.rs` uses are unrelated spawn/tests). No `on_remove_from_master_deck` or equivalent hook exists.
- **Delta:** Any Purge/Purge Hammer/Empty Cage/Fairy Tale/Ascension 10+ remove action that removes Parasite from the master deck skips the 3 max-HP penalty. Economy is off anytime a run removes Parasite. Low frequency (Parasite is uncommon), but a structural missing hook.

### OC7 — Sharp Hide on-trigger surface is wrong
- **Severity:** bug
- **Java:** `powers/SharpHidePower.java:40-46` — `onUseCard(AbstractCard card, UseCardAction action)` fires when the player plays an Attack card: `if (card.type == AbstractCard.CardType.ATTACK) ... DamageAction(player, DamageInfo(owner, amount, THORNS))`. The trigger is on-play, not on-attacked.
- **Rust:** `packages/engine-rs/src/engine.rs:2530-2533` fires Sharp Hide when the enemy takes hp_damage from `deal_damage_to_enemy`.
- **Delta:** Java Guardian's Sharp Hide retaliates on every Attack card played (even if the card missed, or dealt 0 after block). Rust only retaliates when the attack actually dealt HP damage. Observable on Guardian defensive mode: playing Attack-type Shiv/Strike-variant that is blocked by the Guardian's own block in Java still hits the player with Sharp Hide; Rust skips. Impact: Guardian fights are structurally easier than Java.

### OC8 — Non-Watcher curse id strings differ from Java (cosmetic drift)
- **Severity:** unverified (save/persistence scope)
- **Java:** `cards/curses/*.java` IDs: `AscendersBane` (no space), `CurseOfTheBell` (no spaces), `Necronomicurse` — Java does use no-space IDs here, so Rust matches.
- **Rust:** `packages/engine-rs/src/cards/curses.rs:29, 46, 60` uses the same no-space IDs.
- **Delta:** None after checking — curse IDs line up. Leaving this row as `unverified`/"clean" so the audit log records the check. Follow-up: verify that the corpus generator and map/event RNG tables also use the same string-literal keys.

### OC9 — Defect `Impulse` removed by design
- **Severity:** intentional
- **Java:** `cards/blue/Impulse.java` — card class exists with `triggerOrbStartEnd` behavior.
- **Rust:** `packages/engine-rs/content/generated-cards.txt:2060-2066` — card removed with comment "Impulse has no localization entry and is NOT in CardLibrary.initialize(). Dead code — can never appear in gameplay. Removed for parity."
- **Delta:** This is correct; Java base 1.1 ships `Impulse.java` as dead code. Coverage gap surfaced only via file count; intentional and faithful to Java's actual runtime.

### OC10 — Blue/defect aliased card IDs are faithful to Java (not deviations)
- **Severity:** intentional (documented)
- **Java:** `cards/blue/Claw.java` has `ID = "Gash"`; `cards/blue/Recursion.java` has `ID = "Redo"`; `cards/blue/SteamBarrier.java` has `ID = "Steam"`; `cards/blue/Overclock.java` has `ID = "Steam Power"`; `cards/blue/Equilibrium.java` has `ID = "Undo"`; `cards/colorless/Apparition.java` has `ID = "Ghostly"`.
- **Rust:** `defect/gash.rs` uses `id: "Gash"`/`name: "Claw"`; `defect/redo.rs` `Redo`/`Recursion`; `defect/steam.rs` `Steam`/`Steam Barrier`; `defect/steam_power.rs` `Steam Power`/`Overclock`; `defect/undo.rs` `Undo`/`Equilibrium`; `colorless/ghostly.rs` `Ghostly`/`Apparition`.
- **Delta:** Zero — Rust ids match Java ids by design. Flagged here because a naive name-diff shows a mismatch that is actually parity.

### OC11 — Clumsy auto-exhaust handled via ethereal, not triggerOnEndOfPlayerTurn
- **Severity:** intentional
- **Java:** `cards/curses/Clumsy.java:24-25` — `isEthereal = true` plus an explicit `triggerOnEndOfPlayerTurn` that enqueues `ExhaustSpecificCardAction(this, hand)` on top.
- **Rust:** `cards/runtime_meta.rs:147` — Clumsy is in the ethereal set. End-of-turn ethereal exhaust is handled by the general ethereal loop (not Clumsy-specific).
- **Delta:** No behavioral divergence observed: in Java the ethereal flag alone would exhaust Clumsy at end of turn; the duplicated `triggerOnEndOfPlayerTurn` addToTop is defensive/redundant. Rust relies on the single ethereal path which is correct. Leaving as intentional; recorded for the audit trail.

### OC12 — Defect orb damage triggers enemy Slow multiplier (THORNS should skip it)
- **Severity:** bug
- **Java:** `powers/SlowPower.java` — `atDamageReceive(float damage, DamageType type) { if (type == NORMAL) return damage * (1 + amount * 0.1f); return damage; }`. THORNS damage passes through Slow untouched.
- **Rust:** `packages/engine-rs/src/engine.rs:2475-2476` — `deal_damage_to_enemy` unconditionally applies `powers::slow_damage_multiplier(&enemy.entity)` to all damage, including orb passive/evoke that Java marks THORNS. The orb path enters `deal_damage_to_enemy` (see OC2).
- **Delta:** On Lagavulin and Reptomancer fights (both have Slow gain-on-play-X-cards patterns that boost `atDamageReceive`) or any custom Slow-applying scenario, Lightning/Dark orbs incorrectly deal extra damage scaling with cards-played-this-turn. Same root cause as OC2 — routing orb damage through the NORMAL pipeline — but a separate observable outcome. Flight/Invincible halving-and-capping are not type-gated in Java and match Rust behavior.

### OC13 — Shiv token registration
- **Severity:** intentional (documented)
- **Java:** `Shiv.java` is the token card, not a starter deck card; registered dynamically by `Accuracy`/`InfiniteBlades`/`BladeDance`/`CloakAndDagger`/`Die Die Die`/`Storm of Steel` at runtime.
- **Rust:** `packages/engine-rs/src/cards/silent/shiv.rs` registers `Shiv` and `Shiv+` in the card pool via the standard registry. Token-spawning cards (`infinite_blades.rs`, `blade_dance.rs`, etc.) produce `Shiv` copies.
- **Delta:** Zero behavior gap. Registration mechanism differs from Java (dynamic class instantiation vs card registry lookup), but this is a pure architecture choice, and Shiv stats match (cost 0 unplayable-in-deck, 4 damage, 6 upgraded, exhaust).

### OC14 — Status-card `Wound` has no effect beyond unplayable+clutter
- **Severity:** intentional
- **Java:** `cards/status/Wound.java` — empty `use()` method, unplayable, cost -2.
- **Rust:** `packages/engine-rs/src/cards/status.rs:11-16` — empty effect_data, cost -2 → unplayable.
- **Delta:** Zero. Wound is pure deck dilution; Rust matches.

## Items verified clean (no deviation logged)

- **Strike/Defend unification** (`cards/starters.rs`): single ID per name, matches Java Strike_Red/Defend_Red base stats (6 dmg/5 block; 9/8 upgraded); collapse is an explicit architecture decision already documented in module header.
- **Ironclad spot-checks**: Corruption (cost 3/2 upgraded, exhaust removal for skills), Exhume (exhaust-pile retrieve; filter note logged previously as OC-adjacent but Java already allows self-pick), Demon Form (3 turn-start strength), Feel No Pain (3/4 block on exhaust), Dark Embrace (draw 1 on exhaust), Berserk (1 energy/turn + 2 vulnerable self, 1 vulnerable upgraded), Limit Break (double strength, exhaust).
- **Silent spot-checks**: Accuracy (+4/+6 shiv damage), Infinite Blades (innate upgrade, 1 shiv start of turn), Bullet Time (1 cost 0 all hand; no-draw), Deadly Poison (5/7 poison), Bouncing Flask (3 Poison, 3 hits, cardRandomRng use), Envenom.
- **Defect spot-checks**: Biased Cognition (4 focus/5 upgraded + 1/turn focus loss), Echo Form (ethereal upgrade, retain 1 copy, next card each turn repeats; Java-parity), Tempest (X-cost, channel X Lightning).
- **Colorless spot-checks**: Apotheosis (upgrade entire deck), Chrysalis (3 random powers into draw pile), The Bomb (3-turn delayed AoE 40/50 dmg), Mind Blast (innate, unplayable, X = energy dmg), Hand of Greed (20/25 dmg + 20/25 gold on kill).
- **Curse base stats**: Decay base magic 2, Regret (hp_loss = hand_size), Doubt (+1 Weak), Shame (+1 Frail), Pain (+1 HP loss per Pain card when any card played), Pride (cost 1, exhaust, innate) — curse stats match Java.
- **Burn end-of-turn magic scaling** (`status_effects.rs:36-42`): `raw = if card.base_magic > 0 { card.base_magic } else { 2 }`. Burn=2, Burn+=4 — matches Java. **Closes D18** (previously `unverified`).

## Follow-up questions

- **OC2 fix shape**: the existing D58 follow-up (Static Discharge THORNS) should be generalized to all orb passive/evoke calls — add a `deal_thorns_damage_to_enemy` method on CombatEngine that skips Curl-Up/Malleable/Sharp Hide (and applies Strength/Vulnerable already-skipped, matching Java), route Lightning/Dark/Static Discharge through it. Training impact is non-trivial for Defect.
- **OC4 fix shape**: add a new DamageModifier rule that reads `times_upgraded` from `CardInstance` and applies `4 + (times_upgraded - 1)` per upgrade iteratively. Requires tracking `times_upgraded > 1` on CardInstance (currently binary upgraded flag). Low-frequency but important for advanced Ironclad play.
- **OC5 Pride fix shape**: move Pride handling out of `EndTurnInHand` to a new `OnEndOfTurnForPlayingCard` trigger that fires on cards played this turn (before the played-cards-get-discarded pass). Needs a new trigger variant in `CardRuntimeTrigger`.
- **OC6 Parasite fix shape**: add a `cards::on_remove_from_master_deck(engine, card_id)` hook invoked by `run.rs`'s deck-remove paths (rewards screen purge, Empty Cage relic pickup, Fairy Godmother event, Smith upgrade, etc.). Requires auditing every deck-remove callsite in `run.rs` to insert the hook.
- **OC7 Sharp Hide fix shape**: move Sharp Hide from `deal_damage_to_enemy` to the `on_player_play_card` hook; fire THORNS damage to player when `card.card_type == Attack`.
- Confirm training corpus uses the unified Strike/Defend IDs or translates the Java class-suffix IDs before comparing to replay JSON. A separate `docs/work_units/parity-deviations-register.md` row for "id mapping table" may be warranted.

## Cross-refs to existing register

- **Closes D18** — Burn+ magic scaling (re-verified correct).
- **Generalizes D58** — Static Discharge THORNS → every orb passive/evoke (see OC2).
- **Duplicates D71 / D85** — Daze vs Dazed id drift. Recorded here as OC3 for completeness of the status-card audit.

## How to reproduce findings

```bash
# Card-name diffs (from repo root):
JAVA=/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards
RS=/Users/jackswitzer/Desktop/SlayTheSpireRL/.claude/worktrees/happy-boyd-004e01/packages/engine-rs/src/cards

# Java red vs Rust ironclad (exclude mod.rs):
comm -23 \
  <(ls $JAVA/red/ | sed 's/\.java//' | tr '[:upper:]' '[:lower:]' | tr -d '_' | sort) \
  <(ls $RS/ironclad/ | grep -v '^mod.rs' | sed 's/\.rs//' | tr -d '_' | sort)

# Repeat for green/silent, blue/defect, colorless/colorless.
```
