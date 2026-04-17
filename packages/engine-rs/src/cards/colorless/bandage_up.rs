use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Colorless Uncommon ----
        // Bandage Up: 0 cost, heal 4, exhaust
    insert(cards, CardDef {
                id: "Bandage Up", name: "Bandage Up", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effect_data: &[E::Simple(SE::ModifyHp(A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Bandage Up+", name: "Bandage Up+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 6, exhaust: true, enter_stance: None,
                effect_data: &[E::Simple(SE::ModifyHp(A::Magic))], complex_hook: None,
            });
}
