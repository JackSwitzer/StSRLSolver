use crate::state::EntityState;
use super::debuffs::{decrement_debuffs, decrement_status, apply_lose_strength, apply_lose_dexterity, apply_wraith_form, decrement_intangible, decrement_blur, decrement_lock_on};
use super::enemy_powers::{apply_regeneration, reset_slow};
use crate::status_keys::sk;

// Buff-related power trigger functions


// ===========================================================================
// Trigger Dispatch Functions
//
// These are the core functions called by the engine at the appropriate moments.
// They check all relevant powers on the entity and apply effects.
// ===========================================================================

// ---------------------------------------------------------------------------
// Block Decay — checks Barricade, Blur, Calipers
// ---------------------------------------------------------------------------

/// Returns true if block should NOT be removed at start of turn.
/// Barricade prevents all block loss; Blur prevents for its duration.
pub fn should_retain_block(entity: &EntityState) -> bool {
    entity.status(sk::BARRICADE) > 0 || entity.status(sk::BLUR) > 0
}

/// Calculate block retained through Calipers (keep up to 15).
/// Returns the block value after decay.

pub fn apply_block_decay(entity: &EntityState, has_calipers: bool) -> i32 {
    if should_retain_block(entity) {
        return entity.block;
    }
    if has_calipers {
        return (entity.block - 15).max(0).min(entity.block).max(0);
    }
    0
}

// ---------------------------------------------------------------------------
// Decrement turn-based debuffs at end of round
// ---------------------------------------------------------------------------

/// Decrement turn-based debuffs at end of round.
/// Matches the atEndOfRound power trigger in Python.
///
/// Debuffs that tick down: Weakened, Vulnerable, Frail.

pub fn apply_metallicize(entity: &mut EntityState) {
    let metallicize = entity.status(sk::METALLICIZE);
    if metallicize > 0 {
        entity.block += metallicize;
    }
}

/// Apply Plated Armor block gain at end of turn.

pub fn apply_plated_armor(entity: &mut EntityState) {
    let plated = entity.status(sk::PLATED_ARMOR);
    if plated > 0 {
        entity.block += plated;
    }
}

/// Apply Ritual strength gain at start of enemy turn (not first turn).

pub fn remove_flame_barrier(entity: &mut EntityState) {
    entity.set_status(sk::FLAME_BARRIER, 0);
}

/// WrathNextTurn: enter Wrath at start of next turn. Returns true if should enter Wrath.

pub fn check_wrath_next_turn(entity: &mut EntityState) -> bool {
    let wrath = entity.status(sk::WRATH_NEXT_TURN);
    if wrath > 0 {
        entity.set_status(sk::WRATH_NEXT_TURN, 0);
        return true;
    }
    false
}

/// WraithForm: lose N Dexterity at start of turn.

pub fn apply_demon_form(entity: &mut EntityState) {
    let demon_form = entity.status(sk::DEMON_FORM);
    if demon_form > 0 {
        entity.add_status(sk::STRENGTH, demon_form);
    }
}

/// Berserk: gain N energy at start of turn. Returns energy to add.

pub fn apply_berserk(entity: &EntityState) -> i32 {
    entity.status(sk::BERSERK)
}

/// Noxious Fumes: returns the amount of poison to apply to all enemies.

pub fn get_noxious_fumes_amount(entity: &EntityState) -> i32 {
    entity.status(sk::NOXIOUS_FUMES)
}

/// Brutality: returns the amount of cards to draw (and HP to lose).

pub fn get_brutality_amount(entity: &EntityState) -> i32 {
    entity.status(sk::BRUTALITY)
}

/// DrawCardNextTurn: returns the number of extra cards to draw, then removes the power.

pub fn consume_draw_card_next_turn(entity: &mut EntityState) -> i32 {
    let amount = entity.status(sk::DRAW_CARD);
    if amount > 0 {
        entity.set_status(sk::DRAW_CARD, 0);
    }
    amount
}

/// NextTurnBlock: returns the amount of block to gain, then removes the power.

pub fn consume_next_turn_block(entity: &mut EntityState) -> i32 {
    let amount = entity.status(sk::NEXT_TURN_BLOCK);
    if amount > 0 {
        entity.set_status(sk::NEXT_TURN_BLOCK, 0);
    }
    amount
}

/// Energized: returns energy to gain at start of turn, then removes the power.

pub fn consume_energized(entity: &mut EntityState) -> i32 {
    let amount = entity.status(sk::ENERGIZED);
    if amount > 0 {
        entity.set_status(sk::ENERGIZED, 0);
    }
    amount
}

/// Draw power: permanent +draw per turn.

pub fn get_extra_draw(entity: &EntityState) -> i32 {
    entity.status(sk::DRAW)
}

