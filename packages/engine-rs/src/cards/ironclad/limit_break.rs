use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Limit Break ---- (cost 1, double str, exhaust; upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Limit Break", name: "Limit Break", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::MultiplyStatus(T::Player, sid::STRENGTH, 2)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Limit Break+", name: "Limit Break+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::MultiplyStatus(T::Player, sid::STRENGTH, 2)),
                ], complex_hook: None,
            });
}
