//! Enemy turn logic — enemy moves, boss damage hooks.
//!
//! Extracted from engine.rs as a pure refactor.

use crate::damage;
use crate::enemies;
use crate::engine::{CombatEngine, CombatPhase};
use crate::potions;
use crate::powers;
use crate::state::Stance;
use crate::status_ids::sid;

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

        // === POWER HOOKS: enemy turn start (via dispatch) ===
        let is_first = engine.state.enemies[i].first_turn;
        let efx = powers::hooks::dispatch_enemy_turn_start(
            &mut engine.state.enemies[i].entity,
            is_first,
        );

        // Metallicize block
        if efx.block_gain > 0 {
            engine.state.enemies[i].entity.block += efx.block_gain;
        }

        // Regeneration heal (capped at max_hp)
        if efx.heal > 0 {
            let max_hp = engine.state.enemies[i].entity.max_hp;
            engine.state.enemies[i].entity.hp =
                (engine.state.enemies[i].entity.hp + efx.heal).min(max_hp);
        }

        // Growth block (Strength already applied inside hook)
        if efx.block_from_growth > 0 {
            engine.state.enemies[i].entity.block += efx.block_from_growth;
        }

        // Fading: die at 0
        if efx.faded {
            engine.state.enemies[i].entity.hp = 0;
            continue;
        }

        // TheBomb: detonate dealing damage to player
        if efx.bomb_damage > 0 {
            let intangible = engine.state.player.status(sid::INTANGIBLE) > 0;
            let has_tungsten = engine.state.has_relic("Tungsten Rod");
            let hp_loss = damage::apply_hp_loss(efx.bomb_damage, intangible, has_tungsten);
            engine.state.player.hp -= hp_loss;
            engine.state.total_damage_taken += hp_loss;
            if engine.state.player.is_dead() {
                engine.state.player.hp = 0;
                engine.state.combat_over = true;
                engine.state.player_won = false;
                engine.phase = CombatPhase::CombatOver;
                return;
            }
        }

        // Poison tick — kept inline (complex death check + boss hooks)
        let poison_dmg = powers::tick_poison(&mut engine.state.enemies[i].entity);
        if poison_dmg > 0 {
            engine.state.total_damage_dealt += poison_dmg;
            on_enemy_damaged(engine, i, poison_dmg);
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
        let player_intangible = engine.state.player.status(sid::INTANGIBLE) > 0;
        let has_torii = engine.state.has_relic("Torii");
        let has_tungsten = engine.state.has_relic("Tungsten Rod");

        let hits = enemy.move_hits;
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
            );

            engine.state.player.block = result.block_remaining;
            if result.hp_loss > 0 {
                engine.state.player.hp -= result.hp_loss;
                engine.state.total_damage_taken += result.hp_loss;

                // Plated Armor decrements on unblocked HP damage
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
                                if e.entity.hp <= 0 {
                                    e.entity.hp = 0;
                                }
                            }
                        }
                        crate::orbs::EvokeEffect::FrostBlock(blk) => {
                            engine.state.player.block += blk;
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
                if e.entity.hp <= 0 {
                    e.entity.hp = 0;
                }
                if hp_dmg_t > 0 {
                    on_enemy_damaged(engine, enemy_idx, hp_dmg_t);
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
                if e.entity.hp <= 0 {
                    e.entity.hp = 0;
                }
                if hp_dmg_f > 0 {
                    on_enemy_damaged(engine, enemy_idx, hp_dmg_f);
                }
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
        powers::apply_debuff(&mut engine.state.player, sid::WEAKENED, amt);
    }
    if let Some(&amt) = effects.get("vulnerable") {
        powers::apply_debuff(&mut engine.state.player, sid::VULNERABLE, amt);
    }
    if let Some(&amt) = effects.get("frail") {
        powers::apply_debuff(&mut engine.state.player, sid::FRAIL, amt);
    }
    if let Some(&amt) = effects.get("strength") {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::STRENGTH, amt);
    }
    if let Some(&amt) = effects.get("ritual") {
        engine.state.enemies[enemy_idx]
            .entity
            .set_status(sid::RITUAL, amt);
    }
    if let Some(&amt) = effects.get("entangle") {
        if amt > 0 {
            engine.state.player.set_status(sid::ENTANGLED, 1);
        }
    }
    if let Some(&amt) = effects.get("slimed") {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Slimed"));
        }
    }
    if let Some(&amt) = effects.get("daze") {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Daze"));
        }
    }
    if let Some(&amt) = effects.get("burn") {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Burn"));
        }
    }
    if let Some(&amt) = effects.get("burn+") {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Burn"));
        }
    }
    // Lagavulin Siphon Soul: reduce player Strength and Dexterity
    if let Some(&amt) = effects.get("siphon_str") {
        engine.state.player.add_status(sid::STRENGTH, -(amt));
    }
    if let Some(&amt) = effects.get("siphon_dex") {
        engine.state.player.add_status(sid::DEXTERITY, -(amt));
    }

    // Champ Anger / Time Eater Haste: remove ALL debuffs from this enemy
    // Uses PowerDef registry to identify debuffs rather than a hardcoded list
    if effects.get("remove_debuffs").copied().unwrap_or(0) > 0 {
        let statuses = &mut engine.state.enemies[enemy_idx].entity.statuses;
        for i in 0..256 {
            if statuses[i] != 0 {
                let sid = crate::ids::StatusId(i as u16);
                let name = crate::status_ids::status_name(sid);
                if crate::powers::get_power_def(name)
                    .map(|def| def.power_type == crate::powers::PowerType::Debuff)
                    .unwrap_or(false)
                {
                    statuses[i] = 0;
                }
            }
        }
    }

    // Time Eater Haste: heal to half max HP
    if effects.get("heal_to_half").copied().unwrap_or(0) > 0 {
        let half = engine.state.enemies[enemy_idx].entity.max_hp / 2;
        engine.state.enemies[enemy_idx].entity.hp = half;
    }

    // Heal full (Awakened One rebirth, etc.)
    if effects.get("heal_full").copied().unwrap_or(0) > 0 {
        engine.state.enemies[enemy_idx].entity.hp =
            engine.state.enemies[enemy_idx].entity.max_hp;
    }

    // Artifact: give enemy Artifact stacks
    if let Some(&amt) = effects.get("artifact") {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::ARTIFACT, amt);
    }

    // Burn+: add upgraded Burn cards to player discard
    if let Some(&amt) = effects.get("burn_upgrade") {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Burn+"));
        }
    }

    // Confused: apply Confusion to player
    if effects.get("confused").copied().unwrap_or(0) > 0 {
        engine.state.player.set_status(sid::CONFUSION, 1);
    }

    // Constrict: apply Constricted to player
    if let Some(&amt) = effects.get("constrict") {
        engine.state.player.add_status(sid::CONSTRICTED, amt);
    }

    // Dexterity down: reduce player Dexterity
    if let Some(&amt) = effects.get("dexterity_down") {
        engine.state.player.add_status(sid::DEXTERITY, -amt);
    }

    // Draw Reduction: reduce player draw next turn
    if let Some(&amt) = effects.get("draw_reduction") {
        engine.state.player.add_status(sid::DRAW_REDUCTION, amt);
    }

    // Hex: apply Hex to player
    if let Some(&amt) = effects.get("hex") {
        engine.state.player.set_status(sid::HEX, amt);
    }

    // Painful Stabs: add Wound cards to player discard
    if let Some(&amt) = effects.get("painful_stabs") {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Wound"));
        }
    }

    // Stasis: steal random card from player hand (simplified: remove from hand)
    if effects.get("stasis").copied().unwrap_or(0) > 0 {
        if !engine.state.hand.is_empty() {
            let idx = engine.state.hand.len() - 1;
            engine.state.hand.remove(idx);
        }
    }

    // Strength bonus: give enemy Strength
    if let Some(&amt) = effects.get("strength_bonus") {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::STRENGTH, amt);
    }

    // Strength down: reduce player Strength
    if let Some(&amt) = effects.get("strength_down") {
        engine.state.player.add_status(sid::STRENGTH, -amt);
    }

    // Thorns: give enemy Thorns
    if let Some(&amt) = effects.get("thorns") {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::THORNS, amt);
    }

    // Void: add Void card to player draw pile
    if let Some(&amt) = effects.get("void") {
        for _ in 0..amt {
            engine.state.draw_pile.push(engine.card_registry.make_card("Void"));
        }
    }

    // Wound: add Wound cards to player discard
    if let Some(&amt) = effects.get("wound") {
        for _ in 0..amt {
            engine.state.discard_pile.push(engine.card_registry.make_card("Wound"));
        }
    }

    // Beat of Death: set Beat of Death power on enemy
    if let Some(&amt) = effects.get("beat_of_death") {
        engine.state.enemies[enemy_idx]
            .entity
            .set_status(sid::BEAT_OF_DEATH, amt);
    }

    // Advance enemy to next move for next turn
    enemies::roll_next_move(&mut engine.state.enemies[enemy_idx]);
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
            let enemy = &mut engine.state.enemies[enemy_idx];
            if enemy.entity.hp <= enemy.entity.max_hp / 2
                && enemy.entity.status(sid::THRESHOLD_REACHED) == 0
            {
                enemies::roll_next_move(enemy);
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

/// Handle Time Eater card count and other enemy on-card-played triggers.
///
/// Called from `play_card()` after a card is played. Increments Time Warp on
/// all living enemies that have the power. When 12 cards are reached, the
/// Time Eater ends the player's turn and gains Strength.
///
/// Returns `true` if the player's turn should end (Time Warp triggered).
pub fn on_player_card_played(engine: &mut CombatEngine, card_type: crate::cards::CardType) -> bool {
    let mut end_turn = false;

    for i in 0..engine.state.enemies.len() {
        if !engine.state.enemies[i].is_alive() {
            continue;
        }

        // Time Warp: count cards, at 12 end turn + enemy gains 2 Strength
        if powers::increment_time_warp(&mut engine.state.enemies[i].entity) {
            engine.state.enemies[i].entity.add_status(sid::STRENGTH, 2);
            end_turn = true;
        }

        // Curiosity: enemy gains Strength when player plays a Power card
        if card_type == crate::cards::CardType::Power {
            let curiosity = engine.state.enemies[i].entity.status(sid::CURIOSITY);
            if curiosity > 0 {
                engine.state.enemies[i]
                    .entity
                    .add_status(sid::STRENGTH, curiosity);
            }
        }
    }

    end_turn
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
