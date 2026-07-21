use super::{
    EventEffect, EventProgram, EventProgramOp, EventReward, TypedEventDef, TypedEventOption,
};

fn supported(text: impl Into<String>, ops: Vec<EventProgramOp>, effect: EventEffect) -> TypedEventOption {
    TypedEventOption::supported(text, EventProgram::from_ops(ops), effect)
}

fn event(name: &str, options: Vec<TypedEventOption>) -> TypedEventDef {
    TypedEventDef {
        name: name.to_string(),
        options,
    }
}

fn mushrooms_fight_event() -> TypedEventDef {
    event(
        "Mushrooms",
        vec![supported(
            "Fight",
            vec![EventProgramOp::combat_branch_with_random_gold(
                ["FungiBeast", "FungiBeast", "FungiBeast"],
                20,
                30,
                vec![EventProgramOp::gain_unique_relic_or_circlet(
                    "Odd Mushroom",
                )],
            )],
            EventEffect::GainRelic,
        )],
    )
}

pub(super) fn golden_idol_consequence_event() -> TypedEventDef {
    event(
        "Golden Idol",
        vec![
            supported(
                "Escape with an Injury",
                vec![EventProgramOp::curse("Injury")],
                EventEffect::GainCard,
            ),
            supported(
                "Take damage",
                vec![EventProgramOp::adjust_hp_percent_by_ascension(
                    false, 25, 35,
                )],
                EventEffect::Hp(0),
            ),
            supported(
                "Lose max HP",
                vec![EventProgramOp::max_hp_percent_by_ascension(-8, -10, 1)],
                EventEffect::MaxHp(0),
            ),
        ],
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeadAdventurerReward {
    Gold,
    Nothing,
    Relic,
}

fn dead_adventurer_search_reward_ops(reward: DeadAdventurerReward) -> Vec<EventProgramOp> {
    match reward {
        DeadAdventurerReward::Gold => vec![EventProgramOp::gold(30)],
        DeadAdventurerReward::Nothing => vec![EventProgramOp::nothing()],
        DeadAdventurerReward::Relic => {
            vec![EventProgramOp::obtain_random_screenless_relic()]
        }
    }
}

fn dead_adventurer_fight_reward_ops(reward: DeadAdventurerReward) -> Vec<EventProgramOp> {
    match reward {
        DeadAdventurerReward::Gold => vec![EventProgramOp::gain_gold_reward(30)],
        DeadAdventurerReward::Nothing => vec![EventProgramOp::nothing()],
        DeadAdventurerReward::Relic => vec![EventProgramOp::gain_relic("random relic")],
    }
}

fn dead_adventurer_suffix_ops(
    rewards: &[DeadAdventurerReward],
    start: usize,
) -> Vec<EventProgramOp> {
    rewards[start..]
        .iter()
        .flat_map(|reward| dead_adventurer_fight_reward_ops(*reward))
        .collect()
}

fn dead_adventurer_fight_program(
    rewards: &[DeadAdventurerReward],
    start: usize,
    enemy: &str,
) -> EventProgram {
    let suffix_ops = dead_adventurer_suffix_ops(rewards, start);
    EventProgram::from_ops(vec![EventProgramOp::prepare_combat_branch_with_random_gold(
        [enemy],
        25,
        35,
        suffix_ops,
    )])
}

fn dead_adventurer_search_program(
    rewards: &[DeadAdventurerReward],
    start: usize,
    next_event: TypedEventDef,
    encounter_chance: usize,
    enemy: &str,
) -> EventProgram {
    let fight_program = dead_adventurer_fight_program(rewards, start, enemy);
    let mut reward_ops = dead_adventurer_search_reward_ops(rewards[start]);
    reward_ops.push(EventProgramOp::continue_event(next_event));

    let mut outcomes = Vec::with_capacity(100);
    for _ in 0..encounter_chance {
        outcomes.push(fight_program.ops.clone());
    }
    for _ in encounter_chance..100 {
        outcomes.push(reward_ops.clone());
    }

    EventProgram::from_ops(vec![EventProgramOp::random_outcome_table(outcomes)])
}

fn dead_adventurer_page(
    rewards: [DeadAdventurerReward; 3],
    start: usize,
    next_event: TypedEventDef,
    encounter_chance: usize,
    enemy: &str,
) -> TypedEventDef {
    event(
        "Dead Adventurer",
        vec![
            supported(
                format!(
                    "Search ({}% elite fight; otherwise gain gold/relic)",
                    encounter_chance
                ),
                dead_adventurer_search_program(
                    &rewards,
                    start,
                    next_event,
                    encounter_chance,
                    enemy,
                )
                .ops,
                EventEffect::DamageAndGold(0, 30),
            ),
            supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
        ],
    )
}

pub(crate) fn dead_adventurer_event_with_state(
    ascension_level: i32,
    rewards: [DeadAdventurerReward; 3],
    enemy: &str,
) -> TypedEventDef {
    let encounter_chance = if ascension_level >= 15 { 35 } else { 25 };
    let success = event(
        "Dead Adventurer",
        vec![supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing)],
    );
    let page3 = dead_adventurer_page(rewards, 2, success, 75, enemy);
    let page2 = dead_adventurer_page(rewards, 1, page3, 50, enemy);
    let page1 = dead_adventurer_page(rewards, 0, page2, encounter_chance, enemy);

    event(
        "Dead Adventurer",
        page1.options,
    )
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
                    "Take the Golden Idol",
                    vec![
                        // GoldenIdolEvent.java obtains the relic immediately,
                        // then opens a second three-option consequence screen.
                        // Java: decompiled/java-src/com/megacrit/cardcrawl/events/exordium/GoldenIdolEvent.java
                        EventProgramOp::obtain_relic("Golden Idol"),
                        EventProgramOp::continue_event(golden_idol_consequence_event()),
                    ],
                    EventEffect::GoldenIdolTake,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Scrap Ooze",
            vec![
                supported(
                    "Reach inside (take 3 dmg, 25% relic chance)",
                    vec![EventProgramOp::nothing()],
                    EventEffect::DamageAndGold(-3, 0),
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Shining Light",
            vec![
                supported(
                    "Enter (upgrade 2 random cards, take 20% max HP damage)",
                    vec![
                        EventProgramOp::lose_hp_percent_rounded_by_ascension(20, 30),
                        EventProgramOp::upgrade_random_cards(2),
                    ],
                    EventEffect::UpgradeCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Living Wall",
            vec![
                supported(
                    "Upgrade (upgrade a card)",
                    vec![EventProgramOp::deck_selection(
                        "deck_selection_event_upgrade",
                    )],
                    EventEffect::UpgradeCard,
                ),
                supported(
                    "Remove (remove a card)",
                    vec![EventProgramOp::deck_selection(
                        "deck_selection_event_remove",
                    )],
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
                    vec![
                        EventProgramOp::gold(-50),
                        EventProgramOp::deck_selection("deck_selection_event_remove"),
                    ],
                    EventEffect::RemoveCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Dead Adventurer",
            vec![supported(
                "Initialize Dead Adventurer",
                vec![EventProgramOp::nothing()],
                EventEffect::Nothing,
            )],
        ),
        event(
            "Golden Wing",
            vec![
                supported(
                    "Feed (take 7 dmg, remove a card)",
                    vec![
                        EventProgramOp::damage_and_gold(-7, 0),
                        EventProgramOp::deck_selection("deck_selection_event_remove"),
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
                    vec![EventProgramOp::continue_event(mushrooms_fight_event())],
                    EventEffect::Nothing,
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
