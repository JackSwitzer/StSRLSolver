# Parity audit: Potions

**Date:** 2026-04-21
**Auditor:** Opus 4.7 (subagent)
**Scope:** Java `decompiled/java-src/com/megacrit/cardcrawl/potions/` vs Rust `packages/engine-rs/src/potions/`
**Potions reviewed:** 42 (40 usable + `FairyPotion` passive + `PotionSlot` placeholder)
**Potions implemented in Rust:** 40 (plus passive FairyPotion hook; PotionSlot is an empty string in `state.potions`)
**Deviations found:** 13

## Summary

Potion parity is broad: every Java potion has a corresponding Rust entity definition
in `packages/engine-rs/src/potions/defs/`, and the owner-aware runtime is authoritative
for the common/uncommon/rare combat potions used during the Watcher A0 combat loop.
Base potency values match Java exactly (20 Fire, 10 Explosive, 12 Block, 2 Strength,
6 Poison, etc.), and Sacred Bark is honored through `effective_potency_runtime` for
every potion whose Rust effect uses `A::PotionPotency`. The important deviations
cluster in three areas: (1) four potions declare their effect amount with `A::Fixed`
instead of `A::PotionPotency`, so Sacred Bark silently does nothing on
`AncientPotion`, `BloodPotion`, `FruitJuice`, and (indirectly) `FairyPotion`'s
`Fixed`-style percent; (2) several complex potions are simplified — `GamblersBrew`
force-discards the entire hand instead of letting the agent pick, `Elixir` exhausts
the entire hand instead of offering a selectable exhaust, and `DuplicationPotion`
and `CultistPotion` set player statuses that are never consumed (no
PlayerTurnEnd Ritual trigger, no Duplication handling on next-card-played); and
(3) `SneckoOil` incorrectly applies `CONFUSION=1` in addition to draw, whereas
Java only draws cards and randomizes hand cost. A11+ scaling is present only in
the `#[cfg(test)]` helper path — production combat reads base potency because
ascension is not threaded into `CombatState`; this is explicitly documented in
`potions/mod.rs` but it silently diverges from Java at A11+. Dispatch targeting,
the Smoke Bomb boss/BackAttack gate, and the Fairy revive passive path are all
correct.

## Coverage matrix

