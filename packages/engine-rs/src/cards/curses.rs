use crate::cards::prelude::*;

pub fn register_curses(cards: &mut HashMap<&'static str, CardDef>) {
        // Source: reference/extracted/methods/card/Decay.java is an unplayable
        // Curse with no magic number; its end-turn auto-play deals a literal 2.
        insert(cards, CardDef {
            id: "Decay", name: "Decay", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Regret", name: "Regret", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Doubt.java is an unplayable Curse with no magic number. Its queued
        // end-turn auto-play applies a literal 1 Weak.
        // Java: reference/extracted/methods/card/Doubt.java
        insert(cards, CardDef {
            id: "Doubt", name: "Doubt", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
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
        // Injury.java constructs a cost -2 Curse with target NONE and leaves
        // both use() and upgrade() empty; it does not set Ethereal.
        insert(cards, CardDef {
            id: "Injury", name: "Injury", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Necronomicurse.java is an unplayable, unupgradable Curse. Its
        // triggerOnExhaust creates a fresh copy in hand, and master-deck
        // removal recreates it through NecronomicurseEffect.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Necronomicurse.java
        insert(cards, CardDef {
            id: "Necronomicurse", name: "Necronomicurse", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Normality.java constructs an unplayable, unupgradable Curse and its
        // canPlay hook rejects every card once three cards have been played
        // while this copy remains in hand.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Normality.java
        insert(cards, CardDef {
            id: "Normality", name: "Normality", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Pain.java is an unplayable, unupgradable Curse. While it remains in
        // hand, triggerOnOtherCardPlayed queues a separate LoseHPAction(1) for
        // every other card played.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pain.java
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
