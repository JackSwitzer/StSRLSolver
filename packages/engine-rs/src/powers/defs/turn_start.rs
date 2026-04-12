//! Turn-start power definitions.
//!
//! Powers that trigger at the start of the player's turn.

use crate::effects::declarative::{AmountSource, Effect, Pile, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

// ===========================================================================
// Demon Form — TurnStart: gain Strength equal to stacks
// ===========================================================================

static DEMON_FORM_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player,
    sid::STRENGTH,
    AmountSource::StatusValue(sid::DEMON_FORM),
))];

static DEMON_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &DEMON_FORM_EFFECTS,
    counter: None,
}];

pub static DEF_DEMON_FORM: EntityDef = EntityDef {
    id: "demon_form",
    name: "Demon Form",
    kind: EntityKind::Power,
    triggers: &DEMON_FORM_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DEMON_FORM),
};

// ===========================================================================
// Noxious Fumes — TurnStart: poison all enemies
// ===========================================================================

static NOXIOUS_FUMES_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::AllEnemies,
    sid::POISON,
    AmountSource::StatusValue(sid::NOXIOUS_FUMES),
))];

static NOXIOUS_FUMES_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &NOXIOUS_FUMES_EFFECTS,
    counter: None,
}];

pub static DEF_NOXIOUS_FUMES: EntityDef = EntityDef {
    id: "noxious_fumes",
    name: "Noxious Fumes",
    kind: EntityKind::Power,
    triggers: &NOXIOUS_FUMES_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::NOXIOUS_FUMES),
};

// ===========================================================================
// Brutality — TurnStart: draw 1 card, lose HP equal to stacks
// ===========================================================================

// Brutality loses HP equal to stacks. DealDamage(Player, ...) routes through
// player_lose_hp which handles the HP loss correctly (ModifyHp with positive
// StatusValue would heal instead).
static BRUTALITY_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::DrawCards(AmountSource::StatusValue(sid::BRUTALITY))),
    Effect::Simple(SimpleEffect::DealDamage(Target::Player, AmountSource::StatusValue(sid::BRUTALITY))),
];

static BRUTALITY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &BRUTALITY_EFFECTS,
    counter: None,
}];

pub static DEF_BRUTALITY: EntityDef = EntityDef {
    id: "brutality",
    name: "Brutality",
    kind: EntityKind::Power,
    triggers: &BRUTALITY_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::BRUTALITY),
};

// ===========================================================================
// Berserk — TurnStart: gain energy equal to stacks
// ===========================================================================

static BERSERK_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainEnergy(
    AmountSource::StatusValue(sid::BERSERK),
))];

static BERSERK_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &BERSERK_EFFECTS,
    counter: None,
}];

pub static DEF_BERSERK: EntityDef = EntityDef {
    id: "berserk",
    name: "Berserk",
    kind: EntityKind::Power,
    triggers: &BERSERK_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::BERSERK),
};

// ===========================================================================
// Infinite Blades — TurnStart: add Shiv(s) to hand
// ===========================================================================

static INFINITE_BLADES_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Shiv",
    Pile::Hand,
    AmountSource::StatusValue(sid::INFINITE_BLADES),
))];

static INFINITE_BLADES_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &INFINITE_BLADES_EFFECTS,
    counter: None,
}];

pub static DEF_INFINITE_BLADES: EntityDef = EntityDef {
    id: "infinite_blades",
    name: "Infinite Blades",
    kind: EntityKind::Power,
    triggers: &INFINITE_BLADES_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::INFINITE_BLADES),
};

// ===========================================================================
// Battle Hymn — TurnStart: add Smite(s) to hand
// ===========================================================================

static BATTLE_HYMN_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Smite",
    Pile::Hand,
    AmountSource::StatusValue(sid::BATTLE_HYMN),
))];

static BATTLE_HYMN_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &BATTLE_HYMN_EFFECTS,
    counter: None,
}];

pub static DEF_BATTLE_HYMN: EntityDef = EntityDef {
    id: "battle_hymn",
    name: "Battle Hymn",
    kind: EntityKind::Power,
    triggers: &BATTLE_HYMN_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::BATTLE_HYMN),
};

