use crate::cards::prelude::*;
use crate::effects::declarative::{Effect, SimpleEffect as SE};

static FISSION_EFFECTS: [Effect; 1] = [Effect::Simple(SE::ResolveFission { evoke: false })];
static FISSION_PLUS_EFFECTS: [Effect; 1] = [Effect::Simple(SE::ResolveFission { evoke: true })];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Fission.java delegates to FissionAction(upgraded); the card costs zero,
    // exhausts, and its upgrade changes only the action's orb handling.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Fission.java
    insert(cards, CardDef {
        id: "Fission", name: "Fission", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &FISSION_EFFECTS,
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Fission+", name: "Fission+", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &FISSION_PLUS_EFFECTS,
        complex_hook: None,
    });
}
