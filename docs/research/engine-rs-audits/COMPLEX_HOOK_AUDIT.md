# Complex Hook Audit

Snapshot date: 2026-04-13

This audit focuses on the remaining `complex_hook` surface in `packages/engine-rs` and asks two questions:

1. Where are the remaining hooks actually overlapping?
2. What trigger or effect-program primitives would collapse the most hooks at once?

The parity reference for "how Java does this" is now available locally at:

- `decompiled/java-src/com/megacrit/cardcrawl/...`

The local Python parity layer and docs in `packages/engine` remain useful as a semantic index:

- `packages/engine/registry/__init__.py`
- `packages/engine/content/powers.py`
- `packages/engine/combat_engine.py`
- `packages/engine/registry/powers.py`
- `packages/engine/registry/relics.py`
- `packages/engine/registry/potions.py`

## Current Snapshot

- Card hook files: `71`
- Power hook defs: `17`
- Relic hook defs: `23`
- Potion hook defs: `13`
- Card files with non-empty `effect_data`: `208 / 352`
- Card files still using `complex_hook`: `71 / 352`

The tail is no longer "missing card defs." It is mostly "missing reusable runtime primitives."

## Main Finding

The remaining hook surface is not especially diverse. Most of it collapses into seven reusable families:

1. Card-play event phases and replay windows
2. Zone selection and pile movement
3. Random card generation and discover-style choices
4. Orb lifecycle events and orb-count amount sources
5. Damage/HP-loss/debuff/block-broken follow-up events
6. Combat-scoped hidden state and delayed resolution
7. Played-card instance mutation and stat-equivalent copies

Java-style parity already names these families at the right abstraction level. The Rust runtime still has the higher-level `GameplayDef` envelope, but most entities still execute as `AdaptedLegacy` in `src/gameplay/types.rs`.

## Java-Style Trigger Families To Mirror

The Python/Java-parity registry already exposes the shape we want:

- Card phases in `packages/engine/registry/__init__.py`
  - `onPlayCard`
  - `onUseCard`
  - `onAfterUseCard`
  - `onAfterCardPlayed`
  - `onExhaust`
  - `onManualDiscard`
- Damage phases in `packages/engine/content/powers.py`
  - `atDamageGive`
  - `atDamageReceive`
  - `onAttackedToChangeDamage`
  - `onAttack`
  - `onAttacked`
  - `wasHPLost`
  - `onApplyPower`
- Orb phases in `packages/engine/registry/__init__.py`
  - `onChannelOrb`
  - `onEvokeOrb`
- Turn phases in `packages/engine/registry/__init__.py`
  - `atBattleStartPreDraw`
  - `atTurnStart`
  - `atTurnStartPostDraw`
  - `atEndOfTurnPreEndTurnCards`
  - `atEndOfRound`

The relevant Java ordering note is already preserved in `packages/engine/combat_engine.py:1207`:

- `onPlayCard` while the card is still in hand
- card effects resolve
- `onUseCard`
- destination handling after that

That separation explains a large fraction of the remaining Rust hook overlap.

## Card Hook Families

### 1. Zone Selection And Tutoring

Representative files:

- `src/cards/colorless/discovery.rs`
- `src/cards/colorless/secret_technique.rs`
- `src/cards/watcher/foreigninfluence.rs`
- `src/cards/watcher/omniscience.rs`
- `src/cards/defect/seek.rs`
- `src/cards/ironclad/headbutt.rs`
- `src/cards/ironclad/dual_wield.rs`

Shared shape:

- choose `N` cards from a zone
- zone is one of hand/draw/discard/exhaust
- then move selected cards to hand/top/play/free-play/copy

Needed primitive:

- `SelectFromZone { zone, filter, min, max, destination, ordering }`
- `MoveSelectedCard`

Why this matters:

- this single primitive family removes a big share of choice-driven card hooks
- it also helps relics like `Gambling Chip` and potions like `Liquid Memories`

### 2. Pile Choreography

Representative files:

- `src/cards/silent/calculated_gamble.rs`
- `src/cards/ironclad/burning_pact.rs`
- `src/cards/colorless/purity.rs`
- `src/cards/ironclad/true_grit.rs`
- `src/cards/ironclad/second_wind.rs`
- `src/cards/ironclad/fiend_fire.rs`
- `src/cards/defect/reboot.rs`
- `src/cards/silent/storm_of_steel.rs`
- `src/cards/colorless/forethought.rs`

