//! Event definitions for each act.

use serde::{Deserialize, Serialize};

mod exordium;
mod city;
mod beyond;
mod shrines;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    pub name: String,
    pub options: Vec<EventOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOption {
    pub text: String,
    pub effect: EventEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventEffect {
    /// Gain/lose HP (negative = lose)
    Hp(i32),
    /// Gain/lose gold
    Gold(i32),
    /// Gain a random card
    GainCard,
    /// Remove a random curse/status from deck
    RemoveCard,
    /// Gain a random relic
    GainRelic,
    /// Gain max HP
    MaxHp(i32),
    /// Take damage and gain gold
    DamageAndGold(i32, i32),
    /// Nothing (leave)
    Nothing,
    /// Upgrade a random card
    UpgradeCard,
    /// Golden Idol: lose 25% max HP, gain 300 gold
    GoldenIdolTake,
    /// Transform a card into a random one of the same type
    TransformCard,
    /// Duplicate (copy) a card
    DuplicateCard,
    /// Gain a random potion
    GainPotion,
    /// Lose a percentage of max HP (value = percent, e.g. 10 = 10%)
    LosePercentHp(i32),
    /// Gain gold and a curse card
    GoldAndCurse(i32),
}


/// Get event list for the given act.
pub fn events_for_act(act: i32) -> Vec<EventDef> {
    match act {
        2 => city::act2_events(),
        3 => beyond::act3_events(),
        _ => exordium::act1_events(),
    }
}

/// Get shrine events (shared across all acts in Java).
pub fn shrine_events() -> Vec<EventDef> {
    shrines::shrine_events()
}
