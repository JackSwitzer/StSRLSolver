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
        // Impatience: 0 cost, draw 2 if no attacks in hand
    insert(cards, CardDef {
                id: "Impatience", name: "Impatience", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["draw_if_no_attacks"], effect_data: &IMPATIENCE_BASE, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Impatience+", name: "Impatience+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["draw_if_no_attacks"], effect_data: &IMPATIENCE_PLUS, complex_hook: None,
            });
}
