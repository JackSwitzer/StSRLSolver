use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, NamedOptionKind, ScaledNamedOption};

static WISH_OPTIONS: [ScaledNamedOption; 3] = [
    ScaledNamedOption {
        label: "Strength",
        amount: A::Damage,
        kind: NamedOptionKind::AddStatus(crate::status_ids::sid::STRENGTH),
    },
    ScaledNamedOption {
        label: "Gold",
        amount: A::Magic,
        kind: NamedOptionKind::GainRunGold,
    },
    ScaledNamedOption {
        label: "Plated Armor",
        amount: A::Block,
        kind: NamedOptionKind::AddStatus(crate::status_ids::sid::PLATED_ARMOR),
    },
];

static WISH_EFFECTS: [E; 1] = [E::ChooseScaledNamedOptions(&WISH_OPTIONS)];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Wish ---- (cost 3, skill, exhaust, choose: +3 str, or 25 gold, or 6 block; upgrade: +1/+5/+2)
    insert(cards, CardDef {
                id: "Wish", name: "Wish", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: 3, base_block: 6,
                base_magic: 25, exhaust: true, enter_stance: None,
                effect_data: &WISH_EFFECTS, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Wish+", name: "Wish+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: 4, base_block: 8,
                base_magic: 30, exhaust: true, enter_stance: None,
                effect_data: &WISH_EFFECTS, complex_hook: None,
            });
}
