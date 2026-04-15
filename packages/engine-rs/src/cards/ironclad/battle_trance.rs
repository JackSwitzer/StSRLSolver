use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Battle Trance ---- (cost 0, draw 3, no more draw; +1)
    insert(cards, CardDef {
                id: "Battle Trance", name: "Battle Trance", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                    E::Simple(SE::SetFlag(BF::NoDraw)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Battle Trance+", name: "Battle Trance+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DrawCards(A::Magic)),
                    E::Simple(SE::SetFlag(BF::NoDraw)),
                ], complex_hook: None,
            });
}
