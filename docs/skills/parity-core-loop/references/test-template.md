# Test Template

## Feature test structure
1. Name tests by `feature_id` and behavior clause.
2. Assert deterministic action availability and params.
3. Assert deterministic RNG stream movement where relevant.
4. Assert terminal state invariants (deck/relic/powers/rewards/phases).

## Required sections per feature test file
- Baseline fixture/setup
- Precondition assertions
- Action roundtrip assertions
- Regression assertion(s) for known previous bug
