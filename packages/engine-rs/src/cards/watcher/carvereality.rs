use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Carve Reality ---- (cost 1, 6 dmg, add Smite to hand; +4 dmg upgrade)
    insert(cards, CardDef {
                id: "CarveReality", name: "Carve Reality", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["add_smite_to_hand"], effect_data: &[
                    E::Simple(SE::AddCard("Smite", P::Hand, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "CarveReality+", name: "Carve Reality+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["add_smite_to_hand"], effect_data: &[
                    E::Simple(SE::AddCard("Smite", P::Hand, A::Fixed(1))),
                ], complex_hook: None,
            });
}
