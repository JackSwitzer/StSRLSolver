use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Immolate queues 21 AoE damage before adding one Burn to discard. A
    // final lethal DamageAllEnemiesAction clears the queued temp-card action;
    // upgrading adds 7 damage and changes nothing else.
    // Java: reference/extracted/methods/card/Immolate.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAllEnemiesAction.java
    insert(
        cards,
        CardDef {
            id: "Immolate",
            name: "Immolate",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 2,
            base_damage: 21,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Immolate+",
            name: "Immolate+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 2,
            base_damage: 28,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
            ],
            complex_hook: None,
        },
    );
}
