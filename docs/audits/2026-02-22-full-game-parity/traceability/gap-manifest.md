# Gap Manifest (Canonical)

Each gap row must be executable by one feature-sized commit.

## Schema
- `gap_id`
- `domain`
- `java_class`
- `java_method_or_path`
- `python_file`
- `python_symbol`
- `status` (`exact|approximate|missing|action-surface-missing`)
- `rng_streams`
- `decision_surface` (`explicit_action|implicit_ui|n/a`)
- `existing_tests`
- `required_tests`
- `priority`
- `feature_id`
- `planned_pr_region`
- `notes`

## Seed rows (to expand)

| gap_id | domain | java_class | java_method_or_path | python_file | python_symbol | status | rng_streams | decision_surface | existing_tests | required_tests | priority | feature_id | planned_pr_region | notes |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| GAP-EVT-001 | events | multiple selection events | event choice flows requiring card choice | `packages/engine/game.py` | `_handle_event_action` | action-surface-missing | `misc_rng,event_rng` | implicit_ui | `tests/test_audit_events.py` | `tests/parity_campaign/test_event_selection_flows.py` | P0 | EVT-002 | R2 | currently forces `card_idx=None` |
| GAP-REL-001 | relics | Orrery | on-equip card choice flow | `packages/engine/state/run.py` | `_on_relic_obtained` | action-surface-missing | `card_rng` | implicit_ui | `tests/test_relic_pickup.py` | `tests/parity_campaign/test_relic_selection_flows.py` | P0 | REL-003 | R1 | currently auto-picks first option |
| GAP-REL-002 | relics | Bottled* relics | on-acquire bottle assignment | `packages/engine/state/run.py` | `_on_relic_obtained` | action-surface-missing | n/a | implicit_ui | `tests/test_relic_bottled.py` | `tests/parity_campaign/test_relic_selection_flows.py` | P0 | REL-004 | R1 | deterministic first eligible auto-pick |
| GAP-REL-003 | relics | DollysMirror | on-acquire duplicate selection | `packages/engine/state/run.py` | `_on_relic_obtained` | action-surface-missing | n/a | implicit_ui | `tests/test_relic_pickup.py` | `tests/parity_campaign/test_relic_selection_flows.py` | P1 | REL-008 | R1 | currently duplicates deck index 0 |
| GAP-REL-004 | relics | Toolbox | relic content presence | `packages/engine/content/relics.py` | `ALL_RELICS` | missing | n/a | n/a | n/a | `tests/parity_campaign/test_traceability_matrix.py` | P1 | REL-006 | R1 | exists in generation inventory but missing in content |
| GAP-POW-001 | powers | multiple | class-level residual coverage | `packages/engine/registry/powers.py` | registry mapping | missing | mixed | n/a | existing power tests | `tests/parity_campaign/test_traceability_matrix.py` | P1 | POW-001 | R4 | expand after full mapping pass |
