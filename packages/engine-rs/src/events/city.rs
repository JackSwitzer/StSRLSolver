use super::{
    EventDef, EventEffect, EventProgram, EventProgramOp, EventReward, TypedEventDef,
    TypedEventOption,
};

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave11.rs"]
mod test_event_runtime_wave11;

fn supported(text: &str, ops: Vec<EventProgramOp>, effect: EventEffect) -> TypedEventOption {
    TypedEventOption::supported(text, EventProgram::from_ops(ops), effect)
}

fn event(name: &str, options: Vec<TypedEventOption>) -> TypedEventDef {
    TypedEventDef {
        name: name.to_string(),
        options,
    }
}

fn colosseum_fight_prompt() -> TypedEventDef {
    event(
        "Colosseum",
        vec![supported(
            "Fight (enter the arena)",
            vec![EventProgramOp::combat_branch(
                ["TaskMaster", "SlaverBlue", "SlaverRed"],
                vec![EventProgramOp::continue_event(colosseum_post_combat())],
            )],
            EventEffect::Nothing,
        )],
    )
}

fn colosseum_post_combat() -> TypedEventDef {
    event(
        "Colosseum",
        vec![
            supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            supported(
                "Continue (fight Nobs)",
                vec![EventProgramOp::combat_branch(
                    ["GremlinNob", "GremlinNob"],
                    vec![
                        EventProgramOp::gain_relic("rare relic"),
                        EventProgramOp::gain_relic("uncommon relic"),
                        EventProgramOp::gain_gold(100),
                    ],
                )],
                EventEffect::GainRelic,
            ),
        ],
    )
}

fn cursed_tome_page_1() -> TypedEventDef {
    event(
        "Cursed Tome",
        vec![supported(
            "Continue",
            vec![
                EventProgramOp::hp(-1),
                EventProgramOp::continue_event(cursed_tome_page_2()),
            ],
            EventEffect::Hp(-1),
        )],
    )
}

fn cursed_tome_page_2() -> TypedEventDef {
    event(
        "Cursed Tome",
        vec![supported(
            "Continue",
            vec![
                EventProgramOp::hp(-2),
                EventProgramOp::continue_event(cursed_tome_page_3()),
            ],
            EventEffect::Hp(-2),
        )],
    )
}

fn cursed_tome_page_3() -> TypedEventDef {
    event(
        "Cursed Tome",
        vec![supported(
            "Continue",
            vec![
                EventProgramOp::hp(-3),
                EventProgramOp::continue_event(cursed_tome_last_page()),
            ],
            EventEffect::Hp(-3),
        )],
    )
}

fn cursed_tome_last_page() -> TypedEventDef {
    event(
        "Cursed Tome",
        vec![
            supported(
                "Take the book",
                vec![
                    EventProgramOp::hp_by_ascension(-10, -15),
                    EventProgramOp::gain_relic("Cursed Tome reward"),
                ],
                EventEffect::GainRelic,
            ),
            supported(
                "Stop reading",
                vec![EventProgramOp::hp(-3)],
                EventEffect::Hp(-3),
            ),
        ],
    )
}

