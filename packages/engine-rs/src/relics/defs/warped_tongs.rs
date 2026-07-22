//! Warped Tongs: upgrade a random card in hand at start of each turn.
//! Sources: decompiled/java-src/com/megacrit/cardcrawl/relics/WarpedTongs.java
//! and decompiled/java-src/com/megacrit/cardcrawl/actions/common/UpgradeRandomCardAction.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, TurnStartQueuedAction};

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::UpgradeRandomCard);
        return;
    }
    let mut upgradeable = engine
        .state
        .hand
        .iter()
        .enumerate()
        .filter_map(|(index, card)| {
            let def = engine.card_registry.card_def_by_id(card.def_id);
            (def.card_type != crate::cards::CardType::Status
                && engine.card_registry.can_upgrade_card(card))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if upgradeable.is_empty() {
        return;
    }

    // CardGroup.shuffle seeds java.util.Random with one shuffleRng.randomLong.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    crate::seed::card_group_shuffle(&mut upgradeable, &mut engine.shuffle_rng);
    engine
        .card_registry
        .upgrade_card(&mut engine.state.hand[upgradeable[0]]);
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStartPostDraw,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "WarpedTongs",
    name: "Warped Tongs",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
