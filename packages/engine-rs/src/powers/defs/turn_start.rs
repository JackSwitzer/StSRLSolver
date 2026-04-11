//! Turn-start power definitions.
//!
//! Powers that trigger at the start of the player's turn.

use crate::effects::declarative::{AmountSource, Effect, Pile, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
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
};

// ===========================================================================
// Deva Form — TurnStart: gain energy (escalating)
// NOTE: Deva Form escalates each turn (amt, then amt+1, etc.).
// The escalation is a side-effect mutation, so this is approximated
// here as gaining energy = current stacks. The actual escalation
// (incrementing the status value) needs complex_hook or engine support.
// ===========================================================================

static DEVA_FORM_EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::GainEnergy(
    AmountSource::StatusValue(sid::DEVA_FORM),
))];

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
    fn test_all_turn_start_defs_have_correct_trigger() {
        let defs = [
            &DEF_DEMON_FORM, &DEF_NOXIOUS_FUMES, &DEF_BRUTALITY,
            &DEF_BERSERK, &DEF_INFINITE_BLADES, &DEF_BATTLE_HYMN,
            &DEF_DEVOTION, &DEF_WRAITH_FORM, &DEF_DEVA_FORM,
            &DEF_HELLO_WORLD, &DEF_MAGNETISM,
        ];
        for def in &defs {
            assert_eq!(def.kind, EntityKind::Power);
            assert!(!def.triggers.is_empty());
            assert_eq!(def.triggers[0].trigger, Trigger::TurnStart);
        }
    }
}