/// EnergyDown: returns energy to lose at start of turn.

pub fn get_energy_down(entity: &EntityState) -> i32 {
    entity.status(sk::ENERGY_DOWN)
}

/// BattleHymn: returns amount of Smites to add to hand.

pub fn get_battle_hymn_amount(entity: &EntityState) -> i32 {
    entity.status(sk::BATTLE_HYMN)
}

/// Devotion: returns amount of Mantra to gain.

pub fn get_devotion_amount(entity: &EntityState) -> i32 {
    entity.status(sk::DEVOTION)
}

/// InfiniteBlades: returns number of Shivs to add (always 1 per stack).

pub fn get_infinite_blades(entity: &EntityState) -> i32 {
    let amount = entity.status(sk::INFINITE_BLADES);
    if amount > 0 { 1 } else { 0 }
}

// ---------------------------------------------------------------------------
// On-use-card triggers
// ---------------------------------------------------------------------------

/// AfterImage: returns block to gain per card played.

pub fn get_after_image_block(entity: &EntityState) -> i32 {
    entity.status(sk::AFTER_IMAGE)
}

/// A Thousand Cuts: returns damage to deal to ALL enemies per card played.

pub fn get_thousand_cuts_damage(entity: &EntityState) -> i32 {
    entity.status(sk::THOUSAND_CUTS)
}

/// Rage: returns block to gain when playing an Attack.

pub fn get_rage_block(entity: &EntityState) -> i32 {
    entity.status(sk::RAGE)
}

/// BeatOfDeath: returns damage to deal to player per card played.

pub fn check_panache(entity: &mut EntityState) -> i32 {
    // Panache stores remaining count until trigger (starts at 5, decrements)
    // We use a secondary counter approach: sk::PANACHE_COUNT
    if entity.status(sk::PANACHE) <= 0 {
        return 0;
    }
    let count = entity.status(sk::PANACHE_COUNT) + 1;
    if count >= 5 {
        entity.set_status(sk::PANACHE_COUNT, 0);
        entity.status(sk::PANACHE)
    } else {
        entity.set_status(sk::PANACHE_COUNT, count);
        0
    }
}

/// DoubleTap: returns true if the next Attack should be played twice.
/// Decrements the counter.

pub fn consume_double_tap(entity: &mut EntityState) -> bool {
    let dt = entity.status(sk::DOUBLE_TAP);
    if dt > 0 {
        entity.set_status(sk::DOUBLE_TAP, dt - 1);
        return true;
    }
    false
}

/// Burst: returns true if the next Skill should be played twice.
/// Decrements the counter.

pub fn consume_burst(entity: &mut EntityState) -> bool {
    let b = entity.status(sk::BURST);
    if b > 0 {
        entity.set_status(sk::BURST, b - 1);
        return true;
    }
    false
}

/// Heatsink: returns cards to draw when playing a Power card.

pub fn get_heatsink_draw(entity: &EntityState) -> i32 {
    entity.status(sk::HEATSINK)
}

/// Storm: returns true if should channel Lightning when playing a Power.

pub fn should_storm_channel(entity: &EntityState) -> bool {
    entity.status(sk::STORM) > 0
}

/// Forcefield (Automaton): lose Block per card played.
/// Returns true if power is present.

pub fn check_forcefield(entity: &mut EntityState) -> bool {
    let ff = entity.status(sk::FORCEFIELD);
    if ff > 0 {
        entity.add_status(sk::FORCEFIELD, -1);
        return true;
    }
    false
}

/// SkillBurn: returns damage to deal to player when they play a Skill.

pub fn get_skill_burn_damage(entity: &EntityState) -> i32 {
    entity.status(sk::SKILL_BURN)
}

// ---------------------------------------------------------------------------
// On-attacked / on-damaged triggers
// ---------------------------------------------------------------------------

/// Thorns: returns damage to deal back to attacker when hit.

pub fn get_thorns_damage(entity: &EntityState) -> i32 {
    entity.status(sk::THORNS)
}

/// Flame Barrier: returns damage to deal back to attacker when hit.

pub fn get_flame_barrier_damage(entity: &EntityState) -> i32 {
    entity.status(sk::FLAME_BARRIER)
}

/// Plated Armor: decrement by 1 when taking unblocked damage.

pub fn decrement_plated_armor_on_hit(entity: &mut EntityState) {
    let plated = entity.status(sk::PLATED_ARMOR);
    if plated > 0 {
        entity.set_status(sk::PLATED_ARMOR, plated - 1);
    }
}

/// Buffer: returns true if damage should be negated (reduces buffer by 1).

pub fn check_buffer(entity: &mut EntityState) -> bool {
    let buffer = entity.status(sk::BUFFER);
    if buffer > 0 {
        entity.set_status(sk::BUFFER, buffer - 1);
        return true;
    }
    false
}

