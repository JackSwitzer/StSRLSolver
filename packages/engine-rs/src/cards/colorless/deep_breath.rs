use crate::cards::prelude::*;

fn java_next_int(seed: &mut u64, bound: usize) -> usize {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1_u64 << 48) - 1;
    let mut next = |bits: u32| {
        *seed = seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
        (*seed >> (48 - bits)) as u32
    };
    if bound.is_power_of_two() {
        return ((bound as u64 * next(31) as u64) >> 31) as usize;
    }
    loop {
        let bits = next(31) as usize;
        let value = bits % bound;
        if bits - value + (bound - 1) < (1_usize << 31) {
            return value;
        }
    }
}

fn java_shuffle<T>(values: &mut [T], random_seed: u64) {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const MASK: u64 = (1_u64 << 48) - 1;
    let mut seed = (random_seed ^ MULTIPLIER) & MASK;
    for len in (2..=values.len()).rev() {
        let other = java_next_int(&mut seed, len);
        values.swap(len - 1, other);
    }
}

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

        let discard_seed = engine.rng.random_long();
        java_shuffle(&mut engine.state.discard_pile, discard_seed);
        engine
            .state
            .draw_pile
            .append(&mut engine.state.discard_pile);

        let draw_seed = engine.rng.random_long();
        java_shuffle(&mut engine.state.draw_pile, draw_seed);
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
