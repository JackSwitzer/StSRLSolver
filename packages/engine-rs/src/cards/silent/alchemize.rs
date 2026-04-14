use crate::cards::prelude::*;
use crate::effects::declarative::{Effect, SimpleEffect as SE};

static ALCHEMIZE_EFFECT: [Effect; 1] = [Effect::Simple(SE::ObtainRandomPotion)];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Alchemize ---- (cost 1, gain random potion, exhaust; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Alchemize", name: "Alchemize", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["alchemize"], effect_data: &ALCHEMIZE_EFFECT, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Alchemize+", name: "Alchemize+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["alchemize"], effect_data: &ALCHEMIZE_EFFECT, complex_hook: None,
            });
}
