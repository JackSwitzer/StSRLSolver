use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Panache.java installs 10 PanachePower for zero energy; upgrade adds four
    // damage. PanachePower counts every fifth card (including its own play),
    // resets each turn, and deals source-less THORNS damage to all enemies.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Panache.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/PanachePower.java
    insert(cards, CardDef {
        id: "Panache", name: "Panache", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 10, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::PANACHE, A::Magic))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Panache+", name: "Panache+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 14, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::PANACHE, A::Magic))],
        complex_hook: None,
    });
}
