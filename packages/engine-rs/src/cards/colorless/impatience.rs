use crate::cards::prelude::*;
use crate::cards::CardType;
use crate::effects::declarative::{AmountSource as A, Condition, Effect, SimpleEffect as SE};

static IMPATIENCE_BASE: [Effect; 1] = [Effect::Conditional(
    Condition::HandContainsType(CardType::Attack),
    &[],
    &[Effect::Simple(SE::DrawCards(A::Magic))],
)];

static IMPATIENCE_PLUS: [Effect; 1] = [Effect::Conditional(
    Condition::HandContainsType(CardType::Attack),
    &[],
    &[Effect::Simple(SE::DrawCards(A::Magic))],
)];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ConditionalDrawAction scans the hand after Impatience has left it and
    // draws only when no Attack remains. Base draws 2; upgrade draws 3 and
    // changes neither the 0 cost nor any other property.
    // Java: reference/extracted/methods/card/Impatience.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ConditionalDrawAction.java
    insert(
        cards,
        CardDef {
            id: "Impatience",
            name: "Impatience",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &IMPATIENCE_BASE,
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Impatience+",
            name: "Impatience+",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &IMPATIENCE_PLUS,
            complex_hook: None,
        },
    );
}
