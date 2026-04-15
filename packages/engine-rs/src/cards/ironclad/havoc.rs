use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Havoc ---- (cost 1, play top card of draw pile; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Havoc", name: "Havoc", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::PlayTopCardOfDraw)], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Havoc+", name: "Havoc+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::PlayTopCardOfDraw)], complex_hook: None,
            });
}
