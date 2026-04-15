from packages.training.contracts import CombatFrontierLine, CombatOutcomeVector
from packages.training.selector import rank_frontier_lines, select_frontier, select_frontier_line


def _line(
    *,
    idx: int,
    solve_probability: float,
    hp_loss: float,
    potion_cost: float,
    setup_gain: float = 0.0,
    scaling_gain: float = 0.0,
    turns: float = 3.0,
    action_prefix: tuple[int, ...] | None = None,
    visits: int = 100,
) -> CombatFrontierLine:
    return CombatFrontierLine(
        line_index=idx,
        action_prefix=action_prefix or (idx,),
        visits=visits,
        expanded_nodes=250,
        elapsed_ms=12,
        outcome=CombatOutcomeVector(
            solve_probability=solve_probability,
            expected_hp_loss=hp_loss,
            expected_turns=turns,
            potion_cost=potion_cost,
            setup_value_delta=setup_gain,
            persistent_scaling_delta=scaling_gain,
        ),
    )


def test_selector_prefers_lower_hp_loss_before_potion_savings():
    flex_line = _line(idx=0, solve_probability=1.0, hp_loss=2.0, potion_cost=1.0)
    no_potion_line = _line(idx=1, solve_probability=1.0, hp_loss=12.0, potion_cost=0.0)
    chosen = select_frontier_line((flex_line, no_potion_line))
    assert chosen.line_index == 0


def test_selector_prefers_higher_solve_probability_first():
    shaky = _line(idx=0, solve_probability=0.92, hp_loss=0.0, potion_cost=0.0)
    stable = _line(idx=1, solve_probability=0.99, hp_loss=8.0, potion_cost=0.0)
    chosen = select_frontier_line((shaky, stable))
    assert chosen.line_index == 1


def test_selector_breaks_exact_ties_deterministically_and_preserves_frontier():
    late = _line(
        idx=7,
        solve_probability=1.0,
        hp_loss=0.0,
        potion_cost=0.0,
        action_prefix=(2, 1),
        visits=50,
    )
    early = _line(
        idx=4,
        solve_probability=1.0,
        hp_loss=0.0,
        potion_cost=0.0,
        action_prefix=(1, 9),
        visits=999,
    )

    selection = select_frontier((late, early))
    ordered = rank_frontier_lines((late, early))

    assert ordered[0].line_index == 4
    assert selection.chosen.line_index == 4
    assert [line.line_index for line in selection.ordered_frontier] == [4, 7]
