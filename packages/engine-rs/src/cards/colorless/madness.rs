use crate::cards::prelude::*;
use crate::effects::declarative::{Effect, SimpleEffect as SE};

static MADNESS_EFFECT: [Effect; 1] = [Effect::Simple(SE::SetRandomHandCardCost(0))];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Madness.java queues MadnessAction and exhausts. MadnessAction permanently
    // sets one sampled hand card to zero; the upgrade changes only base cost 1 -> 0.
    insert(cards, CardDef {
                id: "Madness", name: "Madness", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &MADNESS_EFFECT, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Madness+", name: "Madness+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &MADNESS_EFFECT, complex_hook: None,
            });
}
