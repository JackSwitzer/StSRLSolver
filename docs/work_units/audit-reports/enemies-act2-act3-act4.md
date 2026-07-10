# Enemies Act 2 / Act 3 / Act 4 — Java vs Rust Parity Audit

Scope: every enemy under `com.megacrit.cardcrawl.monsters.city` (Act 2), `.beyond` (Act 3), and `.ending` (Act 4), paired against `packages/engine-rs/src/enemies/{act2,act3,act4}.rs` plus the per-enemy init branch in `packages/engine-rs/src/enemies/mod.rs::create_enemy`.

IDs `D1..D87` in [`parity-deviations-register.md`](../parity-deviations-register.md) are referenced, not duplicated. New findings use the prefixes `E*A2`, `E*A3`, `E*A4`.

## Summary (top findings)

1. **D1 blanket applies** — every Act 2/3/4 `roll_*` function takes `_num` (unused) and selects moves via Rust-side cycles / `last_move` anti-repeat instead of Java `num`-branches. RNG-faithful moves are absent for all ~35 enemies in these acts.
2. **SphericGuardian is missing its passives** — Java prebattle applies **Barricade + Artifact 3 + 40 initial block** and scales **Activate** (`case 2`) to 25 / 35 A17+. Rust init (`mod.rs:565-568`) gives only an opening block move of 40 with no Barricade, no Artifact, and no ascension scaling on Activate (`act2.rs:164-176`).
3. **TheCollector Revive (MOVE 5) is completely missing** — Java's `case 5` respawns dead TorchHeads (`TheCollector.java:161-169`, `getMove num<=25 branch line 189`). Rust `roll_collector` has no Revive branch at all and never references minion death (`act2.rs:312-330`). Combined with missing A19+ stats (`strAmt=5`, `megaDebuffAmt=5`, `rakeDmg=21`), the Collector fight is strictly under-scaled from A4 upward.
4. **CorruptHeart HP gate conflates A9 with A19** — `mod.rs:793-803` reads `hp>=800` (an A9 threshold) and sets `INVINCIBLE=200, BEAT_OF_DEATH=2, bloodHit=15, echo=45`. Java only flips Invincible-100/Beat+1 at **A19+** (`CorruptHeart.java:92-101`), and bloodHit/echo flip at **A4+**. At A9–A18 the Rust Heart gets an A19 passive profile it shouldn't.
5. **Chosen's Drain, Byrd's Caw, Bear's A17+ -4 Dex, Taskmaster's A18+ self-Strength stacking are all absent.** The Chosen Drain move (Weak 3 + self Str 3) is referenced in comments but never selected (`act2.rs:11-29`).
6. **Champ Phase 2 is always Execute** (`act2.rs:272-281`). Java alternates Execute with Face Slap / Heavy Slash / Gloat using RNG and anti-repeat.
7. **SpireShield / SpireSpear miss Surrounded + Artifact + A3+ damage scaling and A18+ Burn-to-draw-pile.**
8. **SnakeDagger Explode is the wrong damage type** — Java declares `DamageInfo(this, 25, DamageInfo.DamageType.HP_LOSS)` (bypasses block & Barricade), Rust uses plain attack damage (`act3.rs:277-283`, `SnakeDagger.java:48`).

---

## Act 2 matrix (city)

