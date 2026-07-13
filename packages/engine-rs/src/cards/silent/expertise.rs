use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ExpertiseAction calculates the shortfall after Expertise leaves hand,
    // then queues an ordinary DrawCardAction for exactly that many cards.
    // Java: reference/extracted/methods/card/Expertise.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ExpertiseAction.java
    insert(cards, CardDef {
                id: "Expertise", name: "Expertise", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 6, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawToHandSize(A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Expertise+", name: "Expertise+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DrawToHandSize(A::Magic))], complex_hook: None,
            });
}
