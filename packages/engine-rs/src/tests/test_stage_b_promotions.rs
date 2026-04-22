// Cycle 6: Stage B tail P1 promotion tests (D172 – D181).
//
// Each test documents a Java divergence promoted from the Stage B scratch
// reports into the parity-deviations register. All `#[ignore]` cases assert
// the *Java-correct* expectation; the current Rust code diverges, so the
// ignore attribute keeps the suite green while carrying a precise red test
// that will turn green once the bug is fixed in a later cycle.
//
// Cross-references:
//   docs/work_units/parity-deviations-register.md rows D172–D181
//   docs/work_units/audit-reports/scratch-2026-04-21/*.md (stage B reports)
//   decompiled/java-src/... (Java source of truth — paths cited per test)

#![cfg(test)]

// -----------------------------------------------------------------------
// D172 — Collector A19+ scaling (strAmt=5, megaDebuffAmt=5, rakeDmg=21)
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/monsters/city/TheCollector.java`
//   L95-107:  if ascension >= 19 { rakeDmg = 21; strAmt = 5; megaDebuffAmt = 5 }
//             else if ascension >= 4 { rakeDmg = 21; strAmt = 4; megaDebuffAmt = 3 }
//             else { rakeDmg = 18; strAmt = 3; megaDebuffAmt = 3 }
//   L140-148: Buff move block = ascension >= 19 ? blockAmt + 5 : blockAmt
//
// Rust `packages/engine-rs/src/enemies/mod.rs:652-663` only branches on
// `hp >= 300` — missing the A19+ scaling layer and hardcoding
// VULN/WEAK/FRAIL at `3` in `roll_collector` (`enemies/act2.rs:498-500`)
// regardless of megaDebuffAmt.
#[test]
#[ignore = "D172 open — Cycle 8+"]
fn d172_collector_a19_scaling() {
    use crate::enemies::create_enemy_with_ascension;
    use crate::status_ids::sid;

    // At A19+ Java sets megaDebuffAmt = 5; Rust must surface this so the
    // MegaDebuff move applies 5 VULN/WEAK/FRAIL (not the hardcoded 3).
    let collector = create_enemy_with_ascension("TheCollector", 300, 300, 19);
    let mega_amt = collector.entity.status(sid::STR_AMT); // proxy: Rust would need a MEGA_DEBUFF_AMT status
    assert_eq!(
        mega_amt, 5,
        "A19+ Collector strAmt must be 5 per Java TheCollector.java:97"
    );
}

// -----------------------------------------------------------------------
// D173 — Collector Revive (move id 5) never dispatched
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/monsters/city/TheCollector.java`
//   L189-192: if (num <= 25 && isMinionDead() && !lastMove(5))
//               setMove((byte)5, UNKNOWN);
//   L161-169: case 5 respawns TorchHead minions.
//
// Rust `packages/engine-rs/src/enemies/act2.rs:485` has a comment
//   "Revive (deferred: needs minion-dead signal)"
// and the REVIVE branch is skipped entirely. `move_ids::COLL_REVIVE = 5`
// exists (`enemies/mod.rs:286`) but nothing in the dispatcher ever
// selects it, so a dead TorchHead is never re-summoned.
#[test]
#[ignore = "D173 open — Cycle 8+"]
fn d173_collector_revive_dispatch() {
    use crate::enemies::{create_enemy, move_ids, roll_next_move_with_num};

    let mut collector = create_enemy("TheCollector", 282, 282);
    // Simulate three non-revive moves so turns >= 3 has elapsed.
    roll_next_move_with_num(&mut collector, 70);
    roll_next_move_with_num(&mut collector, 70);
    roll_next_move_with_num(&mut collector, 70);
    // Ideal test would set a minion-dead signal and check `num <= 25`
    // routes to REVIVE. Current Rust has no such path; this assertion
    // verifies the constant is wired once the minion-dead signal lands.
    roll_next_move_with_num(&mut collector, 10);
    assert_eq!(
        collector.move_id,
        move_ids::COLL_REVIVE,
        "Collector must route to REVIVE(5) on num<=25 with a dying minion (Java L189-192)"
    );
}

