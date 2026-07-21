use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Source: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Clothesline.java
        // queues 12 damage before applying 2 Weak; the upgrade adds 2 damage and 1 Weak.
    insert(cards, CardDef {
                id: "Clothesline", name: "Clothesline", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Clothesline+", name: "Clothesline+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
}
