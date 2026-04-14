use crate::actions::Action;
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine};
use crate::run::{
    RunAction, RunPhase, ShopState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionKind {
    CombatAction,
    CombatChoice,
    RewardScreen,
    MapPath,
    EventOption,
    ShopAction,
    CampfireAction,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionState {
    pub kind: DecisionKind,
    pub phase: RunPhase,
    pub terminal: bool,
    pub room_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewardScreenSource {
    Combat,
    BossCombat,
    Event,
    Treasure,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardItemState {
    Available,
    Claimed,
    Skipped,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardItemKind {
    CardChoice,
    Relic,
    Gold,
    Potion,
    Key,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewardChoice {
    Card {
        index: usize,
        card_id: String,
    },
    Named {
        index: usize,
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardItem {
    pub index: usize,
    pub kind: RewardItemKind,
    pub state: RewardItemState,
    pub label: String,
    pub claimable: bool,
    pub active: bool,
    pub skip_allowed: bool,
    pub skip_label: Option<String>,
    pub choices: Vec<RewardChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardScreen {
    pub source: RewardScreenSource,
    pub ordered: bool,
    pub active_item: Option<usize>,
    pub items: Vec<RewardItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardChoiceFrame {
    pub item_index: usize,
    pub item_kind: RewardItemKind,
    pub skip_allowed: bool,
    pub choices: Vec<RewardChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatPotionSlotContext {
    pub slot: usize,
    pub occupied: bool,
    pub potion_id: String,
    pub requires_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatChoiceOptionContext {
    pub index: usize,
    pub kind: String,
    pub source_index: i32,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatChoiceContext {
    pub active: bool,
    pub reason: Option<String>,
    pub option_count: usize,
    pub min_picks: usize,
    pub max_picks: usize,
    pub selected: Vec<usize>,
    pub options: Vec<CombatChoiceOptionContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatContext {
    pub potions: Vec<CombatPotionSlotContext>,
    pub choice: CombatChoiceContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDecisionContext {
    pub available_paths: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventOptionContext {
    pub index: usize,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDecisionContext {
    pub name: String,
    pub options: Vec<EventOptionContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopCardOfferContext {
    pub index: usize,
    pub card_id: String,
    pub price: i32,
    pub affordable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopDecisionContext {
    pub offers: Vec<ShopCardOfferContext>,
    pub remove_price: i32,
    pub removal_used: bool,
    pub removable_cards: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampfireDecisionContext {
    pub can_rest: bool,
    pub upgradable_cards: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionContext {
    pub kind: DecisionKind,
    pub combat: Option<CombatContext>,
    pub reward_screen: Option<RewardScreen>,
    pub map: Option<MapDecisionContext>,
    pub event: Option<EventDecisionContext>,
    pub shop: Option<ShopDecisionContext>,
    pub campfire: Option<CampfireDecisionContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionFrame {
    Combat(CombatContext),
    CombatChoice(CombatChoiceContext),
    RewardScreen { source: RewardScreenSource },
    RewardChoice(RewardChoiceFrame),
    Map(MapDecisionContext),
    Event(EventDecisionContext),
    Shop(ShopDecisionContext),
    Campfire(CampfireDecisionContext),
    GameOver,
}

impl DecisionFrame {
    pub fn kind(&self) -> DecisionKind {
        match self {
            Self::Combat(_) => DecisionKind::CombatAction,
            Self::CombatChoice(_) => DecisionKind::CombatChoice,
            Self::RewardScreen { .. } | Self::RewardChoice(_) => DecisionKind::RewardScreen,
            Self::Map(_) => DecisionKind::MapPath,
            Self::Event(_) => DecisionKind::EventOption,
            Self::Shop(_) => DecisionKind::ShopAction,
            Self::Campfire(_) => DecisionKind::CampfireAction,
            Self::GameOver => DecisionKind::GameOver,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecisionStack {
    pub frames: Vec<DecisionFrame>,
}

impl DecisionStack {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn push(&mut self, frame: DecisionFrame) {
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> Option<DecisionFrame> {
        self.frames.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    pub fn current_frame(&self) -> Option<&DecisionFrame> {
        self.frames.last()
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut DecisionFrame> {
        self.frames.last_mut()
    }

    pub fn current_kind(&self) -> Option<DecisionKind> {
        self.current_frame().map(DecisionFrame::kind)
    }

    pub fn current_reward_choice(&self) -> Option<&RewardChoiceFrame> {
        match self.current_frame() {
            Some(DecisionFrame::RewardChoice(frame)) => Some(frame),
            _ => None,
        }
    }

    pub fn current_reward_choice_mut(&mut self) -> Option<&mut RewardChoiceFrame> {
        match self.current_frame_mut() {
            Some(DecisionFrame::RewardChoice(frame)) => Some(frame),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionAction {
    ChooseMapPath(usize),
    Combat(Action),
    ClaimRewardItem {
        item_index: usize,
    },
    PickRewardChoice {
        item_index: usize,
        choice_index: usize,
    },
    SkipRewardItem {
        item_index: usize,
    },
    CampfireRest,
    CampfireUpgrade(usize),
    ShopBuyCard(usize),
    ShopRemoveCard(usize),
    ShopLeave,
    EventChoice(usize),
}

impl DecisionAction {
    pub fn to_run_action(&self) -> RunAction {
        match self {
            Self::ChooseMapPath(idx) => RunAction::ChoosePath(*idx),
            Self::Combat(action) => RunAction::CombatAction(action.clone()),
            Self::ClaimRewardItem { item_index } => RunAction::SelectRewardItem(*item_index),
            Self::PickRewardChoice {
                item_index,
                choice_index,
            } => RunAction::ChooseRewardOption {
                item_index: *item_index,
                choice_index: *choice_index,
            },
            Self::SkipRewardItem { item_index } => RunAction::SkipRewardItem(*item_index),
            Self::CampfireRest => RunAction::CampfireRest,
            Self::CampfireUpgrade(idx) => RunAction::CampfireUpgrade(*idx),
            Self::ShopBuyCard(idx) => RunAction::ShopBuyCard(*idx),
            Self::ShopRemoveCard(idx) => RunAction::ShopRemoveCard(*idx),
            Self::ShopLeave => RunAction::ShopLeave,
            Self::EventChoice(idx) => RunAction::EventChoice(*idx),
        }
    }

    pub fn from_run_action(action: &RunAction, phase: RunPhase) -> Self {
        match action {
            RunAction::ChoosePath(idx) => Self::ChooseMapPath(*idx),
            RunAction::SelectRewardItem(item_index) => Self::ClaimRewardItem {
                item_index: *item_index,
            },
            RunAction::ChooseRewardOption {
                item_index,
                choice_index,
            } => Self::PickRewardChoice {
                item_index: *item_index,
                choice_index: *choice_index,
            },
            RunAction::SkipRewardItem(item_index) => Self::SkipRewardItem {
                item_index: *item_index,
            },
            RunAction::CampfireRest => Self::CampfireRest,
            RunAction::CampfireUpgrade(idx) => Self::CampfireUpgrade(*idx),
            RunAction::ShopBuyCard(idx) => Self::ShopBuyCard(*idx),
            RunAction::ShopRemoveCard(idx) => Self::ShopRemoveCard(*idx),
            RunAction::ShopLeave => Self::ShopLeave,
            RunAction::EventChoice(idx) => Self::EventChoice(*idx),
            RunAction::CombatAction(action) => {
                let _ = phase;
                Self::Combat(action.clone())
            }
        }
    }
}

pub(crate) fn build_shop_context(shop: &ShopState, gold: i32, deck_len: usize) -> ShopDecisionContext {
    ShopDecisionContext {
        offers: shop
            .cards
            .iter()
            .enumerate()
            .map(|(index, (card_id, price))| ShopCardOfferContext {
                index,
                card_id: card_id.clone(),
                price: *price,
                affordable: gold >= *price,
            })
            .collect(),
        remove_price: shop.remove_price,
        removal_used: shop.removal_used,
        removable_cards: if !shop.removal_used && deck_len > 5 { deck_len } else { 0 },
    }
}

pub(crate) fn build_combat_context(combat: &CombatEngine) -> CombatContext {
    CombatContext {
        potions: combat
            .state
            .potions
            .iter()
            .enumerate()
            .map(|(slot, potion_id)| CombatPotionSlotContext {
                slot,
                occupied: !potion_id.is_empty(),
                potion_id: potion_id.clone(),
                requires_target: !potion_id.is_empty()
                    && crate::potions::potion_requires_target(potion_id),
            })
            .collect(),
        choice: build_choice_context(combat),
    }
}

fn build_choice_context(combat: &CombatEngine) -> CombatChoiceContext {
    let Some(choice) = &combat.choice else {
        return CombatChoiceContext {
            active: false,
            reason: None,
            option_count: 0,
            min_picks: 0,
            max_picks: 0,
            selected: Vec::new(),
            options: Vec::new(),
        };
    };

    CombatChoiceContext {
        active: true,
        reason: Some(choice_reason_name(&choice.reason).to_string()),
        option_count: choice.options.len(),
        min_picks: choice.min_picks,
        max_picks: choice.max_picks,
        selected: choice.selected.clone(),
        options: choice
            .options
            .iter()
            .enumerate()
            .map(|(index, option)| {
                let (kind, source_index, label) = choice_option_payload(option, combat);
                CombatChoiceOptionContext {
                    index,
                    kind,
                    source_index,
                    label,
                    selected: choice.selected.contains(&index),
                }
            })
            .collect(),
    }
}

fn choice_reason_name(reason: &ChoiceReason) -> &'static str {
    match reason {
        ChoiceReason::Scry => "scry",
        ChoiceReason::DiscardFromHand => "discard_from_hand",
        ChoiceReason::ExhaustFromHand => "exhaust_from_hand",
        ChoiceReason::PutOnTopFromHand => "put_on_top_from_hand",
        ChoiceReason::PickFromDiscard => "pick_from_discard",
        ChoiceReason::PickFromDrawPile => "pick_from_draw_pile",
        ChoiceReason::DiscoverCard => "discover_card",
        ChoiceReason::PickOption => "pick_option",
        ChoiceReason::PlayCardFree => "play_card_free",
        ChoiceReason::DualWield => "dual_wield",
        ChoiceReason::UpgradeCard => "upgrade_card",
        ChoiceReason::PickFromExhaust => "pick_from_exhaust",
        ChoiceReason::SearchDrawPile => "search_draw_pile",
        ChoiceReason::ReturnFromDiscard => "return_from_discard",
        ChoiceReason::ForethoughtPick => "forethought_pick",
        ChoiceReason::RecycleCard => "recycle_card",
        ChoiceReason::DiscardForEffect => "discard_for_effect",
        ChoiceReason::SetupPick => "setup_pick",
        ChoiceReason::PlayCardFreeFromDraw => "play_card_free_from_draw",
    }
}

fn choice_option_payload(option: &ChoiceOption, combat: &CombatEngine) -> (String, i32, String) {
    match option {
        ChoiceOption::HandCard(idx) => {
            let name = combat
                .state
                .hand
                .get(*idx)
                .map(|card| combat.card_registry.card_name(card.def_id).to_string())
                .unwrap_or_else(|| format!("hand_{}", idx));
            ("hand_card".to_string(), *idx as i32, name)
        }
        ChoiceOption::DrawCard(idx) => {
            let name = combat
                .state
                .draw_pile
                .get(*idx)
                .map(|card| combat.card_registry.card_name(card.def_id).to_string())
                .unwrap_or_else(|| format!("draw_{}", idx));
            ("draw_card".to_string(), *idx as i32, name)
        }
        ChoiceOption::DiscardCard(idx) => {
            let name = combat
                .state
                .discard_pile
                .get(*idx)
                .map(|card| combat.card_registry.card_name(card.def_id).to_string())
                .unwrap_or_else(|| format!("discard_{}", idx));
            ("discard_card".to_string(), *idx as i32, name)
        }
        ChoiceOption::RevealedCard(card) => (
            "revealed_card".to_string(),
            -1,
            combat.card_registry.card_name(card.def_id).to_string(),
        ),
        ChoiceOption::GeneratedCard(card) => (
            "generated_card".to_string(),
            -1,
            combat.card_registry.card_name(card.def_id).to_string(),
        ),
        ChoiceOption::Named(name) => ("named".to_string(), -1, (*name).to_string()),
        ChoiceOption::ExhaustCard(idx) => {
            let name = combat
                .state
                .exhaust_pile
                .get(*idx)
                .map(|card| combat.card_registry.card_name(card.def_id).to_string())
                .unwrap_or_else(|| format!("exhaust_{}", idx));
            ("exhaust_card".to_string(), *idx as i32, name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_stack_tracks_nested_reward_choice() {
        let mut stack = DecisionStack::new();
        stack.push(DecisionFrame::RewardScreen {
            source: RewardScreenSource::Combat,
        });
        assert_eq!(stack.current_kind(), Some(DecisionKind::RewardScreen));
        assert!(stack.current_reward_choice().is_none());

        stack.push(DecisionFrame::RewardChoice(RewardChoiceFrame {
            item_index: 2,
            item_kind: RewardItemKind::CardChoice,
            skip_allowed: true,
            choices: vec![RewardChoice::Named {
                index: 0,
                label: "Skip".to_string(),
            }],
        }));

        assert_eq!(stack.current_kind(), Some(DecisionKind::RewardScreen));
        let choice = stack.current_reward_choice().expect("reward choice frame");
        assert_eq!(choice.item_index, 2);
        assert!(choice.skip_allowed);
        assert_eq!(choice.choices.len(), 1);
    }

    #[test]
    fn decision_frame_kind_maps_combat_choice() {
        let frame = DecisionFrame::CombatChoice(CombatChoiceContext {
            active: true,
            reason: Some("scry".to_string()),
            option_count: 1,
            min_picks: 1,
            max_picks: 1,
            selected: vec![0],
            options: vec![CombatChoiceOptionContext {
                index: 0,
                kind: "hand_card".to_string(),
                source_index: 0,
                label: "Strike".to_string(),
                selected: true,
            }],
        });
        assert_eq!(frame.kind(), DecisionKind::CombatChoice);
    }
}