/// Angry: gain Strength when taking damage.

pub fn get_envenom_amount(entity: &EntityState) -> i32 {
    entity.status(sk::ENVENOM)
}

/// Curiosity: gain Strength when player plays a Power.

pub fn apply_rupture(entity: &mut EntityState) {
    let rupture = entity.status(sk::RUPTURE);
    if rupture > 0 {
        entity.add_status(sk::STRENGTH, rupture);
    }
}

/// StaticDischarge: returns number of Lightning orbs to channel when taking damage.

pub fn get_static_discharge(entity: &EntityState) -> i32 {
    entity.status(sk::STATIC_DISCHARGE)
}

// ---------------------------------------------------------------------------
// On-exhaust triggers
// ---------------------------------------------------------------------------

/// DarkEmbrace: returns cards to draw per exhaust.

pub fn get_dark_embrace_draw(entity: &EntityState) -> i32 {
    entity.status(sk::DARK_EMBRACE)
}

/// FeelNoPain: returns block to gain per exhaust.

pub fn get_feel_no_pain_block(entity: &EntityState) -> i32 {
    entity.status(sk::FEEL_NO_PAIN)
}

// ---------------------------------------------------------------------------
// On-card-draw triggers
// ---------------------------------------------------------------------------

/// Evolve: returns cards to draw when drawing a Status card.

pub fn get_evolve_draw(entity: &EntityState) -> i32 {
    entity.status(sk::EVOLVE)
}

/// FireBreathing: returns damage to deal to all enemies when drawing Status/Curse.

pub fn get_fire_breathing_damage(entity: &EntityState) -> i32 {
    entity.status(sk::FIRE_BREATHING)
}

// ---------------------------------------------------------------------------
// On-change-stance triggers
// ---------------------------------------------------------------------------

/// MentalFortress: returns block to gain on ANY stance change.

pub fn get_mental_fortress_block(entity: &EntityState) -> i32 {
    entity.status(sk::MENTAL_FORTRESS)
}

/// Rushdown: returns cards to draw when entering Wrath.

pub fn get_rushdown_draw(entity: &EntityState) -> i32 {
    entity.status(sk::RUSHDOWN)
}

/// Nirvana: returns block to gain when scrying.

pub fn get_nirvana_block(entity: &EntityState) -> i32 {
    entity.status(sk::NIRVANA)
}

// ---------------------------------------------------------------------------
// On-gained-block triggers
// ---------------------------------------------------------------------------

/// Juggernaut: returns damage to deal to random enemy when gaining block.

pub fn get_juggernaut_damage(entity: &EntityState) -> i32 {
    entity.status(sk::JUGGERNAUT)
}

/// WaveOfTheHand: returns Weak amount to apply when gaining block.

pub fn get_wave_of_the_hand_weak(entity: &EntityState) -> i32 {
    entity.status(sk::WAVE_OF_THE_HAND)
}

// ---------------------------------------------------------------------------
// Damage modification triggers
// ---------------------------------------------------------------------------

/// Modify outgoing damage based on powers.
/// Called during damage calculation for attacks.

pub fn modify_damage_give(entity: &EntityState, damage: f64, _is_attack: bool) -> f64 {
    let mut d = damage;

    // DoubleDamage (Phantasmal Killer active)
    if entity.status(sk::DOUBLE_DAMAGE) > 0 {
        d *= 2.0;
    }

    // Pen Nib is handled separately in engine (relic counter)

    d
}

/// Modify incoming damage based on defender's powers.
/// Returns modified damage value.

pub fn modify_block(entity: &EntityState, block: f64) -> f64 {
    // NoBlock: can't gain block
    if entity.status(sk::NO_BLOCK) > 0 {
        return 0.0;
    }

    // Dexterity is handled in calculate_block() directly
    // Frail is handled in calculate_block() directly

    block
}

// ---------------------------------------------------------------------------
// On-heal triggers
// ---------------------------------------------------------------------------

/// Modify heal amount. Returns final heal amount.

pub fn modify_heal(entity: &EntityState, heal: i32) -> i32 {
    // No power modifies heal in base game except Mark of the Bloom (relic)
    let _ = entity;
    heal
}

// ---------------------------------------------------------------------------
// End-of-round triggers
// ---------------------------------------------------------------------------

/// Reset Slow stacks at end of round.

pub fn get_combust_effect(entity: &EntityState) -> (i32, i32) {
    let combust = entity.status(sk::COMBUST);
    if combust > 0 {
        (1, combust)
    } else {
        (0, 0)
    }
}

// ---------------------------------------------------------------------------
// Omega end-of-turn
// ---------------------------------------------------------------------------

/// Omega: returns damage to deal to ALL enemies at end of turn.

