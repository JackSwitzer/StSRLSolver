//! Membership Card: shop prices are reduced by 50%.
//!
//! Source: `reference/extracted/methods/relic/MembershipCard.java` defines the
//! canonical SHOP-tier relic and multiplier. `StoreRelic.java::purchaseRelic`
//! immediately calls `ShopScreen.applyDiscount(0.5f, true)` when obtained.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Membership Card",
    name: "Membership Card",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
