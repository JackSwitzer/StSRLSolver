use crate::cards::prelude::*;

fn wish_hook(engine: &mut crate::engine::CombatEngine, _ctx: &crate::effects::types::CardPlayContext) {
    let options = vec![
        crate::engine::ChoiceOption::Named("Strength"),
        crate::engine::ChoiceOption::Named("Gold"),
        crate::engine::ChoiceOption::Named("Plated Armor"),
    ];
    engine.begin_choice(crate::engine::ChoiceReason::PickOption, options, 1, 1);
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Wish ---- (cost 3, skill, exhaust, choose: +3 str, or 25 gold, or 6 block; upgrade: +1/+5/+2)
    insert(cards, CardDef {
                id: "Wish", name: "Wish", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["wish"], effect_data: &[], complex_hook: Some(wish_hook),
            });
    insert(cards, CardDef {
                id: "Wish+", name: "Wish+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effects: &["wish"], effect_data: &[], complex_hook: Some(wish_hook),
            });
}
