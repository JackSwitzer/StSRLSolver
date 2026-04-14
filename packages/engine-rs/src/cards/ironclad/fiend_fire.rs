use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, BulkAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Rare: Fiend Fire ----
    // Typed exhaust-first, then typed multi-hit damage. The hand exhaust runs
    // through the declarative interpreter before the damage effect resolves.
    insert(cards, CardDef {
        id: "Fiend Fire",
        name: "Fiend Fire",
        card_type: CardType::Attack,
        target: CardTarget::Enemy,
        cost: 2,
        base_damage: 7,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
        effects: &[],
        effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: crate::effects::declarative::CardFilter::All,
                action: BulkAction::Exhaust,
            },
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::ExtraHits(A::HandSizeAtPlay),
        ],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Fiend Fire+",
        name: "Fiend Fire+",
        card_type: CardType::Attack,
        target: CardTarget::Enemy,
        cost: 2,
        base_damage: 10,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
        effects: &[],
        effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: crate::effects::declarative::CardFilter::All,
                action: BulkAction::Exhaust,
            },
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::ExtraHits(A::HandSizeAtPlay),
        ],
        complex_hook: None,
    });
}
