# Implementation Spec: Python Slay the Spire Clone

This spec summarizes what is implemented vs missing for a full Python clone (all characters, cards, relics, events, potions, etc.). It is based on current engine content (`packages/engine/content`), registries/handlers, and tests in this repo.

## High-Level Status (Updated 2026-02-04)

- **Character support**: Run factories exist for Watcher/Ironclad/Silent/Defect (starting decks/relics + ascension HP). All character cards verified against Java.
- **Core mechanics (100% parity)**:
  - RNG system: 100% (all 13 streams)
  - Damage/block calc: 100% (order of operations exact)
  - Enemies: 100% (all 66 verified)
  - Stances: 100% (all 4 stances)
  - Cards: 100% (all characters verified)
  - Power triggers: 100%
  - Combat relics: 100%
  - Events: 100% (all handlers working)
  - Potions (data): 100%
- **Missing features (139 skipped tests)**:
  - Rest site relics: 36 tests
  - Relic pickup effects: 34 tests
  - Chest relic acquisition: 30 tests
  - Bottled relics: 20 tests
  - Out-of-combat triggers: 13 tests
- **Tests**: 4512 passing, 139 skipped; coverage ~68% (`uv run pytest tests/ --cov=packages/engine`).

## Useful Code Map (Where To Look)

- **Game loop**: `packages/engine/game.py` (GameRunner) and `packages/engine/combat_engine.py`
- **State**: `packages/engine/state/` (RNG, run/combat state)
- **Damage & combat math**: `packages/engine/calc/`
- **Content definitions**: `packages/engine/content/` (cards, relics, potions, enemies, events, powers, stances)
- **Effect system**: `packages/engine/effects/` (registry + executor)
- **Triggers & registries**: `packages/engine/registry/` (relic/power/potion triggers, passive relic flags)
- **Generation**: `packages/engine/generation/` (map, encounters, rewards, shop, treasure)
- **Handlers**: `packages/engine/handlers/` (event, reward, shop, rooms, combat)
- **Parity tooling**: `packages/parity/` (seed catalog, comparators, trackers)

## Cards (Per-Entity Status) - Updated 2026-02-04

**Status**: All cards for all 4 characters have been verified against Java decompiled source.

Totals by group (all supported):
- Watcher: 84 cards ✅
- Ironclad: 75 cards ✅
- Silent: 76 cards ✅
- Defect: 75 cards ✅
- Colorless: 39 cards ✅
- Curse: 14 cards ✅
- Status: 5 cards ✅

**Key fixes applied (2026-02-04)**:
- Ironclad: Berserk, Rupture, Limit Break, Body Slam, Corruption
- Silent: Wraith Form Artifact interaction, Burst end-of-turn, Bouncing Flask RNG
- Defect: Loop timing, Electrodynamics Lightning count
- Watcher: InnerPeace if_calm_draw_else_calm

## Relics (Per-Entity Status)

Status model used here:
- **Active** = trigger registered in `registry/relics.py`
- **Passive** = flag defined in `registry/relics_passive.py`

Counts (180 total): active_only=109, passive_only=20, active_and_passive=7, missing_all=44.

