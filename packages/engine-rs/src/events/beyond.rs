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
        // --- New events below (Java parity) ---
        EventDef {
            name: "Falling".to_string(),
            options: vec![
                EventOption { text: "Land on skill (lose a skill card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Land on power (lose a power card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Land on attack (lose an attack card)".into(), effect: EventEffect::RemoveCard },
            ],
        },
        EventDef {
            name: "The Moai Head".to_string(),
            options: vec![
                EventOption { text: "Offer (lose max HP, heal to full)".into(), effect: EventEffect::MaxHp(-5) },
                EventOption { text: "Give Golden Idol (gain 333 gold)".into(), effect: EventEffect::Gold(333) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Spire Heart".to_string(),
            options: vec![
                EventOption { text: "Approach (deal score dmg, end run or enter Act 4)".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Winding Halls".to_string(),
            options: vec![
                EventOption { text: "Embrace madness (take dmg, gain 2 Madness)".into(), effect: EventEffect::Hp(-5) },
                EventOption { text: "Retrace steps (heal, gain Writhe curse)".into(), effect: EventEffect::Hp(0) },
                EventOption { text: "Press on (lose max HP)".into(), effect: EventEffect::MaxHp(-3) },
            ],
        },
    ]
}

