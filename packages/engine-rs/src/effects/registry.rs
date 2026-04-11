//! Card effect registry — static dispatch tables for card effect hooks.
//!
//! Mirrors the proven `powers/registry.rs` pattern: a static array of entries
//! with optional fn pointers per hook type. Dispatch functions iterate the
//! registry, check the card's EffectFlags bitset, and call matching hooks.

use crate::combat_types::CardInstance;
use crate::cards::{CardDef, CardRegistry};
use crate::engine::CombatEngine;
use crate::state::CombatState;

use std::sync::OnceLock;
use std::collections::HashMap;

use super::flags::EffectFlags;
use super::types::*;

// ===========================================================================
// Bit index constants — named aliases for bit positions in EffectFlags
// ===========================================================================

// can_play hooks (bits 0-3)
pub const BIT_UNPLAYABLE: u8 = 0;
pub const BIT_ONLY_ATTACK_IN_HAND: u8 = 1;
pub const BIT_ONLY_ATTACKS_IN_HAND: u8 = 2;
pub const BIT_ONLY_EMPTY_DRAW: u8 = 3;
// modify_cost hooks (bits 4-7)
pub const BIT_COST_REDUCE_ON_HP_LOSS: u8 = 4;
pub const BIT_REDUCE_COST_PER_POWER: u8 = 5;
pub const BIT_COST_REDUCE_ON_DISCARD: u8 = 6;
pub const BIT_COST_INCREASE_ON_HP_LOSS: u8 = 7;
// on_retain hooks (bits 8-10)
pub const BIT_REDUCE_COST_ON_RETAIN: u8 = 8;
pub const BIT_GROW_BLOCK_ON_RETAIN: u8 = 9;
pub const BIT_GROW_DAMAGE_ON_RETAIN: u8 = 10;
// on_draw hooks (bits 11-12)
pub const BIT_LOSE_ENERGY_ON_DRAW: u8 = 11;
pub const BIT_COPY_ON_DRAW: u8 = 12;
// on_discard hooks (bits 13-14)
pub const BIT_DRAW_ON_DISCARD: u8 = 13;
pub const BIT_ENERGY_ON_DISCARD: u8 = 14;
// post_play_dest hooks (bits 15-16)
pub const BIT_SHUFFLE_SELF_INTO_DRAW: u8 = 15;
pub const BIT_END_TURN: u8 = 16;
// modify_damage hooks (bits 17-25)
pub const BIT_HEAVY_BLADE: u8 = 17;
pub const BIT_DAMAGE_EQUALS_BLOCK: u8 = 18;
pub const BIT_DAMAGE_PLUS_MANTRA: u8 = 19;
pub const BIT_PERFECTED_STRIKE: u8 = 20;
pub const BIT_RAMPAGE: u8 = 21;
pub const BIT_GLASS_KNIFE: u8 = 22;
pub const BIT_RITUAL_DAGGER: u8 = 23;
pub const BIT_SEARING_BLOW: u8 = 24;
pub const BIT_DAMAGE_RANDOM_X_TIMES: u8 = 25;
// on_draw hooks (continued, bits 26+)
pub const BIT_DEUS_EX_MACHINA: u8 = 26;
// modify_damage hooks (continued, bits 27-30)
pub const BIT_CLAW_SCALING: u8 = 27;
pub const BIT_DAMAGE_PER_FROST: u8 = 28;
pub const BIT_DAMAGE_PER_LIGHTNING: u8 = 29;
pub const BIT_DAMAGE_FROM_DRAW_PILE: u8 = 30;

// ===========================================================================
// Hook function type aliases
// ===========================================================================

/// Can this card be played? Return false to block.
pub type CanPlayFn = fn(&CombatState, &CardDef, CardInstance, &CardRegistry) -> bool;

/// Modify the effective cost of a card. Returns the new cost.
pub type ModifyCostFn = fn(&CombatState, &CardDef, CardInstance, i32) -> i32;

/// Pre-damage modifier. Returns adjustments to base damage / strength mult.
pub type ModifyDamageFn = fn(&CombatEngine, &CardDef, CardInstance) -> DamageModifier;

/// On-play hook with full engine access (for complex/choice effects).
pub type OnPlayFn = fn(&mut CombatEngine, &CardPlayContext);

/// On-retain hook (end of turn, card stays in hand).
pub type OnRetainFn = fn(&mut CombatEngine, &mut CardInstance, &CardDef);

/// On-draw hook (card just entered hand from draw pile).
pub type OnDrawFn = fn(&mut CombatEngine, CardInstance);

/// On-discard hook (card moved from hand to discard).
pub type OnDiscardFn = fn(&mut CombatEngine, CardInstance) -> OnDiscardEffect;

