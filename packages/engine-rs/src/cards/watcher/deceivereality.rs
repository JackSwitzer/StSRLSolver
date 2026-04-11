use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Deceive Reality ---- (cost 1, 4 block, add Safety to hand; +3 block upgrade)
    insert(cards, CardDef {
                id: "DeceiveReality", name: "Deceive Reality", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 4,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["add_safety_to_hand"], effect_data: &[
                    E::Simple(SE::AddCard("Safety", P::Hand, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "DeceiveReality+", name: "Deceive Reality+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["add_safety_to_hand"], effect_data: &[
                    E::Simple(SE::AddCard("Safety", P::Hand, A::Fixed(1))),
                ], complex_hook: None,
            });
}
