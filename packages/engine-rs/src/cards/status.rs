use crate::cards::prelude::*;

pub fn register_status(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Universal Status/Curse Cards ----
        // Slimed.java is a one-cost, SELF-targeting Status with empty use and
        // upgrade methods; playing it only exhausts the card.
        // Java: reference/extracted/methods/card/Slimed.java
        insert(cards, CardDef {
            id: "Slimed", name: "Slimed", card_type: CardType::Status,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Wound", name: "Wound", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Source: reference/extracted/methods/card/Dazed.java is an unplayable
        // Ethereal Status with empty use and upgrade methods.
        insert(cards, CardDef {
            id: "Dazed", name: "Dazed", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
        // Unplayable; while held at end of turn, queues 2 THORNS damage (4 upgraded).
        insert(cards, CardDef {
            id: "Burn", name: "Burn", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Burn+", name: "Burn+", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
        // Void is an unplayable, non-upgradable Ethereal Status whose draw
        // trigger queues LoseEnergyAction(1).
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/VoidCard.java
        insert(cards, CardDef {
            id: "Void", name: "Void", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
        });
}
