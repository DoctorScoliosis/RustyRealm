# Section 19 — Realm Content and End-to-End Adventure Loop

**Status:** Planned
**Dependencies:** World, party, combat, class/range, effects/AI, recovery, quest, skill-check, and balance foundations from earlier sections.

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

Prove that the systems form one coherent, persistent playable region before scaling up to many realms.

## Required content

- Connect a starting village to wilderness routes and multiple landmarks.
- Include a dangerous destination with distinct enemy archetypes and rewards.
- Add at least one recruitable companion or a companion-related quest.
- Add at least one multi-stage quest combining travel, combat, and an item, gathering, or crafting objective.
- Integrate settlement services, trade, equipment preparation, and companion recovery into the adventure loop.
- Use reusable world, enemy, companion, and quest definitions rather than one-off hardcoded navigation/control flow.

## Implementation slices

### 19.1 — Content definitions

Define a small connected region, routes, destinations, encounter tables, landmarks, NPCs, and quest content using established data models. Include clear travel and danger feedback.

### 19.2 — Complete player-facing loop

The player must be able to:

1. Prepare a party in a village.
2. Travel to a connected location.
3. Encounter and fight a group of enemies.
4. Gain rewards and quest progress.
5. Experience a companion knockout.
6. Return to a settlement or otherwise resolve the expedition.
7. Recover companions and prepare for another attempt.
8. Complete a multi-objective quest and receive its rewards.

### 19.3 — End-to-end and save/load regression

- Exercise the full loop through domain-level integration tests where possible, with UI smoke checks for terminal flows.
- Save/reload at meaningful points: before travel, during quest progress, after rewards, while a companion is recovering, and after quest completion.
- Verify repeated interactions do not duplicate rewards and that travel, inventory, party, location, recovery, and quest state remain consistent.
- Keep deterministic integration tests for CI practical in runtime and isolate terminal-only checks where the runner cannot provide an interactive terminal.

## Scope boundaries

- Build one coherent region rather than many partially connected realms.
- Do not introduce multiplayer, permanent companion death, or a full graphical frontend.
- Avoid new mechanics that have not passed their foundational section's acceptance tests.
- Prefer a small complete adventure over a large amount of disconnected content.

## Acceptance criteria

- The complete eight-step adventure loop is playable.
- Multi-objective quest progress and rewards persist correctly.
- A knockout has a coherent recovery/rescue outcome.
- Travel and combat rewards use shared domain systems.
- Save/load cycles do not corrupt location, inventory, party, recovery, or quest state.
- Automated tests cover important state transitions and reward-idempotency cases.

## Section-specific verification

Run the full adventure loop and its save/load/reward regression cases. Keep the standard Ubuntu CI checks green, inspect the actual PR-head run, and hand off the open PR only after the entire loop is verified or any environment limitation is clearly disclosed.