| Potion | Java impl | Rust impl | Sacred Bark scaling | Target | Notes |
|---|---|---|---|---|---|
| Ambrosia | `Ambrosia.java` | `potions/defs/ambrosia.rs` + `potions/mod.rs:42` | No-op (fixed enter-Divinity) | Self | Matches Java (ChangeStanceAction "Divinity"). |
| AncientPotion (Artifact) | `AncientPotion.java` | `defs/artifact_potion.rs` | **Broken:** `A::Fixed(1)` ignores bark | Self | PT1. |
| AttackPotion | `AttackPotion.java` | `defs/attack_potion.rs` | Hook uses `effective_potency_runtime` (runtime-scaled copies) | Hand | Opens 3-card Discovery choice correctly. |
| BlessingOfTheForge | `BlessingOfTheForge.java` | `defs/blessing_of_forge.rs` | No-op (fixed behavior) | Hand | Upgrades every upgradeable card in hand; matches ArmamentsAction(true). |
| BlockPotion | `BlockPotion.java` | `defs/block_potion.rs` | Yes (`A::PotionPotency`) | Self | 12 block base, 24 with bark. |
| BloodPotion | `BloodPotion.java` | `defs/blood_potion.rs` | **Broken:** `A::PercentMaxHp(20)` ignores bark | Self | PT2. Java heals 20% base, 40% with bark. |
| BottledMiracle | `BottledMiracle.java` | `defs/bottled_miracle.rs` | Yes | Hand | Miracle count = potency. |
| ColorlessPotion | `ColorlessPotion.java` | `defs/colorless_potion.rs` | Yes (copies count via runtime) | Hand | Discovery from colorless pool. |
| CultistPotion | `CultistPotion.java` | `defs/cultist_potion.rs` | Yes (but Ritual on player is inert) | Self | PT3. |
| CunningPotion | `CunningPotion.java` | `defs/cunning_potion.rs` | Yes | Hand | PT4: adds plain "Shiv" — Java upgrades to Shiv+. |
| DexterityPotion | `DexterityPotion.java` | `defs/dexterity_potion.rs` | Yes | Self | +2 Dex. |
| DistilledChaos | `DistilledChaosPotion.java` | `defs/distilled_chaos.rs` | Yes (hook hardcodes 3/6) | Draw | Plays top N cards free. |
| DuplicationPotion | `DuplicationPotion.java` | `defs/duplication_potion.rs` | Yes (status) | Self | PT5: DUPLICATION status but no consumer. |
| Elixir | `Elixir.java` | `defs/elixir.rs` | N/A | Hand | PT6: exhausts entire hand, Java offers selectable. |
| EnergyPotion | `EnergyPotion.java` | `defs/energy_potion.rs` | Yes | Self | +2 energy. |
| EntropicBrew | `EntropicBrew.java` | `defs/entropic_brew.rs` | N/A (fills empty slots) | Slots | PT7: fills with fixed "Block Potion" proxy. |
| EssenceOfDarkness | `EssenceOfDarkness.java` | `defs/essence_of_darkness.rs` | **Broken:** hook channels 1-per-slot ignoring bark | Orbs | PT8. |
| EssenceOfSteel | `EssenceOfSteel.java` | `defs/essence_of_steel.rs` | Yes | Self | +4 PLATED_ARMOR. |
| ExplosivePotion | `ExplosivePotion.java` | `defs/explosive_potion.rs` | Yes | All enemies | 10 base. |
| FairyPotion | `FairyPotion.java` | `defs/fairy_in_a_bottle.rs` + `potions/mod.rs:573-599` | Yes (in `check_fairy_revive_scaled`) | Passive | PT9: missing `max(1)` clamp on heal. |
| FearPotion (Vulnerable) | `FearPotion.java` | `defs/fear_potion.rs` | Yes | Enemy | 3 Vuln. |
| FirePotion | `FirePotion.java` | `defs/fire_potion.rs` | Yes | Enemy | 20 dmg. |
| FocusPotion | `FocusPotion.java` | `defs/focus_potion.rs` | Yes | Self | +2 Focus. |
| FruitJuice | `FruitJuice.java` | `defs/fruit_juice.rs` | **Broken:** `A::Fixed(5)` ignores bark | Self | PT10. |
| GamblersBrew | `GamblersBrew.java` | `defs/gamblers_brew.rs` | N/A | Hand | PT11: force-discards hand; Java offers a hand-select screen. |
| GhostInAJar (Intangible) | `GhostInAJar.java` | `defs/ghost_in_a_jar.rs` | Yes | Self | 1 Intangible. |
| HeartOfIron (Metallicize) | `HeartOfIron.java` | `defs/heart_of_iron.rs` | Yes | Self | +6 Metallicize. |
| LiquidBronze (Thorns) | `LiquidBronze.java` | `defs/liquid_bronze.rs` | Yes | Self | +3 Thorns. |
| LiquidMemories | `LiquidMemories.java` | `defs/liquid_memories.rs` | Yes | Discard | Returns N to hand, cost 0. |
| PoisonPotion | `PoisonPotion.java` | `defs/poison_potion.rs` | Yes | Enemy | 6 Poison. |
| PotionOfCapacity | `PotionOfCapacity.java` | `defs/potion_of_capacity.rs` | Yes | Self | +2 ORB_SLOTS. |
| PowerPotion | `PowerPotion.java` | `defs/power_potion.rs` | Yes | Hand | Discovery from power pool. |
| RegenPotion | `RegenPotion.java` | `defs/regen_potion.rs` | Yes | Self | +5 Regeneration. |
| SkillPotion | `SkillPotion.java` | `defs/skill_potion.rs` | Yes | Hand | Discovery from skill pool. |
| SmokeBomb | `SmokeBomb.java` | `defs/smoke_bomb.rs` | N/A | Self | Flee; boss/BackAttack gate in `potion_can_use_in_combat` matches Java. |
| SneckoOil | `SneckoOil.java` | `defs/snecko_oil.rs` | Yes (draw) / **buggy confusion** | Self | PT12: Java only draws+randomizes hand costs; Rust adds spurious CONFUSION=1 status. |
| SpeedPotion | `SpeedPotion.java` | `defs/speed_potion.rs` | Yes | Self | +5 Dex/-5 Dex. |
| SteroidPotion (Flex) | `SteroidPotion.java` | `defs/flex_potion.rs` | Yes | Self | +5 Str/-5 Str. |
| StancePotion | `StancePotion.java` | `defs/stance_potion.rs` | N/A | Self | Wrath/Calm choice. |
| StrengthPotion | `StrengthPotion.java` | `defs/strength_potion.rs` | Yes | Self | +2 Strength. |
| SwiftPotion | `SwiftPotion.java` | `defs/swift_potion.rs` | Yes | Self | Draw 3. |
| WeakenPotion (Weak) | `WeakenPotion.java` | `defs/weak_potion.rs` | Yes | Enemy | 3 Weak. |