pub fn get_omega_damage(entity: &EntityState) -> i32 {
    entity.status(sk::OMEGA)
}

// ---------------------------------------------------------------------------
// LikeWater end-of-turn
// ---------------------------------------------------------------------------

/// LikeWater: returns block to gain if in Calm stance.

pub fn get_like_water_block(entity: &EntityState) -> i32 {
    entity.status(sk::LIKE_WATER)
}

// ---------------------------------------------------------------------------
// Regeneration end-of-turn
// ---------------------------------------------------------------------------

/// Regeneration: heal and decrement. Returns HP to heal.

pub fn remove_rage_end_of_turn(entity: &mut EntityState) {
    entity.set_status(sk::RAGE, 0);
}

// ---------------------------------------------------------------------------
// DoubleDamage consumption
// ---------------------------------------------------------------------------

/// DoubleDamage: consumed after playing an Attack.

pub fn consume_double_damage(entity: &mut EntityState) {
    if entity.status(sk::DOUBLE_DAMAGE) > 0 {
        entity.set_status(sk::DOUBLE_DAMAGE, 0);
    }
}

// ---------------------------------------------------------------------------
// On-death triggers
// ---------------------------------------------------------------------------

/// SporeCloud: returns Vulnerable amount to apply to player when this enemy dies.

pub fn has_corruption(entity: &EntityState) -> bool {
    entity.status(sk::CORRUPTION) > 0
}

// ---------------------------------------------------------------------------
// NoSkills — can't play Skills
// ---------------------------------------------------------------------------

/// Check if NoSkills prevents playing Skills.

pub fn has_no_skills(entity: &EntityState) -> bool {
    entity.status(sk::NO_SKILLS_POWER) > 0
}

// ---------------------------------------------------------------------------
// Confusion — randomize card costs
// ---------------------------------------------------------------------------

/// Check if Confusion is active.

pub fn has_confusion(entity: &EntityState) -> bool {
    entity.status(sk::CONFUSION) > 0
}

// ---------------------------------------------------------------------------
// NoDraw — can't draw cards
// ---------------------------------------------------------------------------

/// Check if NoDraw prevents card draw.

pub fn has_no_draw(entity: &EntityState) -> bool {
    entity.status(sk::NO_DRAW) > 0
}

// ---------------------------------------------------------------------------
// CannotChangeStance
// ---------------------------------------------------------------------------

/// Check if stance changes are blocked.

pub fn cannot_change_stance(entity: &EntityState) -> bool {
    entity.status(sk::CANNOT_CHANGE_STANCE) > 0
}

// ---------------------------------------------------------------------------
// FreeAttack — next Attack costs 0
// ---------------------------------------------------------------------------

/// Check and consume FreeAttack. Returns true if active.