/// Post-play destination override.
pub type PostPlayDestFn = fn(&CardDef) -> PostPlayDestination;

// ===========================================================================
// Registry Entry
// ===========================================================================

pub struct CardEffectEntry {
    /// The effect tag string (e.g., "heavy_blade"). Must match CardDef.effects.
    pub tag: &'static str,
    /// Bit position in EffectFlags (0..255). Assigned sequentially.
    pub bit_index: u8,

    // Hook fn pointers — None means this tag doesn't fire on that trigger.
    pub can_play: Option<CanPlayFn>,
    pub modify_cost: Option<ModifyCostFn>,
    pub modify_damage: Option<ModifyDamageFn>,
    pub on_play: Option<OnPlayFn>,
    pub on_retain: Option<OnRetainFn>,
    pub on_draw: Option<OnDrawFn>,
    pub on_discard: Option<OnDiscardFn>,
    pub post_play_dest: Option<PostPlayDestFn>,
}

impl CardEffectEntry {
    pub const NONE: Self = Self {
        tag: "",
        bit_index: 0,
        can_play: None,
        modify_cost: None,
        modify_damage: None,
        on_play: None,
        on_retain: None,
        on_draw: None,
        on_discard: None,
        post_play_dest: None,
    };
}

// ===========================================================================
// The Registry — static table, populated incrementally as hooks are migrated
// ===========================================================================

/// Card effect registry. Each entry is one effect tag with its hook fn pointers.
/// Entries are added as effects are migrated from card_effects.rs.
pub static CARD_EFFECT_REGISTRY: &[CardEffectEntry] = &[
    // ===== can_play hooks (bits 0-3) =====
    CardEffectEntry {
        tag: "unplayable",
        bit_index: 0,
        can_play: Some(super::hooks_can_play::hook_unplayable),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "only_attack_in_hand",
        bit_index: 1,
        can_play: Some(super::hooks_can_play::hook_only_attack_in_hand),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "only_attacks_in_hand",
        bit_index: 2,
        can_play: Some(super::hooks_can_play::hook_only_attacks_in_hand),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "only_empty_draw",
        bit_index: 3,
        can_play: Some(super::hooks_can_play::hook_only_empty_draw),
        ..CardEffectEntry::NONE
    },
    // ===== modify_cost hooks (bits 4-7) =====
    CardEffectEntry {
        tag: "cost_reduce_on_hp_loss",
        bit_index: 4,
        modify_cost: Some(super::hooks_cost::hook_cost_reduce_on_hp_loss),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "reduce_cost_per_power",
        bit_index: 5,
        modify_cost: Some(super::hooks_cost::hook_reduce_cost_per_power),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "cost_reduce_on_discard",
        bit_index: 6,
        modify_cost: Some(super::hooks_cost::hook_cost_reduce_on_discard),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "cost_increase_on_hp_loss",
        bit_index: 7,
        modify_cost: Some(super::hooks_cost::hook_cost_increase_on_hp_loss),
        ..CardEffectEntry::NONE
    },
    // ===== on_retain hooks (bits 8-10) =====
    CardEffectEntry {
        tag: "reduce_cost_on_retain",
        bit_index: 8,
        on_retain: Some(super::hooks_retain::hook_reduce_cost_on_retain),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "grow_block_on_retain",
        bit_index: 9,
        on_retain: Some(super::hooks_retain::hook_grow_block_on_retain),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "grow_damage_on_retain",
        bit_index: 10,
        on_retain: Some(super::hooks_retain::hook_grow_damage_on_retain),
        modify_damage: Some(super::hooks_damage::hook_windmill_strike_damage),
        ..CardEffectEntry::NONE
    },
    // ===== on_draw hooks (bits 11-12) =====
    CardEffectEntry {
        tag: "lose_energy_on_draw",
        bit_index: 11,
        on_draw: Some(super::hooks_draw::hook_lose_energy_on_draw),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "copy_on_draw",
        bit_index: 12,
        on_draw: Some(super::hooks_draw::hook_copy_on_draw),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "deus_ex_machina",
        bit_index: 26,
        on_draw: Some(super::hooks_draw::hook_deus_ex_machina_on_draw),
        ..CardEffectEntry::NONE
    },
    // ===== on_discard hooks (bits 13-14) =====
    CardEffectEntry {
        tag: "draw_on_discard",
        bit_index: 13,
        on_discard: Some(super::hooks_discard::hook_draw_on_discard),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "energy_on_discard",
        bit_index: 14,
        on_discard: Some(super::hooks_discard::hook_energy_on_discard),
        ..CardEffectEntry::NONE
    },
    // ===== post_play_dest hooks (bits 15-16) =====
    CardEffectEntry {
        tag: "shuffle_self_into_draw",
        bit_index: 15,
        post_play_dest: Some(super::hooks_dest::hook_shuffle_self_into_draw),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "end_turn",
        bit_index: 16,
        post_play_dest: Some(super::hooks_dest::hook_end_turn),
        ..CardEffectEntry::NONE
    },
    // ===== modify_damage hooks (bits 17-25) =====
    CardEffectEntry {
        tag: "heavy_blade",
        bit_index: 17,
        modify_damage: Some(super::hooks_damage::hook_heavy_blade),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "damage_equals_block",
        bit_index: 18,
        modify_damage: Some(super::hooks_damage::hook_damage_equals_block),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "damage_plus_mantra",
        bit_index: 19,
        modify_damage: Some(super::hooks_damage::hook_damage_plus_mantra),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "perfected_strike",
        bit_index: 20,
        modify_damage: Some(super::hooks_damage::hook_perfected_strike),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "rampage",
        bit_index: 21,
        modify_damage: Some(super::hooks_damage::hook_rampage),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "glass_knife",
        bit_index: 22,
        modify_damage: Some(super::hooks_damage::hook_glass_knife),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "ritual_dagger",
        bit_index: 23,
        modify_damage: Some(super::hooks_damage::hook_ritual_dagger),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "searing_blow",
        bit_index: 24,
        modify_damage: Some(super::hooks_damage::hook_searing_blow),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "damage_random_x_times",
        bit_index: 25,
        modify_damage: Some(super::hooks_damage::hook_damage_random_x_times),
        ..CardEffectEntry::NONE
    },
    // ===== Defect damage modifiers (bits 27-30) =====
    CardEffectEntry {
        tag: "claw_scaling",
        bit_index: 27,
        modify_damage: Some(super::hooks_damage::hook_claw_damage),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "damage_per_frost_channeled",
        bit_index: 28,
        modify_damage: Some(super::hooks_damage::hook_skip_generic_damage),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "damage_per_lightning_channeled",
        bit_index: 29,
        modify_damage: Some(super::hooks_damage::hook_skip_generic_damage),
        ..CardEffectEntry::NONE
    },
    CardEffectEntry {
        tag: "damage_from_draw_pile",
        bit_index: 30,
        modify_damage: Some(super::hooks_damage::hook_skip_generic_damage),
        ..CardEffectEntry::NONE
    },
];

