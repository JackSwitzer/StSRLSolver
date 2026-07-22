//! Melange: Scry 3 whenever the draw pile is shuffled.
//!
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Melange.java
//! `onShuffle()` queues exactly one `ScryAction(3)`.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::Scry(AmountSource::Fixed(3)))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnShuffle,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Melange",
    name: "Melange",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
