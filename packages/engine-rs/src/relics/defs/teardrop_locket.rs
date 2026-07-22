//! Teardrop Locket: Start combat in Calm stance.
//!
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/TeardropLocket.java
//! `atBattleStart()` queues `ChangeStanceAction("Calm")`, so normal stance-change
//! hooks fire before the opening draw.

use crate::effects::declarative::{Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::state::Stance;

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::ChangeStance(Stance::Calm))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStartTop,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "TeardropLocket",
    name: "Teardrop Locket",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
