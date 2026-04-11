use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Deadly Poison ---- (cost 1, 5 poison; +2)
    insert(cards, CardDef {
                id: "Deadly Poison", name: "Deadly Poison", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["poison"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Deadly Poison+", name: "Deadly Poison+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effects: &["poison"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
                ], complex_hook: None,
            });
}
