use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // The Bomb: 2 cost, deal 40 dmg to all enemies in 3 turns
    insert(cards, CardDef {
                id: "The Bomb", name: "The Bomb", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 40, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::THE_BOMB, A::Magic)),
                    E::Simple(SE::SetStatus(T::Player, sid::THE_BOMB_TURNS, A::Fixed(3))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "The Bomb+", name: "The Bomb+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 50, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::THE_BOMB, A::Magic)),
                    E::Simple(SE::SetStatus(T::Player, sid::THE_BOMB_TURNS, A::Fixed(3))),
                ], complex_hook: None,
            });
}
