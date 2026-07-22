use crate::cards::prelude::*;
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};

static MIND_BLAST: [E; 1] = [E::Simple(SE::DealDamage(T::SelectedEnemy, A::DrawPileSize))];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // MindBlast.java sets baseDamage from the live draw-pile size in applyPowers,
    // is Innate, and upgrade() changes only the base cost from 2 to 1.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/MindBlast.java
    insert(
        cards,
        CardDef {
            id: "Mind Blast",
            name: "Mind Blast",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 0,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &MIND_BLAST,
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Mind Blast+",
            name: "Mind Blast+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 0,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &MIND_BLAST,
            complex_hook: None,
        },
    );
}