pub fn consume_free_attack(entity: &mut EntityState) -> bool {
    let fa = entity.status(sk::FREE_ATTACK_POWER);
    if fa > 0 {
        entity.set_status(sk::FREE_ATTACK_POWER, fa - 1);
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Equilibrium — retain hand
// ---------------------------------------------------------------------------

/// Check if Equilibrium retains hand this turn.

pub fn has_equilibrium(entity: &EntityState) -> bool {
    entity.status(sk::EQUILIBRIUM) > 0
}

/// Decrement Equilibrium at end of turn.

pub fn decrement_equilibrium(entity: &mut EntityState) {
    decrement_status(entity, sk::EQUILIBRIUM);
}

// ---------------------------------------------------------------------------
// Study — shuffle Insight into draw pile
// ---------------------------------------------------------------------------

/// Study: returns number of Insights to add to draw pile.

pub fn get_study_insights(entity: &EntityState) -> i32 {
    entity.status(sk::STUDY)
}

// ---------------------------------------------------------------------------
// LiveForever — gain block at end of turn
// ---------------------------------------------------------------------------

/// LiveForever: returns block to gain at end of turn.

pub fn get_live_forever_block(entity: &EntityState) -> i32 {
    entity.status(sk::LIVE_FOREVER)
}

// ---------------------------------------------------------------------------
// Accuracy — bonus Shiv damage
// ---------------------------------------------------------------------------

/// Accuracy: returns bonus damage for Shiv cards.

pub fn get_accuracy_bonus(entity: &EntityState) -> i32 {
    entity.status(sk::ACCURACY)
}

// ---------------------------------------------------------------------------
// Mark — Pressure Points damage
// ---------------------------------------------------------------------------

/// Get current Mark amount on entity.

pub fn get_mark(entity: &EntityState) -> i32 {
    entity.status(sk::MARK)
}

// ---------------------------------------------------------------------------
// Deva Form — escalating energy
// ---------------------------------------------------------------------------

/// DevaForm energy tracking. Uses sk::DEVA_FORM_ENERGY for the escalating counter.
/// Returns energy to gain this turn.

pub fn apply_deva_form(entity: &mut EntityState) -> i32 {
    let deva = entity.status(sk::DEVA_FORM);
    if deva <= 0 {
        return 0;
    }
    let energy_counter = entity.status(sk::DEVA_FORM_ENERGY) + 1;
    entity.set_status(sk::DEVA_FORM_ENERGY, energy_counter);
    energy_counter
}

// ---------------------------------------------------------------------------
// Apply a debuff, respecting Artifact
// ---------------------------------------------------------------------------

/// Apply a debuff, respecting Artifact (blocks debuffs).
/// Returns true if the debuff was applied, false if blocked by Artifact.

pub fn should_die_end_of_turn(entity: &EntityState) -> bool {
    entity.status(sk::END_TURN_DEATH) > 0
}

// ---------------------------------------------------------------------------
// Comprehensive trigger dispatcher — aggregates all triggers for a phase
// ---------------------------------------------------------------------------

/// Results from start-of-turn power processing.
#[derive(Debug, Default)]
pub struct StartOfTurnResult {
    pub extra_energy: i32,
    pub extra_draw: i32,
    pub noxious_fumes_poison: i32,
    pub demon_form_strength: bool,
    pub brutality_draw: i32,
    pub block_from_next_turn: i32,
    pub enter_wrath: bool,
    pub battle_hymn_smites: i32,
    pub devotion_mantra: i32,
    pub infinite_blades: bool,
    pub draw_card_next_turn: i32,
    pub wraith_form_dex_loss: bool,
    pub berserk_energy: i32,
}

/// Process all start-of-turn power triggers for the player.
/// Returns a result struct with all effects to apply.

pub fn process_start_of_turn(entity: &mut EntityState) -> StartOfTurnResult {
    let mut result = StartOfTurnResult::default();

    // LoseStrength / LoseDexterity
    apply_lose_strength(entity);
    apply_lose_dexterity(entity);

    // Flame Barrier removal
    remove_flame_barrier(entity);

    // WraithForm: lose Dexterity
    let wraith = entity.status(sk::WRAITH_FORM);
    if wraith > 0 {
        apply_wraith_form(entity);
        result.wraith_form_dex_loss = true;
    }

    // Demon Form
    let demon = entity.status(sk::DEMON_FORM);
    if demon > 0 {
        apply_demon_form(entity);
        result.demon_form_strength = true;
    }

    // Berserk
    result.berserk_energy = apply_berserk(entity);

    // Noxious Fumes
    result.noxious_fumes_poison = get_noxious_fumes_amount(entity);

    // Brutality
    result.brutality_draw = get_brutality_amount(entity);

    // DrawCardNextTurn
    result.draw_card_next_turn = consume_draw_card_next_turn(entity);

    // NextTurnBlock
    result.block_from_next_turn = consume_next_turn_block(entity);

    // Energized
    result.extra_energy += consume_energized(entity);

    // EnergyDown
    result.extra_energy -= get_energy_down(entity);

    // WrathNextTurn
    result.enter_wrath = check_wrath_next_turn(entity);

    // BattleHymn
    result.battle_hymn_smites = get_battle_hymn_amount(entity);

    // Devotion
    result.devotion_mantra = get_devotion_amount(entity);

    // Infinite Blades
    result.infinite_blades = get_infinite_blades(entity) > 0;

    // Draw power (permanent)
    result.extra_draw = get_extra_draw(entity);

    // DevaForm
    let deva_energy = apply_deva_form(entity);
    result.extra_energy += deva_energy;

    result
}

/// Results from end-of-turn power processing.
#[derive(Debug, Default)]
pub struct EndOfTurnResult {
    pub metallicize_block: i32,
    pub plated_armor_block: i32,
    pub omega_damage: i32,
    pub like_water_block: i32,
    pub combust_hp_loss: i32,
    pub combust_damage: i32,
    pub regen_heal: i32,
    pub live_forever_block: i32,
    pub study_insights: i32,
    pub should_die: bool,
}

/// Process all end-of-turn power triggers for the player.

pub fn process_end_of_turn(entity: &mut EntityState, in_calm: bool) -> EndOfTurnResult {
    let mut result = EndOfTurnResult::default();

    // Metallicize
    result.metallicize_block = entity.status(sk::METALLICIZE);

    // Plated Armor
    result.plated_armor_block = entity.status(sk::PLATED_ARMOR);

    // Omega
    result.omega_damage = get_omega_damage(entity);

    // LikeWater (only if in Calm)
    if in_calm {
        result.like_water_block = get_like_water_block(entity);
    }

    // Combust
    let (hp_loss, dmg) = get_combust_effect(entity);
    result.combust_hp_loss = hp_loss;
    result.combust_damage = dmg;

    // Regeneration
    result.regen_heal = apply_regeneration(entity);

    // LiveForever
    result.live_forever_block = get_live_forever_block(entity);

    // Study
    result.study_insights = get_study_insights(entity);

    // EndTurnDeath
    result.should_die = should_die_end_of_turn(entity);

    // Remove Rage at end of turn
    remove_rage_end_of_turn(entity);

    // Decrement Equilibrium
    decrement_equilibrium(entity);

    // Decrement Intangible
    decrement_intangible(entity);

    result
}

/// Process end-of-round triggers (after all entities have taken turns).

pub fn process_end_of_round(entity: &mut EntityState) {
    // Debuff decrements
    decrement_debuffs(entity);

    // Blur
    decrement_blur(entity);

    // Lock-On
    decrement_lock_on(entity);

    // Slow reset
    reset_slow(entity);
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::powers::*;

    // -- Power registry tests --

    #[test]
    fn test_power_def_lookup() {
        let def = get_power_def("Strength").unwrap();
        assert_eq!(def.id, "Strength");
        assert!(def.can_go_negative);
        assert!(def.modify_damage_give);
        assert_eq!(def.power_type, PowerType::Buff);
    }

    #[test]
    fn test_power_def_debuff() {
        let def = get_power_def("Weakened").unwrap();
        assert_eq!(def.power_type, PowerType::Debuff);
        assert!(def.is_turn_based);
        assert!(def.on_end_of_round);
    }

    #[test]
    fn test_power_def_unknown() {
        assert!(get_power_def("NonexistentPower").is_none());
    }

    #[test]
    fn test_power_id_key_roundtrip() {
        assert_eq!(PowerId::Strength.key(), "Strength");
        assert_eq!(PowerId::Weakened.key(), "Weakened");
        assert_eq!(PowerId::DemonForm.key(), "DemonForm");
        assert_eq!(PowerId::MentalFortress.key(), "MentalFortress");
        assert_eq!(PowerId::Omega.key(), "Omega");
    }

    // -- Debuff decrement tests --

    #[test]
    fn test_decrement_debuffs() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Weakened", 2);
        entity.set_status("Vulnerable", 1);
        entity.set_status("Frail", 3);

        decrement_debuffs(&mut entity);

        assert_eq!(entity.status("Weakened"), 1);
        assert_eq!(entity.status("Vulnerable"), 0);
        assert_eq!(entity.status("Frail"), 2);
    }

    // -- Poison tests --

    #[test]
    fn test_tick_poison() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Poison", 5);

        let dmg = tick_poison(&mut entity);
        assert_eq!(dmg, 5);
        assert_eq!(entity.hp, 45);
        assert_eq!(entity.status("Poison"), 4);
    }

    #[test]
    fn test_tick_poison_removed_at_zero() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Poison", 1);

        let dmg = tick_poison(&mut entity);
        assert_eq!(dmg, 1);
        assert_eq!(entity.status("Poison"), 0);
        assert!(!entity.statuses.contains_key("Poison"));
    }

    // -- Metallicize / Plated Armor tests --

    #[test]
    fn test_metallicize() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Metallicize", 4);

        apply_metallicize(&mut entity);
        assert_eq!(entity.block, 4);
    }

    #[test]
    fn test_plated_armor() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("PlatedArmor", 6);

        apply_plated_armor(&mut entity);
        assert_eq!(entity.block, 6);
    }

    // -- Ritual tests --

    #[test]
    fn test_ritual() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Ritual", 3);

        apply_ritual(&mut entity);
        assert_eq!(entity.strength(), 3);

        // Second application stacks
        apply_ritual(&mut entity);
        assert_eq!(entity.strength(), 6);
    }

    // -- Artifact tests --

    #[test]
    fn test_artifact_blocks_debuff() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Artifact", 1);

        let applied = apply_debuff(&mut entity, "Weakened", 2);
        assert!(!applied);
        assert_eq!(entity.status("Weakened"), 0);
        assert_eq!(entity.status("Artifact"), 0);
    }

    #[test]
    fn test_debuff_without_artifact() {
        let mut entity = EntityState::new(50, 50);

        let applied = apply_debuff(&mut entity, "Weakened", 2);
        assert!(applied);
        assert_eq!(entity.status("Weakened"), 2);
    }

    // -- Block decay tests --

    #[test]
    fn test_barricade_retains_block() {
        let mut entity = EntityState::new(50, 50);
        entity.block = 10;
        entity.set_status("Barricade", 1);
        assert!(should_retain_block(&entity));
        assert_eq!(apply_block_decay(&entity, false), 10);
    }

    #[test]
    fn test_blur_retains_block() {
        let mut entity = EntityState::new(50, 50);
        entity.block = 10;
        entity.set_status("Blur", 1);
        assert!(should_retain_block(&entity));
    }

    #[test]
    fn test_calipers_retains_15() {
        let mut entity = EntityState::new(50, 50);
        entity.block = 20;
        assert_eq!(apply_block_decay(&entity, true), 5);
    }

    #[test]
    fn test_normal_block_decay() {
        let mut entity = EntityState::new(50, 50);
        entity.block = 10;
        assert_eq!(apply_block_decay(&entity, false), 0);
    }

    // -- Demon Form tests --

    #[test]
    fn test_demon_form() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("DemonForm", 3);

        apply_demon_form(&mut entity);
        assert_eq!(entity.strength(), 3);

        apply_demon_form(&mut entity);
        assert_eq!(entity.strength(), 6);
    }

    // -- Buffer tests --

    #[test]
    fn test_buffer_blocks_damage() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Buffer", 2);

        assert!(check_buffer(&mut entity));
        assert_eq!(entity.status("Buffer"), 1);

        assert!(check_buffer(&mut entity));
        assert_eq!(entity.status("Buffer"), 0);

        assert!(!check_buffer(&mut entity));
    }

    // -- Thorns tests --

    #[test]
    fn test_thorns_damage() {
        let entity = EntityState::new(50, 50);
        assert_eq!(get_thorns_damage(&entity), 0);

        let mut entity2 = EntityState::new(50, 50);
        entity2.set_status("Thorns", 3);
        assert_eq!(get_thorns_damage(&entity2), 3);
    }

    // -- Flame Barrier tests --

    #[test]
    fn test_flame_barrier() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("FlameBarrier", 7);

        assert_eq!(get_flame_barrier_damage(&entity), 7);

        remove_flame_barrier(&mut entity);
        assert_eq!(get_flame_barrier_damage(&entity), 0);
    }

    // -- After Image tests --

    #[test]
    fn test_after_image() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("AfterImage", 2);
        assert_eq!(get_after_image_block(&entity), 2);
    }

    // -- DoubleTap / Burst tests --

    #[test]
    fn test_double_tap() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("DoubleTap", 1);

        assert!(consume_double_tap(&mut entity));
        assert_eq!(entity.status("DoubleTap"), 0);
        assert!(!consume_double_tap(&mut entity));
    }

    #[test]
    fn test_burst() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Burst", 2);

        assert!(consume_burst(&mut entity));
        assert_eq!(entity.status("Burst"), 1);
        assert!(consume_burst(&mut entity));
        assert!(!consume_burst(&mut entity));
    }

    // -- TimeWarp tests --

    #[test]
    fn test_time_warp_countdown() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("TimeWarpActive", 1);

        for _ in 0..11 {
            assert!(!increment_time_warp(&mut entity));
        }
        assert!(increment_time_warp(&mut entity));
        assert_eq!(entity.status("TimeWarp"), 0); // resets
    }

    // -- Slow tests --

    #[test]
    fn test_slow_damage_modification() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Slow", 3);

        let modified = modify_damage_receive(&entity, 10.0);
        assert!((modified - 13.0).abs() < 0.01); // 10 * 1.3 = 13
    }

    // -- Invincible tests --

    #[test]
    fn test_invincible_cap() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Invincible", 200);

        assert_eq!(apply_invincible_cap(&mut entity, 50), 50);
        assert_eq!(entity.status("Invincible"), 150);

        assert_eq!(apply_invincible_cap(&mut entity, 200), 150);
        assert_eq!(entity.status("Invincible"), 0);
    }

    // -- ModeShift tests --

    #[test]
    fn test_mode_shift() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("ModeShift", 10);

        assert!(!apply_mode_shift_damage(&mut entity, 5));
        assert_eq!(entity.status("ModeShift"), 5);

        assert!(apply_mode_shift_damage(&mut entity, 5));
        assert_eq!(entity.status("ModeShift"), 0);
    }

    // -- Fading tests --

    #[test]
    fn test_fading_countdown() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Fading", 2);

        assert!(!decrement_fading(&mut entity));
        assert_eq!(entity.status("Fading"), 1);

        assert!(decrement_fading(&mut entity));
    }

    // -- Comprehensive trigger tests --

    #[test]
    fn test_process_start_of_turn() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("DemonForm", 2);
        entity.set_status("NoxiousFumes", 3);
        entity.set_status("Energized", 2);
        entity.set_status("LoseStrength", 1);
        entity.set_status("WraithForm", 1);
        entity.set_status("FlameBarrier", 5);

        let result = process_start_of_turn(&mut entity);

        // Demon Form adds Strength
        assert_eq!(entity.strength(), 1); // +2 from DemonForm, -1 from LoseStrength
        assert!(result.demon_form_strength);

        // Noxious Fumes
        assert_eq!(result.noxious_fumes_poison, 3);

        // Energized consumed
        assert_eq!(result.extra_energy, 2);
        assert_eq!(entity.status("Energized"), 0);

        // WraithForm
        assert!(result.wraith_form_dex_loss);
        assert_eq!(entity.dexterity(), -1);

        // Flame Barrier removed
        assert_eq!(entity.status("FlameBarrier"), 0);
    }

    #[test]
    fn test_process_end_of_turn() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Metallicize", 4);
        entity.set_status("PlatedArmor", 3);
        entity.set_status("Omega", 50);
        entity.set_status("Rage", 5);
        entity.set_status("Combust", 7);

        let result = process_end_of_turn(&mut entity, false);

        assert_eq!(result.metallicize_block, 4);
        assert_eq!(result.plated_armor_block, 3);
        assert_eq!(result.omega_damage, 50);
        assert_eq!(result.combust_hp_loss, 1);
        assert_eq!(result.combust_damage, 7);

        // Rage removed
        assert_eq!(entity.status("Rage"), 0);
    }

    #[test]
    fn test_like_water_in_calm() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("LikeWater", 5);

        let result_calm = process_end_of_turn(&mut entity, true);
        assert_eq!(result_calm.like_water_block, 5);

        let result_not_calm = process_end_of_turn(&mut entity, false);
        assert_eq!(result_not_calm.like_water_block, 0);
    }

    // -- Damage modification tests --

    #[test]
    fn test_double_damage_modifier() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("DoubleDamage", 1);

        let modified = modify_damage_give(&entity, 10.0, true);
        assert!((modified - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_intangible_caps_damage() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Intangible", 1);

        let modified = modify_damage_receive(&entity, 100.0);
        assert!((modified - 1.0).abs() < 0.01);
    }

    // -- Corruption / NoSkills / Confusion --

    #[test]
    fn test_corruption_flag() {
        let mut entity = EntityState::new(50, 50);
        assert!(!has_corruption(&entity));
        entity.set_status("Corruption", 1);
        assert!(has_corruption(&entity));
    }

    #[test]
    fn test_no_skills_flag() {
        let mut entity = EntityState::new(50, 50);
        assert!(!has_no_skills(&entity));
        entity.set_status("NoSkillsPower", 1);
        assert!(has_no_skills(&entity));
    }

    #[test]
    fn test_cannot_change_stance() {
        let mut entity = EntityState::new(50, 50);
        assert!(!cannot_change_stance(&entity));
        entity.set_status("CannotChangeStance", 1);
        assert!(cannot_change_stance(&entity));
    }

    // -- Panache tests --

    #[test]
    fn test_panache_triggers_every_5() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Panache", 10);

        for _ in 0..4 {
            assert_eq!(check_panache(&mut entity), 0);
        }
        assert_eq!(check_panache(&mut entity), 10);

        // Next cycle
        for _ in 0..4 {
            assert_eq!(check_panache(&mut entity), 0);
        }
        assert_eq!(check_panache(&mut entity), 10);
    }

    // -- Deva Form tests --

    #[test]
    fn test_deva_form_escalating() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("DevaForm", 1);

        assert_eq!(apply_deva_form(&mut entity), 1);
        assert_eq!(apply_deva_form(&mut entity), 2);
        assert_eq!(apply_deva_form(&mut entity), 3);
    }

    // -- Sadistic Nature tests --

    #[test]
    fn test_sadistic_on_debuff() {
        let mut entity = EntityState::new(50, 50);

        let (applied, sadistic_dmg) = apply_debuff_with_sadistic(&mut entity, "Weakened", 1, 5);
        assert!(applied);
        assert_eq!(sadistic_dmg, 5);
        assert_eq!(entity.status("Weakened"), 1);
    }

    #[test]
    fn test_sadistic_blocked_by_artifact() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Artifact", 1);

        let (applied, sadistic_dmg) = apply_debuff_with_sadistic(&mut entity, "Weakened", 1, 5);
        assert!(!applied);
        assert_eq!(sadistic_dmg, 0);
    }

    // -- Process end of round --

    #[test]
    fn test_process_end_of_round() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Weakened", 2);
        entity.set_status("Vulnerable", 1);
        entity.set_status("Blur", 1);
        entity.set_status("Slow", 5);
        entity.set_status("Lock-On", 2);

        process_end_of_round(&mut entity);

        assert_eq!(entity.status("Weakened"), 1);
        assert_eq!(entity.status("Vulnerable"), 0);
        assert_eq!(entity.status("Blur"), 0);
        assert_eq!(entity.status("Slow"), 0);
        assert_eq!(entity.status("Lock-On"), 1);
    }
}

