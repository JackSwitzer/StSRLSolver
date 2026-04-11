use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Panic Button: 0 cost, 30 block, no block next 2 turns, exhaust
    insert(cards, CardDef {
                id: "PanicButton", name: "Panic Button", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 30,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["no_block_next_turns"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::NO_BLOCK, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "PanicButton+", name: "Panic Button+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 40,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["no_block_next_turns"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::NO_BLOCK, A::Magic)),
                ], complex_hook: None,
            });
}