- Active only: `Akabeko`, `Anchor`, `Ancient Tea Set`, `Art of War`, `Bag of Marbles`, `Bag of Preparation`, `Bird Faced Urn`, `Black Blood`, `Blood Vial`, `Bloody Idol`, `Boot`, `Bottled Flame`, `Bottled Lightning`, `Bottled Tornado`, `Brimstone`, `Bronze Scales`, `Burning Blood`, `Calipers`, `CaptainsWheel`, `Centennial Puzzle`, `Champion Belt`, `CloakClasp`, `ClockworkSouvenir`, `CultistMask`, `Damaru`, `Darkstone Periapt`, `Dead Branch`, `Du-Vu Doll`, `Emotion Chip`, `Enchiridion`, `FossilizedHelix`, `Frozen Egg 2`, `Gambling Chip`, `Gremlin Horn`, `GremlinMask`, `Happy Flower`, `HolyWater`, `HornCleat`, `HoveringKite`, `Incense Burner`, `InkBottle`, `Inserter`, `Kunai`, `Lantern`, `Lee's Waffle`, `Letter Opener`, `Lizard Tail`, `Magic Flower`, `Mango`, `Mark of Pain`, `Matryoshka`, `Meat on the Bone`, `Mercury Hourglass`, `Molten Egg 2`, `Mummified Hand`, `MutagenicStrength`, `Necronomicon`, `Nilry's Codex`, `Ninja Scroll`, `Nuclear Battery`, `Nunchaku`, `Oddly Smooth Stone`, `Old Coin`, `Omamori`, `OrangePellets`, `Orichalcum`, `Ornamental Fan`, `Pantograph`, `Pear`, `Pen Nib`, `Philosopher's Stone`, `Pocketwatch`, `PureWater`, `Red Mask`, `Red Skull`, `Ring of the Snake`, `Runic Cube`, `Self Forming Clay`, `Shuriken`, `SlaversCollar`, `Sling`, `Snake Skull`, `Snecko Eye`, `StoneCalendar`, `Strange Spoon`, `Strawberry`, `StrikeDummy`, `Sundial`, `Symbiotic Virus`, `TeardropLocket`, `The Specimen`, `TheAbacus`, `Thread and Needle`, `Tingsha`, `Tiny Chest`, `Torii`, `Tough Bandages`, `Toxic Egg 2`, `Toy Ornithopter`, `TungstenRod`, `TwistedFunnel`, `Unceasing Top`, `Vajra`, `Velvet Choker`, `VioletLotus`, `War Paint`, `Whetstone`, `WristBlade`, `Yang`
- Passive only: `Black Star`, `Coffee Dripper`, `Cursed Key`, `Dream Catcher`, `Ectoplasm`, `Fusion Hammer`, `Ginger`, `Juzu Bracelet`, `Mark of the Bloom`, `Melange`, `Membership Card`, `Odd Mushroom`, `Paper Frog`, `Peace Pipe`, `Regal Pillow`, `Runic Pyramid`, `Smiling Mask`, `Sozu`, `The Courier`, `Turnip`
- Active + passive: `Blue Candle`, `Girya`, `Golden Idol`, `Ice Cream`, `Medical Kit`, `Paper Crane`, `Shovel`
- Missing all: `Astrolabe`, `Busted Crown`, `Cables`, `Calling Bell`, `Cauldron`, `CeramicFish`, `Charon's Ashes`, `Chemical X`, `Circlet`, `Cracked Core`, `DataDisk`, `Discerning Monocle`, `DollysMirror`, `Empty Cage`, `Eternal Feather`, `FaceOfCleric`, `Frozen Eye`, `FrozenCore`, `GoldenEye`, `HandDrill`, `MawBank`, `MealTicket`, `NeowsBlessing`, `Nloth's Gift`, `NlothsMask`, `Orrery`, `Pandora's Box`, `Potion Belt`, `Prayer Wheel`, `PreservedInsect`, `PrismaticShard`, `Question Card`, `Red Circlet`, `Ring of the Serpent`, `Runic Capacitor`, `Runic Dome`, `SacredBark`, `Singing Bowl`, `Spirit Poop`, `SsserpentHead`, `Tiny House`, `WarpedTongs`, `White Beast Statue`, `WingedGreaves`

## Potions (Per-Entity Status)

All 42 potions have registry handlers, but several are **stubbed/partial** or have TODOs in the combat engine.

Potions with handlers (all): `Ambrosia`, `Ancient Potion`, `AttackPotion`, `BlessingOfTheForge`, `Block Potion`, `BloodPotion`, `BottledMiracle`, `ColorlessPotion`, `CultistPotion`, `CunningPotion`, `Dexterity Potion`, `DistilledChaos`, `DuplicationPotion`, `ElixirPotion`, `Energy Potion`, `EntropicBrew`, `EssenceOfDarkness`, `EssenceOfSteel`, `Explosive Potion`, `FairyPotion`, `FearPotion`, `Fire Potion`, `FocusPotion`, `Fruit Juice`, `GamblersBrew`, `GhostInAJar`, `HeartOfIron`, `LiquidBronze`, `LiquidMemories`, `Poison Potion`, `PotionOfCapacity`, `PowerPotion`, `Regen Potion`, `SkillPotion`, `SmokeBomb`, `SneckoOil`, `SpeedPotion`, `StancePotion`, `SteroidPotion`, `Strength Potion`, `Swift Potion`, `Weak Potion`

