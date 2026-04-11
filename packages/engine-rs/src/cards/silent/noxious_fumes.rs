use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Noxious Fumes ---- (cost 1, power, 2 poison to all at turn start; +1)
    insert(cards, CardDef {
                id: "Noxious Fumes", name: "Noxious Fumes", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["noxious_fumes"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Noxious Fumes+", name: "Noxious Fumes+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["noxious_fumes"], effect_data: &[], complex_hook: None,
            });
}
