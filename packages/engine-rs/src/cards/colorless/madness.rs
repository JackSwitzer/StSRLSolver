use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Madness: 1 cost, reduce random card in hand to 0 cost, exhaust (upgrade: cost 0)
    insert(cards, CardDef {
                id: "Madness", name: "Madness", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["madness"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Madness+", name: "Madness+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["madness"], effect_data: &[], complex_hook: None,
            });
}