Partial/stub behaviors to address:
- `ElixirPotion`: simplified to exhaust all cards (no selection UX yet).
- `FairyPotion`: auto-revive supported in combat runner; manual use no-op.
- `DistilledChaos`: still simplified (draws cards, doesn’t auto-play).
- `LiquidMemories`: simplified (no discard selection UI).
- `EntropicBrew`: deterministic fill (no potion RNG parity yet).
- `SmokeBomb`: boss-fight restriction not enforced.
- Discovery potions (`AttackPotion`, `SkillPotion`, `PowerPotion`, `ColorlessPotion`) auto-select instead of offering a 3‑choice.

## Powers (Per-Entity Status)

Power data defined: 94 total. Registry triggers implemented: 30. Missing triggers: 64.

With triggers: `BattleHymn`, `Bias`, `Brutality`, `Buffer`, `Choked`, `Combust`, `Constricted`, `DevaForm`, `Dexterity`, `Energized`, `Envenom`, `Evolve`, `Frail`, `Heatsink`, `Intangible`, `Juggernaut`, `LoseDexterity`, `Metallicize`, `Nirvana`, `Panache`, `Plated Armor`, `Poison`, `Regeneration`, `Rupture`, `Strength`, `Study`, `Thorns`, `Vigor`, `Vulnerable`, `Weakened`

Missing triggers: `Accuracy`, `Rushdown`, `After Image`, `Angry`, `Artifact`, `Barricade`, `BeatOfDeath`, `BlockReturnPower`, `Blur`, `Burst`, `CannotChangeStancePower`, `Controlled`, `Corruption`, `Creative AI`, `Curiosity`, `Dark Embrace`, `Demon Form`, `DevotionPower`, `Double Damage`, `Double Tap`, `Draw`, `Draw Reduction`, `Echo Form`, `Electro`, `Entangled`, `Equilibrium`, `EstablishmentPower`, `Fading`, `Feel No Pain`, `Fire Breathing`, `Flame Barrier`, `Flex`, `Focus`, `FreeAttackPower`, `GrowthPower`, `Infinite Blades`, `IntangiblePlayer`, `Invincible`, `Life Link`, `LikeWaterPower`, `Lockon`, `Mantra`, `MasterRealityPower`, `Mode Shift`, `No Draw`, `NoBlockPower`, `Noxious Fumes`, `OmegaPower`, `PathToVictoryPower`, `Pen Nib`, `Repair`, `Retain Cards`, `Sadistic`, `Slow`, `Split`, `Static Discharge`, `Storm`, `Thievery`, `Thousand Cuts`, `Time Warp`, `WaveOfTheHandPower`, `Foresight`, `Wraith Form`, `WrathNextTurnPower`

## Enemies (Per-Entity Status)

All enemy classes are implemented (parity verified). IDs: `AcidSlime_L`, `AcidSlime_M`, `AcidSlime_S`, `AwakenedOne`, `BanditBear`, `BanditChild`, `BanditLeader`, `BookOfStabbing`, `BronzeAutomaton`, `BronzeOrb`, `Byrd`, `Centurion`, `Champ`, `Chosen`, `CorruptHeart`, `Cultist`, `Dagger`, `Darkling`, `Deca`, `Donu`, `Exploder`, `FungiBeast`, `FuzzyLouseDefensive`, `FuzzyLouseNormal`, `GiantHead`, `GremlinFat`, `GremlinLeader`, `GremlinNob`, `GremlinThief`, `GremlinTsundere`, `GremlinWarrior`, `GremlinWizard`, `Healer`, `Hexaghost`, `JawWorm`, `Lagavulin`, `Looter`, `Louse`, `Maw`, `Mugger`, `Nemesis`, `Orb Walker`, `Reptomancer`, `Repulsor`, `Sentry`, `Serpent`, `Shelled Parasite`, `SlaverBlue`, `SlaverBoss`, `SlaverRed`, `SlimeBoss`, `SnakePlant`, `Snecko`, `SphericGuardian`, `SpikeSlime_L`, `SpikeSlime_M`, `SpikeSlime_S`, `Spiker`, `SpireShield`, `SpireSpear`, `Taskmaster`, `TheCollector`, `TheGuardian`, `TimeEater`, `TorchHead`, `Transient`, `WrithingMass`

