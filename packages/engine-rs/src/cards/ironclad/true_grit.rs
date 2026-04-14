use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: True Grit ---- (cost 1, 7 block, exhaust random card; upgrade: +2 block, choose)
    insert(cards, CardDef {
                id: "True Grit", name: "True Grit", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &[],
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
                complex_hook: Some(crate::effects::hooks_complex::hook_exhaust_random),
            });
    insert(cards, CardDef {
                id: "True Grit+", name: "True Grit+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["exhaust_choose"], effect_data: &[
                    E::ChooseCards {
                        source: P::Hand,
                        filter: crate::effects::declarative::CardFilter::All,
                        action: crate::effects::declarative::ChoiceAction::Exhaust,
                        min_picks: A::Fixed(1),
                        max_picks: A::Fixed(1),
                    },
                ], complex_hook: None,
            });
}
