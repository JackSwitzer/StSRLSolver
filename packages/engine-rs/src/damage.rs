//! Damage and block calculation — mirrors packages/engine/calc/damage.py.
//!
//! Pure functions, no side effects. Optimized for millions of calls in MCTS.
//!
//! Calculation order (from Java AbstractCard.calculateCardDamage):
//! 1. Base damage
//! 2. Flat adds (Strength, Vigor)
//! 3. Attacker multipliers (Weak, Pen Nib, Double Damage)
//! 4. Stance multiplier (Wrath 2.0, Divinity 3.0)
//! 5. Defender multipliers (Vulnerable 1.5, Flight 0.5)
//! 6. Intangible cap (max 1)
//! 7. Floor to i32, minimum 0

// ---- Constants (match Python exactly) ----

pub const WEAK_MULT: f64 = 0.75;
pub const WEAK_MULT_PAPER_CRANE: f64 = 0.60;
pub const VULN_MULT: f64 = 1.50;
pub const VULN_MULT_ODD_MUSHROOM: f64 = 1.25;
pub const VULN_MULT_PAPER_FROG: f64 = 1.75;
pub const FRAIL_MULT: f64 = 0.75;
pub const WRATH_MULT: f64 = 2.0;
pub const DIVINITY_MULT: f64 = 3.0;

// ---- Outgoing Damage ----

/// Calculate final outgoing damage for an attack card.
///
/// Follows the exact Java order from AbstractCard.calculateCardDamage().
pub fn calculate_damage(
    base: i32,
    strength: i32,
    weak: bool,
    stance_mult: f64,
    vulnerable: bool,
    intangible: bool,
) -> i32 {
    // 1. Base + flat adds
    let mut damage = (base + strength) as f64;

    // 3. Attacker multipliers — Weak
    if weak {
        damage *= WEAK_MULT;
    }

    // 4. Stance multiplier
    damage *= stance_mult;

    // 5. Defender multipliers — Vulnerable
    if vulnerable {
        damage *= VULN_MULT;
    }

    // 6. Intangible cap
    if intangible && damage > 1.0 {
        damage = 1.0;
    }

    // 7. Floor to int, min 0
    (damage as i32).max(0)
}

/// Full-featured damage calculation with all modifier flags.
/// Used when relic/power context is available.
pub fn calculate_damage_full(
    base: i32,
    strength: i32,
    vigor: i32,
    weak: bool,
    weak_paper_crane: bool,
    pen_nib: bool,
    double_damage: bool,
    stance_mult: f64,
    vulnerable: bool,
    vuln_paper_frog: bool,
    flight: bool,
    intangible: bool,
) -> i32 {
    let mut damage = (base + strength + vigor) as f64;

    // Attacker multipliers
    if pen_nib {
        damage *= 2.0;
    }
    if double_damage {
        damage *= 2.0;
    }
    if weak {
        damage *= if weak_paper_crane {
            WEAK_MULT_PAPER_CRANE
        } else {
            WEAK_MULT
        };
    }

    // Stance
    damage *= stance_mult;

    // Defender multipliers
    if vulnerable {
        damage *= if vuln_paper_frog {
            VULN_MULT_PAPER_FROG
        } else {
            VULN_MULT
        };
    }
    if flight {
        damage *= 0.5;
    }

    // Intangible cap
    if intangible && damage > 1.0 {
        damage = 1.0;
    }

    (damage as i32).max(0)
}

// ---- Block Calculation ----

/// Calculate final block from a card.
///
/// Order from AbstractCard.applyPowersToBlock():
/// 1. Base block
/// 2. Add Dexterity (flat)
/// 3. Frail (multiplicative, 0.75)
/// 4. Floor to int, min 0
pub fn calculate_block(base: i32, dexterity: i32, frail: bool) -> i32 {
    let mut block = (base + dexterity) as f64;

    if frail {
        block *= FRAIL_MULT;
    }

    (block as i32).max(0)
}

// ---- Incoming Damage (enemy attack -> player) ----

/// Result of incoming damage calculation.
pub struct IncomingDamageResult {
    pub hp_loss: i32,
    pub block_remaining: i32,
}

