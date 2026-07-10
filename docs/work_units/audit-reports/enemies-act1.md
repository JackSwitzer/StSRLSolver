# Act 1 (Exordium) Enemies & Bosses — Rust↔Java Parity Audit

**Scope:** Exordium enemies plus the three Act 1 bosses (Hexaghost, SlimeBoss, TheGuardian).
**Focus:** HP by ascension, move tables, damage numbers, RNG sampling vs deterministic branches,
passives / start-of-combat effects. Per-enemy matrix + deviation register below.

**Rules followed:** no source edits; deviations with existing IDs in
`docs/work_units/parity-deviations-register.md` are referenced by their D-id and not
re-catalogued here — everything else gets a fresh `E<n>A1` id in this report.

---

## Summary (≤200 words)

The Act 1 enemy layer is **structurally** aligned with Java (right monsters, right move
ids, the boss orchestration skeletons for Guardian/Hexaghost/SlimeBoss exist) but **numerically
and statistically very thin**. Three defects stand out:

1. **GremlinTsundere / GremlinSneaky does literally nothing every turn.** The match arm is
   an empty block labelled `/* Does nothing each turn */` (`enemies/mod.rs:853`), whereas
   Java Protects allies for 7/8/11 block or Bashes for 6/8 if alone. In Gremlin Gang combats
   this one enemy is effectively non-existent.
2. **ApologySlime is absent from `create_enemy` and `roll_next_move_with_num`.** It falls
   through to the default branch (`enemies/mod.rs:903-909`), giving a vanilla 6-damage
   attack instead of Java's 3-dmg + 50% Weak-1 apology (`ApologySlime.java`).
3. **Ascension scaling is almost entirely missing.** `create_enemy` (Act 1 arms at
   `enemies/mod.rs:404-515`) and every `roll_*` body in `act1.rs` hard-code A0 damage,
   debuff magnitudes, and passive counts. A2+/A17+/A19 tiers that change Cultist Ritual,
   Louse bite dmg, GremlinNob Anger, Hexaghost burn count, Guardian Sharp Hide, etc. are
   not applied. Combined with the existing D1 RNG bug, the engine is effectively stuck at
   a deterministic A0 balance.

---

## Per-Enemy Parity Matrix

Legend: **OK** = matches Java; **~** = partially matches (see deviations); **BUG** = wrong/missing.
RNG parity assumes D1 (generic RNG stream) is considered an umbrella — this column reports
whether the *Rust body* actually reads its `num` argument.

