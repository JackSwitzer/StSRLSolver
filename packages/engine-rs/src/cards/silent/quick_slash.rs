use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Quick Slash queues DamageAction before DrawCardAction(1); upgrading adds
    // four damage only, taking the attack from eight to twelve.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/QuickSlash.java
    insert(cards, CardDef {
                id: "Quick Slash", name: "Quick Slash", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Quick Slash+", name: "Quick Slash+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}