| Enemy | RNG parity | HP parity | Moves / damage | Passives | Notes |
|---|---|---|---|---|---|
| Chosen | broken (D1) — `_num` unused | OK (95-99 / 98-103 A7+) | partial — Poke→Hex→Debilitate/Zap cycle; **Drain move missing** | Hex power applied OK via `add_effect` | `mod.rs:520-523` uses POKE(5x2) first; Java A17+ skips Poke-first rule |
| Mugger | broken (D1) | OK | 4-turn script: Mug→BigSwipe→SmokeBomb→Escape. **SmokeBomb uses `block=11`** (should be block 10 + Escape pre-roll); Java is Mug→Bash→SmokeBomb(block 10)→Escape | n/a | `act2.rs:31-46` |
| Byrd | broken (D1) | OK | Peck `1x5` hardcoded; **A2+ should be `1x6`**, **A17+ Flight=4**; **CAW move (self Str 1) completely absent**; no `num<50/num<70` probability tree | Flight 3 init (should be 4 at A17+) | `act2.rs:48-69`, `mod.rs:528-532` |
| Shelled Parasite | broken (D1) | OK | Double Strike(6x2)→Life Suck(10, heal 10)→Fell(18 + Frail 2). First move always Double Strike; Java first-turn is RNG between Double Strike and Fell | Plated Armor 14 OK | `act2.rs:71-82` |
| Snake Plant | broken (D1) | OK | 65/35 Chomp(7x3)/Spores(Weak 2 + Frail 2) flattened to `last_two_moves` anti-repeat | Malleable 1 OK | `act2.rs:84-95` |
| Centurion | broken (D1) | OK | Fury(6x3)→Slash(12)→Protect(15 block + BlockAllAllies 15). **No A2+ Fury=7** / **A17+ Protect block=20** | n/a | `act2.rs:97-107` |
| Mystic / Healer | broken (D1) | OK | Attack(8)→Attack→Heal/Buff; **No A2+ damage=9, no A17+ Buff Str=3** | n/a | `act2.rs:109-126` |
| Book of Stabbing | broken (D1) | OK | Stab(6 × growing), BigStab(21). **Stab count starts at 2** (Java: starts at 1 A0 / 2 A18+); **A18+ BigStab=25 missing** | n/a | `act2.rs:128-143` |
| Gremlin Leader | broken (D1) | OK | Rally→Encourage(6 block + 3 Str to allies)→Stab(6x3). Java Encourage A17+ = 10 block / 5 Str — missing; uses 3 Str constant | n/a | `act2.rs:145-156`. Gremlin spawn RNG (Warrior/Thief/Fat/Mad/Shield/Sneaky) absent; Rust likely spawns generic set via Rally handler in engine. |
| Taskmaster | broken (D1) | OK | Always Scouring Whip(7 + Wound 1). **A2+ dmg=8**; **A18+ self-Strength tick each hit absent** | n/a | `act2.rs:158-162` |
| Spheric Guardian | n/a (no num param) | OK | Initial Block 40 →Frail Attack(10 + Frail 5) → Big(10x2) / Block Attack(10 + 15b). **Activate (case 2) should be 25 / 35 A17+**, Rust's 40 is the prebattle gain, and Rust never selects a later Activate | **Barricade MISSING**; **Artifact 3 MISSING** | `act2.rs:164-176`, `mod.rs:565-568`, `SphericGuardian.java:77-79,92-97` |
| Snecko | broken (D1) | OK | Glare(Confused 1)→Tail(8 + Vuln 2)→Bite(15). **A2+ Bite=18** missing | n/a | `act2.rs:178-186` |
| Bandit Bear | broken (D1) | OK | Hug(-2 Dex)→Maul(18)→Lunge(9 + 9b). **A17+ Dex-4, A2+ Maul=20, A2+ Lunge=10 + 10b** missing | n/a | `act2.rs:188-198` |
| Bandit Leader | broken (D1) | OK | Mock→Agonize(10 + Weak 2)→Cross Slash(15). **A2+ Cross Slash=16** missing | n/a | `act2.rs:200-210` |
| Bandit Pointy | n/a | OK | Fixed Stab(5x2). **A2+ Stab(6x2)** missing | n/a | `mod.rs:583-586`; Pointy has no per-turn `roll_` function |
| TorchHead | n/a | OK | Fixed Tackle(7). Correct — Java always Tackle; no A-scaling | n/a | `mod.rs:607-610` |
| Champ | broken (D1) | OK | Phase 1: Face Slap(Frail 2 + Vuln 2, dmg 14/12)→Heavy Slash→Gloat(+2 Str)→Face Slap; Taunt@turn 4. **Phase 2 always Execute**, Java alternates Execute/Heavy Slash/Face Slap/Gloat | Frail/Vuln init OK | `act2.rs:249-310`. `CHAMP_DEFENSIVE` branch in code is unreachable (no call site). Missing Defensive move (block + Metallicize 2 A7+). |
| Bronze Automaton | broken (D1) | OK (uses hp>=320 gate) | Spawn→Flail(7 / 8 A2+, 2 hits)→Boost (block+Str) → Flail → Hyper Beam (45/50 A2+) → Stunned | Artifact 3 OK | `act2.rs:216-236`, `mod.rs:587-601`. `BA_BOOST` branch exists but the `turns >= 4` threshold sends straight to HyperBeam, so Boost rarely fires. |
| Bronze Orb | broken (D1) | OK | Stasis(turn 1 forced) → Beam(8) → Beam → Support(12 block to Automaton). **Java first-turn: `num>=25` Stasis, else 1 Beam + 1 Support**, not deterministic Stasis | Stasis 1 OK | `act2.rs:238-247`, `mod.rs:602-606` |
| The Collector | broken (D1) | partial — A0/A2+ gate via `hp>=300` | Spawn → Fireball cycle → Buff → Mega Debuff(turn 4). **REVIVE move (`MOVE 5`) entirely missing**; **A19+ stats (`strAmt=5`, `megaDebuffAmt=5`, `rakeDmg=21`) missing**; A19+ Buff `blockAmt+5` missing | n/a | `act2.rs:312-330`, `mod.rs:629-640`, `TheCollector.java:161-169,189` |