| Enemy | HP/A | RNG parity | Moves & dmg | Passives / start | Notes |
|---|---|---|---|---|---|
| Cultist | ~ (lower bound only) | BUG (`_num` unused) | ~ (no A2+/A17+ Ritual) | ~ (Ritual hardcoded 3) | `create_enemy:407`, `act1.rs:31` |
| JawWorm | ~ (max-only) | OK (num buckets) | ~ (no A2+ dmg/str, A17+ str/block) | n/a | `run.rs:1285`, `act1.rs:11` |
| FungiBeast | ~ (deterministic) | BUG (ignores num) | ~ (no A2+ str, A17+ str+1) | OK (SporeCloud 2) | `act1.rs:35` |
| LouseNormal (Red) | ~ (deterministic) | BUG (ignores num + no randomised bite) | BUG (no bite roll 5-7/6-8, no A2+ str 4) | ~ (CurlUp hardcoded 5) | `act1.rs:46`, `mod.rs:415` |
| LouseDefensive (Green) | ~ (deterministic) | BUG | BUG (no bite roll, no A17+ Weak 3) | ~ (CurlUp hardcoded 5) | `act1.rs:57`, `mod.rs:419` |
| AcidSlime_S | ~ (deterministic) | BUG (ignores num) | ~ (no A2+ dmg 4, no A17+ determinism flip) | n/a | `act1.rs:98` |
| AcidSlime_M | ~ (deterministic) | BUG | BUG (fixed cycle instead of prob branches; no A2+/A17+ dmg) | n/a | `act1.rs:107` |
| AcidSlime_L | ~ (deterministic) | BUG | BUG (no prob branches; no split trigger) | BUG (no Split passive) | `act1.rs:120`, hook missing |
| SpikeSlime_S | OK | n/a (deterministic in Java too) | ~ (no A2+ dmg 6) | n/a | `act1.rs:133` |
| SpikeSlime_M | ~ (deterministic) | BUG | ~ (no A2+ dmg 10) | n/a | `act1.rs:137` |
| SpikeSlime_L | ~ (deterministic) | BUG | BUG (no A2+ dmg 18, no A17+ Frail 3, no split) | BUG (no Split passive) | `act1.rs:148`, hook missing |
| SlaverBlue | ~ (deterministic) | BUG | BUG (num<40 STAB logic absent; no A2+/A17+ dmg/weak) | n/a | `act1.rs:68` |
| SlaverRed | ~ (deterministic) | BUG | ~ (ENTANGLE forced on turn 2 instead of `num>=75`; no A2+/A17+ dmg/vuln) | n/a | `act1.rs:79` |
| Looter | ~ (deterministic) | BUG | BUG (no SmokeBomb-vs-Lunge coin flip; Thievery passive missing) | BUG (no Thievery 15/20) | `act1.rs:159` |
| GremlinFat | n/a (HP not rolled, handled elsewhere) | — | ~ (Weak 1 in create_enemy, but roll_simple re-applies 1 — double?; no A17+ Frail 1) | n/a | `mod.rs:453`, `act1.rs:174` |
| GremlinThief | — | — | ~ (no A2+ dmg 10) | n/a | `mod.rs:458` |
| GremlinWarrior | — | — | BUG (dmg only; no A2+ dmg 5, no Angry pre-battle power) | BUG (no Angry 1/2) | `mod.rs:462` |
| GremlinWizard | — | — | ~ (2-turn charge ok, but no A17+ 3rd-blast pattern, no A2+ dmg 30) | n/a | `act1.rs:181` |
| GremlinTsundere / Sneaky | — | BUG (empty arm) | **BUG (no moves at all)** | n/a | `mod.rs:853` |
| GremlinNob | ~ (deterministic) | BUG | BUG (Bellow ok, but no num<33 SKULL_BASH branch; no A2+/A18+ dmg) | ~ (Enrage 2 hardcoded; no A18+ Enrage 3) | `act1.rs:191` |
| Lagavulin | ~ (deterministic max 112) | n/a | BUG (wake order ok; A2+ dmg 20, A17+ debuff -2, Siphon misses A17 scaling) | ~ (Metallicize 8 ok, Sleep 3 ok) | `act1.rs:200` |
| Sentry | ~ (deterministic, Java picks 38-42 rolled) | BUG | BUG (first-move position parity; A3+ dmg 10; Daze pile-count wrong) | BUG (Artifact passive missing) | `act1.rs:230`, `mod.rs:483` |
| ApologySlime | **missing entirely** | **missing** | **missing (falls into default 6-dmg attack)** | missing (no HP roll) | — |
| **TheGuardian** (boss) | ~ (A9 flip at hp≥250) | BUG (ignores num) | ~ (skeleton ok; Fierce Bash dmg scales, Roll dmg ok; no A4+ WHIRLWIND dmg 6) | BUG (Sharp Hide thorns 3 only at A19 → Rust gives 3 when threshold≥40 which is A9+, not A19) | `act1.rs:243`, `combat_hooks.rs` |
| **Hexaghost** (boss) | ~ (A9 flip at hp≥264) | BUG | ~ (cycle ok; Divider wired; A4+ fireTackle 6/inferno 3 gated on status; A19 str 3/burn 2 gated on hp≥264) | ~ (no `gainStrengthPower` first-round-only Strength) | `act1.rs:297` |
| **SlimeBoss** (boss) | ~ (A9 flip at hp≥150) | BUG | ~ (Sticky→Prep→Slam cycle ok; A19 Slimed 5 missing; Slam dmg no A4+ 38) | BUG (Split passive hook present, but spawned L-slimes inherit no HP-half math parity) | `act1.rs:352`, `combat_hooks.rs::handle_boss_damage_hook` |

---

## Deviations

### Cultist

- **E1A1** (bug) — Ritual amount hardcoded 3; no A2+ = 4, no A17+ = (ritualAmount + 1). Rust:
  `packages/engine-rs/src/enemies/mod.rs:409` (`add_effect(mfx::RITUAL, 3)`). Java:
  `decompiled/.../Cultist.java:65, 94-98`. **Fix:** read ascension at create_enemy time and
  branch (`if asc >= 17 { base+1 } else if asc >= 2 { 4 } else { 3 }`).
