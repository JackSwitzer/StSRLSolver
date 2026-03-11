# Final Gap Checklist -- Watcher A20 RL Engine

**Date:** 2026-03-03 (verified 2026-03-11)
**Sources:** java-powers-summary.md, java-relics-summary.md, events-parity-report.md, combat-flow-parity.md, monsters-parity-report.md, card-parity-report.md
**Classification:** BLOCKS_RL (wrong combat outcomes, must fix) | AFFECTS_RL (wrong run decisions, should fix) | LOW (skip for now)

> **NOTE:** This checklist was generated from stale audit docs. Many items marked as gaps
> were already fixed earlier in the session. See verification status below.

---

## Summary Counts

| Category | BLOCKS_RL | AFFECTS_RL | LOW | Total |
|----------|-----------|------------|-----|-------|
| Combat Flow | 6 | 8 | 5 | 19 |
| Powers (missing+partial) | 7 | 14 | 29 | 50 |
| Cards (Watcher) | 4 | 5 | 6 | 15 |
| Monsters | 3 | 4 | 3 | 10 |
| Events | 7 | 11 | 5 | 23 |
| Relics | 3 | 12 | 16 | 31 |
| **Total** | **30** | **54** | **64** | **148** |

---

## 1. COMBAT FLOW (19 gaps)

### BLOCKS_RL

- [ ] **CF-01: Block decay fires before start-of-turn triggers (T1)**
  Python decays block at step 4 of `_start_player_turn()`, Java decays at step 10 (AFTER relic/power triggers). Poison ticks against block yield different HP.
  **Fix:** Move block decay in `combat_engine.py:300-313` to AFTER `execute_relic_triggers("atTurnStart")` and `execute_power_triggers("atStartOfTurn")`.

- [ ] **CF-02: Player atEndOfTurn fires AFTER discard instead of BEFORE (E1)**
  Java fires `player.applyEndOfTurnTriggers()` before `DiscardAtEndOfTurnAction`. Python fires `atEndOfTurn` at step 5, after `_discard_hand()` at step 2. Powers like Constricted deal damage while player still has cards in Java.
  **Fix:** In `combat_engine.py:end_turn()`, move `execute_power_triggers("atEndOfTurn")` before `_discard_hand()`.

- [ ] **CF-03: End-of-turn card auto-play missing -- Burn/Regret/Decay (E3)**
  Java `callEndOfTurnActions` iterates hand and calls `c.triggerOnEndOfTurnForPlayingCard()` on each card. Burn deals damage, Regret loses HP per cards in hand, Decay deals damage. Python has no equivalent.
  **Fix:** Add `_trigger_end_of_turn_cards()` in `combat_engine.py:end_turn()` before discard. Loop hand, trigger Burn (deal 2/4 damage to player), Regret (lose HP = hand size), Decay (deal 2 damage to player).

- [ ] **CF-04: atBattleStartPreDraw fires AFTER atBattleStart (C1)**
  Java: `atBattleStartPreDraw` (step 4) fires BEFORE `atBattleStart` (step 7). Python: reversed. Pure Water Miracles should be in draw pile before atBattleStart triggers fire.
  **Fix:** In `combat_engine.py:start_combat()`, swap lines 283 and 286 so `atBattleStartPreDraw` fires first.

- [ ] **CF-05: onPlayCard fires after card removed from hand (P1+P6)**
  Java fires `onPlayCard` hooks while card is still in hand/in-play. Python removes card from hand (step 4) before `onPlayCard` (step 7) and effects (step 8). Hand size is wrong during card execution.
  **Fix:** In `combat_engine.py:play_card()`, fire `onPlayCard` triggers before removing card from hand, or track card-in-play separately.

- [ ] **CF-06: Enemy damage power hooks are fire-and-forget (D9)**
  Python fires `atDamageGive`, `atDamageReceive`, `atDamageFinalReceive` on incoming enemy damage but ignores return values. Java chains these hooks to modify damage. Powers modifying incoming damage are silently ignored.
  **Fix:** In enemy damage path (`combat_engine.py:544-665`), capture return values from power hooks and apply them to damage.

### AFFECTS_RL

- [ ] **CF-07: Blur handled as counter not power (T6)**
  Java Blur retains ALL block while active (like Barricade), decrements at end of round. Python treats as partial-retention counter.
  **Fix:** Match Java: if Blur power exists, skip block decay entirely; decrement Blur in `atEndOfRound`.

