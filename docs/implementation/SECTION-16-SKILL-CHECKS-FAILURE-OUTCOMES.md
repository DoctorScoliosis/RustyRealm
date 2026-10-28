# Section 16 — Skill Checks and Failure Outcomes

**Status:** Planned
**Dependencies:** Existing skills and economy/inventory operations; later used by composite quests and realm content.

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

Introduce reusable, bounded uncertainty for selected non-combat actions while keeping outcomes predictable, testable, and fair.

## Implementation slices

### 16.1 — Skill-check model and probability

- Define a domain-level check input containing relevant skill value, action difficulty, and applicable modifiers.
- Document the success-probability formula and configurable lower/upper bounds.
- Allow equipment, class abilities, mastery, and environmental conditions to modify a check only when relevant to the action.
- Use controlled randomness or an injected roll source so probability and boundary tests are reproducible.

### 16.2 — Action-specific outcomes

- Apply checks selectively to activities such as gathering, thieving, crafting, and exploration; do not roll for every routine interaction by default.
- Define contextual outcomes for failure: wasted time, reduced yield, lost opportunity, or alerted enemies where appropriate.
- Specify which materials, currency, tools, or other resources are consumed on failure before implementing each action.
- Keep routine low-risk actions from becoming excessively frustrating.

### 16.3 — Transactional validation

- Validate prerequisites before rolling or mutating inventory/resources.
- Apply the documented success/failure outcome as one coherent operation so failed validation cannot partially consume resources.
- Keep action policy separate from terminal rendering.
- Audit existing resource/reward semantics before adding checks to established activities.

## Scope boundaries

- Do not use one universal penalty for every failure.
- Do not let equipment, mastery, or skill bonuses exceed configured probability bounds.
- Avoid applying checks to all existing activities without an explicit design reason.
- Do not introduce unreproducible random tests.

## Acceptance criteria

- Every implemented check has a documented difficulty, formula, bounds, and failure outcome.
- Higher relevant skill generally improves success probability.
- Resource consumption is explicit and transactional.
- Tests cover low, medium, and high skill values, probability bounds, and edge cases.
- Controlled rolls make tests deterministic.
- Bonuses cannot push success above the configured maximum or below the configured minimum.

## Section-specific verification

Test pure probability calculations independently from roll resolution, and test every action's resource consumption on both success and failure. Complete the required CI feedback loop and open a review PR rather than merging.