- **E2A1** (bug) — HP seeding uses only the lower bound of Java's random range. Rust
  `run.rs:1289-1292` picks 48 or 50 deterministically. Java `Cultist.java:58-62` rolls
  `setHp(48,54)` / `setHp(50,56)` via the monster HP RNG. **Fix:** use `monster_hp_rng.random(lo, hi)`.
- **E3A1** (unverified) — Cultist `roll_cultist` ignores `num` (`act1.rs:31`); Java never
  consumes RNG for Cultist beyond `rollMove`'s stream advance, so this matches Java semantics
  once the generic D1 stream-consumption is in place. **Status:** intentional/consistent.

### JawWorm

- **E4A1** (bug) — Damage not scaled: A2+ should set CHOMP=12, BELLOW_STR=4, THRASH dmg unchanged;
  A17+ sets BELLOW_STR=5, BELLOW_BLOCK=9. Rust `act1.rs:19-24` hardcodes 11/3/6. Java
  `decompiled/.../JawWorm.java:48-56`. **Fix:** branch at `create_enemy` and in `roll_jaw_worm` (or
  thread ascension through the helper).
- **E5A1** (bug) — HP roll uses max only: `run.rs:1285-1288` gives 40 or 44; Java rolls 40-44 / 42-46
  (`JawWorm.java:40-43`). **Fix:** use monster HP RNG range.
- JawWorm num-bucket logic (0-24/25-54/55-99 with anti-repeat guards) is correct and
  already noted clean under D1 partial-fix.

### FungiBeast

- **E6A1** (bug) — No GROW str scaling. Rust `act1.rs:37-38` hardcodes 3; Java has strAmt=4 at
  A2+ and `strAmt+1` at A17+ (`FungiBeast.java:59-65, 88-93`). **Fix:** thread ascension.
- **E7A1** (bug) — HP range uses min only: `run.rs:1322-1325` returns 22 or 24; Java rolls
  22-28 / 24-28 (`FungiBeast.java:35-38`). **Fix:** monster HP RNG.
- **E8A1** (bug) — `roll_fungi_beast` ignores `num` (`act1.rs:35`). Java `FungiBeast.java:99-111`
  uses `num < 60` + anti-repeat (60% chance BITE on first if history allows). Rust always picks
  BITE when no GROW in last-1 or GROW if BITE repeated twice. The branch distribution deviates
  (Java's move prob is ~60/40, Rust's is deterministic-on-pattern). **Fix:** re-implement num<60 branch.

### Louse (Normal/Red and Defensive/Green)

- **E9A1** (bug) — Bite damage is not randomised. Java's `monsterHpRng.random(5,7)` / `(6,8)`
  locks biteDamage at combat start (`LouseNormal.java:62-71`). Rust hardcodes 6
  (`mod.rs:416, 420`; `act1.rs:51, 62`). **Fix:** roll at `create_enemy`, store on EnemyCombatState.
- **E10A1** (bug) — CurlUp block is not randomised. Java rolls 3-7 / 4-8 / 9-12 by ascension
  (`LouseNormal.java:73-80`). Rust hardcodes 5 (`mod.rs:417, 421`). **Fix:** HP RNG at create time.
- **E11A1** (bug) — GROW (Red) strength amount not scaled. Java applies `strAmt=3/4` and A17 flips
  the Weak applied by Green Louse to 3 (`LouseNormal.java:82-86`, `LouseDefensive.java:82-94`).
  Rust uses 3 / 2 respectively. **Fix:** thread ascension.
- **E12A1** (bug) — `roll_red_louse` / `roll_green_louse` ignore `num` (`act1.rs:46, 57`). Java
  uses `num < 25` branch with anti-repeat (`LouseNormal.java:96-106`). **Fix:** use num buckets.

### Slavers

- **E13A1** (bug) — SlaverBlue ignores `num`. Java: `num >= 40 && !lastTwoMoves(STAB) → STAB`; else
  `num < 40 && !lastMove(RAKE) → RAKE`; else WEAK. A17+ changes last-check from 2-back to 1-back.
  Rust falls back to hardcoded alternation (`act1.rs:68`). **Fix:** implement num buckets.
- **E14A1** (bug) — SlaverRed forces ENTANGLE on turn 2 regardless of `num`; Java gates on
  `num >= 75` with `!usedEntangle`, then `num >= 55` STAB, else SCRAPE (`SlaverRed.java`).
  Rust `act1.rs:79-96` ignores num. **Fix:** num buckets + used_entangle state.
