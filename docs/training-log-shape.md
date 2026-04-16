# Training Log Shape

Claude and SpireMonitor should treat the training runtime as artifact-first. The supported run directory is one `logs/active` tree containing only the current MLX overnight run.

## Required Files

- `manifest.json`
  - run identity, git snapshot, config snapshot, backend policy
  - health check fields:
    - `config.values.backend_policy`
    - `config.values.backend_requested`
    - `config.values.backend_loaded_collection`
    - `config.values.backend_loaded_training`
- `summary.json`
  - top-level run summary and checkpoint outcome
  - health check fields:
    - `collection_summary.record_count`
    - `collection_summary.target_count`
    - `training_summary.example_count`
    - `backend_requested`
    - `backend_loaded_collection`
    - `backend_loaded_training`
- `events.jsonl`
  - append-only lifecycle milestones
  - expected event types:
    - `corpus_generated`
    - `puct_collection_complete`
    - `training_complete`
    - `seed_validation_complete`
- `metrics.jsonl`
  - per-case metrics during collection
  - important fields:
    - `name`
    - `value`
    - `step`
    - `deck_family`
    - `remove_count`
    - `potion_set`
    - `enemy`
    - `corpus_slice`
    - `corpus_case`
    - `seed_source`
- `episodes.jsonl`
  - per-case replay/search summaries
  - important fields:
    - `steps[].search_frontier`
    - `steps[].value`
- `system_stats.jsonl`
  - periodic host/process snapshots during corpus, collection, training, and validation
  - important fields:
    - `phase`
    - `step`
    - `process_cpu_percent`
    - `process_rss_gb`
    - `host_cpu_percent`
    - `host_memory_used_gb`
    - `host_swap_used_gb`
    - `gpu_percent`
    - `gpu_status`
- `frontier_report.json`
  - aggregate frontier ranking report for the monitor
- `frontier_groups.json`
  - grouped frontier slices for comparison
- `benchmark_report.json`
  - benchmark slice rollups
- `puct_targets.jsonl`
  - normalized root-visit policy/value training targets
- `puct_targets_report.json`
  - collection pass summary
  - important fields:
    - `backend_requested`
    - `backend_loaded`
    - `total_records`
    - `collection_passes`
    - `pass_counts`
- `seed_validation_report.json`
  - seed replay status and overnight gate result
  - important fields:
    - `validated_seeds`
    - `failed_seeds`
    - `required_seed_count`
    - `metadata_only_count`
    - `seeds[].status`
- `checkpoint.json`
  - current MLX checkpoint

## Seed Status Semantics

- `reconstructed_validated`
  - part of the required overnight gate
- `metadata_only`
  - reported but non-blocking

Tonight's required seed gate is:

- `4AWM3ECVQDEWJ`
- `4VM6JKC3KR3TD`

The metadata-only seed is:

- `1TPMUARFP690B`

## Health Checks

Claude should consider an overnight run healthy when:

- `manifest.json` and `summary.json` both report MLX for the requested and loaded backends
- `events.jsonl` contains all 4 lifecycle events
- `seed_validation_report.json` shows:
  - `required_seed_count == 2`
  - `failed_seeds == 0`
- `metrics.jsonl` and `episodes.jsonl` are both growing during the run
- `system_stats.jsonl` is growing during long collection/training stages
- `logs/active` does not contain stale old-pipeline files such as `dataset.jsonl`

## Operational Rule

Before launching a new overnight run, clear or archive the previous contents of `logs/active` so Claude and SpireMonitor only read the current artifact set.
