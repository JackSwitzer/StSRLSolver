use crate::cards::prelude::*;

fn concentrate_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    let num_discard = ctx.card.base_magic.max(1) as usize;
    let options: Vec<crate::engine::ChoiceOption> = engine.state.hand.iter()
        .enumerate()
        .map(|(i, _)| crate::engine::ChoiceOption::HandCard(i))
        .collect();
    if options.len() >= num_discard {
        engine.begin_choice(crate::engine::ChoiceReason::DiscardForEffect, options, num_discard, num_discard);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Concentrate", name: "Concentrate", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["discard_gain_energy"], effect_data: &[], complex_hook: Some(concentrate_hook),
            });
    insert(cards, CardDef {
                id: "Concentrate+", name: "Concentrate+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["discard_gain_energy"], effect_data: &[], complex_hook: Some(concentrate_hook),
            });
}
