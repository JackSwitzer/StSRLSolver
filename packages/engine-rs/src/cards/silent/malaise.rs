use crate::cards::prelude::*;
use crate::status_ids::sid;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Malaise", name: "Malaise", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 0, exhaust: true, enter_stance: None,
                effects: &["x_cost"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::MagicPlusX)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::STRENGTH, A::MagicPlusXNeg)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Malaise+", name: "Malaise+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["x_cost"], effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::MagicPlusX)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::STRENGTH, A::MagicPlusXNeg)),
                ], complex_hook: None,
            });
}
