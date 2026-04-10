# M4 Mac Mini System Configuration for High-Throughput MCTS Training

## Executive Summary

Your M4 Mac Mini (10-core CPU, 10-core GPU, 24GB unified memory) is a capable
single-machine training rig, but extracting maximum throughput requires careful
partitioning of unified memory, correct thread pool sizing, and avoiding several
macOS-specific pitfalls. The key bottleneck is memory bandwidth (120 GB/s shared
between CPU and GPU), not compute. Every optimization should be evaluated through
the lens of: does this reduce memory pressure and bandwidth contention?

**Hardware you have**: M4 Mac Mini, 4P+6E cores, 10 GPU cores, 24GB, 120 GB/s bandwidth.

**Key recommendations**:
- Partition memory: 10GB MCTS pool, 2GB neural nets, 4GB training buffer, 8GB OS/Python/headroom
- Use jemalloc for the Rust engine (15-25% allocation throughput gain)
- Size Rust thread pool to 4 threads (P-cores only) for MCTS, leave E-cores for Python workers
- Replace `String` card/relic IDs with integer IDs to cut state clone cost by 60-70%
- Batch MLX inference at 64-128 samples for optimal throughput
- Keep GPU exclusively for MLX inference; MCTS stays on CPU

---

## 1. Hardware Specs

### Your Machine (confirmed via system_profiler)

| Component | Spec |
|-----------|------|
| Chip | Apple M4 |
| CPU | 10 cores: 4 Performance (P) + 6 Efficiency (E) |
| GPU | 10 cores, Metal 4 |
| Neural Engine | 16-core, 38 TOPS |
| Memory | 24 GB unified LPDDR5X |
| Memory Bandwidth | 120 GB/s (shared CPU+GPU) |
| L1 I-Cache | 128 KB per P-core |
| L1 D-Cache | 64 KB per P-core |
| L2 Cache | 4 MB per cluster |
| Page Size | 16 KB (confirmed via vm_stat) |
| NEON SIMD | Yes (hw.optional.neon = 1) |
| Cache Line | 128 bytes |

### M4 SKU Comparison (for upgrade planning)

| SKU | CPU | GPU | Max RAM | Bandwidth | Price Delta |
|-----|-----|-----|---------|-----------|-------------|
| M4 (yours) | 4P+6E | 10 | 32 GB | 120 GB/s | base |
| M4 Pro | 10P+4E or 12P+4E | 16-20 | 48 GB | 273 GB/s | +$400-600 |
| M4 Max | 12P+4E or 14P+2E | 32-40 | 64-128 GB | 400-546 GB/s | +$1400+ |

**Upgrade assessment**: The M4 Pro would give 2.3x memory bandwidth and 2.5x P-core
count, which directly translates to MCTS throughput. If you hit the bandwidth wall
before the compute wall, that is the upgrade path. The M4 Max is overkill unless
you scale to multi-agent parallel training.

---

## 2. Memory Partitioning

### CombatState Memory Footprint Analysis

From your actual `state.rs`, each `CombatState` contains:
- `EntityState` (player): 3 ints + `FxHashMap<StatusId, i32>` (~15 entries typical) = ~240 bytes
- 4 card piles (`Vec<String>`): ~30 cards average, each `String` is 24 bytes header + ~10 bytes data = ~1,020 bytes
- `Vec<EnemyCombatState>`: 1-5 enemies, each ~400 bytes (two Strings + HashMap + Vec) = ~800 bytes typical
- `Vec<String>` potions: 3 slots, ~120 bytes
- `Vec<String>` relics: ~15 relics, ~600 bytes
- `Vec<String>` retained_cards: ~60 bytes
- `OrbSlots`: ~100 bytes
- Scalar fields: ~80 bytes
- **Total per state: ~3,000-3,500 bytes** including heap allocations

After the integer-ID optimization (see Section 4): **~800-1,200 bytes per state**.

### Memory Budget (24 GB)

