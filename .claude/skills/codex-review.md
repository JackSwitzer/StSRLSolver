# Codex GPT 5.4 Review Skill

Use this skill when the user asks for a GPT 5.4 / codex review of code, plans, or architecture.

## Usage
Invoke with: `/codex-review <topic or file path>`

## Instructions

1. **Gather context**: Read the relevant files and construct a focused review prompt
2. **Run codex**: Execute GPT 5.4 via the codex CLI with high reasoning:
```bash
source ~/.claude/.credentials.env && bunx @openai/codex@latest exec \
  -s read-only \
  --ephemeral \
  -m gpt-5.4 \
  -c 'reasoning_effort="high"' \
  "<review prompt>"
```
3. **Parse and present**: Summarize findings back to the user with actionable items

## Prompt Template

Structure the review prompt as:
```
You are reviewing [WHAT] for [PROJECT CONTEXT].

CURRENT STATE:
[Brief description of what exists]

PROPOSED CHANGES / PLAN:
[What's being reviewed]

SPECIFIC QUESTIONS:
1. [Question about correctness]
2. [Question about performance]
3. [Question about architecture]

CONSTRAINTS:
- [Hardware: M4 Mac Mini, 10 cores, 24GB RAM]
- [Performance: X games/min throughput]
- [Other relevant constraints]

Provide: (a) assessment, (b) specific improvements, (c) alternatives if approach is wrong, (d) performance optimizations
```

## Model Selection
- **ALWAYS use `gpt-5.4`** — never 4.1, o4-mini, or other models
- **Reasoning effort**: `high` for reviews, `extra-high` for architecture/scoping
- **Sandbox**: `read-only` for reviews, `full` only if codex needs to write files

## Error Handling
- If codex times out (>3 min), kill and retry with a shorter, more focused prompt
- If model is unsupported, check that `gpt-5.4` is spelled correctly
- Run in background for long reviews, foreground for quick checks

## Common Review Types
- **Code review**: Pass file contents + specific concerns
- **Architecture review**: Pass system diagram + design decisions
- **Performance review**: Include benchmarks + target metrics
- **Bug hunt**: Pass failing test + relevant code + error output