- **E15A1** (bug) — Damage/debuff amounts not scaled: STAB 12→13 / 13→14, RAKE 7→8 / 8→9, WEAK 1→2,
  VULN 1→2 at A2+/A17+. Rust hardcodes base (`act1.rs:70-95`). **Fix:** thread ascension.

### Acid Slimes

- **E16A1** (bug) — AcidSlime_S ignores `num`. Java: `aiRng.randomBoolean()` coin flip on non-A17;
  deterministic LICK-after-TACKLE on A17+ (`AcidSlime_S.java`). Rust always picks LICK after
  TACKLE (`act1.rs:98`). **Fix:** coin-flip on non-A17.
- **E17A1** (bug) — AcidSlime_M uses a fixed Spit→Tackle→Lick cycle (`act1.rs:107-118`); Java uses
  `num<40` SPIT-branch, `num<80` TACKLE-branch, else LICK with anti-repeat. **Fix:** num buckets
  per Java.
- **E18A1** (bug) — AcidSlime_L same issue as M (`act1.rs:120-131`); Java uses prob branches.
  Additionally, AcidSlime_L's **SplitPower** passive is not modelled — when HP drops ≤ 50% it
  should change move to SPLIT and spawn two AcidSlime_M at half HP (`AcidSlime_L.java` + SplitPower).
  Rust has no split hook for regular AcidSlime_L (only `SlimeBoss` in `combat_hooks.rs::handle_boss_damage_hook`).
  **Fix:** extend split hook to cover AcidSlime_L and SpikeSlime_L; spawn halved-HP M-slimes.
- **E19A1** (bug) — A17 AcidSlime damage upgrades missing: AcidSlime_L Spit 11→12, Tackle 16→18;
  AcidSlime_M Spit 7→8, Tackle 10→12; AcidSlime_S Tackle 3→4.

### Spike Slimes

- **E20A1** (bug) — SpikeSlime_L and SpikeSlime_M ignore `num`; Java uses `num < 30` TACKLE anti-repeat
  and else LICK (`SpikeSlime_L.java`, `SpikeSlime_M.java`). **Fix:** num buckets.
- **E21A1** (bug) — A2+ Tackle scaling: S 5→6, M 8→10, L 16→18. Rust hardcodes base
  (`act1.rs:134, 142, 154`). **Fix:** thread ascension.
- **E22A1** (bug) — A17+ Frail: S unchanged, M 1→2 (Rust gives 1 at `act1.rs:140`), L 2→3 (Rust
  gives 2 at `act1.rs:151`). **Fix:** thread ascension.
- **E23A1** (bug) — SpikeSlime_L SplitPower missing (same as E18A1 for AcidSlime_L). **Fix:** extend
  split hook; on ≤50% HP spawn two SpikeSlime_M at halved HP.

### Looter

- **E24A1** (bug) — No SmokeBomb-vs-Lunge coin flip after 2 Mugs. Java rolls
  `aiRng.randomBoolean(0.5f)` to pick Lunge (14 dmg + escape) or SmokeBomb (6 block + escape).
  Rust always picks SmokeBomb (`act1.rs:164-171`). **Fix:** coin flip on the `turns == 2` branch.
- **E25A1** (bug) — Thievery passive (15 gold A0 / 20 A2+) missing. Java `Looter.java` adds a
  StealGold hook on Mug/Lunge hits; Rust has no gold-steal logic in `combat_hooks.rs::execute_enemy_move`
  for Looter. **Fix:** add gold-steal on Looter damage event, capped at max stolen.
- **E26A1** (bug) — A2+ Mug dmg 10→11, Lunge 12→14; Rust hardcodes 10 (`act1.rs:163`).

### Gremlin Gang

- **E27A1** (bug) — GremlinTsundere / GremlinSneaky has no behaviour at all. `mod.rs:853` is empty.
  Java `GremlinTsundere.java` picks Protect (7/8/11 block on a random ally) if other allies alive,
  else Bash (6/8 dmg). **Fix:** implement Protect via a per-combat random-ally-GainBlock action and
  Bash fallback.
- **E28A1** (bug) — GremlinWarrior missing Angry pre-battle power. Java applies `AngryPower(1/2)` at
  combat start (`GremlinWarrior.java`). Rust never sets ANGRY for GremlinWarrior (`mod.rs:462`).
  **Fix:** `enemy.entity.set_status(sid::ANGRY, 1_or_2)` in `create_enemy`.
