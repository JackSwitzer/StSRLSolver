use super::{
    EventDef, EventEffect, EventProgram, EventProgramOp, EventReward, TypedEventDef,
    TypedEventOption,
};

fn supported(text: &str, ops: Vec<EventProgramOp>, effect: EventEffect) -> TypedEventOption {
    TypedEventOption::supported(text, EventProgram::from_ops(ops), effect)
}

fn event(name: &str, options: Vec<TypedEventOption>) -> TypedEventDef {
    TypedEventDef {
        name: name.to_string(),
        options,
    }
}

#[derive(Clone, Copy)]
enum DeadAdventurerReward {
    Gold,
    Nothing,
    Relic,
}

fn dead_adventurer_reward_ops(reward: DeadAdventurerReward) -> Vec<EventProgramOp> {
    match reward {
        DeadAdventurerReward::Gold => vec![EventProgramOp::gold(30)],
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
        .flat_map(|reward| dead_adventurer_reward_ops(*reward))
        .collect()
}

fn dead_adventurer_fight_program(rewards: &[DeadAdventurerReward], start: usize) -> EventProgram {
    let suffix_ops = dead_adventurer_suffix_ops(rewards, start);
    let outcomes = vec![
        vec![EventProgramOp::combat_branch(
            ["3 Sentries"],
            suffix_ops.clone(),
        )],
        vec![EventProgramOp::combat_branch(
            ["GremlinNob"],
            suffix_ops.clone(),
        )],
        vec![EventProgramOp::combat_branch(
            ["Lagavulin Event"],
            suffix_ops,
        )],
    ];
    EventProgram::from_ops(vec![EventProgramOp::random_outcome_table(outcomes)])
}

fn dead_adventurer_search_program(
    rewards: &[DeadAdventurerReward],
    start: usize,
    next_event: TypedEventDef,
    encounter_chance: usize,
) -> EventProgram {
    let fight_program = dead_adventurer_fight_program(rewards, start);
    let mut reward_ops = dead_adventurer_reward_ops(rewards[start]);
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
) -> TypedEventDef {
    event(
        "Dead Adventurer",
        vec![
            supported(
                "Search (risk elite fight, gain gold/relic)",
                dead_adventurer_search_program(&rewards, start, next_event, encounter_chance).ops,
                EventEffect::DamageAndGold(0, 30),
            ),
            supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
        ],
    )
}

pub(crate) fn dead_adventurer_event(ascension_level: i32) -> TypedEventDef {
    let encounter_chance = if ascension_level >= 15 { 35 } else { 25 };
    use DeadAdventurerReward::{Gold, Nothing, Relic};

    let permutations = [
        [Gold, Nothing, Relic],
        [Gold, Relic, Nothing],
        [Nothing, Gold, Relic],
        [Nothing, Relic, Gold],
        [Relic, Gold, Nothing],
        [Relic, Nothing, Gold],
    ];

    let mut outcomes = Vec::with_capacity(permutations.len());
    for rewards in permutations {
        let success = event(
            "Dead Adventurer",
            vec![supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing)],
        );
        let page3 = dead_adventurer_page(rewards, 2, success, 75);
        let page2 = dead_adventurer_page(rewards, 1, page3, 50);
        let page1 = dead_adventurer_page(rewards, 0, page2, encounter_chance);
        outcomes.push(page1.options[0].program.ops.clone());
    }

    event(
        "Dead Adventurer",
        vec![
            supported(
                "Search (risk elite fight, gain gold/relic)",
                vec![EventProgramOp::random_outcome_table(outcomes)],
                EventEffect::DamageAndGold(0, 30),
            ),
            supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
        ],
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
                supported(
                    "Reach inside (take 3 dmg, gain relic)",
                    vec![
                        EventProgramOp::damage_and_gold(-3, 0),
                        EventProgramOp::gain_relic("random relic"),
                    ],
                    EventEffect::DamageAndGold(-3, 0),
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
            dead_adventurer_event(25).options,
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
