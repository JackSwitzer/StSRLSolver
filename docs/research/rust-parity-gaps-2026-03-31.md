# Rust Engine: Complete Parity Gap Analysis (2026-03-31)

## Summary
6 parallel audits (5 Claude + 1 Codex GPT-5.4) against Java source. The Rust engine (28,607 LOC, 912 tests) has strong combat mechanics but significant gaps in events, card effects, run simulation, and ascension scaling.

## GAP 1: EVENTS (41 of 56 missing)

### Implemented (15):
- Exordium: Big Fish, Golden Idol, Scrap Ooze, Shining Light, Living Wall
- City: Forgotten Altar, Council of Ghosts, Masked Bandits, Knowing Skull, Vampires
- Beyond: Mysterious Sphere, Mind Bloom, Tomb of Lord Red Mask, Sensory Stone, Secret Portal

### Missing (41):
**Exordium (6):** Cleric, Dead Adventurer, Golden Wing, Goop Puddle, Mushrooms, Sssserpent
**City (10):** Addict, Back to Basics, Beggar, Colosseum, Cursed Tome, Drug Dealer, Nest, The Joust, The Library, The Mausoleum
**Beyond (4):** Falling, Moai Head, Spire Heart, Winding Halls
**Shrines (17):** Accursed Blacksmith, Bonfire Spirits, Designer, Duplicator, Face Trader, Fountain of Curse Removal, Gold Shrine, Gremlin Match Game, Gremlin Wheel Game, Lab, N'loth, Note For Yourself, Purification Shrine, Transmogrifier, Upgrade Shrine, We Meet Again, Woman in Blue

**Estimated LOC:** ~2,000 (avg 50 LOC per event)

## GAP 2: CARD EFFECTS (208 of 373 effect tags unhandled)

### Critical unhandled categories:
- **Card generation** (35 tags): add_shivs, add_wounds_to_hand, add_random_attacks, copy_on_draw, etc.
- **Discard/exhaust mechanics** (28 tags): discard_gain_energy, exhaust_choose, exhaust_random
- **Status application** (21 tags): apply_vulnerable, apply_weak, poison_all, choke
- **Energy/cost mechanics** (18 tags): cost_reduce_on_discard, energy_on_kill, enlightenment
- **Block mechanics** (15 tags): barricade, double_block, flame_barrier
- **Retain system** (28 cards affected): retain_hand, retain_block

### Card stats: All verified correct (30/30 sampled match Java)

**Estimated LOC:** ~1,500 for top 50 most-used tags

## GAP 3: RUN SIMULATION (6 critical missing mechanics)

| Mechanic | Severity | LOC |
|----------|----------|-----|
| **Neow choices** (first room) | CRITICAL | 150-200 |
| **Boss relic rewards** (choose 1 of 3 after boss) | CRITICAL | 200-300 |
| **Act transitions** (beat boss → next act) | CRITICAL | 150-200 |
| **Treasure room relics** (relic reward, not just gold) | HIGH | 80-120 |
| **Potion rewards** (after combat) | HIGH | 100-150 |
| **Key mechanics** (Ruby/Emerald/Sapphire for Act 4) | HIGH | 200-250 |
| **Character selection** (4 characters, different pools) | MODERATE | 400-600 |

**Total:** ~1,300-1,800 LOC

## GAP 4: ASCENSION SCALING (mostly missing)

| Level | Status | Impact |
|-------|--------|--------|
| A1 | Partial | Harder monsters |
| A2 | **Missing** | +1 elite per act |
| A3-4 | **Missing** | Normal enemy damage/HP |
| A5 | **Missing** | Campfire heal 25% not 30% |
| A6 | **Missing** | Floor 1 harder monsters |
| A7 | Implemented | Boss HP scaling |
| A8 | **Missing** | Elite HP +25% |
| A9 | **Missing** | Monster HP +10% |
| A10 | Implemented | Ascender's Bane |
| A11 | Implemented | Potion potency -50% |
| A12-13 | **Missing** | Boss damage, harder elites |
| A14 | Implemented | Starter max HP 68 |
| A15-16 | **Missing** | Gold loss, harder strikes |
| A17 | **Missing** | Elite patterns harder |
| A18 | **Missing** | Elite HP further boost |
| A19 | **Partial** | Some boss patterns |
| A20 | **Missing** | Double boss effect |

**Estimated LOC:** ~300-500

## GAP 5: POWERS (20 with stubs, 3 missing entirely)

- **Missing:** Echo Form, Malleable, Flight (enemy powers)
- **Stub triggers (20):** EnergyDown, MasterReality, FreeAttackPower, Establishment, Vigor tracking, Equilibrium, Loop, HelloWorld, CreativeAI, Electro, Heatsink, Storm, LockOn, Focus effect, Omniscience, Vault, Mark damage, Panache, NoSkillsPower, CannotChangeStance

**Estimated LOC:** ~400

## GAP 6: RELICS (4 complete stubs, 4 partial)

- **Complete stubs:** Torii (combat trigger missing), Dead Branch (no card gen on exhaust), Necronomicon (no auto-play), Tungsten Rod (referenced but missing)
- **Partial:** Ice Cream, Runic Pyramid, Chemical X, Frozen Eye (counter initialized, no effect)

Wait — Torii and Tungsten Rod were added by the relics agent in PR #106. And Chemical X was wired in card_effects.rs. These audit findings may be from reading an older branch.

**Estimated LOC:** ~200 for remaining stubs

## GAP 7: COMBAT STATE (missing fields)

- **CRITICAL:** SelectScryDiscard action type missing (no way to make scry decisions)
- **CRITICAL:** pending_scry_cards, pending_scry_selection not tracked
- **HIGH:** Buffer relic missing from incoming damage
- **HIGH:** half_dead field for Darkling revive
- **MEDIUM:** relic_counters, combat_type, skills/powers_played_this_turn

**Estimated LOC:** ~300

## GAP 8: DAMAGE PIPELINE

- Buffer relic not in calculate_incoming_damage()
- Basic calculate_damage() missing some modifier variants (use calculate_damage_full instead)
- Otherwise correct order and rounding

**Estimated LOC:** ~50

---

## PRIORITY ORDER FOR FULL PARITY

### Phase 1: Run Simulation (enables multi-act training)
- Act transitions + boss relic rewards + Neow
- ~500 LOC, unlocks full-game runs

### Phase 2: Events (41 missing)
- All 56 events from Java
- ~2,000 LOC

### Phase 3: Card Effects (208 unhandled tags)
- Top 50 most-used effect tags
- ~1,500 LOC

### Phase 4: Ascension + State + Actions
- Full A1-A20 scaling
- SelectScryDiscard action
- Missing state fields
- ~800 LOC

### Phase 5: Power/Relic stubs
- Wire remaining 20 power triggers
- Fix relic stubs
- ~600 LOC

**TOTAL ESTIMATED REMAINING:** ~5,400 LOC to full Java parity
