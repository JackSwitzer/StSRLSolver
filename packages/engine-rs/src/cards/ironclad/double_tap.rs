use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Double Tap ---- (cost 1, next attack played twice; upgrade: 2 attacks)
    insert(cards, CardDef {
                id: "Double Tap", name: "Double Tap", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::DOUBLE_TAP, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Double Tap+", name: "Double Tap+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::DOUBLE_TAP, A::Magic)),
                ], complex_hook: None,
            });
}
