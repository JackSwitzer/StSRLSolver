---
status: active
priority: P0
pr: 133
title: "Watcher External Validation Seed Suite"
scope: training
layer: evaluation
created: 2026-04-15
completed: null
depends_on: [training-architecture]
assignee: claude
tags: [training, watcher, seeds, validation, baalorlord]
---

# Watcher External Validation Seed Suite

This doc is the branch-authoritative reference for the **external seed suite** used
to validate the phase-1 Watcher combat stack.

These seeds are **not** the synthetic phase-1 training corpus.
They are a separate **community replay / validation bank** used to:

- sanity-check that the combat solver learns anything sensible
- compare checkpoints against obviously strong or obviously awkward starts
- drive manual monitor sessions and replay inspection
- seed the future “import real run path + decisions” work

## Current Source Strategy

The first seed bank is sourced from **Baalorlord Watcher runs** because each page gives us:

- exact seed string
- character
- source ascension
- Neow bonus text
- source run URL
- high-level downstream deck/removal context

Current default assumption:

- **source runs are A20**
- **we replay them as validation seeds at A0**

That means the external suite is useful for:

- opening quality
- early route/resource snowballing
- remove-heavy or transform-heavy validation
- monitor visual demos

It is **not yet** a strict run-level oracle.

## Current Seed Bank

The machine-readable version lives in:

- [packages/training/seed_suite.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/seed_suite.py:1)

CLI export:

```bash
./scripts/training.sh print-seed-suite
```

Current categories:

- `easy`
- `minimalist-style`
- `remove-heavy`
- `transform`
- `rare-card`
- `pandoras-box`
- `neows-lament`
- `loss-control`

Current highlights:

1. `9YGUT28YGS0U`
   - Baalorlord Watcher win
   - lose all gold, remove Defend + Defend
   - good easy/minimalist-style validation
2. `2XHHFJ3P7FIZ`
   - second double-remove Watcher win
   - good paired remove-heavy seed
3. `4AWM3ECVQDEWJ`
   - remove-heavy minimalist-style line
   - source run later removed 8 basics total
4. `3FRQITS2NMXWS`
   - transform Defend + Defend into Sanctity + Worship
   - high-roll transform seed
5. `4KSQS5JHKT5QI`
   - transform Defend + Defend into Fasting + Consecrate
   - second transform seed with different shape
6. `1AS4LGHSY0GFL`
   - random Rare card -> `Omniscience`
   - clear easy/high-roll validation
7. `10CWNH9IJ279B`
   - `Neow's Lament`
   - route/easy-fight snowball validation
8. `794YZS4F0DPR`
   - starter swap -> `Pandora's Box`
   - extreme archetype seed for monitor demos
9. `2Q148N2V0TK80`
   - `Black Star` swap
   - loss-control seed
10. `588H4D1XVAAIG`
   - second `Black Star` loss-control seed

## Baalorlord Import Proof Of Concept

The next layer after this seed bank is a **run import scaffold**, not just seed storage.

Target import contract per run:

- seed string
- character
- source ascension
- Neow choice
- floor-by-floor pathing decisions
- shop/remove decisions
- reward picks
- potion purchases / spends when recoverable
- URL back to the source run

The near-term proof of concept is:

1. select 3-5 Baalorlord Watcher runs from the seed suite
2. encode seed + Neow + high-level path/resource decisions
3. replay the same run state at **A0**
4. compare our combat frontier against the resources Baalorlord had

This gives us a bridge between:

- synthetic combat-first training
- external human reference runs
- future strategic/pathing learning

## Placeholder Boundaries

Still true on this branch:

- the phase-1 training corpus is still mostly synthetic
- the external seed bank is not yet automatically harvested into corpus cases
- floor-by-floor Baalorlord decision extraction is not automated yet
- “easy” means source-informed and manually curated, not formally solved

That is acceptable for PR #133.
The goal is to make the seed suite explicit and machine-readable now, then grow it
into a fuller external replay/import layer in the next slices.

## Next Candidate Seeds From Scouting

These are promising additions once we want a wider validation bank:

- `4VM6JKC3KR3TD`
  - Baalorlord Watcher A20 run
  - strong early `Lesson Learned` line with a recognizable stance-dance combat shell
- `1TPMUARFP690B`
  - Steam Watcher seed
  - `Ice Cream` into `Runic Pyramid`, good for retain/control validation
- `1BJHJH51LVJ1Z`
  - community-reported Watcher seed
  - strong 5-card / infinite-style validation candidate
