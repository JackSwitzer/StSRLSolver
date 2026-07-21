use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // use applies one StormPower stack. The upgrade changes only isInnate;
    // magicNumber remains one.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Storm.java
    insert(cards, CardDef {
                id: "Storm", name: "Storm", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::STORM, A::Magic))],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Storm+", name: "Storm+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::STORM, A::Magic))],
                complex_hook: None,
            });
}
