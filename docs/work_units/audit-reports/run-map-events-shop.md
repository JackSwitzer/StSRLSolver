# Run / Map / Events / Shop Parity Audit (Dungeon-Layer)

## Summary (ranked by training impact)

The dungeon-layer is functional but heavily simplified relative to Java. Most deviations are consistent with the branch's documented "combat-first" scope — strategic/pathing is deferred — so the majority are `deferred`, not `bug`. However, five issues actively corrupt the combat-training signal and should be treated as bugs. In priority order for training impact:

1. **Event pool never removes used events (`RN-EV-01`)** — same event can repeat on consecutive `?` rooms in one run; distorts deck-shaping stats and over-exposes certain combat branches (Colosseum, Drug Dealer J.A.X., Nest Ritual Dagger).
2. **No shrine/one-time integration (`RN-EV-02`)** — 17 defined shrine events are unreachable from `enter_event`, so Accursed Blacksmith / Bonfire / Designer / etc. never trigger in rollouts.
3. **Event filter conditions absent (`RN-EV-03`)** — Dead Adventurer, Mushrooms, Cleric, Beggar, Colosseum, Moai Head all can fire in states Java forbids, poisoning the learned value-head.
4. **Neow reduced to 4 fixed options (`RN-NEOW-01`)** — no drawback pairings, no boss-relic swap, no HP/potion/remove-card. First-floor equity is mis-modeled.
5. **Ascension elite 2.5x above A14 missing (`RN-MAP-01`)** — elite count under-reported on A15+ seeds.

All other gaps (relics/potions in shop, chest tier distribution, card reward upgrade chance, map common-ancestor-gap, campfire Toke/Dig/Lift/Recall) are correctly deferred; they widen strategic space but don't mislead the combat-first critic.

---

## Subsystem coverage matrix

| Subsystem | Java reference | Rust impl | Coverage | Key gap |
|---|---|---|---|---|
| Map path generation | `map/MapGenerator.java:133-211` | `map.rs:215-289` | **Partial** | Missing `_createPaths` common-ancestor-gap check (`min=3, max=5`) |
| Row/room type assignment | `map/RoomTypeAssigner.java:30-145` | `map.rs:312-492` | **Good** | Act-specific ratios hardcoded to Exordium; no A15+ 2.5x elite |
| Room entry dispatch | `dungeons/AbstractDungeon.java:generateMap/generateRoomTypes` | `run.rs` RunPhase state machine | **Good** | N/A |
| Event selection RNG | `AbstractDungeon.generateEvent:1854-1980` | `run.rs:2915-2924` | **Missing** | No shrineChance gate, no pool removal, no filters |
| Event content (act 1/2/3) | `events/exordium/*.java` etc. | `events/exordium.rs`, `city.rs`, `beyond.rs` | **Partial** | 11+15+9 events defined; effects simplified in places |
| Shrine events | `events/shrines/*.java` + specialOneTimeEventList | `events/shrines.rs` | **Defined but unreachable** | Module not wired into enter_event |
| Shop - cards | `shop/ShopScreen.java:243-273` | `run.rs:2531-2560` | **Partial** | No colorless, no sale tag, wider price RNG |
| Shop - relics/potions | `ShopScreen.initRelics/initPotions` | — | **Missing** | 0 relics, 0 potions in shop |
| Shop - purge | `ShopScreen.purgeCard:275-289` | `run.rs:compute_shop_remove_price` | **Good** | Matches Java (75+25*n, Smiling Mask=50, Courier 0.8x, Member 0.5x) |
| Campfire | `ui/campfire/*.java` | `run.rs:2486-2525` | **Partial** | Only Rest + Smith; no Toke/Lift/Dig/Recall |
| Neow | `neow/NeowReward.java:435` | `run.rs:586-624` | **Stub** | 4 fixed options, no drawbacks, no boss relic swap |
| Treasure chests | `rewards/chests/*.java` | `run.rs:1813-1869` | **Stub** | No chest-tier distribution, always gold+relic |
| Card reward rolling | `AbstractDungeon.returnRandomCard + Exordium/TheCity/TheBeyond.cardUpgradedChance` | `run.rs:1659-1682` | **Partial** | 60/33/7 rarity; no per-act upgrade chance |
| Reward relic pool | `AbstractDungeon.returnRandomRelic(Tier)` | `run.rs:2314-2340` | **Stub** | Hardcoded 11-item pool, no tier distinction |
| Boss relic pool | `AbstractDungeon.returnRandomRelicEnd` + BossRelicScreen | `run.rs:2370-2401` | **Stub** | Hardcoded 3-relic pool |

---

## Deviations

