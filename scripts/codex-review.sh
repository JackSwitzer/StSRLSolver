#!/usr/bin/env bash
# codex-review.sh — Repeatable GPT 5.4 review via Codex CLI
#
# Usage:
#   ./scripts/codex-review.sh <review-type> [--effort high|extra-high] [--files file1,file2] [--focus "topic"]
#
# Review types:
#   gpu       — GPU integration, MLX optimization, Rust engine merge strategy
#   bugs      — Bug hunt across training pipeline
#   deadcode  — Dead code removal, unused imports, stale configs
#   training  — Training health: rewards, loss, convergence
#   combat    — Combat solver correctness, scoring, search efficiency
#   full      — Run all 5 reviews in sequence
#   custom    — Use --focus to specify a custom review topic
#
# Examples:
#   ./scripts/codex-review.sh bugs --effort extra-high
#   ./scripts/codex-review.sh combat --files packages/training/turn_solver.py
#   ./scripts/codex-review.sh custom --focus "potion scoring and usage in overnight.py"
#   ./scripts/codex-review.sh full --effort extra-high  # Run all reviews

set -euo pipefail

CODEX="/Applications/Codex.app/Contents/Resources/codex"
EFFORT="extra-high"
REVIEW_TYPE="${1:-help}"
FILES=""
FOCUS=""
OUTPUT_DIR="logs/codex-reviews"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Parse flags
shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --effort) EFFORT="$2"; shift 2 ;;
    --files) FILES="$2"; shift 2 ;;
    --focus) FOCUS="$2"; shift 2 ;;
    --output-dir) OUTPUT_DIR="$2"; shift 2 ;;
    *) echo "Unknown flag: $1"; exit 1 ;;
  esac
done

mkdir -p "$OUTPUT_DIR"

# Common project context header
PROJECT_CONTEXT="PROJECT: Slay the Spire RL bot (Watcher, A20 target).
STACK: Python game engine (6060 tests) + PPO training + MLX inference + multiprocessing.
HARDWARE: M4 Mac Mini (10 cores, 24GB unified RAM, MPS GPU, MLX Apple Silicon).
KEY FILES:
- packages/training/overnight.py — Training orchestrator (1300 lines)
- packages/training/turn_solver.py — Combat card search (786 lines)
- packages/training/inference_server.py — Batch MLX inference daemon (762 lines)
- packages/training/strategic_trainer.py — PPO trainer
- packages/training/strategic_net.py — StrategicNet (3M params)
- packages/training/mlx_inference.py — MLX inference backend
- packages/engine/combat_engine.py — Combat engine (2597 lines)
- packages/engine-rs/ — Rust CombatEngine (63 tests, PyO3 bindings, NOT YET INTEGRATED)"

run_review() {
  local name="$1"
  local prompt="$2"
  local outfile="$OUTPUT_DIR/${TIMESTAMP}-${name}.md"

  echo "[$name] Starting review (effort=$EFFORT)..."

  if ! "$CODEX" exec \
    -m gpt-5.4 \
    -c "reasoning_effort=\"$EFFORT\"" \
    --sandbox read-only \
    "$prompt" > "$outfile" 2>&1; then
    echo "[$name] FAILED — see $outfile"
    return 1
  fi

  echo "[$name] Done -> $outfile"
}

case "$REVIEW_TYPE" in
  gpu)
    run_review "gpu" "$PROJECT_CONTEXT

REVIEW: GPU Integration & Merge Strategy

GPU utilization is 0% during training. All combat search runs on pure Python CPU.

EXISTING ASSETS:
1. Rust CombatEngine (packages/engine-rs/): 2810 lines, 63 tests, PyO3 bindings with clone_for_mcts()
2. MLX inference server: forward_batch(obs, mask) on Apple Silicon unified memory
3. TurnSolver: engine.copy() + execute_action() + _score_terminal() hot loop (50ms budget)
4. PyTorch MPS for PPO training

ANALYZE:
1. Fastest path to GPU utilization > 50% on M4
2. Rust CombatEngine integration plan for TurnSolver (fallback for missing features)
3. MLX batched neural value estimation during search (neural MCTS)
4. Dead systems to archive (EffectExecutor, combat_transitions_queue)
5. Step-by-step merge order with test gates

OUTPUT: (a) Ranked integration plan with GPU% gains, (b) concrete file:line changes, (c) deletions, (d) risk per step."
    ;;

  bugs)
    run_review "bugs" "$PROJECT_CONTEXT

REVIEW: Bug Hunt — Training Pipeline

