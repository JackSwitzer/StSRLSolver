use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ForTheEyesAction applies Weak only when getIntentBaseDmg() >= 0,
        // covering all four attacking intent variants.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ForTheEyesAction.java
    insert(cards, CardDef {
                id: "Go for the Eyes", name: "Go for the Eyes", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::EnemyAttacking, &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Go for the Eyes+", name: "Go for the Eyes+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::EnemyAttacking, &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], &[]),
                ], complex_hook: None,
            });
}
