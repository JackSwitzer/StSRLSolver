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
        // --- New events below (Java parity) ---
        EventDef {
            name: "Addict".to_string(),
            options: vec![
                EventOption { text: "Pay (lose 85 gold, gain relic)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Rob (gain relic, gain Shame curse)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Back to Basics".to_string(),
            options: vec![
                EventOption { text: "Elegance (remove a card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Simplicity (upgrade all Strikes/Defends)".into(), effect: EventEffect::UpgradeCard },
            ],
        },
        EventDef {
            name: "Beggar".to_string(),
            options: vec![
                EventOption { text: "Donate (pay 75 gold, remove a card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Colosseum".to_string(),
            options: vec![
                EventOption { text: "Enter (fight Slavers, then optional Nobs)".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Cursed Tome".to_string(),
            options: vec![
                EventOption { text: "Read (take progressive dmg, gain book relic)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Drug Dealer".to_string(),
            options: vec![
                EventOption { text: "Obtain J.A.X. (gain J.A.X. card)".into(), effect: EventEffect::GainCard },
                EventOption { text: "Become test subject (transform 2 cards)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Inject mutagens (gain Mutagenic Strength relic)".into(), effect: EventEffect::GainRelic },
            ],
        },
        EventDef {
            name: "Nest".to_string(),
            options: vec![
                EventOption { text: "Steal gold (gain 99 gold)".into(), effect: EventEffect::Gold(99) },
                EventOption { text: "Join (take 6 dmg, gain Ritual Dagger)".into(), effect: EventEffect::DamageAndGold(-6, 0) },
            ],
        },
        EventDef {
            name: "The Joust".to_string(),
            options: vec![
                EventOption { text: "Bet on Murderer (pay 50 gold, win 100)".into(), effect: EventEffect::Gold(-50) },
                EventOption { text: "Bet on Owner (pay 50 gold, win 250)".into(), effect: EventEffect::Gold(-50) },
            ],
        },
        EventDef {
            name: "The Library".to_string(),
            options: vec![
                EventOption { text: "Read (choose 1 of 20 cards)".into(), effect: EventEffect::GainCard },
                EventOption { text: "Sleep (heal 33% max HP)".into(), effect: EventEffect::Hp(0) },
            ],
        },
        EventDef {
            name: "The Mausoleum".to_string(),
            options: vec![
                EventOption { text: "Open (gain relic, maybe gain Writhe curse)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
    ]
}