Severity key: `bug` = diverges from Java with combat-impacting consequences; `deferred` = gap consistent with combat-first scope; `intentional` = deliberate simplification; `unverified` = needs follow-up.

| ID | Area | Deviation | Java ref | Rust ref | Severity |
|---|---|---|---|---|---|
| RN-MAP-01 | Map elite count | A15+ 2.5x elite multiplier missing (only 1.6x for A1+) | `dungeons/AbstractDungeon.java:~550-570 generateRoomTypes` | `map.rs:357-361` | bug |
| RN-MAP-02 | Path generation | `_createPaths` common-ancestor gap check (`min_ancestor_gap=3, max_ancestor_gap=5`) omitted — paths can merge too eagerly | `map/MapGenerator.java:133-211` | `map.rs:215-289` | deferred |
| RN-MAP-03 | Room ratios | Exordium ratios (shop 0.05 / rest 0.12 / event 0.22 / elite 0.08) hardcoded for all acts instead of per-act (TheCity/TheBeyond use same values in Java but not via shared constants — fragile if values change) | `dungeons/Exordium.java:~20-30` / `TheCity.java` / `TheBeyond.java` | `map.rs:350-353` | unverified |
| RN-MAP-04 | Treasure row | Java generates `count -= n.y == map.size()-2 nodes`; Rust only subtracts row 0/8/14 which matches for 15 rows | `AbstractDungeon.java:~500-530 generateMap` | `map.rs:338-347` | intentional |
| RN-MAP-05 | Floor 0 monsters | Java: row 0 always monster, Rust matches | `RoomTypeAssigner.java:~45` | `map.rs:317-321` | clean |
| RN-EV-01 | Event RNG | Used events not removed from pool — same event can repeat across `?` rooms in a run | `AbstractDungeon.java:1936-1977 generateEvent/eventList.remove` | `run.rs:2915-2924` | bug |
| RN-EV-02 | Shrine integration | `shrineChance` (0.25) + `shrineList`/`specialOneTimeEventList` never consulted; 17 shrine events defined in `events/shrines.rs` are unreachable at runtime | `AbstractDungeon.java:1855-1929 shrineChance branch` | `run.rs:2915-2924` + `events/mod.rs:444-446` (typed_shrine_events defined but unused here) | bug |
| RN-EV-03 | Event filters | Missing conditional filters: Dead Adventurer (floor>6), Mushrooms (floor>6), Moai Head (HP<50%), Cleric (gold>=35), Beggar (gold>=75), Colosseum (y>height/2), Big Fish (act 1 only), Note For Yourself (once per save) | `AbstractDungeon.java:1854-1980 canSpawn checks in updateEventList` | `run.rs:2915-2924` (no filter step) | bug |
| RN-EV-04 | One-time events | `Accursed Blacksmith`, `Bonfire Elementals`, `Designer`, `Duplicator`, `FaceTrader`, `Fountain of Cleansing`, `Knowing Skull`, `Lab`, `N'loth`, `SecretPortal`, `The Joust`, `WeMeetAgain`, `The Woman in Blue` are never removed on use (corollary of RN-EV-01 + RN-EV-02) | `AbstractDungeon.java:1331-1347` | `events/shrines.rs` | bug |
| RN-EV-05 | Golden Idol reward | `EventEffect::GoldenIdolTake` gives +300 gold + 25% HP loss; Java gives the **Golden Idol relic**, then Forgotten Altar path differs | `events/exordium/GoldenIdolEvent.java` | `events/mod.rs:402-405`, `events/city.rs:122-134` (Forgotten Altar gives relic label "Golden Idol" — inconsistent with exordium path) | bug |
| RN-EV-06 | Big Fish heal | If Big Fish "Eat" heals flat rather than 10% max HP (confirm site) | `events/exordium/BigFish.java` | `events/exordium.rs` | unverified |
| RN-EV-07 | Living Wall transform | Missing "Transform a card" option | `events/exordium/LivingWallEvent.java` | `events/exordium.rs` | unverified |
| RN-EV-08 | Fountain of Cleansing | `remove_card(999)` removes arbitrary curses rather than all removable curses gated by "must have at least one curse" | `events/shrines/FountainOfCleansing.java` (canSpawn requires curse in deck) | `events/shrines.rs:135-140` | deferred |
| RN-EV-09 | Match and Keep | Java: full 12-card matching minigame with score-based reward; Rust: simplified rule-explanation + play flow | `events/shrines/MatchAndKeep.java` | `events/shrines.rs:165-172` + MatchAndKeepState in run.rs | deferred |
| RN-EV-10 | The Joust | Java resolves win via RNG with distinct payouts per bet; Rust has `joust_bet(bool)` opcode; confirm RNG parity | `events/city/TheJoust.java` | `events/city.rs:302-316`, run.rs ResolveJoustBet | unverified |
| RN-SHOP-01 | Colorless card | No colorless card slot (Java adds 1 colorless card at 1.2x price) | `shop/ShopScreen.java:243-273 initCards` | `run.rs:2531-2560` | deferred |
| RN-SHOP-02 | Sale card | No "on sale" tag with 50% discount on a random card | `ShopScreen.java:initCards + Merchant banner logic` | `run.rs:2531-2560` | deferred |
| RN-SHOP-03 | Relics slot | 0 relics in shop (Java has 3: common/uncommon/rare) | `ShopScreen.initRelics` | — | deferred |
| RN-SHOP-04 | Potions slot | 0 potions in shop (Java has 3) | `ShopScreen.initPotions` | — | deferred |
| RN-SHOP-05 | Card price range | Rust common 45-80 / uncommon 68-120 / rare 135-200; Java is base × 0.9..1.1 (common 45-55, uncommon 67-82, rare 135-165) — wider than Java, especially uncommon/common overlap | `cards/AbstractCard.java:1915-1934 getPrice` + `ShopScreen.initCards 0.9..1.1` | `run.rs:2546-2551` | bug |
| RN-SHOP-06 | Shop purge | 75+25/purge base, Smiling Mask flat 50, Courier 0.8x, Membership 0.5x | `ShopScreen.purgeCard:275-289` | `run.rs:compute_shop_remove_price` | clean |
| RN-SHOP-07 | Membership discount | Applied to cards (50%) — matches Java | `RelicLibrary Membership Card` | `run.rs:2554-2558` | clean |
| RN-SHOP-08 | Meal Ticket heal | +15 on shop enter, gated by Mark of Bloom, scaled by Magic Flower 1.5x | `MealTicket.atShopEnter` | `run.rs:2572-2581` | clean |
| RN-CAMP-01 | Rest heal | 30% max HP + Regal Pillow +15 + Magic Flower 1.5x + Mark of Bloom block | `RestOption.useOption + Regal Pillow` | `run.rs:2488-2500` | clean |
| RN-CAMP-02 | Smith gating | Coffee Dripper / Fusion Hammer / Mark of Bloom gates; no upgraded cards selectable (only `ends_with('+')` check) | `SmithOption.useOption` | `run.rs:2502-2509` | clean (simplified deck iter) |
| RN-CAMP-03 | Toke option | Peace Pipe's "Remove a card" missing | `ui/campfire/TokeOption.java` | — | deferred |
| RN-CAMP-04 | Lift option | Girya counter-based strength gain missing | `ui/campfire/LiftOption.java` | — | deferred |
| RN-CAMP-05 | Dig option | Shovel's "random relic" missing | `ui/campfire/DigOption.java` | — | deferred |
| RN-CAMP-06 | Recall option | Ruby Key teleport-to-boss missing | `ui/campfire/RecallOption.java` | — | deferred |
| RN-NEOW-01 | Reward categories | Only 4 fixed options (100g / 3 cards / upgrade / random relic), no drawback pairings (category 2), no boss-relic swap (category 3), no HP bonus, no potions, no remove/transform card, no Neow's Lament | `neow/NeowReward.java:1-435 getRewardOptions` | `run.rs:586-612 build_neow_options` | bug |
| RN-NEOW-02 | Shuffle order | Fisher-Yates over the 4-option list matches Java rand shuffle semantics | `NeowEvent.pick rand selection` | `run.rs:606-609` | clean |
| RN-TREAS-01 | Chest tier distribution | Java: Small/Medium/Large chests with distinct relic-tier tables (75/25/0, 35/50/15, 0/75/25); Rust gives fixed gold(50-80) + 1 relic | `rewards/chests/AbstractChest.randomizeReward` + SmallChest/MediumChest/LargeChest | `run.rs:1813-1869` | deferred |
| RN-TREAS-02 | Gold roll | Java rolls chest gold independently (50% chance, 25/50/75 tiers); Rust always gives 50-80 | `AbstractChest.randomizeReward` | `run.rs:1814` | deferred |
| RN-TREAS-03 | Chest key gating | Sapphire Key skip-relic + mini-event missing | `TreasureRoom.java` | `run.rs:1813-1869` | deferred |
| RN-TREAS-04 | Matryoshka | +1 extra relic while counter>0 | `Matryoshka relic` | `run.rs:1815-1820, 1847-1859` | clean |
| RN-REWARD-01 | Relic pool | Hardcoded 11-item pool for normal relic rewards (Vajra/Anchor/BagOfMarbles/etc.), no common/uncommon/rare split | `AbstractDungeon.returnRandomRelic(Tier)` | `run.rs:2314-2340` | deferred |
| RN-REWARD-02 | Boss relic pool | Hardcoded `["Philosopher's Stone", "Velvet Choker", "Snecko Eye"]` for all acts — no act-specific boss relic table | `AbstractDungeon.bossRelicPool + return RandomRelicEnd` | `run.rs:2370-2401` | bug |
| RN-REWARD-03 | Card reward upgrade chance | Exordium 0%, TheCity 25% (12.5% A12+), TheBeyond 50% (25% A12+) missing — card rewards are never upgraded | `Exordium/TheCity/TheBeyond.cardUpgradedChance` + `AbstractDungeon.getCardWithoutRng post-upgrade` | `run.rs:1659-1682` | deferred |
| RN-REWARD-04 | Card pool scope | Uses fixed `WATCHER_COMMON_CARDS` / `WATCHER_UNCOMMON_CARDS` / `WATCHER_RARE_CARDS` lists — no class-specific pools for Ironclad/Silent/Defect | per-class CardPool methods | `run.rs:1663-1674` | intentional (scope: Watcher only) |
| RN-REWARD-05 | Elite +25/+35 gold | Elites drop extra gold on top of normal reward | `MonsterRoom elite branch` | `run.rs:build_combat_reward_screen elite branch` | clean |
| RN-REWARD-06 | Rarity weights | 60/33/7 common/uncommon/rare matches Java defaults | `AbstractDungeon.rollRarity` | `run.rs:1663-1674` | clean |
| RN-REWARD-07 | Potion drop chance | `should_offer_potion_reward` must match Java's ~40% base + White Beast Statue 100% + Sozu 0% etc. | `AbstractDungeon.returnRandomPotion + getPotionedEnemies` | `run.rs:2403` (not expanded here) | unverified |
| RN-FLOW-01 | RunPhase state machine | Rust explicit enum; Java uses CurrentScreen + roomType dispatch — functionally equivalent | `AbstractDungeon.screen + currRoom` | `run.rs RunPhase` | clean |
| RN-FLOW-02 | Boss entry | Rust enters boss by `map_y >= height-1` after campfire step; Java triggers via boss node activation | `MapRoomNode boss` | `run.rs:2515-2520` | clean |

