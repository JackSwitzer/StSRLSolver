//! Turn-end power definitions.
//!
//! Powers that trigger at the end of the player's turn.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, EndTurnQueuedAction};
use crate::state::Stance;
use crate::status_ids::sid;

// ===========================================================================
// Metallicize — TurnEnd: gain block equal to stacks
// ===========================================================================

static METALLICIZE_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEndPreCard,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_metallicize(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind == Trigger::TurnEndPreCard {
        let amount = engine.state.player.status(sid::METALLICIZE);
        engine.queue_end_turn_action_bottom(EndTurnQueuedAction::GainBlock(amount));
    }
}

pub static DEF_METALLICIZE: EntityDef = EntityDef {
    id: "metallicize",
    name: "Metallicize",
    kind: EntityKind::Power,
    triggers: &METALLICIZE_TRIGGERS,
    complex_hook: Some(hook_metallicize),
    status_guard: Some(sid::METALLICIZE),
};

// ===========================================================================
// Plated Armor — TurnEnd: gain block equal to stacks
// ===========================================================================

static PLATED_ARMOR_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEndPreCard,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_plated_armor(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind == Trigger::TurnEndPreCard {
        let amount = engine.state.player.status(sid::PLATED_ARMOR);
        engine.queue_end_turn_action_bottom(EndTurnQueuedAction::GainBlock(amount));
    }
}

pub static DEF_PLATED_ARMOR: EntityDef = EntityDef {
    id: "plated_armor",
    name: "Plated Armor",
    kind: EntityKind::Power,
    triggers: &PLATED_ARMOR_TRIGGERS,
    complex_hook: Some(hook_plated_armor),
    status_guard: Some(sid::PLATED_ARMOR),
};

// ===========================================================================
// Wraith Form — TurnEnd: apply its negative amount as Dexterity
// ===========================================================================

static WRAITH_FORM_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_wraith_form(
    engine: &mut CombatEngine,
    owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if owner != EffectOwner::PlayerPower || event.kind != Trigger::TurnEnd {
        return;
    }
    let amount = engine.state.player.status(sid::WRAITH_FORM);
    if amount > 0 {
        // WraithFormPower stores a negative amount and queues one negative
        // DexterityPower application at player turn end. Negative Dexterity is
        // a DEBUFF, so a later Artifact can block the entire stacked tick.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/WraithFormPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DexterityPower.java
        engine.queue_end_turn_action_bottom(EndTurnQueuedAction::ApplyDexterityLoss(amount));
    }
}

pub static DEF_WRAITH_FORM: EntityDef = EntityDef {
    id: "wraith_form",
    name: "Wraith Form",
    kind: EntityKind::Power,
    triggers: &WRAITH_FORM_TRIGGERS,
    complex_hook: Some(hook_wraith_form),
    status_guard: Some(sid::WRAITH_FORM),
};

// ===========================================================================
// Combust — TurnEnd: lose HP, then deal THORNS damage to all enemies
// ===========================================================================

static COMBUST_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_combust(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    if event.kind != Trigger::TurnEnd {
        return;
    }
    let targets = engine.state.living_enemy_indices();
    if targets.is_empty() {
        return;
    }

    // CombustPower.atEndOfTurn queues LoseHPAction before a source-less
    // DamageAllEnemiesAction with DamageType.THORNS. stackPower adds incoming
    // damage to amount but increments the private hpLoss field by one.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CombustPower.java
    engine.queue_end_turn_action_bottom(EndTurnQueuedAction::PlayerLoseHp(state.get(0).max(1)));
    let damage = engine.state.player.status(sid::COMBUST);
    engine.queue_end_turn_action_bottom(EndTurnQueuedAction::DamageAllEnemies(damage));
}

pub static DEF_COMBUST: EntityDef = EntityDef {
    id: "combust",
    name: "Combust",
    kind: EntityKind::Power,
    triggers: &COMBUST_TRIGGERS,
    complex_hook: Some(hook_combust),
    status_guard: Some(sid::COMBUST),
};

// ===========================================================================
// Omega — TurnEnd: deal source-less THORNS damage to all living enemies.
// ===========================================================================

