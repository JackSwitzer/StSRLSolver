use super::{
    EventDef, EventEffect, EventProgram, EventProgramOp, EventReward, TypedEventDef,
    TypedEventOption,
};

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave7.rs"]
mod test_event_runtime_wave7;

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave8.rs"]
mod test_event_runtime_wave8;

#[cfg(test)]
#[path = "../tests/test_event_runtime_wave13.rs"]
mod test_event_runtime_wave13;

fn supported(text: &str, ops: Vec<EventProgramOp>, effect: EventEffect) -> TypedEventOption {
    TypedEventOption::supported(text, EventProgram::from_ops(ops), effect)
}

fn event(name: &str, options: Vec<TypedEventOption>) -> TypedEventDef {
    TypedEventDef {
        name: name.to_string(),
        options,
    }
}

pub fn typed_act3_events() -> Vec<TypedEventDef> {
    vec![
        event(
            "Mysterious Sphere",
            vec![
                supported(
                    "Open (gain relic, fight)",
                    vec![
                        EventProgramOp::combat_branch(
                            ["OrbWalker", "OrbWalker"],
                            vec![EventProgramOp::gain_relic("random relic")],
                        ),
                    ],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Mind Bloom",
            vec![
                supported(
                    "I am War (fight Act 1 boss, gain rare relic)",
                    vec![
                        EventProgramOp::combat_branch(
                            ["MindBloomAct1Boss"],
                            vec![EventProgramOp::gain_relic("rare relic")],
                        ),
                    ],
                    EventEffect::GainRelic,
                ),
                supported(
                    "I am Awake (upgrade all, lose ability to heal)",
                    vec![
                        EventProgramOp::upgrade_card(999),
                        EventProgramOp::gain_relic("Mark of the Bloom"),
                    ],
                    EventEffect::UpgradeCard,
                ),
                supported(
                    "I am Rich (gain 999 gold)",
                    vec![
                        EventProgramOp::gold(999),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Normality".to_string(),
                        }),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Normality".to_string(),
                        }),
                    ],
                    EventEffect::Gold(999),
                ),
            ],
        ),
        event(
            "Tomb of Lord Red Mask",
            vec![
                supported(
                    "Don the mask (gain Red Mask)",
                    vec![EventProgramOp::gain_relic("Red Mask")],
                    EventEffect::GainRelic,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Sensory Stone",
            vec![
                supported(
                    "Focus (gain a card)",
                    vec![EventProgramOp::gain_card_reward(1)],
                    EventEffect::GainCard,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Secret Portal",
            vec![
                supported(
                    "Enter (skip to boss)",
                    vec![EventProgramOp::start_boss_combat()],
                    EventEffect::Nothing,
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Falling",
            vec![
                supported(
                    "Land on skill (lose a skill card)",
                    vec![EventProgramOp::remove_card(1)],
                    EventEffect::RemoveCard,
                ),
                supported(
                    "Land on power (lose a power card)",
                    vec![EventProgramOp::remove_card(1)],
                    EventEffect::RemoveCard,
                ),
                supported(
                    "Land on attack (lose an attack card)",
                    vec![EventProgramOp::remove_card(1)],
                    EventEffect::RemoveCard,
                ),
            ],
        ),
        event(
            "The Moai Head",
            vec![
                supported(
                    "Offer (lose max HP, heal to full)",
                    vec![
                        EventProgramOp::max_hp(-5),
                        EventProgramOp::heal_to_full(),
                    ],
                    EventEffect::MaxHp(-5),
                ),
                supported(
                    "Give Golden Idol (gain 333 gold)",
                    vec![
                        EventProgramOp::remove_relic("Golden Idol"),
                        EventProgramOp::gold(333),
                    ],
                    EventEffect::Gold(333),
                ),
                supported("Leave", vec![EventProgramOp::nothing()], EventEffect::Nothing),
            ],
        ),
        event(
            "Spire Heart",
            vec![
                supported(
                    "Approach (deal score dmg, end run or enter Act 4)",
                    vec![EventProgramOp::resolve_final_act()],
                    EventEffect::Nothing,
                ),
            ],
        ),
        event(
            "Winding Halls",
            vec![
                supported(
                    "Embrace madness (take dmg, gain 2 Madness)",
                    vec![
                        EventProgramOp::adjust_hp_percent_by_ascension(false, 12, 18),
                        EventProgramOp::gain_specific_cards(["Madness"]),
                        EventProgramOp::gain_specific_cards(["Madness"]),
                    ],
                    EventEffect::Hp(-5),
                ),
                supported(
                    "Retrace steps (heal, gain Writhe curse)",
                    vec![
                        EventProgramOp::adjust_hp_percent_by_ascension(true, 25, 20),
                        EventProgramOp::Reward(EventReward::Curse {
                            label: "Writhe".to_string(),
                        }),
                    ],
                    EventEffect::Hp(0),
                ),
                supported(
                    "Press on (lose max HP)",
                    vec![EventProgramOp::max_hp_percent(-5)],
                    EventEffect::MaxHp(-3),
                ),
            ],
        ),
    ]
}

#[allow(dead_code)]
pub fn act3_events() -> Vec<EventDef> {
    typed_act3_events()
        .into_iter()
        .map(|event| event.legacy())
        .collect()
}
