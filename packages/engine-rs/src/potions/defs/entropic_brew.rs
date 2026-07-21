use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

// PotionHelper.getPotions(WATCHER, false) order, including Fruit Juice because
// returnRandomPotion(true) rejects it only after drawing it. Class-exclusive
// Ironclad, Silent, and Defect potions are absent.
// Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/PotionHelper.java
const WATCHER_POTION_POOL: &[(&str, u8)] = &[
    ("BottledMiracle", 0),
    ("StancePotion", 1),
    ("Ambrosia", 2),
    ("BlockPotion", 0),
    ("DexterityPotion", 0),
    ("EnergyPotion", 0),
    ("ExplosivePotion", 0),
    ("FirePotion", 0),
    ("StrengthPotion", 0),
    ("SwiftPotion", 0),
    ("WeakenPotion", 0),
    ("FearPotion", 0),
    ("AttackPotion", 0),
    ("SkillPotion", 0),
    ("PowerPotion", 0),
    ("ColorlessPotion", 0),
    ("SteroidPotion", 0),
    ("SpeedPotion", 0),
    ("BlessingOfTheForge", 0),
    ("RegenPotion", 1),
    ("AncientPotion", 1),
    ("LiquidBronze", 1),
    ("GamblersBrew", 1),
    ("EssenceOfSteel", 1),
    ("DuplicationPotion", 1),
    ("DistilledChaos", 1),
    ("LiquidMemories", 1),
    ("CultistPotion", 2),
    ("FruitJuice", 2),
    ("SneckoOil", 2),
    ("FairyPotion", 2),
    ("SmokeBomb", 2),
    ("EntropicBrew", 2),
];

#[cfg(test)]
pub(crate) fn is_watcher_limited_potion(id: &str) -> bool {
    id != "FruitJuice" && WATCHER_POTION_POOL.iter().any(|(candidate, _)| *candidate == id)
}

pub(crate) fn roll_limited_watcher_potion(engine: &mut CombatEngine) -> &'static str {
    // returnRandomPotion first rolls rarity (65/25/10), then the `limited`
    // overload deliberately discards its first pool draw and retries until it
    // finds that rarity and a non-Fruit-Juice potion.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
    let rarity_roll = engine.potion_rng.random_int(99);
    let wanted_rarity = if rarity_roll < 65 {
        0
    } else if rarity_roll < 90 {
        1
    } else {
        2
    };

    let _forced_discard = engine
        .potion_rng
        .random_int((WATCHER_POTION_POOL.len() - 1) as i32);
    loop {
        let idx = engine
            .potion_rng
            .random_int((WATCHER_POTION_POOL.len() - 1) as i32)
            as usize;
        let (id, rarity) = WATCHER_POTION_POOL[idx];
        if rarity == wanted_rarity && id != "FruitJuice" {
            return id;
        }
    }
}

/// Entropic Brew rolls once per potion slot up front, then queued obtain
/// actions fill the first empty slots after the brew itself is destroyed.
fn entropic_brew_hook(
    engine: &mut CombatEngine,
    owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let rolled: Vec<&'static str> = (0..engine.state.potions.len())
        .map(|_| roll_limited_watcher_potion(engine))
        .collect();

    if engine.state.has_relic("Sozu") {
        return;
    }

    let crate::effects::runtime::EffectOwner::PotionSlot { slot } = owner else {
        return;
    };
    let used_slot = slot as usize;
    if used_slot < engine.state.potions.len() {
        engine.state.potions[used_slot].clear();
    }
    for potion_id in rolled {
        let Some(empty_slot) = engine.state.potions.iter().position(|potion| potion.is_empty()) else {
            break;
        };
        engine.state.potions[empty_slot] = potion_id.to_string();
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "EntropicBrew",
    name: "Entropic Brew",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(entropic_brew_hook),
    status_guard: None,
};
