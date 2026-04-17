use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Dodge and Roll ---- (cost 1, 4 block, next turn 4 block; +2/+2)
    insert(cards, CardDef {
                id: "Dodge and Roll", name: "Dodge and Roll", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 4,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::NEXT_TURN_BLOCK, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dodge and Roll+", name: "Dodge and Roll+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
                base_magic: 6, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::NEXT_TURN_BLOCK, A::Magic)),
                ], complex_hook: None,
            });
}
