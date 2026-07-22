use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Hemokinesis.use queues LoseHPAction(2) before DamageAction. This order
    // lets top-queued HP-loss reactions such as Rupture's Strength resolve
    // before the hit. Upgrading adds 5 damage and changes nothing else.
    // Java: reference/extracted/methods/card/Hemokinesis.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RupturePower.java
    insert(
        cards,
        CardDef {
            id: "Hemokinesis",
            name: "Hemokinesis",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 15,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-2))),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Hemokinesis+",
            name: "Hemokinesis+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 20,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-2))),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ],
            complex_hook: None,
        },
    );
}