- [ ] **CF-08: onUseCard fires after card destination instead of before (P5)**
  Java: `onUseCard` fires in UseCardAction constructor (before card destination). Python: fires after card destination (step 10 vs step 9). Powers checking exhaust pile during onUseCard see wrong state.
  **Fix:** Move `execute_power_triggers("onUseCard")` before card destination logic.

- [ ] **CF-09: Torii applies to pre-block damage instead of post-block (D8)**
  Python applies Torii check before block subtraction. Java Torii is `onAttacked` (fires after block subtraction on unblocked damage only).
  **Fix:** Move Torii check to after block subtraction in enemy-to-player damage path.

- [ ] **CF-10: Missing monster power onPlayCard (P2)**
  Java fires `p.onPlayCard()` on each monster's powers. Python only triggers relic onPlayCard. Enemy powers like Angry don't react to card plays via this hook.
  **Fix:** Add `execute_power_triggers("onPlayCard")` for all monsters in `play_card()`.

- [ ] **CF-11: applyStartOfTurnCards missing (C4+T4)**
  Java calls `c.atTurnStart()` on all cards in draw/hand/discard. Python has no equivalent. Cards with per-turn state (cost reductions that expire) don't reset.
  **Fix:** Add card `atTurnStart` loop in `_start_player_turn()`.

- [ ] **CF-12: Missing stance.onPlayCard (P3)**
  Java calls `player.stance.onPlayCard(card)` before card execution. Python has no equivalent.
  **Fix:** Add stance onPlayCard hook in `play_card()`.

- [ ] **CF-13: Enemy atEndOfTurn may double-fire (E5)**
  Python step 6 fires `atEndOfTurn` for each enemy, then `_do_enemy_turns()` also handles enemy effects. Java fires enemy atEndOfTurn only once (MonsterGroup step F4).
  **Fix:** Remove duplicate enemy atEndOfTurn trigger; fire only in the correct phase.

- [ ] **CF-14: Enemy debuff decrement at wrong timing (E9)**
  Python decrements Weak/Vuln/Frail inside `_do_enemy_turns()`. Java handles via `atEndOfRound` after all monster actions complete.
  **Fix:** Move debuff decrement to atEndOfRound phase.

### LOW

- [ ] **CF-15: Turn increment at start vs middle (T3)** -- Powers checking turn number see different values. Minor.
- [ ] **CF-16: Energy recharge timing differs (C2+T2)** -- onEnergyRecharge hooks fire at wrong relative position.
- [ ] **CF-17: Divinity exit timing vs Java (T5)** -- Functionally equivalent for Watcher.
- [ ] **CF-18: applyStartOfTurnPreDrawCards missing (C5)** -- Only affects first turn, minor.
- [ ] **CF-19: Ethereal exhaust mechanism differs (E4)** -- Same result, different mechanism.

---

## 2. POWERS (38 gaps -- 38 MISSING + 12 PARTIAL, categorized below)

### BLOCKS_RL (6 powers)

- [ ] **PW-01: AngerPower MISSING** (Java ID: `Anger`)
  Gremlin Nob uses this. On player Skill play, enemy gains Strength. Without it, Nob is trivial.
  **Fix:** Register `onUseCard` trigger: if card is Skill, apply Strength(amount) to owner.

- [ ] **PW-02: EnergyDownPower MISSING** (Java ID: `EnergyDownPower`) [WATCHER]
  Fasting card applies this. Reduces energy per turn. Without it, Fasting is game-breakingly OP.
  **Fix:** Register `atStartOfTurn` trigger: reduce max_energy by amount. Also needed for Snecko Eye's Confused interaction.

- [ ] **PW-03: EndTurnDeathPower MISSING** (Java ID: `EndTurnDeath`) [WATCHER]
  Blasphemy applies this. Player dies at start of next turn. Without it, Blasphemy has no downside.
  **Fix:** Register `atStartOfTurn` trigger: kill player (set HP to 0).

- [ ] **PW-04: ConfusionPower MISSING** (Java ID: `Confusion`)
  Snecko Eye applies this. Randomizes card costs on draw. Core mechanic for Snecko Eye boss relic.
  **Fix:** Register `onCardDraw` trigger: set card cost to random 0-3 via cardRandomRng.

