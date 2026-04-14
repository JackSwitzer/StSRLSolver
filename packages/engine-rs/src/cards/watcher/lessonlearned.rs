use crate::cards::prelude::*;
use crate::effects::declarative::{
    AmountSource as A, Condition as Cond, Effect as E, Pile as P, SimpleEffect as SE,
    Target as T,
};

static LESSON_LEARNED_UPGRADE_PILES: [P; 2] = [P::Draw, P::Discard];
static LESSON_LEARNED_KILL_BRANCH: [E; 1] = [E::Simple(SE::UpgradeRandomCardFromPiles(
    &LESSON_LEARNED_UPGRADE_PILES,
))];
static LESSON_LEARNED_EFFECTS: [E; 2] = [
    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
    E::Conditional(Cond::EnemyKilled, &LESSON_LEARNED_KILL_BRANCH, &[]),
];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Lesson Learned ---- (cost 2, 10 dmg, exhaust, if kill upgrade a random card; +3 dmg upgrade)
    insert(cards, CardDef {
                id: "LessonLearned", name: "Lesson Learned", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["lesson_learned"], effect_data: &LESSON_LEARNED_EFFECTS, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "LessonLearned+", name: "Lesson Learned+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["lesson_learned"], effect_data: &LESSON_LEARNED_EFFECTS, complex_hook: None,
            });
}
