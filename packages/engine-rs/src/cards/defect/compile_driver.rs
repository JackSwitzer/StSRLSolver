use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/blue/CompileDriver.java queues 7 damage before
    // CompileDriverAction(1); upgradeDamage(3) leaves the draw multiplier unchanged.
    insert(
        cards,
        CardDef {
            id: "Compile Driver",
            name: "Compile Driver",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 7,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::UniqueOrbCount))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Compile Driver+",
            name: "Compile Driver+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 10,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::UniqueOrbCount))],
            complex_hook: None,
        },
    );
}
