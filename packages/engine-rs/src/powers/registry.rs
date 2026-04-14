//! Minimal production power metadata helpers.
//!
//! The old hook-table registry was retired once power helper-path tests moved
//! onto engine-path/runtime coverage. The only helpers still needed by live
//! code are:
//! - `status_is_debuff` for enemy debuff-clearing and combat hooks
//! - `active_player_power_count` for Force Field style cost reduction

use crate::ids::StatusId;
use crate::state::EntityState;
use crate::status_ids::sid;

/// Player-visible power statuses that contribute to Force Field cost.
const PLAYER_POWER_STATUSES: &[StatusId] = &[
    sid::BARRICADE,
    sid::DEMON_FORM,
    sid::CORRUPTION,
    sid::DARK_EMBRACE,
    sid::FEEL_NO_PAIN,
    sid::BRUTALITY,
    sid::COMBUST,
    sid::EVOLVE,
    sid::FIRE_BREATHING,
    sid::JUGGERNAUT,
    sid::METALLICIZE,
    sid::RUPTURE,
    sid::BERSERK,
    sid::RAGE,
    sid::FLAME_BARRIER,
    sid::AFTER_IMAGE,
    sid::THOUSAND_CUTS,
    sid::NOXIOUS_FUMES,
    sid::INFINITE_BLADES,
    sid::ENVENOM,
    sid::ACCURACY,
    sid::TOOLS_OF_THE_TRADE,
    sid::RETAIN_CARDS,
    sid::BATTLE_HYMN,
    sid::DEVOTION,
    sid::DEVA_FORM,
    sid::ESTABLISHMENT,
    sid::FASTING,
    sid::MASTER_REALITY,
    sid::LIKE_WATER,
    sid::NIRVANA,
    sid::OMEGA,
    sid::MENTAL_FORTRESS,
    sid::RUSHDOWN,
    sid::STUDY,
    sid::WAVE_OF_THE_HAND,
    sid::WRAITH_FORM,
    sid::BUFFER,
    sid::CREATIVE_AI,
    sid::ECHO_FORM,
    sid::ELECTRODYNAMICS,
    sid::HEATSINK,
    sid::HELLO_WORLD,
    sid::LOOP,
    sid::STORM,
    sid::STATIC_DISCHARGE,
    sid::PANACHE,
    sid::SADISTIC,
    sid::MAGNETISM,
    sid::MAYHEM,
    sid::DRAW,
];

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
    )
}

pub(crate) fn active_player_power_count(entity: &EntityState) -> i32 {
    PLAYER_POWER_STATUSES
        .iter()
        .filter(|status_id| entity.status(**status_id) > 0)
        .count() as i32
}
