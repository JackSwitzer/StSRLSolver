//! Event definitions for each act.

use serde::{Deserialize, Serialize};

mod exordium;
mod city;
mod beyond;
mod shrines;

pub(crate) use exordium::dead_adventurer_event;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventRuntimeStatus {
    Supported,
    Blocked { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventDeckMutation {
    GainCard { count: usize },
    RemoveCard { count: usize },
    TransformCard { count: usize },
    DuplicateCard { count: usize },
    UpgradeCard { count: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventReward {
    Gold { amount: i32 },
    MaxHp { amount: i32 },
    Relic { label: String },
    Potion { count: usize },
    Card { count: usize },
    StoredNoteCard,
    SpecificCards { labels: Vec<String> },
    Curse { label: String },
    Nothing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventProgramOp {
    ContinueEvent {
        event: Box<TypedEventDef>,
    },
    ResolveFinalAct,
    CombatBranch {
        enemies: Vec<String>,
        on_win: Box<EventProgram>,
    },
    StartBossCombat,
    RandomOutcomeTable {
        outcomes: Vec<EventProgram>,
    },
    DeckSelection {
        label: String,
    },
    AdjustGoldByAct {
        exordium: i32,
        city: i32,
        beyond: i32,
    },
    AdjustHp { amount: i32 },
    AdjustHpByAscension {
        base: i32,
        asc15: i32,
    },
    HealPercentHp { percent: i32 },
    AdjustHpPercentByAscension {
        heal: bool,
        base_percent: i32,
        asc15_percent: i32,
    },
    HealToFull,
    AdjustGold { amount: i32 },
    AdjustMaxHp { amount: i32 },
    AdjustMaxHpPercent { percent: i32 },
    DamageAndGold { damage: i32, gold: i32 },
    LosePercentHp { percent: i32 },
    ResolveJoustBet { bet_on_owner: bool },
    RemoveRelic { label: String },
    DeckMutation(EventDeckMutation),
    Reward(EventReward),
    Nothing,
    BlockedPlaceholder { reason: String },
}

impl EventProgramOp {
    pub fn continue_event(event: TypedEventDef) -> Self {
        Self::ContinueEvent {
            event: Box::new(event),
        }
    }

    pub fn resolve_final_act() -> Self {
        Self::ResolveFinalAct
    }

    pub fn combat_branch(
        enemies: impl IntoIterator<Item = impl Into<String>>,
        on_win: Vec<EventProgramOp>,
    ) -> Self {
        Self::CombatBranch {
            enemies: enemies.into_iter().map(Into::into).collect(),
            on_win: Box::new(EventProgram::from_ops(on_win)),
        }
    }

    pub fn start_boss_combat() -> Self {
        Self::StartBossCombat
    }

    pub fn random_outcome_table(outcomes: Vec<Vec<EventProgramOp>>) -> Self {
        Self::RandomOutcomeTable {
            outcomes: outcomes
                .into_iter()
                .map(EventProgram::from_ops)
                .collect(),
        }
    }

    pub fn deck_selection(label: impl Into<String>) -> Self {
        Self::DeckSelection {
            label: label.into(),
        }
    }

    pub fn gold_by_act(exordium: i32, city: i32, beyond: i32) -> Self {
        Self::AdjustGoldByAct {
            exordium,
            city,
            beyond,
        }
    }

    pub fn hp(amount: i32) -> Self {
        Self::AdjustHp { amount }
    }

    pub fn hp_by_ascension(base: i32, asc15: i32) -> Self {
        Self::AdjustHpByAscension { base, asc15 }
    }

    pub fn gold(amount: i32) -> Self {
        Self::AdjustGold { amount }
    }

    pub fn heal_percent_hp(percent: i32) -> Self {
        Self::HealPercentHp { percent }
    }

    pub fn adjust_hp_percent_by_ascension(
        heal: bool,
        base_percent: i32,
        asc15_percent: i32,
    ) -> Self {
        Self::AdjustHpPercentByAscension {
            heal,
            base_percent,
            asc15_percent,
        }
    }

    pub fn heal_to_full() -> Self {
        Self::HealToFull
    }

    pub fn max_hp(amount: i32) -> Self {
        Self::AdjustMaxHp { amount }
    }

    pub fn max_hp_percent(percent: i32) -> Self {
        Self::AdjustMaxHpPercent { percent }
    }

    pub fn damage_and_gold(damage: i32, gold: i32) -> Self {
        Self::DamageAndGold { damage, gold }
    }

    pub fn lose_percent_hp(percent: i32) -> Self {
        Self::LosePercentHp { percent }
    }

    pub fn joust_bet(bet_on_owner: bool) -> Self {
        Self::ResolveJoustBet { bet_on_owner }
    }

    pub fn remove_relic(label: impl Into<String>) -> Self {
        Self::RemoveRelic {
            label: label.into(),
        }
    }

    pub fn gain_card(count: usize) -> Self {
        Self::DeckMutation(EventDeckMutation::GainCard { count })
    }

    pub fn remove_card(count: usize) -> Self {
        Self::DeckMutation(EventDeckMutation::RemoveCard { count })
    }

    pub fn transform_card(count: usize) -> Self {
        Self::DeckMutation(EventDeckMutation::TransformCard { count })
    }

    pub fn duplicate_card(count: usize) -> Self {
        Self::DeckMutation(EventDeckMutation::DuplicateCard { count })
    }

    pub fn upgrade_card(count: usize) -> Self {
        Self::DeckMutation(EventDeckMutation::UpgradeCard { count })
    }

    pub fn gain_relic(label: impl Into<String>) -> Self {
        Self::Reward(EventReward::Relic { label: label.into() })
    }

    pub fn gain_potion(count: usize) -> Self {
        Self::Reward(EventReward::Potion { count })
    }

    pub fn gain_gold(amount: i32) -> Self {
        Self::Reward(EventReward::Gold { amount })
    }

    pub fn gain_max_hp(amount: i32) -> Self {
        Self::Reward(EventReward::MaxHp { amount })
    }

    pub fn gain_card_reward(count: usize) -> Self {
        Self::Reward(EventReward::Card { count })
    }

    pub fn gain_specific_cards(labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Reward(EventReward::SpecificCards {
            labels: labels.into_iter().map(Into::into).collect(),
        })
    }

    pub fn curse(label: impl Into<String>) -> Self {
        Self::Reward(EventReward::Curse { label: label.into() })
    }

    pub fn nothing() -> Self {
        Self::Nothing
    }

    pub fn blocked(reason: impl Into<String>) -> Self {
        Self::BlockedPlaceholder {
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventProgram {
    pub ops: Vec<EventProgramOp>,
}

impl EventProgram {
    pub fn from_ops(ops: Vec<EventProgramOp>) -> Self {
        Self { ops }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedEventOption {
    pub text: String,
    pub program: EventProgram,
    pub legacy_effect: EventEffect,
    pub status: EventRuntimeStatus,
}

impl TypedEventOption {
    pub fn supported(
        text: impl Into<String>,
        program: EventProgram,
        legacy_effect: EventEffect,
    ) -> Self {
        Self {
            text: text.into(),
            program,
            legacy_effect,
            status: EventRuntimeStatus::Supported,
        }
    }

    pub fn blocked(
        text: impl Into<String>,
        program: EventProgram,
        legacy_effect: EventEffect,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            text: text.into(),
            program,
            legacy_effect,
            status: EventRuntimeStatus::Blocked {
                reason: reason.into(),
            },
        }
    }

    pub fn legacy(&self) -> EventOption {
        EventOption {
            text: self.text.clone(),
            effect: self.legacy_effect.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedEventDef {
    pub name: String,
    pub options: Vec<TypedEventOption>,
}

impl TypedEventDef {
    pub fn legacy(&self) -> EventDef {
        EventDef {
            name: self.name.clone(),
            options: self.options.iter().map(TypedEventOption::legacy).collect(),
        }
    }

    pub fn from_legacy(event: EventDef) -> Self {
        Self {
            name: event.name,
            options: event
                .options
                .into_iter()
                .map(TypedEventOption::from_legacy)
                .collect(),
        }
    }
}

impl EventEffect {
    pub fn default_program(&self) -> EventProgram {
        let ops = match self {
            Self::Hp(amount) => vec![EventProgramOp::AdjustHp { amount: *amount }],
            Self::Gold(amount) => vec![EventProgramOp::AdjustGold { amount: *amount }],
            Self::GainCard => vec![EventProgramOp::gain_card(1)],
            Self::RemoveCard => vec![EventProgramOp::remove_card(1)],
            Self::GainRelic => vec![EventProgramOp::gain_relic("random relic")],
            Self::MaxHp(amount) => vec![EventProgramOp::AdjustMaxHp { amount: *amount }],
            Self::DamageAndGold(damage, gold) => vec![EventProgramOp::DamageAndGold {
                damage: *damage,
                gold: *gold,
            }],
            Self::Nothing => vec![EventProgramOp::Nothing],
            Self::UpgradeCard => vec![EventProgramOp::upgrade_card(1)],
            Self::GoldenIdolTake => vec![
                EventProgramOp::LosePercentHp { percent: 25 },
                EventProgramOp::AdjustGold { amount: 300 },
            ],
            Self::TransformCard => vec![EventProgramOp::transform_card(1)],
            Self::DuplicateCard => vec![EventProgramOp::duplicate_card(1)],
            Self::GainPotion => vec![EventProgramOp::gain_potion(1)],
            Self::LosePercentHp(percent) => vec![EventProgramOp::LosePercentHp {
                percent: *percent,
            }],
            Self::GoldAndCurse(amount) => vec![
                EventProgramOp::AdjustGold { amount: *amount },
                EventProgramOp::Reward(EventReward::Curse {
                    label: "Curse".to_string(),
                }),
            ],
        };
        EventProgram::from_ops(ops)
    }
}

impl TypedEventOption {
    pub fn from_legacy(option: EventOption) -> Self {
        let program = option.effect.default_program();
        Self::supported(option.text, program, option.effect)
    }
}

impl From<EventDef> for TypedEventDef {
    fn from(value: EventDef) -> Self {
        Self::from_legacy(value)
    }
}

pub fn typed_events_for_act(act: i32) -> Vec<TypedEventDef> {
    match act {
        2 => city::typed_act2_events(),
        3 => beyond::typed_act3_events(),
        _ => exordium::typed_act1_events(),
    }
}

pub fn typed_shrine_events() -> Vec<TypedEventDef> {
    shrines::typed_shrine_events()
}


/// Get event list for the given act.
pub fn events_for_act(act: i32) -> Vec<EventDef> {
    typed_events_for_act(act)
        .into_iter()
        .map(|event| event.legacy())
        .collect()
}

/// Get shrine events (shared across all acts in Java).
pub fn shrine_events() -> Vec<EventDef> {
    typed_shrine_events()
        .into_iter()
        .map(|event| event.legacy())
        .collect()
}

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave4.rs"]
mod test_event_runtime_wave4;

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave5.rs"]
mod test_event_runtime_wave5;

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave6.rs"]
mod test_event_runtime_wave6;

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave9.rs"]
mod test_event_runtime_wave9;

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave10.rs"]
mod test_event_runtime_wave10;
