<task>
Wave 3b: unblock strict certification by correcting a false assumption, then drive the one complete A0 golden run as deep as it will go. Continue on `codex/oracle-loop-wave3` (your wave-3 branch).

## The operator attestation (this is ground truth, not a guess)

Your wave-3 report stopped the strict-certified prefix at 0 because "every readable bundle lacks authoritative profile initialization" — action 51 hit a relic-pool identity that depends on unlock state, and you correctly refused to *guess* all-unlocked. The operator has now attested it: the recording profile has **every character, card, relic, and ascension unlocked; all bosses seen** (multiple A20 clears incl. a Defect victory). This is committed as machine-readable ground truth at `data/traces/recordings/profile-attestation.json` (v1 `meta.profile` payload: `locked_cards: []`, `locked_relics: []`, `highest_unlocked_ascension: 20`, `final_act_available: true`, all 9 bosses seen).

So "all-unlocked" is no longer manufactured evidence — it is an operator-provided fact for all 14 legacy bundles. Consume the attestation as the profile source when a bundle's own `meta.profile` is absent; cite the attestation file the way you cite Java. Every pool-realization and Neow-option computation that was gated on missing profile data should now resolve.

## Scope, in order

1. FALSE-ASSUMPTION SWEEP. The all-unlocked gap was one instance of a class of error: the replay treating an *unknown* input as an *unrepresentable* one and quarantining, when the input is in fact knowable (operator-attested, Java-defaulted, or derivable). Find the others. For each place the comparator quarantines or refuses a bundle, ask: is this input genuinely unavailable, or did we lack a source we now have? Enumerate every such assumption in `docs/work_units/oracle-assumptions.md` — for each: what was assumed, why it was wrong/right, the authoritative source (attestation / Java default / decompiled logic), and the fix. Do NOT silently flip any assumption without recording it there with a citation. If an input is genuinely still unavailable from the recorder, THAT one stays a recorder-need (leave it in `data/traces/requests/wave3-recorder-needs.md`) — distinguish clearly.

2. DRIVE THE GOLDEN. Take the one direct-reward A0 victory bundle (deepest comparable prefix 51/607) and, with the profile now resolved, extend the certified prefix as far as it goes. Earliest source-confirmed divergence → root-cause against `decompiled/java-src/` → fix with citation + source-derived test → flip the relevant `docs/goal/ledger.json` rows via `scripts/ledger.sh` → repeat. Report the number as "certified through action N/607"; it only goes up. Effort-cap per `docs/goal/GOAL.md` (quarantine with DEV-NNN + register entry, never stall).

3. CONTINUE F16 POWER TAIL if the grind hits it: cross-relic `addToTop`/`addToBot`/startup queue order, dynamic power identity serialization (The Bomb, Minion, BackAttack, Stasis, Pen Nib).

## Rules

Ground truth precedence: running-game trace > operator attestation > decompiled Java > vault notes. `packages/harness-java/` is read-reference only (recorder v2 is being built operator-side in parallel — do not touch it, and do not wait on it; work everything the attestation + existing bundle fields already unblock). Suite stays green (`./scripts/test_engine_rs.sh test --lib`, 3,110+, count only up); `check --all-targets` clean. Citations, ledger discipline, conventional commits per `AGENTS.md`. End every session reporting: certified prefix per bundle, assumptions corrected (with sources), divergences fixed and what was wrong, ledger rows flipped, quarantines, recorder-needs still open.
</task>