## Act 3 matrix (beyond)

| Enemy | RNG parity | HP parity | Moves / damage | Passives | Notes |
|---|---|---|---|---|---|
| Darkling | broken (D1) | OK | Nip(8)→Chomp(8x2)→Harden(12 block + Reanimate). **Nip damage is Java-randomized (`MonsterHpRng.random(7,9)`)**; Rust fixes at 8. **A17+ Harden grants +2 Str missing** | n/a | `act3.rs:11-27`. Dead-body state machine via `hp<=0 → Reincarnate` is present but relies on outer engine to call revive. |
| Orb Walker | broken (D1) | OK | Claw(15)→Laser(10 + Burn 1). **Pre-battle `GenericStrengthUp 3 / 5 A17+` missing** | n/a | `act3.rs:29-42` |
| Spiker | broken (D1) | OK | Attack(7)→Buff(+2 Thorns). **Thorns init 3 (should be 3 / 4 A2+ / +3 A17+ base)**; Buff amount 2 (A17+ = 3) missing | Thorns 3 (may need +1 A2+) | `act3.rs:44-54` |
| Repulsor | broken (D1) | OK | Daze x4 → Attack(11) → repeat. **Java: `num<20` Repulse (add 2 Dazed to draw + Weak 1), else Claw(11); A17+ Claw=14, A17+ Daze=3**; Rust fixed cycle | n/a | `act3.rs:56-66` |
| Exploder | broken (D1) | OK | Slam(9) → Slam → Explode(30). **Java uses `ExplosivePower` timer**, Rust uses a separate TURN_COUNT. **A3+ Slam=10, A19+ Explode=40** missing | ExplosivePower not modeled | `act3.rs:68-78` |
| Writhing Mass | broken (D1) | OK (A2+ 38/9/16/12 vs A0 32/7/15/10 exists in comment only) | Multi-Hit(7x3) → Attack+Block(15 + 15b) → Attack+Debuff(10 + Weak 2 + Vuln 2) → Implant(Parasite curse) → Big Hit(32) → repeat. **Damage numbers hardcoded to A0**; **Reactive re-roll uses cycle-step pick, not a fresh RNG draw** | Reactive 1, Malleable 1 init OK; `USED_MEGA_DEBUFF` gate OK | `act3.rs:80-136`. `writhing_mass_reactive_reroll` exists as helper but its cycle ignores Reactive's original RNG re-draw semantics. |
| Spire Growth | broken (D1) | OK | Quick Tackle(16) / Smash(22) / Constrict(10). **Constrict 10 hardcoded, A17+ = 12** missing | n/a | `act3.rs:138-150` |
| Maw | broken (D1) | OK | Roar(Weak 3 + Frail 3) → Drool(+3 Str) → Slam(25) / NomNom(5 × turnCount/2). **Slam base 25, A2+ = 30 missing**; **Drool +3 Str, A17+ = +4 missing**; **Roar debuffs 3 hardcoded, A17+ = 4 missing** | n/a | `act3.rs:152-171` |
| Transient | broken (D1) | n/a (999 HP) | `startingDeathDmg + attackCount * 10`. See **D87** register entry (+10 off-by-one still open). `STARTING_DMG` init = 30; Java A2+ = 40 not wired. | Fading 5 (Java A17+ = 6) missing; Shifting 1 present | `act3.rs:173-182`, `mod.rs:690-700`. Rust increments ATTACK_COUNT *before* damage calculation (register D87). |
| Giant Head | broken (D1) | OK | Glare/Count(13) alternation → "It Is Time"(`startingDeathDmg - count*5`). **count init 5, A18+ = 4 missing**; **`startingDeathDmg` 30, A3+ = 40 missing** | Slow 1 init OK | `act3.rs:188-221`, `mod.rs:701-709` |
| Nemesis | broken (D1) | OK | Tri Attack(fireDmg×3) / Scythe(45) / Burn(add Burns). **fireDmg hardcoded to 6, A3+ = 7**; **burn count 3, A18+ = 5 missing**; **A18+ Burn → draw pile vs discard missing** | Intangible re-applied per turn in Java's takeTurn (engine-wide hook; not in `roll_nemesis`) | `act3.rs:223-260`, `mod.rs:710-718`. Scythe cooldown decrement mirrored, but `num<30 / num<65` branches collapsed into deterministic pattern. |
| Reptomancer | broken (D1) | OK | Spawn → Snake Strike(13x2 + Weak 1) → Big Bite(30) → Spawn. **A3+ Snake Strike=15, A18+ Big Bite=34**; **A18+ `daggersPerSpawn=2`** missing | n/a | `act3.rs:262-273` |
| Snake Dagger | broken (D1) | OK | Wound(9 + Wound 1) → **Explode(25)** — **Java declares Explode as `DamageType.HP_LOSS`**, Rust treats as normal attack; this bypasses Barricade and block in Java | n/a | `act3.rs:275-283`, `SnakeDagger.java:48` |
| Awakened One | broken (D1) | OK | Phase 1 Slash(20) / Soul Strike(6x4); Phase 2 Dark Echo(40) → Sludge(18 + Void) / Tackle(10x3). **A4+ `Strength 2` init is gated on `hp>=320`**; Java applies it regardless of A4+ (HP gate = A9). **Curiosity 1/2 A19+ OK**. **Regenerate 10/15 A19+ OK** | Unawakened power not wired (Phase 1 reaches 0 HP → rebirth helper `awakened_one_rebirth`) | `act3.rs:289-346`, `mod.rs:728-743`. Sludge `VOID` effect key present but actual draw-pile Void insertion depends on engine handler. |
| Donu | broken (D1) | OK | Circle(+3 Str) → Beam(10x2 / 12x2 A4+). **Beam 2 hits OK, Artifact 2/3 gate by `hp>=265` (A4 threshold) OK** | Artifact init OK | `act3.rs:348-359` |
| Deca | broken (D1) | OK | Beam(10x2 + Daze 2) → Square(16 block). **A19+ Square additionally applies Plated Armor 3 missing**; Beam/Artifact gate `hp>=265` OK | Artifact init OK | `act3.rs:361-372`, `mod.rs:750-756` |
| Time Eater | broken (D1) | OK | Reverberate(rd×3) → Head Slam(hsd) / Ripple(20b + Vuln 1 + Weak 1) → Haste at HP<50%. **A19+ Head Slam extra 2 Slimed in discard missing** (`TimeEater.java:137`); **A19+ Ripple + Frail 1 missing**; **A19+ Haste also grants `headSlamDmg` block missing** | Haste trigger logic OK; Time Warp applied externally | `act3.rs:374-412`, `mod.rs:757-765` |

