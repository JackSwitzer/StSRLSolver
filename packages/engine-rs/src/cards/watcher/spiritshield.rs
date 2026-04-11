use crate::cards::prelude::*;

fn spirit_shield_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    let cards_in_hand = engine.state.hand.len() as i32;
    let per_card = ctx.card.base_magic.max(1);
    let dex = engine.state.player.dexterity();
    let frail = engine.state.player.is_frail();
    let block = crate::damage::calculate_block(per_card * cards_in_hand, dex, frail);
    engine.gain_block_player(block);
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Spirit Shield ---- (cost 2, skill, gain 3 block per card in hand; +1 magic upgrade)
    insert(cards, CardDef {
                id: "SpiritShield", name: "Spirit Shield", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["block_per_card_in_hand"], effect_data: &[], complex_hook: Some(spirit_shield_hook),
            });
    insert(cards, CardDef {
                id: "SpiritShield+", name: "Spirit Shield+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["block_per_card_in_hand"], effect_data: &[], complex_hook: Some(spirit_shield_hook),
            });
}
