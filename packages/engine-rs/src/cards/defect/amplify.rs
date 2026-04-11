use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Amplify: 1 cost, next power played this turn is played twice
    insert(cards, CardDef {
                id: "Amplify", name: "Amplify", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["amplify_power"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::AMPLIFY, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Amplify+", name: "Amplify+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["amplify_power"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::AMPLIFY, A::Magic)),
                ], complex_hook: None,
            });
}
