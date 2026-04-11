use crate::cards::prelude::*;

fn foreign_influence_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    // Generate 3 random attacks from other classes for the player to choose
    let is_upgraded = ctx.card.id.ends_with('+');
    let attacks: &[&str] = if is_upgraded {
        &["Bash+", "Carnage+", "Headbutt+"]
    } else {
        &["Bash", "Carnage", "Headbutt"]
    };
    let options: Vec<crate::engine::ChoiceOption> = attacks.iter()
        .map(|name| crate::engine::ChoiceOption::GeneratedCard(engine.temp_card(name)))
        .collect();
    if !options.is_empty() {
        engine.begin_choice(crate::engine::ChoiceReason::DiscoverCard, options, 1, 1);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Foreign Influence ---- (cost 0, skill, exhaust, choose attack from other class; upgrade: upgraded choices)
    insert(cards, CardDef {
                id: "ForeignInfluence", name: "Foreign Influence", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["foreign_influence"], effect_data: &[], complex_hook: Some(foreign_influence_hook),
            });
    insert(cards, CardDef {
                id: "ForeignInfluence+", name: "Foreign Influence+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["foreign_influence"], effect_data: &[], complex_hook: Some(foreign_influence_hook),
            });
}
