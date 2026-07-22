use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Carnage.java
    // deals 20 damage, is Ethereal, and upgradeDamage(8) raises damage to 28.
    insert(
        cards,
        CardDef {
            id: "Carnage",
            name: "Carnage",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 20,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Carnage+",
            name: "Carnage+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 28,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
}
