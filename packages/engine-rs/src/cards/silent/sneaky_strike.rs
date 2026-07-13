use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // SneakyStrike.java (ledger ID "Underhanded Strike") queues 12 damage then
    // a two-energy refund if any card was discarded this turn, carries STRIKE,
    // and upgradeDamage(4) changes only damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/SneakyStrike.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GainEnergyIfDiscardAction.java
    insert(cards, CardDef {
                id: "Sneaky Strike", name: "Sneaky Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::DiscardedThisTurn, &[E::Simple(SE::GainEnergy(A::Fixed(2)))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sneaky Strike+", name: "Sneaky Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::DiscardedThisTurn, &[E::Simple(SE::GainEnergy(A::Fixed(2)))], &[]),
                ], complex_hook: None,
            });
}
