use super::{
    EventDef, EventEffect, EventProgram, EventProgramOp, EventReward, TypedEventDef,
    TypedEventOption,
};

fn supported(text: &str, ops: Vec<EventProgramOp>, effect: EventEffect) -> TypedEventOption {
    TypedEventOption::supported(text, EventProgram::from_ops(ops), effect)
}

fn blocked(
    text: &str,
    ops: Vec<EventProgramOp>,
    effect: EventEffect,
    reason: &str,
) -> TypedEventOption {
    TypedEventOption::blocked(text, EventProgram::from_ops(ops), effect, reason)
}

fn event(name: &str, options: Vec<TypedEventOption>) -> TypedEventDef {
    TypedEventDef {
        name: name.to_string(),
        options,
    }
}

pub fn typed_act1_events() -> Vec<TypedEventDef> {
    vec![
        event(
            "Big Fish",
            vec![
                supported("Eat (heal 5 HP)", vec![EventProgramOp::hp(5)], EventEffect::Hp(5)),
                supported(
                    "Banana (gain 2 max HP)",
                    vec![EventProgramOp::max_hp(2)],
                    EventEffect::MaxHp(2),
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Golden Idol",
            vec![
                supported(
                    "Take (gain 300 gold, lose 25% max HP)",
                    vec![
                        EventProgramOp::lose_percent_hp(25),
                        EventProgramOp::gain_gold(300),
                    ],
                    EventEffect::GoldenIdolTake,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Scrap Ooze",
            vec![
                blocked(
                    "Reach inside (take 3 dmg, gain relic)",
                    vec![
                        EventProgramOp::damage_and_gold(-3, 0),
                        EventProgramOp::gain_relic("random relic"),
                    ],
                    EventEffect::DamageAndGold(-3, 0),
                    "requires a fight branch plus a relic reward primitive",
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Shining Light",
            vec![
                supported(
                    "Enter (upgrade 2 cards, take 10 dmg)",
                    vec![
                        EventProgramOp::damage_and_gold(-10, 0),
                        EventProgramOp::upgrade_card(2),
                    ],
                    EventEffect::DamageAndGold(-10, 0),
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Living Wall",
            vec![
                supported(
                    "Upgrade (upgrade a card)",
                    vec![EventProgramOp::upgrade_card(1)],
                    EventEffect::UpgradeCard,
                ),
                supported(
                    "Remove (remove a card)",
                    vec![EventProgramOp::remove_card(1)],
                    EventEffect::RemoveCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "The Cleric",
            vec![
                supported(
                    "Heal (pay 35 gold, heal 25% max HP)",
                    vec![
                        EventProgramOp::gold(-35),
                        EventProgramOp::heal_percent_hp(25),
                    ],
                    EventEffect::Gold(-35),
                ),
                supported(
                    "Purify (pay 50 gold, remove a card)",
                    vec![EventProgramOp::gold(-50), EventProgramOp::remove_card(1)],
                    EventEffect::RemoveCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Dead Adventurer",
            vec![
                blocked(
                    "Search (risk elite fight, gain gold/relic)",
                    vec![
                        EventProgramOp::damage_and_gold(0, 30),
                        EventProgramOp::gain_relic("random relic"),
                    ],
                    EventEffect::DamageAndGold(0, 30),
                    "requires persistent search-state tracking plus a room-reward queue for the one-shot gold/relic pulls and elite-combat continuation",
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Golden Wing",
            vec![
                supported(
                    "Feed (take 7 dmg, remove a card)",
                    vec![
                        EventProgramOp::damage_and_gold(-7, 0),
                        EventProgramOp::remove_card(1),
                    ],
                    EventEffect::RemoveCard,
                ),
                supported(
                    "Attack (gain 50-80 gold if strong card)",
                    vec![EventProgramOp::random_outcome_table(
                        (50..=80)
                            .map(|amount| vec![EventProgramOp::gold(amount)])
                            .collect(),
                    )],
                    EventEffect::Gold(65),
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "World of Goop",
            vec![
                supported(
                    "Gather gold (gain 75 gold, take 11 dmg)",
                    vec![
                        EventProgramOp::damage_and_gold(-11, 75),
                        EventProgramOp::gain_gold(75),
                    ],
                    EventEffect::DamageAndGold(-11, 75),
                ),
                supported(
                    "Leave (lose some gold)",
                    vec![EventProgramOp::gold(-35)],
                    EventEffect::Gold(-35),
                ),
            ],
        ),
        event(
            "Mushrooms",
            vec![
                supported(
                    "Stomp (fight, gain Odd Mushroom relic)",
                    vec![
                        EventProgramOp::combat_branch(
                            ["FungiBeast", "FungiBeast", "FungiBeast"],
                            vec![EventProgramOp::gain_relic("Odd Mushroom")],
                        ),
                    ],
                    EventEffect::GainRelic,
                ),
                supported(
                    "Eat (heal 25% max HP, gain Parasite curse)",
                    vec![
                        EventProgramOp::heal_percent_hp(25),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Parasite".to_string(),
                        }),
                    ],
                    EventEffect::Hp(0),
                ),
            ],
        ),
        event(
            "Liars Game",
            vec![
                supported(
                    "Agree (gain 175 gold, gain Doubt curse)",
                    vec![
                        EventProgramOp::gold(175),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Doubt".to_string(),
                        }),
                    ],
                    EventEffect::Gold(175),
                ),
                supported("Disagree", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
    ]
}

#[allow(dead_code)]
pub fn act1_events() -> Vec<EventDef> {
    typed_act1_events()
        .into_iter()
        .map(|event| event.legacy())
        .collect()
}
