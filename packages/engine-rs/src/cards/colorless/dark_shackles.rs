use crate::cards::prelude::*;

fn apply_temporary_strength_loss(
    engine: &mut crate::engine::CombatEngine,
    ctx: &crate::effects::types::CardPlayContext,
) {
    let Ok(enemy_idx) = usize::try_from(ctx.target_idx) else {
        return;
    };
    if enemy_idx >= engine.state.enemies.len() {
        return;
    }

    // DarkShackles.java first queues negative StrengthPower, then queues the
    // matching GainStrengthPower only when the target did not have Artifact.
    // TEMP_STRENGTH_LOSS is the engine's end-of-turn restoration counter.
    // Source: reference/extracted/methods/card/DarkShackles.java
    let amount = ctx.card.base_magic;
    if engine.apply_player_debuff_to_enemy(enemy_idx, sid::STRENGTH, -amount) {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status(sid::TEMP_STRENGTH_LOSS, amount);
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Dark Shackles costs 0, temporarily removes 9 Strength, and exhausts;
    // upgrading adds 6 to both the loss and its restoration.
    insert(
        cards,
        CardDef {
            id: "Dark Shackles",
            name: "Dark Shackles",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 9,
            exhaust: true,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(apply_temporary_strength_loss),
        },
    );
    insert(
        cards,
        CardDef {
            id: "Dark Shackles+",
            name: "Dark Shackles+",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 15,
            exhaust: true,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(apply_temporary_strength_loss),
        },
    );
}