- **E29A1** (bug) — GremlinFat applies Weak 1 in create_enemy (`mod.rs:456`) *and* Weak is re-applied
  each turn by `roll_gremlin_simple(..., 1)` (`act1.rs:174`). Java only applies Weak on the Smash
  move (`GremlinFat.java`), not at combat start. **Fix:** remove the start-of-combat `add_effect(WEAK,1)`.
- **E30A1** (bug) — GremlinWizard: no A17+ third-hit pattern. Java at A17+ fires UltBlast and then
  immediately sets next move to UltBlast again (`GremlinWizard.java`). Rust resets to Charge
  (`act1.rs:181-189`). A2+ dmg 25→30 also missing. **Fix:** ascension branching.
- **E31A1** (bug) — GremlinThief no A2+ dmg 10 (Rust hardcodes 9 at `mod.rs:460`).
- **E32A1** (bug) — GremlinFat no A2+ dmg 5, no A17+ Frail 1 (Rust hardcodes 4 Weak 1).

### GremlinNob

- **E33A1** (bug) — SKULL_BASH-vs-RUSH num branch missing. Java first turn always BELLOW (ok),
  then `num < 33 && !lastTwoMoves(RUSH) → RUSH`, else SKULL_BASH (`GremlinNob.java`). Rust uses
  deterministic alternation (`act1.rs:191-198`). **Fix:** num bucket.
- **E34A1** (bug) — Damage scaling missing: BASH 6→8, RUSH 14→16 at A2+; VULN 2→3 at A17+. Rust
  hardcodes 6/14/2 (`act1.rs:195-196, 193`).
- **E35A1** (bug) — A18+ Enrage 3 (not 2). Rust hardcodes `ENRAGE, 2` (`mod.rs:476`). **Fix:**
  ascension branching.

### Lagavulin

- **E36A1** (bug) — A2+ attack dmg 18→20, A17+ debuff -1→-2. Rust hardcodes 18 / -1
  (`act1.rs:207, 218, 227`; `combat_hooks.rs` siphon handling). **Fix:** thread ascension.
- **E37A1** (bug) — Sleep wake-up sequence: Java uses `idleCount == 3` with Metallicize applied
  during sleep. Rust matches structurally but when wake happens it clears Metallicize entirely
  (`act1.rs:204-207`) — Java's wake removes Metallicize only after the first `WakeUp` move, not
  before the transition. Verify ordering in a test. **Status:** unverified nuance.
- **E38A1** (bug) — HP roll: Java `setHp(109,111)` / `setHp(110,112)`. Rust returns 109 / 112
  (`run.rs:1338-1341`). Low bound on A0, max on A8+. **Fix:** monster HP RNG.

### Sentry

- **E39A1** (bug) — No Artifact 1 pre-battle power. Java applies `ArtifactPower(1)`
  at combat start. Rust `create_enemy` for Sentry only sets the first move (`mod.rs:483-485`).
  **Fix:** `enemy.entity.set_status(sid::ARTIFACT, 1)` at create.
- **E40A1** (bug) — First-move position parity missing. Java picks BOLT vs BEAM based on
  `monsters.lastIndexOf(this) % 2` (even index → BEAM, odd → BOLT) for the trio setup
  (`Sentry.java`). Rust always starts with BOLT (`mod.rs:484`). **Fix:** pass an index flag to
  `create_enemy` or branch on encounter layout.
- **E41A1** (bug) — A3+ bolt dmg 9→10. Rust hardcodes 9 (`act1.rs:232, 235`).
- **E42A1** (bug) — Beam applies `DAZE` to the player's discard pile. Java adds Dazed cards to
  discard; Rust's `mfx::DAZE` usage in `act1.rs:233` should map to Dazed-status-card insertion.
  **Status:** unverified — depends on how `mfx::DAZE` is consumed in `combat_hooks.rs`.

### ApologySlime

- **E43A1** (bug) — ApologySlime is entirely absent from both `create_enemy`
  (`mod.rs:397-515`) and `roll_next_move_with_num` (`mod.rs:833-910`). It falls into the default
  arm (`mod.rs:903-909`) which sets a 6-damage move. Java `ApologySlime.java` uses
  `aiRng.randomBoolean()` each turn to pick between Apology (3 dmg, no effect) and a small Weak
  application. **Fix:** add an `"ApologySlime"` arm that sets a 3-dmg move + coin-flip Weak 1
  application, plus register the id in `known_enemy_ids()` (`mod.rs:22-64`).

