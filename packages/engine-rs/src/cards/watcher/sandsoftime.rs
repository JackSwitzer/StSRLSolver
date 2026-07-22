use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // selfRetain calls onRetained every turn; SandsOfTime.onRetained uses
    // modifyCostForCombat(-1), so the per-copy discount accumulates.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SandsOfTime.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
    insert(
        cards,
        CardDef {
            id: "SandsOfTime",
            name: "Sands of Time",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 4,
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
            id: "SandsOfTime+",
            name: "Sands of Time+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 4,
            base_damage: 26,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
}
