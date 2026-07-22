use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Vigilance.java
    // Generic base-block resolution precedes the declared Calm stance change.
    insert(
        cards,
        CardDef {
            id: "Vigilance",
            name: "Vigilance",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 8,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::ChangeStance(Stance::Calm))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Vigilance+",
            name: "Vigilance+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 12,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::ChangeStance(Stance::Calm))],
            complex_hook: None,
        },
    );
}
