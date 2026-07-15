#![cfg(test)]

use crate::powers::registry::status_is_debuff;
use crate::status_ids::sid;

#[test]
fn power_registry_debuff_classification_matches_runtime_needs() {
    assert!(status_is_debuff(sid::WEAKENED));
    assert!(status_is_debuff(sid::SLOW));
    assert!(status_is_debuff(sid::NO_BLOCK));
    assert!(!status_is_debuff(sid::STRENGTH));
    assert!(!status_is_debuff(sid::ARTIFACT));
}
