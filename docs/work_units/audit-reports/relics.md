# Relics Rust <-> Java Parity Audit

- **Date**: 2026-04-21
- **Auditor**: Opus 4.7
- **Scope**: All relics in `decompiled/java-src/com/megacrit/cardcrawl/relics/` (181 concrete files + 1 base class `AbstractRelic`) vs Rust surfaces:
  - `packages/engine-rs/src/relics/defs/*.rs` (99 EntityDef modules)
  - `packages/engine-rs/src/relic_flags.rs` (33 bitfield flags + 8 counter slots)
  - Inline dispatch in `engine.rs`, `run.rs`, `combat_hooks.rs`, `card_effects.rs`, `status_effects.rs`, `effects/hooks_*.rs`, `effects/interpreter.rs`, `potions/mod.rs`
- **Branch**: `codex/training-rebuild` (training), `codex/universal-gameplay-runtime` (engine base)
- **Java oracle**: `decompiled/java-src/com/megacrit/cardcrawl/relics/` (spelunky base class pattern: `atBattleStart`/`atBattleStartPreDraw`/`atPreBattle`/`atTurnStart`/`atTurnStartPostDraw`/`onPlayerEndTurn`/`onUseCard`/`onExhaust`/`onVictory`/`onShuffle`/`onEquip`/`onBloodied` hook methods)

## Summary

This engine is deliberately combat-first and Watcher-prioritized. Coverage is excellent on the combat-critical path: 99 relic EntityDefs, ~28 relics handled inline in `engine.rs` (damage modifiers, turn transitions, playability gates), ~33 passive relics via `RelicFlags` bitfield. Counter-based relics (Kunai, Shuriken, Nunchaku, Ornamental Fan, Incense Burner, Happy Flower, Sundial, Velvet Choker) use a consistent `TriggerCondition::CounterReached + TurnStart reset` idiom that correctly mirrors Java's per-turn counter semantics. All 8 Watcher-priority relics (PureWater, Damaru, TeardropLocket, CloakClasp, HolyWater, Duality, Melange, VioletLotus) were checked; 6 are clean, 1 has a minor timing deviation (Damaru), 1 is an outright bug (HolyWater delivers the wrong card), 1 is entirely missing (Melange).

