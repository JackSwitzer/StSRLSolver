use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // GashAction mutates the played Claw plus Claws currently in hand, draw,
    // and discard; it does not establish a combat-wide modifier.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/GashAction.java
    insert(
        cards,
        CardDef {
            id: "Gash",
            name: "Claw",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 3,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::IncreaseAllClawDamage(A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Gash+",
            name: "Claw+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 5,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::IncreaseAllClawDamage(A::Magic))],
            complex_hook: None,
        },
    );
}