static OMEGA_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_omega(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::TurnEnd {
        return;
    }

    // OmegaPower creates a pure damage matrix and resolves it as source-less
    // THORNS damage. It therefore uses block/Intangible/Buffer/Invincible, but
    // skips NORMAL-only Slow, Flight, Curl Up, Malleable, and offensive mods.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/OmegaPower.java
    let damage = engine.state.player.status(sid::OMEGA);
    engine.queue_end_turn_action_bottom(EndTurnQueuedAction::DamageAllEnemies(damage));
}

pub static DEF_OMEGA: EntityDef = EntityDef {
    id: "omega",
    name: "Omega",
    kind: EntityKind::Power,
    triggers: &OMEGA_TRIGGERS,
    complex_hook: Some(hook_omega),
    status_guard: Some(sid::OMEGA),
};

// ===========================================================================
// Like Water — TurnEnd: gain block if in Calm stance
// ===========================================================================

static LIKE_WATER_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEndPreCard,
    condition: TriggerCondition::InStance(Stance::Calm),
    effects: &[],
    counter: None,
}];

fn hook_like_water(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind == Trigger::TurnEndPreCard {
        let amount = engine.state.player.status(sid::LIKE_WATER);
        engine.queue_end_turn_action_bottom(EndTurnQueuedAction::GainBlock(amount));
    }
}

pub static DEF_LIKE_WATER: EntityDef = EntityDef {
    id: "like_water",
    name: "Like Water",
    kind: EntityKind::Power,
    triggers: &LIKE_WATER_TRIGGERS,
    complex_hook: Some(hook_like_water),
    status_guard: Some(sid::LIKE_WATER),
};

// ===========================================================================
// Study — TurnEnd: add Insight(s) to draw pile
// ===========================================================================

// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/StudyPower.java
// MakeTempCardInDrawPileAction(..., randomSpot=true) uses cardRandomRng for
// each Insight rather than shuffling the existing draw pile.
static STUDY_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_study(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind == Trigger::TurnEnd {
        let amount = engine.state.player.status(sid::STUDY);
        engine.queue_end_turn_action_bottom(EndTurnQueuedAction::AddInsightsToRandomDrawSpots(
            amount,
        ));
    }
}

pub static DEF_STUDY: EntityDef = EntityDef {
    id: "study",
    name: "Study",
    kind: EntityKind::Power,
    triggers: &STUDY_TRIGGERS,
    complex_hook: Some(hook_study),
    status_guard: Some(sid::STUDY),
};

// ===========================================================================
// No Draw — TurnEnd: remove the one-turn draw restriction
// ===========================================================================

// Source: powers/NoDrawPower.java::atEndOfTurn queues removal of power ID
// "No Draw" when the player's turn ends.
static NO_DRAW_TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook_no_draw(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind == Trigger::TurnEnd {
        engine.queue_end_turn_action_bottom(EndTurnQueuedAction::RemovePlayerPower(sid::NO_DRAW));
    }
}

pub static DEF_NO_DRAW: EntityDef = EntityDef {
    id: "no_draw",
    name: "No Draw",
    kind: EntityKind::Power,
    triggers: &NO_DRAW_TRIGGERS,
    complex_hook: Some(hook_no_draw),
    status_guard: Some(sid::NO_DRAW),
};

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metallicize_def() {
        assert_eq!(DEF_METALLICIZE.triggers.len(), 1);
        assert_eq!(DEF_METALLICIZE.triggers[0].trigger, Trigger::TurnEndPreCard);
    }

    #[test]
    fn test_combust_uses_ordered_complex_hook() {
        assert!(DEF_COMBUST.triggers[0].effects.is_empty());
        assert!(DEF_COMBUST.complex_hook.is_some());
    }

    #[test]
    fn test_like_water_requires_calm() {
        assert_eq!(
            DEF_LIKE_WATER.triggers[0].condition,
            TriggerCondition::InStance(Stance::Calm)
        );
    }

    #[test]
    fn test_all_turn_end_defs() {
        let defs = [
            &DEF_METALLICIZE,
            &DEF_PLATED_ARMOR,
            &DEF_WRAITH_FORM,
            &DEF_COMBUST,
            &DEF_OMEGA,
            &DEF_LIKE_WATER,
            &DEF_STUDY,
        ];
        for def in &defs {
            assert_eq!(def.kind, EntityKind::Power);
            let expected = if matches!(def.id, "metallicize" | "plated_armor" | "like_water") {
                Trigger::TurnEndPreCard
            } else {
                Trigger::TurnEnd
            };
            assert_eq!(def.triggers[0].trigger, expected);
        }
    }
}
