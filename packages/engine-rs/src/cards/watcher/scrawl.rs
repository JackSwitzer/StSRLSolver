use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};

static SCRAWL: [E; 1] = [E::Simple(SE::DrawCards(A::Fixed(10)))];
static SCRAWL_PLUS: [E; 1] = [E::Simple(SE::DrawCards(A::Fixed(10)))];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ExpertiseAction draws only the number needed to reach hand size 10;
        // Scrawl exhausts and its upgrade changes only cost 1 -> 0.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Scrawl.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExpertiseAction.java
    insert(cards, CardDef {
                id: "Scrawl", name: "Scrawl", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &SCRAWL, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Scrawl+", name: "Scrawl+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &SCRAWL_PLUS, complex_hook: None,
            });
}
