use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Uncommon: Ghostly Armor ----
    // cost 1, 10 block, ethereal; upgrade: 13 block
    insert(cards, CardDef {
        id: "Ghostly Armor",
        name: "Ghostly Armor",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: 10,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
        effects: &["ethereal"],
        effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Ghostly Armor+",
        name: "Ghostly Armor+",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: 13,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
        effects: &["ethereal"],
        effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
}
