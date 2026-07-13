use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Miracle.java: zero-cost, target NONE, Exhaust and selfRetain; use() gains
    // one energy, or two when upgraded. upgrade() changes no numeric base field.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Miracle.java
    insert(cards, CardDef {
                id: "Miracle", name: "Miracle", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainEnergy(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Miracle+", name: "Miracle+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainEnergy(A::Magic)),
                ], complex_hook: None,
            });
}
