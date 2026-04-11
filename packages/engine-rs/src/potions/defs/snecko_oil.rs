use super::prelude::*;

/// Snecko Oil: draw cards + set Confusion.
/// The draw effect uses PotionPotency but Confusion is always 1.
/// complex_hook needed because draw must be deferred to engine.
static EFFECTS: [E; 2] = [
    E::Simple(SE::DrawCards(A::PotionPotency)),
    E::Simple(SE::SetStatus(T::Player, sid::CONFUSION, A::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "SneckoOil",
    name: "Snecko Oil",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
};