## Act 4 matrix (ending)

| Enemy | RNG parity | HP parity | Moves / damage | Passives | Notes |
|---|---|---|---|---|---|
| Spire Shield | broken (D1) | OK | 3-move cycle. Bash(12 + Str-1) / Fortify(30 block to all) / Smash(34 + block). **Bash hardcoded 12, A3+ = 14 missing**; **Smash hardcoded 34, A3+ = 38 missing**; **A18+ Smash grants 99 block instead of damage-dealt missing**; **Bash `-1 Focus` alternative (50/50 in Java when player has orbs) missing** | **Surrounded MISSING** (Java prebattle applies it to player); **Artifact 1 / 2 A18+ MISSING** | `act4.rs:11-42`, `mod.rs:770-777`, `SpireShield.java:68-73` |
| Spire Spear | broken (D1) | OK | Burn Strike(5x2 + Burn 2) / Piercer(+2 Str) / Skewer(10 × skewerCount). **Burn Strike dmg 5, A3+ = 6 missing**; **skewerCount fallback 3, A3+ = 4 OK if `SKEWER_COUNT` status set**; **A18+ Burns → draw pile (not discard) missing** | **Artifact 1 / 2 A18+ MISSING** | `act4.rs:44-78`, `mod.rs:778-784`, `SpireSpear.java:72-75` |
| Corrupt Heart | broken (D1) | OK base (A0 750, A9+ 800) | First: Debilitate (Vuln/Weak/Frail 2 each). Then 3-move cycle: Blood Shots(2 × bloodHit) / Echo(40/45) / Buff (+2 Str + escalating power). **Buff escalation (Artifact 2 → +1 Beat → PainfulStabs → +10 Str → +50 Str) is present and matches Java**. **First-turn Debilitate MISSING curse card insertion (1 Dazed + 1 Slimed + 1 Wound + 1 Burn + 1 Void into draw pile)** (`CorruptHeart.java:112-116`) | **`INVINCIBLE=200, BEAT_OF_DEATH=2` gated on `hp>=800`** — but Java gates Invincible-100 & Beat+1 on `ascensionLevel>=19`, not HP/A9 | `act4.rs:80-126`, `mod.rs:785-804`, `CorruptHeart.java:92-101,104-167` |

