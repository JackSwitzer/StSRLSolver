use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "CrushJoints", name: "Crush Joints", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::LastCardType(CardType::Skill), &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "CrushJoints+", name: "Crush Joints+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::LastCardType(CardType::Skill), &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic))], &[]),
                ], complex_hook: None,
            });
}
