//! Declarative power definitions using the unified EntityDef system.
//!
//! Each power is expressed as a static `EntityDef` with triggered effects
//! that describe WHEN and WHAT happens. Simple powers are fully declarative;
//! complex powers (card replay, on-attacked reactions) use `complex_hook`.
//! The owner-aware runtime in `effects::runtime` now executes the migrated
//! subset directly, while `powers/registry.rs` remains as install-time
//! metadata plus a shrinking legacy-oracle surface during the cutover.

mod turn_start;
mod turn_end;
mod card_play;
mod exhaust;
mod stance;
mod enemy;
mod complex;

use crate::effects::entity_def::EntityDef;

// Re-export individual definitions for direct access
pub use turn_start::*;
pub use turn_end::*;
pub use card_play::*;
pub use exhaust::*;
pub use stance::*;
pub use enemy::*;
pub use complex::*;

// ===========================================================================
// POWER_DEFS — static registry of all declarative power definitions
// ===========================================================================

/// All declarative power definitions, in a single static array.
/// Grouped by trigger type for clarity.
pub static POWER_DEFS: &[&EntityDef] = &[
    // -- Turn Start (simple) --
    &turn_start::DEF_DEMON_FORM,
    &turn_start::DEF_NOXIOUS_FUMES,
    &turn_start::DEF_BRUTALITY,
    &turn_start::DEF_BERSERK,
    &turn_start::DEF_INFINITE_BLADES,
    &turn_start::DEF_BATTLE_HYMN,
    &turn_start::DEF_DEVOTION,
    &turn_start::DEF_WRAITH_FORM,
    &turn_start::DEF_DEVA_FORM,
    &turn_start::DEF_HELLO_WORLD,
    &turn_start::DEF_MAGNETISM,
    &turn_start::DEF_DOPPELGANGER_DRAW,
    &turn_start::DEF_DOPPELGANGER_ENERGY,

    // -- Turn Start (complex) --
    &turn_start::DEF_CREATIVE_AI,
    &turn_start::DEF_ENTER_DIVINITY,
    &turn_start::DEF_MAYHEM,
    &turn_start::DEF_TOOLS_OF_THE_TRADE,

    // -- Turn End --
    &turn_end::DEF_METALLICIZE,
    &turn_end::DEF_PLATED_ARMOR,
    &turn_end::DEF_COMBUST,
    &turn_end::DEF_OMEGA,
    &turn_end::DEF_LIKE_WATER,
    &turn_end::DEF_STUDY,

    // -- Card Play --
    &card_play::DEF_AFTER_IMAGE,
    &card_play::DEF_RAGE,
    &card_play::DEF_HEATSINK,
    &card_play::DEF_STORM,
    &card_play::DEF_CURIOSITY,
    &card_play::DEF_BEAT_OF_DEATH,
    &card_play::DEF_SLOW,
    &card_play::DEF_FORCEFIELD,
    &card_play::DEF_SKILL_BURN,

    // -- Exhaust --
    &exhaust::DEF_FEEL_NO_PAIN,
    &exhaust::DEF_DARK_EMBRACE,

    // -- Stance Change --
    &stance::DEF_MENTAL_FORTRESS,
    &stance::DEF_RUSHDOWN,

    // -- Enemy Turn Start --
    &enemy::DEF_RITUAL,
    &enemy::DEF_REGENERATION,
    &enemy::DEF_GROWTH,
    &enemy::DEF_METALLICIZE_ENEMY,

    // -- Complex (hook-based) --
    &complex::DEF_ECHO_FORM,
    &complex::DEF_DOUBLE_TAP,
    &complex::DEF_BURST,
    &complex::DEF_THORNS,
    &complex::DEF_FLAME_BARRIER,
    &complex::DEF_ENVENOM,
    &complex::DEF_SADISTIC_NATURE,
    &complex::DEF_THOUSAND_CUTS,
    &complex::DEF_PANACHE,
    &complex::DEF_ELECTRODYNAMICS,
    &complex::DEF_TIME_WARP,
    &complex::DEF_STATIC_DISCHARGE,
];