Shared shape:

- batch move cards between hand/draw/discard/exhaust
- often preserve order or apply a follow-up draw/block/energy effect
- sometimes attach per-card side effects like `on_card_discarded`

Needed primitive:

- `MoveCardBatch { from, to, filter, selection, preserve_order, mutate_cost }`
- `AfterBatchResolution`

Why this matters:

- this is the single biggest card-hook family after replay/random generation
- it also covers discard-driven relic/power interactions more honestly than bespoke hooks

### 3. Random Generation And Discover

Representative files:

- `src/cards/colorless/chrysalis.rs`
- `src/cards/colorless/metamorphosis.rs`
- `src/cards/colorless/jack_of_all_trades.rs`
- `src/cards/ironclad/infernal_blade.rs`
- `src/cards/silent/distraction.rs`
- `src/cards/colorless/transmutation.rs`
- `src/cards/colorless/madness.rs`

Shared shape:

- generate from a typed pool or fixed pool
- add to hand/draw/discard
- sometimes set cost to `0`
- sometimes present a choose-one discover frame

Needed primitive:

- `GenerateCard { pool, count, destination, cost_rule, upgrade_rule, temporary }`
- `DiscoverCardChoice`

Why this matters:

- it removes both card hooks and potion hooks
- it gives deterministic RNG labels for RL replay/search

### 4. Orb Lifecycle

Representative files:

- `src/cards/defect/barrage.rs`
- `src/cards/defect/chaos.rs`
- `src/cards/defect/consume.rs`
- `src/cards/defect/darkness.rs`
- `src/cards/defect/fission.rs`
- `src/cards/defect/tempest.rs`
- `src/cards/defect/thunder_strike.rs`
- `src/cards/defect/redo.rs`

Shared shape:

- channel orb(s)
- evoke one/all orbs
- count orb types or slots
- trigger passive orbs or use front-orb identity

Needed primitive:

- `ChannelOrb`
- `EvokeOrb`
- `EvokeAllOrbs`
- `TriggerOrbPassive`
- `CountOrbsByType`
- `OrbSlotsChangedEvent`

Why this matters:

- the same family also removes several Defect relic hooks

### 5. State-Scaled Damage/Block/Heal

Representative files:

- `src/cards/colorless/mind_blast.rs`
- `src/cards/watcher/pressurepoints.rs`
- `src/cards/watcher/judgement.rs`
- `src/cards/watcher/wallop.rs`
- `src/cards/watcher/spiritshield.rs`
- `src/cards/silent/flechettes.rs`
- `src/cards/silent/bane.rs`
- `src/cards/ironclad/feed.rs`
- `src/cards/ironclad/reaper.rs`
- `src/cards/ironclad/sword_boomerang.rs`
- `src/cards/defect/blizzard.rs`

Shared shape:

- amount depends on combat state
- examples: draw pile size, hand composition, enemy status, orb counters, kill result
- several are really "standard hit plus post-hit scaling/follow-up"

Needed primitive:

- richer `AmountSource`
- `CombatPredicate`
- `HitResultAmountSource`
- `ForEachLivingEnemy`
- `ForRandomEnemyRepeated`

Why this matters:

- these hooks are not all missing triggers
- many are missing expressive amount sources and target iterators

### 6. Hidden Runtime State, Copy, Replay, Delayed Effects

Representative files:

- `src/cards/watcher/collect.rs`
- `src/cards/watcher/fasting.rs`
- `src/cards/watcher/scrawl.rs`
- `src/cards/defect/streamline.rs`
- `src/cards/defect/double_energy.rs`
- `src/cards/defect/doppelganger.rs`
- `src/cards/ironclad/havoc.rs`
- `src/cards/silent/nightmare.rs`
- `src/cards/watcher/wish.rs`
- `src/cards/watcher/conjureblade.rs`
- `src/cards/watcher/lessonlearned.rs`

Shared shape:

- install hidden combat-scoped state
- schedule next-turn effects
- free-play/replay another card
- resolve a named choice outcome with delayed application

Needed primitive:

