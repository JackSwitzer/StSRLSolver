use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Wild Strike ---- (cost 1, 12 dmg, shuffle Wound into draw; +5 dmg)
    insert(cards, CardDef {
                id: "Wild Strike", name: "Wild Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["add_wound_to_draw"], effect_data: &[
                    E::Simple(SE::AddCard("Wound", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Wild Strike+", name: "Wild Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 17, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["add_wound_to_draw"], effect_data: &[
                    E::Simple(SE::AddCard("Wound", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
}
