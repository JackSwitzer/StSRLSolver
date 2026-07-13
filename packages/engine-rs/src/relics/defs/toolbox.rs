//! Toolbox: choose one of three random colorless cards before the opening draw.
//!
//! Sources: `reference/extracted/methods/relic/Toolbox.java` and
//! `decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ChooseOneColorless.java`.

use crate::effects::declarative::{GeneratedCardPool, GeneratedCostRule};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{ChoiceOption, CombatEngine};

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::CombatStart {
        return;
    }
    let options = crate::effects::interpreter::generate_unique_random_cards(
        engine,
        GeneratedCardPool::Colorless,
        3,
    )
    .into_iter()
    .map(ChoiceOption::GeneratedCard)
    .collect();
    engine.begin_discovery_choice(options, 1, 1, 1, GeneratedCostRule::Base);
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStart,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Toolbox",
    name: "Toolbox",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