// ===========================================================================
// Devotion — TurnStart: gain mantra equal to stacks
// ===========================================================================

static DEVOTION_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainMantra(
    AmountSource::StatusValue(sid::DEVOTION),
))];

static DEVOTION_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &DEVOTION_EFFECTS,
    counter: None,
}];

pub static DEF_DEVOTION: EntityDef = EntityDef {
    id: "devotion",
    name: "Devotion",
    kind: EntityKind::Power,
    triggers: &DEVOTION_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DEVOTION),
};

// ===========================================================================
// Wraith Form — TurnStart: lose 1 Dexterity each turn
// ===========================================================================

static WRAITH_FORM_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player,
    sid::DEXTERITY,
    AmountSource::Fixed(-1),
))];

static WRAITH_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &WRAITH_FORM_EFFECTS,
    counter: None,
}];

pub static DEF_WRAITH_FORM: EntityDef = EntityDef {
    id: "wraith_form",
    name: "Wraith Form",
    kind: EntityKind::Power,
    triggers: &WRAITH_FORM_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::WRAITH_FORM),
};

// ===========================================================================
// Deva Form — TurnStart: gain energy (escalating)
// NOTE: Deva Form escalates each turn (amt, then amt+1, etc.).
// The escalation is a side-effect mutation, so this is approximated
// here as gaining energy = current stacks. The actual escalation
// (incrementing the status value) needs complex_hook or engine support.
// ===========================================================================

static DEVA_FORM_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::GainEnergy(
        AmountSource::StatusValue(sid::DEVA_FORM),
    )),
    // Escalate: increment status so next turn grants more energy
    Effect::Simple(SimpleEffect::AddStatus(
        Target::Player,
        sid::DEVA_FORM,
        AmountSource::Fixed(1),
    )),
];

static DEVA_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &DEVA_FORM_EFFECTS,
    counter: None,
}];

pub static DEF_DEVA_FORM: EntityDef = EntityDef {
    id: "deva_form",
    name: "Deva Form",
    kind: EntityKind::Power,
    triggers: &DEVA_FORM_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DEVA_FORM),
};

// ===========================================================================
// Hello World — TurnStart: add Strike to hand (MCTS approximation)
// ===========================================================================

static HELLO_WORLD_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Strike",
    Pile::Hand,
    AmountSource::StatusValue(sid::HELLO_WORLD),
))];

static HELLO_WORLD_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &HELLO_WORLD_EFFECTS,
    counter: None,
}];

pub static DEF_HELLO_WORLD: EntityDef = EntityDef {
    id: "hello_world",
    name: "Hello World",
    kind: EntityKind::Power,
    triggers: &HELLO_WORLD_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::HELLO_WORLD),
};

// ===========================================================================
// Magnetism — TurnStart: add Strike to hand (MCTS approximation)
// ===========================================================================

static MAGNETISM_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Strike",
    Pile::Hand,
    AmountSource::Fixed(1),
))];

static MAGNETISM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &MAGNETISM_EFFECTS,
    counter: None,
}];

pub static DEF_MAGNETISM: EntityDef = EntityDef {
    id: "magnetism",
    name: "Magnetism",
    kind: EntityKind::Power,
    triggers: &MAGNETISM_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::MAGNETISM),
};

// ===========================================================================
// Creative AI — TurnStart: add random Power card to hand (complex_hook)
// ===========================================================================

fn hook_noop(_engine: &mut CombatEngine, _ctx: &TriggerContext) {}

pub static DEF_CREATIVE_AI: EntityDef = EntityDef {
    id: "creative_ai",
    name: "Creative AI",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::CREATIVE_AI),
};

// ===========================================================================
// Doppelganger Draw — TurnStart: draw N cards (one-shot, consumed)
// ===========================================================================

static DOPPELGANGER_DRAW_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DrawCards(
    AmountSource::StatusValue(sid::DOPPELGANGER_DRAW),
))];

static DOPPELGANGER_DRAW_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &DOPPELGANGER_DRAW_EFFECTS,
    counter: None,
}];