## Missing potions

None. Every Java potion (Watcher-legal and otherwise) has a Rust EntityDef.
(Note: `PotionSlot.java` is a UI placeholder object, not a usable potion; Rust
represents empty slots with an empty string in `CombatState.potions`.)

## Deviations

| ID | Summary | Severity | Rust ref | Java ref | Description | Proposed fix |
|----|---------|----------|----------|----------|-------------|--------------|
| PT1 | Ancient Potion ignores Sacred Bark | bug | `packages/engine-rs/src/potions/defs/artifact_potion.rs:4` | `AncientPotion.java:38-48`, `AbstractPotion.java:639-645` | Rust applies `ARTIFACT = A::Fixed(1)`, so Sacred Bark never doubles it. Java: `AbstractPotion.getPotency()` doubles potency with Sacred Bark, so Ancient Potion would grant 2 Artifact with the relic. The potency table in `potions/mod.rs:174` lists `(1, 1)` so even the test-only path never scales past base. | Change to `A::PotionPotency` and add `AncientPotion => (1, 1)` handling (already present) so `effective_potency_runtime` × bark works. |
| PT2 | Blood Potion ignores Sacred Bark | bug | `packages/engine-rs/src/potions/defs/blood_potion.rs:4` | `BloodPotion.java:36-43` | Rust uses `SE::HealHp(Player, A::PercentMaxHp(20))`. `AmountSource::PercentMaxHp` in `effects/runtime.rs:1378` does not multiply by Sacred Bark. Java heals `maxHealth * potency / 100` where potency is 20 base → 40 with bark. | Replace with a bark-aware amount source (e.g. extend `PercentMaxHp` to honor Sacred Bark when owner is a potion slot) or use `A::PotionPotency` plus a bespoke HealHp-percent variant. |
| PT3 | Cultist Potion applies Ritual to player but never triggers Strength gain | bug | `packages/engine-rs/src/potions/defs/cultist_potion.rs:4`, `engine-rs/src/powers/defs/enemy.rs:24-38` | `CultistPotion.java:47-55`, `RitualPower.java:37-43` | Java passes `playerControlled=true` to `RitualPower`, which triggers end-of-turn Strength gain via `atEndOfTurn(isPlayer)`. Rust's `DEF_RITUAL` is gated on `Trigger::EnemyTurnStart`, so `RITUAL` on `state.player` never fires. The potion behaves as a no-op status sticker. | Add a player-side Ritual handler (new trigger or separate power def) so `sid::RITUAL` on the player adds Strength at PlayerTurnEnd. |
| PT4 | Cunning Potion summons plain Shiv instead of Shiv+ | bug | `packages/engine-rs/src/potions/defs/cunning_potion.rs:4-12` | `CunningPotion.java:42-48` | Java constructs `new Shiv()` then calls `.upgrade()`, so the potion always produces Shiv+ (1 damage → 6 damage). Rust adds `"Shiv"` literals to hand. The runtime registry likely has `"Shiv+"` registered for Storm of Steel. | Change `AddCard("Shiv", …)` to `AddCard("Shiv+", …)` in the cunning_potion def. |
| PT5 | Duplication Potion status is never consumed | bug | `packages/engine-rs/src/potions/defs/duplication_potion.rs:4`, search for `DUPLICATION` consumer | `DuplicationPotion.java:38-43`, `powers/DuplicationPower` | Rust sets `sid::DUPLICATION` on the player but nothing in the card-play pipeline checks for it to duplicate the next card played and decrement the counter. Java's DuplicationPower triggers on `onUseCard` to re-queue the action, decrement amount, and remove at 0. | Add a Trigger::OnCardPlayed hook (or interpreter hook) reading `sid::DUPLICATION`, re-executing the card's effect queue, and decrementing. |
| PT6 | Elixir exhausts entire hand instead of offering a selectable choice | deferred | `packages/engine-rs/src/potions/defs/elixir.rs:13-19` | `Elixir.java:38-43`, `ExhaustAction.java:25-35` | Java constructs `new ExhaustAction(false, true, true)` (not random, anyNumber, canPickZero) — opens a hand-select screen allowing the player to pick any subset (including none). Rust drains the entire hand into the exhaust pile unconditionally. This is a meaningful downgrade for decks that use it to save a single expensive card. | Introduce a ChoiceReason::ExhaustAnyFromHand with pick-any-number semantics; invoke it from the Elixir hook. |
| PT7 | Entropic Brew fills slots with fixed Block Potion proxy | intentional | `packages/engine-rs/src/potions/defs/entropic_brew.rs:13-20`, `potions/mod.rs:129-138` | `EntropicBrew.java:34-47`, `ObtainPotionAction`, `returnRandomPotion` | Java calls `returnRandomPotion(true)` for each empty potion slot (random weighted pick). The comment in mod.rs calls this an MCTS proxy. For combat sims the exact identity of the next potion is rarely resolved, but this silently biases downstream policy toward block. | Keep as intentional in MCTS, but record as a known simplification. If re-enabled properly, use the same weighted sampler the engine uses for shop/drop potion rolls. |
| PT8 | Essence of Darkness ignores Sacred Bark scaling | bug | `packages/engine-rs/src/potions/defs/essence_of_darkness.rs:20-24` | `EssenceOfDarkness.java:38-45`, `EssenceOfDarknessAction.java:19-25` | Java channels `potency` Dark orbs per slot (nested loops: `for i in 0..orbs.size() for j in 0..amount channelOrb(Dark)`). Base potency 1, so 1-per-slot matches. With Sacred Bark, potency becomes 2, giving 2 Darks per slot. Rust hardcodes one Dark per slot regardless of the Sacred Bark relic. | Read `effective_potency_runtime` and use it as the inner-loop count. |
| PT9 | Fairy revive lacks the `max(1)` heal clamp | deferred | `packages/engine-rs/src/potions/mod.rs:573-589`, `engine.rs:2441-2464` | `FairyPotion.java:32-39` | Java `FairyPotion.use` computes `healAmt = max(1, maxHealth * potency / 100)`. Rust `check_fairy_revive_scaled` returns `(max_hp * potency) / 100` with no clamp. Edge case: fails only at extremely low max HP, so practically this doesn't fire, but it's still a divergence. | Add `.max(1)` to the revive amount to match Java exactly. |
| PT10 | Fruit Juice ignores Sacred Bark | bug | `packages/engine-rs/src/potions/defs/fruit_juice.rs:4` | `FruitJuice.java:33-36` | Rust uses `A::Fixed(5)` so Sacred Bark has no effect. Java uses `AbstractDungeon.player.increaseMaxHp(potency, true)` with potency doubled by Sacred Bark (5 → 10 max HP and heal). | Change to `A::PotionPotency` and rely on `effective_potency_runtime` to scale. |
| PT11 | Gambler's Brew discards entire hand unconditionally | deferred | `packages/engine-rs/src/potions/defs/gamblers_brew.rs:13-22` | `GamblersBrew.java:31-36`, `GamblingChipAction.java:36-59` | Java opens a hand-card-select screen with limit 99 (`anyNumber=true`, `canPickZero=true`) — pick any subset to discard, draw that many. Rust discards the whole hand. Policy loses the option to keep good cards and rip-replace only the bad ones. | Add a ChoiceReason::DiscardAnyFromHand (anyNumber, canPickZero) and drive draw from the selected-count. |
| PT12 | Snecko Oil applies spurious Confusion status | bug | `packages/engine-rs/src/potions/defs/snecko_oil.rs:6-9`, `potions/mod.rs:338-343` | `SneckoOil.java:34-39`, `RandomizeHandCostAction.java:23-32` | Java's SneckoOil draws `potency` cards and then calls `RandomizeHandCostAction`, which re-rolls `card.cost` on every card currently in hand to `random(0..3)`. Java never applies the `Confused` power. Rust sets `sid::CONFUSION = 1` in addition to scheduling draw, so the Snecko *power* effect persists and keeps randomizing costs on draw for the rest of combat. | Remove the `sid::CONFUSION` effect; instead, emit a one-shot "randomize-hand-cost" effect consumed immediately by the interpreter (similar to how Rust would handle RandomizeHandCostAction). |
| PT13 | A11+ potency reduction is test-only; production combat always uses base potency | unverified | `packages/engine-rs/src/potions/mod.rs:158-211` | `AbstractPotion.java:637-645` + each `getPotency(int ascensionLevel)` override | The `potion_potency` table, `effective_potency`, and `apply_potion_scaled` are gated on `#[cfg(test)]` or use `ascension=0` in `effective_potency_runtime`. `potions/mod.rs:203-211` documents this explicitly as "the current combat engine path does not thread ascension into CombatState". Because ascension is part of run config but never reaches combat potency, A11+ values (e.g. Fire 20 → 15, Fruit Juice 5 → 3) silently regress toward base. Watcher A0 base is a11=0 so training is unaffected today, but the gap exists. | Thread the run's ascension level into `CombatState` (or a combat bootstrap arg) and pass it into `effective_potency_runtime`. |

