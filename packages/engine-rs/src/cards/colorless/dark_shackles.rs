use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Dark Shackles: 0 cost, reduce enemy str by 9 for one turn, exhaust
    insert(cards, CardDef {
                id: "Dark Shackles", name: "Dark Shackles", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 9, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::LOSE_STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dark Shackles+", name: "Dark Shackles+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 15, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::LOSE_STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
}
