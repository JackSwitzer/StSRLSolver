use super::{EventDef, EventOption, EventEffect};

pub fn act2_events() -> Vec<EventDef> {
    vec![
        EventDef {
            name: "Forgotten Altar".to_string(),
            options: vec![
                EventOption { text: "Offer (lose 5 HP, gain golden idol)".into(), effect: EventEffect::Hp(-5) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Council of Ghosts".to_string(),
            options: vec![
                EventOption { text: "Accept (gain 5 Apparitions, lose max HP)".into(), effect: EventEffect::MaxHp(-5) },
                EventOption { text: "Refuse".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Masked Bandits".to_string(),
            options: vec![
                EventOption { text: "Pay (lose all gold)".into(), effect: EventEffect::Gold(-999) },
                EventOption { text: "Fight".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Knowing Skull".to_string(),
            options: vec![
                EventOption { text: "Ask for gold (gain 90 gold, lose 10% HP)".into(), effect: EventEffect::DamageAndGold(-6, 90) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Vampires".to_string(),
            options: vec![
                EventOption { text: "Accept (remove Strikes, gain Bites)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Refuse".into(), effect: EventEffect::Nothing },
            ],
        },
    ]
}