// ===========================================================================
// Precomputed hook masks — one per hook type
// ===========================================================================

// ===========================================================================
// Precomputed hook masks — initialized once on first access
// ===========================================================================

static HOOK_MASKS: OnceLock<HookMasks> = OnceLock::new();
static TAG_TO_BIT_MAP: OnceLock<HashMap<&'static str, u8>> = OnceLock::new();

struct HookMasks {
    can_play: EffectFlags,
    modify_cost: EffectFlags,
    modify_damage: EffectFlags,
    on_play: EffectFlags,
    on_retain: EffectFlags,
    on_draw: EffectFlags,
    on_discard: EffectFlags,
    post_play_dest: EffectFlags,
}

fn init_hook_masks() -> HookMasks {
    HookMasks {
        can_play: build_hook_mask(|e| e.can_play.is_some()),
        modify_cost: build_hook_mask(|e| e.modify_cost.is_some()),
        modify_damage: build_hook_mask(|e| e.modify_damage.is_some()),
        on_play: build_hook_mask(|e| e.on_play.is_some()),
        on_retain: build_hook_mask(|e| e.on_retain.is_some()),
        on_draw: build_hook_mask(|e| e.on_draw.is_some()),
        on_discard: build_hook_mask(|e| e.on_discard.is_some()),
        post_play_dest: build_hook_mask(|e| e.post_play_dest.is_some()),
    }
}

fn masks() -> &'static HookMasks {
    HOOK_MASKS.get_or_init(init_hook_masks)
}

fn tag_to_bit() -> &'static HashMap<&'static str, u8> {
    TAG_TO_BIT_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        for entry in CARD_EFFECT_REGISTRY.iter() {
            if !entry.tag.is_empty() {
                map.insert(entry.tag, entry.bit_index);
            }
        }
        map
    })
}

fn build_hook_mask(predicate: fn(&CardEffectEntry) -> bool) -> EffectFlags {
    let mut mask = EffectFlags::EMPTY;
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if predicate(entry) {
            mask.set(entry.bit_index);
        }
    }
    mask
}

// ===========================================================================
// EffectFlags computation for CardRegistry
// ===========================================================================

/// Build an EffectFlags bitset from a CardDef's string effect tags.
/// Called once per card at CardRegistry::new() time.
pub fn build_effect_flags(effects: &[&str]) -> EffectFlags {
    let mut flags = EffectFlags::EMPTY;
    let tag_map = tag_to_bit();
    for tag in effects {
        if let Some(&bit) = tag_map.get(tag) {
            flags.set(bit);
        }
        // Tags not in registry are silently ignored — they're still handled
        // by the old card_effects.rs path during migration.
    }
    flags
}

