use crate::cards::prelude::*;

fn infernal_blade_hook(engine: &mut crate::engine::CombatEngine, _ctx: &crate::effects::types::CardPlayContext) {
    if engine.state.hand.len() < 10 {
        let card = engine.temp_card("Bash");
        engine.state.hand.push(card);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Infernal Blade", name: "Infernal Blade", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["random_attack_to_hand"], effect_data: &[], complex_hook: Some(infernal_blade_hook),
            });
    insert(cards, CardDef {
                id: "Infernal Blade+", name: "Infernal Blade+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["random_attack_to_hand"], effect_data: &[], complex_hook: Some(infernal_blade_hook),
            });
}
