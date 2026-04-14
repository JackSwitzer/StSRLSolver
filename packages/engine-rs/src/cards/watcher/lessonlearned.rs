use crate::cards::prelude::*;

fn lesson_learned_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    if ctx.enemy_killed {
        let mut upgradeable: Vec<(bool, usize)> = Vec::new();
        for (i, c) in engine.state.draw_pile.iter().enumerate() {
            if !c.is_upgraded() { upgradeable.push((true, i)); }
        }
        for (i, c) in engine.state.discard_pile.iter().enumerate() {
            if !c.is_upgraded() { upgradeable.push((false, i)); }
        }
        if !upgradeable.is_empty() {
            let pick = engine.rng_gen_range(0..upgradeable.len());
            let (is_draw, idx) = upgradeable[pick];
            if is_draw {
                engine.card_registry.upgrade_card(&mut engine.state.draw_pile[idx]);
            } else {
                engine.card_registry.upgrade_card(&mut engine.state.discard_pile[idx]);
            }
        }
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Lesson Learned ---- (cost 2, 10 dmg, exhaust, if kill upgrade a random card; +3 dmg upgrade)
    insert(cards, CardDef {
                id: "LessonLearned", name: "Lesson Learned", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["lesson_learned"], effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: Some(lesson_learned_hook),
            });
    insert(cards, CardDef {
                id: "LessonLearned+", name: "Lesson Learned+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["lesson_learned"], effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: Some(lesson_learned_hook),
            });
}
