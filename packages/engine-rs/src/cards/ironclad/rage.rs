use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Rage ---- (cost 0, gain 3 block per attack played this turn; +2 magic)
    insert(cards, CardDef {
                id: "Rage", name: "Rage", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["rage"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::RAGE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Rage+", name: "Rage+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["rage"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::RAGE, A::Magic)),
                ], complex_hook: None,
            });
}