---

## Deviations (new IDs)

### Act 2 — city

| ID | Severity | Enemy | File:line (Java) | File:line (Rust) | Deviation |
|---|---|---|---|---|---|
| E1A2 | bug | Chosen | `Chosen.java:118-119,150-195` | `act2.rs:11-29` | Chosen's Drain move (Weak 3 on player + Str 3 on self) is defined as `CHOSEN_DRAIN` in move_ids but never selected by `roll_chosen`. Cycle is stuck on Debilitate after Hex. |
| E2A2 | bug | Chosen | `Chosen.java:83-91` | `mod.rs:520-523`, `act2.rs` | A2+ damage scaling (`zapDmg=21, debilitateDmg=12, pokeDmg=6`) not applied. Rust uses base values regardless of ascension. |
| E3A2 | bug | Byrd | `Byrd.java:62-68,174-216` | `act2.rs:48-69`, `mod.rs:528-532` | CAW move (`self +1 Str`, Java `byte 6`) completely absent. Flight 3 hardcoded (A17+ = 4). A2+ `peckCount=6` missing. |
| E4A2 | bug | Byrd | `Byrd.java:185-215` | `act2.rs:59-68` | RNG probability tree (`num<50 Peck / num<70 Swoop / Caw with anti-repeat`) replaced by deterministic Peck/Swoop alternation keyed on `last_two_moves`. |
| E5A2 | bug | Shelled Parasite | `ShelledParasite.java:getMove` | `act2.rs:71-82`, `mod.rs:533-537` | First move always Double Strike; Java chooses Fell ~50% on turn 1. |
| E6A2 | bug | Centurion | `Centurion.java` (ascension branches) | `act2.rs:97-107` | No A2+ Fury=7 damage, no A17+ Protect block=20. |
| E7A2 | bug | Healer / Mystic | `Healer.java` | `act2.rs:109-126` | Mystic Buff amount hardcoded to 2 Strength; Java A17+ = 3. Attack A2+ = 9 missing. |
| E8A2 | bug | Book of Stabbing | `BookOfStabbing.java:getMove, takeTurn` | `act2.rs:128-143`, `mod.rs:551-555` | Stab count initialized to 2; Java starts at 1 and increments on A18+ only. A18+ BigStab=25 missing. |
| E9A2 | bug | Gremlin Leader | `GremlinLeader.java` | `act2.rs:145-156` | Encourage block 6 / Str 3 hardcoded; Java A17+ = 10 block / +5 Str. Spawn RNG (Warrior/Thief/Fat/Mad/Shield/Sneaky) not exposed. |
| E10A2 | bug | Taskmaster | `Taskmaster.java` | `act2.rs:158-162`, `mod.rs:560-564` | A2+ Whip damage=8 missing. A18+ self-Strength increment after each attack absent. |
| E11A2 | bug | Spheric Guardian | `SphericGuardian.java:77-79` | `mod.rs:565-568`, `act2.rs:164-176` | **Barricade passive MISSING**; **Artifact 3 MISSING**. Initial block 40 is wired but no subsequent Activate; Java Activate (case 2) scales 25/35 A17+. |
| E12A2 | bug | Snecko | `Snecko.java` | `act2.rs:178-186` | A2+ Bite=18 missing. A17+ Glare also applies Confused 2 missing. |
| E13A2 | bug | Bandit Bear | `BanditBear.java` | `act2.rs:188-198`, `mod.rs:574-578` | Bear Hug Dex-2 hardcoded; Java A17+ = -4. Maul A2+ = 20, Lunge A2+ = 10 + 10 block missing. |
| E14A2 | bug | Bandit Leader | `BanditLeader.java` | `act2.rs:200-210` | A2+ Cross Slash=16 missing. |
| E15A2 | bug | Bandit Pointy | `BanditPointy.java` | `mod.rs:583-586` | A2+ Stab=(6x2) missing; Rust fixes 5x2. No per-turn `roll_pointy`. |
| E16A2 | bug | Champ | `Champ.java:getMove, takeTurn, Phase2` | `act2.rs:249-310` | Phase 2 always returns Execute (10x2). Java alternates Execute / Face Slap / Heavy Slash / Gloat via RNG with anti-repeat. Also `CHAMP_DEFENSIVE` (15 block + Metallicize 2 A7+) is defined but unreachable. |
| E17A2 | bug | Bronze Automaton | `BronzeAutomaton.java` | `act2.rs:216-236` | A19+ HyperBeam=75 + Flail 8/10 A19 not wired; Boost Str increments (A0=3, A2+=4, A19+=5) collapsed into a single `STR_AMT` status. Boost frequency under-counts because `turns >= 4` sends straight to HyperBeam. |
| E18A2 | bug | Bronze Orb | `BronzeOrb.java:getMove` | `act2.rs:238-247`, `mod.rs:602-606` | First turn deterministic Stasis; Java gates first turn on `num>=25` Stasis vs 1 Beam + 1 Support alt. |
| E19A2 | bug | The Collector | `TheCollector.java:161-169,95-107,189` | `act2.rs:312-330`, `mod.rs:629-640` | **REVIVE move (case 5, `num<=25 && isMinionDead`) completely missing**; **A19+ stats (`strAmt=5, megaDebuffAmt=5, rakeDmg=21`) missing**; A19+ Buff block `blockAmt+5` missing. |

