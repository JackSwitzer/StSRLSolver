use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Multi-Cast: X cost, evoke frontmost orb X times (upgrade: X+1)
    insert(cards, CardDef {
                id: "Multi-Cast", name: "Multi-Cast", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["evoke_orb_x"], effect_data: &[
                    E::Simple(SE::EvokeOrb(A::XCost)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Multi-Cast+", name: "Multi-Cast+", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["evoke_orb_x_plus_1"], effect_data: &[
                    E::Simple(SE::EvokeOrb(A::MagicPlusX)),
                ], complex_hook: None,
            });
}
