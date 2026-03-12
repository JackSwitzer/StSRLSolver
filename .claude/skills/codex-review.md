# Codex Review Skill

Use this skill when the user asks for a GPT 5.4 / codex review of code, plans, or architecture.

## Usage
Invoke with: `/codex-review <topic or file path>`

## Instructions

1. **Gather context**: Read the relevant files and construct a focused review prompt
2. **Write prompt to temp file**, then run:
```bash
/Applications/Codex.app/Contents/Resources/codex exec \
  -m gpt-5.4 \
  -c 'reasoning_effort="high"' \
  --sandbox read-only \
  "$(cat /tmp/gpt54-prompt.txt)"
```
3. **Or use the script**: `./scripts/gpt54-review.sh /tmp/gpt54-prompt.txt /tmp/gpt54-result.md high`
4. **Parse and present**: Summarize findings with actionable items

## Auth
- Codex CLI at `/Applications/Codex.app/Contents/Resources/codex` -- has its own login/auth
- **NOT OpenRouter** -- uses native Codex app authentication
- If auth fails: run `/Applications/Codex.app/Contents/Resources/codex login`

## Command Format

The Codex CLI `exec` subcommand runs a one-shot prompt:

```bash
# Quick check (high effort)
/Applications/Codex.app/Contents/Resources/codex exec \
  -m gpt-5.4 \
  -c 'reasoning_effort="high"' \
  --sandbox read-only \
  "Your prompt here"

# Deep review (extra-high effort)
/Applications/Codex.app/Contents/Resources/codex exec \
  -m gpt-5.4 \
  -c 'reasoning_effort="extra-high"' \
  --sandbox read-only \
  "Your prompt here"
```

Key flags:
- `-m gpt-5.4` -- model selection
- `-c 'reasoning_effort="high"'` -- config override (high or extra-high)
- `--sandbox read-only` -- filesystem access (read-only for safety)
- The prompt is the final positional argument (string)

## Scripts
- `scripts/gpt54-review.sh <prompt_file> [output_file] [effort]` -- ad-hoc reviews
- `scripts/nightly-audit.sh [run-dir]` -- automated training audit + email notification
- `scripts/audit-setup.sh install|uninstall|status|run-now` -- manage launchd schedule (9pm daily)

## Effort Levels

| Level | Flag | Use Case | Approx Tokens |
|-------|------|----------|---------------|
| **high** | `reasoning_effort="high"` | Code reviews, bug hunts, quick checks | ~8K |
| **extra-high** | `reasoning_effort="extra-high"` | Architecture decisions, scoping reviews, overnight run audits | ~16K |

## Prompt Template
```
You are reviewing [WHAT] for the Slay the Spire RL project.
PROJECT: Python game engine + RL training (PPO, MLX inference, multiprocessing).
HARDWARE: M4 Mac Mini (10 cores, 24GB RAM, MPS GPU, MLX for inference).

CURRENT STATE: [description]
CODE TO REVIEW: [paste key sections]
SPECIFIC QUESTIONS:
1. [Correctness]
2. [Performance]
3. [Architecture]

Provide: (a) critical bugs, (b) performance wins, (c) dead code, (d) ranked improvements.
```

## Nightly Audit

The nightly audit (`scripts/nightly-audit.sh`) runs three Codex reviews:
1. **Training status** (high effort) -- progress, health, recommendations
2. **Code quality sweep** (extra-high effort) -- recently changed files, bugs, dead code
3. **Combat performance** (high effort) -- floor distribution, death analysis, bottlenecks

Results written to `{run-dir}/audits/YYYY-MM-DD-HHMM-audit.md` and emailed via Mail.app.
