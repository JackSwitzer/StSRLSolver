use crate::cards::prelude::*;

fn alchemize_hook(
    engine: &mut crate::engine::CombatEngine,
    _ctx: &crate::effects::types::CardPlayContext,
) {
    let _ = engine.obtain_random_potion();
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Alchemize ---- (cost 1, gain random potion, exhaust; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Alchemize", name: "Alchemize", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["alchemize"], effect_data: &[], complex_hook: Some(alchemize_hook),
            });
    insert(cards, CardDef {
                id: "Alchemize+", name: "Alchemize+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["alchemize"], effect_data: &[], complex_hook: Some(alchemize_hook),
            });
}