/// Calculate HP loss and remaining block when player takes a hit.
///
/// Matches Java order:
/// 1. Apply Wrath multiplier (2x incoming if in Wrath)
/// 2. Apply Vulnerable
/// 3. Floor
/// 4. Intangible cap (max 1)
/// 5. Block absorption
/// 6. Torii (post-block damage 2-5 -> 1)
/// 7. Tungsten Rod (-1 HP loss)
pub fn calculate_incoming_damage(
    damage: i32,
    block: i32,
    is_wrath: bool,
    vulnerable: bool,
    intangible: bool,
    torii: bool,
    tungsten_rod: bool,
) -> IncomingDamageResult {
    let mut final_damage = damage as f64;

    // 1. Wrath incoming multiplier
    if is_wrath {
        final_damage *= WRATH_MULT;
    }

    // 2. Vulnerable
    if vulnerable {
        final_damage *= VULN_MULT;
    }

    // 3. Floor
    let mut final_damage_i = final_damage as i32;

    // 4. Intangible cap
    if intangible && final_damage_i > 1 {
        final_damage_i = 1;
    }

    // 5. Block absorption
    let blocked = block.min(final_damage_i);
    let mut hp_loss = final_damage_i - blocked;
    let block_remaining = block - blocked;

    // 6. Torii (2-5 unblocked -> 1)
    if torii && hp_loss >= 2 && hp_loss <= 5 {
        hp_loss = 1;
    }

    // 7. Tungsten Rod (-1)
    if tungsten_rod && hp_loss > 0 {
        hp_loss = (hp_loss - 1).max(0);
    }

    IncomingDamageResult {
        hp_loss,
        block_remaining,
    }
}

// ---- HP Loss (poison, self-damage — bypasses block) ----

