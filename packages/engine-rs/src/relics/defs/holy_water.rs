//! Holy Water: Add 3 Miracles to hand before the opening draw.
//!
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/HolyWater.java
//! `atBattleStartPreDraw()` queues `MakeTempCardInHandAction(new Miracle(), 3, false)`.

use crate::effects::declarative::{AmountSource, Effect, Pile, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Miracle",
    Pile::Hand,
    AmountSource::Fixed(3),
))];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "HolyWater",
    name: "Holy Water",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
