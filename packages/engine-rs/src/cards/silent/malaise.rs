use crate::cards::prelude::*;

fn apply_malaise(
    engine: &mut crate::engine::CombatEngine,
    ctx: &crate::effects::types::CardPlayContext,
) {
    let Ok(enemy_idx) = usize::try_from(ctx.target_idx) else {
        return;
    };
    let amount = ctx.x_value + ctx.card.base_magic.max(0);
    if amount <= 0 || enemy_idx >= engine.state.enemies.len() {
        return;
    }

    // MalaiseAction.java gates both actions on effect > 0, then queues negative
    // StrengthPower before WeakPower. They are independent debuffs, so one
    // Artifact charge blocks the Strength loss and still allows Weak to land.
    engine.apply_player_debuff_to_enemy(enemy_idx, sid::STRENGTH, -amount);
    engine.apply_player_debuff_to_enemy(enemy_idx, sid::WEAKENED, amount);
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Malaise.java passes energyOnUse/freeToPlayOnce to MalaiseAction; upgrade
    // adds one to the action effect without changing this X cost or Exhaust.
    insert(cards, CardDef {
                id: "Malaise", name: "Malaise", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 0, exhaust: true, enter_stance: None,
                effect_data: &[], complex_hook: Some(apply_malaise),
            });
    insert(cards, CardDef {
                id: "Malaise+", name: "Malaise+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &[], complex_hook: Some(apply_malaise),
            });
}
