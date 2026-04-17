use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon Watcher Cards ----
    insert(cards, CardDef {
                id: "InnerPeace", name: "Inner Peace", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::InStance(Stance::Calm), &[E::Simple(SE::DrawCards(A::Magic))], &[E::Simple(SE::ChangeStance(Stance::Calm))]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "InnerPeace+", name: "Inner Peace+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::InStance(Stance::Calm), &[E::Simple(SE::DrawCards(A::Magic))], &[E::Simple(SE::ChangeStance(Stance::Calm))]),
                ], complex_hook: None,
            });
}
