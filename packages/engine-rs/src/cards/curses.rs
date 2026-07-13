use crate::cards::prelude::*;

pub fn register_curses(cards: &mut HashMap<&'static str, CardDef>) {
        insert(cards, CardDef {
            id: "Decay", name: "Decay", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Regret", name: "Regret", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Doubt", name: "Doubt", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Shame", name: "Shame", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            // Source: cards/curses/AscendersBane.java sets cost -2, Ethereal,
            // and leaves both use() and upgrade() empty.
            id: "AscendersBane", name: "Ascender's Bane", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });

        // AscendersBane already registered above

        // Source: cards/curses/Clumsy.java sets cost -2 and isEthereal, while
        // leaving use() and upgrade() empty.
        insert(cards, CardDef {
            id: "Clumsy", name: "Clumsy", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Source: cards/curses/CurseOfTheBell.java sets cost -2 with empty use
        // and upgrade methods. CardGroup.getPurgeableCards excludes its ID.
        insert(cards, CardDef {
            id: "CurseOfTheBell", name: "Curse of the Bell", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Injury: unplayable
        insert(cards, CardDef {
            id: "Injury", name: "Injury", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Necronomicurse: unplayable, cannot be removed
        insert(cards, CardDef {
            id: "Necronomicurse", name: "Necronomicurse", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Normality: unplayable, can only play 3 cards per turn
        insert(cards, CardDef {
            id: "Normality", name: "Normality", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Pain: unplayable, lose 1 HP when played from hand
        insert(cards, CardDef {
            id: "Pain", name: "Pain", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Parasite: unplayable, lose 3 max HP if removed
        insert(cards, CardDef {
            id: "Parasite", name: "Parasite", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Pride: 1 cost, exhaust, innate, add copy to draw pile at end of turn
        insert(cards, CardDef {
            id: "Pride", name: "Pride", card_type: CardType::Curse,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Writhe: unplayable, innate
        insert(cards, CardDef {
            id: "Writhe", name: "Writhe", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
}
