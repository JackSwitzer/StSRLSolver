//! Enemy turn logic — enemy moves, boss damage hooks.
//!
//! Extracted from engine.rs as a pure refactor.

use crate::combat_types::mfx;
use crate::damage;
use crate::enemies;
use crate::engine::{CombatEngine, CombatPhase};
use crate::potions;
use crate::powers;
use crate::state::Stance;
use crate::status_ids::sid;
use smallvec::SmallVec;

/// Execute all enemy turns: poison ticks, ritual, moves.
pub fn do_enemy_turns(engine: &mut CombatEngine) {
    engine.phase = CombatPhase::EnemyTurn;

    // Sources: MonsterStartTurnAction.java and MonsterGroup.java
    // (`applyPreTurnLogic`). Clear every monster's block once before any
    // monster acts; block granted later in this queue must remain intact.
    for enemy in &mut engine.state.enemies {
        enemy.entity.block = 0;
    }

    let num_enemies = engine.state.enemies.len();
    for i in 0..num_enemies {
        if !engine.state.enemies[i].is_alive() {
            continue;
        }

        // Reset Invincible per-turn damage tracker
        powers::reset_invincible_damage_taken(&mut engine.state.enemies[i].entity);

        // FlightPower.atStartOfTurn restores its constructor amount as long as
        // the power still exists. Byrd's BLOCK_AMT stores that amount (3/4).
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FlightPower.java
        if engine.state.enemies[i].id == "Byrd"
            && engine.state.enemies[i].entity.status(sid::FLIGHT) > 0
        {
            let stored = engine.state.enemies[i].entity.status(sid::BLOCK_AMT);
            engine.state.enemies[i].entity.set_status(sid::FLIGHT, stored);
        }

        let is_first = engine.state.enemies[i].first_turn;
        engine.emit_event(crate::effects::runtime::GameEvent {
            kind: crate::effects::trigger::Trigger::EnemyTurnStart,
            card_type: None,
            card_inst: None,
            is_first_turn: is_first,
            target_idx: i as i32,
            enemy_idx: i as i32,
            potion_slot: -1,
            status_id: None,
            amount: 0,
            replay_window: false,
        });

        // ChokePower.atStartOfTurn removes the whole power. Vault never enters
        // this enemy-turn flow, so a skipped turn correctly leaves Choke armed.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ChokePower.java
        engine.state.enemies[i].entity.set_status(sid::CONSTRICTED, 0);

        // Enemy status countdowns still modeled directly in the turn-flow loop.
        let fading = engine.state.enemies[i].entity.status(sid::FADING);
        if fading > 0 {
            engine.state.enemies[i]
                .entity
                .set_status(sid::FADING, fading - 1);
            if engine.state.enemies[i].entity.status(sid::FADING) <= 0 {
                engine.state.enemies[i].entity.hp = 0;
                continue;
            }
        }

        let bomb = engine.state.enemies[i].entity.status(sid::THE_BOMB);
        if bomb > 0 {
            engine.state.enemies[i]
                .entity
                .set_status(sid::THE_BOMB, bomb - 1);
            if engine.state.enemies[i].entity.status(sid::THE_BOMB) <= 0 {
                let intangible = engine.state.player.status(sid::INTANGIBLE) > 0;
                let has_tungsten = engine.state.has_relic("Tungsten Rod");
                let hp_loss = damage::apply_hp_loss(40, intangible, has_tungsten);
                engine.player_lose_hp(hp_loss);
                if engine.state.combat_over {
                    return;
                }
            }
        }

        if engine.state.enemies[i].entity.hp <= 0
            && engine.state.enemies[i].entity.status(sid::REBIRTH_PENDING) <= 0
        {
            engine.state.enemies[i].entity.hp = 0;
            continue;
        }

        // Poison tick — kept inline (complex death check + boss hooks)
        let poison_dmg = powers::tick_poison(&mut engine.state.enemies[i].entity);
        if poison_dmg > 0 {
            engine.state.total_damage_dealt += poison_dmg;
            engine.record_enemy_hp_damage(i, poison_dmg);
            if engine.state.enemies[i].entity.is_dead() {
                engine.state.enemies[i].entity.hp = 0;
                continue;
            }
        }

        // Ritual strength already applied inside hook (skipped on first turn)

        // Execute enemy move
        execute_enemy_move(engine, i);

        // Sources: reference/extracted/methods/monster/Exploder.java and
        // decompiled/java-src/com/megacrit/cardcrawl/powers/ExplosivePower.java.
        // GameActionManager calls
        // applyTurnPowers after takeTurn, so Explosive ticks after Exploder's
        // queued RollMoveAction. At 1 it queues SuicideAction before the
        // source monster's 30 DamageInfo.THORNS hit to the player.
        if engine.state.enemies[i].id == "Exploder" {
            let explosive = engine.state.enemies[i].entity.status(sid::EXPLOSIVE);
            if explosive > 1 {
                engine.state.enemies[i]
                    .entity
                    .set_status(sid::EXPLOSIVE, explosive - 1);
            } else if explosive == 1 {
                engine.state.enemies[i].entity.set_status(sid::EXPLOSIVE, 0);
                if engine.state.enemies[i].entity.hp > 0 {
                    engine.state.enemies[i].entity.hp = 0;
                    engine.finalize_enemy_death(i);
                }
                engine.deal_thorns_damage_to_player(30);
            }
        }

        // Check player death
        if engine.state.player.is_dead() {
            engine.state.player.hp = 0;
            engine.state.combat_over = true;
            engine.state.player_won = false;
            engine.phase = CombatPhase::CombatOver;
            return;
        }

        // Mark first turn complete
        engine.state.enemies[i].first_turn = false;
    }

    // MonsterGroup.applyEndOfTurnPowers runs only after every queued monster
    // has acted. RegenerateMonsterPower heals here, not at enemy-turn start.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RegenerateMonsterPower.java
    for i in 0..num_enemies {
        if engine.state.enemies[i].is_alive() {
            engine.emit_event(crate::effects::runtime::GameEvent {
                kind: crate::effects::trigger::Trigger::EnemyTurnEnd,
                card_type: None,
                card_inst: None,
                is_first_turn: false,
                target_idx: i as i32,
                enemy_idx: i as i32,
                potion_slot: -1,
                status_id: None,
                amount: 0,
                replay_window: false,
            });
        }
    }
}

