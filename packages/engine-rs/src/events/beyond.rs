use super::{EventDef, EventOption, EventEffect};

pub fn act3_events() -> Vec<EventDef> {
    vec![
        EventDef {
            name: "Mysterious Sphere".to_string(),
            options: vec![
                EventOption { text: "Open (gain relic, fight)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Mind Bloom".to_string(),
            options: vec![
                EventOption { text: "I am War (fight Act 1 boss, gain rare relic)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "I am Awake (upgrade all, lose ability to heal)".into(), effect: EventEffect::UpgradeCard },
                EventOption { text: "I am Rich (gain 999 gold)".into(), effect: EventEffect::Gold(999) },
            ],
        },
        EventDef {
            name: "Tomb of Lord Red Mask".to_string(),
            options: vec![
                EventOption { text: "Don the mask (gain Red Mask)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Sensory Stone".to_string(),
            options: vec![
                EventOption { text: "Focus (gain a card)".into(), effect: EventEffect::GainCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Secret Portal".to_string(),
            options: vec![
                EventOption { text: "Enter (skip to boss)".into(), effect: EventEffect::Nothing },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
    ]
}