// -----------------------------------------------------------------------
// D174 — Damaru Mantra timing (TurnStartPostDrawLate vs Java atTurnStart)
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/relics/Damaru.java`
//   L32-36: atTurnStart() applies MantraPower(1). `atTurnStart` is the
//   pre-draw relic hook — Mantra must be up before any turn-1 draw logic
//   (Devotion, Miracle retain decisions, Scry count from Cut Through Fate).
//
// Rust `packages/engine-rs/src/relics/defs/damaru.rs:11-18` wires the
// effect to `Trigger::TurnStartPostDrawLate` — two trigger phases too
// late. Impact: on turn 1, hand is drawn before Damaru grants the
// Mantra, so turn-1 Collect / Pray / Devotion consume the wrong Mantra
// count.
#[test]
fn d174_damaru_timing_currently_post_draw_late() {
    use crate::effects::trigger::Trigger;
    use crate::relics::defs::damaru::DEF as DAMARU_DEF;

    // Current Rust timing — captures the bug for the parity ledger.
    assert_eq!(DAMARU_DEF.triggers.len(), 1);
    assert_eq!(
        DAMARU_DEF.triggers[0].trigger,
        Trigger::TurnStartPostDrawLate,
        "D174: Damaru currently wired to TurnStartPostDrawLate; Java atTurnStart pre-draw"
    );
}

#[test]
#[ignore = "D174 open — Cycle 8+"]
fn d174_damaru_timing_java_is_pre_draw() {
    use crate::effects::trigger::Trigger;
    use crate::relics::defs::damaru::DEF as DAMARU_DEF;

    // Java `Damaru.atTurnStart` fires pre-draw — Rust should use `TurnStart`.
    assert_eq!(
        DAMARU_DEF.triggers[0].trigger,
        Trigger::TurnStart,
        "Damaru.java:32 uses atTurnStart (pre-draw)"
    );
}

// -----------------------------------------------------------------------
// D175 — Devotion timing + A>=10 Divinity-entry branch (WS1 + WS2)
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/DevotionPower.java`
//   L34-42: atStartOfTurnPostDraw() — post-draw.
//           if (!player.hasPower("Mantra") && this.amount >= 10) {
//               addToBot(new ChangeStanceAction("Divinity"));
//           } else { apply MantraPower(amount) }
//
// Rust `packages/engine-rs/src/powers/defs/turn_start.rs:178-187` uses
//   Trigger::TurnStart (pre-draw) and has NO `amount >= 10 && !Mantra`
//   Divinity-entry branch — Devotion always applies MantraPower. Net:
//   timing skew + the primary Devotion-to-Divinity win condition is
//   silently broken.
#[test]
fn d175_devotion_timing_currently_pre_draw() {
    use crate::effects::trigger::Trigger;
    use crate::powers::defs::DEF_DEVOTION;

    assert_eq!(DEF_DEVOTION.triggers.len(), 1);
    assert_eq!(
        DEF_DEVOTION.triggers[0].trigger,
        Trigger::TurnStart,
        "D175: Devotion currently fires pre-draw; Java uses atStartOfTurnPostDraw"
    );
}

#[test]
#[ignore = "D175 open — Cycle 8+"]
fn d175_devotion_timing_java_is_post_draw() {
    use crate::effects::trigger::Trigger;
    use crate::powers::defs::DEF_DEVOTION;

    assert_eq!(
        DEF_DEVOTION.triggers[0].trigger,
        Trigger::TurnStartPostDraw,
        "DevotionPower.java:34 uses atStartOfTurnPostDraw"
    );
    // WS2: also needs the `amount >= 10 && !has(Mantra)` -> ChangeStance(Divinity)
    // branch. Rust has no such branch today (see turn_start.rs:178-187 —
    // unconditional GainMantra). Fix will add a TriggerCondition guard or a
    // complex_hook that routes to ChangeStance on the threshold.
}

