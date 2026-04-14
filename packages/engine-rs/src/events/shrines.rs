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

pub fn typed_shrine_events() -> Vec<TypedEventDef> {
    vec![
        event(
            "Accursed Blacksmith",
            vec![
                supported(
                    "Forge (upgrade a card)",
                    vec![EventProgramOp::upgrade_card(1)],
                    EventEffect::UpgradeCard,
                ),
                supported(
                    "Rummage (obtain random relic, gain Pain curse)",
                    vec![
                        EventProgramOp::gain_relic("random relic"),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Pain".to_string(),
                        }),
                    ],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Bonfire Elementals",
            vec![
                supported(
                    "Offer (remove a card, reward based on rarity)",
                    vec![EventProgramOp::deck_selection("deck_selection_bonfire_offer")],
                    EventEffect::RemoveCard,
                ),
            ],
        ),
        event(
            "Designer",
            vec![
                supported(
                    "Adjustment (pay gold, upgrade 1-2 cards)",
                    vec![
                        EventProgramOp::gold(-75),
                        EventProgramOp::upgrade_card(2),
                    ],
                    EventEffect::UpgradeCard,
                ),
                supported(
                    "Clean Up (pay gold, remove or transform cards)",
                    vec![
                        EventProgramOp::gold(-75),
                        EventProgramOp::remove_card(1),
                    ],
                    EventEffect::RemoveCard,
                ),
                supported(
                    "Full Service (pay gold, remove + upgrade)",
                    vec![
                        EventProgramOp::gold(-100),
                        EventProgramOp::remove_card(1),
                        EventProgramOp::upgrade_card(1),
                    ],
                    EventEffect::RemoveCard,
                ),
                supported(
                    "Punch (take HP loss)",
                    vec![EventProgramOp::hp(-5)],
                    EventEffect::Hp(-5),
                ),
            ],
        ),
        event(
            "Duplicator",
            vec![
                supported(
                    "Pray (duplicate a card)",
                    vec![EventProgramOp::duplicate_card(1)],
                    EventEffect::DuplicateCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "FaceTrader",
            vec![
                supported(
                    "Touch (take dmg, gain gold, swap face relic)",
                    vec![
                        EventProgramOp::hp(-5),
                        EventProgramOp::gold(100),
                        EventProgramOp::gain_relic("Face Trader reward"),
                    ],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Fountain of Cleansing",
            vec![
                supported(
                    "Drink (remove all removable curses)",
                    vec![EventProgramOp::remove_card(999)],
                    EventEffect::RemoveCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Golden Shrine",
            vec![
                supported(
                    "Pray (gain 50-100 gold)",
                    vec![EventProgramOp::gold(100)],
                    EventEffect::Gold(100),
                ),
                supported(
                    "Desecrate (gain 275 gold, gain Regret curse)",
                    vec![
                        EventProgramOp::gold(275),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Regret".to_string(),
                        }),
                    ],
                    EventEffect::GoldAndCurse(275),
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Match and Keep!",
            vec![
                supported(
                    "Play (temporary fixed reward)",
                    vec![EventProgramOp::gain_specific_cards(["Adaptation+"])],
                    EventEffect::GainCard,
                ),
            ],
        ),
        event(
            "Wheel of Change",
            vec![
                supported(
                    "Spin (random outcome)",
                    vec![EventProgramOp::random_outcome_table(vec![
                        vec![EventProgramOp::gold_by_act(100, 200, 300)],
                        vec![EventProgramOp::gain_relic("random relic")],
                        vec![EventProgramOp::heal_to_full()],
                        vec![EventProgramOp::Reward(EventReward::Curse {
                            label: "Decay".to_string(),
                        })],
                        vec![EventProgramOp::deck_selection("deck_selection_purge")],
                        vec![EventProgramOp::adjust_hp_percent_by_ascension(
                            false,
                            10,
                            15,
                        )],
                    ])],
                    EventEffect::Nothing,
                ),
            ],
        ),
        event(
            "Lab",
            vec![
                supported(
                    "Search (gain 3 random potions)",
                    vec![EventProgramOp::gain_potion(3)],
                    EventEffect::GainPotion,
                ),
            ],
        ),
        event(
            "N'loth",
            vec![
                supported(
                    "Trade relic 1 (exchange for N'loth's Gift)",
                    vec![EventProgramOp::gain_relic("N'loth's Gift")],
                    EventEffect::GainRelic,
                ),
                supported(
                    "Trade relic 2 (exchange for N'loth's Gift)",
                    vec![EventProgramOp::gain_relic("N'loth's Gift")],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "NoteForYourself",
            vec![
                supported(
                    "Take (take the note card)",
                    vec![EventProgramOp::gain_card_reward(1)],
                    EventEffect::GainCard,
                ),
                supported("Leave (leave a new note)", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Purifier",
            vec![
                supported(
                    "Pray (remove a card)",
                    vec![EventProgramOp::remove_card(1)],
                    EventEffect::RemoveCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Transmorgrifier",
            vec![
                supported(
                    "Pray (transform a card)",
                    vec![EventProgramOp::transform_card(1)],
                    EventEffect::TransformCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Upgrade Shrine",
            vec![
                supported(
                    "Pray (upgrade a card)",
                    vec![EventProgramOp::upgrade_card(1)],
                    EventEffect::UpgradeCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "WeMeetAgain",
            vec![
                supported(
                    "Give potion (gain relic)",
                    vec![EventProgramOp::gain_relic("random relic")],
                    EventEffect::GainRelic,
                ),
                supported(
                    "Give gold (gain card)",
                    vec![EventProgramOp::gain_card_reward(1)],
                    EventEffect::GainCard,
                ),
                supported(
                    "Give card (gain potion)",
                    vec![EventProgramOp::gain_potion(1)],
                    EventEffect::GainPotion,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "The Woman in Blue",
            vec![
                supported(
                    "Buy 1 potion (pay 20 gold)",
                    vec![EventProgramOp::gold(-20), EventProgramOp::gain_potion(1)],
                    EventEffect::GainPotion,
                ),
                supported(
                    "Buy 2 potions (pay 30 gold)",
                    vec![EventProgramOp::gold(-30), EventProgramOp::gain_potion(2)],
                    EventEffect::GainPotion,
                ),
                supported(
                    "Buy 3 potions (pay 40 gold)",
                    vec![EventProgramOp::gold(-40), EventProgramOp::gain_potion(3)],
                    EventEffect::GainPotion,
                ),
                supported(
                    "Leave (take 5% max HP dmg)",
                    vec![EventProgramOp::lose_percent_hp(5)],
                    EventEffect::LosePercentHp(5),
                ),
            ],
        ),
    ]
}

#[allow(dead_code)]
pub fn shrine_events() -> Vec<EventDef> {
    typed_shrine_events()
        .into_iter()
        .map(|event| event.legacy())
        .collect()
}
