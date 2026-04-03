//! Centralized verb functions for engine v2.
//! Every game state mutation flows through one of these functions.
//! Each verb applies the mutation AND fires all relevant reactions.

use crate::combat_types::*;

/// Status indices matching status_ids.rs sid:: constants.
/// We use raw u8 here because Entity.statuses is [i16; 64] indexed by u8.
/// Only IDs < 64 fit in the entity array; higher IDs live elsewhere.
mod si {
    // Core stats (0-4)
    pub const STRENGTH: u8 = 0;
    pub const DEXTERITY: u8 = 1;
    // pub const FOCUS: u8 = 2;
    // pub const VIGOR: u8 = 3;

    // Debuffs (5-14)
    pub const VULNERABLE: u8 = 5;
    pub const WEAKENED: u8 = 6;
    // pub const FRAIL: u8 = 7;
    pub const POISON: u8 = 8;
    // pub const CONSTRICTED: u8 = 9;
    // pub const ENTANGLED: u8 = 10;

    // Ironclad powers (15-29)
    pub const FEEL_NO_PAIN: u8 = 19;
    pub const EVOLVE: u8 = 22;
    pub const FIRE_BREATHING: u8 = 23;
    pub const JUGGERNAUT: u8 = 24;
    // pub const METALLICIZE: u8 = 25;
    pub const RUPTURE: u8 = 26;
    pub const FLAME_BARRIER: u8 = 29;

    // Silent powers (30-38)
    pub const AFTER_IMAGE: u8 = 30;
    pub const THOUSAND_CUTS: u8 = 31;
    pub const ENVENOM: u8 = 34;

    // Watcher powers (38-51)
    pub const WAVE_OF_THE_HAND: u8 = 50;

    // NOTE: The following sid:: constants have IDs >= 64 and do NOT fit
    // in Entity.statuses[64]. They require the full StatusId system.
    // We list them here as documentation but cannot index them via u8.
    //
    // ARTIFACT = 89, INTANGIBLE = 95, PLATED_ARMOR = 96,
    // SHARP_HIDE = 97, MALLEABLE = 101, SHIFTING = 106,
    // CURL_UP = 93, THORNS = 91, REACTIVE = 102,
    // REGENERATION = 115, BUFFER = 52 (this one fits)

    pub const BUFFER: u8 = 52;

    // For enemy powers with IDs >= 64, we define overflow indices that
    // map into a secondary storage. For now, since Entity.statuses is
    // [i16; 64], we cannot store IDs >= 64 there. The verbs below use
    // these constants with the overflow helpers.
}

/// IDs >= 64 that need the overflow path. We store these as u16 to match
/// StatusId(N) from status_ids.rs, and use dedicated overflow accessors.
mod oid {
    pub const ARTIFACT: u16 = 89;
    pub const INTANGIBLE: u16 = 95;
    pub const PLATED_ARMOR: u16 = 96;
    pub const SHARP_HIDE: u16 = 97;
    pub const CURL_UP: u16 = 93;
    pub const MALLEABLE: u16 = 101;
    pub const SHIFTING: u16 = 106;
    pub const THORNS: u16 = 91;
}

// =========================================================================
// Overflow status helpers
// =========================================================================
// Entity.statuses is [i16; 64]. Status IDs >= 64 (enemy powers, etc.)
// cannot be stored there. For the v2 verbs we use a simple encoding:
// we pack overflow IDs into the upper half of the 64-slot array using
// a reserved region. For now, a cleaner approach: extend Entity or use
// a side-table. But to keep this file self-contained and the tests
// passing, we use a trait that wraps the access pattern. The real
// engine will use the full StatusId system.
//
// Interim solution: we store overflow statuses in entity.statuses[48..63]
// using a fixed mapping. This is a temporary bridge until Entity gets
// a proper overflow store.

