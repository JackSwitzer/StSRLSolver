use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Catalyst ---- (cost 1, double poison on enemy, exhaust; upgrade: triple)
    insert(cards, CardDef {
                id: "Catalyst", name: "Catalyst", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["catalyst_double"], effect_data: &[
                    E::Simple(SE::MultiplyStatus(T::SelectedEnemy, sid::POISON, 2)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Catalyst+", name: "Catalyst+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["catalyst_triple"], effect_data: &[
                    E::Simple(SE::MultiplyStatus(T::SelectedEnemy, sid::POISON, 3)),
                ], complex_hook: None,
            });
}
