use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // DamagePerAttackPlayedAction counts Attacks in cardsPlayedThisTurn, then
    // subtracts the current Finisher; playing it first therefore deals no hit.
    // Java: reference/extracted/methods/card/Finisher.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DamagePerAttackPlayedAction.java
    insert(cards, CardDef {
                id: "Finisher", name: "Finisher", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::ExtraHits(A::PriorAttacksThisTurn)], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Finisher+", name: "Finisher+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::ExtraHits(A::PriorAttacksThisTurn)], complex_hook: None,
            });
}