- combat-scoped `EffectState` fields on cards
- `ReplayCard`
- `PlayCardFromZone`
- `ChoiceOutcomeEffect`
- `DelayedEffect { until_event }`

Why this matters:

- this is the family behind many remaining "I need a hook" claims even when the trigger is already correct

### 7. Played-Card Instance Mutation And Stat-Equivalent Copies

Representative Java files:

- `decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java`
- `decompiled/java-src/com/megacrit/cardcrawl/cards/red/Rampage.java`
- `decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SteamBarrier.java`
- `decompiled/java-src/com/megacrit/cardcrawl/cards/green/GlassKnife.java`
- `decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java`
- `decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/RitualDagger.java`

Current Rust shape:

- several of these cards still use player-global statuses such as `RAMPAGE_BONUS`, `STEAM_BARRIER_LOSS`, `GENETIC_ALG_BONUS`, and `GLASS_KNIFE_PENALTY`
- `Streamline` is now a confirmed Java-backed engine-path failure rather than an inferred risk: the played card returns to discard at effective cost `2` instead of carrying forward the reduced cost `1`

Needed primitive:

- mutate the played `CardInstance` before destination handling
- preserve that state across copies that should be stat-equivalent
- keep the card runtime compact and clone-friendly instead of adding more bespoke player-global statuses

Why this matters:

- this family is larger than a one-off `Streamline` fix
- it is one of the cleanest remaining opportunities to replace legacy hacks with a reusable runtime primitive

## Non-Card Hook Families

### 1. Card-Play Reaction Family

Representative files:

- `src/powers/defs/complex.rs`
- `src/powers/defs/card_play.rs`
- `src/relics/defs/orange_pellets.rs`
- `src/relics/defs/pocketwatch.rs`

Shared shape:

- react at different card-play phases
- count cards or card types
- sometimes replay the just-played card

Examples:

- `Echo Form`, `Double Tap`, `Burst`
- `Thousand Cuts`, `Panache`
- `Orange Pellets`
- `Pocketwatch`

Needed trigger family:

- `CardPlayedEvent { phase: Pre | Use | AfterUse | AfterPlayed, card_type, source_owner, target_owner, is_copy }`
- `ReplayRequested`
- `ReplayResolved`

Important current mismatch:

- `src/powers/defs/complex.rs` uses `Trigger::OnAnyCardPlayed` and `Trigger::OnCardPlayedPost`, but Java-style parity distinguishes `onUseCard`, `onAfterUseCard`, and `onAfterCardPlayed`.

### 2. Turn-Start / Pre-Draw Generation Family

Representative files:

- `src/powers/defs/turn_start.rs`
- `src/relics/defs/pure_water.rs`
- `src/relics/defs/holy_water.rs`
- `src/relics/defs/ninja_scroll.rs`
- `src/relics/defs/mark_of_pain.rs`
- `src/relics/defs/gambling_chip.rs`

Shared shape:

- add temp cards at combat start or post-draw
- open a discard choice after the starting hand is visible
- these want explicit pre-draw versus post-draw timing, not generic "combat start"

Needed trigger family:

- `BattleStartPreDraw`
- `TurnStartPostDraw`
- `TurnStartPostDrawLate`

Needed effect family:

- `GrantCardToHand`
- `GrantCardToDrawPile`
- `DrawThenOpenChoice`

### 3. Damage / HP-Loss / Debuff / Stance Family

Representative files:

- `src/powers/defs/complex.rs`
- `src/relics/defs/centennial_puzzle.rs`
- `src/relics/defs/red_skull.rs`
- `src/relics/defs/the_specimen.rs`
- `src/relics/defs/emotion_chip.rs`
- `src/relics/defs/preserved_insect.rs`
- `src/relics/defs/slavers_collar.rs`

Shared shape:

- post-hit retaliation or follow-up
- unblocked-damage only behavior
- debuff-applied reactions
- HP threshold activation
- enemy-death and victory timing

Needed trigger family:

- `OutgoingDamageAdjusted`
- `IncomingDamageAfterBlock`
- `HpLost`
- `DebuffApplied`
- `EnemyDeath`
- `Victory`
- `StanceChanged`

Important current mismatch:

- `Envenom` is currently on `Trigger::DamageResolved` in `src/powers/defs/complex.rs`, but Java-style parity is `onAttack` after dealing attack damage with richer context.

