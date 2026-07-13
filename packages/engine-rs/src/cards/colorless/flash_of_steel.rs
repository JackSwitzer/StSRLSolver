use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // FlashOfSteel.java deals card damage, then queues a literal one-card
        // draw; the card has no magicNumber. Upgrading adds 3 damage only.
        // Java: reference/extracted/methods/card/FlashOfSteel.java
    insert(cards, CardDef {
                id: "Flash of Steel", name: "Flash of Steel", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Fixed(1)))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Flash of Steel+", name: "Flash of Steel+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawCards(A::Fixed(1)))], complex_hook: None,
            });
}