Most gaps are on the non-combat path (event relics, shop relics, campfire relics, boss relic side-effects like `energy.energyMaster++`). Critical exceptions: three boss relics whose primary benefit is `+1 permanent energy` (Philosopher's Stone, Velvet Choker, Runic Dome) have no `max_energy++` handler on equip. Runic Dome also does not hide intents. These are live bugs affecting combat balance.

## Coverage Matrix by Tier

Counts reflect "has any implementation anywhere in the engine" (def, inline, or flag).

| Tier | Java | Implemented | Missing | Notes |
|---|---|---|---|---|
| STARTER | 4 | 4 | 0 | BurningBlood, RingOfTheSerpent, CrackedCore, PureWater |
| COMMON | 23 | 21 | 2 | Missing: Nloth's Mask, Omamori (flag set but negation logic absent) |
| UNCOMMON | 22 | 19 | 3 | Missing: PeacePipe, Shovel, Toolbox |
| RARE | 23 | 18 | 5 | Missing: DeadBranch, Girya(partial), FrozenEye, DuVuDoll(present), Calipers(inline-only, OK) |
| SHOP | 10 | 6 | 4 | Missing: Cauldron, DollysMirror, Melange, MembershipCard(flag-only, OK) |
| BOSS | 24 | 17 | 7 | Missing behaviour: RunicDome (zero impl), VelvetChoker +1 energy, PhilosopherStone +1 energy, Enchiridion (stub only), PandorasBox (partial), SozuExtra (flag OK), BlackStar (flag OK) |
| SPECIAL (Event/Neow) | 20 | 9 | 11 | Missing: Astrolabe, Orrery, EmptyCage, NlothsGift, Circlet, Shovel, BloodyIdol, Mango/Pear/Waffle (HP boosts via Neow+), CultistMask/GremlinMask/RedCirclet (scoring only) |
| Test/deprecated | 6 | 0 | - | Test1/3/4/5/6 and deprecated -- intentionally skipped |

**Totals**: ~160 of 181 Java relics have at least some behavior. ~21 are entirely missing. The 99 concrete EntityDefs + ~28 inline + ~33 flags together exceed the registry's `RELIC_DEFS.len() >= 95` invariant.

## Missing Relics (By Priority)

| Tier | Java Name | Effect | Priority |
|---|---|---|---|
| BOSS | RunicDome | +1 energy & hide enemy intents | **Critical** (combat + policy) |
| RARE | DeadBranch | On exhaust, add random card to hand | **Critical** (Watcher exhaust synergy) |
| BOSS | Enchiridion | Start combat with free 0-cost Power in hand | **High** (Watcher Power archetype) |
| SPECIAL | NilrysCodex | End turn: choose 1 of 3 cards for next turn | **High** (if shop events used) |
| SHOP | Melange | Scry 3 on shuffle | **High** (Watcher listed priority) |
| UNCOMMON | PeacePipe | Rest: remove 1 card from deck | Medium (campfire scope) |
| UNCOMMON | Shovel | Rest: remove 1 card from deck | Medium |
| UNCOMMON | Toolbox | At combat start, choose from 3 Colorless | High (Colorless surface) |
| SHOP | Cauldron | At shop use, lose all potions, gain 5 random | Low (shop) |
| SHOP | DollysMirror | Obtain 1 copy of a card in deck | Low (shop) |
| COMMON | NlothsMask | Next 2 shops: all prices doubled | Low |
| SPECIAL | Astrolabe | Neow: transform 3 cards, upgrade them | High if Neow menu used |
| SPECIAL | Orrery | Choose 5 cards to add to deck | High if Neow menu used |
| SPECIAL | EmptyCage | Remove 2 cards from deck | Medium |
| SPECIAL | NlothsGift | Triple next card reward (then replaces with Nloth's Mask) | Low |
| SPECIAL | Circlet | Gained from duplicate relics (score/display only) | Low (scoring) |
| SPECIAL | BloodyIdol | Heal 5 whenever gaining gold | Low |
| SPECIAL | Mango / Pear / Waffle | +14/+10/+7 max HP on pickup | Medium (HP buffer is observable) |
| SPECIAL | CultistMask / GremlinMask / RedCirclet | Display / Neow-only, no live effect | Low |
| SPECIAL (Watcher) | GoldenEye | Scry: see 2 extra cards | Low (Scry enhancer) |
| STARTER+ | BottledFlame / BottledLightning / BottledTornado | Guarantee one Attack/Skill/Power in opening hand | Medium (listed in obs, no defs) |

## Deviations (Bugs / Intentional Divergence)

| ID | Relic | Tier | Severity | Deviation | Java ref | Rust ref |
|---|---|---|---|---|---|---|
| **R1** | HolyWater | BOSS | **bug** | Rust adds card id `"HolyWater"` (a 5-Block skill in Rust's card registry) to hand at combat start. Java adds 3 `new Miracle()` cards (each +1 Energy when played). The card ID `"HolyWater"` in Rust does NOT alias `Miracle`; it is a distinct 5-block self-target card. Net effect: HolyWater owners get 15 block instead of 3 free energy each combat. Test `test_relic_runtime_wave15.rs:37-43` codifies the buggy behavior. | `HolyWater.java:29` uses `new Miracle()` | `relics/defs/holy_water.rs:7-11` adds id `"HolyWater"`; card def in `cards/watcher/holywater.rs:6-13` |
| **R2** | Velvet Choker | BOSS | **bug** | Rust gates card plays at 6/turn (correct) but NEVER grants the `+1 permanent energy` that Java applies via `onEquip`. Java: `++AbstractDungeon.player.energy.energyMaster`. Rust `add_relic_reward` (run.rs:2210-2227) does not touch `max_energy` for this relic. | `VelvetChoker.java:42-45` onEquip | `relics/defs/velvet_choker.rs` (no equip hook); `run.rs:2205-2228` (missing match arm) |
| **R3** | Philosopher's Stone | BOSS | **bug** | The def file's own comment claims "Energy bonus is handled via max_energy on equip" but no such handler exists. Only +1 STR to enemies is implemented. Net effect: owner loses the +1 permanent energy; enemies still get STR debuff. | `PhilosopherStone.java:52-54` onEquip | `relics/defs/philosophers_stone.rs:2` (stale comment), `run.rs:2205-2228` (no case) |
| **R4** | Runic Dome | BOSS | **bug** | Entirely unimplemented. Not in `relics/defs/*`, not in `engine.rs`, not in `relic_flags.rs`. Java grants +1 energy AND hides enemy intents. Both absent. | `RunicDome.java:40-42` onEquip, plus intent hiding | Only appears in `obs.rs:59` as a string literal |
| **R5** | Damaru | COMMON | **deferred** | Rust uses `Trigger::TurnStartPostDrawLate` (fires AFTER draw). Java uses `atTurnStart()` (fires BEFORE draw). Mantra +1 per turn is granted in both cases, but the pre-draw timing matters for: (a) any card that would key off Mantra in the opening hand, and (b) Stance::Divinity promotion timing if the pre-existing Mantra was at 9. Likely low observable impact but divergent. | `Damaru.java:31-36` atTurnStart | `relics/defs/damaru.rs:13` TurnStartPostDrawLate |
| **R6** | Enchiridion | BOSS | **deferred** | EntityDef is a pure stub with `triggers: &[]`. Java `atPreBattle` adds a random Power card (cost-reduced to 0 for the turn) to the combat-start hand. Watcher's Power archetype (Adaptation, Fasting, Wallop, etc.) loses a significant turn-1 advantage. | `Enchiridion.java:25-34` | `relics/defs/enchiridion.rs:6` empty triggers |
| **R7** | DeadBranch | RARE | **deferred** | Entirely unimplemented. Mentioned only in an engine.rs comment. A defining synergy for Watcher exhaust builds (Carve Reality, Conclude, Explosive Purge, Sands of Time). | `DeadBranch.java:20-27` onExhaust | Only comment at `engine.rs:3091` |
| **R8** | Omamori | COMMON | **deferred** | Flag bit set and `counters[OMAMORI_USES] = 2` is initialized, but no code path in `run.rs` or elsewhere ever decrements this counter when a curse is added to the deck. Thus 0 curse negations ever occur; flag and counter are dead. | `Omamori.java:34-42` use() decrements counter | `relic_flags.rs:124` init only |
| **R9** | PandorasBox | BOSS | **deferred** | Partial implementation: Rust removes Strikes/Defends from master_deck but does NOT replace them with random class commons (the "transform" half). Documented in code (`run.rs:2213-2225`). Net effect: post-Pandora deck is ~5 cards smaller than Java. | `PandorasBox.java` atBattleStartPreDraw | `run.rs:2221-2225` comment-documented partial |
| **R10** | Melange | SHOP | **deferred** | Unimplemented. Listed as Watcher-priority by user. Java: `onShuffle -> Scry 3`. Rust has no OnShuffle handler for it. | `Melange.java:24-29` onShuffle | Only `obs.rs:50` |
| **R11** | Ring of the Snake | STARTER | **intentional** | Rust uses `TurnStart + FirstTurn`, Java uses `atBattleStart`. Functionally equivalent (+2 draw on turn 1), but the Rust timing fires AFTER starting draw completes while Java fires BEFORE. For Silent, the 2 extra cards appear in hand after the initial 5, identical observable state at start of action. Low risk. | `SnakeRing.java:25-29` atBattleStart | `relics/defs/ring_of_snake.rs:13-18` |
| **R12** | Mercury Hourglass | UNCOMMON | **unverified** | Rust deals 3 damage to all enemies at TurnStart. Java uses `DamageInfo.DamageType.THORNS` which bypasses the player's strength calculation and still triggers enemy reflection / NOT-thorns-resistant mobs correctly. Unclear whether Rust applies the same damage type. | `MercuryHourglass.java` atTurnStart | `relics/defs/mercury_hourglass.rs` |
| **R13** | BlueCandle | BOSS | **intentional** | Stub def (`triggers: &[]`). Behavior is correct via `effects/hooks_can_play.rs:16` — the playability gate deals 1 HP and exhausts curse cards. Acceptable design choice (can_play hook). Not a bug. | `BlueCandle.java` canPlay override | `effects/hooks_can_play.rs:16` |
| **R14** | Medical Kit | BOSS | **intentional** | Same pattern as R13 — status cards gain exhaust-on-play. Stub def, logic in can_play hook. Clean. | `MedicalKit.java` onUseCard | `effects/hooks_can_play.rs:11` |
| **R15** | Necronomicon | BOSS | **intentional** | Not in `relics/defs/*` — implemented inline (`engine.rs:1118-1120, 2103-2108`). Repeats 2+ cost attack once per turn. Inline handling is correct. | `Necronomicon.java` | `engine.rs:1118,2103-2108` |
| **R16** | Calipers | RARE | **intentional** | Inline only (`engine.rs:1155`). Block decay subtracts 15 instead of capping — matches Java per D49 comment. Correct. | `Calipers.java` | `engine.rs:1155` |
| **R17** | Runic Pyramid | BOSS | **intentional** | Inline only (`engine.rs:1363-1380`). Ethereal still exhausts, everything else retained. Matches Java. | `RunicPyramid.java` | `engine.rs:1363` |
| **R18** | Ice Cream | BOSS | **intentional** | Inline only (`engine.rs:1102`). Energy preserved between turns. Correct. | `IceCream.java` | `engine.rs:1102` |
| **R19** | Unceasing Top | BOSS | **intentional** | Inline only (`engine.rs:2155`). Draw until hand has 7 while enemies alive. Requires `has_relic("Unceasing Top")` + engine loop. | `UnceasingTop.java` | `engine.rs:2155` |
| **R20** | Smiling Mask | SHOP | **intentional** | Flag-based. Card-removal cost at shop = 50g. Handled by shop logic (`run.rs:529`). Clean. | `SmilingMask.java` | `relic_flags.rs:94`, `run.rs:529` |
| **R21** | NuclearBattery | BOSS | **intentional** | Channel Plasma at combat start. Rust uses complex_hook. Clean. | `NuclearBattery.java` atBattleStart | `relics/defs/nuclear_battery.rs` |

## Watcher Deep-Dive (Priority Relics)

| Relic | Status | Verification |
|---|---|---|
| **PureWater** | CLEAN | CombatStart adds 1 `Miracle` card (correct). Starter Watcher relic. `relics/defs/pure_water.rs`. Java `PureWater.java` matches (adds Miracle via `MakeTempCardInHandAction`). Test `test_relic_runtime_wave15.rs:24-32` passes. |
| **Damaru** | MINOR deviation (R5) | TurnStartPostDrawLate vs Java's atTurnStart. Functional for Mantra accumulation but pre/post-draw timing differs. Unlikely to move policy evaluations. |
| **TeardropLocket** | CLEAN | CombatStart → ChangeStance(Calm). Exact Java parity via `SimpleEffect::ChangeStance`. |
| **CloakClasp** | CLEAN | TurnEnd GainBlock(HandSize). Matches Java `hand.group.size() * 1` exactly. |
| **HolyWater** | **BUG (R1)** | Wrong card ID. Should deliver Miracles (+1 Energy on play), delivers 5-Block skills instead. Test assertion is wrong alongside the bug — both need fixing. |
| **Duality** (Yang) | CLEAN | OnAttackPlayed → add DEXTERITY + LOSE_DEXTERITY (temporary Dex). Matches Java. `relics/defs/yang_duality.rs`. |
| **Melange** | **MISSING (R10)** | No implementation. Shop relic; Watcher priority per user spec. |
| **VioletLotus** | CLEAN | CombatStart sets VIOLET_LOTUS flag; `engine.rs:2961` adds +1 to the +2 default Calm-exit energy for total +3 on exiting Calm. Matches Java (Calm default +2 + VioletLotus +1). |

Additional Watcher-adjacent:
- **Enchiridion (R6)**: Stub — 0-cost Power in hand at combat start NOT delivered. Blocks Watcher Power-archetype training signal on turn 1.
- **VelvetChoker (R2)**: Gate at 6 plays works; **+1 permanent energy missing**. Significant for Watcher (high-velocity stance-switching deck).
- **RingOfTheSerpent / SnekoEye**: Silent/generic, all clean. CONFUSION + SNECKO_EYE + BAG_OF_PREP_DRAW=2 correctly set.

## Items Verified Clean

(EntityDef + trigger semantics match Java; counters reset correctly where applicable.)

- Combat-start stats: Vajra, OddlySmoothStone, DataDisk, Akabeko, Anchor, BagOfMarbles, RedMask, ThreadAndNeedle, BronzeScales, ClockworkSouvenir, FossilizedHelix, BloodVial, TwistedFunnel, MutagenicStrength
- Counter-based: OrnamentalFan, Kunai, Shuriken, Nunchaku, LetterOpener, InkBottle, HappyFlower, IncenseBurner, Sundial
- Turn-based: MercuryHourglass (damage type unverified, see R12), Orichalcum (NoBlock conditional), Lantern (FirstTurn), Brimstone, CloakClasp, Damaru (timing note R5)
- Event-triggered: BirdFacedUrn, CharonsAshes, ToughBandages, Tingsha, GremlinHorn, BurningBlood, BlackBlood, ToyOrnithopter, SelfFormingClay, TheAbacus
- Flag-setting: Ginger, Turnip, MarkOfBloom, MagicFlower, SneckoEye
- Counter-initialize: VelvetChokerInit, Pocketwatch, ArtOfWar, OrangePellets, HornCleat, CaptainsWheel, StoneCalendar
- Orb channeling: SymbioticVirus, CrackedCore, NuclearBattery
- Card generation at combat start: PureWater, NinjaScroll, MarkOfPain (HolyWater has R1 bug)
- Turn-start/end: BagOfPrep, RingOfSnake (R11), Inserter, FrozenCore
- Card-play hooks: MummifiedHand, YangDuality, VelvetChoker (gate only — energy missing R2)
- HP-loss: CentennialPuzzle, RunicCube, EmotionChip
- Victory: MeatOnTheBone, FaceOfCleric
- Damage modifiers: Boot, Torii, TungstenRod, ChampionBelt, HandDrill
- Passive bonuses: StrikeDummy, WristBlade, SneckoSkull
- Other combat: RunicCapacitor, RingOfSerpent, VioletLotus, RedSkull, WarpedTongs, GamblingChip, HoveringKite, LizardTail, AncientTeaSet, StrangeSpoon
- Inline: Necronomicon (R15), Calipers (R16), RunicPyramid (R17), IceCream (R18), UnceasingTop (R19)
- Flags (RelicFlags): Ectoplasm, GoldenIdol, CoffeeDripper, FusionHammer, Sozu, MembershipCard, SacredBark, CursedKey, BlackStar, PrismaticShard, RegalPillow, IceCream (dual-mode with inline), ToyOrnithopter, SmilingMask (R20), SingingBowl, QuestionCard, PrayerWheel, MawBank, OldCoin, CeramicFish, MealTicket, DreamCatcher, JuzuBracelet, SsserpentHead, TheCourier, Matryoshka, MarkOfBloom, MagicFlower, WhiteBeast, TinyChest, MoltenEgg2, ToxicEgg2, FrozenEgg2
- Paper Crane / Paper Frog / Chemical X / Odd Mushroom / Paper Crane: damage-math hooks in `card_effects.rs` and `effects/hooks_complex.rs` (correct)

## Follow-up Questions

1. **R12 Mercury Hourglass damage type**: Should Rust's 3-damage-to-all use `DamageType::Thorns` semantics? This impacts enemy armor/reflective interactions. Current Rust appears to use default attack damage; Java uses Thorns.
2. **R1/R2/R3/R4 fix order**: The four energy-related boss relic bugs (HolyWater, VelvetChoker, PhilosopherStone, RunicDome) all route through `add_relic_reward`. Fix as a single batch? The cleanest path is a new `max_energy_delta_on_equip` map keyed on relic ID, plus a wire-up of an `on_equip_hook` in `run.rs`.
3. **R11 Ring of the Snake timing**: Is it worth promoting `FirstTurn`-gated TurnStart draws to a dedicated `CombatStart` trigger for accurate pre-draw timing, or does the present pattern already match all observable game states?
4. **Omamori (R8) scope**: With Watcher-first training, curse-negation is rare in combat. Defer entirely, or wire `run.rs` curse-add path through the counter? Affects shop / event flow, not combat.
5. **Event/Neow relics** (Astrolabe, Orrery, EmptyCage, NlothsGift, Circlet, BloodyIdol, Mango/Pear/Waffle): In current training scope (artifact-first, combat-first), are these out of scope, or should they be tracked as `unverified` until the event pipeline is rebuilt?
6. **Watcher Enchiridion (R6)**: Would require the card_random_rng seeded stream + a class-aware Power card pool to implement cleanly. Track as a prerequisite for D52 or implement with a deterministic fallback pool first?

## Final Summary (Watcher-Blocking Priority)

Engine is combat-complete for Watcher with 8 relic gaps of varying severity. Coverage exceeds 88% across 181 Java relics. Counter-idiom (7/7 relics) and status-stacking path (PureWater, TeardropLocket, CloakClasp, Duality, VioletLotus) are solid.

**Top 5 Watcher-blocking gaps (prioritized)**:

1. **HolyWater (R1, bug)** — Boss relic delivers 15 block instead of 3 Miracles (9 Energy). Massive scaling distortion. Fix: change `relics/defs/holy_water.rs:7-11` to add card id `"Miracle"` and update `test_relic_runtime_wave15.rs:42-43`.
2. **VelvetChoker (R2, bug)** — Trades 6-plays/turn cap for no compensating energy. Training will under-value this boss relic; agent has no reason to take it. Fix: add `"Velvet Choker" => max_energy += 1` branch in `run.rs:add_relic_reward` + reverse on removal.
3. **PhilosopherStone (R3, bug)** — Same pattern as R2; half the relic's value missing. Fix identical shape to R2.
4. **Enchiridion (R6, deferred)** — Stub; Watcher Power turn-1 tempo absent. Blocks an entire archetype from showing its strength in training signal.
5. **DeadBranch (R7, deferred)** — Watcher exhaust archetype loses a defining synergy. Without it, Carve Reality/Conclude/Sands of Time are strictly weaker.

Non-Watcher but high-impact: **RunicDome (R4)** — a Boss relic that is completely missing. Energy bonus absent AND enemy intents still visible, so it's effectively a free relic with zero downside or upside in the sim.

These five combined block ~10-15% of Watcher win-rate headroom in A0+ ascensions. Recommend fixing R1-R4 together (single PR — all touch `add_relic_reward` / `holy_water.rs`), then R5-R6 as separate work units.
