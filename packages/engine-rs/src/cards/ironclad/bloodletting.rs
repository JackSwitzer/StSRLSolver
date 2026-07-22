use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/red/Bloodletting.java costs 0 and queues LoseHPAction(3)
    // before gaining 2 energy; upgrading raises only the energy to 3.
    insert(
        cards,
        CardDef {
            id: "Bloodletting",
            name: "Bloodletting",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-3))),
                E::Simple(SE::GainEnergy(A::Magic)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Bloodletting+",
            name: "Bloodletting+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-3))),
                E::Simple(SE::GainEnergy(A::Magic)),
            ],
            complex_hook: None,
        },
    );
}