Current: 5K games, avg floor 7.3, 0% WR, 303 g/min.

CHECK THESE SPECIFIC PATTERNS:
- PBRS reward sign: gamma * Phi(s') - Phi(s), verify positive = good
- Terminal reward applied to correct transition
- GAE computation off-by-one
- Importance sampling ratio with epsilon-greedy
- Multiprocessing race conditions (shared queues, weight sync)
- TurnSolver fallback contract (returns None on failure?)
- Entropy decay gating (conditional on avg_floor)
- Action masking in StrategicNet vs worker sampling
- Memory leaks in long-running workers
- InferenceServer weight sync race with training thread

READ the actual files. OUTPUT: P0/P1/P2 bugs with file:line and fix suggestions."
    ;;

  deadcode)
    run_review "deadcode" "$PROJECT_CONTEXT

REVIEW: Dead Code + MLX Optimization

PART 1 — DEAD CODE: Find ALL dead code, unused imports, orphaned systems:
- packages/engine/effects/executor.py (669 lines) — is it truly dead?
- combat_transitions_queue in packages/server/training_runner.py
- Stale config fields (mcts_sims, etc.)
- Unused power/relic triggers
- Dead WebSocket message handlers

PART 2 — MLX OPTIMIZATION:
- Is mx.compile() on forward_batch hot path? Expected speedup?
- Batch timeout 5ms — optimal for M4?
- Unnecessary numpy<->MLX<->torch conversions?
- Unified memory utilization?

READ the actual files. OUTPUT: (a) deletions with file:line, (b) MLX optimizations ranked, (c) post-Rust optimization roadmap."
    ;;

  training)
    run_review "training" "$PROJECT_CONTEXT

REVIEW: Training Health & Reward Signal

ANALYZE:
1. PBRS: Phi(s) = 0.45*floor + 0.30*hp + 0.15*deck + 0.10*relics — are weights sensible?
2. Event rewards: combat=0.05, elite=0.15, boss=0.40 — too sparse?
3. Terminal: +1.0 win, -0.5*(1-progress) loss — gradient signal strength?
4. Entropy decay: conditional on avg_floor (>5.5: 0.9999, >7.0: 0.999) — too fast/slow?
5. LR schedule: cosine with T_max=30000 — appropriate for run length?
6. Potion heuristic scores 3.0 vs cards 6-20+ — potions never used
7. Card removal: only PBRS signal (~0.03) — should be explicit reward?
8. Sweep configs: 12 configs with adaptive 3-phase — good coverage?

READ the actual files. OUTPUT: ranked improvements with expected impact on avg_floor."
    ;;

  combat)
    run_review "combat" "$PROJECT_CONTEXT

REVIEW: Combat Solver Correctness

ANALYZE turn_solver.py:
1. Scoring: -6*hp_lost + 60*kills - 1.5*ehp_ratio*10 - 12*est_turns — balanced?
2. Wrath: -60 penalty if enemies alive, +25 Calm bonus — correct?
3. DFS pruning: is alpha-beta actually cutting branches?
4. Simulation: engine.copy() + EndTurn + tick() — correct enemy turn sim?
5. Fallback chain: TurnSolver -> CombatPlanner -> heuristic — all return types correct?
6. Potion consideration: TurnSolver treats potions as setup actions — working?
7. Cached plan replay: _turn_plan_cache hash invalidation — correct?
8. Beam search: reserved slots for setup cards — effective?

${FOCUS:+ADDITIONAL FOCUS: $FOCUS}

READ the actual files. OUTPUT: P0/P1/P2 issues with file:line and fix code."
    ;;

  custom)
    if [[ -z "$FOCUS" ]]; then
      echo "Error: --focus required for custom review type"
      exit 1
    fi
    run_review "custom" "$PROJECT_CONTEXT

REVIEW: $FOCUS

READ the actual files. Provide specific file:line references and concrete code fixes."
    ;;

  full)
    echo "Running all 5 reviews in parallel..."
    for type in gpu bugs deadcode training combat; do
      "$0" "$type" --effort "$EFFORT" --output-dir "$OUTPUT_DIR" &
    done
    wait
    echo "All reviews complete. Results in $OUTPUT_DIR/"
    ls -la "$OUTPUT_DIR/${TIMESTAMP}"-*.md 2>/dev/null
    ;;

  help|--help|-h)
    head -18 "$0" | tail -16
    ;;

  *)
    echo "Unknown review type: $REVIEW_TYPE"
    echo "Run: $0 help"
    exit 1
    ;;
esac
