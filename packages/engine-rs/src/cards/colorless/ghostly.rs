use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Colorless Special ----
        // Apparition (Java ID: Ghostly) applies exactly one Intangible and
        // exhausts. Its upgrade removes Ethereal without changing any stats.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Apparition.java
    insert(cards, CardDef {
                id: "Ghostly", name: "Apparition", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::INTANGIBLE, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Ghostly+", name: "Apparition+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::INTANGIBLE, A::Fixed(1))),
                ], complex_hook: None,
            });
}