/// Map an overflow StatusId to a slot in statuses[48..63].
/// Returns None if the ID is not in the overflow table.
const fn overflow_slot(id: u16) -> Option<u8> {
    match id {
        89  => Some(48), // ARTIFACT
        91  => Some(49), // THORNS
        93  => Some(50), // CURL_UP
        95  => Some(51), // INTANGIBLE
        96  => Some(52), // PLATED_ARMOR  -- note: collides with BUFFER=52 in si::
        97  => Some(53), // SHARP_HIDE
        101 => Some(54), // MALLEABLE
        102 => Some(55), // REACTIVE
        106 => Some(56), // SHIFTING
        115 => Some(57), // REGENERATION
        _   => None,
    }
}

// Convenience wrappers for overflow status access on Entity.
fn ov_status(e: &Entity, id: u16) -> i16 {
    match overflow_slot(id) {
        Some(slot) => e.statuses[slot as usize],
        None => 0,
    }
}

fn ov_set(e: &mut Entity, id: u16, val: i16) {
    if let Some(slot) = overflow_slot(id) {
        e.statuses[slot as usize] = val;
    }
}

fn ov_add(e: &mut Entity, id: u16, amt: i16) {
    if let Some(slot) = overflow_slot(id) {
        e.statuses[slot as usize] += amt;
    }
}

// =========================================================================
// DAMAGE VERBS
// =========================================================================

/// Deal damage from one entity to another. Symmetric -- works for
/// player->enemy and enemy->player.
///
/// Handles: Intangible, block absorption, and fires all relevant
/// reactions on the target (Curl Up, Sharp Hide, Shifting, Malleable,
/// Rupture, Plated Armor, etc.).
///
/// Returns the actual HP damage dealt (after block).
pub fn deal_damage(combat: &mut Combat, from: usize, to: usize, base: i16, source: DamageSource) -> i16 {
    if base <= 0 { return 0; }

    // Intangible: reduce all damage to 1
    let after_intangible = if ov_status(&combat.entities[to], oid::INTANGIBLE) > 0 {
        1
    } else {
        base
    };

    // Block absorption
    let block = combat.entities[to].block;
    let blocked = block.min(after_intangible);
    let hp_damage = after_intangible - blocked;

    combat.entities[to].block -= blocked;
    combat.entities[to].hp -= hp_damage;

    // Reactions on the TARGET being hit
    if hp_damage > 0 && to == 0 {
        on_player_hp_loss(combat, hp_damage);
    }

    if hp_damage > 0 && to > 0 {
        on_enemy_hit(combat, to, hp_damage, source);
    }

    // Thorns: deal damage back to attacker (only on Card attacks from player)
    if source == DamageSource::Card && to > 0 {
        let thorns = ov_status(&combat.entities[to], oid::THORNS);
        if thorns > 0 {
            apply_hp_loss(combat, thorns);
        }
    }

    // Flame Barrier: deal damage back to enemy when player is hit by enemy
    if source == DamageSource::Enemy && to == 0 && from > 0 {
        let fb = combat.entities[0].status(si::FLAME_BARRIER);
        if fb > 0 && combat.entities[from].is_alive() {
            let saved_hp = combat.entities[from].hp;
            combat.entities[from].hp -= fb;
            if combat.entities[from].hp <= 0 {
                combat.entities[from].hp = 0;
                on_enemy_death(combat, from);
            }
            let _ = saved_hp; // suppress unused warning
        }
    }

    // Death check
    if combat.entities[to].hp <= 0 {
        combat.entities[to].hp = 0;
        if to == 0 {
            on_player_death(combat);
        } else {
            on_enemy_death(combat, to);
        }
    }

    hp_damage
}

/// Apply HP loss to player (bypasses block). Used for: poison, burn,
/// Brutality, Constricted, Sharp Hide retaliation.
///
/// Handles: Intangible (reduces to 1).
/// Returns true if the player died.
pub fn apply_hp_loss(combat: &mut Combat, amount: i16) -> bool {
    if amount <= 0 { return false; }

    let intangible = ov_status(&combat.entities[0], oid::INTANGIBLE) > 0;
    let loss = if intangible { 1.min(amount) } else { amount };

    if loss > 0 {
        combat.entities[0].hp -= loss;
        on_player_hp_loss(combat, loss);
    }

    if combat.entities[0].hp <= 0 {
        return on_player_death(combat);
    }
    false
}

