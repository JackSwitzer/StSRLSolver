use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Uncommon: Burning Pact ----
    // Typed choice body carries its own post-choice draw amount.
    insert(cards, CardDef {
        id: "Burning Pact",
        name: "Burning Pact",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: 2,
        exhaust: false,
        enter_stance: None,
        effects: &["exhaust_choose", "draw"],
        effect_data: &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::Exhaust,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
            post_choice_draw: A::Magic,
        }],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Burning Pact+",
        name: "Burning Pact+",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: 3,
        exhaust: false,
        enter_stance: None,
        effects: &["exhaust_choose", "draw"],
        effect_data: &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::Exhaust,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
            post_choice_draw: A::Magic,
        }],
        complex_hook: None,
    });
}
