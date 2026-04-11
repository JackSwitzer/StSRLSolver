use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Wish ---- (cost 3, skill, exhaust, choose: +3 str, or 25 gold, or 6 block; upgrade: +1/+5/+2)
        // TODO: Full effect requires ChooseOne UI (BecomeAlmighty, FameAndFortune, LiveForever)
    insert(cards, CardDef {
                id: "Wish", name: "Wish", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["wish"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Wish+", name: "Wish+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effects: &["wish"], effect_data: &[], complex_hook: None,
            });
}
