use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Swivel ---- (cost 2, 8 block, next attack costs 0; +3 block upgrade)
    insert(cards, CardDef {
                id: "Swivel", name: "Swivel", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 8,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["next_attack_free"], effect_data: &[
                    E::Simple(SE::SetFlag(BF::NextAttackFree)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Swivel+", name: "Swivel+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 11,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["next_attack_free"], effect_data: &[
                    E::Simple(SE::SetFlag(BF::NextAttackFree)),
                ], complex_hook: None,
            });
}
