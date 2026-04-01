//! Enemy turn logic — enemy moves, boss damage hooks.
//!
//! Extracted from engine.rs as a pure refactor.

use crate::damage;
use crate::enemies;
use crate::engine::{CombatEngine, CombatPhase};
use crate::potions;
use crate::powers;
use crate::state::Stance;

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

        // Metallicize: enemy gains block
        powers::apply_metallicize(&mut engine.state.enemies[i].entity);

        // Poison tick
        let poison_dmg = powers::tick_poison(&mut engine.state.enemies[i].entity);
        if poison_dmg > 0 {
            engine.state.total_damage_dealt += poison_dmg;
            if engine.state.enemies[i].entity.is_dead() {
                engine.state.enemies[i].entity.hp = 0;
                continue;
            }
        }

        // Ritual strength gain (not first turn)
        if !engine.state.enemies[i].first_turn {
            powers::apply_ritual(&mut engine.state.enemies[i].entity);
        }

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
    let enemy = &engine.state.enemies[enemy_idx];
    if enemy.move_id == -1 {
        return;
    }

    // Attack
    if enemy.move_damage > 0 {
        let enemy_strength = enemy.entity.strength();
        let enemy_weak = enemy.entity.is_weak();
        let base_damage = enemy.move_damage + enemy_strength;

        // Apply Weak to enemy's attack
        let mut damage_f = base_damage as f64;
        if enemy_weak {
            damage_f *= damage::WEAK_MULT;
        }

        // Floor the per-hit base (before stance/vuln/intangible)
        let per_hit_base = (damage_f as i32).max(0);

        let is_wrath = engine.state.stance == Stance::Wrath;
        let player_vuln = engine.state.player.is_vulnerable();
        let player_intangible = engine.state.player.status("Intangible") > 0;
        let has_torii = engine.state.has_relic("Torii");
        let has_tungsten = engine.state.has_relic("Tungsten Rod");

        let hits = enemy.move_hits;
        for _ in 0..hits {
            let result = damage::calculate_incoming_damage(
                per_hit_base,
                engine.state.player.block,
                is_wrath,
                player_vuln,
                player_intangible,
                has_torii,
                has_tungsten,
            );

            engine.state.player.block = result.block_remaining;
            if result.hp_loss > 0 {
                engine.state.player.hp -= result.hp_loss;
                engine.state.total_damage_taken += result.hp_loss;

                // Plated Armor decrements on unblocked HP damage
                let plated = engine.state.player.status("Plated Armor");
                if plated > 0 {
                    let new_plated = plated - 1;
                    engine.state.player.set_status("Plated Armor", new_plated);
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
        }
    }

    // Block
    if engine.state.enemies[enemy_idx].move_block > 0 {
        let block = engine.state.enemies[enemy_idx].move_block;
        engine.state.enemies[enemy_idx].entity.block += block;
    }

    // Apply move effects
    let effects = engine.state.enemies[enemy_idx].move_effects.clone();
    if let Some(&amt) = effects.get("weak") {
        powers::apply_debuff(&mut engine.state.player, "Weakened", amt);
    }
    if let Some(&amt) = effects.get("vulnerable") {
        powers::apply_debuff(&mut engine.state.player, "Vulnerable", amt);
    }
    if let Some(&amt) = effects.get("frail") {
        powers::apply_debuff(&mut engine.state.player, "Frail", amt);
    }
    if let Some(&amt) = effects.get("strength") {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status("Strength", amt);
    }
    if let Some(&amt) = effects.get("ritual") {
        engine.state.enemies[enemy_idx]
            .entity
            .set_status("Ritual", amt);
    }
    if let Some(&amt) = effects.get("entangle") {
        if amt > 0 {
            engine.state.player.set_status("Entangled", 1);
        }
    }
    if let Some(&amt) = effects.get("slimed") {
        for _ in 0..amt {
            engine.state.discard_pile.push("Slimed".to_string());
        }
    }
    if let Some(&amt) = effects.get("daze") {
        for _ in 0..amt {
            engine.state.discard_pile.push("Daze".to_string());
        }
    }
    if let Some(&amt) = effects.get("burn") {
        for _ in 0..amt {
            engine.state.discard_pile.push("Burn".to_string());
        }
    }
    if let Some(&amt) = effects.get("burn+") {
        for _ in 0..amt {
            engine.state.discard_pile.push("Burn".to_string());
        }
    }
    // Lagavulin Siphon Soul: reduce player Strength and Dexterity
    if let Some(&amt) = effects.get("siphon_str") {
        engine.state.player.add_status("Strength", -(amt));
    }
    if let Some(&amt) = effects.get("siphon_dex") {
        engine.state.player.add_status("Dexterity", -(amt));
    }

    // Advance enemy to next move for next turn
    enemies::roll_next_move(&mut engine.state.enemies[enemy_idx]);
}

/// Handle boss-specific damage hooks (Guardian mode shift, SlimeBoss split, Lagavulin wake).
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
            let sleep_turns = engine.state.enemies[enemy_idx].entity.status("SleepTurns");
            if sleep_turns > 0 {
                enemies::lagavulin_wake_up(&mut engine.state.enemies[enemy_idx]);
            }
        }
        "SlimeBoss" => {
            if enemies::slime_boss_should_split(&engine.state.enemies[enemy_idx]) {
                do_slime_boss_split(engine, enemy_idx);
            }
        }
        _ => {}
    }
}

/// Handle Slime Boss splitting into two smaller slimes.
fn do_slime_boss_split(engine: &mut CombatEngine, boss_idx: usize) {
    // Kill the boss
    engine.state.enemies[boss_idx].entity.hp = 0;

    // Spawn two medium slimes (one Acid, one Spike) at the boss's remaining HP split
    let boss_max_hp = engine.state.enemies[boss_idx].entity.max_hp;
    let slime_hp = boss_max_hp / 4; // Each medium slime gets 1/4 of boss max HP

    let acid = enemies::create_enemy("AcidSlime_M", slime_hp, slime_hp);
    let spike = enemies::create_enemy("SpikeSlime_M", slime_hp, slime_hp);

    engine.state.enemies.push(acid);
    engine.state.enemies.push(spike);
}
