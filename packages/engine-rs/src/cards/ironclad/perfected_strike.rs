use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // PerfectedStrike.java deals 6 plus magic 2 for every STRIKE-tagged card
    // in hand, draw, and discard; upgrade adds one magic only.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/PerfectedStrike.java
    insert(cards, CardDef {
        id: "Perfected Strike", name: "Perfected Strike", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 2, base_damage: 6, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Perfected Strike+", name: "Perfected Strike+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 2, base_damage: 6, base_block: -1,
        base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: None,
    });
}
