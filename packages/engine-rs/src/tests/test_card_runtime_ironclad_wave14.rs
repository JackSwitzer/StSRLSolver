#![cfg(test)]

// Java oracle sources for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SecondWind.java

use crate::cards::global_registry;

#[test]
fn ironclad_wave14_registry_exports_keep_the_blocker_cluster_explicit() {
    for card_id in ["Dual Wield", "Fiend Fire", "Havoc", "Second Wind"] {
        let card = global_registry()
            .get(card_id)
            .unwrap_or_else(|| panic!("{card_id} should exist"));
        assert!(card.effect_data.is_empty(), "{card_id} should stay blocked");
        assert!(card.complex_hook.is_some(), "{card_id} should remain hook-backed");
    }
}

#[test]
#[ignore = "Blocked on Java attack-or-power union filtering for Dual Wield; the current declarative filter surface cannot express the card's option set. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java"]
fn ironclad_wave14_dual_wield_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java exhaust/per-hit sequencing for Fiend Fire; the current hook still owns the hand-exhaust + per-card damage loop. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java"]
fn ironclad_wave14_fiend_fire_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java top-of-draw play sequencing for Havoc; the current runtime still needs a dedicated play-top-card primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java"]
fn ironclad_wave14_havoc_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java non-attack bulk exhaust sequencing for Second Wind; the current runtime still needs a typed exhaust-all-non-attacks + per-card block primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SecondWind.java"]
fn ironclad_wave14_second_wind_stays_explicitly_hook_backed() {}
