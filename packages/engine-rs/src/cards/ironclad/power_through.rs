use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Power Through ---- (cost 1, 15 block, add 2 Wounds to hand; +5 block)
    insert(cards, CardDef {
                id: "Power Through", name: "Power Through", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 15,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Wound", P::Hand, A::Fixed(2))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Power Through+", name: "Power Through+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 20,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Wound", P::Hand, A::Fixed(2))),
                ], complex_hook: None,
            });
}