### 4. Orb / Owner-Aware Runtime Family

Representative files:

- `src/relics/defs/cracked_core.rs`
- `src/relics/defs/nuclear_battery.rs`
- `src/relics/defs/symbiotic_virus.rs`
- `src/relics/defs/frozen_core.rs`
- `src/relics/defs/emotion_chip.rs`

Shared shape:

- channel orb at combat start
- trigger orb passive at a later owner-aware point
- react to empty slots or slot changes

Needed trigger family:

- `OrbChanneled`
- `OrbEvoked`
- `OrbPassiveTriggered`
- `OrbSlotsChanged`

Needed effect family:

- `ChannelOrb`
- `TriggerFrontOrbPassive`
- `ChannelOrbIfEmptySlot`

### 5. Potion Use / Generated Card Family

Representative files:

- `src/potions/defs/attack_potion.rs`
- `src/potions/defs/skill_potion.rs`
- `src/potions/defs/power_potion.rs`
- `src/potions/defs/colorless_potion.rs`
- `src/potions/defs/distilled_chaos.rs`
- `src/potions/defs/liquid_memories.rs`
- `src/potions/defs/blessing_of_forge.rs`
- `src/potions/defs/elixir.rs`
- `src/potions/defs/gamblers_brew.rs`
- `src/potions/defs/entropic_brew.rs`
- `src/potions/defs/stance_potion.rs`
- `src/potions/defs/ambrosia.rs`
- `src/potions/defs/essence_of_darkness.rs`

Shared shape:

- manual activation
- optional target gating
- generate a card, move cards between piles, or install a temporary state effect

Needed trigger family:

- `PotionActivated`
- `PotionResolved`
- `PotionConsumed`

Needed effect family:

- `GenerateCard`
- `ReturnDiscardToHand`
- `ExhaustSelectedCards`
- `UpgradeSelectedCards`
- `ChangeStance`
- `ChannelOrb`
- `GrantPotionToEmptySlot`

## What Still Looks Truly Complex

Very little looks permanently irreducible.

The best current candidates to remain hook-backed the longest are:

- `Wish`
- `Nightmare`
- `Omniscience`
- `Lesson Learned`
- `Time Warp`

Even these do not need bespoke trigger families. They mostly need:

- better choice/result plumbing
- replay/copy semantics
- delayed effects
- rewardless but named outcome resolution

## Highest-Leverage Primitive Additions

If we want the next worker wave to delete the most hooks per unit of effort, the best order is:

1. `CardPlayedEvent` phases plus replay events
2. `SelectFromZone` plus `MoveSelectedCard`
3. `MoveCardBatch`
4. `GenerateCard`
5. owner-aware orb lifecycle ops/events
6. richer `AmountSource` and combat predicates
7. delayed hidden-state effects on cards/potions

## Worker Slice Recommendation

Use worker waves by primitive, not by character class:

1. `runtime-card-phases`
   - add `onPlayCard` / `onUseCard` / `onAfterUseCard` / `onAfterCardPlayed` payload-rich events
   - migrate `Echo Form`, `Double Tap`, `Burst`, `Thousand Cuts`, `Panache`, `Orange Pellets`, `Pocketwatch`
2. `runtime-zone-selection`
   - add `SelectFromZone` and `MoveSelectedCard`
   - migrate `Seek`, `Discovery`, `Headbutt`, `Secret Technique`, `Dual Wield`, `Liquid Memories`
3. `runtime-batch-moves`
   - add `MoveCardBatch`
   - migrate `Calculated Gamble`, `Burning Pact`, `Purity`, `True Grit`, `Second Wind`, `Fiend Fire`, `Storm of Steel`
4. `runtime-random-generation`
   - add `GenerateCard`
   - migrate random-card cards and potions together
5. `runtime-orb-family`
   - add orb events and orb-count amount sources
   - migrate Defect cards plus `Cracked Core`, `Nuclear Battery`, `Frozen Core`, `Emotion Chip`

## Bottom Line

The remaining hook surface does not argue for more bespoke registrars.

It argues for a smaller number of richer runtime primitives that match the Java-style semantic families already named in the local parity layer. Once those families exist, the remaining hook tail should collapse quickly across cards, relics, powers, and potions at the same time.
