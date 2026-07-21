//! modify_damage hooks — adjust damage calculation before the generic damage loop.

use crate::cards::CardDef;
use crate::combat_types::CardInstance;
use crate::engine::CombatEngine;
use super::types::DamageModifier;

/// Heavy Blade: multiply strength contribution (3x base, 5x upgraded).
pub fn hook_heavy_blade(engine: &CombatEngine, card: &CardDef, _card_inst: CardInstance) -> DamageModifier {
    // HeavyBlade.applyPowers/calculateCardDamage temporarily multiply the
    // actual StrengthPower amount, including negative Strength.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/HeavyBlade.java
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

/// Perfected Strike: +N damage per Strike card in hand, draw, and discard.
pub fn hook_perfected_strike(engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    let per_strike = card.base_magic.max(1);
    let mut strike_count = engine.state.hand.iter()
        .chain(engine.state.draw_pile.iter())
        .chain(engine.state.discard_pile.iter())
        .filter(|c| engine.card_registry.is_strike(c.def_id))
        .count() as i32;

    // Java calculates ordinary hand plays before AbstractPlayer removes the
    // played card, so Perfected Strike counts itself. Autoplayed copies live in
    // limbo and are not part of countCards(), matching the transient flag.
    // Exhaust is intentionally excluded by PerfectedStrike.countCards().
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/PerfectedStrike.java
    if card_inst.flags & CardInstance::FLAG_AUTOPLAY == 0
        && engine.card_registry.is_strike(card_inst.def_id)
    {
        strike_count += 1;
    }
    DamageModifier {
        base_damage_bonus: per_strike * strike_count,
        ..DamageModifier::default()
    }
}

/// Rampage: scaling damage bonus from status counter.
pub fn hook_rampage(_engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    let current_damage = if card_inst.misc >= 0 {
        card_inst.misc
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
        card_inst.misc
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
        card_inst.misc
    } else {
        card.base_damage
    };
    DamageModifier {
        base_damage_bonus: current_damage - card.base_damage,
        ..DamageModifier::default()
    }
}

/// Searing Blow's nth upgrade adds `4 + previous timesUpgraded` damage.
pub fn hook_searing_blow(_engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    // Starting from 12, n upgrades total 12 + 4n + n(n-1)/2. The instance misc
    // stores n; named `Searing Blow+` cards without explicit state mean n=1.
    // Java: reference/extracted/methods/card/SearingBlow.java
    let upgrades = if card_inst.misc >= 0 {
        card_inst.misc
    } else if card.id == "Searing Blow+" || card_inst.is_upgraded() {
        1
    } else {
        0
    };
    let current_damage = 12 + 4 * upgrades + upgrades * (upgrades - 1) / 2;
    DamageModifier {
        base_damage_bonus: current_damage - card.base_damage,
        ..DamageModifier::default()
    }
}

/// Windmill Strike: damage growth belongs to the retained card instance.
pub fn hook_windmill_strike_damage(_engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    let current_damage = if card_inst.misc >= 0 {
        card_inst.misc
    } else {
        card.base_damage
    };
    DamageModifier {
        base_damage_bonus: current_damage - card.base_damage,
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

/// Claw: GashAction stores the mutable base damage on each affected instance.
pub fn hook_claw_damage(_engine: &CombatEngine, card: &CardDef, card_inst: CardInstance) -> DamageModifier {
    let current_damage = if card_inst.misc >= 0 {
        card_inst.misc
    } else {
        card.base_damage
    };
    DamageModifier {
        base_damage_bonus: current_damage - card.base_damage,
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