// -----------------------------------------------------------------------
// D176 — Scry onScry timing (Nirvana fires after discard, not before grid)
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ScryAction.java`
//   L37-40: for (AbstractPower p : player.powers) p.onScry();
//           (fires BEFORE the gridSelectScreen is opened)
//   L56-60: after player selects, selected cards move to discard;
//           then each discard-pile card's triggerOnScry() fires.
//
// Rust `packages/engine-rs/src/engine.rs:586-632` resolve_scry fires
// Nirvana block (L610-613) AFTER the discard split is already committed,
// and there is no generic `onScry` trigger/surface — every consumer
// (Nirvana block, Weave return) is hardcoded inline. A new `onScry`
// trigger (akin to Trigger::DamageResolved) would let custom powers
// attach here without further engine patches.
#[test]
#[ignore = "D176 open — Cycle 8+"]
fn d176_scry_has_on_scry_trigger_variant() {
    // The cleanest fix surfaces `Trigger::OnScry` on the trigger enum and
    // routes resolve_scry through it. Check the trigger variant exists.
    // Until the fix lands, the enum has no OnScry variant; the match
    // compiles only after the fix. We document the target here.
    //
    // When the fix lands, uncomment and use:
    //   use crate::effects::trigger::Trigger;
    //   let _ = Trigger::OnScry;
    panic!("D176: scry has no onScry trigger surface; Java ScryAction:37-40 fires p.onScry() pre-grid");
}

// -----------------------------------------------------------------------
// D177 — Electrodynamics evoke/passive uses NORMAL (through block)
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/actions/defect/LightningOrbPassiveAction.java`
//   L56: DamageAllEnemiesAction(..., DamageType.THORNS, ...)
//   (Thorns damage type bypasses block.)
//
// Rust `packages/engine-rs/src/engine.rs:2810-2852` routes the multi-
// enemy Electrodynamics branch through `deal_damage_to_enemy`, which
// subtracts block first. Result: an 8-damage Lightning passive against
// a 5-block enemy under Electrodynamics deals 3 in Rust, 8 in Java.
#[test]
#[ignore = "D177 open — Cycle 8+"]
fn d177_electrodynamics_bypasses_enemy_block() {
    use crate::orbs::OrbType;
    use crate::status_ids::sid;
    use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

    let enemies = vec![
        enemy_no_intent("JawWorm", 40, 40),
        enemy_no_intent("Cultist", 35, 35),
    ];
    let mut engine = engine_without_start(make_deck_n("Strike", 5), enemies, 3);
    // Both enemies walk in with 5 block (simulating a Defend/taskmaster stack).
    engine.state.enemies[0].entity.block = 5;
    engine.state.enemies[1].entity.block = 5;

    engine.init_defect_orbs(1);
    engine.state.player.set_status(sid::ELECTRODYNAMICS, 1);
    engine.channel_orb(OrbType::Lightning);
    engine.evoke_front_orb();

    // Java (THORNS damage type): 8 HP loss ignoring block on each enemy.
    assert_eq!(
        engine.state.enemies[0].entity.hp, 32,
        "Enemy 0 must lose 8 HP straight (Java Lightning.java:56 uses DamageType.THORNS)"
    );
    assert_eq!(
        engine.state.enemies[1].entity.hp, 27,
        "Enemy 1 must lose 8 HP straight"
    );
}

