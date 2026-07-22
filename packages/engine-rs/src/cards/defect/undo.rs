use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Equilibrium grants Block and a stackable turn-based power. Each power
    // stack retains non-Ethereal cards for one round before decrementing.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Equilibrium.java
    // and powers/EquilibriumPower.java.
    insert(
        cards,
        CardDef {
            id: "Undo",
            name: "Equilibrium",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 13,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::AddStatus(T::Player, sid::EQUILIBRIUM, A::Magic)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Undo+",
            name: "Equilibrium+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 16,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::AddStatus(T::Player, sid::EQUILIBRIUM, A::Magic)),
            ],
            complex_hook: None,
        },
    );
}
