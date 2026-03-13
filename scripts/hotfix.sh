#!/bin/bash
# Hot-fix training without restart. Writes reload.json and sends SIGUSR1.
#
# Usage:
#   ./scripts/hotfix.sh entropy 0.03
#   ./scripts/hotfix.sh lr 5e-5
#   ./scripts/hotfix.sh temperature 0.7
#   ./scripts/hotfix.sh batch_size 2048
#   ./scripts/hotfix.sh stance '{"Calm":0.8,"Wrath":1.2}'
#   ./scripts/hotfix.sh event '{"elite_win":0.5,"boss_win":1.0}'
#   ./scripts/hotfix.sh epsilon 0.5 0.1 60000   # start end decay
#   ./scripts/hotfix.sh replay_mix 0.3
#   ./scripts/hotfix.sh json '{"lr":1e-4,"entropy_coeff":0.05}'
#   ./scripts/hotfix.sh status                    # show current reload-able state

set -e
cd "$(dirname "$0")/.."

RUN_DIR="${HOTFIX_RUN_DIR:-logs/weekend-run}"
RELOAD_FILE="$RUN_DIR/reload.json"

send_reload() {
    echo "$1" > "$RELOAD_FILE"
    echo "Wrote: $RELOAD_FILE"
    cat "$RELOAD_FILE"

    local pid
    pid=$(pgrep -f "packages.training.overnight" | head -1)
    if [ -n "$pid" ]; then
        kill -SIGUSR1 "$pid"
        echo "Sent SIGUSR1 to PID $pid"
    else
        echo "WARNING: No overnight process found. Reload file saved for next start."
    fi
}

case "${1:-help}" in
    entropy)
        send_reload "{\"entropy_coeff\": $2}" ;;
    lr)
        send_reload "{\"lr\": $2}" ;;
    temperature|temp)
        send_reload "{\"temperature\": $2}" ;;
    batch_size|batch)
        send_reload "{\"batch_size\": $2}" ;;
    clip)
        send_reload "{\"clip_epsilon\": $2}" ;;
    stance)
        send_reload "{\"stance_rewards\": $2}" ;;
    event)
        send_reload "{\"event_rewards\": $2}" ;;
    floor)
        send_reload "{\"floor_milestones\": $2}" ;;
    epsilon|eps)
        if [ -n "$4" ]; then
            send_reload "{\"epsilon_start\": $2, \"epsilon_end\": $3, \"epsilon_decay\": $4}"
        elif [ -n "$3" ]; then
            send_reload "{\"epsilon_start\": $2, \"epsilon_end\": $3}"
        else
            send_reload "{\"epsilon_end\": $2}"
        fi ;;
    replay_mix)
        send_reload "{\"replay_mix_ratio\": $2}" ;;
    replay_floor)
        send_reload "{\"replay_min_floor\": $2}" ;;
    json)
        send_reload "$2" ;;
    status)
        echo "=== Hot-Reloadable Parameters ==="
        echo "Training:"
        echo "  entropy_coeff, lr, temperature, batch_size, clip_epsilon"
        echo "Rewards:"
        echo "  stance_rewards, event_rewards, floor_milestones"
        echo "Replay:"
        echo "  replay_mix_ratio, replay_min_floor"
        echo "Epsilon:"
        echo "  epsilon_start, epsilon_end, epsilon_decay"
        echo ""
        if [ -f "$RUN_DIR/status.json" ]; then
            echo "=== Current Status ==="
            python3 -c "
import json
with open('$RUN_DIR/status.json') as f:
    s = json.load(f)
print(f'Games:    {s.get(\"total_games\", \"?\")}')
print(f'Floor:    {s.get(\"avg_floor_100\", \"?\")}')
print(f'g/min:    {s.get(\"games_per_min\", \"?\")}')
print(f'Steps:    {s.get(\"train_steps\", \"?\")}')
print(f'Replay:   {s.get(\"replay_buffer\", \"?\")}/{s.get(\"replay_best_floor\", \"?\")}')
print(f'Entropy:  {s.get(\"entropy_coeff\", \"?\")}')
cfg = s.get('sweep_config', {})
print(f'LR:       {cfg.get(\"lr\", \"?\")}')
print(f'Temp:     {cfg.get(\"temperature\", \"?\")}')
print(f'Batch:    {cfg.get(\"batch_size\", \"?\")}')
print(f'Epsilon:  {cfg.get(\"epsilon_start\", \"?\")}->{cfg.get(\"epsilon_end\", \"?\")} / {cfg.get(\"epsilon_decay\", \"?\")}')
"
        fi ;;
    *)
        echo "Usage: $0 <command> [args]"
        echo ""
        echo "Commands:"
        echo "  entropy <val>         Set entropy coefficient"
        echo "  lr <val>              Set learning rate"
        echo "  temp <val>            Set temperature"
        echo "  batch <val>           Set PPO batch size"
        echo "  clip <val>            Set PPO clip epsilon"
        echo "  stance '{...}'        Set stance rewards JSON"
        echo "  event '{...}'         Set event rewards JSON"
        echo "  floor '{...}'         Set floor milestones JSON"
        echo "  eps <start> <end> <decay>  Set epsilon schedule"
        echo "  replay_mix <val>      Set replay mix ratio (0-1)"
        echo "  replay_floor <val>    Set min floor for replay buffer"
        echo "  json '{...}'          Send raw JSON config"
        echo "  status                Show current state + reloadable params"
        exit 1 ;;
esac
