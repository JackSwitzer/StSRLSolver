//! Shared potion helpers for owner-aware runtime definitions in `potions/defs`.

pub mod defs;

use crate::state::CombatState;

#[cfg(test)]
pub(crate) fn equip_potion_slot(
    engine: &mut crate::engine::CombatEngine,
    slot: usize,
    potion_id: &str,
) {
    if slot >= engine.state.potions.len() {
        return;
    }
    engine.state.potions[slot] = potion_id.to_string();
    engine.rebuild_effect_runtime();
}

fn is_boss_enemy_id(enemy_id: &str) -> bool {
    let normalized: String = enemy_id
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    matches!(
        normalized.as_str(),
        "theguardian"
            | "hexaghost"
            | "slimeboss"
            | "bronzeautomaton"
            | "thecollector"
            | "champ"
            | "awakenedone"
            | "timeeater"
            | "donu"
            | "deca"
            | "corruptheart"
    )
}

pub fn potion_can_use_in_combat(state: &CombatState, potion_id: &str) -> bool {
    match potion_id {
        // FairyPotion.canUse() is always false; it triggers passively on lethal
        // damage from AbstractPlayer.damage instead.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FairyPotion.java
        "FairyPotion" | "Fairy in a Bottle" => false,
        "SmokeBomb" | "Smoke Bomb" => {
            // SmokeBomb.canUse scans BackAttack without filtering dead or
            // dying monsters. This matters when the current BackAttack owner
            // dies: Spire Shield/Spear cleanup deliberately skips that corpse.
            // Java: SmokeBomb.java and SpireShield.java::die.
            !state.enemies.iter().any(|enemy| {
                (enemy.is_alive() && is_boss_enemy_id(enemy.id.as_str())) || enemy.has_back_attack()
            })
        }
        _ => true,
    }
}

pub(crate) fn upgrade_hand_for_combat(state: &mut CombatState) {
    let registry = crate::cards::global_registry();
    for card in &mut state.hand {
        registry.upgrade_card(card);
    }
}

pub(crate) fn return_discard_to_hand(state: &mut CombatState, amount: i32) -> i32 {
    let mut moved = 0;
    for _ in 0..amount {
        if state.discard_pile.is_empty() || state.hand.len() >= 10 {
            break;
        }
        // BetterDiscardPileToHandAction copies CardGroup.group in its stored
        // order and moves from the front when every card is auto-selected.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDiscardPileToHandAction.java
        let card = state.discard_pile.remove(0);
        state.hand.push(card);
        moved += 1;
    }
    moved
}

/// Check if a potion requires a target enemy.
pub fn potion_requires_target(potion_id: &str) -> bool {
    matches!(
        potion_id,
        "Fire Potion"
            | "FirePotion"
            | "Weak Potion"
            | "WeakenPotion"
            | "FearPotion"
            | "Fear Potion"
            | "Poison Potion"
            | "PoisonPotion"
    )
}

/// Return (base_potency, a11_potency) for the named potion.
/// Ascension 11 reduces most potion values. Potions not in this table
/// are unaffected by ascension.
fn potion_potency(potion_id: &str) -> Option<(i32, i32)> {
    match potion_id {
        // FirePotion.java getPotency ignores ascension and always returns 20.
        "Fire Potion" | "FirePotion" => Some((20, 20)),
        // ExplosivePotion.java getPotency ignores ascension and always returns 10.
        "Explosive Potion" | "ExplosivePotion" => Some((10, 10)),
        // BlockPotion.java getPotency ignores ascension and always returns 12.
        "Block Potion" | "BlockPotion" => Some((12, 12)),
        // StrengthPotion.java getPotency ignores ascension and always returns 2.
        "Strength Potion" | "StrengthPotion" => Some((2, 2)),
        // DexterityPotion.java getPotency ignores ascension and always returns 2.
        "Dexterity Potion" | "DexterityPotion" => Some((2, 2)),
        // FocusPotion.java getPotency ignores ascension and always returns 2.
        "Focus Potion" | "FocusPotion" => Some((2, 2)),
        // SteroidPotion.java getPotency ignores ascension and always returns 5.
        "SteroidPotion" | "Flex Potion" => Some((5, 5)),
        // SpeedPotion.java getPotency ignores ascension and always returns 5.
        "SpeedPotion" => Some((5, 5)),
        // WeakenPotion.java getPotency ignores ascension and always returns 3.
        "Weak Potion" | "WeakenPotion" => Some((3, 3)),
        // FearPotion.java getPotency ignores ascension and always returns 3.
        "FearPotion" | "Fear Potion" => Some((3, 3)),
        // PoisonPotion.java getPotency ignores ascension and always returns 6.
        "Poison Potion" | "PoisonPotion" => Some((6, 6)),
        // EnergyPotion.java getPotency ignores ascension and always returns 2.
        "Energy Potion" | "EnergyPotion" => Some((2, 2)),
        // SwiftPotion.java getPotency ignores ascension and always returns 3.
        "Swift Potion" | "SwiftPotion" => Some((3, 3)),
        // SneckoOil.java getPotency ignores ascension and always returns 5.
        "SneckoOil" => Some((5, 5)),
        "Ancient Potion" | "AncientPotion" => Some((1, 1)),
        // RegenPotion.java getPotency ignores ascension and always returns 5.
        "Regen Potion" | "RegenPotion" => Some((5, 5)),
        // EssenceOfSteel.java getPotency ignores ascension and always returns 4.
        "EssenceOfSteel" => Some((4, 4)),
        // LiquidBronze.java getPotency ignores ascension and always returns 3.
        "LiquidBronze" => Some((3, 3)),
        "CultistPotion" => Some((1, 1)),
        // HeartOfIron.java getPotency ignores ascension and always returns 6.
        "Heart of Iron" | "HeartOfIron" => Some((6, 6)),
        "GhostInAJar" => Some((1, 1)),
        "DuplicationPotion" => Some((1, 1)),
        // BloodPotion.java getPotency ignores ascension and always returns 20.
        "Blood Potion" | "BloodPotion" => Some((20, 20)),
        // FruitJuice.java getPotency ignores ascension and always returns 5.
        "Fruit Juice" | "FruitJuice" => Some((5, 5)),
        "BottledMiracle" => Some((2, 1)),
        // CunningPotion.java getPotency ignores ascension and always returns 3.
        "CunningPotion" => Some((3, 3)),
        // PotionOfCapacity.java getPotency ignores ascension and always returns 2.
        "Potion of Capacity" | "PotionOfCapacity" => Some((2, 2)),
        _ => None,
    }
}

