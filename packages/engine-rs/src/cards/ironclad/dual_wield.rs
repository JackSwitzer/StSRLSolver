use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // DualWieldAction.java offers only Attacks/Powers and makes magicNumber
    // stat-equivalent copies; the upgrade raises that count from 1 to 2.
    // Java: reference/extracted/methods/card/DualWield.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DualWieldAction.java
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
                effect_data: &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::AttackOrPower,
            action: ChoiceAction::CopyToHand,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
            post_choice_draw: A::Fixed(0),
        }],
        complex_hook: None,
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
                effect_data: &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::AttackOrPower,
            action: ChoiceAction::CopyToHand,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
            post_choice_draw: A::Fixed(0),
        }],
        complex_hook: None,
    });
}
