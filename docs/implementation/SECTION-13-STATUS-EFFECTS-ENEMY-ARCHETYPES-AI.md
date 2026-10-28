# Section 13 — Status Effects, Enemy Archetypes, and Combat AI

**Status:** Planned — begin only after Sections 11 and 12 have landed in the chosen base.
**Dependencies:** Shared combatants/actions, turn progression, class attributes, range/target validation, and accuracy rules.

## Instructions for the implementing agent

Treat this as an executable implementation brief, not a request to return only a proposal. Any capable coding agent may do the work; ChatGPT is the primary implementation partner in the current workflow, but no particular model or vendor is mandatory. Use the repository and tools actually available to you, and be transparent about limitations.

Before editing, inspect the latest `main`, the relevant implementation, save/data models, tests, and dependencies. Treat the current source as authoritative: the names and examples in this brief describe desired behaviour, not guaranteed existing APIs. Confirm that prerequisite sections have landed in the selected base. If a required foundation is absent, stop before building on speculative types and report the dependency blocker.

Make focused, reviewable changes on a feature branch from the latest `main`. Preserve user work. Do not push feature work directly to `main`, reset or force-push, disable CI, merge your own PR, or bundle unrelated cleanup. Keep domain rules independently testable and separate from terminal presentation. Use deterministic/injected randomness where relevant. Never claim a command or check passed unless it was actually run.

## Required CI feedback loop and PR handoff

RustyRealm has GitHub Actions CI in `.github/workflows/rust.yml`, using GitHub-hosted **Ubuntu 24.04** runners. It triggers on pushes, pull requests, and manual dispatch, with three checks:

- Formatting: `cargo fmt --check`
- Tests: `cargo test --package rustyrealm`
- Clippy: `cargo clippy --package rustyrealm --tests`

1. Run the equivalent checks locally whenever your environment supports Rust. Then commit and push the feature branch and open a pull request against `main` for the repository owner to review. Do not merge it.
2. Inspect the real Actions run associated with the PR's latest head commit. Confirm that **all three jobs** have completed successfully; do not infer success from the PR merely existing or from a previous commit's run.
3. If any job fails, inspect its logs and identify the actual root cause. Make the smallest appropriate fix on the same feature branch, commit and push it, then inspect the fresh run for that new commit. Repeat this feedback loop until all checks pass.
4. Do not silence warnings by weakening checks, add `continue-on-error`, skip a failing job, loosen tests to manufacture a pass, or state that CI is green without evidence. If a failure is an external outage, an environment limitation, or a product decision that requires the owner's judgement, report the exact blocker and leave the PR open rather than pretending it passed.
5. Hand off with the PR link, a concise summary, files changed, local commands actually run, the latest CI run/result, and any remaining limitations. Leave the PR **open and unmerged** for the owner.

The CI workflow is the shared verification feedback loop for ChatGPT and any other agent willing and able to participate. Where an agent cannot access GitHub Actions or push updates, it must provide a truthful limitation and a usable patch/handoff rather than claiming the loop completed.

## Objective

Create one consistent lifecycle for combat effects and give enemies tactically distinct behaviour through explicit, testable AI decisions. The systems must work across player characters, companions, and enemies while retaining existing bosses.

## Implementation slices

### 13.1 — Shared status-effect lifecycle

Adapt existing condition handling rather than leaving two competing sources of truth. Use typed identifiers and represent the data required by each effect, such as source, potency, duration, application chance, and resistance interaction.

Initial effect coverage:

- **Poison, bleed, burn:** recurring damage under a documented tick rule.
- **Stun:** restrict or skip the affected combatant's action under an explicit policy.
- **Slow:** reduce speed/initiative using a bounded rule.
- **Mark:** create a defined targeting or hit-chance vulnerability.
- **Weakened defences:** change defensive calculations by a bounded amount.

For every effect define whether it stacks, refreshes, replaces an existing application, or is resisted/immune. Choose a shared default and exceptions only where gameplay requires them. Document timing precisely: when effects are applied, when they tick, whether the application turn counts toward duration, when duration decreases, and when effects expire relative to actor turns and round transitions. Avoid ambiguous “each turn” rules.

Use one lifecycle for every combatant type. Invalid actions must not consume costs or partially mutate state; effect resolution belongs in domain code, not presentation code.

### 13.2 — Enemy archetypes and decisions

- Represent enemy behaviour with data or focused domain rules: preferred range, available actions, target priorities, resistances, and tactical tendencies.
- AI should propose a structured action; Section 11's common resolver validates and applies it.
- Reuse Section 12's range and target rules rather than duplicating them in AI.
- Start with a small number of distinct behaviours, such as aggressive melee, ranged pressure, defensive/protective, and status-focused.
- Preserve Section 8's random encounter restrictions: generic civilians and ordinary real-world wildlife must not return to the hostile wandering pool unless the owner explicitly changes that design.
- Control RNG for AI tie-breaking and choices so tests are reproducible.

### 13.3 — Telegraphs and boss compatibility

- Model telegraphed attacks as explicit state transitions: announce a defined action, offer a response opportunity, then resolve or cancel according to documented rules.
- A telegraph must match the action that subsequently resolves, including target and effect. Test interrupted/cancelled telegraphs and target validity at resolution.
- Preserve Stronghold guardians, Lacoyx phases, and special story encounters by migrating them incrementally to shared systems.

## Scope boundaries

- Do not implement separate effect systems per combatant type or let AI bypass the shared resolver.
- Do not make random tests flaky or dependent on terminal text.
- Avoid adding a large effect/enemy roster before lifecycle and balance rules are stable.
- Companion rescue and recovery remain Section 14 work.

## Acceptance criteria

- Effects apply, tick, resist, stack/refresh, and expire consistently across all combatant types.
- Timing and stacking policies are documented and deterministic under controlled randomness.
- Resistance and application chance influence results through explicit, bounded rules.
- AI returns structured actions validated by the common resolver.
- Telegraphs precede and accurately describe the action that resolves.
- Existing Section 8 encounter restrictions, bosses, Stronghold combat, and Lacoyx phases remain intact.

## Section-specific verification

Add deterministic lifecycle, resistance, stacking, AI decision, and telegraph sequence tests. Use the required CI feedback loop to catch regressions and fix failures before handoff.
