use super::{EventDef, EventOption, EventEffect};

pub fn shrine_events() -> Vec<EventDef> {
    vec![
        EventDef {
            name: "Accursed Blacksmith".to_string(),
            options: vec![
                EventOption { text: "Forge (upgrade a card)".into(), effect: EventEffect::UpgradeCard },
                EventOption { text: "Rummage (obtain random relic, gain Pain curse)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Bonfire Elementals".to_string(),
            options: vec![
                EventOption { text: "Offer (remove a card, reward based on rarity)".into(), effect: EventEffect::RemoveCard },
            ],
        },
        EventDef {
            name: "Designer".to_string(),
            options: vec![
                EventOption { text: "Adjustment (pay gold, upgrade 1-2 cards)".into(), effect: EventEffect::UpgradeCard },
                EventOption { text: "Clean Up (pay gold, remove or transform cards)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Full Service (pay gold, remove + upgrade)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Punch (take HP loss)".into(), effect: EventEffect::Hp(-5) },
            ],
        },
        EventDef {
            name: "Duplicator".to_string(),
            options: vec![
                EventOption { text: "Pray (duplicate a card)".into(), effect: EventEffect::DuplicateCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "FaceTrader".to_string(),
            options: vec![
                EventOption { text: "Touch (take dmg, gain gold, swap face relic)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Fountain of Cleansing".to_string(),
            options: vec![
                EventOption { text: "Drink (remove all removable curses)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Golden Shrine".to_string(),
            options: vec![
                EventOption { text: "Pray (gain 50-100 gold)".into(), effect: EventEffect::Gold(100) },
                EventOption { text: "Desecrate (gain 275 gold, gain Regret curse)".into(), effect: EventEffect::GoldAndCurse(275) },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Match and Keep!".to_string(),
            options: vec![
                EventOption { text: "Play (match cards to keep them)".into(), effect: EventEffect::GainCard },
            ],
        },
        EventDef {
            name: "Wheel of Change".to_string(),
            options: vec![
                EventOption { text: "Spin (random outcome)".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Lab".to_string(),
            options: vec![
                EventOption { text: "Search (gain 3 random potions)".into(), effect: EventEffect::GainPotion },
            ],
        },
        EventDef {
            name: "N'loth".to_string(),
            options: vec![
                EventOption { text: "Trade relic 1 (exchange for N'loth's Gift)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Trade relic 2 (exchange for N'loth's Gift)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "NoteForYourself".to_string(),
            options: vec![
                EventOption { text: "Take (take the note card)".into(), effect: EventEffect::GainCard },
                EventOption { text: "Leave (leave a new note)".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Purifier".to_string(),
            options: vec![
                EventOption { text: "Pray (remove a card)".into(), effect: EventEffect::RemoveCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Transmorgrifier".to_string(),
            options: vec![
                EventOption { text: "Pray (transform a card)".into(), effect: EventEffect::TransformCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "Upgrade Shrine".to_string(),
            options: vec![
                EventOption { text: "Pray (upgrade a card)".into(), effect: EventEffect::UpgradeCard },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "WeMeetAgain".to_string(),
            options: vec![
                EventOption { text: "Give potion (gain relic)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Give gold (gain relic)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Give card (gain relic)".into(), effect: EventEffect::GainRelic },
                EventOption { text: "Leave".into(), effect: EventEffect::Nothing },
            ],
        },
        EventDef {
            name: "The Woman in Blue".to_string(),
            options: vec![
                EventOption { text: "Buy 1 potion (pay 20 gold)".into(), effect: EventEffect::GainPotion },
                EventOption { text: "Buy 2 potions (pay 30 gold)".into(), effect: EventEffect::GainPotion },
                EventOption { text: "Buy 3 potions (pay 40 gold)".into(), effect: EventEffect::GainPotion },
                EventOption { text: "Leave (take 5% max HP dmg)".into(), effect: EventEffect::LosePercentHp(5) },
            ],
        },
    ]
}
