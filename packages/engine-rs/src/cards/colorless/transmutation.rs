use crate::cards::prelude::*;

fn transmutation_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    let count = ctx.x_value;
    for _ in 0..count {
        if engine.state.hand.len() >= 10 { break; }
        let card = engine.temp_card("Smite");
        engine.state.hand.push(card);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Transmutation", name: "Transmutation", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["transmutation"], effect_data: &[], complex_hook: Some(transmutation_hook),
            });
    insert(cards, CardDef {
                id: "Transmutation+", name: "Transmutation+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["transmutation"], effect_data: &[], complex_hook: Some(transmutation_hook),
            });
}