/// Get the effective potency for a potion, accounting for ascension 11+
/// and Sacred Bark.
fn effective_potency(potion_id: &str, ascension: i32, bark_mult: i32) -> i32 {
    match potion_potency(potion_id) {
        Some((base, a11)) => {
            let raw = if ascension >= 11 { a11 } else { base };
            raw * bark_mult
        }
        None => bark_mult,
    }
}

/// Runtime-facing potion potency helper for combat engine activation.
///
/// The current combat engine path does not thread ascension into `CombatState`,
/// so owner-aware potion activation matches the existing base-potency combat
/// behavior while still respecting Sacred Bark.
pub fn effective_potency_runtime(state: &CombatState, potion_id: &str) -> i32 {
    let bark_mult = if state.has_relic("SacredBark") { 2 } else { 1 };
    effective_potency(potion_id, 0, bark_mult)
}

/// Check if player should auto-revive (Fairy in a Bottle).
/// Returns the HP to revive to (30% of max_hp), or 0 if no fairy.
pub fn check_fairy_revive(state: &CombatState) -> i32 {
    check_fairy_revive_scaled(state, 0)
}

/// Check fairy revive with the retained ascension parameter used by helper
/// tests. FairyPotion.java has no ascension branch: potency is always 30%.
pub fn check_fairy_revive_scaled(state: &CombatState, _ascension: i32) -> i32 {
    let bark = state.has_relic("SacredBark");
    let base_pct = 30;
    let potency = if bark { base_pct * 2 } else { base_pct };
    for potion in &state.potions {
        if potion == "FairyPotion" || potion == "Fairy in a Bottle" {
            return ((state.player.max_hp * potency) / 100).max(1);
        }
    }
    0
}

/// Consume the Fairy in a Bottle potion slot after reviving.
pub fn consume_fairy(state: &mut CombatState) {
    for slot in &mut state.potions {
        if slot == "FairyPotion" || slot == "Fairy in a Bottle" {
            *slot = String::new();
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{CombatState, EnemyCombatState};
    use crate::tests::support::make_deck_n;

    fn make_test_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 44, 44);
        let mut state = CombatState::new(80, 80, vec![enemy], make_deck_n("Strike", 5), 3);
        state.potions = vec!["".to_string(); 3];
        state
    }

    #[test]
    fn test_fairy_revive_check() {
        let mut state = make_test_state();
        assert_eq!(check_fairy_revive(&state), 0);
        state.potions[0] = "FairyPotion".to_string();
        assert_eq!(check_fairy_revive(&state), 24);
    }

    #[test]
    fn test_fairy_consume() {
        let mut state = make_test_state();
        state.potions[1] = "FairyPotion".to_string();
        consume_fairy(&mut state);
        assert!(state.potions[1].is_empty());
    }

    #[test]
    fn test_sacred_bark_fairy_revive() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        state.potions[0] = "FairyPotion".to_string();
        assert_eq!(check_fairy_revive(&state), 48);
    }

    #[test]
    fn test_potion_requires_target() {
        assert!(potion_requires_target("Fire Potion"));
        assert!(potion_requires_target("Weak Potion"));
        assert!(potion_requires_target("FearPotion"));
        assert!(potion_requires_target("Poison Potion"));
        assert!(!potion_requires_target("Block Potion"));
        assert!(!potion_requires_target("Strength Potion"));
        assert!(!potion_requires_target("Energy Potion"));
    }

    #[test]
    fn test_a11_fairy_revive_stays_at_thirty_percent() {
        // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FairyPotion.java
        let mut state = make_test_state();
        state.potions[0] = "FairyPotion".to_string();
        assert_eq!(check_fairy_revive_scaled(&state, 11), 24);
    }
}
