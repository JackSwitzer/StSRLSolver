//! Duality (Yang): On Attack play, gain 1 temporary Dexterity.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Duality.java
//!
//! The Dex is temporary, so the runtime adds LOSE_DEXTERITY alongside it.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::OnAttackPlayed {
        return;
    }
    // DamageAction.clearPostCombatActions removes Duality's queued
    // ApplyPowerActions when the triggering attack kills the encounter. The
    // surviving UseCardAction is a different callback class; do not turn the
    // discarded power actions into immediate post-lethal Dexterity.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Duality.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    if engine.state.is_victory() {
        return;
    }
    engine.state.player.add_status(sid::DEXTERITY, 1);
    engine.state.player.add_status(sid::LOSE_DEXTERITY, 1);
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnAttackPlayed,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Yang",
    name: "Duality",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