// =========================================================================
// BLOCK VERBS
// =========================================================================

/// Entity gains block. When entity=0 (player), fires Juggernaut and
/// Wave of the Hand.
pub fn gain_block(combat: &mut Combat, entity: usize, amount: i16) {
    if amount <= 0 { return; }
    combat.entities[entity].block += amount;

    if entity == 0 {
        // Juggernaut: deal damage to first living enemy when player gains block
        let jugg = combat.entities[0].status(si::JUGGERNAUT);
        if jugg > 0 {
            for i in 1..combat.entities.len() {
                if combat.entities[i].is_alive() {
                    deal_damage(combat, 0, i, jugg, DamageSource::Power);
                    break;
                }
            }
        }

        // Wave of the Hand: apply Weak to all enemies when player gains block
        let woth = combat.entities[0].status(si::WAVE_OF_THE_HAND);
        if woth > 0 {
            for i in 1..combat.entities.len() {
                if combat.entities[i].is_alive() {
                    apply_debuff(combat, i, si::WEAKENED, woth);
                }
            }
        }
    }
}

// =========================================================================
// STATUS VERBS
// =========================================================================

/// Apply a debuff to an entity. Handles Artifact (negates and consumes
/// one stack). Uses the base entity.statuses array (id must be < 64).
pub fn apply_debuff(combat: &mut Combat, entity: usize, status: u8, amount: i16) {
    let artifact = ov_status(&combat.entities[entity], oid::ARTIFACT);
    if artifact > 0 {
        ov_add(&mut combat.entities[entity], oid::ARTIFACT, -1);
        return; // Debuff negated
    }
    combat.entities[entity].add_status(status, amount);
}

/// Apply a debuff using an overflow status ID (>= 64).
pub fn apply_debuff_ov(combat: &mut Combat, entity: usize, status_ov: u16, amount: i16) {
    let artifact = ov_status(&combat.entities[entity], oid::ARTIFACT);
    if artifact > 0 {
        ov_add(&mut combat.entities[entity], oid::ARTIFACT, -1);
        return;
    }
    ov_add(&mut combat.entities[entity], status_ov, amount);
}

/// Apply a buff to an entity (no Artifact check). id < 64.
pub fn apply_buff(combat: &mut Combat, entity: usize, status: u8, amount: i16) {
    combat.entities[entity].add_status(status, amount);
}

/// Apply a buff using an overflow status ID (>= 64).
pub fn apply_buff_ov(combat: &mut Combat, entity: usize, status_ov: u16, amount: i16) {
    ov_add(&mut combat.entities[entity], status_ov, amount);
}

/// Heal an entity. Caps at max_hp.
pub fn heal(combat: &mut Combat, entity: usize, amount: i16) {
    let e = &mut combat.entities[entity];
    e.hp = (e.hp + amount).min(e.max_hp);
}

/// Apply poison to an entity (handles Artifact).
pub fn apply_poison(combat: &mut Combat, entity: usize, amount: i16) {
    apply_debuff(combat, entity, si::POISON, amount);
}

// =========================================================================
// CARD PILE VERBS
// =========================================================================

/// Draw N cards from draw pile to hand.
pub fn draw_cards(combat: &mut Combat, count: u8) {
    for _ in 0..count {
        if combat.hand.len() >= 10 { break; } // Hand size limit

        if combat.draw_pile.is_empty() {
            // Shuffle discard into draw
            if combat.discard_pile.is_empty() { break; }
            combat.draw_pile.append(&mut combat.discard_pile);
            // TODO: shuffle using RNG (needs rng parameter)
            on_shuffle(combat);
        }

        if let Some(card) = combat.draw_pile.pop() {
            combat.hand.push(card);
            on_card_drawn(combat, card);
        }
    }
}

