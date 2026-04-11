use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "TalkToTheHand", name: "Talk to the Hand", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["apply_block_return"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::BLOCK_RETURN, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "TalkToTheHand+", name: "Talk to the Hand+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["apply_block_return"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::BLOCK_RETURN, A::Magic)),
                ], complex_hook: None,
            });
}
