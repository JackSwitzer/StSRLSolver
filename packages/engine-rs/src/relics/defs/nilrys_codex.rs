//! Nilry's Codex: choose one of three random Watcher cards at end of turn.
//!
//! Sources: `reference/extracted/methods/relic/NilrysCodex.java` and
//! `decompiled/java-src/com/megacrit/cardcrawl/actions/unique/CodexAction.java`.

use crate::effects::declarative::GeneratedCardPool;
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{ChoiceOption, CombatEngine};

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::TurnEnd || engine.state.living_enemy_indices().is_empty() {
        return;
    }
    let options = crate::effects::interpreter::generate_unique_random_cards(
        engine,
        GeneratedCardPool::WatcherAny,
        3,
    )
    .into_iter()
    .map(ChoiceOption::GeneratedCard)
    .collect();
    engine.begin_codex_choice(options);
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEnd,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Nilry's Codex",
    name: "Nilry's Codex",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
