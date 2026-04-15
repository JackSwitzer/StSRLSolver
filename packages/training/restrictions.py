"""Restriction-policy helpers layered above engine legality."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum

from .contracts import RestrictionBuiltin, RestrictionPolicy


class DecisionSurface(str, Enum):
    NEOW = "neow"
    COMBAT = "combat"
    REWARD = "reward"
    SHOP = "shop"
    CAMPFIRE = "campfire"
    MAP = "map"
    EVENT = "event"


class ActionCategory(str, Enum):
    NEOW_OPTION = "neow_option"
    CLAIM = "claim"
    SKIP = "skip"
    REMOVE = "remove"
    BUY = "buy"
    PLAY_CARD = "play_card"
    USE_POTION = "use_potion"
    END_TURN = "end_turn"


@dataclass(frozen=True)
class CandidateAction:
    action_id: str
    surface: DecisionSurface
    category: ActionCategory
    choice_index: int | None = None
    reward_kind: str | None = None
    tags: tuple[str, ...] = ()


@dataclass(frozen=True)
class RestrictionRule:
    name: str
    reason: str
    surfaces: tuple[DecisionSurface, ...] = ()
    categories: tuple[ActionCategory, ...] = ()
    required_tags: tuple[str, ...] = ()
    allowed_choice_indices: tuple[int, ...] | None = None
    blocked_reward_kinds: tuple[str, ...] = ()


@dataclass(frozen=True)
class RestrictionVerdict:
    allowed: bool
    reason: str | None = None
    matched_rule: str | None = None


@dataclass(frozen=True)
class BlockedAction:
    action: CandidateAction
    verdict: RestrictionVerdict


@dataclass(frozen=True)
class RestrictionEvaluation:
    allowed_actions: tuple[CandidateAction, ...]
    blocked_actions: tuple[BlockedAction, ...]


@dataclass(frozen=True)
class ActionRestrictionPolicy:
    name: str
    rules: tuple[RestrictionRule, ...] = ()

    @classmethod
    def combat_first(cls, *, allowed_neow_indices: tuple[int, ...]) -> "ActionRestrictionPolicy":
        return cls(
            name="combat_first",
            rules=(
                RestrictionRule(
                    name="limit_neow_choices",
                    reason="neow_choice_not_allowed",
                    surfaces=(DecisionSurface.NEOW,),
                    categories=(ActionCategory.NEOW_OPTION,),
                    allowed_choice_indices=allowed_neow_indices,
                ),
                RestrictionRule(
                    name="disable_card_rewards",
                    reason="card_rewards_disabled",
                    surfaces=(DecisionSurface.REWARD,),
                    categories=(ActionCategory.CLAIM,),
                    blocked_reward_kinds=("card_choice",),
                ),
            ),
        )

    def verdict(self, action: CandidateAction) -> RestrictionVerdict:
        for rule in self.rules:
            if rule.surfaces and action.surface not in rule.surfaces:
                continue
            if rule.categories and action.category not in rule.categories:
                continue
            if rule.required_tags and not all(tag in action.tags for tag in rule.required_tags):
                continue
            if rule.allowed_choice_indices is not None and action.choice_index not in rule.allowed_choice_indices:
                return RestrictionVerdict(False, rule.reason, rule.name)
            if rule.blocked_reward_kinds and action.reward_kind in rule.blocked_reward_kinds:
                return RestrictionVerdict(False, rule.reason, rule.name)
            if rule.allowed_choice_indices is None and not rule.blocked_reward_kinds:
                return RestrictionVerdict(False, rule.reason, rule.name)
        return RestrictionVerdict(True)

    def evaluate(self, actions: list[CandidateAction]) -> RestrictionEvaluation:
        allowed: list[CandidateAction] = []
        blocked: list[BlockedAction] = []
        for action in actions:
            verdict = self.verdict(action)
            if verdict.allowed:
                allowed.append(action)
            else:
                blocked.append(BlockedAction(action, verdict))
        return RestrictionEvaluation(tuple(allowed), tuple(blocked))


def no_card_rewards() -> RestrictionPolicy:
    return RestrictionPolicy((RestrictionBuiltin.NO_CARD_REWARDS,))


def no_card_adds() -> RestrictionPolicy:
    return RestrictionPolicy((RestrictionBuiltin.NO_CARD_ADDS,))


def upgrade_remove_only() -> RestrictionPolicy:
    return RestrictionPolicy((RestrictionBuiltin.UPGRADE_REMOVE_ONLY,))


def merge_policies(*policies: RestrictionPolicy) -> RestrictionPolicy:
    ordered: list[RestrictionBuiltin] = []
    for policy in policies:
        for builtin in policy.builtins:
            if builtin not in ordered:
                ordered.append(builtin)
    return RestrictionPolicy(tuple(ordered))


def policy_slug(policy: RestrictionPolicy) -> str:
    if not policy.builtins:
        return "unrestricted"
    return "+".join(builtin.value for builtin in policy.builtins)
