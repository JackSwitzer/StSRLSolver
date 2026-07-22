use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/blue/ConserveBattery.java queues 7 Block then one
    // EnergizedBluePower; upgradeBlock(3) leaves Energized unchanged.
    insert(
        cards,
        CardDef {
            id: "Conserve Battery",
            name: "Conserve Battery",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 7,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::ENERGIZED_BLUE,
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Conserve Battery+",
            name: "Conserve Battery+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 10,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::ENERGIZED_BLUE,
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
}
