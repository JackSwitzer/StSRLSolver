use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Glass Knife deals its current damage twice, then ModifyDamageAction
    // reduces only that battle instance by 2. upgradeDamage(4) makes 12.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/GlassKnife.java
    insert(
        cards,
        CardDef {
            id: "Glass Knife",
            name: "Glass Knife",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 8,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Simple(SE::ModifyPlayedCardDamage(A::Fixed(-2))),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Glass Knife+",
            name: "Glass Knife+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 12,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Simple(SE::ModifyPlayedCardDamage(A::Fixed(-2))),
            ],
            complex_hook: None,
        },
    );
}