/// Execute a single enemy's move (attack, block, status effects).
fn execute_enemy_move(engine: &mut CombatEngine, enemy_idx: usize) {
    // Awakened One rebirth: if pending, execute the rebirth this turn instead of normal move
    if engine.state.enemies[enemy_idx].entity.status(sid::REBIRTH_PENDING) > 0 {
        if matches!(engine.state.enemies[enemy_idx].id.as_str(),
            "AwakenedOne" | "Awakened One")
        {
            engine.state.enemies[enemy_idx].entity.set_status(sid::REBIRTH_PENDING, 0);
            enemies::awakened_one_rebirth(&mut engine.state.enemies[enemy_idx]);
            // Rebirth's takeTurn still ends with RollMoveAction. The phase-two
            // first-turn branch ignores the rolled value but consumes one aiRng
            // tick before setting Dark Echo.
            // Java: reference/extracted/methods/monster/AwakenedOne.java
            enemies::roll_initial_move(&mut engine.state.enemies[enemy_idx], &mut engine.ai_rng);
            return;
        }

        if engine.state.enemies[enemy_idx].id == "Darkling"
            && engine.state.enemies[enemy_idx].move_id
                == enemies::move_ids::DARK_REINCARNATE
        {
            // Source: reference/extracted/methods/monster/Darkling.java
            // (`takeTurn`, case 5). Heal from zero to half max HP, leave the
            // half-dead state, reinstall Regrow, fire onSpawnMonster relics,
            // then consume the queued RollMoveAction.
            let heal = engine.state.enemies[enemy_idx].entity.max_hp / 2;
            engine.state.enemies[enemy_idx].entity.hp = heal;
            engine.state.enemies[enemy_idx].entity.set_status(sid::REBIRTH_PENDING, 0);
            engine.state.enemies[enemy_idx].entity.set_status(sid::REGROW, 1);
            if engine.state.has_relic("Philosopher's Stone")
                || engine.state.has_relic("PhilosopherStone")
            {
                engine.state.enemies[enemy_idx].entity.add_status(sid::STRENGTH, 1);
            }
            enemies::roll_next_move(
                &mut engine.state.enemies[enemy_idx], &mut engine.ai_rng);
            return;
        }
    }

    if engine.state.enemies[enemy_idx].move_id == -1 {
        return;
    }

    let player_hp_before_move = engine.state.player.hp;

    if engine.state.enemies[enemy_idx].id == "Exploder" {
        // Source: reference/extracted/methods/monster/Exploder.java
        // (`takeTurn`): turnCount increments before the queued move resolves.
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::TURN_COUNT, 1);
    }

    if engine.state.enemies[enemy_idx].id == "Maw"
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::MAW_ROAR
    {
        // Source: reference/extracted/methods/monster/Maw.java (`takeTurn`,
        // case ROAR). `roared` changes before the queued RollMoveAction.
        engine.state.enemies[enemy_idx].entity.set_status(sid::FIRST_MOVE, 1);
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "GremlinLeader" | "Gremlin Leader")
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::GL_ENCOURAGE
    {
        // Source: reference/extracted/methods/monster/GremlinLeader.java
        // (`getEncourageQuote` is visible in the full source).
        engine.ai_rng.random_range(0, 2);
    }

    if engine.state.enemies[enemy_idx].id == "TheGuardian"
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::GUARD_TWIN_SLAM
    {
        // Source: TheGuardian.java `useTwinSmash`: Offensive Mode resolves
        // before the two attacks, so Thorns can count toward the new threshold.
        enemies::act1::guardian_begin_twin_smash(&mut engine.state.enemies[enemy_idx]);
    }

    // Sources: reference/extracted/methods/monster/{Looter,Mugger}.java plus
    // DamageAction.java (`stealGold`): both attack variants steal before their
    // damage resolves, even when block prevents HP loss.
    let thief_attack = match engine.state.enemies[enemy_idx].id.as_str() {
        "Looter" => matches!(engine.state.enemies[enemy_idx].move_id,
            enemies::move_ids::LOOTER_MUG | enemies::move_ids::LOOTER_LUNGE),
        "Mugger" => matches!(engine.state.enemies[enemy_idx].move_id,
            enemies::move_ids::MUGGER_MUG | enemies::move_ids::MUGGER_BIG_SWIPE),
        _ => false,
    };
    if thief_attack
    {
        let amount = engine.state.enemies[enemy_idx].entity.status(sid::TURN_COUNT);
        let stolen = amount.min(engine.state.run_gold);
        engine.state.run_gold -= stolen;
        engine.state.enemies[enemy_idx].entity.add_status(sid::COUNT, stolen);
    }

    // Attack
    let enemy = &engine.state.enemies[enemy_idx];
    let move_dmg = enemy.move_damage();
    if move_dmg > 0 {
        let enemy_strength = enemy.entity.strength();
        let enemy_weak = enemy.entity.is_weak();
        let base_damage = move_dmg + enemy_strength;

        // Apply Weak to enemy's attack (Paper Crane: 0.60 instead of 0.75)
        let mut damage_f = base_damage as f64;
        if enemy_weak {
            let weak_mult = if engine.state.has_relic("Paper Crane") {
                damage::WEAK_MULT_PAPER_CRANE
            } else {
                damage::WEAK_MULT
            };
            damage_f *= weak_mult;
        }

        // Floor the per-hit base (before stance/vuln/intangible)
        let per_hit_base = (damage_f as i32).max(0);

        let is_wrath = engine.state.stance == Stance::Wrath;
        let player_vuln = engine.state.player.is_vulnerable();
        let player_intangible = engine.state.player.status(sid::INTANGIBLE) > 0;
        let has_torii = engine.state.has_relic("Torii");
        let has_tungsten = engine.state.has_relic("Tungsten Rod");
        let has_odd_mushroom = engine.state.has_relic("Odd Mushroom");

        let hits = enemy.move_hits();
        for _ in 0..hits {
            // AbstractPlayer.damage applies Intangible and block before
            // BufferPower.onAttackedToChangeDamage. Buffer therefore consumes
            // a stack only when positive damage remains after block. Torii's
            // onAttacked and Tungsten Rod's onLoseHpLast run afterward.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BufferPower.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Torii.java
            // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/TungstenRod.java
            let result_before_buffer = damage::calculate_incoming_damage(
                per_hit_base,
                engine.state.player.block,
                is_wrath,
                player_vuln,
                player_intangible,
                false,
                false,
                has_odd_mushroom,
            );

            engine.state.player.block = result_before_buffer.block_remaining;
            let mut hp_loss = result_before_buffer.hp_loss;
            let mut static_discharge = 0;
            if hp_loss > 0 {
                let buffer = engine.state.player.status(sid::BUFFER);
                if buffer > 0 {
                    engine.state.player.set_status(sid::BUFFER, buffer - 1);
                    hp_loss = 0;
                } else {
                    // StaticDischargePower.onAttacked observes damage after
                    // block and Buffer, but before Torii.onAttacked and
                    // TungstenRod.onLoseHpLast. Snapshot the trigger here;
                    // channeling remains queued until this hit resolves.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/StaticDischargePower.java
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
                    static_discharge = engine.state.player.status(sid::STATIC_DISCHARGE);
                    if has_torii && (2..=5).contains(&hp_loss) {
                        hp_loss = 1;
                    }
                    if has_tungsten {
                        hp_loss = (hp_loss - 1).max(0);
                    }
                }
            }

            if hp_loss > 0 {
                // PainfulStabsPower.onInflictDamage queues one Wound per
                // unblocked, non-THORNS hit. Enemy move damage here is NORMAL.
                // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/PainfulStabsPower.java
                if engine.state.enemies[enemy_idx]
                    .entity
                    .status(sid::PAINFUL_STABS) > 0
                {
                    engine.state.discard_pile.push(
                        engine.card_registry.make_card("Wound"));
                }
                engine.player_lose_hp(hp_loss);
                if engine.state.combat_over {
                    return;
                }

                // Plated Armor decrements on unblocked HP damage from enemy attacks.
                let plated = engine.state.player.status(sid::PLATED_ARMOR);
                if plated > 0 {
                    let new_plated = plated - 1;
                    engine.state.player.set_status(sid::PLATED_ARMOR, new_plated);
                }

            }

            for _ in 0..static_discharge {
                let focus = engine.state.player.focus();
                let evoke_effect = engine.state.orb_slots.channel(
                    crate::orbs::OrbType::Lightning,
                    focus,
                );
                match evoke_effect {
                    crate::orbs::EvokeEffect::LightningDamage(dmg) => {
                        let living = engine.state.living_enemy_indices();
                        if let Some(&target) = living.first() {
                            let e = &mut engine.state.enemies[target];
                            let blocked_e = e.entity.block.min(dmg);
                            let hp_dmg_e = dmg - blocked_e;
                            e.entity.block -= blocked_e;
                            e.entity.hp -= hp_dmg_e;
                            engine.state.total_damage_dealt += hp_dmg_e;
                            if hp_dmg_e > 0 {
                                engine.record_enemy_hp_damage(target, hp_dmg_e);
                            }
                        }
                    }
                    crate::orbs::EvokeEffect::FrostBlock(blk) => {
                        engine.gain_block_player(blk);
                    }
                    _ => {}
                }
            }

            if engine.state.player.hp <= 0 {
                // Check Fairy in a Bottle
                let revive_hp = potions::check_fairy_revive(&engine.state);
                if revive_hp > 0 {
                    potions::consume_fairy(&mut engine.state);
                    engine.state.player.hp = revive_hp;
                } else {
                    engine.state.player.hp = 0;
                }
            }

            if engine.state.player.is_dead() {
                return;
            }

            // BufferPower.onAttackedToChangeDamage can reduce the hit to zero,
            // but ThornsPower.onAttacked runs afterward and still retaliates.
            // Java: AbstractPlayer.damage and ThornsPower.onAttacked.
            let thorns = engine.state.player.status(sid::THORNS);
            if thorns > 0 && engine.state.enemies[enemy_idx].is_alive() {
                engine.deal_thorns_damage_to_enemy(enemy_idx, thorns);
            }

            // FlameBarrierPower.onAttacked retaliates once per sourced hit,
            // including hits fully absorbed by Block; enemy attacks here are
            // NORMAL, so the THORNS/HP_LOSS exclusions are already satisfied.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FlameBarrierPower.java
            let flame_barrier = engine.state.player.status(sid::FLAME_BARRIER);
            if flame_barrier > 0 && engine.state.enemies[enemy_idx].is_alive() {
                engine.deal_thorns_damage_to_enemy(enemy_idx, flame_barrier);
            }
        }
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "Shelled Parasite" | "ShelledParasite")
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::SP_LIFE_SUCK
        && engine.state.enemies[enemy_idx].is_alive()
    {
        // Source: VampireDamageAction heals the source for the target's
        // lastDamageTaken, not the attack's printed base damage.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/
        // VampireDamageAction.java.
        let actual_hp_loss = (player_hp_before_move - engine.state.player.hp).max(0);
        let enemy = &mut engine.state.enemies[enemy_idx].entity;
        enemy.hp = (enemy.hp + actual_hp_loss).min(enemy.max_hp);
    }

    // Block
    let move_blk = engine.state.enemies[enemy_idx].move_block();
    if move_blk > 0 {
        if matches!(engine.state.enemies[enemy_idx].id.as_str(), "BronzeOrb" | "Bronze Orb")
            && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::BO_SUPPORT
        {
            // BronzeOrb SUPPORT_BEAM targets the boss, not the Orb itself.
            // Java: reference/extracted/methods/monster/BronzeOrb.java
            if let Some(automaton) = engine.state.enemies.iter_mut().find(|enemy|
                matches!(enemy.id.as_str(), "BronzeAutomaton" | "Bronze Automaton")
                    && enemy.is_alive())
            {
                automaton.entity.block += move_blk;
            }
        } else {
            engine.state.enemies[enemy_idx].entity.block += move_blk;
        }
    }

    // Apply move effects
    let effects: SmallVec<[(u8, i16); 4]> = engine.state.enemies[enemy_idx].move_effects.clone();
    let champ_anger = matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "Champ" | "TheChamp")
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::CHAMP_ANGER;

    if engine.state.enemies[enemy_idx].id == "Byrd"
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::BYRD_FLY_UP
    {
        // Byrd.takeTurn case FLY_UP applies a fresh FlightPower before its
        // queued RollMoveAction selects the next airborne intent.
        // Java: reference/extracted/methods/monster/Byrd.java.
        let flight = engine.state.enemies[enemy_idx].entity.status(sid::BLOCK_AMT);
        engine.state.enemies[enemy_idx].entity.set_status(sid::FLIGHT, flight);
    }

    fn get_fx(effects: &SmallVec<[(u8, i16); 4]>, id: u8) -> Option<i16> {
        effects.iter().find(|e| e.0 == id).map(|e| e.1)
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "CorruptHeart" | "Corrupt Heart")
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::HEART_BUFF
    {
        // Source: reference/extracted/methods/monster/CorruptHeart.java
        // (`takeTurn`, case 4). Each buff first cancels any negative Strength,
        // then grants 2 Strength and applies the current escalation stage.
        let strength = engine.state.enemies[enemy_idx].entity.strength();
        if strength < 0 {
            engine.state.enemies[enemy_idx].entity.set_status(sid::STRENGTH, 0);
        }
        engine.state.enemies[enemy_idx].entity.add_status(sid::STRENGTH, 2);
        let buff_count = engine.state.enemies[enemy_idx].entity.status(sid::BUFF_COUNT);
        match buff_count {
            0 => engine.state.enemies[enemy_idx].entity.add_status(sid::ARTIFACT, 2),
            1 => engine.state.enemies[enemy_idx].entity.add_status(sid::BEAT_OF_DEATH, 1),
            2 => engine.state.enemies[enemy_idx].entity.set_status(sid::PAINFUL_STABS, 1),
            3 => engine.state.enemies[enemy_idx].entity.add_status(sid::STRENGTH, 10),
            _ => engine.state.enemies[enemy_idx].entity.add_status(sid::STRENGTH, 50),
        }
        engine.state.enemies[enemy_idx]
            .entity.set_status(sid::BUFF_COUNT, buff_count + 1);
    }

    // D59: enemy-applied debuffs use `apply_debuff_from_enemy` so Java's
    // `justApplied=true` semantics kick in and the first end-of-round
    // decrement is skipped. Without this, 1-stack Weak/Vuln/Frail vanishes
    // the same turn it lands -- radically under-models enemy debuff pressure
    // (Sentry beam, Boot Steel Pads, Sphere Slam, Time Eater Ripple).
    if let Some(amt) = get_fx(&effects, mfx::WEAK) {
        powers::apply_debuff_from_enemy(&mut engine.state.player, sid::WEAKENED, amt as i32);
    }
    if let Some(amt) = get_fx(&effects, mfx::VULNERABLE) {
        powers::apply_debuff_from_enemy(&mut engine.state.player, sid::VULNERABLE, amt as i32);
    }
    if let Some(amt) = get_fx(&effects, mfx::FRAIL) {
        powers::apply_debuff_from_enemy(&mut engine.state.player, sid::FRAIL, amt as i32);
    }
    if get_fx(&effects, mfx::HEART_STATUS_CARDS).unwrap_or(0) > 0 {
        // Source: reference/extracted/methods/monster/CorruptHeart.java
        // (`takeTurn`, case 3). Each MakeTempCardInDrawPileAction uses a
        // random spot, in this exact action order.
        for id in ["Dazed", "Slimed", "Wound", "Burn", "Void"] {
            let card = engine.card_registry.make_card(id);
            if engine.state.draw_pile.is_empty() {
                engine.state.draw_pile.push(card);
            } else {
                let idx = engine.card_random_rng.random_range(
                    0,
                    (engine.state.draw_pile.len() - 1) as i32,
                ) as usize;
                engine.state.draw_pile.insert(idx, card);
            }
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::STRENGTH).filter(|_| !champ_anger) {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::STRENGTH, amt as i32);
    }
    if let Some(amt) = get_fx(&effects, mfx::METALLICIZE) {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::METALLICIZE, amt as i32);
        // MetallicizePower.atEndOfTurnPreEndTurnCards fires later in this same
        // monster round. Enemy Metallicize's persistent runtime trigger is
        // modeled at the following enemy-turn start (equivalent after block
        // clear), so realize this one installation-round proc here.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/MetallicizePower.java
        engine.state.enemies[enemy_idx].entity.block += amt as i32;
        // The owner-aware runtime is built from installed powers; refresh it
        // so the new power participates in subsequent rounds.
        engine.rebuild_effect_runtime();
    }
    if let Some(amt) = get_fx(&effects, mfx::RITUAL) {
        engine.state.enemies[enemy_idx]
            .entity
            .set_status(sid::RITUAL, amt as i32);
    }
    if let Some(amt) = get_fx(&effects, mfx::ENRAGE) {
        engine.state.enemies[enemy_idx]
            .entity.set_status(sid::ENRAGE, amt as i32);
    }
    if let Some(amt) = get_fx(&effects, mfx::SHARP_HIDE) {
        engine.state.enemies[enemy_idx]
            .entity.set_status(sid::SHARP_HIDE, amt as i32);
    }
    if let Some(amt) = get_fx(&effects, mfx::ENTANGLE) {
        if amt > 0 {
            engine.state.player.set_status(sid::ENTANGLED, 1);
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::SLIMED) {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Slimed"));
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::DAZE) {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Dazed"));
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::DAZE_DRAW) {
        // Source: reference/extracted/methods/monster/Repulsor.java and
        // decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java.
        // MakeTempCardInDrawPileAction(..., randomSpot=true) inserts each
        // Dazed separately and consumes cardRandomRng whenever the pile is
        // already nonempty.
        for _ in 0..amt {
            let card = engine.card_registry.make_card("Dazed");
            if engine.state.draw_pile.is_empty() {
                engine.state.draw_pile.push(card);
            } else {
                let idx = engine.card_random_rng.random_range(
                    0,
                    (engine.state.draw_pile.len() - 1) as i32,
                ) as usize;
                engine.state.draw_pile.insert(idx, card);
            }
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::BURN) {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Burn"));
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::BURN_DRAW_DISCARD) {
        // Source: reference/extracted/methods/monster/OrbWalker.java and
        // MakeTempCardInDiscardAndDeckAction.java. Each copy goes to a random
        // draw-pile position and to discard, in that order.
        for _ in 0..amt {
            let draw_burn = engine.card_registry.make_card("Burn");
            if engine.state.draw_pile.is_empty() {
                engine.state.draw_pile.push(draw_burn);
            } else {
                let idx = engine.card_random_rng.random(
                    engine.state.draw_pile.len() as i32 - 1) as usize;
                engine.state.draw_pile.insert(idx, draw_burn);
            }
            engine.state.discard_pile.push(engine.card_registry.make_card("Burn"));
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::BURN_PLUS) {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Burn+"));
        }
    }
    // Lagavulin Siphon Soul: reduce player Strength and Dexterity
    if let Some(amt) = get_fx(&effects, mfx::SIPHON_STR) {
        engine.state.player.add_status(sid::STRENGTH, -(amt as i32));
    }
    if let Some(amt) = get_fx(&effects, mfx::SIPHON_DEX) {
        engine.state.player.add_status(sid::DEXTERITY, -(amt as i32));
    }

    // Champ Anger / Time Eater Haste: remove ALL debuffs from this enemy
    if get_fx(&effects, mfx::REMOVE_DEBUFFS).unwrap_or(0) > 0 {
        let statuses = &mut engine.state.enemies[enemy_idx].entity.statuses;
        for i in 0..256 {
            if statuses[i] != 0 {
                let sid = crate::ids::StatusId(i as u16);
                if crate::powers::registry::status_is_debuff(sid) {
                    statuses[i] = 0;
                }
            }
        }
        // Negative StrengthPower and GainStrengthPower ("Shackled") are both
        // DEBUFF powers. Champ explicitly removes Shackled before applying its
        // phase-two Strength, so the gain must resolve after this cleanup.
        // Java: reference/extracted/methods/monster/Champ.java (`takeTurn`).
        if engine.state.enemies[enemy_idx].entity.strength() < 0 {
            engine.state.enemies[enemy_idx].entity.set_status(sid::STRENGTH, 0);
        }
        engine.state.enemies[enemy_idx]
            .entity.set_status(sid::TEMP_STRENGTH_LOSS, 0);
        if champ_anger {
            if let Some(amt) = get_fx(&effects, mfx::STRENGTH) {
                engine.state.enemies[enemy_idx]
                    .entity.add_status(sid::STRENGTH, amt as i32);
            }
        }
    }

    // Time Eater Haste: heal to half max HP
    if get_fx(&effects, mfx::HEAL_TO_HALF).unwrap_or(0) > 0 {
        let half = engine.state.enemies[enemy_idx].entity.max_hp / 2;
        engine.state.enemies[enemy_idx].entity.hp = half;
    }

    // Heal full (Awakened One rebirth, etc.)
    if get_fx(&effects, mfx::HEAL_FULL).unwrap_or(0) > 0 {
        engine.state.enemies[enemy_idx].entity.hp =
            engine.state.enemies[enemy_idx].entity.max_hp;
    }

    // Artifact: give enemy Artifact stacks
    if let Some(amt) = get_fx(&effects, mfx::ARTIFACT) {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::ARTIFACT, amt as i32);
    }

    // Source: BurnIncreaseAction.java. Upgrade every Burn in draw/discard,
    // then add three upgraded Burns to discard. Future Sear cards are upgraded.
    if get_fx(&effects, mfx::BURN_UPGRADE).unwrap_or(0) > 0 {
        let burn_id = engine.card_registry.make_card("Burn").def_id;
        let burn_plus = engine.card_registry.make_card("Burn+");
        for card in &mut engine.state.draw_pile {
            if card.def_id == burn_id { *card = burn_plus; }
        }
        for card in &mut engine.state.discard_pile {
            if card.def_id == burn_id { *card = burn_plus; }
        }
        for _ in 0..3 {
            engine.state.discard_pile.push(burn_plus);
        }
        engine.state.enemies[enemy_idx].entity.set_status(sid::BUFF_COUNT, 1);
    }

    // Confused: apply Confusion to player
    if get_fx(&effects, mfx::CONFUSED).unwrap_or(0) > 0 {
        engine.state.player.set_status(sid::CONFUSION, 1);
    }

    // Constrict: apply Constricted to player
    if let Some(amt) = get_fx(&effects, mfx::CONSTRICT) {
        // Source: SpireGrowth.takeTurn uses ApplyPowerAction, so Artifact can
        // block the debuff before ConstrictedPower is installed.
        powers::apply_debuff_from_enemy(
            &mut engine.state.player, sid::CONSTRICTED, amt as i32);
    }

    // Dexterity down: reduce player Dexterity
    if let Some(amt) = get_fx(&effects, mfx::DEX_DOWN) {
        engine.state.player.add_status(sid::DEXTERITY, -(amt as i32));
    }

    // Draw Reduction: reduce player draw next turn
    if let Some(amt) = get_fx(&effects, mfx::DRAW_REDUCTION) {
        engine.state.player.add_status(sid::DRAW_REDUCTION, amt as i32);
    }

    // Hex: apply Hex to player
    if let Some(amt) = get_fx(&effects, mfx::HEX) {
        engine.state.player.set_status(sid::HEX, amt as i32);
    }

    // Painful Stabs: add Wound cards to player discard
    if let Some(amt) = get_fx(&effects, mfx::PAINFUL_STABS) {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Wound"));
        }
    }

    // ApplyStasisAction uses draw if nonempty, otherwise discard. It tries
    // Rare, Uncommon, Common, then any card, and getRandomCard consumes one
    // cardRandomRng tick among the chosen candidates. The card remains held by
    // StasisPower until this Orb dies.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ApplyStasisAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    if get_fx(&effects, mfx::STASIS).unwrap_or(0) > 0 {
        let use_draw = !engine.state.draw_pile.is_empty();
        let zone = if use_draw {
            &engine.state.draw_pile
        } else {
            &engine.state.discard_pile
        };
        if !zone.is_empty() {
            let preferred_rank = [3_u8, 2, 1]
                .into_iter()
                .find(|rank| zone.iter().any(|card|
                    crate::run::stasis_card_rarity_rank(
                        engine.card_registry.card_def_by_id(card.def_id).id) == *rank));
            let mut candidates: Vec<usize> = zone.iter().enumerate()
                .filter(|(_, card)| preferred_rank.is_none_or(|rank|
                    crate::run::stasis_card_rarity_rank(
                        engine.card_registry.card_def_by_id(card.def_id).id) == rank))
                .map(|(idx, _)| idx)
                .collect();
            if preferred_rank.is_some() {
                candidates.sort_by_key(|idx|
                    engine.card_registry.card_def_by_id(zone[*idx].def_id).id);
            }
            let pick = engine.card_random_rng.random(candidates.len() as i32 - 1) as usize;
            let zone_idx = candidates[pick];
            let card = if use_draw {
                engine.state.draw_pile.remove(zone_idx)
            } else {
                engine.state.discard_pile.remove(zone_idx)
            };
            engine.state.enemies[enemy_idx].stasis_card = Some(card);
        }
    }

    // Strength bonus: give enemy Strength
    if let Some(amt) = get_fx(&effects, mfx::STRENGTH_BONUS) {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::STRENGTH, amt as i32);
    }

    // Strength down: reduce player Strength
    if let Some(amt) = get_fx(&effects, mfx::STRENGTH_DOWN) {
        engine.state.player.add_status(sid::STRENGTH, -(amt as i32));
    }

    // Thorns: give enemy Thorns
    if let Some(amt) = get_fx(&effects, mfx::THORNS) {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::THORNS, amt as i32);
    }

    // Void: add Void card to player draw pile
    if let Some(amt) = get_fx(&effects, mfx::VOID) {
        for _ in 0..amt {
            let card = engine.card_registry.make_card("Void");
            if engine.state.draw_pile.is_empty() {
                engine.state.draw_pile.push(card);
            } else {
                // MakeTempCardInDrawPileAction(..., randomSpot=true) delegates
                // to CardGroup.addToRandomSpot and consumes cardRandomRng.
                // Java: reference/extracted/methods/monster/AwakenedOne.java
                // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
                let idx = engine.card_random_rng.random_range(
                    0,
                    (engine.state.draw_pile.len() - 1) as i32,
                ) as usize;
                engine.state.draw_pile.insert(idx, card);
            }
        }
    }

    // Wound: add Wound cards to player discard
    if let Some(amt) = get_fx(&effects, mfx::WOUND) {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Wound"));
        }
    }

    // Beat of Death: set Beat of Death power on enemy
    if let Some(amt) = get_fx(&effects, mfx::BEAT_OF_DEATH) {
        engine.state.enemies[enemy_idx]
            .entity
            .set_status(sid::BEAT_OF_DEATH, amt as i32);
    }

    // Cross-enemy effects (Mystic Heal, GremlinLeader Encourage)
    if let Some(amt) = get_fx(&effects, mfx::BLOCK_ALL_ALLIES) {
        for j in 0..engine.state.enemies.len() {
            if j != enemy_idx && engine.state.enemies[j].is_alive() {
                engine.state.enemies[j].entity.block += amt as i32;
            }
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::PLATED_ARMOR_ALL) {
        // Source: reference/extracted/methods/monster/Deca.java (`takeTurn`,
        // case 2). A19 Square applies Plated Armor to every monster, including
        // Deca, and repeated Squares stack it.
        for enemy in engine.state.enemies.iter_mut().filter(|enemy| enemy.is_alive()) {
            enemy.entity.add_status(sid::PLATED_ARMOR, amt as i32);
        }
        engine.rebuild_effect_runtime();
    }
    if let Some(amt) = get_fx(&effects, mfx::HEAL_LOWEST_ALLY) {
        let mut lowest_idx: Option<usize> = None;
        let mut lowest_hp = i32::MAX;
        for j in 0..engine.state.enemies.len() {
            if j != enemy_idx && engine.state.enemies[j].is_alive()
                && engine.state.enemies[j].entity.hp < lowest_hp
            {
                lowest_idx = Some(j);
                lowest_hp = engine.state.enemies[j].entity.hp;
            }
        }
        if let Some(idx) = lowest_idx {
            let e = &mut engine.state.enemies[idx].entity;
            e.hp = (e.hp + amt as i32).min(e.max_hp);
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::HEAL_ALL) {
        // Source: reference/extracted/methods/monster/Healer.java (`takeTurn`,
        // case HEAL): HealAction targets every living, non-escaping monster,
        // including the Healer itself.
        for enemy in engine.state.enemies.iter_mut().filter(|enemy| enemy.is_alive()) {
            enemy.entity.hp = (enemy.entity.hp + amt as i32).min(enemy.entity.max_hp);
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::STRENGTH_ALL_ALLIES) {
        for j in 0..engine.state.enemies.len() {
            if j != enemy_idx && engine.state.enemies[j].is_alive() {
                engine.state.enemies[j].entity.add_status(sid::STRENGTH, amt as i32);
            }
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::BLOCK_RANDOM_OTHER) {
        // Source: decompiled GainBlockRandomMonsterAction.java. Exclude the
        // source and escaping/dying monsters; use self without RNG if empty.
        let valid: Vec<usize> = engine.state.enemies.iter().enumerate()
            .filter(|(idx, enemy)| *idx != enemy_idx && enemy.is_alive()
                && enemy.move_id != enemies::move_ids::GREMLIN_ESCAPE)
            .map(|(idx, _)| idx)
            .collect();
        let target = if valid.is_empty() {
            enemy_idx
        } else {
            valid[engine.ai_rng.random(valid.len() as i32 - 1) as usize]
        };
        engine.state.enemies[target].entity.block += amt as i32;
    }

    let large_slime_split = matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "AcidSlime_L" | "SpikeSlime_L")
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::AS_SPLIT;

    // Spawn minions for boss spawn moves
    {
        use crate::enemies::move_ids;
        let eid = engine.state.enemies[enemy_idx].id.as_str();
        let mid = engine.state.enemies[enemy_idx].move_id;
        match (eid, mid) {
            ("TheCollector" | "Collector", x) if x == move_ids::COLL_SPAWN => {
                for _ in 0..2 {
                    engine.add_spawned_enemy(enemies::create_enemy("TorchHead", 6, 6));
                }
            }
            ("BronzeAutomaton" | "Bronze Automaton", x) if x == move_ids::BA_SPAWN_ORBS => {
                // SpawnMonsterAction calls init() on each new minion before the
                // Automaton's queued RollMoveAction. Thus aiRng order is Orb 0,
                // Orb 1, Automaton. Run-level monsterHp streams are not split
                // yet, so choose an in-range HP semantically.
                // Java: reference/extracted/methods/monster/BronzeAutomaton.java
                // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/SpawnMonsterAction.java
                let high_hp = engine.state.enemies[enemy_idx].entity.max_hp >= 320;
                for count in 0..2 {
                    let hp = if high_hp { 54 } else { 52 };
                    let mut orb = enemies::create_enemy("BronzeOrb", hp, hp);
                    orb.is_minion = true;
                    orb.entity.set_status(sid::COUNT, count);
                    enemies::roll_initial_move(&mut orb, &mut engine.ai_rng);
                    engine.add_spawned_enemy(orb);
                }
            }
            ("Reptomancer", x) if x == move_ids::REPTO_SPAWN => {
                // Source: reference/extracted/methods/monster/Reptomancer.java
                // (`takeTurn`) fills up to four tracked dagger slots, one at
                // baseline and two at ascension 18.
                let alive_daggers = engine.state.enemies.iter().filter(|enemy|
                    matches!(enemy.id.as_str(), "SnakeDagger" | "Snake Dagger")
                        && enemy.is_alive()).count();
                let per_spawn = engine.state.enemies[enemy_idx]
                    .entity.status(sid::BLOCK_AMT).max(1) as usize;
                let spawn_count = (4usize.saturating_sub(alive_daggers)).min(per_spawn);
                for _ in 0..spawn_count {
                    // SpawnMonsterAction.init consumes one aiRng num before
                    // SnakeDagger.getMove selects its forced first move.
                    // HP uses an in-range semantic value until run-level RNG
                    // streams are split.
                    // Java: reference/extracted/methods/monster/SnakeDagger.java
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/SpawnMonsterAction.java
                    let mut minion = enemies::create_enemy("SnakeDagger", 22, 22);
                    minion.is_minion = true;
                    enemies::roll_initial_move(&mut minion, &mut engine.ai_rng);
                    engine.add_spawned_enemy(minion);
                }
            }
            ("GremlinLeader" | "Gremlin Leader", x) if x == move_ids::GL_RALLY => {
                // Sources: reference/extracted/methods/monster/GremlinLeader.java
                // and decompiled/java-src/com/megacrit/cardcrawl/actions/unique/
                // SummonGremlinAction.java.
                // Each of the two queued actions fills one of three slots,
                // draws from the weighted pool, then init consumes an AI roll.
                let alive_others = engine.state.enemies.iter().enumerate()
                    .filter(|(idx, enemy)| *idx != enemy_idx && enemy.is_alive())
                    .count();
                let spawn_count = (3usize.saturating_sub(alive_others)).min(2);
                let ascension = engine.state.enemies[enemy_idx]
                    .entity.status(sid::STARTING_DMG);
                for _ in 0..spawn_count {
                    let gremlin_id = match engine.ai_rng.random_range(0, 7) {
                        0 | 1 => "GremlinWarrior",
                        2 | 3 => "GremlinThief",
                        4 | 5 => "GremlinFat",
                        6 => "GremlinTsundere",
                        _ => "GremlinWizard",
                    };
                    // Monster HP uses a separate run-level stream in Java;
                    // choose the lower source-valid endpoint semantically.
                    let high_hp = ascension >= 7;
                    let hp = match gremlin_id {
                        "GremlinFat" => if high_hp { 14 } else { 13 },
                        "GremlinThief" => if high_hp { 11 } else { 10 },
                        "GremlinWarrior" => if high_hp { 21 } else { 20 },
                        "GremlinWizard" => if high_hp { 22 } else { 21 },
                        _ => if high_hp { 13 } else { 12 },
                    };
                    let mut minion = enemies::create_enemy(gremlin_id, hp, hp);
                    minion.is_minion = true;
                    match gremlin_id {
                        "GremlinFat" => {
                            let damage = if ascension >= 2 { 5 } else { 4 };
                            minion.entity.set_status(sid::STARTING_DMG, damage);
                            minion.entity.set_status(sid::BLOCK_AMT,
                                if ascension >= 17 { 17 } else { 0 });
                        }
                        "GremlinThief" => {
                            minion.entity.set_status(sid::STARTING_DMG,
                                if ascension >= 2 { 10 } else { 9 });
                        }
                        "GremlinWarrior" => {
                            minion.entity.set_status(sid::STARTING_DMG,
                                if ascension >= 2 { 5 } else { 4 });
                            minion.entity.set_status(sid::ANGRY,
                                if ascension >= 17 { 2 } else { 1 });
                        }
                        "GremlinWizard" => {
                            minion.entity.set_status(sid::STARTING_DMG,
                                if ascension >= 2 { 30 } else { 25 });
                            minion.entity.set_status(sid::COUNT, 1);
                            minion.entity.set_status(sid::BLOCK_AMT,
                                if ascension >= 17 { 17 } else { 0 });
                        }
                        _ => {
                            minion.entity.set_status(sid::STARTING_DMG,
                                if ascension >= 2 { 8 } else { 6 });
                            minion.entity.set_status(sid::BLOCK_AMT,
                                if ascension >= 17 { 11 }
                                else if ascension >= 7 { 8 } else { 7 });
                        }
                    }
                    enemies::roll_initial_move(&mut minion, &mut engine.ai_rng);
                    engine.add_spawned_enemy(minion);
                }
            }
            ("AcidSlime_L", x) if x == move_ids::AS_SPLIT => {
                // AcidSlime_L.takeTurn spawns two AcidSlime_M at current HP.
                let hp = engine.state.enemies[enemy_idx].entity.hp;
                let upgraded = engine.state.enemies[enemy_idx]
                    .entity.status(sid::STARTING_DMG) >= 12;
                let a17 = engine.state.enemies[enemy_idx].entity.status(sid::BLOCK_AMT) >= 17;
                engine.state.enemies[enemy_idx].entity.hp = 0;
                for _ in 0..2 {
                    let mut child = enemies::create_enemy("AcidSlime_M", hp, hp);
                    child.entity.set_status(sid::STARTING_DMG, if upgraded { 8 } else { 7 });
                    child.entity.set_status(sid::STR_AMT, if upgraded { 12 } else { 10 });
                    child.entity.set_status(sid::BLOCK_AMT, if a17 { 17 } else { 0 });
                    enemies::roll_initial_move(&mut child, &mut engine.ai_rng);
                    engine.add_spawned_enemy(child);
                }
            }
            ("SpikeSlime_L", x) if x == move_ids::SS_SPLIT => {
                // Source: reference/extracted/methods/monster/SpikeSlime_L.java
                // (`takeTurn` case SPLIT spawns two initialized SpikeSlime_M).
                let hp = engine.state.enemies[enemy_idx].entity.hp;
                let upgraded = engine.state.enemies[enemy_idx]
                    .entity.status(sid::STARTING_DMG) >= 18;
                let a17 = engine.state.enemies[enemy_idx].entity.status(sid::BLOCK_AMT) >= 17;
                engine.state.enemies[enemy_idx].entity.hp = 0;
                for _ in 0..2 {
                    let mut child = enemies::create_enemy("SpikeSlime_M", hp, hp);
                    child.entity.set_status(sid::STARTING_DMG, if upgraded { 10 } else { 8 });
                    child.entity.set_status(sid::BLOCK_AMT, if a17 { 17 } else { 0 });
                    enemies::roll_initial_move(&mut child, &mut engine.ai_rng);
                    engine.add_spawned_enemy(child);
                }
            }
            _ => {}
        }
    }

    if large_slime_split {
        return;
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "SnakeDagger" | "Snake Dagger")
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::SD_EXPLODE
    {
        // Source: reference/extracted/methods/monster/SnakeDagger.java
        // (`takeTurn`, case 2). LoseHPAction kills the dagger after its attack.
        // Its queued RollMoveAction survives only while another monster keeps
        // combat alive; LoseHPAction clears post-combat actions otherwise.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/LoseHPAction.java
        engine.state.enemies[enemy_idx].entity.hp = 0;
        engine.finalize_enemy_death(enemy_idx);
        if engine.state.enemies.iter().any(|enemy| enemy.is_alive()) {
            enemies::roll_next_move(
                &mut engine.state.enemies[enemy_idx], &mut engine.ai_rng);
        }
        return;
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "Shelled Parasite" | "ShelledParasite")
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::SP_STUNNED
    {
        // Source: ShelledParasite.takeTurn case STUNNED first installs Fell,
        // then its queued RollMoveAction records that Fell for getMove.
        let damage = engine.state.enemies[enemy_idx]
            .entity.status(sid::STR_AMT).max(18);
        engine.state.enemies[enemy_idx].set_move(
            enemies::move_ids::SP_FELL, damage, 1, 0);
        engine.state.enemies[enemy_idx].add_effect(mfx::FRAIL, 2);
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "GremlinFat" | "GremlinThief" | "GremlinWarrior" | "GremlinWizard"
            | "GremlinTsundere")
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::GREMLIN_ESCAPE
    {
        // Source: GremlinFat.java `takeTurn` case 99 (EscapeAction; no roll).
        engine.state.enemies[enemy_idx].is_escaping = true;
        engine.state.enemies[enemy_idx].entity.hp = 0;
        return;
    }

    if engine.state.enemies[enemy_idx].id == "GremlinTsundere" {
        let alive_count = engine.state.enemies.iter()
            .filter(|enemy| enemy.is_alive()).count();
        enemies::act1::advance_gremlin_tsundere_after_turn(
            &mut engine.state.enemies[enemy_idx], alive_count);
        return;
    }

    if engine.state.enemies[enemy_idx].id == "GremlinWizard" {
        enemies::act1::advance_gremlin_wizard_after_turn(
            &mut engine.state.enemies[enemy_idx]);
        return;
    }

    if engine.state.enemies[enemy_idx].id == "Lagavulin" {
        enemies::act1::advance_lagavulin_after_turn(
            &mut engine.state.enemies[enemy_idx], &mut engine.ai_rng);
        return;
    }

    if engine.state.enemies[enemy_idx].id == "TheGuardian" {
        enemies::act1::advance_guardian_after_turn(
            &mut engine.state.enemies[enemy_idx]);
        return;
    }

    if engine.state.enemies[enemy_idx].id == "Hexaghost" {
        let player_hp = engine.state.player.hp;
        enemies::act1::advance_hexaghost_after_turn(
            &mut engine.state.enemies[enemy_idx], player_hp, &mut engine.ai_rng);
        return;
    }

    if engine.state.enemies[enemy_idx].id == "SlimeBoss" {
        if engine.state.enemies[enemy_idx].move_id == enemies::move_ids::SB_SPLIT {
            do_slime_boss_split(engine, enemy_idx);
        } else {
            enemies::act1::advance_slime_boss_after_turn(
                &mut engine.state.enemies[enemy_idx]);
        }
        return;
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "Apology Slime" | "ApologySlime")
    {
        enemies::act1::advance_apology_slime_after_turn(
            &mut engine.state.enemies[enemy_idx]);
        return;
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "GremlinThief" | "GremlinWarrior")
    {
        // Sources: GremlinThief.java and GremlinWarrior.java. Their attacks
        // set the same attack directly without RollMoveAction or aiRng.
        engine.state.enemies[enemy_idx].move_history
            .push(enemies::move_ids::GREMLIN_ATTACK);
        return;
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(), "BanditBear" | "Bear") {
        // BanditBear.takeTurn uses SetMoveAction for every post-opener intent;
        // there is no RollMoveAction and therefore no aiRng consumption.
        // Java: reference/extracted/methods/monster/BanditBear.java
        enemies::act2::advance_bear_after_turn(&mut engine.state.enemies[enemy_idx]);
        return;
    }

    if matches!(engine.state.enemies[enemy_idx].id.as_str(),
        "BanditChild" | "BanditPointy" | "Pointy")
    {
        // BanditPointy.takeTurn repeats POINTY_SPECIAL with SetMoveAction;
        // the canonical game ID is BanditChild and no aiRng is consumed.
        // Java: reference/extracted/methods/monster/BanditPointy.java
        enemies::act2::advance_bandit_pointy_after_turn(
            &mut engine.state.enemies[enemy_idx]);
        return;
    }

    if engine.state.enemies[enemy_idx].id == "BanditLeader" {
        // Every post-Mock intent is installed by SetMoveAction in takeTurn;
        // BanditLeader never queues RollMoveAction after combat initialization.
        // Java: reference/extracted/methods/monster/BanditLeader.java
        enemies::act2::advance_bandit_leader_after_turn(
            &mut engine.state.enemies[enemy_idx]);
        return;
    }

    if engine.state.enemies[enemy_idx].id == "Byrd"
        && engine.state.enemies[enemy_idx].move_id == enemies::move_ids::BYRD_HEADBUTT
    {
        // Byrd.takeTurn case HEADBUTT sets FLY_UP directly and returns without
        // queuing RollMoveAction, so this transition consumes no aiRng tick.
        // Java: reference/extracted/methods/monster/Byrd.java.
        engine.state.enemies[enemy_idx].move_history
            .push(enemies::move_ids::BYRD_HEADBUTT);
        engine.state.enemies[enemy_idx].set_move(
            enemies::move_ids::BYRD_FLY_UP, 0, 0, 0);
        return;
    }

    if engine.state.enemies[enemy_idx].id == "Nemesis"
        && engine.state.enemies[enemy_idx].entity.status(sid::INTANGIBLE) <= 0
    {
        // Source: reference/extracted/methods/monster/Nemesis.java (`takeTurn`)
        // and IntangiblePower.java. Java installs the power after the move and
        // its justApplied flag skips this round's decrement. Rust stores 2 so
        // the shared end-of-round decrement leaves the visible amount at 1.
        engine.state.enemies[enemy_idx].entity.set_status(sid::INTANGIBLE, 2);
    }

    // These Java takeTurn methods set their next move directly and do not queue
    // RollMoveAction.
    if engine.state.enemies[enemy_idx].id == "AcidSlime_S" {
        enemies::advance_acid_slime_s_after_turn(&mut engine.state.enemies[enemy_idx]);
    } else if engine.state.enemies[enemy_idx].id == "Looter" {
        enemies::act1::advance_looter_after_turn(
            &mut engine.state.enemies[enemy_idx], &mut engine.ai_rng);
    } else if engine.state.enemies[enemy_idx].id == "Mugger" {
        enemies::act2::advance_mugger_after_turn(
            &mut engine.state.enemies[enemy_idx], &mut engine.ai_rng);
    } else {
        if engine.state.enemies[enemy_idx].id == "Centurion" {
            // Centurion.getMove checks the current monster group's alive count
            // to choose Protect with allies or Fury while alone.
            // Java: reference/extracted/methods/monster/Centurion.java.
            let alive_count = engine.state.enemies.iter()
                .filter(|enemy| enemy.is_alive()).count() as i32;
            engine.state.enemies[enemy_idx].entity.set_status(sid::COUNT, alive_count);
        } else if matches!(engine.state.enemies[enemy_idx].id.as_str(),
            "GremlinLeader" | "Gremlin Leader")
        {
            // Source: reference/extracted/methods/monster/GremlinLeader.java
            // (`numAliveGremlins` in the full source); despite its name, it
            // counts every other living monster in the encounter.
            let alive_count = engine.state.enemies.iter().enumerate()
                .filter(|(idx, enemy)| *idx != enemy_idx && enemy.is_alive())
                .count() as i32;
            engine.state.enemies[enemy_idx].entity.set_status(sid::COUNT, alive_count);
        } else if engine.state.enemies[enemy_idx].id == "Reptomancer" {
            // Source: decompiled/java-src/com/megacrit/cardcrawl/monsters/
            // beyond/Reptomancer.java (`canSpawn`) counts every other
            // non-dying monster, not only daggers.
            let alive_count = engine.state.enemies.iter().enumerate()
                .filter(|(idx, enemy)| *idx != enemy_idx && enemy.is_alive())
                .count() as i32;
            engine.state.enemies[enemy_idx].entity.set_status(sid::COUNT, alive_count);
        } else if matches!(engine.state.enemies[enemy_idx].id.as_str(),
            "Serpent" | "SpireGrowth" | "Spire Growth")
        {
            // Source: SpireGrowth.getMove checks whether the player currently
            // has Constricted before choosing its next intent.
            let player_constricted = engine.state.player.status(sid::CONSTRICTED) > 0;
            engine.state.enemies[enemy_idx].entity.set_status(
                sid::COUNT, i32::from(player_constricted));
        } else if matches!(engine.state.enemies[enemy_idx].id.as_str(),
            "Healer" | "Mystic")
        {
            // Source: reference/extracted/methods/monster/Healer.java
            // (`getMove`): needToHeal sums missing HP across the living group.
            let missing_hp = engine.state.enemies.iter()
                .filter(|enemy| enemy.is_alive())
                .map(|enemy| enemy.entity.max_hp - enemy.entity.hp)
                .sum();
            engine.state.enemies[enemy_idx].entity.set_status(sid::COUNT, missing_hp);
        }
        enemies::roll_next_move(&mut engine.state.enemies[enemy_idx], &mut engine.ai_rng);
    }
}

/// Handle boss-specific damage hooks (Guardian mode shift, SlimeBoss split,
/// Lagavulin wake, and Awakened One rebirth).
///
/// Called from `deal_damage_to_enemy()` when HP damage is dealt.
pub fn on_enemy_damaged(engine: &mut CombatEngine, enemy_idx: usize, hp_damage: i32) {
    if hp_damage <= 0 {
        return;
    }

    let enemy_id = engine.state.enemies[enemy_idx].id.clone();
    match enemy_id.as_str() {
        "TheGuardian" => {
            enemies::guardian_check_mode_shift(
                &mut engine.state.enemies[enemy_idx],
                hp_damage,
            );
        }
        "Lagavulin" => {
            // Wake Lagavulin if damaged while sleeping
            let sleep_turns = engine.state.enemies[enemy_idx].entity.status(sid::SLEEP_TURNS);
            if sleep_turns > 0 {
                enemies::lagavulin_wake_up(&mut engine.state.enemies[enemy_idx]);
            }
        }
        "SlimeBoss" => {
            if enemies::slime_boss_should_split(&engine.state.enemies[enemy_idx]) {
                // Source: SlimeBoss.java `damage`: crossing half HP interrupts
                // the current intent with Split; spawning waits for takeTurn.
                let enemy = &mut engine.state.enemies[enemy_idx];
                enemy.move_effects.clear();
                enemy.set_move(enemies::move_ids::SB_SPLIT, 0, 0, 0);
            }
        }
        "AcidSlime_L" | "SpikeSlime_L" => {
            let enemy = &mut engine.state.enemies[enemy_idx];
            let split_move = if enemy_id == "AcidSlime_L" {
                enemies::move_ids::AS_SPLIT
            } else {
                enemies::move_ids::SS_SPLIT
            };
            if enemy.entity.hp > 0
                && enemy.entity.hp * 2 <= enemy.entity.max_hp
                && enemy.move_id != split_move
                && enemy.entity.status(sid::THRESHOLD_REACHED) == 0
            {
                // Source: extracted AcidSlime_L/SpikeSlime_L `damage` methods.
                enemy.entity.set_status(sid::THRESHOLD_REACHED, 1);
                enemy.move_history.push(split_move);
                enemy.set_move(split_move, 0, 0, 0);
                enemy.move_effects.clear();
            }
        }
        "AwakenedOne" | "Awakened One" => {
            // Phase 1 death triggers rebirth — body stays at 0 HP and untargetable
            // until next enemy turn when rebirth executes (heal to full, phase 2).
            let phase = engine.state.enemies[enemy_idx].entity.status(sid::PHASE);
            if phase == 1 && engine.state.enemies[enemy_idx].entity.hp <= 0 {
                let enemy = &mut engine.state.enemies[enemy_idx];
                enemy.entity.hp = 0;
                enemy.entity.set_status(sid::REBIRTH_PENDING, 1);
                // damage() immediately removes all debuffs, Curiosity,
                // Unawakened, and Shackled before the later REBIRTH turn.
                // PHASE represents Unawakened in the Rust state and remains 1
                // until changeState; the other removable powers clear here.
                // Java: reference/extracted/methods/monster/AwakenedOne.java
                enemy.entity.set_status(sid::CURIOSITY, 0);
                enemy.entity.set_status(sid::TEMP_STRENGTH_LOSS, 0);
                if enemy.entity.strength() < 0 {
                    enemy.entity.set_status(sid::STRENGTH, 0);
                }
                for status_idx in 0..256 {
                    let status = crate::ids::StatusId(status_idx as u16);
                    if crate::powers::registry::status_is_debuff(status) {
                        enemy.entity.statuses[status_idx] = 0;
                    }
                }
                enemy.set_move(enemies::move_ids::AO_REBIRTH, 0, 0, 0);
            }
        }
        "Darkling" => {
            if engine.state.enemies[enemy_idx].entity.hp <= 0
                && engine.state.enemies[enemy_idx]
                    .entity.status(sid::REBIRTH_PENDING) == 0
            {
                // Source: reference/extracted/methods/monster/Darkling.java
                // (`damage`). A lethal hit fires ordinary death hooks, clears
                // every power, then either waits half-dead or ends the fight
                // when every Darkling is half-dead.
                let all_darklings_half_dead = engine.state.enemies.iter()
                    .enumerate()
                    .filter(|(_, enemy)| enemy.id == "Darkling")
                    .all(|(idx, enemy)| idx == enemy_idx
                        || enemy.entity.status(sid::REBIRTH_PENDING) > 0
                        || enemy.entity.hp <= 0);

                if all_darklings_half_dead {
                    // Previously half-dead Darklings are basically dead for
                    // relic death predicates while the final hooks resolve.
                    for enemy in engine.state.enemies.iter_mut()
                        .filter(|enemy| enemy.id == "Darkling")
                    {
                        enemy.entity.set_status(sid::REBIRTH_PENDING, 0);
                    }
                }

                engine.finalize_enemy_death(enemy_idx);

                let stored = [
                    (sid::STARTING_DMG,
                        engine.state.enemies[enemy_idx].entity.status(sid::STARTING_DMG)),
                    (sid::STR_AMT,
                        engine.state.enemies[enemy_idx].entity.status(sid::STR_AMT)),
                    (sid::HIGH_ASCENSION_AI,
                        engine.state.enemies[enemy_idx].entity.status(sid::HIGH_ASCENSION_AI)),
                    (sid::FIRST_MOVE,
                        engine.state.enemies[enemy_idx].entity.status(sid::FIRST_MOVE)),
                    (sid::COUNT,
                        engine.state.enemies[enemy_idx].entity.status(sid::COUNT)),
                ];
                engine.state.enemies[enemy_idx].entity.statuses.fill(0);
                for (status, value) in stored {
                    engine.state.enemies[enemy_idx].entity.set_status(status, value);
                }

                if all_darklings_half_dead {
                    // Darkling.damage calls die() on the entire monster group.
                    for enemy in &mut engine.state.enemies {
                        enemy.entity.hp = 0;
                        enemy.entity.set_status(sid::REBIRTH_PENDING, 0);
                    }
                } else {
                    let enemy = &mut engine.state.enemies[enemy_idx];
                    enemy.entity.hp = 0;
                    enemy.entity.set_status(sid::REBIRTH_PENDING, 1);
                    enemy.set_move(enemies::move_ids::DARK_WAIT, 0, 0, 0);
                }
            }
        }
        _ => {}
    }

    // Angry: enemy gains Strength when damaged
    let angry = engine.state.enemies[enemy_idx].entity.status(sid::ANGRY);
    if angry > 0 {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::STRENGTH, angry);
    }
}

/// Handle Slime Boss splitting into two smaller slimes.
fn do_slime_boss_split(engine: &mut CombatEngine, boss_idx: usize) {
    // Source: SlimeBoss.java `takeTurn` case 3. Spawn Spike first, then Acid,
    // both with the boss's current HP as current and maximum HP.
    let boss_current_hp = engine.state.enemies[boss_idx].entity.hp;

    // Kill the boss
    engine.state.enemies[boss_idx].entity.hp = 0;

    let upgraded = engine.state.enemies[boss_idx].entity.status(sid::STARTING_DMG) > 0;
    let a17 = engine.state.enemies[boss_idx].entity.status(sid::BLOCK_AMT) >= 17;
    let mut spike = enemies::create_enemy("SpikeSlime_L", boss_current_hp, boss_current_hp);
    spike.entity.set_status(sid::STARTING_DMG, if upgraded { 18 } else { 16 });
    spike.entity.set_status(sid::STR_AMT, if a17 { 3 } else { 2 });
    spike.entity.set_status(sid::BLOCK_AMT, if a17 { 17 } else { 0 });
    enemies::roll_initial_move(&mut spike, &mut engine.ai_rng);
    let mut acid = enemies::create_enemy("AcidSlime_L", boss_current_hp, boss_current_hp);
    acid.entity.set_status(sid::STARTING_DMG, if upgraded { 12 } else { 11 });
    acid.entity.set_status(sid::STR_AMT, if upgraded { 18 } else { 16 });
    acid.entity.set_status(sid::BLOCK_AMT, if a17 { 17 } else { 0 });
    enemies::roll_initial_move(&mut acid, &mut engine.ai_rng);

    engine.add_spawned_enemy(spike);
    engine.add_spawned_enemy(acid);
}