## Events (Per-Entity Status)

Data definitions: 51 (`content/events.py`). Runtime handler definitions: 50 (`handlers/event_handler.py`). Choice generators implemented: 17/50. Missing handlers: `GremlinMatchGame`, `GremlinWheelGame`.
Event ID alias normalization is now handled in `handlers/event_handler.py` to map content IDs to handler IDs.

Content-only event IDs (exist in `content/events.py` but not in handler definitions; mostly spacing/ID mismatches):
- `Accursed Blacksmith`, `Back to Basics`, `Big Fish`, `Bonfire Elementals`, `Cursed Tome`, `Dead Adventurer`, `Drug Dealer`, `Forgotten Altar`, `Fountain of Cleansing`, `Golden Idol`, `Golden Shrine`, `Golden Wing`, `Knowing Skull`, `Lab`, `Liars Game`, `Living Wall`, `Masked Bandits`, `Match and Keep!`, `Mysterious Sphere`, `N'loth`, `NoteForYourself`, `Scrap Ooze`, `Shining Light`, `The Cleric`, `The Joust`, `The Library`, `The Mausoleum`, `The Moai Head`, `The Woman in Blue`, `Tomb of Lord Red Mask`, `Transmorgrifier`, `Upgrade Shrine`, `Wheel of Change`, `Winding Halls`, `World of Goop`

Handler-only IDs (exist in `handlers/event_handler.py` but not in `content/events.py`):
- `AccursedBlacksmith`, `Augmenter`, `BackToBasics`, `BigFish`, `BonfireElementals`, `CursedTome`, `DeadAdventurer`, `ForgottenAltar`, `FountainOfCleansing`, `GoldenIdol`, `GoldenShrine`, `GremlinMatchGame`, `GremlinWheelGame`, `KnowingSkull`, `LivingWall`, `MaskedBandits`, `MoaiHead`, `MysteriousSphere`, `Nloth`, `ScrapOoze`, `ShiningLight`, `Sssserpent`, `TheCleric`, `TheJoust`, `TheLab`, `TheLibrary`, `TheMausoleum`, `TombOfLordRedMask`, `Transmogrifier`, `UpgradeShrine`, `WindingHalls`, `WingStatue`, `WomanInBlue`, `WorldOfGoop`

Handler-defined events missing choice generators:
- `AccursedBlacksmith`, `Addict`, `Augmenter`, `BackToBasics`, `Beggar`, `BonfireElementals`, `CursedTome`, `DeadAdventurer`, `Designer`, `FaceTrader`, `Falling`, `ForgottenAltar`, `FountainOfCleansing`, `Ghosts`, `GremlinMatchGame`, `GremlinWheelGame`, `MoaiHead`, `Mushrooms`, `MysteriousSphere`, `Nest`, `Nloth`, `SecretPortal`, `SensoryStone`, `ShiningLight`, `Sssserpent`, `TheJoust`, `TheLab`, `TombOfLordRedMask`, `Vampires`, `WeMeetAgain`, `WindingHalls`, `WingStatue`, `WomanInBlue`

## Stances

Watcher stances implemented: `Neutral`, `Calm`, `Wrath`, `Divinity`.

## Tests & Coverage

- **Coverage**: ~68% (tests cover `packages/engine` per `CLAUDE.md`).
- **Latest full test run** (`uv run pytest tests/ -ra`, 2026-02-04): collected 3950 tests; aborted during collection with 3 import errors:
  - `tests/test_ascension.py`: `WATCHER_BASE_GOLD` import no longer exists (replaced by `BASE_STARTING_GOLD`).
  - `tests/test_coverage_boost.py`: `EventHandler` moved out of `handlers.rooms`.
  - `tests/test_handlers.py`: `EventHandler` moved out of `handlers.rooms`.