/// Calculate actual HP loss for HP_LOSS damage type (poison, etc.).
/// Ignores block but affected by Intangible and Tungsten Rod.
pub fn apply_hp_loss(amount: i32, intangible: bool, tungsten_rod: bool) -> i32 {
    let mut hp_loss = amount;

    if intangible && hp_loss > 1 {
        hp_loss = 1;
    }

    if tungsten_rod && hp_loss > 0 {
        hp_loss = (hp_loss - 1).max(0);
    }

    hp_loss
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    // -- Outgoing damage tests --

    #[test]
    fn test_basic_damage() {
        assert_eq!(calculate_damage(6, 0, false, 1.0, false, false), 6);
    }

    #[test]
    fn test_damage_with_strength() {
        assert_eq!(calculate_damage(6, 3, false, 1.0, false, false), 9);
    }

    #[test]
    fn test_damage_with_weak() {
        // 10 * 0.75 = 7.5 -> 7
        assert_eq!(calculate_damage(10, 0, true, 1.0, false, false), 7);
    }

    #[test]
    fn test_damage_in_wrath() {
        assert_eq!(calculate_damage(6, 0, false, WRATH_MULT, false, false), 12);
    }

    #[test]
    fn test_damage_in_divinity() {
        assert_eq!(calculate_damage(6, 0, false, DIVINITY_MULT, false, false), 18);
    }

    #[test]
    fn test_damage_wrath_plus_vulnerable() {
        // 6 * 2.0 * 1.5 = 18
        assert_eq!(calculate_damage(6, 0, false, WRATH_MULT, true, false), 18);
    }

    #[test]
    fn test_damage_strength_wrath_vulnerable() {
        // (6+3) * 2.0 * 1.5 = 27
        assert_eq!(calculate_damage(6, 3, false, WRATH_MULT, true, false), 27);
    }

    #[test]
    fn test_damage_intangible() {
        assert_eq!(calculate_damage(100, 0, false, 1.0, false, true), 1);
    }

    #[test]
    fn test_damage_minimum_zero() {
        // Negative strength can drive damage below 0
        assert_eq!(calculate_damage(2, -5, false, 1.0, false, false), 0);
    }

    #[test]
    fn test_damage_full_pen_nib() {
        assert_eq!(
            calculate_damage_full(6, 0, 0, false, false, true, false, 1.0, false, false, false, false),
            12
        );
    }

    #[test]
    fn test_damage_full_vigor() {
        assert_eq!(
            calculate_damage_full(6, 3, 5, false, false, false, false, 1.0, false, false, false, false),
            14
        );
    }

    #[test]
    fn test_damage_full_flight() {
        // 10 * 0.5 = 5
        assert_eq!(
            calculate_damage_full(10, 0, 0, false, false, false, false, 1.0, false, false, true, false),
            5
        );
    }

    #[test]
    fn test_damage_full_paper_frog_vuln() {
        // 10 * 1.75 = 17.5 -> 17
        assert_eq!(
            calculate_damage_full(10, 0, 0, false, false, false, false, 1.0, true, true, false, false),
            17
        );
    }

    // -- Block tests --

    #[test]
    fn test_basic_block() {
        assert_eq!(calculate_block(5, 0, false), 5);
    }

    #[test]
    fn test_block_with_dexterity() {
        assert_eq!(calculate_block(5, 2, false), 7);
    }

    #[test]
    fn test_block_with_frail() {
        // 8 * 0.75 = 6
        assert_eq!(calculate_block(8, 0, true), 6);
    }

    #[test]
    fn test_block_dex_plus_frail() {
        // (5+2) * 0.75 = 5.25 -> 5
        assert_eq!(calculate_block(5, 2, true), 5);
    }

    #[test]
    fn test_block_negative_dex() {
        assert_eq!(calculate_block(5, -2, false), 3);
    }

    #[test]
    fn test_block_negative_dex_floored() {
        assert_eq!(calculate_block(5, -10, false), 0);
    }

    // -- Incoming damage tests --

    #[test]
    fn test_incoming_basic() {
        let r = calculate_incoming_damage(10, 5, false, false, false, false, false);
        assert_eq!(r.hp_loss, 5);
        assert_eq!(r.block_remaining, 0);
    }

    #[test]
    fn test_incoming_fully_blocked() {
        let r = calculate_incoming_damage(5, 10, false, false, false, false, false);
        assert_eq!(r.hp_loss, 0);
        assert_eq!(r.block_remaining, 5);
    }

    #[test]
    fn test_incoming_wrath() {
        // 10 * 2.0 = 20, - 5 block = 15 hp loss
        let r = calculate_incoming_damage(10, 5, true, false, false, false, false);
        assert_eq!(r.hp_loss, 15);
        assert_eq!(r.block_remaining, 0);
    }

    #[test]
    fn test_incoming_vulnerable() {
        // 10 * 1.5 = 15
        let r = calculate_incoming_damage(10, 0, false, true, false, false, false);
        assert_eq!(r.hp_loss, 15);
    }

    #[test]
    fn test_incoming_intangible() {
        let r = calculate_incoming_damage(100, 0, false, false, true, false, false);
        assert_eq!(r.hp_loss, 1);
    }

    #[test]
    fn test_incoming_torii() {
        // 4 unblocked -> 1 (Torii range 2-5)
        let r = calculate_incoming_damage(4, 0, false, false, false, true, false);
        assert_eq!(r.hp_loss, 1);
    }

    #[test]
    fn test_incoming_torii_below() {
        // 1 unblocked -> 1 (below Torii range)
        let r = calculate_incoming_damage(1, 0, false, false, false, true, false);
        assert_eq!(r.hp_loss, 1);
    }

    #[test]
    fn test_incoming_torii_above() {
        // 10 unblocked -> 10 (above Torii range)
        let r = calculate_incoming_damage(10, 0, false, false, false, true, false);
        assert_eq!(r.hp_loss, 10);
    }

    #[test]
    fn test_incoming_tungsten_rod() {
        // 10 - 5 block = 5 hp loss, -1 tungsten = 4
        let r = calculate_incoming_damage(10, 5, false, false, false, false, true);
        assert_eq!(r.hp_loss, 4);
    }

    // -- HP loss tests --

    #[test]
    fn test_hp_loss_basic() {
        assert_eq!(apply_hp_loss(5, false, false), 5);
    }

    #[test]
    fn test_hp_loss_intangible() {
        assert_eq!(apply_hp_loss(10, true, false), 1);
    }

    #[test]
    fn test_hp_loss_tungsten_rod() {
        assert_eq!(apply_hp_loss(5, false, true), 4);
    }

    #[test]
    fn test_hp_loss_intangible_plus_tungsten() {
        // Intangible caps to 1, Tungsten Rod -1 = 0
        assert_eq!(apply_hp_loss(10, true, true), 0);
    }
}
