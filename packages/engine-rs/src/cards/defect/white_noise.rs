use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, GeneratedCostRule};

static WHITE_NOISE: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::DefectPower,
    count: AmountSource::Fixed(1),
    cost_rule: GeneratedCostRule::ZeroThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // WhiteNoise.use selects one non-healing Power from the Defect source
    // rarity pools through cardRandomRng, makes a copy free this turn, and
    // queues it into hand. The card Exhausts; upgrading only changes cost 1
    // to 0.
    // Java: reference/extracted/methods/card/WhiteNoise.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
    insert(cards, CardDef {
                id: "White Noise", name: "White Noise", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &WHITE_NOISE, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "White Noise+", name: "White Noise+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &WHITE_NOISE, complex_hook: None,
            });
}
