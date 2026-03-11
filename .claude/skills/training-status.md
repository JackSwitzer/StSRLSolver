---
name: training-status
description: Check training status and metrics from the WS server
---

Run `./scripts/training-status.sh` from the project root and parse the JSON output.

Report the following clearly:
- **Running**: Is the training server up? (`running: true/false`)
- **Games played**: Total episodes across all agents
- **Throughput**: Games per hour
- **Avg floor**: Mean floor reached this session
- **Win rate**: Wins / total games (as %)
- **System usage**: CPU %, memory MB if available

If `running: false`, suggest running `./scripts/dev.sh --train` or `./scripts/start-training.sh`.

If the server responds with `running: true` but no stats fields, report that training may be initializing or the get_status message type is not yet handled by the server.