- **XFail inventory (140 matches across 8 files)**:
  - `tests/test_relic_rest_site.py` (37)
  - `tests/test_relic_bottled.py` (20)
  - `tests/test_relic_pickup.py` (34)
  - `tests/test_relic_acquisition.py` (30)
  - `tests/test_relic_triggers_outofcombat.py` (14)
  - `tests/test_relic_card_rewards.py` (3)
  - `tests/test_audit_relics_cardplay.py` (1)
  - `tests/test_enemy_ai_parity.py` (1)
- **Incomplete tests**:
  - `tests/test_rng_parity.py` has a `NotImplementedError` placeholder for expected cards.
  - `tests/test_rng.py` has TODO placeholders for expected values.

## TODOs / Stubs Index

Known TODOs and pass stubs (non-exhaustive):
- `packages/engine/combat_engine.py`: Smoke Bomb boss check; Distilled Chaos; Liquid Memories; Entropic Brew.
- `packages/engine/registry/relics.py`: Defect orb TODOs (multiple relics); Ice Cream energy carry; Blue Candle/Medical Kit play behavior.
- `packages/engine/game.py`: enemy count tracking for rewards.
- `packages/engine/handlers/reward_handler.py`: action classes are `pass` (Gold/Potion/Relic/Key claim & proceed).
- `packages/engine/handlers/combat.py`: Snecko Eye cost randomization, Ice Cream energy carry, and other relic hooks are `pass`.
- `packages/engine/generation/rewards.py`: `on_uncommon` no-op.
- `packages/engine/generation/shop.py`: fallback `pass` when retrying shop card rolls.
- `tests/test_rng_parity.py`, `tests/test_rng.py`, `docs/vault/stsrlsolver-analysis.md`: TODO placeholders remain.
- `packages/engine/effects/cards.py`: several marker effects are `pass` but intentionally handled elsewhere.

## Docs Inventory

- `docs/ARCHITECTURE.md` (core architecture; currently references `core/` paths that no longer exist).
- `docs/vault/` (mechanics ground truth): RNG, damage, relic effects, map generation, events, card rewards, verified seeds, etc.
  - Notable: `rng-system-analysis.md`, `damage-mechanics.md`, `event-mechanics.md`, `relic-effects.md`, `shop-mechanics.md`, `map-generation.md`, `watcher-cards-complete.md`.

## Cleanup & Consolidation Opportunities

- **Path/packaging alignment**: README and `pyproject.toml` now reflect `packages/engine`, but keep docs and tooling consistent as the layout evolves.
- **Events duplication**: event definitions exist in both `content/events.py` and `handlers/event_handler.py` with mismatched IDs; alias normalization exists but full consolidation remains.
- **TDD placeholders**: many `xfail` tests are placeholders; either implement or remove/retag to avoid masking regressions.

## Work Units (Small-Model Tasks)

These unit-sized tasks are split by domain to keep scope manageable and parallelizable:

- Cards (Watcher): [docs/work_units/cards-watcher.md](docs/work_units/cards-watcher.md)
- Cards (Ironclad): [docs/work_units/cards-ironclad.md](docs/work_units/cards-ironclad.md)
- Cards (Silent): [docs/work_units/cards-silent.md](docs/work_units/cards-silent.md)
- Cards (Defect): [docs/work_units/cards-defect.md](docs/work_units/cards-defect.md)
- Potions: [docs/work_units/potions.md](docs/work_units/potions.md)
- Powers: [docs/work_units/powers.md](docs/work_units/powers.md)
- Events: [docs/work_units/events.md](docs/work_units/events.md)
- Rewards: [docs/work_units/rewards.md](docs/work_units/rewards.md)
- Relics: [docs/work_units/relics.md](docs/work_units/relics.md)

