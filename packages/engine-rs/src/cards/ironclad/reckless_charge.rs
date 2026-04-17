use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Reckless Charge ---- (cost 0, 7 dmg, shuffle Dazed into draw; +3 dmg)
    insert(cards, CardDef {
                id: "Reckless Charge", name: "Reckless Charge", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Dazed", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Reckless Charge+", name: "Reckless Charge+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Dazed", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
}
