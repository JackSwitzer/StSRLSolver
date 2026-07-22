//! Tingsha: Deal 3 damage to a random enemy when a card is discarded.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Tingsha.java
//! Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageRandomEnemyAction.java

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let living = engine.state.living_enemy_indices();
    if !living.is_empty() {
        // MonsterGroup.getRandomMonster(..., cardRandomRng) calls random(0,
        // size-1), consuming one tick even when there is only one candidate.
        let pick = engine.card_random_rng.random_int((living.len() - 1) as i32) as usize;
        engine.deal_damage_to_enemy(living[pick], 3);
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnCardDiscard,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Tingsha",
    name: "Tingsha",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
