//! post_play_dest hooks — where a card goes after being played.

use crate::cards::CardDef;
use super::types::PostPlayDestination;

/// Tantrum: shuffle back into draw pile.
pub fn hook_shuffle_self_into_draw(_card: &CardDef) -> PostPlayDestination {
    PostPlayDestination::ShuffleIntoDraw
}

/// Conclude: end the turn after playing.
pub fn hook_end_turn(_card: &CardDef) -> PostPlayDestination {
    PostPlayDestination::EndTurn
}
