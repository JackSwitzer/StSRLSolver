use crate::cards::prelude::*;

fn defer_block_and_damage_until_scry_resolves(
    engine: &mut crate::engine::CombatEngine,
    ctx: &crate::effects::types::CardPlayContext,
) {
    if let Some(choice) = engine.choice.as_mut() {
        if choice.reason == crate::engine::ChoiceReason::Scry {
            choice.deferred_scry_card_effects = Some(crate::engine::DeferredScryCardEffects {
                card_inst: ctx.card_inst,
                target_idx: ctx.target_idx,
                x_value: ctx.x_value,
                pen_nib_active: ctx.pen_nib_active,
                vigor: ctx.vigor,
                hand_size_at_play: ctx.hand_size_at_play,
                gain_block: true,
                deal_damage: true,
            });
        }
    }
}

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/JustLucky.java
// Java queues ScryAction before GainBlockAction and DamageAction, so neither
// block nor damage resolves while the Scry choice is still open.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Common: Just Lucky ---- (cost 0, 3 dmg, 2 block, scry 1; +1/+1/+1 upgrade)
    insert(
        cards,
        CardDef {
            id: "JustLucky",
            name: "Just Lucky",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 3,
            base_block: 2,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::Scry(A::Magic)),
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ],
            complex_hook: Some(defer_block_and_damage_until_scry_resolves),
        },
    );
    insert(
        cards,
        CardDef {
            id: "JustLucky+",
            name: "Just Lucky+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 4,
            base_block: 3,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::Scry(A::Magic)),
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ],
            complex_hook: Some(defer_block_and_damage_until_scry_resolves),
        },
    );
}
