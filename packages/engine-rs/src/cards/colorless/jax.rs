use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // J.A.X.: 0 cost, lose 3 HP, gain 2 str
    insert(cards, CardDef {
                id: "J.A.X.", name: "J.A.X.", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["lose_hp_gain_str"], effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-3))),
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "J.A.X.+", name: "J.A.X.+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["lose_hp_gain_str"], effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-3))),
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
}