| Component | Current | Optimized | Notes |
|-----------|---------|-----------|-------|
| macOS + system | 3.0 GB | 3.0 GB | Irreducible |
| Python runtime (10 workers) | 2.0 GB | 1.5 GB | Each worker ~150-200 MB |
| PyTorch model (training) | 1.5 GB | 1.5 GB | 18M param StrategicNet + CombatNet + optimizer states |
| MLX model (inference) | 0.3 GB | 0.3 GB | Same weights, MLX format, no optimizer |
| MCTS state pool | 6.0 GB | 10.0 GB | See calculation below |
| Training replay buffer | 2.0 GB | 2.0 GB | 75 trajectories x ~50 transitions x 480-dim float32 |
| Inference batch buffers | 0.5 GB | 0.5 GB | Batch staging for MLX |
| Headroom (swap avoidance) | 8.7 GB | 5.2 GB | CRITICAL: macOS compressor activates at ~20GB |
| **Total** | **24 GB** | **24 GB** | |

### MCTS State Pool Capacity

With 10 GB allocated and ~1,200 bytes per optimized state:
- **~8.3 million states** in the pool simultaneously
- At 200 sims/action for boss fights with ~10 actions per turn: 2,000 states per turn
- At 5 sims/action for monsters: trivial memory usage
- Deep strategic MCTS (200 sims, 100-step rollouts): ~20,000 states per decision

This is more than sufficient. Memory is not the MCTS bottleneck; CPU throughput is.

### 32 GB Upgrade (if purchased)

With 32 GB, the MCTS pool grows to ~18 GB, allowing 15M+ simultaneous states.
More importantly, the extra 8 GB of headroom eliminates all swap risk during
sustained training. **Recommended if budget allows** -- the M4 Mac Mini 32GB is
only ~$200 more than the 24GB model.

---

## 3. Rust Engine Optimization

### 3.1 Allocator: jemalloc vs System

**Recommendation: Use jemalloc.**

macOS's system allocator (`libmalloc`) is competent but optimized for general-purpose
workloads. MCTS creates an extreme allocation pattern: millions of short-lived
`CombatState` clones with many small heap allocations (Strings, Vecs, HashMaps).

| Allocator | Strengths for MCTS | Weaknesses |
|-----------|--------------------|------------|
| System (libmalloc) | Zero setup, good for mixed workloads | Higher fragmentation under clone-heavy patterns, no thread-local caching on Apple Silicon [source: Apple dev forums] |
| jemalloc | Thread-local caching, size-class arenas, lower fragmentation | 1-2 MB overhead, requires `tikv-jemallocator` crate |
| mimalloc | Similar to jemalloc, sometimes faster on ARM64 | Less battle-tested on macOS |

**Expected gain**: 15-25% improvement in allocation-heavy benchmarks on ARM64.
[source: tikv-jemallocator benchmarks, Rust allocation benchmark suites]

Add to `Cargo.toml`:
```toml
[dependencies]
tikv-jemallocator = "0.6"
```

In `lib.rs`:
```rust
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

### 3.2 The String Problem (CRITICAL)

Your `CombatState` uses `String` for card IDs, relic IDs, enemy IDs, and potion IDs.
Each `clone()` of a `CombatState` must clone every one of these Strings, which means:
- ~50 String allocations per state clone (30 cards + 15 relics + 3 potions + 2 enemy IDs)
- Each allocation hits the heap, even with jemalloc
- This is likely **60-70% of your clone cost**

**Fix**: Replace all `String` identifiers with integer IDs (`u16` or `u32`).

```rust
// Before: ~24 bytes per card (String header) + heap alloc
pub hand: Vec<String>,

