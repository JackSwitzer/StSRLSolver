# Work Unit: Training Profiling

## Goal
Identify and fix bottlenecks in the COLLECT phase. Only 1-3 of 8 workers are typically active at once.

## Investigation Areas

### Inference Latency
- Profile InferenceServer batch sizes (target: 8-12 per batch)
- Measure queue wait time vs inference time
- Is MLX actually using Metal? Check with `METAL_DEVICE_WRAPPER_TYPE=1`

### TurnSolver Performance
- Profile solver time distribution across room types
- How often does solver hit timeout vs node budget?
- Cache hit rates for repeated states

### Worker Utilization
- Why only 1-3 active? Possible causes:
  - GIL contention in multiprocessing.Pool callbacks
  - Memory pressure (24GB shared across 12 workers)
  - Inference bottleneck (single GPU, serial batching)
  - Engine construction overhead (GameRunner init)

### GPU Utilization
- MLX Metal utilization during COLLECT vs TRAIN phases
- MPS fallback detection
- Batch inference throughput (games/sec at various batch sizes)

## Profiling Commands
```bash
# Python profiling
uv run python -m cProfile -o collect_profile.prof -m packages.training.overnight --games 100
# Metal GPU trace
METAL_CAPTURE_ENABLED=1 uv run python -m packages.training.overnight --games 10
# Memory
uv run python -m memory_profiler packages/training/worker.py
```

## Target
- 10 workers all active simultaneously
- >100 games/min sustained throughput
- <50ms p95 inference latency
