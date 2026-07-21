use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Gambler's Brew opens GamblingChipAction's any-number hand selection.
fn gamblers_brew_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    if engine.state.hand.is_empty() {
        return;
    }

    let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.hand.len())
        .map(crate::engine::ChoiceOption::HandCard)
        .collect();
    let count = options.len();
    // GamblingChipAction opens an any-number selection capped at 99, then
    // manually discards only the selected cards and draws the same count.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/GamblersBrew.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GamblingChipAction.java
    engine.begin_choice(crate::engine::ChoiceReason::DiscardFromHand, options, 0, count);
    engine
        .state
        .player
        .set_status(sid::GAMBLING_CHIP_ACTIVE, 1);
}

pub static DEF: EntityDef = EntityDef {
    id: "GamblersBrew",
    name: "Gambler's Brew",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(gamblers_brew_hook),
    status_guard: None,
};