// -----------------------------------------------------------------------
// D178 — Distilled Chaos targets first living enemy, not random per card
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/potions/DistilledChaosPotion.java`
//   L36-41: for (i = 0; i < potency; ++i)
//             addToBot(new PlayTopCardAction(
//                 getMonsters().getRandomMonster(null, true, cardRandomRng),
//                 false));
//   Each top-card plays against a FRESH random monster using cardRandomRng.
//
// Rust `packages/engine-rs/src/potions/defs/distilled_chaos.rs:39-50`
//   let target_idx = living_enemy_indices().first()... — deterministic
//   first enemy every iteration. Against a 3-Louse or Sentry encounter,
//   the 3 cards all hit the first monster instead of spreading randomly.
//   This also burns a wrong amount of cardRandomRng draws (Java makes
//   one per card, Rust makes zero).
#[test]
#[ignore = "D178 open — Cycle 8+"]
fn d178_distilled_chaos_random_target_per_card() {
    // When fixed, the Rust hook must pull cardRandomRng once per card and
    // pick a living monster via that draw. Current hook targets
    // `living_enemy_indices().first()` every call.
    //
    // A proper test uses a seeded engine with 3 enemies and verifies at
    // least one card lands on a non-first monster for some seed.
    // Until the fix lands, the hook's first-enemy determinism is
    // observable and the target distribution collapses.
    panic!(
        "D178: DistilledChaos always hits first living enemy; Java picks getRandomMonster per card"
    );
}

// -----------------------------------------------------------------------
// D179 — Plated Armor decrement on HP_LOSS / THORNS paths
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/powers/PlatedArmorPower.java`
//   L54-59: wasHPLost: if info.owner != null && info.owner != this.owner
//                      && info.type != HP_LOSS && info.type != THORNS
//                      && damageAmount > 0 { reduce Plated by 1 }
//
// Rust `packages/engine-rs/src/combat_hooks.rs:186-191` decrements
//   Plated on every `hp_loss > 0` in the enemy-attack loop, with no
//   damage-type gate. A Spiker's THORNS retaliation (3 dmg) against the
//   player's attack strips a Plated stack in Rust but not Java; a Burn
//   card or Parasite tick (HP_LOSS) strips Plated in Rust but not Java.
#[test]
#[ignore = "D179 open — Cycle 8+"]
fn d179_plated_armor_not_decremented_by_thorns_or_hp_loss() {
    use crate::status_ids::sid;
    use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

    let enemies = vec![enemy_no_intent("JawWorm", 40, 40)];
    let mut engine = engine_without_start(make_deck_n("Strike", 5), enemies, 3);
    engine.state.player.set_status(sid::PLATED_ARMOR, 3);

    // Simulate an HP_LOSS tick (e.g. Parasite, Burn). Java wasHPLost skips
    // because info.type == HP_LOSS. Rust's current path decrements to 2.
    engine.player_lose_hp(3);
    assert_eq!(
        engine.state.player.status(sid::PLATED_ARMOR),
        3,
        "D179: Plated must NOT decrement on HP_LOSS ticks (Java PlatedArmorPower.java:55)"
    );
}

