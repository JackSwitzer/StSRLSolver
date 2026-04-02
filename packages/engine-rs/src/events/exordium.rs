use super::{EventDef, EventOption, EventEffect};

pub fn act1_events() -> Vec<EventDef> {
    vec![
        EventDef {
            name: "Big Fish".to_string(),
            options: vec![
                EventOption { text: "Eat (heal 5 HP)".into(), effect: EventEffect::Hp(5) },
                EventOption { text: "Banana (gain 2 max HP)".into(), effect: EventEffect::MaxHp(2) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Golden Idol".to_string(),
            options: vec![
                EventOption { text: "Take (gain 300 gold, lose 25% max HP)".into(), effect: EventEffect::GoldenIdolTake },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Scrap Ooze".to_string(),
            options: vec![
                EventOption { text: "Reach inside (take 3 dmg, gain relic)".into(), effect: EventEffect::DamageAndGold(-3, 0) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Shining Light".to_string(),
            options: vec![
                EventOption { text: "Enter (upgrade 2 cards, take 10 dmg)".into(), effect: EventEffect::DamageAndGold(-10, 0) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Living Wall".to_string(),
            options: vec![
                EventOption { text: "Upgrade (upgrade a card)".into(), effect: EventEffect::UpgradeCard },
                EventOption { text: "Remove (remove a card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        // --- New events below (Java parity) ---
        EventDef {
            name: "The Cleric".to_string(),
            options: vec![
                EventOption { text: "Heal (pay 35 gold, heal 25% max HP)".into(), effect: EventEffect::Gold(-35) },
                EventOption { text: "Purify (pay 50 gold, remove a card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Dead Adventurer".to_string(),
            options: vec![
                EventOption { text: "Search (risk elite fight, gain gold/relic)".into(), effect: EventEffect::DamageAndGold(0, 30) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Golden Wing".to_string(),
            options: vec![
                EventOption { text: "Pray (take 7 dmg, remove a card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Attack (gain 50-80 gold if strong card)".into(), effect: EventEffect::Gold(65) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "World of Goop".to_string(),
            options: vec![
                EventOption { text: "Gather gold (gain 75 gold, take 11 dmg)".into(), effect: EventEffect::DamageAndGold(-11, 75) },
                EventOption { text: "Leave (lose some gold)".into(), effect: EventEffect::Gold(-35) },
            ],
        },
        EventDef {
            name: "Mushrooms".to_string(),
            options: vec![
                EventOption { text: "Stomp (fight, gain Odd Mushroom relic)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Eat (heal 25% max HP, gain Parasite curse)".into(), effect: EventEffect::Hp(0) },
            ],
        },
        EventDef {
            name: "Liars Game".to_string(),
            options: vec![
                EventOption { text: "Agree (gain 175 gold, gain Doubt curse)".into(), effect: EventEffect::Gold(175) },
                EventOption { text: "Disagree".into(), effect: EventEffect::Nothing },
            ],
        },
    ]
}