---

## Items verified clean

- Map dimensions (15 rows × 7 cols, path_density=6).
- Row 0 always monster, row 8 treasure, row 14 rest; matches `RoomTypeAssigner` fixed rows.
- Room-placement rules: elites forbidden y≤4, rests forbidden y≤4 or y≥13; parent-same-type block; sibling-same-type block.
- Shop purge: 75g base + 25g per purge, Smiling Mask flat 50, Courier 0.8x, Membership 0.5x.
- Campfire Rest 30% max HP, Regal Pillow +15 bonus, Magic Flower 1.5x, Mark of Bloom zeroes heal.
- Campfire Smith gated by Coffee Dripper / Fusion Hammer / Mark of Bloom.
- Meal Ticket +15 on shop enter with same relic stack.
- Matryoshka extra relic at treasure, counter decrement.
- Deterministic map generation across same-seed calls.
- 60/33/7 common/uncommon/rare rarity split on card rewards.
- Elite combat reward path goes through `build_combat_reward_screen` with Black Star doubling relic count.
- Fisher-Yates shuffle semantics on the Neow option list (matches Java `Random.nextInt(i+1)` swap).

---

## Follow-up questions

1. **RN-EV-06 / RN-EV-07**: Confirm exact effect tables for Big Fish ("Eat" 10% max HP vs flat) and Living Wall (does Rust expose the transform branch?). Worth spot-reading `events/exordium.rs` in detail.
2. **RN-MAP-03**: Are the hardcoded Exordium ratios in `map.rs:350-353` actually used for act 2/3 in `generate_map`? Need to check whether the caller passes act context.
3. **RN-REWARD-07**: Does `should_offer_potion_reward` implement the Sozu/White Beast Statue stack and 40/50/60% drop progression after no-potion streaks?
4. **RN-EV-10**: Joust RNG — Java uses a specific `combat.random(100) < threshold` check; worth verifying `ResolveJoustBet` matches probabilities.
5. **RN-NEOW-01 scope**: Is the simplified Neow a deliberate combat-first stub (should stay `deferred`), or blocker for strategic training (promote to `bug`)? My ranking treats it as `bug` because Neow affects floor-0 deck state that feeds every combat rollout.
6. **RN-SHOP-05**: Confirm the Watcher card price floor — is the wider RNG a conscious exploration-variance choice or a port mistake?
