use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Sanctity ---- (cost 1, 6 block, draw 2 if last card played was Skill; +3 block upgrade)
    insert(cards, CardDef {
                id: "Sanctity", name: "Sanctity", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
                base_magic: 2, exhaust: false, enter_stance: None, effects: &[], effect_data: &[
                    E::Conditional(Cond::LastCardType(CardType::Skill), &[E::Simple(SE::DrawCards(A::Magic))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sanctity+", name: "Sanctity+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
                base_magic: 2, exhaust: false, enter_stance: None, effects: &[], effect_data: &[
                    E::Conditional(Cond::LastCardType(CardType::Skill), &[E::Simple(SE::DrawCards(A::Magic))], &[]),
                ], complex_hook: None,
            });
}