### Act 3 — beyond

| ID | Severity | Enemy | File:line (Java) | File:line (Rust) | Deviation |
|---|---|---|---|---|---|
| E1A3 | bug | Darkling | `Darkling.java` | `act3.rs:11-27`, `mod.rs:645-648` | Nip damage randomized 7-9 in Java (`MonsterHpRng`), Rust fixes at 8. A17+ Harden grants +2 Str missing. |
| E2A3 | bug | Orb Walker | `OrbWalker.java:usePreBattleAction` | `mod.rs:649-653`, `act3.rs:29-42` | Pre-battle `GenericStrengthUp` 3 / 5 A17+ buff missing. |
| E3A3 | bug | Spiker | `Spiker.java` | `act3.rs:44-54`, `mod.rs:654-658` | Thorns init 3 hardcoded (Java A2+ = 4, +3 A17+). Buff amount 2 / A17+ = 3 missing. |
| E4A3 | bug | Repulsor | `Repulsor.java:getMove` | `act3.rs:56-66`, `mod.rs:659-663` | Java: `num<20` Repulse (2 Dazed + Weak 1), else Claw(11). Rust: deterministic Daze×4 → Attack(11). A17+ Claw=14 / Daze=3 missing. |
| E5A3 | bug | Exploder | `Exploder.java` (ExplosivePower) | `act3.rs:68-78`, `mod.rs:664-668` | Rust simulates explode via TURN_COUNT; Java uses `ExplosivePower` decrementing each turn. A3+ Slam=10 / A19+ Explode=40 missing. |
| E6A3 | bug | Writhing Mass | `WrithingMass.java` (getMove + Reactive) | `act3.rs:80-136`, `mod.rs:669-678` | Damage values hardcoded to A0 (32/7/15/10); A2+ variant (38/9/16/12) referenced only in comment. Reactive re-roll uses mechanical next-in-cycle pick rather than a fresh `num` draw. |
| E7A3 | bug | Spire Growth | `SpireGrowth.java` | `act3.rs:138-150`, `mod.rs:679-682` | Constrict 10 hardcoded; A17+ = 12 missing. Quick Tackle A2+ = 18, Smash A2+ = 24 missing. |
| E8A3 | bug | Maw | `Maw.java` | `act3.rs:152-171`, `mod.rs:683-689` | Slam 25 hardcoded (A2+ = 30). Drool +3 Str (A17+ = +4) missing. Roar debuffs 3 each (A17+ = 4) missing. |
| E9A3 | bug | Transient | `Transient.java` | `act3.rs:173-182`, `mod.rs:690-700` | See register entry **D87** (ATTACK_COUNT off-by-one) — still open. Additional: `STARTING_DMG` init 30 (A2+ = 40) missing. Fading 5 (A17+ = 6) missing. |
| E10A3 | bug | Giant Head | `GiantHead.java` | `act3.rs:188-221`, `mod.rs:701-709` | `COUNT` init 5 (A18+ = 4) missing. `STARTING_DEATH_DMG` 30 (A3+ = 40) missing. |
| E11A3 | bug | Nemesis | `Nemesis.java:144-189` | `act3.rs:223-260`, `mod.rs:710-718` | `fireDmg` hardcoded 6 (A3+ = 7). Burn count 3 (A18+ = 5) missing. A18+ Burn goes to draw pile not discard — missing. RNG branches (`num<30, num<65`) collapsed. |
| E12A3 | bug | Reptomancer | `Reptomancer.java` | `act3.rs:262-273` | Snake Strike 13 hardcoded (A3+ = 15). Big Bite 30 (A18+ = 34) missing. A18+ `daggersPerSpawn=2` missing. |
| E13A3 | bug | Snake Dagger | `SnakeDagger.java:48` | `act3.rs:275-283`, `mod.rs:723-727` | Java Explode is `DamageType.HP_LOSS` (25 dmg, bypasses block & Barricade). Rust treats as a normal 25 attack; blocks fully stop it. |
| E14A3 | bug | Awakened One | `AwakenedOne.java` | `act3.rs:289-346`, `mod.rs:728-743` | A4+ starting Strength 2 is gated on `hp>=320` in Rust; Java applies based on `ascensionLevel>=4` regardless of HP. Unawakened power (damage taken in Phase 1 → triggers Phase 2 on first lethal) not wired via Power registry, only via `awakened_one_rebirth` helper. |
| E15A3 | bug | Deca | `Deca.java` (A19+ branch) | `act3.rs:361-372`, `mod.rs:750-756` | A19+ Square additionally grants Plated Armor 3 missing. |
| E16A3 | bug | Time Eater | `TimeEater.java:137,case 3, case 5` | `act3.rs:374-412`, `mod.rs:757-765` | A19+ Head Slam adds 2 Slimed to discard missing. A19+ Ripple adds Frail 1 missing. A19+ Haste grants `headSlamDmg` block missing. |

