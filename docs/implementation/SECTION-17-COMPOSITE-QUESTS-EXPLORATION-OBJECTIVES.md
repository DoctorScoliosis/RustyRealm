# Section 17 — Composite Quests and Exploration Objectives

**Status:** Planned
**Dependencies:** Sections 9 and 16 for location and skill-check objectives; Section 19 integrates the full adventure loop.

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

Expand quests from single objectives to structured quests containing sequential stages, parallel objectives, optional goals, and multiple gameplay activities.

## Implementation slices

### 17.1 — Objective and stage model

Support typed objective records for:

- Defeating a specified number or type of enemy.
- Collecting or handing in specified items.
- Gathering resources or crafting an item.
- Reaching or traversing a location.
- Interacting with a landmark or NPC.
- Completing a prerequisite quest.

Represent sequential stages and parallel objective groups explicitly. Optional objectives may grant bonus rewards without blocking main completion unless the quest definition says otherwise.

### 17.2 — Event-driven progress

- Audit the current `QuestObjective`, `Quest`, and `GameEvent` model before deciding on a migration.
- Progress quests from typed events emitted after successful gameplay actions, not display-text matching.
- Define event payloads and matching rules so unrelated kills, items, locations, or recipes cannot advance objectives accidentally.
- Ensure one successful action advances each applicable objective at most once per emitted event.

### 17.3 — Rewards, UI, and save migration

- Maintain clear accepted, in-progress, completed, and claimed states or equivalent lifecycle semantics.
- Guarantee rewards are granted exactly once even if the claim action is repeated.
- Display progress, next stage, optional objectives, and rewards in the TUI.
- Migrate existing single-objective quests and legacy saves explicitly. Use serde defaults only where they preserve meaning; test migration rather than silently discarding/resetting progress.

## Scope boundaries

- Do not replace quest structures without a compatibility plan.
- Do not make objectives depend on UI labels or incidental call order.
- Avoid a large quest-content expansion until reusable objective types and migration tests pass.
- Preserve existing quest progress and idempotent reward semantics.

## Acceptance criteria

- A quest tracks multiple objectives with independent progress.
- Sequential stages unlock in order while parallel objectives progress together.
- Location objectives use the world model; gathering/crafting objectives use successful domain events.
- Rewards are granted exactly once.
- Existing quest saves remain valid or use an explicit, tested migration.
- UI progress reflects domain state and does not determine it.

## Section-specific verification

Test event/objective matching, sequential and parallel progression, optional goals, duplicate event delivery, one-time reward claims, and legacy save migration. Use the CI feedback loop to resolve failures before handing the PR to the owner.
