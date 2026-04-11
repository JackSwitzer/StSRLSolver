//! Declarative potion definitions using the unified EntityDef system.
//!
//! Each potion is a static EntityDef with ManualActivation trigger.
//! Simple potions express their effects purely as data (AddStatus, DealDamage, etc.).
//! Complex potions (Elixir, Gambler's Brew, Entropic Brew) use complex_hook fn pointers.
//!
//! The existing match-block dispatch in potions/mod.rs remains active.
//! These definitions are for future interpreter wiring.

mod prelude;

// --- Simple potions (pure declarative effects) ---
pub mod strength_potion;
pub mod dexterity_potion;
pub mod focus_potion;
pub mod block_potion;
pub mod swift_potion;
pub mod energy_potion;
pub mod weak_potion;
pub mod fear_potion;
pub mod poison_potion;
pub mod speed_potion;
pub mod flex_potion;
pub mod artifact_potion;
pub mod regen_potion;
pub mod essence_of_steel;
pub mod liquid_bronze;
pub mod heart_of_iron;
pub mod blood_potion;
pub mod fruit_juice;
pub mod smoke_bomb;
pub mod fire_potion;
pub mod explosive_potion;

// --- Complex potions (need fn pointer hooks) ---
pub mod snecko_oil;
pub mod elixir;
pub mod gamblers_brew;
pub mod entropic_brew;
pub mod fairy_in_a_bottle;

use crate::effects::entity_def::EntityDef;

/// Static registry of all potion EntityDefs.
/// Index order matches the module declarations above.
pub static POTION_DEFS: &[&EntityDef] = &[
    // Simple potions
    &strength_potion::DEF,
    &dexterity_potion::DEF,
    &focus_potion::DEF,
    &block_potion::DEF,
    &swift_potion::DEF,
    &energy_potion::DEF,
    &weak_potion::DEF,
    &fear_potion::DEF,
    &poison_potion::DEF,
    &speed_potion::DEF,
    &flex_potion::DEF,
    &artifact_potion::DEF,
    &regen_potion::DEF,
    &essence_of_steel::DEF,
    &liquid_bronze::DEF,
    &heart_of_iron::DEF,
    &blood_potion::DEF,
    &fruit_juice::DEF,
    &smoke_bomb::DEF,
    &fire_potion::DEF,
    &explosive_potion::DEF,
    // Complex potions
    &snecko_oil::DEF,
    &elixir::DEF,
    &gamblers_brew::DEF,
    &entropic_brew::DEF,
    &fairy_in_a_bottle::DEF,
];

/// Look up a potion EntityDef by id.
pub fn potion_def_by_id(id: &str) -> Option<&'static EntityDef> {
    POTION_DEFS.iter().find(|d| d.id == id).copied()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::entity_def::EntityKind;
    use crate::effects::trigger::Trigger;

    #[test]
    fn test_potion_defs_count() {
        assert_eq!(POTION_DEFS.len(), 26);
    }

    #[test]
    fn test_all_potions_are_potion_kind() {
        for def in POTION_DEFS.iter() {
            assert_eq!(def.kind, EntityKind::Potion, "non-Potion kind for {}", def.id);
        }
    }

    #[test]
    fn test_simple_potions_have_manual_activation_trigger() {
        let simple_ids = [
            "StrengthPotion", "DexterityPotion", "FocusPotion", "BlockPotion",
            "SwiftPotion", "EnergyPotion", "WeakenPotion", "FearPotion",
            "PoisonPotion", "SpeedPotion", "SteroidPotion", "AncientPotion",
            "RegenPotion", "EssenceOfSteel", "LiquidBronze", "HeartOfIron",
            "BloodPotion", "FruitJuice", "SmokeBomb", "FirePotion",
            "ExplosivePotion", "SneckoOil",
        ];
        for id in &simple_ids {
            let def = potion_def_by_id(id)
                .unwrap_or_else(|| panic!("missing potion def: {}", id));
            assert!(!def.triggers.is_empty(), "no triggers for {}", id);
            assert_eq!(def.triggers[0].trigger, Trigger::ManualActivation,
                "wrong trigger for {}", id);
        }
    }

    #[test]
    fn test_complex_potions_have_hooks_or_empty_triggers() {
        let complex_ids = ["Elixir", "GamblersBrew", "EntropicBrew"];
        for id in &complex_ids {
            let def = potion_def_by_id(id)
                .unwrap_or_else(|| panic!("missing potion def: {}", id));
            assert!(def.complex_hook.is_some(), "no complex_hook for {}", id);
        }
    }

    #[test]
    fn test_fairy_is_passive() {
        let def = potion_def_by_id("FairyPotion").unwrap();
        assert!(def.triggers.is_empty());
        assert!(def.complex_hook.is_none());
    }

    #[test]
    fn test_unique_ids() {
        let mut ids: Vec<&str> = POTION_DEFS.iter().map(|d| d.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), POTION_DEFS.len(), "duplicate potion def ids");
    }

    #[test]
    fn test_lookup_by_id() {
        assert!(potion_def_by_id("FirePotion").is_some());
        assert!(potion_def_by_id("NonExistent").is_none());
    }
}
