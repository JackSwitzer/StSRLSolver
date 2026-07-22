use crate::cards::prelude::*;
use crate::effects::declarative::{Effect, SimpleEffect as SE};

static ALCHEMIZE_EFFECT: [Effect; 1] = [Effect::Simple(SE::ObtainRandomPotion)];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Alchemize obtains returnRandomPotion(true), exhausts, and carries the
    // source SELF target. Upgrading changes only its cost from one to zero.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
    insert(
        cards,
        CardDef {
            id: "Alchemize",
            name: "Alchemize",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &ALCHEMIZE_EFFECT,
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Alchemize+",
            name: "Alchemize+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &ALCHEMIZE_EFFECT,
            complex_hook: None,
        },
    );
}
