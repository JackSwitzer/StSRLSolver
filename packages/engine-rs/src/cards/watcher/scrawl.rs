use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};

static SCRAWL: [E; 1] = [E::Simple(SE::DrawCards(A::Fixed(10)))];
static SCRAWL_PLUS: [E; 1] = [E::Simple(SE::DrawCards(A::Fixed(10)))];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Scrawl ---- (cost 1, skill, exhaust, draw until you have 10 cards; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Scrawl", name: "Scrawl", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["draw_to_ten"], effect_data: &SCRAWL, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Scrawl+", name: "Scrawl+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["draw_to_ten"], effect_data: &SCRAWL_PLUS, complex_hook: None,
            });
}
