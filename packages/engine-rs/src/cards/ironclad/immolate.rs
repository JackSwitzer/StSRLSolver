use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Immolate ---- (cost 2, 21 AoE dmg, add Burn to discard; +7 dmg)
    insert(cards, CardDef {
                id: "Immolate", name: "Immolate", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 21, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Immolate+", name: "Immolate+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 28, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
                ], complex_hook: None,
            });
}