// After: 2 bytes per card, no heap alloc, trivial clone
pub hand: SmallVec<[CardId; 10]>,  // CardId = u16
```

Expected impact on `clone_for_mcts` benchmark: **3-5x speedup**.

This is the single highest-impact optimization for MCTS throughput. The card
registry already exists in your codebase (`CardRegistry` in `cards.rs`); you
just need to use integer keys everywhere instead of String keys.

### 3.3 SmallVec for Fixed-Size Collections

Several collections in `CombatState` have known maximum sizes:
- Hand: max 10 cards (SmallVec<[CardId; 10]>)
- Enemies: max 5 (SmallVec<[EnemyCombatState; 5]>)
- Potions: max 5 slots (SmallVec<[PotionId; 5]>)
- Orb slots: max 10 ([Orb; 10] with a count field)

You already depend on `smallvec`. Using it for these fields eliminates heap
allocations for the common case. Combined with integer IDs, `CombatState`
becomes almost entirely stack-allocated.

### 3.4 FxHashMap to Array-Backed Status Map

`FxHashMap<StatusId, i32>` for statuses is fast but still heap-allocated.
Since `StatusId` is an integer type you control, consider:

```rust
// If StatusId range is < 256, use a fixed array
pub statuses: [i32; MAX_STATUS_ID],  // ~1 KB, trivial clone
```

Or if the range is larger but usage is sparse:
```rust
// Sorted SmallVec for 10-20 active statuses
pub statuses: SmallVec<[(StatusId, i32); 16]>,
```

Either eliminates the HashMap heap allocation entirely.

### 3.5 Thread Pool Sizing

Your M4 has 4 Performance cores and 6 Efficiency cores. For MCTS:

| Thread Count | Expected Behavior |
|--------------|-------------------|
| 1-4 | All on P-cores, maximum single-thread perf, linear scaling |
| 5-6 | Starts spilling to E-cores, ~60% per-thread perf on E-cores |
| 7-10 | All cores saturated, memory bandwidth becomes bottleneck |

**Recommendation for MCTS tree search**: **4 threads** (one per P-core).

Rationale: MCTS tree search is latency-sensitive (you want each simulation
to complete fast so UCB statistics are fresh). P-cores have ~2x the
single-thread performance of E-cores. Using only P-cores gives ~90% of
the throughput of using all 10 cores but with much lower memory bandwidth
pressure, leaving headroom for MLX inference and Python workers.

**Recommendation for Python worker processes**: **6 workers** on E-cores.
Use `taskpolicy -b` to hint the scheduler toward E-cores (see Section 6).

**Recommendation for combined pipeline**:
```
P-cores (4): Rust MCTS threads
E-cores (6): Python game workers (non-combat), inference batching
GPU (10 cores): MLX inference exclusively
```

### 3.6 NEON SIMD Opportunities

The M4 has 128-bit NEON with SVE2 extensions. Realistic SIMD opportunities:

| Operation | SIMD Viable? | Notes |
|-----------|-------------|-------|
| State comparison (equality) | Yes | Compare 128-bit chunks of packed state |
| Bulk status application | Marginal | Too few statuses per entity (~10-20) |
| Damage calculation batch | Marginal | Usually 1-5 enemies, not worth vectorizing |
| Card energy filtering | No | Branching logic, data-dependent |
| Hash computation | Yes | Use NEON-accelerated hashing for transposition table |
| Observation encoding | Yes | 480-dim float32 vector, trivially SIMD-friendly |

**Bottom line**: SIMD is most valuable for observation encoding (the 480-dim
float32 vector sent to the neural net). The combat logic itself is too
branch-heavy to benefit much. Rust's auto-vectorizer with `-C target-cpu=native`
will handle the observation encoding case automatically.

Add to `.cargo/config.toml`:
```toml
[target.aarch64-apple-darwin]
rustflags = ["-C", "target-cpu=native"]
```

### 3.7 Arena Allocation for Tree Nodes

For MCTS tree construction where nodes are allocated during search and
bulk-freed afterward, an arena allocator avoids per-node deallocation:

```rust
// bumpalo crate for arena allocation
use bumpalo::Bump;

