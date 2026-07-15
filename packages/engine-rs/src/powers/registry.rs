//! Minimal production power metadata helpers.
//!
//! `status_is_debuff` is used by enemy debuff-clearing and combat hooks.

use crate::ids::StatusId;
use crate::status_ids::sid;

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
