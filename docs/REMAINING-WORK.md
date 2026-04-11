# Remaining Work — Master List

## A. BUGS (13 total)

### A1. Data Value Bugs (9 — FIXED ce445cc)
All fixed: Reaper exhaust, Setup top-of-draw, Expunger+ damage, Empty Body+ block, Genetic Algorithm block, 4 card ID mismatches.

### A2. Logic Bugs (4 — OPEN)
| # | Entity | Bug | Fix |
|---|--------|-----|-----|
| 1 | Burning Blood + Black Blood | Both heal on victory | Guard: skip Burning Blood if Black Blood present |
| 2 | Disarm | Empty effect_data, relies on dead old dispatch | Add `AddStatus(SelectedEnemy, LOSE_STRENGTH, Magic)` |
| 3 | Combust | HP loss hardcoded to 1, should scale with stacks | Change `combust_hp_loss: 1` to `combust_hp_loss: amt` in hook |
| 4 | Biased Cognition | Multiple casts don't stack focus loss/turn | Change `SetStatus(Fixed(1))` to `AddStatus(Fixed(1))` for BIASED_COG_FOCUS_LOSS |

### A3. Empty Stub Functions (3 — OPEN)
| # | File | Function | Card Affected |
|---|------|----------|---------------|
| 5 | hooks_simple.rs:123 | hook_energy_on_kill | Sunder — gain 3 energy on kill is a NO-OP |
| 6 | hooks_simple.rs:169 | hook_block_from_damage | Wallop — gain block = unblocked damage is a NO-OP |
| 7 | hooks_simple.rs:248 | hook_reaper | Reaper — heal for unblocked damage is a NO-OP |

**NOTE**: Items 5-7 may be superseded by the complex_hooks added in commit 9e88433. Need to verify whether the old hooks_simple stubs are still called or dead code.

## B. CARD MIGRATION (81 cards need effect_data)

### B1. Completely Non-Functional Stubs (8 cards — HIGHEST PRIORITY)
Cards with NO implementation at all (no effects, no effect_data, no complex_hook):
| Card | Class | What It Should Do |
|------|-------|-------------------|
| Collect | Watcher | X-cost, create X Miracles next turn |
| Deus Ex Machina | Watcher | Unplayable, add 2/3 Miracles on draw |
| Simmering Fury (Vengeance) | Watcher | Enter Wrath + draw 2/3 next turn |
| Foresight (Wireheading) | Watcher | Power, scry 3/4 at start of turn |
| Flurry of Blows | Watcher | Return from discard on stance change |
| Unraveling | Watcher | Play all hand cards for free |
| Discipline | Watcher | Deprecated — remove or stub |
| Omniscience | Watcher | Choose card from draw, play it twice |

### B2. String-Tag Only Cards (73 cards — MEDIUM PRIORITY)
These WORK via old string-based dispatch but need declarative effect_data:

**Ironclad (12):** Body Slam, Burning Pact, Disarm, Dual Wield, Fiend Fire, Havoc, Heavy Blade, Perfected Strike, Searing Blow, Second Wind, Sentinel, True Grit (base)

**Silent (14):** Acrobatics, Alchemize, All-Out Attack, Bouncing Flask, Calculated Gamble, Dagger Throw, Distraction, Expertise, Finisher, Flechettes, Nightmare, Piercing Wail, Prepared, Storm of Steel, Survivor

**Defect (21):** Auto-Shields, Blizzard, Buffer, Capacitor, Chaos, Force Field, FTL, Heatsinks, Hello World, Impulse, Loop, Machine Learning, Melter, Rebound, Recursion, Reinforced Body, Scrape, Self Repair, Static Discharge, Storm, Thunder Strike

**Watcher (12):** Battle Hymn, Brilliance, Conclude, Deva Form, Devotion, Establishment, Fasting, Like Water, Master Reality, Nirvana, Omega, Study, Weave + 3 more

**Colorless (16):** Apotheosis, Chrysalis, Discovery, Enlightenment, Forethought, Impatience, Jack of All Trades, Madness, Magnetism, Mayhem, Metamorphosis, Mind Blast, Panache, Purity, Ritual Dagger, Sadistic Nature, Secret Technique, Violence

## C. TEST GAPS (need new tests)

### C1. Weak Assertions (37 — convert to exact ==)
Tests that use `<`, `>=`, `>` instead of exact `==`. Full list in AUDIT-2026-04-11.md.
Files: test_cards_defect.rs (6), test_cards_watcher.rs (5), test_integration.rs (26)

### C2. Critical Untested Entities (top 20)
1. Corruption, 2. Barricade, 3. Echo Form replay, 4. Double Tap replay, 5. Demon Form, 6. Noxious Fumes, 7. After Image, 8. Burst replay, 9. Blade Dance + Accuracy, 10. Whirlwind, 11. Offering, 12. Feel No Pain, 13. Dark Embrace, 14. Inflame, 15. Heavy Blade STR scaling, 16. Disarm, 17. Apotheosis, 18. Fiend Fire, 19. A Thousand Cuts, 20. Well-Laid Plans

### C3. Missing Interaction Tests
- Strength + multi-hit attacks
- Vulnerable + multi-hit attacks
- Corruption + skills costing 0 + exhaust
- Barricade + block retention across turns
- Echo Form + card replay behavior
- Double Tap + attack replay
- Blade Dance + Accuracy (Shiv damage bonus)

### C4. Untested Colorless Cards (40 cards with zero tests)

### C5. Untested Upgrade Behavior Changes (11)
Limit Break+, Calculated Gamble+, Hologram+, Rainbow+, Phantasmal Killer+, Infinite Blades+, Deva Form+, Battle Hymn+, Worship+, Armaments+ (upgrade all), Impulse+

## D. INFRASTRUCTURE / CLEANUP

### D1. Wire Unified Dispatch
- EntityDef registries exist (37 relics, 37 powers, 26 potions) but are NOT wired into engine.rs
- Need `dispatch_trigger()` calls at each trigger point in engine.rs
- Then delete old match blocks in relics/combat.rs, powers/registry.rs, potions/mod.rs

### D2. Dead Code
- 3 dead cards: Impulse, Discipline, Unraveling (removed from Java)
- Legacy getters in powers/buffs.rs (get_infinite_blades, apply_demon_form, etc.)
- 3 empty stub hooks in hooks_simple.rs (may be dead)
- 10 complex power defs with hook_noop placeholders

### D3. Silent Catch-All Match Arms (6 risky)
- engine.rs:500, engine.rs:1700, run.rs:1075, run.rs:1121, run.rs:1228, combat_hooks.rs:521
- These `_ => {}` arms silently swallow unmatched cases

## E. SUMMARY COUNTS

| Category | Count | Priority |
|----------|-------|----------|
| Logic bugs | 4 | P0 |
| Non-functional card stubs | 8 | P0 |
| String-tag cards needing migration | 73 | P1 |
| Weak test assertions | 37 | P1 |
| Critical untested entities | 20 | P1 |
| Missing interaction tests | 7 | P1 |
| Untested colorless cards | 40 | P2 |
| Untested upgrade behaviors | 11 | P2 |
| Wire unified dispatch | 1 big task | P2 |
| Dead code cleanup | ~20 items | P3 |