let arena = Bump::with_capacity(1024 * 1024); // 1 MB per search
// Allocate nodes in the arena
let node = arena.alloc(MCTSNode { ... });
// At end of search, drop the arena (bulk free)
drop(arena);
```

This eliminates individual `free()` calls for potentially thousands of tree
nodes per MCTS search. Expected improvement: 10-20% for deep searches
(200+ simulations).

---

## 4. MLX Inference Tuning

### 4.1 Network Sizes

From your codebase:
- **StrategicNet**: 480 input, 1024 hidden, 8 residual blocks, 512 action output
  - ~18M parameters, ~72 MB in float32, ~36 MB in float16
- **CombatNet**: 298 input, 256 hidden, 3 layers
  - ~200K parameters, ~0.8 MB in float32

### 4.2 Batch Size Tuning

MLX on Apple Silicon achieves peak throughput when batches are large enough to
saturate the GPU's compute units but not so large that they cause memory pressure.

| Batch Size | Latency (est.) | Throughput (est.) | Notes |
|------------|----------------|-------------------|-------|
| 1 | ~0.3 ms | ~3,300/s | Dominated by kernel launch overhead |
| 8 | ~0.5 ms | ~16,000/s | Good for low-latency needs |
| 32 | ~1.2 ms | ~26,700/s | Current config (TRAIN_MAX_BATCH_INFERENCE) |
| 64 | ~2.0 ms | ~32,000/s | Sweet spot for throughput |
| 128 | ~3.5 ms | ~36,500/s | Near-peak GPU utilization |
| 256 | ~6.5 ms | ~39,400/s | Diminishing returns, higher latency |
| 512 | ~12 ms | ~42,700/s | Bandwidth-limited, latency too high for MCTS |

[est.] Estimates based on MLX benchmarks for similarly-sized MLPs on M-series chips.
Actual numbers depend on operation fusion and memory access patterns.

**Recommendation**: Increase `TRAIN_MAX_BATCH_INFERENCE` from 32 to **64-128**.

Your current `INFERENCE_BATCH_TIMEOUT_MS = 75ms` is reasonable. With 10 workers
generating inference requests, a batch of 64 should fill within 75ms during
active collection. Consider reducing to 50ms if you increase batch size, since
the GPU can process larger batches without proportional latency increase.

### 4.3 float16 Inference

MLX natively supports float16 and the M4 GPU has full-rate float16. Converting
inference weights to float16:
- Halves memory from 72 MB to 36 MB for StrategicNet
- Increases inference throughput by ~1.5-2x on M4 GPU (bandwidth-bound operations)
- No measurable accuracy loss for inference (training stays float32)

```python
# In mlx_inference.py, after loading weights:
for key in weights:
    weights[key] = mx.array(weights[key], dtype=mx.float16)
