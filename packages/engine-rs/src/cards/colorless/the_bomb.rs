use crate::cards::prelude::*;

fn install_independent_bomb(
    engine: &mut crate::engine::CombatEngine,
    ctx: &crate::effects::types::CardPlayContext,
) {
    // TheBombPower appends a global offset to its ID, which prevents
    // ApplyPowerAction from stacking separate applications together.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/TheBombPower.java
    engine.schedule_the_bomb(3, ctx.card.base_magic);
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // TheBomb.java applies a fresh three-turn power for 40 damage; upgrade
    // adds 10 damage without changing cost or countdown.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/TheBomb.java
    insert(
        cards,
        CardDef {
            id: "The Bomb",
            name: "The Bomb",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 40,
            exhaust: false,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(install_independent_bomb),
        },
    );
    insert(
        cards,
        CardDef {
            id: "The Bomb+",
            name: "The Bomb+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 50,
            exhaust: false,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(install_independent_bomb),
        },
    );
}
