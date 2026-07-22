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
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
    // The three option cards read Wish's unmodified damage/magic/block fields;
    // Wish.applyPowers is intentionally empty so combat modifiers cannot scale them.
    insert(
        cards,
        CardDef {
            id: "Wish",
            name: "Wish",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 3,
            base_damage: 3,
            base_block: 6,
            base_magic: 25,
            exhaust: true,
            enter_stance: None,
            effect_data: &WISH_EFFECTS,
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Wish+",
            name: "Wish+",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 3,
            base_damage: 4,
            base_block: 8,
            base_magic: 30,
            exhaust: true,
            enter_stance: None,
            effect_data: &WISH_EFFECTS,
            complex_hook: None,
        },
    );
}
