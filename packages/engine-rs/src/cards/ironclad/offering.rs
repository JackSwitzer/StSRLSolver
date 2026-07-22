use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Offering.java queues LoseHPAction(6), GainEnergyAction(2), then draws
    // magic 3. Upgrade adds two draw only; the fixed HP loss stays six.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Offering.java
    insert(
        cards,
        CardDef {
            id: "Offering",
            name: "Offering",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: true,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-6))),
                E::Simple(SE::GainEnergy(A::Fixed(2))),
                E::Simple(SE::DrawCards(A::Magic)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Offering+",
            name: "Offering+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 5,
            exhaust: true,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::ModifyHp(A::Fixed(-6))),
                E::Simple(SE::GainEnergy(A::Fixed(2))),
                E::Simple(SE::DrawCards(A::Magic)),
            ],
            complex_hook: None,
        },
    );
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_ironclad_wave7.rs"]
mod test_card_runtime_ironclad_wave7;
