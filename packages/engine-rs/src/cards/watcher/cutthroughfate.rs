use crate::cards::prelude::*;

fn defer_draw_until_scry_resolves(
    engine: &mut crate::engine::CombatEngine,
    _ctx: &crate::effects::types::CardPlayContext,
) {
    if let Some(choice) = engine.choice.as_mut() {
        if choice.reason == crate::engine::ChoiceReason::Scry {
            choice.post_choice_draw = 1;
        }
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java queues DamageAction, ScryAction(magicNumber), then DrawCardAction(1).
    // The draw amount is never magicNumber and must wait for the Scry choice.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/CutThroughFate.java
    insert(cards, CardDef {
                id: "CutThroughFate", name: "Cut Through Fate", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::Scry(A::Magic)),
                    E::Simple(SE::DrawCards(A::Fixed(1))),
                ], complex_hook: Some(defer_draw_until_scry_resolves),
            });
    insert(cards, CardDef {
                id: "CutThroughFate+", name: "Cut Through Fate+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::Scry(A::Magic)),
                    E::Simple(SE::DrawCards(A::Fixed(1))),
                ], complex_hook: Some(defer_draw_until_scry_resolves),
            });
}
