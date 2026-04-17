use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Flex ---- (cost 0, +2 str this turn; +2 magic)
    insert(cards, CardDef {
                id: "Flex", name: "Flex", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                    E::Simple(SE::AddStatus(T::Player, sid::TEMP_STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Flex+", name: "Flex+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                    E::Simple(SE::AddStatus(T::Player, sid::TEMP_STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
}