```

### 4.4 Neural Engine

The M4's 16-core Neural Engine (38 TOPS at INT8) could theoretically handle
the CombatNet inference, but:
- MLX does not currently route to the Neural Engine (it uses GPU)
- CoreML could use the Neural Engine but requires model conversion
- The CombatNet is too small to benefit (kernel launch overhead dominates)
- The StrategicNet could benefit but CoreML integration adds complexity

**Verdict**: Not worth pursuing now. Revisit if Apple adds Neural Engine support
to MLX or if you move to a much larger model.

---

## 5. GPU vs CPU for MCTS

### Analysis

| Property | GPU (Metal) | CPU (ARM64) |
|----------|-------------|-------------|
| Branching | Terrible (warp divergence) | Native, fast |
| Irregular memory access | Slow (no caching benefit) | Good (large L1/L2) |
| Many small allocations | Not supported | Supported (jemalloc) |
| Parallelism model | SIMT (thousands of identical threads) | MIMD (10 independent threads) |
| Latency per operation | High (kernel launch ~10us) | Low (~ns) |
| Throughput at scale | Only if workload is regular | Good up to core count |

**MCTS is fundamentally CPU work.** Each simulation involves:
1. Tree traversal (pointer chasing, branch-heavy)
2. State cloning (irregular memory allocation)
3. Action execution (data-dependent branching: if-else chains for card effects)
4. Backpropagation (small updates to tree statistics)

None of these map well to GPU execution. The only part of the MCTS pipeline
that benefits from GPU is the neural network evaluation at leaf nodes, which
is already handled by MLX on the GPU.

**Optimal split**:
```
CPU (P-cores): MCTS tree search, state cloning, action execution
CPU (E-cores): Python game loop, data collection, batch assembly
GPU: MLX inference (policy + value evaluation at MCTS leaf nodes)
```

### Leaf Batching Strategy

When running MCTS on CPU with neural net evaluation on GPU, the key optimization
is batching leaf evaluations:

1. Run multiple MCTS trees in parallel (one per P-core thread)
2. When a tree reaches a leaf that needs neural net evaluation, queue the state
3. When the batch reaches 64-128 states (or timeout expires), send batch to GPU
4. Continue other MCTS trees while waiting for GPU result
5. When GPU returns, resume the paused MCTS trees

This requires an async leaf evaluation design in the MCTS engine, which is a
non-trivial refactor but delivers the highest throughput. Without it, each MCTS
simulation blocks waiting for GPU inference.

---

## 6. macOS System Tuning

### 6.1 Memory Pressure Management

macOS uses a memory compressor before swapping. The compressor activates when
memory pressure reaches "yellow" (typically ~80-85% usage = ~19-20 GB on your
machine). Once active, it steals CPU cycles for compression.

**Rules**:
- Keep total working set under 20 GB (83% of 24 GB)
- Monitor with: `memory_pressure` command (shows current level)
- In training scripts: `vm_stat | grep "Pages compressor"` -- if nonzero, you are over budget

### 6.2 Process Priority

```bash
# Pin Rust MCTS to P-cores with high priority
taskpolicy -t utility -p <rust_mcts_pid>  # NOT recommended; use default
# Actually: just use nice -n -5 for the Rust process

# Pin Python workers to E-cores with background priority
taskpolicy -b -p <python_worker_pid>  # Background QoS = E-cores preferred

# MLX inference: default priority (GPU scheduling is separate)
```

**Caution**: macOS does not support CPU pinning (no `taskset` equivalent).
`taskpolicy` sets QoS hints that the scheduler may ignore under load. The
scheduler is generally good at placing P-core work on P-cores, but it is
not guaranteed.

### 6.3 VM Tuning

macOS does not expose the same VM tunables as Linux. What you can do:

```bash
# Disable Spotlight indexing on training data directories
mdutil -i off /path/to/logs

# Disable Time Machine for training directories
tmutil addexclusion /Users/jackswitzer/Desktop/SlayTheSpireRL/logs

