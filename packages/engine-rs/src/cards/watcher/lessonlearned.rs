use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Lesson Learned ---- (cost 2, 10 dmg, exhaust, if kill upgrade a random card; +3 dmg upgrade)
    insert(cards, CardDef {
                id: "LessonLearned", name: "Lesson Learned", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["lesson_learned"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "LessonLearned+", name: "Lesson Learned+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["lesson_learned"], effect_data: &[], complex_hook: None,
            });
}