// -----------------------------------------------------------------------
// D180 — Wraith Form end-of-turn + stack-amount Dex decay
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/powers/WraithFormPower.java`
//   L33-35: atEndOfTurn(isPlayer):
//             addToBot(ApplyPower(DexterityPower, this.amount))
//
// Rust `packages/engine-rs/src/powers/defs/turn_start.rs:202-213` uses
//   Trigger::TurnStart with AmountSource::Fixed(-1). Two defects:
//     (a) Timing is start-of-turn, not end-of-turn — the Dex hit lands
//         before the player plays cards (Java applies it after).
//     (b) Amount is hardcoded -1 regardless of stack. Stacking two
//         WraithForms via Deva Form / card replay should lose 2 Dex per
//         end-of-turn; Rust only loses 1.
#[test]
fn d180_wraith_form_currently_pre_draw_fixed_neg_one() {
    use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
    use crate::effects::trigger::Trigger;
    use crate::powers::defs::DEF_WRAITH_FORM;
    use crate::status_ids::sid;

    // Current Rust wiring (to keep the bug ledger honest until fixed).
    assert_eq!(DEF_WRAITH_FORM.triggers.len(), 1);
    assert_eq!(
        DEF_WRAITH_FORM.triggers[0].trigger,
        Trigger::TurnStart,
        "D180 (a): Wraith Form currently fires pre-draw; Java atEndOfTurn"
    );
    let effects = DEF_WRAITH_FORM.triggers[0].effects;
    assert_eq!(effects.len(), 1);
    match &effects[0] {
        Effect::Simple(SimpleEffect::AddStatus(Target::Player, stat, AmountSource::Fixed(n))) => {
            assert_eq!(*stat, sid::DEXTERITY);
            assert_eq!(*n, -1, "D180 (b): currently Fixed(-1); Java uses stack amount");
        }
        other => panic!("unexpected effect shape: {other:?}"),
    }
}

#[test]
#[ignore = "D180 open — Cycle 8+"]
fn d180_wraith_form_java_is_end_of_turn_and_stack_scaled() {
    use crate::effects::declarative::{AmountSource, Effect, SimpleEffect};
    use crate::effects::trigger::Trigger;
    use crate::powers::defs::DEF_WRAITH_FORM;
    use crate::status_ids::sid;

    assert_eq!(
        DEF_WRAITH_FORM.triggers[0].trigger,
        Trigger::TurnEnd,
        "WraithFormPower.java:33 uses atEndOfTurn"
    );
    // When fixed, the Dex delta should scale with the WraithForm stack via
    // StatusValue (negated) rather than Fixed(-1).
    match &DEF_WRAITH_FORM.triggers[0].effects[0] {
        Effect::Simple(SimpleEffect::AddStatus(_, _, AmountSource::StatusValue(src))) => {
            assert_eq!(*src, sid::WRAITH_FORM);
        }
        other => panic!(
            "D180 target shape: AddStatus(Player, DEXTERITY, StatusValue(WRAITH_FORM) negated); got {other:?}"
        ),
    }
}

// -----------------------------------------------------------------------
// D181 — Council of Ghosts hardcodes max_hp(-5), Java loses 50% max HP
// -----------------------------------------------------------------------
//
// Java `decompiled/java-src/com/megacrit/cardcrawl/events/city/Ghosts.java`
//   L36-38: hpLoss = MathUtils.ceil(maxHealth * 0.5f);
//           if (hpLoss >= maxHealth) hpLoss = maxHealth - 1;
//   L62-98: Accept -> decreaseMaxHealth(hpLoss) + grant `amount` Apparitions
//           amount = 5; if ascensionLevel >= 15 amount -= 2;  (5 or 3)
//
// Rust `packages/engine-rs/src/events/city.rs:135-148` hardcodes the
// Accept branch to `max_hp(-5)` and `gain_card_reward(5)` regardless of
// player max HP or ascension. At Watcher A0 max_hp = 72, Java loses 36
// max HP; Rust loses 5 — a 7x divergence. At A15+, Rust still grants 5
// apparitions instead of 3.
#[test]
#[ignore = "D181 open — Cycle 8+"]
fn d181_council_of_ghosts_scales_with_max_hp_and_ascension() {
    use crate::events::{typed_events_for_act, EventProgramOp, TypedEventDef};

    let event: TypedEventDef = typed_events_for_act(2)
        .into_iter()
        .find(|e| e.name == "Council of Ghosts")
        .expect("Council of Ghosts event must exist in act 2");
    let accept = event
        .options
        .iter()
        .find(|opt| opt.text.starts_with("Accept"))
        .expect("Accept option missing");

    // The Java-correct op would read `max_hp(ceil(maxHealth * -0.5))`, which
    // is not expressible as a static EventProgramOp today. Expect a non-
    // static scaler to appear (or the hardcoded -5 to change) once the fix
    // lands. Here we assert the *current* value is NOT -5 (must be dynamic).
    let ops = &accept.program.ops;
    let max_hp_op = ops
        .iter()
        .find(|op| matches!(op, EventProgramOp::AdjustMaxHp { .. }))
        .expect("AdjustMaxHp op expected on Accept branch");
    if let EventProgramOp::AdjustMaxHp { amount } = max_hp_op {
        assert_ne!(
            *amount, -5,
            "D181: Council of Ghosts currently hardcodes AdjustMaxHp(-5); Java scales to ceil(maxHp * 0.5)"
        );
    }
    // Apparition count must also drop from 5 to 3 at A15+; current program
    // unconditionally gains 5 card rewards. Once fixed, a conditional
    // amount should replace the static 5.
}
