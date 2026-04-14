use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Rampage", name: "Rampage", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["rampage"],
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::ModifyPlayedCardDamage(A::Magic)),
                ],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Rampage+", name: "Rampage+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 8, exhaust: false, enter_stance: None,
                effects: &["rampage"],
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::ModifyPlayedCardDamage(A::Magic)),
                ],
                complex_hook: None,
            });
}
