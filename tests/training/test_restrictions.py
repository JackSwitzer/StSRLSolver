from packages.training.restrictions import (
    ActionCategory,
    ActionRestrictionPolicy,
    CandidateAction,
    DecisionSurface,
    RestrictionRule,
)


def test_combat_first_policy_blocks_card_rewards_but_keeps_skip():
    policy = ActionRestrictionPolicy.combat_first(allowed_neow_indices=(0, 2))
    actions = [
        CandidateAction(
            action_id="neow:0",
            surface=DecisionSurface.NEOW,
            category=ActionCategory.NEOW_OPTION,
            choice_index=0,
        ),
        CandidateAction(
            action_id="neow:1",
            surface=DecisionSurface.NEOW,
            category=ActionCategory.NEOW_OPTION,
            choice_index=1,
        ),
        CandidateAction(
            action_id="reward:claim",
            surface=DecisionSurface.REWARD,
            category=ActionCategory.CLAIM,
            reward_kind="card_choice",
        ),
        CandidateAction(
            action_id="reward:skip",
            surface=DecisionSurface.REWARD,
            category=ActionCategory.SKIP,
            reward_kind="card_choice",
        ),
    ]

    evaluation = policy.evaluate(actions)

    assert [action.action_id for action in evaluation.allowed_actions] == ["neow:0", "reward:skip"]
    blocked = {entry.action.action_id: entry.verdict.reason for entry in evaluation.blocked_actions}
    assert blocked["neow:1"] == "neow_choice_not_allowed"
    assert blocked["reward:claim"] == "card_rewards_disabled"


def test_custom_rule_can_block_by_surface_and_tag():
    policy = ActionRestrictionPolicy(
        name="no-shop-removes",
        rules=(
            RestrictionRule(
                name="disable_shop_remove",
                reason="shop_remove_disabled",
                surfaces=(DecisionSurface.SHOP,),
                categories=(ActionCategory.REMOVE,),
                required_tags=("purge",),
            ),
        ),
    )
    action = CandidateAction(
        action_id="shop:remove:0",
        surface=DecisionSurface.SHOP,
        category=ActionCategory.REMOVE,
        tags=("purge",),
    )

    verdict = policy.verdict(action)

    assert not verdict.allowed
    assert verdict.reason == "shop_remove_disabled"
    assert verdict.matched_rule == "disable_shop_remove"
