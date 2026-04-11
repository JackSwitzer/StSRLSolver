use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Secret Weapon: 0 cost, choose Attack from draw pile, put in hand, exhaust (upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Secret Weapon", name: "Secret Weapon", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["search_attack"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Secret Weapon+", name: "Secret Weapon+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["search_attack"], effect_data: &[], complex_hook: None,
            });
}
