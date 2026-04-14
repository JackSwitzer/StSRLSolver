//! Gambling Chip: discard any number of cards and redraw that many at start of combat.
//! Uses a complex hook because it opens an interactive discard choice.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine};
use crate::status_ids::sid;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    if engine.state.hand.is_empty() {
        return;
    }

    let options: Vec<ChoiceOption> = (0..engine.state.hand.len())
        .map(ChoiceOption::HandCard)
        .collect();
    let count = options.len();
    engine.begin_choice(ChoiceReason::DiscardFromHand, options, 0, count);
    engine.state.player.set_status(sid::GAMBLING_CHIP_ACTIVE, 1);
}

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnStartPostDrawLate,
        condition: TriggerCondition::FirstTurn,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Gambling Chip",
    name: "Gambling Chip",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
