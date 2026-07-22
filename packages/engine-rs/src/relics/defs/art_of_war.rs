//! Art of War: if no Attacks were played last turn, gain 1 energy next turn.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/ArtOfWar.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, TurnStartQueuedAction};

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    match event.kind {
        Trigger::CombatSetup => state.set(0, 1),
        Trigger::TurnStart => {
            if state.get(0) > 0 && engine.state.turn > 1 {
                if engine.is_collecting_turn_start_actions() {
                    engine.queue_turn_start_action_bottom(TurnStartQueuedAction::GainEnergy(1));
                } else {
                    engine.state.energy += 1;
                }
            }
            state.set(0, 1);
        }
        Trigger::OnUseCard if event.card_type == Some(crate::cards::CardType::Attack) => {
            state.set(0, 0);
        }
        Trigger::CombatVictory => state.set(0, 0),
        _ => {}
    }
}

static TRIGGERS: [TriggeredEffect; 4] = [
    TriggeredEffect {
        trigger: Trigger::CombatSetup,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::OnUseCard,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::CombatVictory,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Art of War",
    name: "Art of War",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
