//! Declarative potion definitions using the owner-aware EntityDef runtime.
//!
//! Potions with `ManualActivation` triggers execute through the owner-aware
//! runtime first. `potions/mod.rs` remains as an oracle/helper surface for
//! legacy tests and any unmigrated fallback callers.

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
pub mod cultist_potion;
pub mod ghost_in_a_jar;
pub mod duplication_potion;

// --- Complex potions (need fn pointer hooks) ---
pub mod snecko_oil;
pub mod elixir;
pub mod gamblers_brew;
pub mod entropic_brew;
pub mod fairy_in_a_bottle;
pub mod bottled_miracle;
pub mod cunning_potion;
pub mod ambrosia;
pub mod stance_potion;
pub mod blessing_of_forge;
pub mod liquid_memories;
pub mod distilled_chaos;
pub mod essence_of_darkness;
pub mod attack_potion;
pub mod skill_potion;
pub mod power_potion;
pub mod colorless_potion;
pub mod potion_of_capacity;

use crate::effects::entity_def::EntityDef;
use crate::effects::trigger::Trigger;

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
    &cultist_potion::DEF,
    &ghost_in_a_jar::DEF,
    &duplication_potion::DEF,
    // Complex potions
    &snecko_oil::DEF,
    &elixir::DEF,
    &gamblers_brew::DEF,
    &entropic_brew::DEF,
    &fairy_in_a_bottle::DEF,
    &bottled_miracle::DEF,
    &cunning_potion::DEF,
    &ambrosia::DEF,
    &stance_potion::DEF,
    &blessing_of_forge::DEF,
    &liquid_memories::DEF,
    &distilled_chaos::DEF,
    &essence_of_darkness::DEF,
    &attack_potion::DEF,
    &skill_potion::DEF,
    &power_potion::DEF,
    &colorless_potion::DEF,
    &potion_of_capacity::DEF,
];

/// Look up a potion EntityDef by id.
pub fn potion_def_by_id(id: &str) -> Option<&'static EntityDef> {
    POTION_DEFS.iter().find(|d| d.id == id).copied()
}

fn normalize_potion_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

/// Look up a potion EntityDef by either compact id or display-style runtime name.
pub fn potion_def_by_runtime_id(id: &str) -> Option<&'static EntityDef> {
    let wanted = normalize_potion_key(id);
    POTION_DEFS
        .iter()
        .find(|def| {
            normalize_potion_key(def.id) == wanted
                || normalize_potion_key(def.name) == wanted
        })
        .copied()
}

pub fn potion_uses_runtime_manual_activation(id: &str) -> bool {
    potion_def_by_runtime_id(id).is_some_and(|def| {
        def.triggers
            .iter()
            .any(|trigger| trigger.trigger == Trigger::ManualActivation)
    })
}

