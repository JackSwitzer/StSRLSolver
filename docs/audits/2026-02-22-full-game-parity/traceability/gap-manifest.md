# Gap Manifest (Canonical)

Each row should be closable by one feature-sized commit.

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

## Active rows

| gap_id | domain | java_class | java_method_or_path | python_file | python_symbol | status | rng_streams | decision_surface | existing_tests | required_tests | priority | feature_id | planned_pr_region | notes |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| GAP-REL-001 | relics | `Orrery` | `relics/Orrery.java::onEquip` | `packages/engine/state/run.py`, `packages/engine/game.py` | `_on_relic_obtained`, `take_action_dict` | exact | `card_rng` | explicit_action | `tests/test_relic_pickup.py`, `tests/test_agent_api.py::test_shop_orrery_requires_card_selection`, `tests/test_agent_api.py::test_shop_orrery_selection_roundtrip_adds_five_cards`, `tests/test_agent_api.py::test_reward_orrery_requires_card_selection` | regression lock for deterministic IDs/validation | P0 | REL-003 | R1 | fixed: explicit follow-up `select_cards` with valid one-per-offer combinations |
| GAP-REL-002 | relics | `BottledFlame/Lightning/Tornado` | `relics/Bottled*.java::onEquip` | `packages/engine/state/run.py` | `_on_relic_obtained` | action-surface-missing | n/a | implicit_ui | `tests/test_relic_bottled.py` | new explicit action-surface tests | P0 | REL-004 | R1 | currently chooses first eligible card when unset |
| GAP-REL-003 | relics | `DollysMirror` | `relics/DollysMirror.java::onEquip` | `packages/engine/state/run.py` | `_on_relic_obtained` | action-surface-missing | n/a | implicit_ui | `tests/test_relic_pickup.py` | new explicit selection tests | P0 | REL-008 | R1 | currently duplicates deck index 0 |
| GAP-EVT-001 | events | multiple card-select events | `events/**` choice flows | `packages/engine/game.py` | `take_action_dict` + pending selection plumbing | action-surface-missing | `event_rng,misc_rng` | implicit_ui | `tests/test_events.py` | new event selection follow-up tests | P0 | EVT-001 | R2 | event actions do not yet emit follow-up `select_cards` |
| GAP-EVT-002 | events | multiple | `events/** execute with selected card` | `packages/engine/game.py` | `_handle_event_action` | action-surface-missing | `event_rng,misc_rng` | implicit_ui | `tests/test_events.py` | new passthrough tests for card-indexed event resolution | P0 | EVT-002 | R2 | current execution forces `card_idx=None` |
| GAP-RWD-001 | rewards/shop | multiple | reward + shop relic acquisition paths | `packages/engine/game.py` | `_handle_reward_action`, `_handle_shop_action` | approximate | `card_rng,relic_rng,misc_rng,potion_rng` | implicit_ui | `tests/test_rewards.py`, `tests/test_agent_api.py` | new unified selection interception tests | P1 | RWD-001 | R3 | relic acquisition paths bypass pending-selection interception |
| GAP-REL-004 | relic IDs | multiple | relic library ID canonicalization | `packages/engine/content/relics.py` | `ALL_RELICS` | missing | n/a | n/a | inventory tests | alias + coverage tests (`Toolbox` included) | P1 | REL-006 | R1 | Java-vs-Python ID shape mismatch; `Toolbox` missing in content |
| GAP-POW-001 | powers | multiple | `powers/**/*.java` inventory | `packages/engine/content/powers.py` | `POWER_DATA` | missing | mixed | n/a | power audit tests | per-class inventory + behavior tests | P1 | POW-001 | R4 | 149 Java classes vs 94 Python entries; 69 normalized gap candidates |
| GAP-ORB-001 | orbs | multiple orb-linked relics | relic/power orb interactions | `packages/engine/registry/relics.py` | orb-linked trigger handlers | missing | mixed | n/a | `tests/test_relic_*` partial | orb infrastructure + integration tests | P1 | ORB-001 | R4 | placeholder TODO logic still present |
| GAP-TST-001 | tests | n/a | audit assertions should enforce parity | `tests/test_audit_relics_combat.py` | bug-documentation tests | approximate | n/a | n/a | current audit tests | convert to parity assertions after fixes | P1 | AUD-001 | R6 | some tests currently assert that known bugs exist |
| GAP-CI-001 | testing/ci | n/a | replay artifact dependency | `tests/test_parity.py` | replay checks | approximate | n/a | n/a | baseline run | split artifact tests from normal CI profile | P2 | AUD-002 | R6 | current baseline includes 5 artifact-missing skips |
| GAP-POT-001 | audit-inventory | potion classes | `potions/*.java` not present in local decompile snapshot | `docs/audits/.../traceability/java-inventory.md` | inventory intake | missing | n/a | n/a | n/a | restore source or reference list | P2 | AUD-001 | R6 | local Java snapshot lacks dedicated potion class directory |
