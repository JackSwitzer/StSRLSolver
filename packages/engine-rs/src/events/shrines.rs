use super::{
    EventEffect, EventProgram, EventProgramOp, EventReward, TypedEventDef, TypedEventOption,
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

fn face_trader_main_event() -> TypedEventDef {
    event(
        "FaceTrader",
        vec![
            supported(
                "Touch",
                vec![EventProgramOp::face_trader_touch()],
                EventEffect::DamageAndGold(0, 0),
            ),
            supported(
                "Trade for a face",
                vec![EventProgramOp::obtain_random_face()],
                EventEffect::GainRelic,
            ),
            supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
        ],
    )
}

fn note_for_yourself_choose_state() -> TypedEventDef {
    event(
        "NoteForYourself",
        vec![
            supported(
                "Take (claim the stored note, then leave one card for a future run)",
                vec![
                    EventProgramOp::Reward(EventReward::StoredNoteCard),
                    EventProgramOp::deck_selection("deck_selection_note_for_yourself"),
                ],
                EventEffect::GainCard,
            ),
            supported(
                "Leave (ignore the note)",
                vec![EventProgramOp::nothing()],
                EventEffect::Nothing,
            ),
        ],
    )
}

fn nloth_complete_event() -> TypedEventDef {
    event(
        "N'loth (complete)",
        vec![supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing)],
    )
}

pub(crate) fn nloth_trade_program(relic_id: &str, already_has_gift: bool) -> EventProgram {
    let mut ops = Vec::new();
    if !already_has_gift {
        ops.push(EventProgramOp::remove_relic(relic_id));
    }
    ops.push(EventProgramOp::obtain_relic("Nloth's Gift"));
    ops.push(EventProgramOp::continue_event(nloth_complete_event()));
    EventProgram::from_ops(ops)
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
                    "Rummage (obtain Warped Tongs, gain Pain curse)",
                    vec![
                        EventProgramOp::gain_relic("WarpedTongs"),
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
            vec![supported(
                "Approach the Face Trader",
                vec![
                    // FaceTrader.java first reveals a second screen containing
                    // touch, trade, and leave; the intro choice has no reward.
                    // Java: decompiled/java-src/com/megacrit/cardcrawl/events/shrines/FaceTrader.java
                    EventProgramOp::continue_event(face_trader_main_event()),
                ],
                EventEffect::Nothing,
            )],
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
                    "Hear the rules",
                    vec![EventProgramOp::nothing()],
                    EventEffect::Nothing,
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
                    vec![EventProgramOp::nothing()],
                    EventEffect::GainRelic,
                ),
                supported(
                    "Trade relic 2 (exchange for N'loth's Gift)",
                    vec![EventProgramOp::nothing()],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "NoteForYourself",
            vec![
                supported(
                    "Read the note",
                    vec![EventProgramOp::continue_event(note_for_yourself_choose_state())],
                    EventEffect::Nothing,
                ),
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
