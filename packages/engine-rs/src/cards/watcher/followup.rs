use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FollowUp.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/FollowUpAction.java
// FollowUpAction inspects cardsPlayedThisCombat[size - 2], because Follow-Up
// itself is already the last entry when the queued action resolves.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "FollowUp", name: "Follow-Up", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(Cond::LastCardType(CardType::Attack), &[E::Simple(SE::GainEnergy(A::Fixed(1)))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "FollowUp+", name: "Follow-Up+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(Cond::LastCardType(CardType::Attack), &[E::Simple(SE::GainEnergy(A::Fixed(1)))], &[]),
                ], complex_hook: None,
            });
}
