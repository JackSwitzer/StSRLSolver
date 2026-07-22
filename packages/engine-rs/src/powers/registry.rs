//! Minimal production power metadata helpers.
//!
//! `status_is_debuff` is used by enemy debuff-clearing and combat hooks.

use crate::ids::StatusId;
use crate::status_ids::{sid, status_name};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TracePowerAmount {
    Stored,
    Marker,
    Negated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TracePowerSpec {
    pub java_id: &'static str,
    pub priority: i32,
    pub amount: TracePowerAmount,
}

/// Java-visible `AbstractPower` metadata for status-backed powers.
///
/// `status_name` remains a Rust debug identity. This table owns the exact
/// `AbstractPower.ID` strings used by TraceLab and rejects backing slots,
/// relic counters, and private AI state instead of leaking them as powers.
pub(crate) fn trace_power_spec(status: StatusId) -> Option<TracePowerSpec> {
    let visible = matches!(
        status.0,
        0..=41
            | 43..=54
            | 56..=65
            | 67..=68
            | 70..=84
            | 86..=99
            | 101..=104
            | 106..=115
            | 122..=128
            | 131
            | 183
            | 233
            | 241..=243
            | 245
            | 247
            | 251
            | 256..=258
            | 261..=269
    );
    if !visible {
        return None;
    }

    let java_id = match status {
        sid::NO_DRAW => "No Draw",
        sid::DRAW_REDUCTION => "Draw Reduction",
        sid::DEMON_FORM => "Demon Form",
        sid::DARK_EMBRACE => "Dark Embrace",
        sid::FEEL_NO_PAIN => "Feel No Pain",
        sid::FIRE_BREATHING => "Fire Breathing",
        sid::FLAME_BARRIER => "Flame Barrier",
        sid::AFTER_IMAGE => "After Image",
        sid::THOUSAND_CUTS => "Thousand Cuts",
        sid::NOXIOUS_FUMES => "Noxious Fumes",
        sid::INFINITE_BLADES => "Infinite Blades",
        sid::TOOLS_OF_THE_TRADE => "Tools Of The Trade",
        sid::RETAIN_CARDS => "Retain Cards",
        sid::DEVOTION => "DevotionPower",
        sid::ESTABLISHMENT => "EstablishmentPower",
        sid::LIKE_WATER => "LikeWaterPower",
        sid::MASTER_REALITY => "MasterRealityPower",
        sid::MENTAL_FORTRESS => "Controlled",
        sid::OMEGA => "OmegaPower",
        sid::RUSHDOWN => "Adaptation",
        sid::WAVE_OF_THE_HAND => "WaveOfTheHandPower",
        sid::WRAITH_FORM => "Wraith Form v2",
        sid::CREATIVE_AI => "Creative AI",
        sid::ECHO_FORM => "Echo Form",
        sid::ELECTRODYNAMICS => "Electro",
        sid::HELLO_WORLD => "Hello",
        sid::SADISTIC => "Sadistic",
        sid::TEMP_STRENGTH_LOSS => "Shackled",
        sid::NEXT_ATTACK_FREE => "FreeAttackPower",
        sid::DOUBLE_TAP => "Double Tap",
        sid::LOSE_STRENGTH => "Flex",
        sid::LOSE_DEXTERITY => "DexLoss",
        sid::DOUBLE_DAMAGE => "Double Damage",
        sid::NO_BLOCK => "NoBlockPower",
        sid::ENERGY_DOWN => "EnergyDownPower",
        sid::DRAW_CARD | sid::DOPPELGANGER_DRAW => "Draw Card",
        sid::NEXT_TURN_BLOCK => "Next Turn Block",
        sid::WRATH_NEXT_TURN => "WrathNextTurnPower",
        sid::CANNOT_CHANGE_STANCE => "CannotChangeStancePower",
        sid::NO_SKILLS_POWER => "NoSkills",
        sid::DOPPELGANGER_ENERGY => "Energized",
        sid::CURL_UP => "Curl Up",
        sid::PLATED_ARMOR => "Plated Armor",
        sid::SHARP_HIDE => "Sharp Hide",
        sid::MODE_SHIFT => "Mode Shift",
        sid::REACTIVE => "Compulsive",
        sid::TIME_WARP => "Time Warp",
        sid::GENERIC_STRENGTH_UP => "Generic Strength Up Power",
        sid::GROWTH => "GrowthPower",
        sid::SPORE_CLOUD => "Spore Cloud",
        sid::REGROW => "Life Link",
        sid::SKILL_BURN => "Skill Burn",
        sid::FORCEFIELD => "Nullify Attack",
        sid::LOCK_ON => "Lockon",
        sid::BLOCK_RETURN => "BlockReturnPower",
        sid::MARK => "PathToVictoryPower",
        sid::LIVE_FOREVER => "AngelForm",
        sid::DUPLICATION => "DuplicationPower",
        sid::FORESIGHT => "WireheadingPower",
        sid::SELF_REPAIR => "Repair",
        sid::CORPSE_EXPLOSION => "CorpseExplosionPower",
        sid::BIASED_COG_FOCUS_LOSS => "Bias",
        sid::COLLECT_MIRACLES => "Collect",
        sid::PAINFUL_STABS => "Painful Stabs",
        sid::UNAWAKENED => "Unawakened",
        sid::SPLIT_POWER => "Split",
        sid::ENERGIZED_BLUE => "EnergizedBlue",
        sid::MINION_POWER => "Minion",
        sid::BACK_ATTACK_POWER => "BackAttack",
        sid::STASIS_POWER => "Stasis",
        sid::PEN_NIB_POWER => "Pen Nib",
        sid::SURROUNDED_POWER => "Surrounded",
        sid::THIEVERY => "Thievery",
        _ => status_name(status),
    };

    let priority = match status {
        sid::CONFUSION => 0,
        sid::DOUBLE_DAMAGE | sid::PEN_NIB_POWER => 6,
        sid::FRAIL => 10,
        sid::DRAW_CARD | sid::DOPPELGANGER_DRAW => 20,
        sid::TOOLS_OF_THE_TRADE | sid::ESTABLISHMENT => 25,
        sid::FLIGHT | sid::REACTIVE => 50,
        sid::INTANGIBLE => 75,
        sid::WEAKENED | sid::INVINCIBLE => 99,
        sid::CONSTRICTED => 105,
        _ => 5,
    };

    let amount = if matches!(
        status,
        sid::CONFUSION
            | sid::NO_DRAW
            | sid::BARRICADE
            | sid::CORRUPTION
            | sid::MASTER_REALITY
            | sid::ELECTRODYNAMICS
            | sid::WRATH_NEXT_TURN
            | sid::CANNOT_CHANGE_STANCE
            | sid::END_TURN_DEATH
            | sid::REACTIVE
            | sid::SHIFTING
            | sid::REGROW
            | sid::FORCEFIELD
            | sid::PAINFUL_STABS
            | sid::UNAWAKENED
            | sid::SPLIT_POWER
            | sid::MINION_POWER
            | sid::BACK_ATTACK_POWER
            | sid::STASIS_POWER
            | sid::SURROUNDED_POWER
    ) {
        TracePowerAmount::Marker
    } else if status == sid::WRAITH_FORM {
        TracePowerAmount::Negated
    } else {
        TracePowerAmount::Stored
    };

    Some(TracePowerSpec {
        java_id,
        priority,
        amount,
    })
}

pub(crate) fn java_power_priority(status: StatusId) -> i32 {
    let projected = if status == sid::TIME_WARP_ACTIVE {
        sid::TIME_WARP
    } else {
        status
    };
    trace_power_spec(projected).map_or(5, |spec| spec.priority)
}

pub(crate) fn is_java_power_status(status: StatusId) -> bool {
    let projected = if status == sid::TIME_WARP_ACTIVE {
        sid::TIME_WARP
    } else {
        status
    };
    trace_power_spec(projected).is_some()
}

pub(crate) fn status_is_debuff(status_id: StatusId) -> bool {
    matches!(
        status_id,
        sid::WEAKENED
            | sid::VULNERABLE
            | sid::FRAIL
            | sid::POISON
            | sid::CONSTRICTED
            | sid::ENTANGLED
            | sid::HEX
            | sid::CONFUSION
            | sid::NO_DRAW
            | sid::DRAW_REDUCTION
            | sid::SLOW
            | sid::LOCK_ON
            | sid::FADING
            | sid::NO_BLOCK
            | sid::ENERGY_DOWN
            | sid::LOSE_STRENGTH
            | sid::WRAITH_FORM
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_projection_classifies_every_current_status_and_fails_reserved_ids_closed() {
        let visible = (0..sid::NUM_IDS)
            .filter(|index| trace_power_spec(StatusId(*index as u16)).is_some())
            .count();
        assert_eq!(visible, 137);
        for index in sid::NUM_IDS..sid::MAX_STATUS_ID {
            assert!(trace_power_spec(StatusId(index as u16)).is_none());
        }
        assert_eq!(
            trace_power_spec(sid::SPORE_CLOUD).unwrap().java_id,
            "Spore Cloud"
        );
        assert_eq!(
            trace_power_spec(sid::RUSHDOWN).unwrap().java_id,
            "Adaptation"
        );
        assert_eq!(trace_power_spec(sid::MANTRA).unwrap().java_id, "Mantra");
        assert!(trace_power_spec(sid::FIRST_MOVE).is_none());
        assert!(trace_power_spec(sid::PEN_NIB_COUNTER).is_none());
        assert!(trace_power_spec(sid::THE_BOMB).is_none());
        assert!(trace_power_spec(sid::HIGH_ASCENSION_AI).is_none());
        assert_eq!(
            trace_power_spec(sid::MINION_POWER).unwrap().java_id,
            "Minion"
        );
        assert_eq!(
            trace_power_spec(sid::BACK_ATTACK_POWER).unwrap().java_id,
            "BackAttack"
        );
        assert_eq!(
            trace_power_spec(sid::STASIS_POWER).unwrap().java_id,
            "Stasis"
        );
        assert_eq!(
            trace_power_spec(sid::SURROUNDED_POWER).unwrap().java_id,
            "Surrounded"
        );
        assert_eq!(trace_power_spec(sid::THIEVERY).unwrap().java_id, "Thievery");
        assert_eq!(trace_power_spec(sid::PEN_NIB_POWER).unwrap().priority, 6);
    }
}
