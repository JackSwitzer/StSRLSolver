# Test Coverage Audit: Slay the Spire RL Engine

**Date:** 2026-03-03
**Auditor:** Claude Opus 4.6 (1M context)
**Total Tests:** 5728 collected (4914 `def test_` across 88 files)
**Skipped:** ~1 explicit skip marker (test_enemy_ai_parity.py)
**Test Framework:** pytest, fixtures in `tests/conftest.py`

---

## 1. Current Test File Inventory

### Core Engine Tests (57 files, ~3700 tests)

| File | Tests | Domain | Coverage Quality |
|------|-------|--------|-----------------|
| `test_effects_and_combat.py` | 187 | Effect registry, card effects, combat handler | BEHAVIORAL - tests actual card play pipelines |
| `test_coverage_boost.py` | 157 | Mixed coverage fill-in | BEHAVIORAL + DATA |
| `test_damage.py` | 133 | Damage calculation formulas | BEHAVIORAL - rounding, multipliers, edge cases |
| `test_silent_card_verification.py` | 130 | Silent card data fields | DATA - verifies card stats match Java |
| `test_defect_card_verification.py` | 125 | Defect card data fields | DATA - verifies card stats match Java |
| `test_enemy_ai_parity.py` | 118 | Enemy AI patterns vs Java | BEHAVIORAL - move sequences, ascension thresholds |
| `test_powers.py` | 118 | Power data + basic behavior | MIXED - 60% data, 40% behavioral |
| `test_combat.py` | 117 | Combat flow, turn order, death checks | BEHAVIORAL - full combat sequences |
| `test_cards.py` | 115 | Card data fields, cost, damage, block | DATA - verifies card definitions |
| `test_events.py` | 111 | Event pools, choices, outcomes | DATA + BEHAVIORAL - event structure + outcome values |
| `test_ironclad_card_verification.py` | 108 | Ironclad card data fields | DATA - verifies card stats match Java |
| `test_status_curse.py` | 107 | Status/curse card behavior | BEHAVIORAL - Burn, Wound, Dazed effects |
| `test_agent_api.py` | 106 | Agent JSON API, action dicts | BEHAVIORAL - RL interface contract |
| `test_rewards.py` | 104 | Card/relic/potion rewards | BEHAVIORAL - reward generation, rarity tiers |
| `test_potions.py` | 101 | Potion data + basic effects | MIXED - data validation + effect tests |
| `test_audit_bosses.py` | 99 | Boss enemy data + AI patterns | MIXED - HP/damage values + move sequences |
| `test_ironclad_cards.py` | 97 | Ironclad card effects | BEHAVIORAL - plays cards, checks outcomes |
| `test_silent_cards.py` | 96 | Silent card effects | BEHAVIORAL - plays cards, checks outcomes |
| `test_audit_events.py` | 95 | Event outcome parity vs Java | DATA + BEHAVIORAL - outcome values, heal amounts |
| `test_relic_registry_integration.py` | 92 | Relic trigger registration + execution | BEHAVIORAL - registry wiring |
| `test_encounter_combat_integration.py` | 90 | Encounter table -> combat pipeline | BEHAVIORAL - encounter creation, enemy instantiation |
| `test_rng.py` | 90 | RNG streams, XorShift128 | BEHAVIORAL - deterministic sequence verification |
| `test_ascension.py` | 85 | Ascension modifiers | DATA + BEHAVIORAL - HP, damage, curse adjustments |
| `test_watcher_card_effects.py` | 83 | Watcher card effects via EffectContext | BEHAVIORAL - stance, mantra, scry, damage |
| `test_defect_cards.py` | 80 | Defect card effects | BEHAVIORAL - orbs, focus, channeling |
| `test_relic_triggers_combat.py` | 77 | Combat relic triggers | BEHAVIORAL - TDD approach, actual relic fire |
| `test_handlers.py` | 75 | Event/combat/rest handlers | BEHAVIORAL - handler dispatch |
| `test_power_handlers_new.py` | 70 | New power handler implementations | BEHAVIORAL - trigger hooks |
| `test_audit_power_offensive.py` | 68 | Offensive powers (Strength, etc.) | BEHAVIORAL - damage modification |
| `test_map.py` | 68 | Map generation, node types | BEHAVIORAL - seed determinism, path rules |
| `test_power_edge_cases.py` | 65 | Power edge cases | BEHAVIORAL - stacking, expiration, interaction |
| `test_audit_relics_damage.py` | 65 | Damage-modifying relics | BEHAVIORAL - damage pipeline interaction |
| `test_behavioral_core.py` | 58 | Core behavioral tests (actual gameplay) | BEHAVIORAL - full game sequences, card play |
| `test_damage_edge_cases.py` | 57 | Damage calculation edge cases | BEHAVIORAL - rounding, negative stats, relics |
| `test_power_registry_integration.py` | 57 | Power registry wiring | BEHAVIORAL - trigger fire, context propagation |
| `test_potion_effects_full.py` | 56 | Full potion effect execution | BEHAVIORAL - potion use in combat |
| `test_audit_stances.py` | 54 | Stance mechanics vs Java | BEHAVIORAL - Wrath/Calm/Divinity/Mantra |
| `test_audit_potions.py` | 53 | Potion data parity | DATA - potion fields, rarity, class |
| `test_audit_power_defensive.py` | 50 | Defensive powers (Block, etc.) | BEHAVIORAL - block modification |
| `test_parity.py` | 50 | Seed-deterministic parity | BEHAVIORAL - encounter/map/reward verification |
| `test_power_integration_lock.py` | 48 | Cross-system power integration | BEHAVIORAL - power+card+relic interactions |
| `test_generation.py` | 47 | Encounter/card/map generation | BEHAVIORAL - generation algorithms |
| `test_audit_gameloop.py` | 45 | Game loop phases, rewards, rest | BEHAVIORAL - phase transitions, gold, bosses |
| `test_rng_parity.py` | 45 | RNG parity with Java | BEHAVIORAL - stream-level parity |
| `test_integration.py` | 43 | Multi-step integration sequences | BEHAVIORAL - full game flow |
| `test_game_runner.py` | 42 | GameRunner orchestrator | BEHAVIORAL - init, phases, Neow, decisions |
| `test_relic_passive.py` | 39 | Passive relic effects | BEHAVIORAL - always-on relics |
| `test_audit_relics_cardplay.py` | 38 | Card-play relics | BEHAVIORAL - onPlayCard triggers |
| `test_audit_cards_skill.py` | 38 | Skill card behavior | BEHAVIORAL - block, draw, stance |
| `test_relic_pickup.py` | 34 | Relic acquisition effects | BEHAVIORAL - onEquip hooks |
| `test_potion_prediction.py` | 33 | Potion RNG prediction | BEHAVIORAL - drop rate, seed parity |
| `test_watcher_powers_behavioral.py` | 27 | Watcher power behavior | BEHAVIORAL - MentalFortress, Rushdown, etc. |
| `test_audit_cards_attack.py` | 27 | Attack card behavior | BEHAVIORAL - damage dealing |
| `test_audit_power_turnbased.py` | 27 | Turn-based powers | BEHAVIORAL - start/end turn triggers |
| `test_ws_server.py` | 27 | WebSocket server | BEHAVIORAL - connection, message handling |
| `test_audit_block.py` | 26 | Block calculation | BEHAVIORAL - frail, dex, floor |
| `test_audit_relics_combat.py` | 26 | Combat relic behavior | BEHAVIORAL - combat-time triggers |
| `test_relic_events.py` | 25 | Event-related relics | BEHAVIORAL - relic effects at events |
| `test_potion_registry.py` | 25 | Potion registry wiring | BEHAVIORAL - registration, lookup |
| `test_rng_migration_determinism.py` | 20 | RNG after migration | BEHAVIORAL - determinism preserved |
| `test_rng_audit.py` | 20 | RNG stream audit | BEHAVIORAL - stream isolation |
| `test_relic_bottled.py` | 20 | Bottled relic effects | BEHAVIORAL - innate hand placement |
| `test_relic_card_rewards.py` | 17 | Relic card reward modifiers | BEHAVIORAL - Busted Crown, etc. |
| `test_potion_sacred_bark.py` | 17 | Sacred Bark potion doubling | BEHAVIORAL - potency modification |
| `test_relic_triggers_outofcombat.py` | 17 | Out-of-combat relic triggers | BEHAVIORAL - shop/event hooks |
| `test_enemies.py` | 16 | Enemy data definitions | DATA - HP ranges, move values |
| `test_agent_readiness.py` | 14 | Agent readiness checks | BEHAVIORAL - RL component validation |
| `test_relic_rest_site.py` | 13 | Rest site relic effects | BEHAVIORAL - Dream Catcher, Regal Pillow |
| `test_relic_acquisition.py` | 9 | Relic acquisition pipeline | BEHAVIORAL - equip hooks |
| `test_relic_implementation.py` | 8 | Relic implementations | BEHAVIORAL - specific relic effects |
| `test_orb_runtime_orb001.py` | 5 | Orb system runtime | BEHAVIORAL - channel, evoke, passive |
| `test_audit_inventory_manifest.py` | 5 | Inventory completeness | DATA - all cards/relics/potions exist |
| `test_relic_ordering_rel007.py` | 5 | Relic ordering | BEHAVIORAL - trigger order |
| `test_relic_eggs.py` | 5 | Egg relic effects | BEHAVIORAL - upgrade on pickup |
| `test_potion_rng_streams.py` | 4 | Potion RNG stream usage | BEHAVIORAL - correct stream |
| `test_relic_aliases.py` | 4 | Relic ID aliases | DATA - name mapping |
| `test_card_id_aliases_audit.py` | 2 | Card ID aliases | DATA - name mapping |
| `test_potion_runtime_dedup.py` | 2 | Potion deduplication | BEHAVIORAL - no double-fire |
| `test_combat_runner_compat.py` | 2 | Combat runner compatibility | BEHAVIORAL - API compat |
| `test_audit_power_dispatch.py` | 1 | Power dispatch | BEHAVIORAL - dispatch correctness |
| `test_audit_power_manifest.py` | 2 | Power manifest | DATA - all powers listed |

