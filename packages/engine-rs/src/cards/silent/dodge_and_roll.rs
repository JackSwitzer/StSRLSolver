use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // DodgeAndRoll.java uses `this.block` for both immediate Block and
    // NextTurnBlockPower, so Dexterity/Frail modify both amounts. Upgrade adds
    // 2 Block and there is no magic number.
    // Java: reference/extracted/methods/card/DodgeAndRoll.java
    insert(
        cards,
        CardDef {
            id: "Dodge and Roll",
            name: "Dodge and Roll",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 4,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::NEXT_TURN_BLOCK,
                A::ModifiedBlock,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Dodge and Roll+",
            name: "Dodge and Roll+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 6,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::NEXT_TURN_BLOCK,
                A::ModifiedBlock,
            ))],
            complex_hook: None,
        },
    );
}
