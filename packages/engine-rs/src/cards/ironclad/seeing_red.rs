use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/SeeingRed.java
    // GainEnergyAction grants two; upgrading changes only the base cost to zero.
    insert(
        cards,
        CardDef {
            id: "Seeing Red",
            name: "Seeing Red",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainEnergy(A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Seeing Red+",
            name: "Seeing Red+",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainEnergy(A::Magic))],
            complex_hook: None,
        },
    );
}