- [ ] **PW-05: CurlUpPower MISSING** (Java ID: `Curl Up`)
  Louse uses this. On first attack received, gain block. Currently hardcoded in enemy logic but should be power-driven.
  **Fix:** Register `onAttacked` trigger: if first attack, gain block(amount), remove power.

- [ ] **PW-06: ArtifactPower MISSING** (Java ID: `Artifact`)
  Prevents debuff application. Clockwork Souvenir grants 1, several events grant it. Without it, debuff prevention is broken.
  **Fix:** Register `onSpecificTrigger` (called when debuff would be applied): decrement stacks, prevent debuff.

### AFFECTS_RL (10 powers)

- [ ] **PW-07: WrathNextTurnPower MISSING** [WATCHER]
  Simmering Fury applies this. Enter Wrath at start of next turn.
  **Fix:** Register `atStartOfTurn`: change stance to Wrath, remove self.

- [ ] **PW-08: CollectPower MISSING**
  Collect card (Watcher) applies this. Add Miracles to hand on energy recharge.
  **Fix:** Register `onEnergyRecharge`: add X Miracle cards to hand, remove self.

- [ ] **PW-09: VaultPower MISSING** [WATCHER]
  Vault card applies this. Skip enemy turn at end of round.
  **Fix:** Register `atEndOfRound`: skip enemy turns this round, remove self.

- [ ] **PW-10: LiveForeverPower MISSING** (Java ID: `AngelForm`) [WATCHER]
  Wish "Live Forever" option applies this. Gain Plated Armor each turn.
  **Fix:** Register `atEndOfTurn`: apply PlatedArmor(amount).

- [ ] **PW-11: GainStrengthPower MISSING** (Java ID: `Shackled`)
  Shackle card / Taskmaster applies this. Regain Strength at end of turn.
  **Fix:** Register `atEndOfTurn`: apply Strength(amount), remove self.

- [ ] **PW-12: HexPower MISSING**
  Hexaghost/Hex applies this. On non-Attack card play, shuffle Daze into draw pile.
  **Fix:** Register `onUseCard`: if card is not Attack, shuffle Daze into draw pile.

- [ ] **PW-13: SporeCloudPower MISSING** (Java ID: `Spore Cloud`)
  Fungi Beast death trigger. Apply 2 Vulnerable to player on death.
  **Fix:** Register `onDeath`: apply Vulnerable(amount) to player.

- [ ] **PW-14: SharpHidePower MISSING** (Java ID: `Sharp Hide`)
  Shelled Parasite uses this. Deal damage to player on Attack play.
  **Fix:** Register `onUseCard`: if Attack, deal damage(amount) to player.

- [ ] **PW-15: ReactivePower MISSING** (Java ID: `Compulsive`)
  Writhing Mass uses this. Re-roll intent when attacked.
  **Fix:** Register `onAttacked`: re-roll enemy move.

- [ ] **PW-16: RegenerateMonsterPower MISSING** (Java ID: `Regenerate`)
  Darkling / Healer uses this. Heal at end of turn.
  **Fix:** Register `atEndOfTurn`: heal owner by amount.

### LOW (22 powers -- non-Watcher or edge cases)

- [ ] **PW-17: AmplifyPower MISSING** -- Defect (Amplify card)
- [ ] **PW-18: AttackBurnPower MISSING** -- Enemy-only (Book of Stabbing)
- [ ] **PW-19: ConservePower MISSING** -- Defect (Conserve)
- [ ] **PW-20: DrawPower MISSING** -- Modifies hand size (Offering, etc.)
- [ ] **PW-21: DrawReductionPower MISSING** -- Modifies hand size
- [ ] **PW-22: EnergizedBluePower MISSING** -- Defect variant
- [ ] **PW-23: ExplosivePower MISSING** -- Exploder enemy (hardcoded instead)
- [ ] **PW-24: ForcefieldPower MISSING** -- Sphere Guardian (Nullify Attack)
- [ ] **PW-25: GenericStrengthUpPower MISSING** -- Awakened One grows
- [ ] **PW-26: HelloPower MISSING** -- Defect (Hello World)
- [ ] **PW-27: NightmarePower MISSING** -- Silent (Nightmare card)
- [ ] **PW-28: NoSkillsPower MISSING** -- Watcher (minor, Chosen debuff)
- [ ] **PW-29: PainfulStabsPower MISSING** -- Book of Stabbing (adds Wound on damage)
- [ ] **PW-30: ReboundPower MISSING** -- Defect (Hologram)
- [ ] **PW-31: RechargingCorePower MISSING** -- Defect (Bronze Automaton)
- [ ] **PW-32: ShiftingPower MISSING** -- Nemesis (gain Str on attack)
- [ ] **PW-33: SkillBurnPower MISSING** -- Enemy (Skill exhaust)
- [ ] **PW-34: StasisPower MISSING** -- Bronze Orb (return card on death)
- [ ] **PW-35: TheBombPower MISSING** -- Time Eater (ticking bomb)
- [ ] **PW-36: TimeMazePower MISSING** -- Time Eater maze (hardcoded instead)
- [ ] **PW-37: WinterPower MISSING** -- Defect (Winter)
- [ ] **PW-38: StrikeUpPower MISSING** -- Whetstone (display only)

