use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Bloodletting ---- (cost 0, lose 3 HP, gain 2 energy; +1 energy)
    insert(cards, CardDef {
                id: "Bloodletting", name: "Bloodletting", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-3))),
                    E::Simple(SE::GainEnergy(A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Bloodletting+", name: "Bloodletting+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ModifyHp(A::Fixed(-3))),
                    E::Simple(SE::GainEnergy(A::Magic)),
                ], complex_hook: None,
            });
}
