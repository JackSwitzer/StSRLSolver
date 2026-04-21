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

/// Execute all enemy turns: block decay, poison ticks, ritual, moves.
pub fn do_enemy_turns(engine: &mut CombatEngine) {
    engine.phase = CombatPhase::EnemyTurn;

    let num_enemies = engine.state.enemies.len();
    for i in 0..num_enemies {
        if !engine.state.enemies[i].is_alive() {
            continue;
        }

        // Block decays at start of enemy turn
        engine.state.enemies[i].entity.block = 0;

        // Reset Invincible per-turn damage tracker
        powers::reset_invincible_damage_taken(&mut engine.state.enemies[i].entity);

        // Nemesis: gain Intangible at start of turn if not already present
        if engine.state.enemies[i].id == "Nemesis"
            && engine.state.enemies[i].entity.status(sid::INTANGIBLE) <= 0
        {
            engine.state.enemies[i].entity.set_status(sid::INTANGIBLE, 1);
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
}

/// Execute a single enemy's move (attack, block, status effects).
fn execute_enemy_move(engine: &mut CombatEngine, enemy_idx: usize) {
    // Awakened One rebirth: if pending, execute the rebirth this turn instead of normal move
    if engine.state.enemies[enemy_idx].entity.status(sid::REBIRTH_PENDING) > 0 {
        engine.state.enemies[enemy_idx].entity.set_status(sid::REBIRTH_PENDING, 0);
        enemies::awakened_one_rebirth(&mut engine.state.enemies[enemy_idx]);
        return;
    }

    let enemy = &engine.state.enemies[enemy_idx];
    if enemy.move_id == -1 {
        return;
    }

    // Attack
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
            // Buffer: negate the entire hit and decrement Buffer
            let buffer = engine.state.player.status(sid::BUFFER);
            if buffer > 0 {
                engine.state.player.set_status(sid::BUFFER, buffer - 1);
                continue;
            }

            let result = damage::calculate_incoming_damage(
                per_hit_base,
                engine.state.player.block,
                is_wrath,
                player_vuln,
                player_intangible,
                has_torii,
                has_tungsten,
                has_odd_mushroom,
            );

            engine.state.player.block = result.block_remaining;
            if result.hp_loss > 0 {
                engine.player_lose_hp(result.hp_loss);
                if engine.state.combat_over {
                    return;
                }

                // Plated Armor decrements on unblocked HP damage from enemy attacks.
                let plated = engine.state.player.status(sid::PLATED_ARMOR);
                if plated > 0 {
                    let new_plated = plated - 1;
                    engine.state.player.set_status(sid::PLATED_ARMOR, new_plated);
                }

                // Static Discharge: channel Lightning when taking unblocked damage
                let static_discharge = engine.state.player.status(sid::STATIC_DISCHARGE);
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

            // Thorns: deal Thorns damage back per hit (Java: ThornsPower.onAttacked)
            let thorns = engine.state.player.status(sid::THORNS);
            if thorns > 0 && engine.state.enemies[enemy_idx].is_alive() {
                let e = &mut engine.state.enemies[enemy_idx];
                let blocked_t = e.entity.block.min(thorns);
                let hp_dmg_t = thorns - blocked_t;
                e.entity.block -= blocked_t;
                e.entity.hp -= hp_dmg_t;
                engine.state.total_damage_dealt += hp_dmg_t;
                if hp_dmg_t > 0 {
                    engine.record_enemy_hp_damage(enemy_idx, hp_dmg_t);
                }
            }

            // Flame Barrier: deal FlameBarrier damage back per hit (Java: FlameBarrierPower.onAttacked)
            let flame_barrier = engine.state.player.status(sid::FLAME_BARRIER);
            if flame_barrier > 0 && engine.state.enemies[enemy_idx].is_alive() {
                let e = &mut engine.state.enemies[enemy_idx];
                let blocked_f = e.entity.block.min(flame_barrier);
                let hp_dmg_f = flame_barrier - blocked_f;
                e.entity.block -= blocked_f;
                e.entity.hp -= hp_dmg_f;
                engine.state.total_damage_dealt += hp_dmg_f;
                if hp_dmg_f > 0 {
                    engine.record_enemy_hp_damage(enemy_idx, hp_dmg_f);
                }
            }
        }
    }

    // Block
    let move_blk = engine.state.enemies[enemy_idx].move_block();
    if move_blk > 0 {
        engine.state.enemies[enemy_idx].entity.block += move_blk;
    }

    // Apply move effects
    let effects: SmallVec<[(u8, i16); 4]> = engine.state.enemies[enemy_idx].move_effects.clone();

    fn get_fx(effects: &SmallVec<[(u8, i16); 4]>, id: u8) -> Option<i16> {
        effects.iter().find(|e| e.0 == id).map(|e| e.1)
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
    if let Some(amt) = get_fx(&effects, mfx::STRENGTH) {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::STRENGTH, amt as i32);
    }
    if let Some(amt) = get_fx(&effects, mfx::RITUAL) {
        engine.state.enemies[enemy_idx]
            .entity
            .set_status(sid::RITUAL, amt as i32);
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
            engine.state.discard_pile.push(engine.card_registry.make_card("Daze"));
        }
    }
    if let Some(amt) = get_fx(&effects, mfx::BURN) {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Burn"));
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

    // Burn+: add upgraded Burn cards to player discard
    if let Some(amt) = get_fx(&effects, mfx::BURN_UPGRADE) {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Burn+"));
        }
    }

    // Confused: apply Confusion to player
    if get_fx(&effects, mfx::CONFUSED).unwrap_or(0) > 0 {
        engine.state.player.set_status(sid::CONFUSION, 1);
    }

    // Constrict: apply Constricted to player
    if let Some(amt) = get_fx(&effects, mfx::CONSTRICT) {
        engine.state.player.add_status(sid::CONSTRICTED, amt as i32);
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

    // Stasis: steal random card from player hand (simplified: remove from hand)
    if get_fx(&effects, mfx::STASIS).unwrap_or(0) > 0 {
        if !engine.state.hand.is_empty() {
            let idx = engine.state.hand.len() - 1;
            engine.state.hand.remove(idx);
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
            engine.state.draw_pile.push(engine.card_registry.make_card("Void"));
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

    // Cross-enemy effects (Centurion Protect, Mystic Heal, GremlinLeader Encourage)
    if let Some(amt) = get_fx(&effects, mfx::BLOCK_ALL_ALLIES) {
        for j in 0..engine.state.enemies.len() {
            if j != enemy_idx && engine.state.enemies[j].is_alive() {
                engine.state.enemies[j].entity.block += amt as i32;
            }
        }
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
    if let Some(amt) = get_fx(&effects, mfx::STRENGTH_ALL_ALLIES) {
        for j in 0..engine.state.enemies.len() {
            if j != enemy_idx && engine.state.enemies[j].is_alive() {
                engine.state.enemies[j].entity.add_status(sid::STRENGTH, amt as i32);
            }
        }
    }

    // Spawn minions for boss spawn moves
    {
        use crate::enemies::move_ids;
        let eid = engine.state.enemies[enemy_idx].id.as_str();
        let mid = engine.state.enemies[enemy_idx].move_id;
        match (eid, mid) {
            ("TheCollector" | "Collector", x) if x == move_ids::COLL_SPAWN => {
                for _ in 0..2 {
                    engine.state.enemies.push(enemies::create_enemy("TorchHead", 6, 6));
                }
            }
            ("BronzeAutomaton" | "Bronze Automaton", x) if x == move_ids::BA_SPAWN_ORBS => {
                for _ in 0..2 {
                    engine.state.enemies.push(enemies::create_enemy("BronzeOrb", 52, 52));
                }
            }
            ("Reptomancer", x) if x == move_ids::REPTO_SPAWN => {
                for _ in 0..2 {
                    engine.state.enemies.push(enemies::create_enemy("SnakeDagger", 22, 22));
                }
            }
            ("GremlinLeader" | "Gremlin Leader", x) if x == move_ids::GL_RALLY => {
                // Deterministic MCTS: fixed gremlin types
                engine.state.enemies.push(enemies::create_enemy("GremlinWarrior", 20, 20));
                engine.state.enemies.push(enemies::create_enemy("GremlinThief", 28, 28));
            }
            _ => {}
        }
    }

    // Advance enemy to next move for next turn
    enemies::roll_next_move(&mut engine.state.enemies[enemy_idx], &mut engine.ai_rng);
}

/// Handle boss-specific damage hooks (Guardian mode shift, SlimeBoss split, Lagavulin wake,
/// Awakened One rebirth, Champ execute threshold).
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
                do_slime_boss_split(engine, enemy_idx);
            }
        }
        "AwakenedOne" | "Awakened One" => {
            // Phase 1 death triggers rebirth — body stays at 0 HP and untargetable
            // until next enemy turn when rebirth executes (heal to full, phase 2).
            let phase = engine.state.enemies[enemy_idx].entity.status(sid::PHASE);
            if phase == 1 && engine.state.enemies[enemy_idx].entity.hp <= 0 {
                engine.state.enemies[enemy_idx].entity.hp = 0;
                engine.state.enemies[enemy_idx].entity.set_status(sid::REBIRTH_PENDING, 1);
            }
        }
        "Champ" => {
            // When Champ drops to <= 50% HP, immediately trigger Phase 2 (Anger).
            // roll_champ handles the move selection, but we re-roll here so the
            // transition happens mid-turn rather than waiting for next enemy turn.
            if engine.state.enemies[enemy_idx].entity.hp
                <= engine.state.enemies[enemy_idx].entity.max_hp / 2
                && engine.state.enemies[enemy_idx]
                    .entity
                    .status(sid::THRESHOLD_REACHED)
                    == 0
            {
                enemies::roll_next_move(
                    &mut engine.state.enemies[enemy_idx],
                    &mut engine.ai_rng,
                );
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
    // Capture boss's current HP before killing (each child gets the boss's current HP)
    let boss_current_hp = engine.state.enemies[boss_idx].entity.hp;

    // Kill the boss
    engine.state.enemies[boss_idx].entity.hp = 0;

    // Spawn two Large slimes (one Acid, one Spike) with boss's current HP
    let acid = enemies::create_enemy("AcidSlime_L", boss_current_hp, boss_current_hp);
    let spike = enemies::create_enemy("SpikeSlime_L", boss_current_hp, boss_current_hp);

    engine.state.enemies.push(acid);
    engine.state.enemies.push(spike);
}