/// Runtime-authoritative potions have full production behavior on the
/// owner-aware manual-activation path and should never fall back to the
/// legacy `apply_potion` oracle path.
pub fn potion_runtime_manual_activation_is_authoritative(id: &str) -> bool {
    potion_def_by_runtime_id(id).is_some_and(|def| {
        matches!(
            def.id,
            "BlockPotion"
                | "SwiftPotion"
                | "EnergyPotion"
                | "Ambrosia"
                | "BlessingOfTheForge"
                | "BottledMiracle"
                | "DexterityPotion"
                | "EntropicBrew"
                | "EssenceOfDarkness"
                | "ExplosivePotion"
                | "FearPotion"
                | "FirePotion"
                | "GamblersBrew"
                | "LiquidMemories"
                | "PoisonPotion"
                | "PotionOfCapacity"
                | "StancePotion"
                | "StrengthPotion"
                | "WeakenPotion"
                | "AncientPotion"
                | "RegenPotion"
                | "EssenceOfSteel"
                | "LiquidBronze"
                | "CultistPotion"
                | "GhostInAJar"
                | "DuplicationPotion"
                | "SmokeBomb"
                | "AttackPotion"
                | "SkillPotion"
                | "PowerPotion"
                | "ColorlessPotion"
        )
    })
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
        assert_eq!(POTION_DEFS.len(), 42);
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
            "CultistPotion", "GhostInAJar", "DuplicationPotion",
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
        let complex_ids = [
            "Elixir", "GamblersBrew", "EntropicBrew",
            "BottledMiracle", "CunningPotion", "Ambrosia",
            "BlessingOfTheForge", "LiquidMemories",
            "DistilledChaos", "EssenceOfDarkness",
            "AttackPotion", "SkillPotion", "PowerPotion",
            "ColorlessPotion", "PotionOfCapacity",
        ];
        for id in &complex_ids {
            let def = potion_def_by_id(id)
                .unwrap_or_else(|| panic!("missing potion def: {}", id));
            assert!(def.complex_hook.is_some(), "no complex_hook for {}", id);
        }
    }

    #[test]
    fn test_stance_potion_uses_declarative_choice_effects() {
        let def = potion_def_by_id("StancePotion").unwrap();
        assert!(def.complex_hook.is_none());
        assert_eq!(def.triggers.len(), 1);
        assert_eq!(def.triggers[0].effects.len(), 1);
        match def.triggers[0].effects[0] {
            crate::effects::declarative::Effect::ChooseScaledNamedOptions(options) => {
                assert_eq!(options.len(), 2);
                assert_eq!(options[0].label, "Wrath");
                assert_eq!(options[1].label, "Calm");
            }
            other => panic!("expected declarative stance choice, got {other:?}"),
        }
    }

    #[test]
    fn test_runtime_manual_activation_lookup_covers_migrated_complex_potions() {
        let migrated_ids = [
            "AttackPotion", "SkillPotion", "PowerPotion", "ColorlessPotion",
            "Elixir", "GamblersBrew", "EntropicBrew", "BottledMiracle",
            "CunningPotion", "Ambrosia", "StancePotion", "BlessingOfTheForge",
            "LiquidMemories", "DistilledChaos", "EssenceOfDarkness",
            "PotionOfCapacity",
        ];
        for id in migrated_ids {
            assert!(
                potion_uses_runtime_manual_activation(id),
                "{id} should advertise runtime manual activation"
            );
        }
    }

    #[test]
    fn test_runtime_manual_activation_authority_covers_wave6_simple_combat_potions() {
        let ids = [
            "BlockPotion",
            "SwiftPotion",
            "EnergyPotion",
            "Ambrosia",
            "BlessingOfTheForge",
            "BottledMiracle",
            "DexterityPotion",
            "EntropicBrew",
            "EssenceOfDarkness",
            "ExplosivePotion",
            "FearPotion",
            "FirePotion",
            "GamblersBrew",
            "LiquidMemories",
            "PoisonPotion",
            "PotionOfCapacity",
            "StancePotion",
            "StrengthPotion",
            "WeakenPotion",
            "AncientPotion",
            "RegenPotion",
            "EssenceOfSteel",
            "LiquidBronze",
            "CultistPotion",
            "GhostInAJar",
            "DuplicationPotion",
            "SmokeBomb",
            "Block Potion",
            "Swift Potion",
            "Energy Potion",
            "Ambrosia",
            "Blessing of the Forge",
            "Bottled Miracle",
            "Dexterity Potion",
            "Entropic Brew",
            "Essence of Darkness",
            "Explosive Potion",
            "Fear Potion",
            "Fire Potion",
            "Gambler's Brew",
            "Liquid Memories",
            "Poison Potion",
            "Potion of Capacity",
            "Stance Potion",
            "Strength Potion",
            "Weak Potion",
            "Ancient Potion",
            "Regen Potion",
            "Essence of Steel",
            "Liquid Bronze",
            "Cultist Potion",
            "Ghost in a Jar",
            "Duplication Potion",
            "Smoke Bomb",
        ];

        for id in ids {
            assert!(
                potion_runtime_manual_activation_is_authoritative(id),
                "{id} should be runtime-authoritative"
            );
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

    #[test]
    fn test_lookup_by_runtime_name() {
        assert_eq!(
            potion_def_by_runtime_id("Fire Potion").map(|def| def.id),
            Some("FirePotion")
        );
        assert_eq!(
            potion_def_by_runtime_id("Fairy in a Bottle").map(|def| def.id),
            Some("FairyPotion")
        );
    }
}

#[cfg(test)]
#[path = "../../tests/test_potion_runtime_action_path.rs"]
mod test_potion_runtime_action_path;

#[cfg(test)]
#[path = "../../tests/test_potion_runtime_wave4.rs"]
mod test_potion_runtime_wave4;

#[cfg(test)]
#[path = "../../tests/test_potion_runtime_wave5.rs"]
mod test_potion_runtime_wave5;

#[cfg(test)]
#[path = "../../tests/test_potion_runtime_wave6.rs"]
mod test_potion_runtime_wave6;

#[cfg(test)]
#[path = "../../tests/test_potion_runtime_wave7.rs"]
mod test_potion_runtime_wave7;

#[cfg(test)]
#[path = "../../tests/test_potion_runtime_wave8.rs"]
mod test_potion_runtime_wave8;