pub static DEF_DOPPELGANGER_DRAW: EntityDef = EntityDef {
    id: "doppelganger_draw",
    name: "Doppelganger (Draw)",
    kind: EntityKind::Power,
    triggers: &DOPPELGANGER_DRAW_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DOPPELGANGER_DRAW),
};

// ===========================================================================
// Doppelganger Energy — TurnStart: gain N energy (one-shot, consumed)
// ===========================================================================

static DOPPELGANGER_ENERGY_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainEnergy(
    AmountSource::StatusValue(sid::DOPPELGANGER_ENERGY),
))];

static DOPPELGANGER_ENERGY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &DOPPELGANGER_ENERGY_EFFECTS,
    counter: None,
}];

pub static DEF_DOPPELGANGER_ENERGY: EntityDef = EntityDef {
    id: "doppelganger_energy",
    name: "Doppelganger (Energy)",
    kind: EntityKind::Power,
    triggers: &DOPPELGANGER_ENERGY_TRIGGERS,
    complex_hook: None,
    status_guard: Some(sid::DOPPELGANGER_ENERGY),
};

// ===========================================================================
// Enter Divinity — TurnStart: enter Divinity stance (one-shot flag)
// ===========================================================================

pub static DEF_ENTER_DIVINITY: EntityDef = EntityDef {
    id: "enter_divinity",
    name: "Enter Divinity",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::ENTER_DIVINITY),
};

// ===========================================================================
// Mayhem — TurnStart: play top card of draw pile for free (complex_hook)
// ===========================================================================

pub static DEF_MAYHEM: EntityDef = EntityDef {
    id: "mayhem",
    name: "Mayhem",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::MAYHEM),
};

// ===========================================================================
// Tools of the Trade — TurnStart: draw 1, then discard 1 (complex_hook)
// ===========================================================================

pub static DEF_TOOLS_OF_THE_TRADE: EntityDef = EntityDef {
    id: "tools_of_the_trade",
    name: "Tools of the Trade",
    kind: EntityKind::Power,
    triggers: &[],
    complex_hook: Some(hook_noop),
    status_guard: Some(sid::TOOLS_OF_THE_TRADE),
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demon_form_def() {
        assert_eq!(DEF_DEMON_FORM.triggers.len(), 1);
        assert_eq!(DEF_DEMON_FORM.triggers[0].trigger, Trigger::TurnStart);
        assert_eq!(DEF_DEMON_FORM.triggers[0].condition, TriggerCondition::Always);
        assert!(DEF_DEMON_FORM.complex_hook.is_none());
    }

    #[test]
    fn test_brutality_has_two_effects() {
        assert_eq!(DEF_BRUTALITY.triggers[0].effects.len(), 2);
    }

    #[test]
    fn test_all_simple_turn_start_defs_have_correct_trigger() {
        let defs = [
            &DEF_DEMON_FORM, &DEF_NOXIOUS_FUMES, &DEF_BRUTALITY,
            &DEF_BERSERK, &DEF_INFINITE_BLADES, &DEF_BATTLE_HYMN,
            &DEF_DEVOTION, &DEF_WRAITH_FORM, &DEF_DEVA_FORM,
            &DEF_HELLO_WORLD, &DEF_MAGNETISM,
            &DEF_DOPPELGANGER_DRAW, &DEF_DOPPELGANGER_ENERGY,
        ];
        for def in &defs {
            assert_eq!(def.kind, EntityKind::Power);
            assert!(!def.triggers.is_empty());
            assert_eq!(def.triggers[0].trigger, Trigger::TurnStart);
        }
    }

    #[test]
    fn test_complex_turn_start_defs_have_hooks() {
        let defs = [
            &DEF_CREATIVE_AI, &DEF_ENTER_DIVINITY,
            &DEF_MAYHEM, &DEF_TOOLS_OF_THE_TRADE,
        ];
        for def in &defs {
            assert_eq!(def.kind, EntityKind::Power);
            assert!(def.complex_hook.is_some());
        }
    }
}
