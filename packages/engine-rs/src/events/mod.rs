//! Event definitions for each act.

use serde::{Deserialize, Serialize};

mod exordium;
mod city;
mod beyond;

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
}


/// Get event list for the given act.
pub fn events_for_act(act: i32) -> Vec<EventDef> {
    match act {
        2 => city::act2_events(),
        3 => beyond::act3_events(),
        _ => exordium::act1_events(),
    }
}
