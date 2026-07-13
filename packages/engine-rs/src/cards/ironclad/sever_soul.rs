use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java queues ExhaustAllNonAttackAction before its single DamageAction;
    // upgrading raises only damage from 16 to 22.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/SeverSoul.java
    insert(cards, CardDef {
                id: "Sever Soul", name: "Sever Soul", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: crate::effects::declarative::CardFilter::NonAttacks,
                        action: crate::effects::declarative::BulkAction::Exhaust,
                    },
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sever Soul+", name: "Sever Soul+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 22, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: crate::effects::declarative::CardFilter::NonAttacks,
                        action: crate::effects::declarative::BulkAction::Exhaust,
                    },
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
}
