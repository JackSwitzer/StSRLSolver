use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java deals 4 damage (6 upgraded) for zero energy. Every actual stance
    // change queues DiscardToHandAction for this card instance.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FlurryOfBlows.java
    // decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DiscardToHandAction.java
    insert(
        cards,
        CardDef {
            id: "FlurryOfBlows",
            name: "Flurry of Blows",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 4,
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
            id: "FlurryOfBlows+",
            name: "Flurry of Blows+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 6,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
}
