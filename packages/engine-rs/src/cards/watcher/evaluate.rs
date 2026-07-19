use crate::cards::prelude::*;

fn add_insight_to_random_non_top_draw_position(
    engine: &mut crate::engine::CombatEngine,
    _ctx: &crate::effects::types::CardPlayContext,
) {
    let insight = engine.temp_card("Insight");
    let len = engine.state.draw_pile.len();
    if len == 0 {
        engine.state.draw_pile.push(insight);
    } else {
        // CardGroup.addToRandomSpot uses cardRandomRng.random(size - 1), so
        // it consumes one tick and never inserts above the existing top card.
        let index = engine.card_random_rng.random_int((len - 1) as i32) as usize;
        engine.state.draw_pile.insert(index, insight);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Java gains block, then inserts one Insight at a random non-top draw
        // pile position. upgradeBlock(4) changes 6 -> 10.
        // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Evaluate.java
        // decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
        // ---- Common: Evaluate ---- (cost 1, 6 block, add Insight to draw; +4 block upgrade)
    insert(cards, CardDef {
                id: "Evaluate", name: "Evaluate", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                ], complex_hook: Some(add_insight_to_random_non_top_draw_position),
            });
    insert(cards, CardDef {
                id: "Evaluate+", name: "Evaluate+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                ], complex_hook: Some(add_insight_to_random_non_top_draw_position),
            });
}