// ===========================================================================
// Dispatch functions — called from engine.rs at each hook point
// ===========================================================================

/// Check if a card can be played (all can_play hooks must return true).
pub fn dispatch_can_play(
    state: &CombatState,
    card: &CardDef,
    card_inst: CardInstance,
    card_flags: EffectFlags,
    registry: &CardRegistry,
) -> bool {
    if !card_flags.intersects(&masks().can_play) {
        return true;
    }
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if let Some(hook) = entry.can_play {
            if card_flags.has(entry.bit_index) {
                if !hook(state, card, card_inst, registry) {
                    return false;
                }
            }
        }
    }
    true
}

/// Modify the effective cost of a card through all cost hooks.
pub fn dispatch_modify_cost(
    state: &CombatState,
    card: &CardDef,
    card_inst: CardInstance,
    card_flags: EffectFlags,
    base_cost: i32,
) -> i32 {
    if !card_flags.intersects(&masks().modify_cost) {
        return base_cost;
    }
    let mut cost = base_cost;
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if let Some(hook) = entry.modify_cost {
            if card_flags.has(entry.bit_index) {
                cost = hook(state, card, card_inst, cost);
            }
        }
    }
    cost
}

/// Compute pre-damage modifiers (Heavy Blade, Perfected Strike, Body Slam, etc.)
pub fn dispatch_modify_damage(
    engine: &CombatEngine,
    card: &CardDef,
    card_inst: CardInstance,
    card_flags: EffectFlags,
) -> DamageModifier {
    if !card_flags.intersects(&masks().modify_damage) {
        return DamageModifier::default();
    }
    let mut out = DamageModifier::default();
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if let Some(hook) = entry.modify_damage {
            if card_flags.has(entry.bit_index) {
                out.merge(hook(engine, card, card_inst));
            }
        }
    }
    out
}

/// Dispatch on_play hooks. Returns early if engine enters AwaitingChoice.
pub fn dispatch_on_play(engine: &mut CombatEngine, ctx: &CardPlayContext, card_flags: EffectFlags) {
    if !card_flags.intersects(&masks().on_play) {
        return;
    }
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if let Some(hook) = entry.on_play {
            if card_flags.has(entry.bit_index) {
                hook(engine, ctx);
                // If a hook triggered a choice (Meditate, Concentrate, etc.), stop
                if engine.phase == crate::engine::CombatPhase::AwaitingChoice {
                    return;
                }
            }
        }
    }
}

/// Dispatch on_retain hooks for a retained card at end of turn.
pub fn dispatch_on_retain(
    engine: &mut CombatEngine,
    card_inst: &mut CardInstance,
    card: &CardDef,
    card_flags: EffectFlags,
) {
    if !card_flags.intersects(&masks().on_retain) {
        return;
    }
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if let Some(hook) = entry.on_retain {
            if card_flags.has(entry.bit_index) {
                hook(engine, card_inst, card);
            }
        }
    }
}

/// Dispatch on_draw hooks for a card just drawn.
pub fn dispatch_on_draw(engine: &mut CombatEngine, card_inst: CardInstance, card_flags: EffectFlags) {
    if !card_flags.intersects(&masks().on_draw) {
        return;
    }
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if let Some(hook) = entry.on_draw {
            if card_flags.has(entry.bit_index) {
                hook(engine, card_inst);
            }
        }
    }
}

/// Dispatch on_discard hooks. Returns merged effect.
pub fn dispatch_on_discard(
    engine: &mut CombatEngine,
    card_inst: CardInstance,
    card_flags: EffectFlags,
) -> OnDiscardEffect {
    let mut out = OnDiscardEffect::default();
    if !card_flags.intersects(&masks().on_discard) {
        return out;
    }
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if let Some(hook) = entry.on_discard {
            if card_flags.has(entry.bit_index) {
                out.merge(hook(engine, card_inst));
            }
        }
    }
    out
}

/// Get post-play destination override.
pub fn dispatch_post_play_dest(card: &CardDef, card_flags: EffectFlags) -> PostPlayDestination {
    if !card_flags.intersects(&masks().post_play_dest) {
        return PostPlayDestination::Normal;
    }
    for entry in CARD_EFFECT_REGISTRY.iter() {
        if let Some(hook) = entry.post_play_dest {
            if card_flags.has(entry.bit_index) {
                let dest = hook(card);
                if dest != PostPlayDestination::Normal {
                    return dest;
                }
            }
        }
    }
    PostPlayDestination::Normal
}
