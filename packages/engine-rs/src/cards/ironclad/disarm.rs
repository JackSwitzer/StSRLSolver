use crate::cards::prelude::*;

fn apply_permanent_strength_loss(
    engine: &mut crate::engine::CombatEngine,
    ctx: &crate::effects::types::CardPlayContext,
) {
    let Ok(enemy_idx) = usize::try_from(ctx.target_idx) else {
        return;
    };
    if enemy_idx < engine.state.enemies.len() {
        // Negative StrengthPower is a debuff, so ApplyPowerAction lets Artifact
        // block it. Disarm queues no GainStrengthPower restoration.
        // Source: reference/extracted/methods/card/Disarm.java
        engine.apply_player_debuff_to_enemy(enemy_idx, sid::STRENGTH, -ctx.card.base_magic);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Disarm costs 1, permanently applies -2 Strength, and exhausts; upgrading
    // increases the loss to -3.
    insert(
        cards,
        CardDef {
            id: "Disarm",
            name: "Disarm",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: true,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(apply_permanent_strength_loss),
        },
    );
    insert(
        cards,
        CardDef {
            id: "Disarm+",
            name: "Disarm+",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: true,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(apply_permanent_strength_loss),
        },
    );
}