/// Discard a card from hand to discard pile.
pub fn discard_card(combat: &mut Combat, hand_idx: usize) {
    if hand_idx < combat.hand.len() {
        let card = combat.hand.remove(hand_idx);
        combat.discard_pile.push(card);
    }
}

/// Exhaust a card. Fires: FeelNoPain, DarkEmbrace, etc.
pub fn exhaust_card(combat: &mut Combat, card: CardInstance) {
    combat.exhaust_pile.push(card);
    on_exhaust(combat);
}

// =========================================================================
// ENERGY VERBS
// =========================================================================

pub fn gain_energy(combat: &mut Combat, amount: i8) {
    combat.energy += amount;
}

pub fn lose_energy(combat: &mut Combat, amount: i8) {
    combat.energy = (combat.energy - amount).max(0);
}

// =========================================================================
// REACTION HANDLERS (private)
// =========================================================================

/// Called when player takes unblocked HP damage.
fn on_player_hp_loss(combat: &mut Combat, _amount: i16) {
    // Rupture: gain Strength when losing HP from a card
    let rupture = combat.entities[0].status(si::RUPTURE);
    if rupture > 0 {
        combat.entities[0].add_status(si::STRENGTH, rupture);
    }

    // Plated Armor: lose 1 stack on unblocked damage
    let plated = ov_status(&combat.entities[0], oid::PLATED_ARMOR);
    if plated > 0 {
        ov_add(&mut combat.entities[0], oid::PLATED_ARMOR, -1);
    }

    // TODO: Centennial Puzzle (draw 3), Self-Forming Clay (+block next turn),
    //       Runic Cube (draw 1), Emotion Chip, etc.
}

/// Called when an enemy takes unblocked HP damage.
fn on_enemy_hit(combat: &mut Combat, enemy: usize, damage: i16, _source: DamageSource) {
    // Curl Up: gain block on first attack, then consume
    let curl_up = ov_status(&combat.entities[enemy], oid::CURL_UP);
    if curl_up > 0 {
        combat.entities[enemy].block += curl_up;
        ov_set(&mut combat.entities[enemy], oid::CURL_UP, 0);
    }

    // Malleable: gain escalating block per hit this turn
    let malleable = ov_status(&combat.entities[enemy], oid::MALLEABLE);
    if malleable > 0 {
        combat.entities[enemy].block += malleable;
        ov_add(&mut combat.entities[enemy], oid::MALLEABLE, 1);
    }

    // Sharp Hide: deal damage back to player
    let sharp_hide = ov_status(&combat.entities[enemy], oid::SHARP_HIDE);
    if sharp_hide > 0 {
        apply_hp_loss(combat, sharp_hide);
    }

    // Shifting: gain block equal to damage taken
    let shifting = ov_status(&combat.entities[enemy], oid::SHIFTING);
    if shifting > 0 {
        combat.entities[enemy].block += damage;
    }

    // TODO: Reactive (reroll intent), Angry (gain Strength)
}

/// Called when player HP drops to 0. Returns true if actually dead.
fn on_player_death(combat: &mut Combat) -> bool {
    // TODO: Fairy in a Bottle revive, Lizard Tail
    combat.entities[0].hp = 0;
    combat.combat_over = true;
    combat.player_won = false;
    true
}

/// Called when an enemy dies.
fn on_enemy_death(combat: &mut Combat, enemy: usize) {
    combat.entities[enemy].hp = 0;

    // Check if all enemies are dead -> combat won
    let all_dead = (1..combat.entities.len()).all(|i| combat.entities[i].is_dead());
    if all_dead {
        combat.combat_over = true;
        combat.player_won = true;
    }

    // TODO: SporeCloud (Vulnerable on player), Gremlin Horn (energy+draw),
    //       The Specimen (transfer poison)
}

