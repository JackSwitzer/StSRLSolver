# CommunicationMod reference index

CommunicationMod is an optional external-control fallback mentioned by
docs/goal/TOOLING.md. It is not vendored in this repository and is not the
current TraceLab parity path.

## Protocol authority

Use the source and documentation for the exact CommunicationMod build being
run: [ForgottenArbiter/CommunicationMod](https://github.com/ForgottenArbiter/CommunicationMod).
Record the commit or JAR version before depending on command names, JSON fields,
stability gates, or timeout behavior. This repository deliberately does not
duplicate a command table because that table can drift from the installed mod.

At a high level, CommunicationMod launches an external process and exchanges
commands and state over standard input/output once its game-state listener
considers the game stable. That description is not a schema guarantee.

## Current local parity path

- packages/harness-java/ is the TraceLab mod used to mint source-of-truth
  action traces.
- packages/harness-java/src/main/java/tracelab/ScriptRunner.java drives scripted
  game actions.
- packages/harness-java/src/main/java/tracelab/TraceWriter.java writes the
  repository's JSONL schema and all 13 RNG counters.
- scripts/trace_java.sh is the human-only minting entry point.
- scripts/trace_diff.sh is the offline Rust comparison entry point.

Do not translate a CommunicationMod state dump into a Java golden without
first proving that it contains the ordered piles, intent details, relic
counters, and RNG counters required by docs/goal/TOOLING.md.
