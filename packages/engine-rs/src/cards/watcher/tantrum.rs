use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tantrum.java
    // The hits resolve before Wrath, then UseCardAction inserts this card at a
    // cardRandomRng-selected draw-pile position because shuffleBackIntoDrawPile is true.
    insert(cards, CardDef {
                id: "Tantrum", name: "Tantrum", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChangeStance(Stance::Wrath)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Tantrum+", name: "Tantrum+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChangeStance(Stance::Wrath)),
                ], complex_hook: None,
            });
}
