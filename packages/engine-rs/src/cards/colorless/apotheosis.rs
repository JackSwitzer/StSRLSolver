use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Colorless Rare ----
        // Apotheosis: 2 cost, upgrade all cards in deck, exhaust (upgrade: cost 1)
    insert(cards, CardDef {
                id: "Apotheosis", name: "Apotheosis", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["upgrade_all_cards"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Apotheosis+", name: "Apotheosis+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["upgrade_all_cards"], effect_data: &[], complex_hook: None,
            });
}
