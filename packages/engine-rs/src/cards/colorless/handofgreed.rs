use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Hand of Greed: 2 cost, 20 dmg, if kill gain 20 gold
    insert(cards, CardDef {
                id: "HandOfGreed", name: "Hand of Greed", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
                base_magic: 20, exhaust: false, enter_stance: None,
                effects: &["gold_on_kill"], effect_data: &[
                    E::Conditional(Cond::EnemyKilled, &[E::Simple(SE::ModifyGold(A::Magic))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "HandOfGreed+", name: "Hand of Greed+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 25, base_block: -1,
                base_magic: 25, exhaust: false, enter_stance: None,
                effects: &["gold_on_kill"], effect_data: &[
                    E::Conditional(Cond::EnemyKilled, &[E::Simple(SE::ModifyGold(A::Magic))], &[]),
                ], complex_hook: None,
            });
}
