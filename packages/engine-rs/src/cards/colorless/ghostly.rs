use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Colorless Special ----
        // Apparition (Java ID: Ghostly): 1 cost, gain 1 Intangible, exhaust, ethereal
    insert(cards, CardDef {
                id: "Ghostly", name: "Apparition", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["intangible", "ethereal"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Ghostly+", name: "Apparition+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["intangible"], effect_data: &[], complex_hook: None,
            });
}
