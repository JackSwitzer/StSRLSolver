use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, GeneratedCostRule};

static INFERNAL_BLADE: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::Attack,
    count: AmountSource::Fixed(1),
    cost_rule: GeneratedCostRule::ZeroThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // InfernalBlade.use immediately selects one ATTACK from the current
    // character's source rarity pools through cardRandomRng, makes a base
    // copy, sets its turn cost to zero, then adds it to hand. Base Exhausts at
    // cost 1; upgrade changes only its cost to 0.
    // Java: reference/extracted/methods/card/InfernalBlade.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
    insert(cards, CardDef {
        id: "Infernal Blade",
        name: "Infernal Blade",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
                effect_data: &INFERNAL_BLADE,
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Infernal Blade+",
        name: "Infernal Blade+",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 0,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
                effect_data: &INFERNAL_BLADE,
        complex_hook: None,
    });
}
