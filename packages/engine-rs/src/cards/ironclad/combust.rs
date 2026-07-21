use crate::cards::prelude::*;

fn record_combust_application(
    engine: &mut crate::engine::CombatEngine,
    _ctx: &crate::effects::types::CardPlayContext,
) {
    // CombustPower stores damage in amount and HP loss in a separate field.
    // Each subsequent ApplyPowerAction adds damage but increments HP loss by
    // exactly one, including replayed power applications.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CombustPower.java
    engine.rebuild_effect_runtime();
    let owner = crate::effects::runtime::EffectOwner::PlayerPower;
    let hp_loss = engine.hidden_effect_value("combust", owner, 0);
    let _ = engine.set_hidden_effect_value("combust", owner, 0, hp_loss + 1);
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Source: cards/red/Combust.java applies CombustPower(1, 5), with
        // upgradeMagicNumber(2).
    insert(cards, CardDef {
                id: "Combust", name: "Combust", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::COMBUST, A::Magic)),
                ], complex_hook: Some(record_combust_application),
            });
    insert(cards, CardDef {
                id: "Combust+", name: "Combust+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::COMBUST, A::Magic)),
                ], complex_hook: Some(record_combust_application),
            });
}
