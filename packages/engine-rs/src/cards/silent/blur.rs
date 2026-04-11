use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Blur ---- (cost 1, 5 block, block not removed next turn; +3 block)
    insert(cards, CardDef {
                id: "Blur", name: "Blur", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["retain_block"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::BLUR, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Blur+", name: "Blur+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["retain_block"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::BLUR, A::Fixed(1))),
                ], complex_hook: None,
            });
}
