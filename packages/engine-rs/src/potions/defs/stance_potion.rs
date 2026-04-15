use super::prelude::*;
use crate::effects::declarative::{AmountSource as A, NamedOptionKind, ScaledNamedOption};

static STANCE_OPTIONS: [ScaledNamedOption; 2] = [
    ScaledNamedOption {
        label: "Wrath",
        amount: A::Fixed(0),
        kind: NamedOptionKind::SetStance(crate::state::Stance::Wrath),
    },
    ScaledNamedOption {
        label: "Calm",
        amount: A::Fixed(0),
        kind: NamedOptionKind::SetStance(crate::state::Stance::Calm),
    },
];

static EFFECTS: [E; 1] = [E::ChooseScaledNamedOptions(&STANCE_OPTIONS)];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

/// Stance Potion: Enter Wrath or Calm stance.
pub static DEF: EntityDef = EntityDef {
    id: "StancePotion",
    name: "Stance Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
