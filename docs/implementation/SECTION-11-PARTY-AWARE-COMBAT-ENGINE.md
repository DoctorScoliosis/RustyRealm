# Section 11 — Party-Aware Combat Engine Foundation

**Status:** Planned — begin only after Section 10 has landed in the chosen base.
**Dependencies:** Section 10 companion definitions, persistent companion state, and party operations.
**Next sections:** Section 12 adds class/range rules; Section 13 adds effects and AI; Section 14 connects combat outcomes to recovery.

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

Refactor combat around explicit encounters containing multiple allies and multiple enemies. Establish a shared action-resolution pipeline and clear combat outcomes without coupling the rules to the terminal UI or attempting a wholesale rewrite of every legacy encounter.

## Implementation requirements

### 11.1 — Combatant and encounter model

- Introduce stable combatant identities and explicit allegiance/faction. Never identify combatants by display name.
- Fit the model to the landed Section 10 companion types and current player/enemy data. Separate reusable definitions from encounter-local mutable state where that fits the current architecture.
- Represent resources, eligibility to act, knockout/defeat, and encounter membership explicitly. Avoid contradictory state flags.
- Model encounter state, round/turn progression, eligible actors, and terminal outcomes in domain code.
- Leave useful extension points for abilities, effects, and position/range without prematurely implementing Sections 12–13.
- Migrate existing entry points incrementally. Preserve story encounters, Stronghold guardians, and Lacoyx phases.

### 11.2 — Validated action-resolution pipeline

- Define explicit action data for actions required by current combat, with extension points for abilities, defence, movement/range changes, and retreat.
- Resolve an action in a consistent order: validate actor/target and prerequisites; validate allegiance, availability, and supported range rules; validate resource costs; resolve RNG/effects; then commit resource and health changes.
- A rejected action must not partially consume resources or mutate combat state.
- Reuse the existing random facility where possible; avoid dependencies without a concrete need. Make rule tests reproducible.
- Return structured action results rather than requiring the UI to parse display text.

### 11.3 — Turn order and encounter outcomes

- Define and document the initial initiative policy. It must be deterministic for the same state and RNG; Section 12 may refine it to use speed.
- Define victory, player/party defeat, retreat, and partial-party incapacitation.
- A companion becoming unable to fight must not end an encounter while another eligible ally can act.
- Prevent ordinary actions after a terminal encounter outcome until a new encounter starts.

## Scope boundaries

- No full square-grid battlefield.
- Do not implement final class balance, full range rules, Section 13's effect lifecycle, or Section 14's recovery services here.
- Do not mix combat rules with rendering or terminal event polling.
- Do not rewrite every encounter at once or alter unrelated economy, crafting, quests, mastery, save security, or progression.

## Acceptance criteria

- An encounter supports multiple allies and multiple enemies with stable identities and explicit sides.
- Actor eligibility, action validation, initiative, and outcomes are independently testable.
- Invalid targets, ineligible actors, and insufficient resources are rejected before state changes.
- One companion's knockout does not end combat when other eligible allies remain.
- Controlled randomness makes outcomes reproducible.
- Story encounters, Stronghold battles, and Lacoyx phase transitions remain functional.
- Tests cover multiple actors, invalid actions, costs, partial knockouts, terminal outcomes, and compatibility paths.

## Section-specific verification

Alongside the required CI loop, add or update focused unit tests for multi-actor turns, invalid actions, resource costs, partial knockouts, victory/defeat/retreat, and legacy encounter compatibility. Do not mark Sections 12–14 complete as a side effect of this work.
