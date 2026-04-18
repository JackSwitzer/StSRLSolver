//! Turn-start power definitions.
//!
//! Powers that trigger at the start of the player's turn.

use crate::effects::declarative::{AmountSource, Effect, Pile, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::state::Stance;
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
// Creative AI — TurnStartPostDraw: add random Power card to hand
// Current MCTS approximation is preserved exactly by adding "Smite".
// ===========================================================================

static EMPTY_EFFECTS: [Effect; 0] = [];

static CREATIVE_AI_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_creative_ai(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let creative_ai = engine.state.player.status(sid::CREATIVE_AI);
    for _ in 0..creative_ai {
        if engine.state.hand.len() >= 10 {
            break;
        }
        let smite_id = engine.temp_card("Smite");
        engine.state.hand.push(smite_id);
    }
}

pub static DEF_CREATIVE_AI: EntityDef = EntityDef {
    id: "creative_ai",
    name: "Creative AI",
    kind: EntityKind::Power,
    triggers: &CREATIVE_AI_TRIGGERS,
    complex_hook: Some(hook_creative_ai),
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
// Enter Divinity — TurnStartPostDraw: enter Divinity stance (one-shot flag)
// ===========================================================================

static ENTER_DIVINITY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_enter_divinity(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    if engine.state.player.status(sid::ENTER_DIVINITY) > 0 {
        engine.state.player.set_status(sid::ENTER_DIVINITY, 0);
        engine.change_stance(Stance::Divinity);
    }
}

pub static DEF_ENTER_DIVINITY: EntityDef = EntityDef {
    id: "enter_divinity",
    name: "Enter Divinity",
    kind: EntityKind::Power,
    triggers: &ENTER_DIVINITY_TRIGGERS,
    complex_hook: Some(hook_enter_divinity),
    status_guard: Some(sid::ENTER_DIVINITY),
};

// ===========================================================================
// Mayhem — TurnStartPostDraw: move top draw card(s) into hand
// Preserve current engine behavior exactly, including the MCTS approximation
// that adds the top card to hand instead of auto-playing it.
// ===========================================================================

static MAYHEM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_mayhem(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let mayhem = engine.state.player.status(sid::MAYHEM);
    for _ in 0..mayhem {
        if engine.state.hand.len() >= 10 {
            break;
        }
        if let Some(card_id) = engine.state.draw_pile.pop() {
            engine.state.hand.push(card_id);
        }
    }
}

pub static DEF_MAYHEM: EntityDef = EntityDef {
    id: "mayhem",
    name: "Mayhem",
    kind: EntityKind::Power,
    triggers: &MAYHEM_TRIGGERS,
    complex_hook: Some(hook_mayhem),
    status_guard: Some(sid::MAYHEM),
};

// ===========================================================================
// Tools of the Trade — TurnStartPostDraw: draw N, then choose one discard
// Preserve current engine behavior exactly: it draws `N` cards but still
// opens a single-card discard choice rather than `N` discards.
// ===========================================================================

static TOOLS_OF_THE_TRADE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &EMPTY_EFFECTS,
    counter: None,
}];

fn hook_tools_of_the_trade(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let tott = engine.state.player.status(sid::TOOLS_OF_THE_TRADE);
    if tott <= 0 {
        return;
    }

    engine.draw_cards(tott);
    if engine.state.hand.is_empty() {
        return;
    }

    let options: Vec<ChoiceOption> = (0..engine.state.hand.len())
        .map(ChoiceOption::HandCard)
        .collect();
    engine.begin_choice(ChoiceReason::DiscardFromHand, options, 1, 1);
}

pub static DEF_TOOLS_OF_THE_TRADE: EntityDef = EntityDef {
    id: "tools_of_the_trade",
    name: "Tools of the Trade",
    kind: EntityKind::Power,
    triggers: &TOOLS_OF_THE_TRADE_TRIGGERS,
    complex_hook: Some(hook_tools_of_the_trade),
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
            assert_eq!(def.triggers.len(), 1);
            assert_eq!(def.triggers[0].trigger, Trigger::TurnStartPostDraw);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/test_power_runtime_turn_start.rs"]
mod test_power_runtime_turn_start;
