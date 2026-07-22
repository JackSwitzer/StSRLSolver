use crate::cards::prelude::*;

fn apply_temporary_strength_loss_to_all(
    engine: &mut crate::engine::CombatEngine,
    ctx: &crate::effects::types::CardPlayContext,
) {
    // PiercingWail.java queues negative StrengthPower for every monster, then
    // queues GainStrengthPower only for monsters that did not have Artifact.
    // A successful first application proves the source pre-check was clear;
    // TEMP_STRENGTH_LOSS models GainStrengthPower's post-monster-turn restore.
    // Source: reference/extracted/methods/card/PiercingWail.java
    let amount = ctx.card.base_magic;
    for enemy_idx in engine.state.living_enemy_indices() {
        if engine.apply_player_debuff_to_enemy(enemy_idx, sid::STRENGTH, -amount) {
            engine.state.enemies[enemy_idx]
                .entity
                .add_status(sid::TEMP_STRENGTH_LOSS, amount);
        }
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Common: Piercing Wail ---- (cost 1, -6 str to all enemies this turn, exhaust; +2 magic)
    insert(
        cards,
        CardDef {
            id: "Piercing Wail",
            name: "Piercing Wail",
            card_type: CardType::Skill,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 6,
            exhaust: true,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(apply_temporary_strength_loss_to_all),
        },
    );
    insert(
        cards,
        CardDef {
            id: "Piercing Wail+",
            name: "Piercing Wail+",
            card_type: CardType::Skill,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 8,
            exhaust: true,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(apply_temporary_strength_loss_to_all),
        },
    );
}