# Use RAMdisk for temporary MCTS data (if needed)
diskutil erasevolume HFS+ "MCTSTemp" $(hdiutil attach -nomount ram://4194304)
# Creates a 2GB RAMdisk -- useful if intermediate files cause disk I/O
```

### 6.4 Huge Pages

macOS supports "superpage" allocation (16 MB pages on ARM64) via `mmap` with
`VM_FLAGS_SUPERPAGE_SIZE_16MB`. Rust can use this for the MCTS state pool:

```rust
use libc::{mmap, MAP_ANON, MAP_PRIVATE, PROT_READ, PROT_WRITE};

// VM_FLAGS_SUPERPAGE_SIZE_16MB = 0x2000000 (from mach/vm_statistics.h)
const VM_FLAGS_SUPERPAGE_16MB: i32 = 0x2000000;

unsafe {
    let ptr = mmap(
        std::ptr::null_mut(),
        pool_size,
        PROT_READ | PROT_WRITE,
        MAP_ANON | MAP_PRIVATE | VM_FLAGS_SUPERPAGE_16MB,
        -1,
        0,
    );
}
```

**Expected benefit**: Reduced TLB misses for large contiguous allocations.
Measurable only if the MCTS state pool is allocated as a single large region
(arena allocator pattern). With jemalloc's default behavior, the benefit is
minimal because allocations are spread across many small pages.

**Recommendation**: Only pursue if profiling shows TLB misses as a bottleneck
(use `Instruments > Counters` to check).

### 6.5 Thermal Throttling

The Mac Mini has active cooling (fan) but can still throttle under sustained
all-core load. At sustained 100% utilization:

- M4 Mac Mini sustains ~90-95% of peak performance indefinitely [source: community thermal tests]
- Thermal throttling typically kicks in after 10-15 minutes of all-core + GPU load
- The fan runs at ~3000-4000 RPM under sustained load

**Mitigation**:
- Ensure adequate ventilation (do not place in enclosed cabinet)
- Monitor with: `sudo powermetrics --samplers cpu_power -i 5000`
- If throttling is observed, reduce to 3 MCTS threads + 5 Python workers

---

## 7. Benchmark Targets

### 7.1 State Operations (Rust Engine)

| Operation | Current (est.) | After String->IntID | After Arena+SmallVec |
|-----------|---------------|---------------------|---------------------|
| State clone | ~500 ns | ~150 ns | ~80 ns |
| States cloned/sec | ~2M | ~6.7M | ~12.5M |
| Full turn cycle | ~5 us | ~2 us | ~1.5 us |
| Turns/sec (1 thread) | ~200K | ~500K | ~670K |
| get_legal_actions | ~200 ns | ~100 ns | ~80 ns |
| start_combat | ~2 us | ~800 ns | ~500 ns |

[est.] Run `cargo bench` in `packages/engine-rs` to get your actual baselines.

### 7.2 MCTS Throughput

| Scenario | Sims/sec (1 thread) | Sims/sec (4 P-cores) | Notes |
|----------|--------------------|-----------------------|-------|
| Pure rollout (no NN) | ~100K | ~350K | CPU-bound |
| With CombatNet eval | ~20K | ~70K | Bottlenecked by inference latency |
| With batched NN eval | ~50K | ~180K | Async leaf batching, 64-batch |

### 7.3 MLX Inference

| Network | Batch 1 | Batch 32 | Batch 128 | Unit |
|---------|---------|----------|-----------|------|
| StrategicNet (18M) float32 | 3,000 | 25,000 | 35,000 | inferences/sec |
| StrategicNet (18M) float16 | 5,000 | 40,000 | 55,000 | inferences/sec |
| CombatNet (200K) float32 | 15,000 | 100,000 | 200,000 | inferences/sec |
| CombatNet (200K) float16 | 20,000 | 150,000 | 300,000 | inferences/sec |

[est.] Based on MLX MLP benchmarks on M-series. Your actual results will vary
with model architecture details (LayerNorm, residual connections, etc.).

### 7.4 End-to-End Games/Hour

| Configuration | Games/Hour | Bottleneck |
|---------------|-----------|------------|
| Current (Python engine, 10 workers) | ~120-180 | Python engine speed |
| Rust engine, no MCTS | ~2,000-3,000 | Worker coordination overhead |
| Rust engine + MCTS (5 sims/monster) | ~500-800 | MCTS sim budget |
| Rust engine + MCTS (200 sims/boss) | ~200-400 | Boss combat MCTS |
| Rust engine + batched NN + async MCTS | ~800-1,200 | Memory bandwidth |

[est.] These estimates assume the Rust engine is ~50-100x faster than Python for
raw combat simulation. Actual speedup depends on PyO3 boundary overhead and
how much time is spent in Python vs Rust per game.

---

## 8. Recommended Configuration

### Immediate Changes (no code changes required)

```python
# training_config.py
TRAIN_WORKERS = 6              # Was 10; leave 4 cores for Rust MCTS
TRAIN_MAX_BATCH_INFERENCE = 64 # Was 32; better GPU utilization
INFERENCE_BATCH_TIMEOUT_MS = 50.0  # Was 75; faster batch fill at batch=64
```

```bash
# training.sh additions
mdutil -i off /Users/jackswitzer/Desktop/SlayTheSpireRL/logs
tmutil addexclusion /Users/jackswitzer/Desktop/SlayTheSpireRL/logs
```

### Rust Engine Changes (ordered by impact)

1. **Integer IDs** (highest impact, ~3-5x clone speedup)
   - Replace `String` with `u16` for card/relic/potion/enemy IDs
   - Add lookup tables for ID-to-name conversion (debugging only)
   - Touch: `state.rs`, `engine.rs`, `cards.rs`, `actions.rs`, all PyO3 wrappers

2. **jemalloc** (15-25% allocation throughput)
   - Add `tikv-jemallocator` dependency
   - One line in `lib.rs`

3. **SmallVec everywhere** (10-20% clone speedup on top of integer IDs)
   - `hand: SmallVec<[CardId; 10]>`
   - `enemies: SmallVec<[EnemyCombatState; 5]>`
   - `potions: SmallVec<[PotionId; 5]>`

4. **Arena allocator for MCTS tree** (10-20% for deep searches)
   - Add `bumpalo` dependency
   - Allocate tree nodes in arena, drop after search

5. **Async leaf batching** (2-3x MCTS throughput with NN evaluation)
   - Non-trivial refactor: decouple MCTS simulation from NN eval
   - Queue leaf states, batch-evaluate, resume simulations

### Build Configuration

```toml
# .cargo/config.toml
[target.aarch64-apple-darwin]
rustflags = ["-C", "target-cpu=native", "-C", "link-arg=-fuse-ld=lld"]

# Cargo.toml [profile.release]
[profile.release]
opt-level = 3
lto = "thin"        # Already set
codegen-units = 1   # Better optimization, slower compile
panic = "abort"     # Smaller binary, no unwind overhead
```

### Monitoring Commands

```bash
# Memory pressure (run during training)
memory_pressure

# CPU utilization by core type
sudo powermetrics --samplers cpu_power -i 5000

# Rust engine benchmarks (run periodically)
cd packages/engine-rs && cargo bench

# MLX inference throughput test
uv run python -c "
import time, numpy as np
from packages.training.mlx_inference import MLXStrategicNet
net = MLXStrategicNet.from_pytorch('logs/strategic_checkpoints/latest_strategic.pt')
obs = np.random.randn(128, 480).astype(np.float32)
mask = np.ones((128, 512), dtype=bool)
t0 = time.perf_counter()
for _ in range(1000):
    net.forward_batch(obs, mask)
elapsed = time.perf_counter() - t0
print(f'{128000/elapsed:.0f} inferences/sec at batch=128')
"
```

---

## Appendix: Memory Bandwidth Analysis

The M4's 120 GB/s memory bandwidth is shared between CPU and GPU. During training:

| Consumer | Estimated BW Usage | Notes |
|----------|--------------------|-------|
| MCTS state cloning (4 threads) | ~15-20 GB/s | 12.5M clones/s * 1.2 KB = 15 GB/s |
| MLX inference (batch=128) | ~20-30 GB/s | 18M params * 4 bytes * 55K infer/s = 3.6 GB/s weights + activations |
| Python workers (data) | ~2-5 GB/s | Numpy array operations, observation encoding |
| PyTorch training | ~10-20 GB/s | Only during TRAIN phase, not concurrent with collection |
| OS + misc | ~5 GB/s | Page table walks, filesystem, IPC |

**Total during COLLECT phase**: ~42-60 GB/s (35-50% of available bandwidth)
**Total during TRAIN phase**: ~30-45 GB/s (collection paused during training)

The M4 has headroom, but it is not unlimited. If you add more MCTS threads or
increase batch sizes aggressively, you will hit the bandwidth wall before the
compute wall. The M4 Pro's 273 GB/s would give 2.3x bandwidth headroom.
