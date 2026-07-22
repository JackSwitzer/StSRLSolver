use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // DoubleEnergyAction.java gains the current EnergyPanel.totalCount, so the
    // base card doubles energy remaining after paying 1; upgrade costs 0.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/DoubleEnergyAction.java
    insert(
        cards,
        CardDef {
            id: "Double Energy",
            name: "Double Energy",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DoubleEnergy)],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Double Energy+",
            name: "Double Energy+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DoubleEnergy)],
            complex_hook: None,
        },
    );
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_defect_wave5.rs"]
mod test_card_runtime_defect_wave5;