Ultra-granular checklists (per-category):
- Action spec (model-facing): [docs/work_units/granular-actions.md](docs/work_units/granular-actions.md)
- Agent interface: [docs/work_units/granular-agent-interface.md](docs/work_units/granular-agent-interface.md)
- Observation schema: [docs/work_units/granular-observation.md](docs/work_units/granular-observation.md)
- Determinism & RNG: [docs/work_units/granular-determinism.md](docs/work_units/granular-determinism.md)
- Phase flow: [docs/work_units/granular-phase-flow.md](docs/work_units/granular-phase-flow.md)
- Map visibility: [docs/work_units/granular-map-visibility.md](docs/work_units/granular-map-visibility.md)
- Cards (Watcher): [docs/work_units/granular-cards-watcher.md](docs/work_units/granular-cards-watcher.md)
- Cards (Ironclad): [docs/work_units/granular-cards-ironclad.md](docs/work_units/granular-cards-ironclad.md)
- Cards (Silent): [docs/work_units/granular-cards-silent.md](docs/work_units/granular-cards-silent.md)
- Cards (Defect): [docs/work_units/granular-cards-defect.md](docs/work_units/granular-cards-defect.md)
- Defect orbs: [docs/work_units/granular-orbs.md](docs/work_units/granular-orbs.md)
- Potions: [docs/work_units/granular-potions.md](docs/work_units/granular-potions.md)
- Powers: [docs/work_units/granular-powers.md](docs/work_units/granular-powers.md)
- Events: [docs/work_units/granular-events.md](docs/work_units/granular-events.md)
- Rewards: [docs/work_units/granular-rewards.md](docs/work_units/granular-rewards.md)
- Relics: [docs/work_units/granular-relics.md](docs/work_units/granular-relics.md)

Granular checklists incorporate the latest failed/skip test mappings (2026-02-04).
Model‑facing actions are prioritized over UI (choices should be traversable via explicit actions). Parameter signatures are explicit in the granular specs and defined in `granular-actions.md`.

## Agent Readiness Gates (minimum for RL)
1. **Action API**: `get_available_actions()` / `take_action()` adhere to `granular-actions.md` with deterministic IDs and no dead‑ends.
2. **Observation schema**: `get_observation()` returns stable, JSON‑serializable payloads per `granular-observation.md`.
3. **Determinism**: RNG streams and counters are synchronized; identical seed+actions yield identical outcomes (`granular-determinism.md`).
4. **Phase flow**: transitions obey the state machine (`granular-phase-flow.md`) and never strand the agent.
5. **Decision coverage**: rewards, events, and potions expose explicit selection actions (including boss relic skip).
6. **Map visibility**: current‑act map and `available_paths` are observable; `path_choice` actions align to map indices.

### Work Units Coverage Audit
- All `docs/work_units/granular-*.md` checklists include explicit `(action: ...)` tags.
- No legacy IDs are referenced in work_units docs (modern names only).
- All work_units docs are linked in this spec for discoverability.
- Map visibility spec is linked and covered by observation/action requirements.

## RL Readiness (Engine-Only Next Steps)

1. **Character completion**: run factories added for Ironclad/Silent/Defect; remaining work is effect parity (cards/powers/relics).
2. **Defect orbs system**: channel/evoke mechanics, orb slots, focus, and all orb-triggered relics/powers/cards.
3. **Card effects**: implement missing effect handlers for Ironclad/Silent/Defect and the single Watcher gap (`InnerPeace`).
4. **Relics**: implement missing active triggers (44 relics missing all + xfail buckets: bottled, pickup, acquisition, rest-site).
5. **Potions**: finish TODOs and discovery/selection logic for interactive potions.
6. **Events**: unify definitions and implement missing handlers/choice generators.
7. **Rewards/actions**: implement reward handler actions and ensure reward resolution mirrors Java.

### Watcher RL readiness (current max)
Safe (high parity): RNG, damage/block, enemy AI, Watcher stances.
Cautious (partial parity): potions, powers, relic triggers, events.
Risky (low fidelity): reward action processing, cross-class systems (Prismatic Shard/Defect orbs).

Suggested constraints if starting now:
- Prefer Watcher-only runs; avoid Prismatic Shard and cross-class dependencies.
- Treat event rooms as low-fidelity unless you implement missing choice generators.
- Avoid potions with incomplete behavior (discovery, Distilled Chaos, Liquid Memories, Entropic Brew, Smoke Bomb).

Training plan (doc-only): [docs/RL_TRAINING_PLAN.md](docs/RL_TRAINING_PLAN.md)