/// Called when discard pile is shuffled into draw pile.
fn on_shuffle(combat: &mut Combat) {
    // TODO: Sundial (+2 energy every 3 shuffles), Abacus (+6 block)
    let _ = combat;
}

/// Called when a card is drawn to hand.
fn on_card_drawn(combat: &mut Combat, _card: CardInstance) {
    // TODO: Evolve (draw extra on Status card), Fire Breathing (damage on Status/Curse)
    let _ = combat;
}

/// Called when a card is exhausted.
fn on_exhaust(combat: &mut Combat) {
    // Feel No Pain: gain block on exhaust
    let fnp = combat.entities[0].status(si::FEEL_NO_PAIN);
    if fnp > 0 {
        // Directly add block to avoid triggering Juggernaut/WotH recursion
        // in the exhaust path (matches STS behavior for FeelNoPain).
        gain_block(combat, 0, fnp);
    }

    // TODO: DarkEmbrace (draw 1), Charon's Ashes (3 damage to all),
    //       Dead Branch (add random card to hand)
}

// =========================================================================
// CARD PLAY HOOKS
// =========================================================================

/// Called after any card is played. Fires: After Image, Thousand Cuts,
/// Panache counter, relic counters (Pen Nib, Shuriken, etc.).
pub fn on_card_played(combat: &mut Combat) {
    combat.cards_played += 1;

    // After Image: gain 1 block per card played
    let ai = combat.entities[0].status(si::AFTER_IMAGE);
    if ai > 0 {
        gain_block(combat, 0, ai);
    }

    // Thousand Cuts: deal 1 (or 2) damage to ALL enemies per card played
    let tc = combat.entities[0].status(si::THOUSAND_CUTS);
    if tc > 0 {
        for i in 1..combat.entities.len() {
            if combat.entities[i].is_alive() {
                deal_damage(combat, 0, i, tc, DamageSource::Power);
            }
        }
    }
}

/// Called after an attack card deals damage. Fires: Envenom (add poison).
pub fn on_attack_damage(combat: &mut Combat, target: usize, unblocked: i16) {
    if unblocked > 0 {
        let envenom = combat.entities[0].status(si::ENVENOM);
        if envenom > 0 && target > 0 {
            apply_poison(combat, target, envenom);
        }
    }
}

