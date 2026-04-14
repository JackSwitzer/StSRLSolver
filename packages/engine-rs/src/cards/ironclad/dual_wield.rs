use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Uncommon: Dual Wield ----
    // Still hook-backed until the shared attack-or-power union filter and
    // copy-selection sequencing are fully typed.
    insert(cards, CardDef {
        id: "Dual Wield",
        name: "Dual Wield",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: 1,
        exhaust: false,
        enter_stance: None,
        effects: &["dual_wield"],
        effect_data: &[],
        complex_hook: Some(crate::effects::hooks_complex::hook_dual_wield),
    });
    insert(cards, CardDef {
        id: "Dual Wield+",
        name: "Dual Wield+",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: 2,
        exhaust: false,
        enter_stance: None,
        effects: &["dual_wield"],
        effect_data: &[],
        complex_hook: Some(crate::effects::hooks_complex::hook_dual_wield),
    });
}
