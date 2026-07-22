//! Ginger: Set HAS_GINGER flag at combat start (prevents Weak).
//! Sources: decompiled/java-src/com/megacrit/cardcrawl/relics/Ginger.java and
//! actions/common/ApplyPowerAction.java.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    // ApplyPowerAction.java refuses Weakened on a player who owns Ginger; the
    // runtime flag lets the shared enemy-debuff pipeline enforce that check.
    Effect::Simple(SimpleEffect::SetStatus(
        Target::Player,
        sid::HAS_GINGER,
        AmountSource::Fixed(1),
    )),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatSetup,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Ginger",
    name: "Ginger",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