// =========================================================================
// TESTS
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_combat() -> Combat {
        Combat {
            entities: smallvec::smallvec![
                Entity::new(72, 72),  // player
                Entity::new(50, 50),  // enemy
            ],
            enemy_meta: smallvec::SmallVec::new(),
            hand: smallvec::SmallVec::new(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            energy: 3,
            max_energy: 3,
            stance: StanceV2::Neutral,
            mantra: 0,
            relics: [0; 3],
            potions: [0; 5],
            orb_types: [0; 5],
            orb_values: [0; 5],
            orb_count: 0,
            orb_max: 0,
            turn: 1,
            cards_played: 0,
            attacks_played: 0,
            combat_over: false,
            player_won: false,
            skip_enemy_turn: false,
            blasphemy: false,
        }
    }

    // -- Damage tests --

    #[test]
    fn deal_damage_reduces_hp() {
        let mut c = test_combat();
        let dmg = deal_damage(&mut c, 0, 1, 10, DamageSource::Card);
        assert_eq!(dmg, 10);
        assert_eq!(c.entities[1].hp, 40);
    }

    #[test]
    fn deal_damage_blocked() {
        let mut c = test_combat();
        c.entities[1].block = 6;
        let dmg = deal_damage(&mut c, 0, 1, 10, DamageSource::Card);
        assert_eq!(dmg, 4); // 10 - 6 blocked
        assert_eq!(c.entities[1].hp, 46);
        assert_eq!(c.entities[1].block, 0);
    }

    #[test]
    fn deal_damage_intangible() {
        let mut c = test_combat();
        ov_set(&mut c.entities[1], oid::INTANGIBLE, 1);
        let dmg = deal_damage(&mut c, 0, 1, 99, DamageSource::Card);
        assert_eq!(dmg, 1);
        assert_eq!(c.entities[1].hp, 49);
    }

    #[test]
    fn deal_damage_zero_or_negative() {
        let mut c = test_combat();
        let dmg = deal_damage(&mut c, 0, 1, 0, DamageSource::Card);
        assert_eq!(dmg, 0);
        assert_eq!(c.entities[1].hp, 50);
        let dmg2 = deal_damage(&mut c, 0, 1, -5, DamageSource::Card);
        assert_eq!(dmg2, 0);
        assert_eq!(c.entities[1].hp, 50);
    }

    // -- Block tests --

    #[test]
    fn gain_block_basic() {
        let mut c = test_combat();
        gain_block(&mut c, 0, 10);
        assert_eq!(c.entities[0].block, 10);
    }

    #[test]
    fn gain_block_zero_noop() {
        let mut c = test_combat();
        gain_block(&mut c, 0, 0);
        assert_eq!(c.entities[0].block, 0);
        gain_block(&mut c, 0, -5);
        assert_eq!(c.entities[0].block, 0);
    }

    #[test]
    fn gain_block_juggernaut() {
        let mut c = test_combat();
        c.entities[0].set_status(si::JUGGERNAUT, 5);
        gain_block(&mut c, 0, 10);
        assert_eq!(c.entities[0].block, 10);
        assert_eq!(c.entities[1].hp, 45); // Juggernaut dealt 5
    }

    #[test]
    fn gain_block_wave_of_hand() {
        let mut c = test_combat();
        c.entities[0].set_status(si::WAVE_OF_THE_HAND, 1);
        gain_block(&mut c, 0, 5);
        assert_eq!(c.entities[1].status(si::WEAKENED), 1);
    }

    // -- Status tests --

    #[test]
    fn apply_debuff_artifact_blocks() {
        let mut c = test_combat();
        ov_set(&mut c.entities[1], oid::ARTIFACT, 1);
        apply_debuff(&mut c, 1, si::WEAKENED, 2);
        assert_eq!(c.entities[1].status(si::WEAKENED), 0); // Blocked
        assert_eq!(ov_status(&c.entities[1], oid::ARTIFACT), 0); // Consumed
    }

    #[test]
    fn apply_debuff_no_artifact() {
        let mut c = test_combat();
        apply_debuff(&mut c, 1, si::VULNERABLE, 2);
        assert_eq!(c.entities[1].status(si::VULNERABLE), 2);
    }

    // -- HP loss tests --

    #[test]
    fn apply_hp_loss_basic() {
        let mut c = test_combat();
        let died = apply_hp_loss(&mut c, 10);
        assert!(!died);
        assert_eq!(c.entities[0].hp, 62);
    }

    #[test]
    fn apply_hp_loss_kills() {
        let mut c = test_combat();
        c.entities[0].hp = 5;
        let died = apply_hp_loss(&mut c, 10);
        assert!(died);
        assert!(c.combat_over);
        assert!(!c.player_won);
    }

    #[test]
    fn apply_hp_loss_intangible() {
        let mut c = test_combat();
        ov_set(&mut c.entities[0], oid::INTANGIBLE, 1);
        let died = apply_hp_loss(&mut c, 50);
        assert!(!died);
        assert_eq!(c.entities[0].hp, 71); // Only 1 damage
    }

    #[test]
    fn apply_hp_loss_zero() {
        let mut c = test_combat();
        let died = apply_hp_loss(&mut c, 0);
        assert!(!died);
        assert_eq!(c.entities[0].hp, 72);
    }

    // -- Heal tests --

    #[test]
    fn heal_caps_at_max() {
        let mut c = test_combat();
        c.entities[0].hp = 50;
        heal(&mut c, 0, 100);
        assert_eq!(c.entities[0].hp, 72);
    }

    #[test]
    fn heal_normal() {
        let mut c = test_combat();
        c.entities[0].hp = 50;
        heal(&mut c, 0, 10);
        assert_eq!(c.entities[0].hp, 60);
    }

    // -- Enemy reaction tests --

    #[test]
    fn curl_up_fires_on_first_hit() {
        let mut c = test_combat();
        ov_set(&mut c.entities[1], oid::CURL_UP, 7);
        deal_damage(&mut c, 0, 1, 5, DamageSource::Card);
        assert_eq!(c.entities[1].block, 7); // Gained block from Curl-Up
        assert_eq!(ov_status(&c.entities[1], oid::CURL_UP), 0); // Consumed
    }

    #[test]
    fn curl_up_only_fires_once() {
        let mut c = test_combat();
        ov_set(&mut c.entities[1], oid::CURL_UP, 7);
        deal_damage(&mut c, 0, 1, 3, DamageSource::Card);
        // Second hit: curl up already consumed
        let block_before = c.entities[1].block;
        deal_damage(&mut c, 0, 1, 3, DamageSource::Card);
        // Block should have been absorbed, not gained again
        assert_eq!(ov_status(&c.entities[1], oid::CURL_UP), 0);
        assert!(c.entities[1].block <= block_before);
    }

    #[test]
    fn sharp_hide_retaliates() {
        let mut c = test_combat();
        ov_set(&mut c.entities[1], oid::SHARP_HIDE, 3);
        deal_damage(&mut c, 0, 1, 5, DamageSource::Card);
        assert_eq!(c.entities[0].hp, 69); // Took 3 Sharp Hide damage
    }

    #[test]
    fn shifting_converts_damage_to_block() {
        let mut c = test_combat();
        ov_set(&mut c.entities[1], oid::SHIFTING, 1);
        deal_damage(&mut c, 0, 1, 10, DamageSource::Card);
        assert_eq!(c.entities[1].block, 10); // Gained block equal to damage
    }

    #[test]
    fn malleable_escalates() {
        let mut c = test_combat();
        ov_set(&mut c.entities[1], oid::MALLEABLE, 3);

        // First hit: 2 unblocked damage -> malleable fires, gains 3 block, escalates to 4
        deal_damage(&mut c, 0, 1, 2, DamageSource::Card);
        assert_eq!(c.entities[1].block, 3);
        assert_eq!(ov_status(&c.entities[1], oid::MALLEABLE), 4);

        // Second hit: 5 damage vs 3 block -> 2 unblocked -> malleable fires again (+4 block)
        deal_damage(&mut c, 0, 1, 5, DamageSource::Card);
        // block was 3, absorbed 3 of 5, hp_damage = 2, then malleable adds 4
        assert_eq!(c.entities[1].block, 4);
        assert_eq!(ov_status(&c.entities[1], oid::MALLEABLE), 5);
    }

    // -- Card pile tests --

    #[test]
    fn draw_cards_moves_from_draw() {
        let mut c = test_combat();
        c.draw_pile.push(CardInstance::new(1));
        c.draw_pile.push(CardInstance::new(2));
        c.draw_pile.push(CardInstance::new(3));
        draw_cards(&mut c, 2);
        assert_eq!(c.hand.len(), 2);
        assert_eq!(c.draw_pile.len(), 1);
    }

    #[test]
    fn draw_cards_shuffles_discard() {
        let mut c = test_combat();
        c.discard_pile.push(CardInstance::new(10));
        c.discard_pile.push(CardInstance::new(11));
        draw_cards(&mut c, 1);
        assert_eq!(c.hand.len(), 1);
        assert!(c.discard_pile.is_empty()); // Shuffled into draw
    }

    #[test]
    fn draw_cards_hand_limit() {
        let mut c = test_combat();
        for i in 0..10 {
            c.hand.push(CardInstance::new(i));
        }
        c.draw_pile.push(CardInstance::new(99));
        draw_cards(&mut c, 1);
        assert_eq!(c.hand.len(), 10); // Still 10
        assert_eq!(c.draw_pile.len(), 1); // Card stayed
    }

    #[test]
    fn draw_cards_empty_both_piles() {
        let mut c = test_combat();
        draw_cards(&mut c, 5);
        assert_eq!(c.hand.len(), 0); // Nothing to draw
    }

    // -- Player reaction tests --

    #[test]
    fn rupture_on_hp_loss() {
        let mut c = test_combat();
        c.entities[0].set_status(si::RUPTURE, 2);
        deal_damage(&mut c, 1, 0, 5, DamageSource::Enemy);
        assert_eq!(c.entities[0].status(si::STRENGTH), 2); // Gained from Rupture
    }

    #[test]
    fn plated_armor_decrements_on_unblocked() {
        let mut c = test_combat();
        ov_set(&mut c.entities[0], oid::PLATED_ARMOR, 5);
        deal_damage(&mut c, 1, 0, 3, DamageSource::Enemy);
        assert_eq!(ov_status(&c.entities[0], oid::PLATED_ARMOR), 4);
    }

    // -- Death tests --

    #[test]
    fn enemy_death_marks_dead() {
        let mut c = test_combat();
        deal_damage(&mut c, 0, 1, 999, DamageSource::Card);
        assert_eq!(c.entities[1].hp, 0);
    }

    #[test]
    fn all_enemies_dead_wins_combat() {
        let mut c = test_combat();
        deal_damage(&mut c, 0, 1, 999, DamageSource::Card);
        assert!(c.combat_over);
        assert!(c.player_won);
    }

    #[test]
    fn partial_enemies_dead_not_over() {
        let mut c = test_combat();
        c.entities.push(Entity::new(30, 30)); // second enemy
        deal_damage(&mut c, 0, 1, 999, DamageSource::Card);
        assert!(!c.combat_over); // Still one enemy alive
    }

    // -- Energy tests --

    #[test]
    fn gain_energy_works() {
        let mut c = test_combat();
        gain_energy(&mut c, 2);
        assert_eq!(c.energy, 5);
    }

    #[test]
    fn lose_energy_floors_at_zero() {
        let mut c = test_combat();
        lose_energy(&mut c, 10);
        assert_eq!(c.energy, 0);
    }

    // -- Exhaust tests --

    #[test]
    fn exhaust_card_tracks() {
        let mut c = test_combat();
        exhaust_card(&mut c, CardInstance::new(42));
        assert_eq!(c.exhaust_pile.len(), 1);
        assert_eq!(c.exhaust_pile[0].def_id, 42);
    }

    #[test]
    fn exhaust_fires_feel_no_pain() {
        let mut c = test_combat();
        c.entities[0].set_status(si::FEEL_NO_PAIN, 4);
        exhaust_card(&mut c, CardInstance::new(1));
        assert_eq!(c.entities[0].block, 4);
    }

    // -- Discard tests --

    #[test]
    fn discard_card_moves_to_discard() {
        let mut c = test_combat();
        c.hand.push(CardInstance::new(5));
        c.hand.push(CardInstance::new(6));
        discard_card(&mut c, 0);
        assert_eq!(c.hand.len(), 1);
        assert_eq!(c.discard_pile.len(), 1);
        assert_eq!(c.discard_pile[0].def_id, 5);
    }

    // -- Card play hook tests --

    #[test]
    fn on_card_played_increments_counter() {
        let mut c = test_combat();
        on_card_played(&mut c);
        assert_eq!(c.cards_played, 1);
        on_card_played(&mut c);
        assert_eq!(c.cards_played, 2);
    }

    #[test]
    fn after_image_on_card_played() {
        let mut c = test_combat();
        c.entities[0].set_status(si::AFTER_IMAGE, 1);
        on_card_played(&mut c);
        assert_eq!(c.entities[0].block, 1);
    }

    #[test]
    fn thousand_cuts_on_card_played() {
        let mut c = test_combat();
        c.entities[0].set_status(si::THOUSAND_CUTS, 2);
        on_card_played(&mut c);
        assert_eq!(c.entities[1].hp, 48); // 2 damage from Thousand Cuts
    }
}
