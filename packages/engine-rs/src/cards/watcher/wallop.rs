use crate::cards::prelude::*;

fn wallop_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    if ctx.total_unblocked_damage > 0 {
        engine.gain_block_player(ctx.total_unblocked_damage);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Wallop ---- (cost 2, 9 dmg, gain block equal to unblocked damage; +3 dmg upgrade)
    insert(cards, CardDef {
                id: "Wallop", name: "Wallop", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["block_from_damage"], effect_data: &[], complex_hook: Some(wallop_hook),
            });
    insert(cards, CardDef {
                id: "Wallop+", name: "Wallop+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["block_from_damage"], effect_data: &[], complex_hook: Some(wallop_hook),
            });
}
