use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Flame Barrier ---- (cost 2, 12 block + 4 fire dmg when hit; +4/+2)
    insert(cards, CardDef {
                id: "Flame Barrier", name: "Flame Barrier", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FLAME_BARRIER, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Flame Barrier+", name: "Flame Barrier+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
                base_magic: 6, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FLAME_BARRIER, A::Magic)),
                ], complex_hook: None,
            });
}