### Act 4 — ending

| ID | Severity | Enemy | File:line (Java) | File:line (Rust) | Deviation |
|---|---|---|---|---|---|
| E1A4 | bug | Spire Shield | `SpireShield.java:57-73,111-130` | `act4.rs:11-42`, `mod.rs:770-777` | **Surrounded debuff applied to player at battle start — MISSING.** **Artifact 1 / A18+ = 2 MISSING.** Bash damage 12 (A3+ = 14), Smash 34 (A3+ = 38) hardcoded. A18+ Smash grants 99 flat block instead of damage-dealt block — missing. |
| E2A4 | bug | Spire Shield | `SpireShield.java:getMove` | `act4.rs:11-42` | Bash's `-1 Focus` alternative (applied 50/50 in Java when player has orbs) not implemented — Rust always applies `StrengthDown 1`. |
| E3A4 | bug | Spire Spear | `SpireSpear.java:59-75,114-134` | `act4.rs:44-78`, `mod.rs:778-784` | **Artifact 1 / A18+ = 2 MISSING.** Burn Strike 5 (A3+ = 6) / skewer 3 (A3+ = 4) fallback values. A18+ Burn Strike puts Burns in draw pile vs discard — missing. |
| E4A4 | bug | Corrupt Heart | `CorruptHeart.java:92-101` | `mod.rs:793-803` | **HP threshold `hp>=800` used to set `INVINCIBLE=200, BEAT_OF_DEATH=2`.** Java gates Invincible-100 & Beat+1 on `ascensionLevel>=19` (not HP). At A9–A18, Heart should still have `INVINCIBLE=300, BEAT=1`. |
| E5A4 | bug | Corrupt Heart | `CorruptHeart.java:112-116` | `act4.rs:80-126`, `mod.rs:785-792` | First-turn Debilitate (`setMove byte 3`) should insert 1 Dazed + 1 Slimed + 1 Wound + 1 Burn + 1 Void into the player's draw pile; Rust Debilitate only applies Vuln/Weak/Frail 2 each. |
| E6A4 | bug | Corrupt Heart | `CorruptHeart.java:120-123` | `act4.rs:114` | Buff move: Java reads the monster's current negative Strength and overrides it (`additionalAmount = -power.amount` when Str < 0) before applying +2 Str. Rust applies +2 Str directly without compensating for negative Strength (e.g. from Weak Knees / Dark Shackles). |