### PARTIAL powers (12 -- upgrade to full)

- [ ] **PW-P1: BlurPower PARTIAL** -- Block retention logic may differ with multiple stacks. **BLOCKS_RL** (see CF-07).
- [ ] **PW-P2: PlatedArmorPower PARTIAL** -- `wasHPLost` decrement may be missing. **AFFECTS_RL**.
- [ ] **PW-P3: RitualPower PARTIAL** -- Fires at atEndOfTurn AND atEndOfRound; check both are wired. **AFFECTS_RL**.
- [ ] **PW-P4: FlightPower PARTIAL** -- atDamageFinalReceive (halve damage) may be incomplete. **AFFECTS_RL**.
- [ ] **PW-P5: FadingPower PARTIAL** -- duringTurn kill-self logic. **AFFECTS_RL**.
- [ ] **PW-P6: RagePower PARTIAL** -- onUseCard block gain only on Attacks. **LOW**.
- [ ] **PW-P7: DoubleTapPower PARTIAL** -- Queue copy logic. **LOW**.
- [ ] **PW-P8: DuplicationPower PARTIAL** -- Queue copy logic. **LOW**.
- [ ] **PW-P9: AccuracyPower PARTIAL** -- onDrawOrDiscard Shiv cost. **LOW**.
- [ ] **PW-P10: JuggernautPower PARTIAL** -- onGainedBlock deal damage. **LOW**.
- [ ] **PW-P11: WaveOfTheHandPower PARTIAL** -- onGainedBlock apply Weak. **LOW**.
- [ ] **PW-P12: RegenPower PARTIAL** -- atEndOfTurn heal + decrement. **LOW**.

---

## 3. CARDS -- WATCHER (15 gaps)

### BLOCKS_RL (4 cards)

- [ ] **CD-01: Fasting missing EnergyDown**
  Python only applies Str+Dex. Java also applies `EnergyDownPower(1)`. Fasting without energy penalty is massively OP.
  **Fix:** `effects/cards.py:1121-1132` -- add `ctx.apply_status_to_player("EnergyDown", 1)`.

- [ ] **CD-02: Spirit Shield includes self in hand count**
  Java explicitly skips `this` card when counting hand. Python counts `len(ctx.hand)` which includes Spirit Shield.
  **Fix:** `effects/cards.py:554-558` -- use `len(ctx.hand) - 1`.

- [ ] **CD-03: Conjure Blade upgraded should give X+1 hits**
  Java passes `energyOnUse + 1` when upgraded. Python has no upgrade difference.
  **Fix:** `effects/cards.py:865-871` -- when upgraded, add 1 to X value for Expunger creation.

- [ ] **CD-04: Bowling Bash should hit target once per living enemy**
  Java loops all living monsters and adds DamageAction to TARGET for each. If engine doesn't multiply by enemy count, damage is wrong.
  **Fix:** Verify `damage_per_enemy` effect multiplies hits by `len(living_enemies)`.

### AFFECTS_RL (5 cards)

- [ ] **CD-05: Wreath of Flame uses "WreathOfFlame" status instead of "Vigor"**
  Java uses `VigorPower`. Using a separate status prevents correct stacking with other Vigor sources.
  **Fix:** `effects/cards.py:752-756` -- change to `ctx.apply_status_to_player("Vigor", amount)`.

- [ ] **CD-06: Simmering Fury applies combined power instead of two separate powers**
  Java applies `WrathNextTurnPower` + `DrawCardNextTurnPower(magic)`. Python uses single "SimmeringFury".
  **Fix:** Apply two separate powers: "WrathNextTurn" and "DrawCardNextTurn(amount)".

