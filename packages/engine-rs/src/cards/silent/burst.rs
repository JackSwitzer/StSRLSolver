use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Burst ---- (cost 1, next skill played twice; upgrade: next 2 skills)
    insert(cards, CardDef {
                id: "Burst", name: "Burst", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["burst"], effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::BURST, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Burst+", name: "Burst+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["burst"], effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::BURST, A::Magic)),
                ], complex_hook: None,
            });
}