### TheGuardian

- **E44A1** (bug) — SharpHide thorns magnitude: Java uses 3 until A19 (boss tier), then 4. Rust
  uses `threshold >= 40 ? 4 : 3` (`act1.rs:277`). Threshold 40 corresponds to A9+ HP branching,
  *not* A19 power branching — these are different ascension tiers. **Fix:** use actual ascension,
  not HP-derived proxy.
- **E45A1** (bug) — `roll_guardian` ignores `num` (`act1.rs:243`); Java Guardian also does not
  consume `num` for move selection (`TheGuardian.java` is deterministic on mode/history), so
  this column is actually **OK**. Keep this entry to confirm. **Status:** intentional/consistent.
- **E46A1** (bug) — WhirlWind damage per hit scales A4+: base 5 → 6. Rust hardcodes 5
  (`act1.rs:261`). Fierce Bash (`FIERCE_BASH_DMG` status) and Roll (`ROLL_DMG`) both flip at
  `hp >= 250` (`mod.rs:486-497`); WhirlWind doesn't have an equivalent status. **Fix:** add
  ascension-gated Whirlwind per-hit.
- **E47A1** (bug) — Mode-shift threshold increments by 10 each cycle (Rust matches, `act1.rs:280`)
  but Java caps at the base threshold and does not loop forever — after a certain number of shifts
  Guardian defaults to offensive without DEFENSIVE_BLOCK 20 refresh. Java's `DEFENSIVE_BLOCK`
  on mode-switch (20 block, `TheGuardian.java`) is missing from Rust's `guardian_check_mode_shift`
  (`act1.rs:269-287`). **Fix:** add 20 block on mode transition.

### Hexaghost

- **E48A1** (bug) — A19 strength `gainStrengthPower` missing: Java applies StrengthPower first
  turn (not Inflame-sourced) when ascension ≥ 19. Rust Inflame branch sets STR_AMT on Inflame
  only (`act1.rs:330-331`). **Fix:** add combat-start StrengthPower at A19.
- **E49A1** (bug) — A4+ FIRE_TACKLE/INFERNO gated on `hp >= 264`. Rust branches on HP threshold
  (`mod.rs:500-510`) which is the A9 HP bump, not the A4 damage upgrade. These are different
  ascension gates in Java. **Fix:** gate on ascension, not HP.
- **E50A1** (intentional?) — Divider per-hit damage uses `player_hp / 12 + 1` integer division,
  seeded once on entry. Rust wires this via `hexaghost_set_divider` (`act1.rs:347-349`). **Status:**
  matches Java. OK.

### SlimeBoss

- **E51A1** (bug) — A19 Slimed on Sticky 3→5. Rust hardcodes `SLIMED, 3` (`act1.rs:359`; also
  `mod.rs:514`). **Fix:** ascension branching.
- **E52A1** (bug) — A4+ Slam damage 35→38. Rust hardcodes 35 (`act1.rs:356`).
- **E53A1** (bug) — Split spawns AcidSlime_L and SpikeSlime_L at half HP. Rust `combat_hooks.rs::handle_boss_damage_hook`
  wires split triggers, but the spawned enemies' HP should be `boss_hp / 2` and they should
  inherit any current HP/status state. Verify split correctly uses halved HP (see
  `slime_boss_should_split` at `act1.rs:364-366`). **Status:** unverified — need to read
  `handle_boss_damage_hook` body.

---

## Items Verified Clean

- **JawWorm num-bucket logic** (0-24 CHOMP, 25-54 BELLOW, 55-99 THRASH with anti-repeat):
  matches Java. `act1.rs:18-28` ↔ `JawWorm.java:getMove`. (Tracked under D1 partial-fix.)
- **Hexaghost orb cycle** (Activate → Divider → Sear/Tackle/Inflame/Inferno by orbActiveCount):
  structural match. `act1.rs:297-342` ↔ `Hexaghost.java`.
- **Divider formula** `player_hp / 12 + 1` integer division: correct. `act1.rs:347-349`.
- **SlimeBoss cycle** Sticky → PrepSlam → Slam → Sticky: correct. `act1.rs:352-361`.
- **Guardian two-mode state machine** (offensive Charging → FierceBash → Vent → Whirlwind loop;
  defensive Roll ↔ TwinSlam): structural match. `act1.rs:243-295`.
