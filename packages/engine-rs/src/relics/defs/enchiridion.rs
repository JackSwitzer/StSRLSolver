//! Enchiridion: add a random Watcher Power card to hand at combat start.
//!
//! Source: `reference/extracted/methods/relic/Enchiridion.java` —
//! `atPreBattle` selects one Power from the source color pools using
//! cardRandomRng, makes a temporary copy, and sets its turn cost to zero
//! unless its base cost is X.

use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, GeneratedCostRule};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::WatcherPower,
    count: AmountSource::Fixed(1),
    cost_rule: GeneratedCostRule::ZeroIfPositiveThisTurn,
}];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatSetup,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Enchiridion",
    name: "Enchiridion",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