- [ ] **CD-07: WindmillStrike upgraded gains +4/turn instead of +5/turn**
  Java `baseMagicNumber=4, upgradeMagicNumber(1)=5`. Python effect hardcodes +4.
  **Fix:** Set `base_magic=4, upgrade_magic=1` in card definition; update effect to use `ctx.magic_number`.

- [ ] **CD-08: Vengeance effect mismatch**
  Needs verification that the effect correctly enters Wrath if in Calm, or gains Mantra if not.
  **Fix:** Verify effect handler matches Java `Vengeance.java:use()`.

- [ ] **CD-09: BowlingBash / FlyingSleeves multi-hit verification**
  Both cards rely on pass-through effects. Need to verify EffectExecutor handles multi-hit correctly.
  **Fix:** Write test: BowlingBash with 3 enemies should hit target 3 times.

### LOW (6 cards)

- [ ] **CD-10: Halt missing base_magic=9** -- Display/observation only.
- [ ] **CD-11: Brilliance missing base_magic=0** -- Display/observation only.
- [ ] **CD-12: WheelKick missing base_magic=2** -- Display/observation only.
- [ ] **CD-13: Omniscience missing base_magic=2** -- Display/observation only.
- [ ] **CD-14: FlyingSleeves missing explicit hits field** -- Verify 2-hit behavior.
- [ ] **CD-15: Indignation doc says "Mantra" but code correctly applies Vulnerable** -- Doc fix only.

---

## 4. MONSTERS (10 gaps)

### BLOCKS_RL (3 monsters)

- [ ] **MN-01: AcidSlime_S A17 pattern uses getMove instead of takeTurn override**
  Java alternates TACKLE/LICK via direct setMove in `takeTurn()`. Python calls `get_move()` every turn which produces different sequences at A17+.
  **Fix:** `enemies.py:892-898` -- replicate Java takeTurn pattern: move 1 (TACKLE) sets next to LICK, move 2 (LICK) sets next to TACKLE.

- [ ] **MN-02: Looter/Mugger simplified to always MUG (no flee pattern)**
  Java: MUG->MUG->50% SMOKE_BOMB/LUNGE->ESCAPE. Python always returns MUG. These enemies never flee, significantly different combat behavior.
  **Fix:** `enemies.py:2511-2516` and `enemies.py:2454-2460` -- implement full multi-turn pattern with flee logic.

- [ ] **MN-03: BronzeOrb first move should always be Stasis**
  Java uses `firstMove` flag (guaranteed Stasis on turn 1). Python uses 75% chance.
  **Fix:** `enemies.py:5277` -- if `firstMove`, always use Stasis.

### AFFECTS_RL (4 monsters)

- [ ] **MN-04: Hexaghost burn upgrade after Inferno**
  Java sets `burnUpgraded=true` after first Inferno. Subsequent Sear moves produce Burn+ (upgraded). Python always produces regular Burns.
  **Fix:** `enemies.py:1880-1919` -- track `burn_upgraded` flag, toggle after Inferno.

- [ ] **MN-05: WrithingMass Reactive re-roll on damage**
  Java re-rolls intent when damaged mid-turn. Python has `reactive` flag but no re-roll behavior wired.
  **Fix:** CombatEngine must call `roll_move()` on WrithingMass when it takes damage.

- [ ] **MN-06: SpireGrowth Constricted check requires caller context**
  Python `get_move(roll, player_constricted=False)` requires CombatEngine to pass player's Constricted status.
  **Fix:** CombatEngine must pass `player_constricted=has_power("Constricted")` when rolling SpireGrowth moves.

- [ ] **MN-07: Unified Louse class uses wrong HP for green variant**
  Unified `Louse` class uses LouseNormal HP (10-15 / 11-16) for both variants. Green Louse should be 11-17 / 12-18.
  **Fix:** `enemies.py:1130-1133` -- branch HP by `is_red` flag.

### LOW (3 monsters)

- [ ] **MN-08: Louse CurlUp set in __init__ instead of usePreBattleAction** -- RNG timing difference.
- [ ] **MN-09: Exploder uses manual turn counting instead of ExplosivePower** -- Works but fragile.
- [ ] **MN-10: Maw HP may differ at A7+ (300 vs 310)** -- Needs Java source verification.

