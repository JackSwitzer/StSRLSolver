use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Predator ---- (cost 2, 15 dmg, draw 2 next turn; +5 dmg)
    insert(cards, CardDef {
                id: "Predator", name: "Predator", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["draw_next_turn"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DRAW_CARD, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Predator+", name: "Predator+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["draw_next_turn"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DRAW_CARD, A::Magic)),
                ], complex_hook: None,
            });
}
