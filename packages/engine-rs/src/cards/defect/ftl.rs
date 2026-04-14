use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // FTL: 0 cost, 5 dmg, draw 1 if <3 cards played this turn
    insert(cards, CardDef {
                id: "FTL", name: "FTL", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 5, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["draw_if_few_cards_played"], effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::CardsPlayedThisTurnLessThan(3),
                        &[E::Simple(SE::DrawCards(A::Magic))],
                        &[],
                    ),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "FTL+", name: "FTL+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["draw_if_few_cards_played"], effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::CardsPlayedThisTurnLessThan(4),
                        &[E::Simple(SE::DrawCards(A::Magic))],
                        &[],
                    ),
                ], complex_hook: None,
            });
}
