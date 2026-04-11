use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Wreath of Flame ---- (cost 1, gain 5 Vigor; +3 magic upgrade)
    insert(cards, CardDef {
                id: "WreathOfFlame", name: "Wreath of Flame", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["vigor"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::VIGOR, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "WreathOfFlame+", name: "Wreath of Flame+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 8, exhaust: false, enter_stance: None,
                effects: &["vigor"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::VIGOR, A::Magic)),
                ], complex_hook: None,
            });
}