---

## 5. EVENTS (23 gaps)

### BLOCKS_RL (7 events -- CRITICAL wrong mechanics)

- [ ] **EV-01: Back to Basics -- "Simplicity" removes ALL non-basics instead of 1 card**
  Java: remove 1 card (player chooses). Python: removes ALL non-Strike/Defend cards.
  **Fix:** `event_handler.py` -- change to remove 1 purgeable card (present selection).

- [ ] **EV-02: The Beggar -- completely wrong mechanics**
  Java: 2 options (pay 75g to remove 1 card, or leave). Python: 3 options (donate 50g/100g for relic). Entirely different event.
  **Fix:** Rewrite Beggar handler to match Java.

- [ ] **EV-03: Forgotten Altar -- "Shed Blood" option wrong**
  Java: +5 max HP, take 25%/35% damage. Python: -5/7 HP, gain random relic. Different mechanics.
  **Fix:** Rewrite option 1 to match Java (max HP gain + percentage damage).

- [ ] **EV-04: The Mausoleum -- relic/curse logic inverted**
  Java: ALWAYS get relic; 50%/100%(A15) also get Writhe. Python: 50% relic only, 50% curse only.
  **Fix:** Always give relic; conditionally also give Writhe.

- [ ] **EV-05: The Nest -- both options wrong**
  Java: steal=gold only (99/50 A15); join=6 HP damage + Ritual Dagger. Python: smash=99g+random card; stay=Ritual Dagger (no HP cost).
  **Fix:** Rewrite both options to match Java.

- [ ] **EV-06: Sensory Stone -- completely different structure**
  Java: 3 choices (1/2/3 colorless cards for 0/5/10 HP). Python: 1 choice, act-based card count.
  **Fix:** Rewrite to 3 options with HP costs.

- [ ] **EV-07: Designer -- completely different option structure**
  Java: randomized options via `miscRng.randomBoolean()` (upgrade 1 or 2, remove 1 or transform 2, full service); costs 40-110. Python: fixed options, wrong costs.
  **Fix:** Rewrite Designer to use randomized option generation matching Java.

### AFFECTS_RL (11 events -- significant value/choice errors)

- [ ] **EV-08: Wing Statue -- missing conditional gold option**
  Java has 3rd option: if player has Attack with 10+ damage, gain 50-80 gold.
  **Fix:** Add conditional gold option when player has qualifying Attack card.

- [ ] **EV-09: Pleading Vagrant (Addict) -- option 1 wrong**
  Java option 1 = steal relic + curse. Python option 1 = curse only (no relic).
  **Fix:** Add relic reward to option 1.

- [ ] **EV-10: Drug Dealer (Augmenter) -- two errors**
  Option 0: Java gives J.A.X. only, Python also removes card. Option 2: Java gives MutagenicStrength specifically, Python randomizes.
  **Fix:** Remove card removal from option 0; fix option 2 to always give MutagenicStrength.

- [ ] **EV-11: The Joust -- missing 50g bet cost, wrong murderer reward**
  Java: costs 50g to bet. Murderer win = 100g, not 50g.
  **Fix:** Add 50g cost; fix murderer reward to 100g.

- [ ] **EV-12: The Library -- "Sleep" heals wrong amount**
  Java: 33%/20%(A15+). Python: heals to full.
  **Fix:** Change sleep heal to `ceil(max_hp * 0.33)` or `ceil(max_hp * 0.20)` on A15+.

- [ ] **EV-13: Tomb of Lord Red Mask -- choice structure inverted, wrong gold**
  Java: if has Red Mask -> gain 222 gold. If not -> pay all gold for Red Mask. Python: inverted.
  **Fix:** Rewrite choice structure matching Java.

- [ ] **EV-14: Face Trader -- both options wrong**
  Option 0: Java gives gold+damage, not relic. Option 1: random face relic (no gold cost).
  **Fix:** Rewrite both options to match Java.

- [ ] **EV-15: N'loth -- wrong relic selection**
  Java: presents 2 random relics to choose from. Python: auto-trades oldest relic.
  **Fix:** Present 2 random relics for player selection.

- [ ] **EV-16: Relic tier system -- affects ~10 events**
  Multiple events use `returnRandomRelicTier()` (tier-weighted Common/Uncommon/Rare/Shop). Python always uses common.
  **Fix:** Implement `return_random_relic_tier()` equivalent; apply to BigFish, DeadAdventurer, ScrapOoze, Addict, Bonfire, WeMeetAgain, etc.

