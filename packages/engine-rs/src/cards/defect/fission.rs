use crate::cards::prelude::*;
use crate::effects::declarative::{Effect, SimpleEffect as SE};

static FISSION_EFFECTS: [Effect; 1] = [Effect::Simple(SE::ResolveFission { evoke: false })];
static FISSION_PLUS_EFFECTS: [Effect; 1] = [Effect::Simple(SE::ResolveFission { evoke: true })];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Fission: 0 cost, remove all orbs, gain energy+draw per orb, exhaust
    // (upgrade: evoke instead of remove)
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