/// Power defs that are executed by the owner-aware runtime today.
/// Complex powers that still execute inline in `engine.rs` are intentionally
/// excluded until their runtime hooks are migrated.
pub static RUNTIME_PLAYER_POWER_DEFS: &[&EntityDef] = &[
    &turn_start::DEF_DEMON_FORM,
    &turn_start::DEF_NOXIOUS_FUMES,
    &turn_start::DEF_BRUTALITY,
    &turn_start::DEF_BERSERK,
    &turn_start::DEF_INFINITE_BLADES,
    &turn_start::DEF_BATTLE_HYMN,
    &turn_start::DEF_DEVOTION,
    &turn_start::DEF_WRAITH_FORM,
    &turn_start::DEF_DEVA_FORM,
    &turn_start::DEF_HELLO_WORLD,
    &turn_start::DEF_MAGNETISM,
    &turn_start::DEF_CREATIVE_AI,
    &turn_start::DEF_DOPPELGANGER_DRAW,
    &turn_start::DEF_DOPPELGANGER_ENERGY,
    &turn_start::DEF_ENTER_DIVINITY,
    &turn_start::DEF_MAYHEM,
    &turn_start::DEF_TOOLS_OF_THE_TRADE,
    &turn_end::DEF_METALLICIZE,
    &turn_end::DEF_PLATED_ARMOR,
    &turn_end::DEF_COMBUST,
    &turn_end::DEF_OMEGA,
    &turn_end::DEF_LIKE_WATER,
    &turn_end::DEF_STUDY,
    &card_play::DEF_AFTER_IMAGE,
    &card_play::DEF_RAGE,
    &card_play::DEF_HEATSINK,
    &card_play::DEF_STORM,
    &complex::DEF_ECHO_FORM,
    &complex::DEF_DOUBLE_TAP,
    &complex::DEF_BURST,
    &complex::DEF_THOUSAND_CUTS,
    &complex::DEF_PANACHE,
    &complex::DEF_SADISTIC_NATURE,
    &exhaust::DEF_FEEL_NO_PAIN,
    &exhaust::DEF_DARK_EMBRACE,
    &stance::DEF_MENTAL_FORTRESS,
    &stance::DEF_RUSHDOWN,
];

/// Enemy-owned power defs that can safely execute through the owner-aware
/// runtime without proxying through player state.
pub static RUNTIME_ENEMY_POWER_DEFS: &[&EntityDef] = &[
    &card_play::DEF_CURIOSITY,
    &card_play::DEF_BEAT_OF_DEATH,
    &card_play::DEF_SLOW,
    &card_play::DEF_FORCEFIELD,
    &card_play::DEF_SKILL_BURN,
    &enemy::DEF_RITUAL,
    &enemy::DEF_REGENERATION,
    &enemy::DEF_GROWTH,
    &enemy::DEF_METALLICIZE_ENEMY,
];

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::entity_def::EntityKind;

    #[test]
    fn test_power_defs_count() {
        assert!(
            POWER_DEFS.len() >= 49,
            "Expected at least 49 power defs, got {}",
            POWER_DEFS.len()
        );
    }

    #[test]
    fn test_all_defs_are_powers() {
        for def in POWER_DEFS.iter() {
            assert_eq!(
                def.kind,
                EntityKind::Power,
                "Power def '{}' has wrong EntityKind",
                def.id
            );
        }
    }

    #[test]
    fn test_all_defs_have_triggers_or_hooks() {
        for def in POWER_DEFS.iter() {
            assert!(
                !def.triggers.is_empty() || def.complex_hook.is_some(),
                "Power def '{}' has no triggers and no complex_hook",
                def.id
            );
        }
    }

    #[test]
    fn test_no_duplicate_ids() {
        for (i, def_a) in POWER_DEFS.iter().enumerate() {
            for (j, def_b) in POWER_DEFS.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        def_a.id, def_b.id,
                        "Duplicate power def id: '{}'",
                        def_a.id
                    );
                }
            }
        }
    }

    #[test]
    fn test_simple_defs_have_no_hooks() {
        // Simple declarative powers should not need complex_hook
        let simple_ids = [
            "demon_form", "noxious_fumes", "brutality", "berserk",
            "metallicize", "plated_armor", "after_image", "rage",
            "feel_no_pain", "dark_embrace", "mental_fortress", "rushdown",
            "doppelganger_draw", "doppelganger_energy", "heatsink",
            "curiosity", "beat_of_death", "slow", "forcefield", "skill_burn",
        ];
        for id in &simple_ids {
            let def = POWER_DEFS.iter().find(|d| d.id == *id);
            if let Some(def) = def {
                assert!(
                    def.complex_hook.is_none(),
                    "Simple power '{}' should not have complex_hook",
                    id
                );
            }
        }
    }

    #[test]
    fn test_complex_defs_have_hooks() {
        let complex_ids = [
            "echo_form", "double_tap", "burst", "thorns", "flame_barrier",
            "creative_ai", "enter_divinity", "mayhem", "tools_of_the_trade",
            "storm", "time_warp", "static_discharge",
        ];
        for id in &complex_ids {
            let def = POWER_DEFS.iter().find(|d| d.id == *id);
            if let Some(def) = def {
                assert!(
                    def.complex_hook.is_some(),
                    "Complex power '{}' should have complex_hook",
                    id
                );
            }
        }
    }
}