---

## Items verified clean

- Byrd's Flight init at `FLIGHT=3` is stored as a status tied to the passive; the first-turn Peck `1x5` matches Java A0 exactly.
- Shelled Parasite's `PLATED_ARMOR=14` init matches Java A0 & A17+ (unchanged across ascension).
- Snake Plant's Malleable 1 init matches.
- Torch Head's single fixed Tackle(7) matches Java (no ascension variance).
- Chosen's HEX effect (1 stack) and Debilitate's 10 dmg + Vuln 2 match Java A0.
- Centurion's Fury(6x3) / Slash(12) A0 numbers are correct; only ascension branches missing.
- Bronze Automaton's A2+ gate using `hp>=320` corresponds to `ascensionLevel>=7` HP 320 threshold (correct).
- CorruptHeart's Echo damage 40 / 45 (hp>=800) and blood hit 12 / 15 (hp>=800) match Java A0 / A4+ (A4 HP threshold happens to align with A9 HP gate in Rust — coincidentally correct for blood/echo, incorrect for Invincible/Beat).
- Transient's Shifting power applied at init.
- Darkling's dead-body `hp<=0 → Reincarnate` branch exists (reacts to external engine call).
- Awakened One's Sludge effect key `VOID` is emitted (draw-pile insertion depends on external handler — not verifiable here).

## Open follow-up questions

1. **CorruptHeart first-turn curse cards.** Should the engine model the 5-card draw-pile insertion (Dazed/Slimed/Wound/Burn/Void) of Debilitate, or is this deferred because curse/status card pipelines are incomplete? Affects E5A4 severity.
2. **Surrounded semantics.** The SpireShield passive grants Surrounded to the player (double damage from back). Is "facing" modeled at all? If not, Surrounded is inherently untestable and E1A4's severity should be tagged `deferred`.
3. **Ascension ladder reconstruction.** Every `E*` deviation in this report depends on `ascensionLevel` not being threaded into `create_enemy`. A top-level design question: is the plan to keep HP-based inference gates (current Rust pattern) and fold ascension damage scaling into the status values, or to carry an `ascension: i32` into `create_enemy` and branch exactly like Java?
4. **Writhing Mass Reactive RNG.** Java's Reactive re-runs `getMove(aiRng.random(99))` on hit. Rust's helper picks the next item in a fixed cycle. Is a deterministic re-pick acceptable for MCTS purposes, or does this need a true RNG-consuming re-roll for faithful search?
5. **Intangible re-application on Nemesis.** Java's `takeTurn` unconditionally refreshes Intangible if missing. Where (if at all) is that applied in the Rust engine (`roll_nemesis` does not do it)?
6. **TheCollector Revive prerequisites.** Revive requires `isMinionDead()` detection. Is there an existing engine hook to flag slot-ownership for dead TorchHeads, or does the fix require a new data channel?
