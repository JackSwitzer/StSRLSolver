use crate::cards::prelude::*;

fn shuffle_discard_twice_then_draw(
    engine: &mut crate::engine::CombatEngine,
    ctx: &crate::effects::types::CardPlayContext,
) {
    // DeepBreath.java enters this branch only with a nonempty discard pile.
    // EmptyDeckShuffleAction first shuffles that pile with one shuffleRng
    // randomLong; ShuffleAction then shuffles the combined draw pile with a
    // second randomLong. Its constructor fires onShuffle exactly once.
    // Java: reference/extracted/methods/card/DeepBreath.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/EmptyDeckShuffleAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    if !engine.state.discard_pile.is_empty() {
        engine.emit_event(crate::effects::runtime::GameEvent::empty(
            crate::effects::trigger::Trigger::OnShuffle,
        ));

        crate::seed::card_group_shuffle(&mut engine.state.discard_pile, &mut engine.rng);
        engine
            .state
            .draw_pile
            .append(&mut engine.state.discard_pile);

        crate::seed::card_group_shuffle(&mut engine.state.draw_pile, &mut engine.rng);
    }
    engine.draw_cards(ctx.card.base_magic);
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/DeepBreath.java costs 0, shuffles
    // only when discard is nonempty, then draws 1 (2 upgraded).
    insert(cards, CardDef {
                id: "Deep Breath", name: "Deep Breath", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: Some(shuffle_discard_twice_then_draw),
            });
    insert(cards, CardDef {
                id: "Deep Breath+", name: "Deep Breath+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: Some(shuffle_discard_twice_then_draw),
            });
}