### Training/RL Tests (5 files, ~120 tests)

| File | Tests | Domain | Coverage Quality |
|------|-------|--------|-----------------|
| `training/test_train.py` | 37 | PPO trainer, softmax, GAE | BEHAVIORAL - training loop mechanics |
| `training/test_gym_env.py` | 23 | Gymnasium env wrapper | BEHAVIORAL - reset, step, action masks |
| `training/test_planner.py` | 22 | Strategic planner + MCTS | BEHAVIORAL - search, plan quality |
| `training/test_conquerer.py` | 21 | Seed conquerer beam search | BEHAVIORAL - multi-path search |
| `training/test_kill_calculator.py` | 17 | Kill probability calculator | BEHAVIORAL - kill detection |

---

## 2. Coverage Gaps by Area

### 2.1 Combat Flow (CRITICAL)

**Audit findings from `combat-flow-parity.md`:**

The combat flow audit identified **9 CRITICAL/HIGH** parity issues. **Zero** of these have dedicated regression tests:

| ID | Issue | Has Test? | Priority |
|----|-------|-----------|----------|
| **E1** | Player `atEndOfTurn` fires AFTER discard in Python (should be BEFORE) | NO | CRITICAL |
| **C1** | `atBattleStart`/`atBattleStartPreDraw` order reversed | NO | CRITICAL |
| **T1** | Block decay AFTER start-of-turn in Java, BEFORE in Python | NO | CRITICAL |
| **E3** | `triggerOnEndOfTurnForPlayingCard` missing (Burn, Regret, Decay) | NO | HIGH |
| **T6** | Blur handled as counter not power (full block retain) | NO | HIGH |
| **P1** | `onPlayCard` fires after card removed from hand | NO | HIGH |
| **P5** | `onUseCard` vs card destination ordering reversed | NO | HIGH |
| **P6** | Card removed from hand before effects | NO | HIGH |
| **D9** | Enemy damage hooks fire-and-forget (don't modify damage) | NO | HIGH |
| **D8** | Torii/Intangible ordering wrong for enemy attacks | NO | HIGH |

**Existing coverage:** `test_combat.py` (117 tests) covers basic turn flow, card play, death checks. `test_behavioral_core.py` (58 tests) covers gameplay sequences. But neither tests the **specific ordering differences** identified in the audit.

### 2.2 Card Effects (HIGH)

**Audit findings from `card-parity-report.md`:**

| Card | Issue | Has Test? | Priority |
|------|-------|-----------|----------|
| Fasting | Missing EnergyDown penalty | NO | CRITICAL |
| Conjure Blade | Upgraded should give X+1 hits | NO | CRITICAL |
| Spirit Shield | Counts itself in hand | NO | HIGH |
| Wreath of Flame | Uses "WreathOfFlame" status not Vigor | NO | HIGH |
| Simmering Fury | One combined status vs two separate | NO | HIGH |
| WindmillStrike | Upgrade gives +4 not +5 per retain | NO | HIGH |
| Bowling Bash | Multi-hit per living enemy unverified | NO | HIGH |

**Existing coverage:** `test_watcher_card_effects.py` (83 tests) covers many card effects but not these specific bugs. `test_cards.py` (115 tests) is mostly DATA (verifying card stat fields), not behavioral.

### 2.3 Enemy AI (HIGH)

**Audit findings from `monsters-parity-report.md`:**

| Monster | Issue | Has Test? | Priority |
|---------|-------|-----------|----------|
| AcidSlime_S | A17 pattern uses wrong mechanism | NO | CRITICAL |
| GremlinNob | Enrage vs Anger power naming | PARTIAL (naming checked, not behavior) | CRITICAL |
| Looter/Mugger | Simplified to always MUG (no flee) | NO | HIGH |
| Hexaghost | Burn upgrade after Inferno missing | NO | MEDIUM |
| BronzeOrb | First-move Stasis should be guaranteed | NO | MEDIUM |
| WrithingMass | Reactive re-roll on damage | NO | MEDIUM |
| Louse (unified) | Green Louse HP wrong in unified class | NO | HIGH |

**Existing coverage:** `test_enemy_ai_parity.py` (118 tests) covers major enemies but misses the above specific issues. `test_encounter_combat_integration.py` (90 tests) verifies encounter creation but not AI logic.

### 2.4 Events (HIGH)

**Audit findings from `events-parity-report.md`:**

7 CRITICAL + 8 HIGH event parity issues identified. Most have incorrect mechanics, not just wrong values.

| Event | Issue | Has Test? | Priority |
|-------|-------|-----------|----------|
| Back to Basics | "Simplicity" removes ALL cards (should remove 1) | NO | CRITICAL |
| The Beggar | Completely wrong mechanics | NO | CRITICAL |
| Forgotten Altar | Option 1 wrong (max HP gain vs relic) | NO | CRITICAL |
| The Mausoleum | 50/50 relic/curse (should always give relic) | NO | CRITICAL |
| The Nest | Both options wrong | NO | CRITICAL |
| Sensory Stone | Wrong structure (3 choices vs 1) | NO | CRITICAL |
| Designer | Random option types vs fixed | NO | CRITICAL |
| The Joust | Missing 50g bet cost, wrong murderer reward | NO | HIGH |
| The Library | Sleep heals full (should heal 33%/20%) | NO | HIGH |
| Face Trader | Option 0 gives relic (should give gold) | NO | HIGH |

**Existing coverage:** `test_events.py` (111 tests) and `test_audit_events.py` (95 tests) together cover event pool membership, choice structure, and basic outcome values. But they test the **data model** (OutcomeType, amounts), not whether the **handler** produces correct results.

### 2.5 Powers (MEDIUM)

**Audit findings from `system-audit-summary.md`:**

| Gap | Count | Has Test? | Priority |
|-----|-------|-----------|----------|
| Missing power implementations | 38 | NO (7 Watcher-specific) | HIGH |
| Double-trigger regression (fixed bugs) | 4 powers | PARTIAL (test_watcher_powers_behavioral covers some) | HIGH |
| Power hook ordering (onPlayCard, onUseCard, onAfterUseCard) | - | NO | HIGH |
| Missing atDamageFinalGive/Receive hooks | - | NO | MEDIUM |

**Existing coverage:** `test_powers.py` (118), `test_power_handlers_new.py` (70), `test_power_edge_cases.py` (65), `test_power_integration_lock.py` (48), `test_power_registry_integration.py` (57), `test_watcher_powers_behavioral.py` (27). Coverage is decent for implemented powers but does not test **hook ordering** or **missing hooks**.

### 2.6 Relics (MEDIUM)

**Audit findings from `system-audit-summary.md`:**

| Gap | Count | Has Test? | Priority |
|-----|-------|-----------|----------|
| Missing relic implementations | 45 | Partial (skip markers) | MEDIUM |
| Rest site relics | 36 skipped | YES (test_relic_rest_site, 13 tests, most skipped) | HIGH |
| Pickup effects | 34 skipped | YES (test_relic_pickup, 34 tests, most skipped) | HIGH |
| Chest acquisition | 30 skipped | Partial | HIGH |
| Bottled relics | 20 skipped | YES (test_relic_bottled, 20 tests) | MEDIUM |

**Existing coverage:** ~14 relic test files (~350 tests total). Structure is good but many tests are skipped pending implementation.

### 2.7 RL/Training Pipeline (LOW for parity, HIGH for training)

| Gap | Has Test? | Priority |
|-----|-----------|----------|
| Observation encoder correctness | Minimal (test_behavioral_core has a few) | MEDIUM |
| Action mask completeness (all valid actions masked) | YES (test_gym_env, basic) | MEDIUM |
| Full game determinism (same seed = same trajectory) | YES (test_parity, test_rng_parity) | OK |
| Self-play training loop correctness | Minimal (test_train) | LOW |
| MCTS correctness under copy() | Minimal (test_planner) | LOW |

---

## 3. Test Recommendations

### CRITICAL Priority (Blocks correct training data)

#### CR-1: Combat Flow Ordering Tests
**Effort:** 2-3 hours
**Tests needed:** 6-8 tests

```python
class TestCombatFlowOrdering:
    """Regression tests for combat-flow-parity.md issues."""

    def test_block_decay_happens_after_start_of_turn_triggers(self):
        """T1: Block should persist through start-of-turn power triggers.

        Scenario: Player has 10 block, Poison 3. Start of turn:
        Java: Poison ticks against block (takes 0 HP damage, block reduced to 7)
        Python (bug): Block already decayed, Poison deals 3 HP damage
        """
        # Setup: player with 10 block, Poison 3
        # Action: end turn + start next turn
        # Assert: player took 0 HP damage from poison (blocked)
        pass

    def test_end_of_turn_powers_fire_before_discard(self):
        """E1: Player atEndOfTurn powers should fire while hand still exists.

        Scenario: Player has Constricted power (deals damage at end of turn).
        Java: Constricted fires while cards still in hand.
        Python (bug): Hand discarded first, then Constricted fires.
        """
        pass

    def test_battle_start_pre_draw_fires_before_battle_start(self):
        """C1: atBattleStartPreDraw should fire before atBattleStart.

        Scenario: PureWater relic (atBattleStartPreDraw adds Miracle).
        Java: Miracle added to draw pile BEFORE atBattleStart relics trigger.
        """
        pass

    def test_end_of_turn_cards_auto_play(self):
        """E3: Burn/Regret/Decay should auto-play at end of turn.

        Scenario: Player has Burn in hand.
        Java: Burn deals 2 damage to player at end of turn.
        Python (bug): No end-of-turn card auto-play.
        """
        pass

    def test_card_not_removed_from_hand_before_effects(self):
        """P6: Card should be in 'limbo' during execution, not removed.

        Scenario: Spirit Shield counts cards in hand.
        If card is removed before effects, hand count is wrong.
        """
        pass

    def test_blur_retains_all_block_like_barricade(self):
        """T6: Blur should retain ALL block for the active turn.

        Java: Blur is a power that prevents block decay entirely.
        Python (bug): Treats Blur as a counter that partially retains.
        """
        pass
```

#### CR-2: Card Effect Regression Tests
**Effort:** 1-2 hours
**Tests needed:** 7 tests

```python
class TestCardEffectParity:
    """Regression tests for card-parity-report.md findings."""

    def test_fasting_applies_energy_down(self):
        """Fasting must apply EnergyDown power (-1 energy per turn).

        Without this, Fasting gives free +3/+4 Str/Dex with no downside.
        """
        # Play Fasting card
        # Assert: player has EnergyDown power
        # End turn, start new turn
        # Assert: energy is max_energy - 1
        pass

    def test_conjure_blade_upgraded_gives_x_plus_1(self):
        """Upgraded Conjure Blade should give X+1 hits on Expunger.

        Java: ConjureBlade.java:27-30 passes energyOnUse + 1 when upgraded.
        """
        pass

    def test_spirit_shield_excludes_self_from_hand_count(self):
        """Spirit Shield should not count itself when calculating block.

        Java: SpiritShield.java:34-37 explicitly skips `this` card.
        Expected block = magicNumber * (hand_size - 1)
        """
        pass

    def test_wreath_of_flame_uses_vigor_not_custom_status(self):
        """Wreath of Flame should apply standard Vigor power.

        Java: WreathOfFlame.java:34 uses VigorPower.
        This ensures correct stacking with other Vigor sources.
        """
        pass

    def test_simmering_fury_applies_two_separate_powers(self):
        """Simmering Fury should apply WrathNextTurn + DrawCardNextTurn.

        Java: SimmeringFury.java:27-28 applies two distinct powers.
        """
        pass

    def test_windmill_strike_upgrade_gains_5_per_retain(self):
        """Upgraded WindmillStrike should gain +5 damage per retain (not +4).

        Java: baseMagicNumber=4, upgradeMagicNumber(1) -> 5.
        """
        pass

    def test_bowling_bash_hits_target_per_living_enemy(self):
        """Bowling Bash hits the target N times where N = living enemy count.

        Java: BowlingBash.java:30-35 loops over non-dead monsters.
        """
        pass
```

#### CR-3: AcidSlime_S A17 Pattern Test
**Effort:** 30 minutes
**Tests needed:** 2 tests

```python
class TestAcidSlimeSA17Pattern:
    """CRIT-1: AcidSlime_S alternates LICK->TACKLE via takeTurn, not getMove."""

    def test_a17_alternates_lick_tackle(self):
        """At A17+, AcidSlime_S should alternate: LICK, TACKLE, LICK, TACKLE...

        Java: takeTurn() directly sets next move. getMove() only called once.
        """
        slime = AcidSlimeS(make_rng(), ascension=17)
        moves = []
        for _ in range(6):
            move = slime.get_move(50)
            moves.append(move.name)
            slime.take_turn()  # This should set next move directly
        # Pattern should be LICK, TACKLE, LICK, TACKLE, LICK, TACKLE
        assert moves == ["Lick", "Tackle", "Lick", "Tackle", "Lick", "Tackle"]

    def test_below_a17_uses_rng_based_pattern(self):
        """Below A17, AcidSlime_S uses RNG-based move selection."""
        slime = AcidSlimeS(make_rng(), ascension=16)
        move = slime.get_move(50)
        assert move.name in ("Lick", "Tackle")
```

### HIGH Priority (Affects strategy learning accuracy)

#### H-1: Event Mechanics Tests
**Effort:** 4-6 hours (7 CRITICAL + 8 HIGH events)
**Tests needed:** 15-20 tests

```python
class TestEventMechanicsParity:
    """Tests for events-parity-report.md CRITICAL findings."""

    def test_back_to_basics_simplicity_removes_one_card(self):
        """Back to Basics 'Simplicity' should remove ONE card, not strip deck.

        Java: BackToBasics.java option 0 = grid select to remove 1 purgeable card.
        Python bug: removes ALL non-Strike/Defend cards.
        """
        pass

    def test_beggar_pays_75g_to_remove_card(self):
        """The Beggar should cost 75g to remove 1 card, or leave. 2 options.

        Java: Beggar.java has 2 options (pay 75g remove card, leave).
        Python bug: 3 options with relic rewards (completely wrong).
        """
        pass

    def test_mausoleum_always_gives_relic(self):
        """The Mausoleum should ALWAYS give a relic. 50/100% chance of curse.

        Java: Always gives random-tier relic. 50% curse (A15: 100% curse).
        Python bug: 50% relic OR 50% curse (never both).
        """
        pass

    def test_sensory_stone_three_damage_gated_choices(self):
        """Sensory Stone should have 3 choices with increasing HP cost.

        Java: 1 colorless card (free), 2 cards (5 HP), 3 cards (10 HP).
        Python bug: 1 choice giving act-number cards with no HP cost.
        """
        pass

    def test_joust_requires_50g_bet(self):
        """The Joust should require 50g bet. Murderer wins 100g, not 50g.

        Java: 50g entry, Owner win = 250g, Murderer win = 100g.
        Python bug: No bet cost, Murderer win = 50g.
        """
        pass

    def test_library_sleep_heals_33_percent(self):
        """The Library 'Sleep' should heal 33% max HP (20% at A15+).

        Java: TheLibrary.java heals 33%/20% max HP.
        Python bug: Heals to FULL HP.
        """
        pass
```

#### H-2: Enemy AI Missing Patterns
**Effort:** 2-3 hours
**Tests needed:** 5-8 tests

```python
class TestEnemyAIMissingPatterns:
    """Tests for monsters-parity-report.md HIGH findings."""

    def test_looter_flee_sequence(self):
        """Looter should: MUG, MUG, 50% SMOKE/LUNGE, then ESCAPE.

        Java: Looter.java has multi-phase flee pattern.
        Python bug: Always returns MUG.
        """
        pass

    def test_mugger_flee_sequence(self):
        """Mugger should have same flee pattern as Looter."""
        pass

    def test_hexaghost_burn_upgrade_after_inferno(self):
        """After first Inferno, subsequent Sear moves should produce Burn+.

        Java: burnUpgraded = true after Inferno. All later Sear burns are upgraded.
        """
        pass

    def test_bronze_orb_always_stasis_first(self):
        """BronzeOrb should ALWAYS use Stasis on first turn.

        Java: Uses firstMove flag (guaranteed first turn).
        Python bug: 75% chance for Stasis.
        """
        pass

    def test_green_louse_has_correct_hp(self):
        """LouseDefensive (green) should have HP 11-17 (A7: 12-18).

        Python bug: Unified Louse class uses LouseNormal HP for both.
        """
        pass
```

#### H-3: Power Hook Ordering Tests
**Effort:** 2-3 hours
**Tests needed:** 5-6 tests

```python
class TestPowerHookOrdering:
    """Verify power hooks fire in correct order relative to game events."""

    def test_on_play_card_fires_before_card_use(self):
        """P1: onPlayCard hooks should fire BEFORE card effects execute.

        Java: A1-A5 all fire before card.use().
        Python bug: Card removed from hand before onPlayCard triggers.
        """
        pass

    def test_on_use_card_fires_before_card_destination(self):
        """P5: onUseCard should fire BEFORE card goes to exhaust/discard.

        Java: UseCardAction constructor fires onUseCard, update() handles destination.
        Python bug: Card destination (step 9) before onUseCard (step 10).
        """
        pass

    def test_monster_powers_react_to_card_play(self):
        """P2: Monster powers should get onPlayCard notification.

        Java: A2 calls p.onPlayCard for each monster power (e.g., Angry).
        Python bug: Only relic onPlayCard is triggered.
        """
        pass

    def test_damage_hooks_chain_modifies_value(self):
        """D9: Enemy damage power hooks should modify the damage value.

        Java: Each hook in the chain modifies tmp which feeds into next.
        Python bug: execute_power_triggers return values not used.
        """
        pass
```

#### H-4: Double-Trigger Regression Tests
**Effort:** 1 hour
**Tests needed:** 5 tests

```python
class TestDoubleTriggerRegression:
    """Ensure fixed double-trigger bugs don't regress."""

    def test_metallicize_fires_exactly_once(self):
        """Metallicize should grant block exactly once at end of turn.

        Bug: registry handler + inline fallback caused double-fire.
        """
        # Give player Metallicize 3
        # End turn
        # Assert player.block == 3 (not 6)
        pass

    def test_plated_armor_fires_exactly_once(self):
        """Plated Armor should grant block exactly once at end of turn."""
        pass

    def test_like_water_fires_exactly_once(self):
        """Like Water should grant block once at end of turn (in Calm)."""
        pass

    def test_study_fires_exactly_once(self):
        """Study should add one Insight at end of turn, not two."""
        pass

    def test_devotion_triggers_divinity_via_stance_system(self):
        """Devotion should add mantra through _change_stance, not directly.

        Bug: Used player.statuses['Mantra'] instead of state.mantra.
        """
        pass
```

### MEDIUM Priority (Completeness / edge cases)

#### M-1: Missing Power Implementation Tests
**Effort:** 3-4 hours
**Tests needed:** 7 tests (Watcher-specific)

```python
class TestMissingWatcherPowers:
    """Tests for the 7 missing Watcher-specific powers."""

    def test_no_skills_power_blocks_skill_plays(self):
        """NoSkillsPower should prevent playing Skill cards."""
        pass

    def test_vault_power_not_extra_turn(self):
        """VaultPower should apply damage correctly, not grant extra turn."""
        pass

    def test_cannot_change_stance_blocks_stance_changes(self):
        """CannotChangeStancePower should block all stance transitions."""
        pass

    def test_establishment_reduces_retained_card_costs(self):
        """EstablishmentPower should reduce cost of retained cards."""
        pass

    def test_mark_power_pressure_points(self):
        """MarkPower should accumulate and deal damage on Pressure Points."""
        pass

    def test_omega_power_end_of_turn_damage(self):
        """OmegaPower should deal 50 damage to all enemies at end of turn."""
        pass

    def test_energy_down_power_reduces_energy(self):
        """EnergyDownPower should reduce energy at start of turn."""
        pass
```

#### M-2: Damage Application Parity Tests
**Effort:** 2-3 hours
**Tests needed:** 5-6 tests

```python
class TestDamageApplicationParity:
    """Tests for damage-flow-parity findings."""

    def test_torii_applies_to_post_block_damage(self):
        """D8: Torii should reduce unblocked damage >= 5 to 1.

        Java: Torii is onAttacked hook (post-block).
        Python bug: Applies to pre-block damage.
        """
        pass

    def test_intangible_caps_after_torii(self):
        """D8: Intangible should cap damage after Torii check.

        Java: Intangible is atDamageFinalReceive (pre-block, damage calc).
        Need to verify ordering: Intangible then block, or block then Torii.
        """
        pass

    def test_buffer_reduces_unblocked_damage_to_zero(self):
        """Buffer should consume a stack and reduce damage to 0."""
        pass

    def test_multi_hit_stops_on_enemy_death(self):
        """K1: Multi-hit attacks should stop when enemy dies."""
        pass

    def test_relic_at_damage_modify_applied(self):
        """D1: Relic damage modifiers (Pen Nib) should apply."""
        pass
```

#### M-3: Relic Tier System Tests (for events)
**Effort:** 1-2 hours
**Tests needed:** 3-4 tests

```python
class TestRelicTierSystem:
    """Events should use tier-weighted relic selection, not always common."""

    def test_return_random_relic_tier_distribution(self):
        """Verify returnRandomRelicTier produces correct distribution."""
        pass

    def test_event_relic_rewards_use_tier_system(self):
        """Events like Big Fish, Scrap Ooze should use tier-weighted relics."""
        pass

    def test_transform_card_preserves_rarity(self):
        """transformCard should produce a card of the same rarity."""
        pass
```

#### M-4: Full Game Seed Replay Tests
**Effort:** 3-4 hours
**Tests needed:** 3-5 tests

```python
class TestSeedReplay:
    """Seed-deterministic full game replay tests."""

    def test_same_seed_same_actions_identical_outcome(self):
        """Two runs with same seed and same actions should produce identical state."""
        pass

    def test_combat_seed_replay_exact_hp(self):
        """Known seed + known deck + known enemy = exact HP after combat."""
        pass

    def test_card_reward_seed_replay(self):
        """Known floor + known seed = exact card reward options."""
        pass

    def test_map_generation_seed_replay(self):
        """Known seed = exact map layout across acts."""
        pass
```

---

## 4. Coverage Quality Analysis

### Test Type Distribution

| Type | Count | Percentage | Health |
|------|-------|------------|--------|
| **Behavioral** (plays cards, runs combat, checks outcomes) | ~3200 | ~56% | GOOD |
| **Data** (verifies card/power/relic field values) | ~1900 | ~33% | ADEQUATE |
| **Integration** (multi-step game sequences) | ~600 | ~10% | NEEDS MORE |
| **Parity** (seed-deterministic Java comparison) | ~100 | ~2% | NEEDS MORE |

### What Tests Catch Well

1. **Card data correctness** -- 130+ tests per character for stat fields
2. **Damage/block calculation** -- 190+ tests with rounding edge cases
3. **RNG determinism** -- 155+ tests across streams
4. **Enemy HP/damage values** -- Extensive ascension threshold checks
5. **Effect registry wiring** -- 250+ tests for handler registration
6. **Basic combat flow** -- Turn order, energy, draw/discard

### What Tests Miss

1. **Hook ordering** -- No tests verify the sequence of power/relic hooks within a turn phase
2. **Cross-system interactions** -- Limited testing of power+relic+stance+card combos
3. **End-of-turn card auto-play** -- Burn, Regret, Decay completely untested
4. **Event handler correctness** -- Tests verify data structures but not handler output
5. **Combat flow ordering** -- Block decay timing, discard timing relative to powers
6. **Enemy flee/phase patterns** -- Looter/Mugger flee, Hexaghost burn upgrade
7. **Full combat parity** -- No tests that replay an entire combat and compare exact HP

---

## 5. Effort Estimates

| Priority | Area | Tests Needed | Effort | Impact |
|----------|------|-------------|--------|--------|
| **CRITICAL** | Combat flow ordering (CR-1) | 6-8 | 2-3 hours | Fixes wrong HP outcomes in ~10% of combats |
| **CRITICAL** | Card effect regression (CR-2) | 7 | 1-2 hours | Fasting alone makes training data invalid |
| **CRITICAL** | AcidSlime_S pattern (CR-3) | 2 | 30 minutes | Wrong A17 AI sequence |
| **HIGH** | Event mechanics parity (H-1) | 15-20 | 4-6 hours | 7 events have completely wrong mechanics |
| **HIGH** | Enemy AI missing patterns (H-2) | 5-8 | 2-3 hours | Looter/Mugger never flee |
| **HIGH** | Power hook ordering (H-3) | 5-6 | 2-3 hours | Wrong power trigger sequence |
| **HIGH** | Double-trigger regression (H-4) | 5 | 1 hour | Prevent re-introduction of fixed bugs |
| **MEDIUM** | Missing Watcher powers (M-1) | 7 | 3-4 hours | 7 powers have no implementation |
| **MEDIUM** | Damage application parity (M-2) | 5-6 | 2-3 hours | Torii/Intangible order |
| **MEDIUM** | Relic tier system (M-3) | 3-4 | 1-2 hours | Events give wrong relic tiers |
| **MEDIUM** | Seed replay (M-4) | 3-5 | 3-4 hours | No full combat parity verification |

**Total:** ~62-76 new tests, ~22-32 hours of effort

---

## 6. Recommendations Summary

### Immediate (before next training run)

1. **CR-2: Fasting EnergyDown test** -- Without this, Fasting is game-breakingly OP. Write the test, then fix the one-line bug.
2. **CR-1: Block decay ordering test** -- Poison against block gives different HP. Write the test to lock down correct behavior.
3. **H-4: Double-trigger regression tests** -- Bugs were fixed but have no regression guards.

### Before seed-deterministic parity

4. **CR-3: AcidSlime_S A17 pattern**
5. **H-2: Looter/Mugger flee pattern**
6. **H-3: Power hook ordering** (especially onPlayCard before card.use())
7. **M-4: Seed replay tests** for at least 3 known seeds

### Before event-heavy training

8. **H-1: Fix 7 CRITICAL events** (Back to Basics, Beggar, Forgotten Altar, Mausoleum, Nest, Sensory Stone, Designer)
9. **M-3: Relic tier system** (affects 10+ events)

### Ongoing

10. Every bug fix should be preceded by a failing test (per CLAUDE.md bug-fixing policy)
11. Convert DATA-only card tests to BEHAVIORAL tests when touching card implementations
12. Add parity tests for each combat interaction verified against Java

---

## 7. Test Infrastructure Notes

### Strengths

- `conftest.py` provides clean fixtures for combat state creation
- Consistent helper pattern (`_make_engine`, `_make_state`, `_find_card_in_hand`) across test files
- Good use of `@pytest.mark.parametrize` for data-driven tests
- Combat tests use real `CombatEngine` with `start_combat()` for realistic state

### Improvements Needed

- **No shared hook-ordering assertion helpers** -- Need a utility that records the sequence of hooks fired during a turn/action
- **No combat replay utility** -- Need a function that takes (seed, deck, actions[]) and returns final state for parity testing
- **Event handler tests need actual handler execution** -- Current tests check data model, not handler output
- **Missing `@pytest.mark.parity` usage** -- Only defined in conftest but rarely used

### Suggested Test Utility

```python
def record_hook_sequence(engine, action):
    """Execute an action and record which hooks fired in order.

    Returns list of (hook_name, power_name, timing) tuples.
    Usage: verify that atEndOfTurn fires before discard, etc.
    """
    # Monkey-patch execute_power_triggers to record calls
    # Execute action
    # Return recorded sequence
    pass

def replay_combat(seed, deck, actions, ascension=20):
    """Replay a combat and return final state.

    Usage: deterministic parity testing.
    """
    runner = GameRunner(seed=seed, ascension=ascension)
    # Navigate to combat
    # Execute actions in order
    # Return (final_hp, enemy_hp, block, statuses, etc.)
    pass
```
