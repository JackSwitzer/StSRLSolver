//! modify_damage hooks — adjust damage calculation before the generic damage loop.

use crate::cards::CardDef;
use crate::combat_types::CardInstance;
use crate::engine::CombatEngine;
use crate::status_ids::sid;
use super::types::DamageModifier;

/// Heavy Blade: multiply strength contribution (3x base, 5x upgraded).
pub fn hook_heavy_blade(engine: &CombatEngine, card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    let _ = engine;
    DamageModifier {
        strength_multiplier: card.base_magic.max(1),
        ..DamageModifier::default()
    }
}

/// Body Slam: damage = player's current block.
pub fn hook_damage_equals_block(engine: &CombatEngine, _card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    DamageModifier {
        base_damage_override: engine.state.player.block,
        ..DamageModifier::default()
    }
}

/// Brilliance: extra damage from mantra gained this combat.
pub fn hook_damage_plus_mantra(engine: &CombatEngine, _card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    DamageModifier {
        base_damage_bonus: engine.state.mantra_gained,
        ..DamageModifier::default()
    }
}

/// Perfected Strike: +N damage per Strike card in all piles.
pub fn hook_perfected_strike(engine: &CombatEngine, card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    let per_strike = card.base_magic.max(1);
    let strike_count = engine.state.hand.iter()
        .chain(engine.state.draw_pile.iter())
        .chain(engine.state.discard_pile.iter())
        .chain(engine.state.exhaust_pile.iter())
        .filter(|c| engine.card_registry.is_strike(c.def_id))
        .count() as i32;
    DamageModifier {
        base_damage_bonus: per_strike * strike_count,
        ..DamageModifier::default()
    }
}

/// Rampage: scaling damage bonus from status counter.
pub fn hook_rampage(_engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    let current_damage = if card_inst.misc >= 0 {
        card_inst.misc as i32
    } else {
        card.base_damage
    };
    DamageModifier {
        base_damage_bonus: current_damage - card.base_damage,
        ..DamageModifier::default()
    }
}

/// Glass Knife: damage decreases each play (negative bonus from penalty counter).
pub fn hook_glass_knife(_engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    let current_damage = if card_inst.misc >= 0 {
        card_inst.misc as i32
    } else {
        card.base_damage
    };
    DamageModifier {
        base_damage_bonus: current_damage - card.base_damage,
        ..DamageModifier::default()
    }
}

/// Ritual Dagger: scaling damage bonus from kills.
pub fn hook_ritual_dagger(_engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    let current_damage = if card_inst.misc >= 0 {
        card_inst.misc as i32
    } else {
        card.base_damage
    };
    DamageModifier {
        base_damage_bonus: current_damage - card.base_damage,
        ..DamageModifier::default()
    }
}

/// Searing Blow: +4 bonus when upgraded (simplified for MCTS).
pub fn hook_searing_blow(_engine: &CombatEngine, _card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    let bonus = if card_inst.flags & 0x04 != 0 { 4 } else { 0 };
    DamageModifier {
        base_damage_bonus: bonus,
        ..DamageModifier::default()
    }
}

/// Windmill Strike: damage bonus from retaining (reads WINDMILL_STRIKE_BONUS status).
pub fn hook_windmill_strike_damage(engine: &CombatEngine, _card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    DamageModifier {
        base_damage_bonus: engine.state.player.status(sid::WINDMILL_STRIKE_BONUS),
        ..DamageModifier::default()
    }
}

/// damage_random_x_times: skip generic damage loop (card handles own hits).
pub fn hook_damage_random_x_times(_engine: &CombatEngine, _card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    DamageModifier {
        skip_generic_damage: true,
        ..DamageModifier::default()
    }
}

/// Claw: bonus damage from CLAW_BONUS status (incremented each time any Claw is played).
pub fn hook_claw_damage(engine: &CombatEngine, _card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    DamageModifier {
        base_damage_bonus: engine.state.player.status(sid::CLAW_BONUS),
        ..DamageModifier::default()
    }
}

/// Mind Blast / Blizzard / Thunder Strike: skip generic damage loop (handled by complex_hook).
pub fn hook_skip_generic_damage(_engine: &CombatEngine, _card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    DamageModifier {
        skip_generic_damage: true,
        ..DamageModifier::default()
    }
}
