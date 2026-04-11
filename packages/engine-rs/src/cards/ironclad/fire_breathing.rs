use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Fire Breathing ---- (cost 1, power, 6 dmg on Status/Curse draw; +4 magic)
    insert(cards, CardDef {
                id: "Fire Breathing", name: "Fire Breathing", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 6, exhaust: false, enter_stance: None,
                effects: &["fire_breathing"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FIRE_BREATHING, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Fire Breathing+", name: "Fire Breathing+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 10, exhaust: false, enter_stance: None,
                effects: &["fire_breathing"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FIRE_BREATHING, A::Magic)),
                ], complex_hook: None,
            });
}
