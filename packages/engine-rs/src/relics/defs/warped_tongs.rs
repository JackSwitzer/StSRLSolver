//! Warped Tongs: upgrade a random card in hand at start of each turn.
//! Sources: decompiled/java-src/com/megacrit/cardcrawl/relics/WarpedTongs.java
//! and decompiled/java-src/com/megacrit/cardcrawl/actions/common/UpgradeRandomCardAction.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

fn java_next_int(seed: &mut u64, bound: usize) -> usize {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1_u64 << 48) - 1;
    let mut next = |bits: u32| {
        *seed = seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
        (*seed >> (48 - bits)) as u32
    };
    if bound.is_power_of_two() {
        return ((bound as u64 * next(31) as u64) >> 31) as usize;
    }
    loop {
        let bits = next(31) as usize;
        let value = bits % bound;
        if bits - value + (bound - 1) < (1_usize << 31) {
            return value;
        }
    }
}

fn java_shuffle<T>(values: &mut [T], random_seed: u64) {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const MASK: u64 = (1_u64 << 48) - 1;
    let mut seed = (random_seed ^ MULTIPLIER) & MASK;
    for len in (2..=values.len()).rev() {
        let other = java_next_int(&mut seed, len);
        values.swap(len - 1, other);
    }
}

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let mut upgradeable = engine
        .state
        .hand
        .iter()
        .enumerate()
        .filter_map(|(index, card)| {
            let def = engine.card_registry.card_def_by_id(card.def_id);
            (!card.is_upgraded()
                && def.card_type != crate::cards::CardType::Status
                && engine.card_registry.get(&format!("{}+", def.id)).is_some())
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if upgradeable.is_empty() {
        return;
    }

    // CardGroup.shuffle seeds java.util.Random with one shuffleRng.randomLong.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    let shuffle_seed = engine.rng.random_long();
    java_shuffle(&mut upgradeable, shuffle_seed);
    engine
        .card_registry
        .upgrade_card(&mut engine.state.hand[upgradeable[0]]);
}

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnStartPostDrawLate,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "WarpedTongs",
    name: "Warped Tongs",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