- [ ] **EV-17: Transform card system -- affects ~4 events**
  Java `transformCard()` preserves rarity. Python `_get_random_card(common)` always gives common.
  **Fix:** Implement rarity-preserving transform; apply to LivingWall, Transmogrifier, DrugDealer, Designer.

- [ ] **EV-18: Living Wall -- "Change" transforms to common instead of same-rarity**
  Same as EV-17 but specifically high-impact for Living Wall (common event).
  **Fix:** Use rarity-preserving transform.

### LOW (5 events)

- [ ] **EV-19: Colosseum -- button mapping reversed** -- MEDIUM, choice ordering.
- [ ] **EV-20: Cursed Tome -- missing "stop reading" option** -- Simplified OK for RL.
- [ ] **EV-21: Mushrooms -- wrong A15 heal (20% vs 25%)** -- Small value difference.
- [ ] **EV-22: MathUtils.round vs int() rounding** -- Affects ~5 events, edge cases.
- [ ] **EV-23: Potion class weighting** -- Several events use flat random instead of class-weighted.

---

## 6. RELICS (31 gaps)

### BLOCKS_RL (3 relics -- combat impact)

- [ ] **RL-01: Chemical X MISSING**
  X-cost cards get +2 effect. Passive check needed in card play code. Directly affects combat damage/block.
  **Fix:** In X-cost card handling, check `has_relic("Chemical X")` and add 2 to X value.

- [ ] **RL-02: Blue Candle PARTIAL (onUseCard TODO)**
  Playing Curses should exhaust them and cost 1 HP. Passive flag exists but exhaust+HP loss not wired.
  **Fix:** Add `onUseCard` trigger: if Curse, exhaust card, lose 1 HP.

- [ ] **RL-03: Medical Kit PARTIAL (onUseCard TODO)**
  Status cards should be playable and exhaust. Passive flag exists but not wired.
  **Fix:** Add `onUseCard` trigger: if Status, exhaust card.

### AFFECTS_RL (12 relics -- run-level impact)

- [ ] **RL-04: Neow's Lament MISSING**
  First 3 combats: enemies have 1 HP. Counter decrements. Major Act 1 impact.
  **Fix:** `atBattleStart` trigger: if counter > 0, set all enemy HP to 1, decrement.

- [ ] **RL-05: Omamori MISSING**
  Negate next 2 Curses. Counter decrements. Major for pathing through curse events.
  **Fix:** Passive check on curse acquisition: if counter > 0, negate curse, decrement.

- [ ] **RL-06: Astrolabe MISSING**
  Boss relic: transform+upgrade 3 cards. Affects boss relic valuation.
  **Fix:** `onEquip`: present 3 cards for transform, upgrade each result.

- [ ] **RL-07: Calling Bell MISSING**
  Boss relic: get 1 common + 1 uncommon + 1 rare relic + 1 curse.
  **Fix:** `onEquip`: generate 3 relics (one per tier) + add curse.

- [ ] **RL-08: Empty Cage MISSING**
  Boss relic: remove 2 cards. Affects boss relic valuation.
  **Fix:** `onEquip`: present 2 cards for removal.

- [ ] **RL-09: Pandora's Box MISSING**
  Boss relic: transform all Strikes and Defends.
  **Fix:** `onEquip`: transform all starter cards.

- [ ] **RL-10: Tiny House MISSING**
  Boss relic: 50g, +5 max HP, potion, card, upgrade.
  **Fix:** `onEquip`: apply all 5 bonuses.

- [ ] **RL-11: Prayer Wheel MISSING**
  2 card rewards instead of 1. Affects deck building.
  **Fix:** Passive check in card reward generation: double rewards.

- [ ] **RL-12: Question Card MISSING**
  Card rewards have 1 extra choice. Affects card reward decisions.
  **Fix:** Passive check in card reward: add 1 extra card choice.

- [ ] **RL-13: Old Coin MISSING**
  300 Gold on pickup. Impacts shop decisions.
  **Fix:** `onEquip`: add 300 gold.

- [ ] **RL-14: Busted Crown PARTIAL**
  Passive flag for -2 card choices missing. Energy equip hook missing.
  **Fix:** Wire passive card choice reduction; add energy on equip.