## Items verified clean

- Fire Potion damage magnitude and Vulnerable interaction (`defs/fire_potion.rs:4`, damage goes through `DealDamage` which respects Vulnerable/Intangible/Invincible in `deal_damage_to_enemy` / the declarative interpreter).
- Explosive Potion hits all living enemies (`defs/explosive_potion.rs:4`, `T::AllEnemies` target mode).
- Block Potion grants 12 block, doubled to 24 with Sacred Bark (`defs/block_potion.rs` + `effective_potency_runtime` path; `test_sacred_bark_doubles_block`).
- Strength / Dexterity / Focus potions grant +2 of each (Sacred Bark doubles to +4).
- Flex (Steroid) and Speed potions correctly stack STRENGTH/LOSE_STRENGTH and DEXTERITY/LOSE_DEXTERITY with matching potency, so net buff unwinds at end of turn.
- Weak, Fear, Poison potions hit the selected enemy for their canonical stacks (3 Weak, 3 Vulnerable, 6 Poison).
- Energy Potion adds 2 energy (`defs/energy_potion.rs`).
- Swift Potion sets `POTION_DRAW = 3`, so the draw handling matches Java's `DrawCardAction(player, potency)`.
- Heart of Iron, Liquid Bronze, Essence of Steel, Regen Potion apply Metallicize / Thorns / Plated Armor / Regeneration with correct base values and respect Sacred Bark through `A::PotionPotency`.
- Ghost in a Jar applies 1 Intangible (matches Java `IntangiblePlayerPower(target, potency)`).
- Liquid Memories uses a discard-pile choice flow and applies `cost = 0` to the returned cards; correctly falls through to "move all" when the discard pile is smaller than the pick count.
- Distilled Chaos plays the top 3 (6 with Sacred Bark) cards for free via the real card pipeline, including auto-targeting the first living enemy for Enemy-targeted cards.
- Smoke Bomb flees combat, and `potion_can_use_in_combat` blocks boss combats and enemies exposing BackAttack (`potions/mod.rs:77-91`).
- Bottled Miracle matches Java — `new Miracle()` in Java is unupgraded, and Rust adds `"Miracle"` (also unupgraded).
- Stance Potion correctly exposes a Wrath/Calm declarative choice (matches Java `ChooseOneAction` with `ChooseWrath`/`ChooseCalm`).
- Ambrosia enters Divinity stance (matches Java `ChangeStanceAction("Divinity")`).
- Blessing of the Forge upgrades all upgradeable cards in hand; non-upgradeable cards (Curses, Status, Shivs) are silently skipped, matching Java's `canUpgrade()` gate.
- Attack / Skill / Power / Colorless potions open a 3-card Discovery choice from the appropriate pool, with Sacred Bark adding a second copy via `effective_potency_runtime` (the fixed "Strike/Defend/Smite" path in `potions/mod.rs` is `#[cfg(test)]` only; `mod.rs:524` confirms runtime authority).
- Fairy in a Bottle passive revive: 30% max HP base (20% at A11+), doubled to 60%/40% with Sacred Bark (`check_fairy_revive_scaled`), wired into `engine.rs:2441` and the `combat_hooks.rs:226` / `status_effects.rs` death paths. Rust consumes the fairy slot (sets to empty string) before assigning HP.
- Potion of Capacity applies +2 ORB_SLOTS (matches Java `IncreaseMaxOrbAction(potency)` provided the engine has an ORB_SLOTS → `orb_slots.slot_count` handler).
- Target requirements match Java 1:1: `potion_requires_target` at `potions/mod.rs:141-153` declares Fire, Weak, Fear, Poison as targeted, which matches the four `targetRequired = true` entries in Java.
- Potion slot handling: empty slots in `CombatState.potions` are represented as empty strings; Rust doesn't need the `PotionSlot` placeholder class.

## Follow-up questions

1. Should PT3 (Cultist Potion → player Ritual) be a combat-correctness priority? The potion is uncommon and Watcher-relevant in a few builds. If low priority, reclassify as `deferred`.
2. Should PT5 (Duplication Potion → player DUPLICATION consumer) block on wiring a generic `onCardPlayed` decrement-status hook, or should Duplication share infrastructure with a future Wraith Form / Mayhem-style Next-Card consumer?
3. For PT13 (A11+ potency), is the plan to thread `ascension_level` through `CombatState` from `RunState`, or do training seeds always use A0/A1 such that the gap is acceptable? The reference Watcher run is A0 so base potency is correct for the current corpus.
4. PT7 (Entropic Brew proxy) should be double-checked against what the search/MCTS actually does with the generated proxy slots — if the agent treats "Block Potion" as a guaranteed outcome during planning, the value function may be biased. Is the proxy exposed to policy training?
5. PT11 / PT6 (Gambler's Brew / Elixir hand-select): if introducing a ChoiceReason::*AnyFromHand is a larger combat-UX overhaul, these should likely stay `deferred` for this cycle. Flagging here to confirm scope.
