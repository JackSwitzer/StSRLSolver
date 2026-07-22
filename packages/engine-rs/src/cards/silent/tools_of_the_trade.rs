use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ToolsOfTheTrade.java applies one stack of its power; upgradeBaseCost
    // changes cost from 1 to 0 without changing the stack amount.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/ToolsOfTheTrade.java
    insert(
        cards,
        CardDef {
            id: "Tools of the Trade",
            name: "Tools of the Trade",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::TOOLS_OF_THE_TRADE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Tools of the Trade+",
            name: "Tools of the Trade+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::TOOLS_OF_THE_TRADE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