pub fn typed_act2_events() -> Vec<TypedEventDef> {
    vec![
        event(
            "Forgotten Altar",
            vec![
                supported(
                    "Offer (lose 5 HP, gain golden idol)",
                    vec![
                        EventProgramOp::hp(-5),
                        EventProgramOp::gain_relic("Golden Idol"),
                    ],
                    EventEffect::Hp(-5),
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Council of Ghosts",
            vec![
                supported(
                    "Accept (gain 5 Apparitions, lose max HP)",
                    vec![
                        EventProgramOp::max_hp(-5),
                        EventProgramOp::gain_card_reward(5),
                    ],
                    EventEffect::MaxHp(-5),
                ),
                supported("Refuse", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Masked Bandits",
            vec![
                supported(
                    "Pay (lose all gold)",
                    vec![EventProgramOp::gold(-999)],
                    EventEffect::Gold(-999),
                ),
                supported("Fight", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Knowing Skull",
            vec![
                supported(
                    "Ask for gold (gain 90 gold, lose 10% HP)",
                    vec![
                        EventProgramOp::lose_percent_hp(10),
                        EventProgramOp::gold(90),
                    ],
                    EventEffect::DamageAndGold(-6, 90),
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Vampires",
            vec![
                supported(
                    "Accept (remove Strikes, gain Bites)",
                    vec![
                        EventProgramOp::remove_card(1),
                        EventProgramOp::gain_card(5),
                    ],
                    EventEffect::RemoveCard,
                ),
                supported("Refuse", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Addict",
            vec![
                supported(
                    "Pay (lose 85 gold, gain relic)",
                    vec![
                        EventProgramOp::gold(-85),
                        EventProgramOp::gain_relic("random relic"),
                    ],
                    EventEffect::GainRelic,
                ),
                supported(
                    "Rob (gain relic, gain Shame curse)",
                    vec![
                        EventProgramOp::gain_relic("random relic"),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Shame".to_string(),
                        }),
                    ],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Back to Basics",
            vec![
                supported(
                    "Elegance (remove a card)",
                    vec![EventProgramOp::remove_card(1)],
                    EventEffect::RemoveCard,
                ),
                supported(
                    "Simplicity (upgrade all Strikes/Defends)",
                    vec![EventProgramOp::upgrade_card(999)],
                    EventEffect::UpgradeCard,
                ),
            ],
        ),
        event(
            "Beggar",
            vec![
                supported(
                    "Donate (pay 75 gold, remove a card)",
                    vec![
                        EventProgramOp::gold(-75),
                        EventProgramOp::remove_card(1),
                    ],
                    EventEffect::RemoveCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Colosseum",
            vec![
                supported(
                    "Enter",
                    vec![EventProgramOp::continue_event(colosseum_fight_prompt())],
                    EventEffect::Nothing,
                ),
            ],
        ),
        event(
            "Cursed Tome",
            vec![
                supported(
                    "Read",
                    vec![
                        EventProgramOp::continue_event(cursed_tome_page_1()),
                    ],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Drug Dealer",
            vec![
                supported(
                    "Obtain J.A.X. (gain J.A.X. card)",
                    vec![EventProgramOp::gain_specific_cards(["J.A.X."])],
                    EventEffect::GainCard,
                ),
                supported(
                    "Become test subject (transform 2 cards)",
                    vec![EventProgramOp::transform_card(2)],
                    EventEffect::TransformCard,
                ),
                supported(
                    "Inject mutagens (gain Mutagenic Strength relic)",
                    vec![EventProgramOp::gain_relic("Mutagenic Strength")],
                    EventEffect::GainRelic,
                ),
            ],
        ),
        event(
            "Nest",
            vec![
                supported(
                    "Steal gold (gain 99 gold)",
                    vec![EventProgramOp::gold(99)],
                    EventEffect::Gold(99),
                ),
                supported(
                    "Join (take 6 dmg, gain Ritual Dagger)",
                    vec![
                        EventProgramOp::damage_and_gold(-6, 0),
                        EventProgramOp::gain_specific_cards(["RitualDagger"]),
                    ],
                    EventEffect::DamageAndGold(-6, 0),
                ),
            ],
        ),
        event(
            "The Joust",
            vec![
                supported(
                    "Bet on Murderer (pay 50 gold, win 100)",
                    vec![EventProgramOp::joust_bet(false)],
                    EventEffect::Gold(-50),
                ),
                supported(
                    "Bet on Owner (pay 50 gold, win 250)",
                    vec![EventProgramOp::joust_bet(true)],
                    EventEffect::Gold(-50),
                ),
            ],
        ),
        event(
            "The Library",
            vec![
                supported(
                    "Read (choose 1 of 20 cards)",
                    vec![EventProgramOp::gain_card_reward(1)],
                    EventEffect::GainCard,
                ),
                supported(
                    "Sleep (heal 33% max HP)",
                    vec![EventProgramOp::heal_percent_hp(33)],
                    EventEffect::Hp(0),
                ),
            ],
        ),
        event(
            "The Mausoleum",
            vec![
                supported(
                    "Open (gain relic, maybe gain Writhe curse)",
                    vec![
                        EventProgramOp::gain_relic("random relic"),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Writhe".to_string(),
                        }),
                    ],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
    ]
}

#[allow(dead_code)]
pub fn act2_events() -> Vec<EventDef> {
    typed_act2_events()
        .into_iter()
        .map(|event| event.legacy())
        .collect()
}
