//! Pure Water: Add 1 Miracle to hand at combat start.
//!
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/PureWater.java
//!   ctor: STARTER tier relic (Watcher starting relic), id "PureWater".
//!   atBattleStartPreDraw(): MakeTempCardInHandAction(new Miracle(), 1, false)
//!     -> exactly one un-upgraded Miracle, added BEFORE the initial draw
//!        (engine fires Trigger::CombatStart before shuffling/drawing in
//!        CombatEngine::start_combat, matching the PreDraw hook).

use crate::effects::declarative::{AmountSource, Effect, Pile, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Miracle",
    Pile::Hand,
    AmountSource::Fixed(1),
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
    id: "PureWater",
    name: "Pure Water",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
