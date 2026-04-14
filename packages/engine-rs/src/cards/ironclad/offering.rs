use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Offering ---- (cost 0, lose 6 HP, gain 2 energy, draw 3, exhaust; +2 draw)
    insert(cards, CardDef {
                id: "Offering", name: "Offering", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["offering"], effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-6))),
                    E::Simple(SE::GainEnergy(A::Fixed(2))),
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Offering+", name: "Offering+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effects: &["offering"], effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-6))),
                    E::Simple(SE::GainEnergy(A::Fixed(2))),
                    E::Simple(SE::DrawCards(A::Magic)),
                ], complex_hook: None,
            });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_ironclad_wave7.rs"]
mod test_card_runtime_ironclad_wave7;