- [ ] **RL-15: Singing Bowl MISSING**
  Skip card reward = +2 Max HP. Affects card reward decisions.
  **Fix:** Passive check in card reward screen: offer skip-for-HP option.

### LOW (16 relics)

- [ ] **RL-16: Cursed Key (onChestOpen)** -- Curse on chest open missing.
- [ ] **RL-17: Matryoshka MISSING** -- Extra chest relics.
- [ ] **RL-18: Eternal Feather MISSING** -- Heal at rest sites.
- [ ] **RL-19: White Beast Statue MISSING** -- More potion drops.
- [ ] **RL-20: Wing Boots MISSING** -- Path flexibility (3 flies per act).
- [ ] **RL-21: Cauldron MISSING** -- Choose 5 potions on pickup.
- [ ] **RL-22: Dolly's Mirror MISSING** -- Duplicate card on pickup.
- [ ] **RL-23: Orrery MISSING** -- Add 5 cards on pickup.
- [ ] **RL-24: Waffle MISSING** -- +7 Max HP, full heal on pickup.
- [ ] **RL-25: Potion Belt MISSING** -- +2 potion slots.
- [ ] **RL-26: Tiny Chest MISSING** -- Every 4th ? room becomes Treasure.
- [ ] **RL-27: Runic Dome MISSING** -- Boss energy relic (can't see intent -- UI only).
- [ ] **RL-28: Ring of the Serpent MISSING** -- Silent boss relic.
- [ ] **RL-29: Prismatic Shard MISSING** -- Cross-class cards.
- [ ] **RL-30: Circlet / Red Circlet / Spirit Poop MISSING** -- No game effect.
- [ ] **RL-31: N'loth's Gift / Mask MISSING** -- Event-specific.

---

## Fix Priority Order

### Phase 1: BLOCKS_RL (30 items) -- Must fix before training

**Combat Flow (6):** CF-01 through CF-06
**Powers (7):** PW-01 (Anger), PW-02 (EnergyDown), PW-03 (EndTurnDeath), PW-04 (Confusion), PW-05 (CurlUp), PW-06 (Artifact), PW-P1 (Blur block retention)
**Cards (4):** CD-01 (Fasting), CD-02 (Spirit Shield), CD-03 (Conjure Blade), CD-04 (Bowling Bash)
**Monsters (3):** MN-01 (AcidSlime_S), MN-02 (Looter/Mugger), MN-03 (BronzeOrb)
**Events (7):** EV-01 through EV-07
**Relics (3):** RL-01 (Chemical X), RL-02 (Blue Candle), RL-03 (Medical Kit)

### Phase 2: AFFECTS_RL (54 items) -- Should fix before serious training

**Combat Flow (8):** CF-07 through CF-14
**Powers (14):** PW-07 through PW-16, PW-P2 through PW-P5
**Cards (5):** CD-05 through CD-09
**Monsters (4):** MN-04 through MN-07
**Events (11):** EV-08 through EV-18
**Relics (12):** RL-04 through RL-15

### Phase 3: LOW (64 items) -- Fix as needed

Everything else. Most are non-Watcher, edge cases, or cosmetic.

---

## Quick Wins (< 30 min each)

These are one-line or small fixes with outsized impact:

1. **CD-01: Fasting EnergyDown** -- 1 line: `ctx.apply_status_to_player("EnergyDown", 1)`
2. **CD-02: Spirit Shield hand count** -- 1 line: `len(ctx.hand) - 1`
3. **CF-04: Swap atBattleStartPreDraw order** -- swap 2 lines in `start_combat()`
4. **CD-05: Wreath of Flame -> Vigor** -- change string `"WreathOfFlame"` to `"Vigor"`
5. **MN-03: BronzeOrb first move** -- change `roll >= 25` to `if firstMove: always Stasis`
6. **RL-01: Chemical X** -- add `+ 2` check in X-cost card handling
7. **EV-04: Mausoleum** -- always give relic, conditionally add Writhe
8. **EV-12: Library sleep heal** -- change full heal to 33%/20%

---

## Test Strategy

For each BLOCKS_RL fix:
1. Write a failing test that demonstrates the wrong behavior
2. Fix the code
3. Verify the test passes
4. Run full suite to check no regressions

For AFFECTS_RL: batch fixes by category (all events, all relics, etc.) with integration tests.

For LOW: fix opportunistically when touching nearby code.