- **Lagavulin sleep-wake state machine** (3-turn sleep, then alternate Attack ↔ Siphon): correct.
  `act1.rs:200-227`.
- **Looter escape cadence** (2 Mugs → SmokeBomb-style turn → Escape): structural match modulo the
  missing coin flip (E24A1). `act1.rs:159-172`.
- **GremlinWizard charge-up** (Charge → Charge → Blast) two-turn skeleton: match. `act1.rs:181-189`.

---

## Follow-up Questions

1. **Monster HP RNG stream.** Rust uses `self.rng.gen_range(..)` in `run.rs:1282-1296` for the
   handful of Louse entries that randomise HP. Java uses a dedicated `monsterHpRng` per floor
   (`AbstractDungeon.monsterHpRng`). For seed-reproduction parity, should the Rust engine expose
   an independent `monster_hp_rng` thread distinct from `ai_rng`? Currently the only Louse HP
   roll uses the generic floor RNG.
2. **Position-based first move for Sentry.** The trio Sentry encounter depends on the spawn
   ordering (index % 2 flip). Does the encounter builder pass per-slot indices to `create_enemy`?
   If not, E40A1 requires plumbing.
3. **GremlinFat double-Weak.** Confirm whether the start-of-combat Weak in `mod.rs:456` is an
   earlier design decision (e.g., "Fat is always visibly threatening") or a bug. If design,
   capture as an intentional deviation in `DESIGN_DECISIONS.md`.
4. **Ascension propagation.** The `create_enemy` signature accepts only `(id, hp, max_hp)`. To
   scale damage/debuffs correctly we need to either thread ascension (likely the cleanest),
   read from `CombatEngine` or `RunState` at roll time, or pre-bake into per-enemy starting
   statuses (as Hexaghost does today with `STR_AMT`, `FIRE_TACKLE_DMG`, etc.). Which is preferred?
5. **Split hook for L-slimes.** AcidSlime_L and SpikeSlime_L both have SplitPower in Java but
   are treated as regular enemies in `known_enemy_ids()`. The split hook in
   `combat_hooks.rs::handle_boss_damage_hook` is boss-only today. Should split be moved to a
   generic `on_damage_taken` hook so L-slimes (regular) and SlimeBoss (boss) both benefit?

---

## Files / References

### Rust
- `packages/engine-rs/src/enemies/act1.rs` (all Act 1 + 3 boss rolls)
- `packages/engine-rs/src/enemies/mod.rs` lines 22-64 (known_enemy_ids), 397-515 (create_enemy
  Act 1 arm), 829-911 (roll_next_move_with_num)
- `packages/engine-rs/src/run.rs` lines 1282-1357 (roll_enemy_hp Act 1 arm)
- `packages/engine-rs/src/combat_hooks.rs` (`handle_boss_damage_hook`, `execute_enemy_move`)

### Java (decompiled)
- `decompiled/java-src/com/megacrit/cardcrawl/monsters/exordium/Cultist.java`
- `.../JawWorm.java`
- `.../FungiBeast.java`
- `.../LouseNormal.java`, `.../LouseDefensive.java`
- `.../AcidSlime_S.java`, `.../AcidSlime_M.java`, `.../AcidSlime_L.java`
- `.../SpikeSlime_S.java`, `.../SpikeSlime_M.java`, `.../SpikeSlime_L.java`
- `.../SlaverBlue.java`, `.../SlaverRed.java`
- `.../Looter.java`
- `.../GremlinFat.java`, `.../GremlinThief.java`, `.../GremlinWarrior.java`,
  `.../GremlinWizard.java`, `.../GremlinTsundere.java`, `.../GremlinNob.java`
- `.../Lagavulin.java`
- `.../Sentry.java`
- `.../ApologySlime.java`
- `decompiled/java-src/com/megacrit/cardcrawl/monsters/bosses/TheGuardian.java`
- `.../Hexaghost.java`
- `.../SlimeBoss.java`

### Register cross-refs
- **D1** (Enemy AI consumes no RNG) is the umbrella for all "ignores num" findings above.
  Per-enemy fixes here are scoped follow-ups once D1 phase-2 lands.
- **D12** (Multi-enemy intent stream order) — relevant for Sentry trio spawn ordering (E40A1).

---

*Audit scope: Act 1 (Exordium) enemies + 3 Act 1 bosses only. Acts 2/3/4 and the Corrupt Heart
are out of scope for this report and are catalogued separately.*
