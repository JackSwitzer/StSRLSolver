use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // MulticastAction evokes the same front orb X times without removing it,
    // then removes it on the final evoke; Chemical X adds 2 and upgrade adds 1.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/MulticastAction.java
    insert(cards, CardDef {
                id: "Multi-Cast", name: "Multi-Cast", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::EvokeOrb(A::XCost)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Multi-Cast+", name: "Multi-Cast+", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::EvokeOrb(A::MagicPlusX)),
                ], complex_hook: None,
            });
}
