use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // JAX.java queues a literal 3-point LoseHPAction before applying
    // magicNumber Strength (2); upgradeMagicNumber(1) changes only Strength.
    insert(cards, CardDef {
                id: "J.A.X.", name: "J.A.X.", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-3))),
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "J.A.X.+", name: "J.A.X.+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-3))),
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                ], complex_hook: None,
            });
}
