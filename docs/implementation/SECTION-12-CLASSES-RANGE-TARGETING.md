# Section 12 — Character Classes, Positioning, Range, and Targeting

**Status:** Planned — begin only after Section 11 has landed in the chosen base.
**Dependencies:** Section 11 combatants, action resolution, turn progression, and outcome handling.
**Follow-on:** Section 13 uses these rules for shared effects and enemy decisions.

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

Make class identity and relative battlefield position influence tactical choices through measurable rules, without committing prematurely to a full tactical grid.

## Core design constraints

- Keep class identity separate from attributes, equipment, abilities, and mastery perks. Reuse existing combat/player concepts instead of creating a parallel character system.
- Start with four data-driven archetypes: **Warrior** (durability, defence, control), **Ranger** (ranged precision), **Mage** (elemental damage and later status effects), and **Rogue** (speed, critical hits, evasion). Treat these as provisional balance roles, not scattered name-based exceptions.
- Use the minimum useful attribute set supported by the code; likely candidates include speed, accuracy, evasion, power, and resistance. Document units, defaults, and caps.
- Represent position first through documented range bands such as melee, near, and far. Do not create generic abstractions with no current use or a full grid in this section.
- Class labels must yield testable mechanical differences, not just different menu descriptions.

## Implementation slices

### 12.1 — Classes and derived attributes

- Introduce or extend reusable class definitions and explicit mechanical trade-offs.
- Document how class, equipment, and mastery perks contribute to each relevant attribute. Avoid double-counted bonuses.
- Extend Section 11's initiative policy so speed influences turn order predictably. Define deterministic tie-breaking.
- Bound attribute modifiers and test minimum/maximum values.

### 12.2 — Range and legal targets

- Store combat position in encounter state, not UI widgets or global mutable state.
- Define legal ranges for attacks and abilities. Add movement/range-changing actions only where needed to make the rules playable.
- Validate target existence, allegiance, actor/target eligibility, and range before costs or effects are applied.
- Derive UI target lists and enabled actions from the same domain rules used by action resolution.

### 12.3 — Accuracy and combat feedback

- Define a documented hit-chance formula using relevant attacker/defender attributes, class capability, and distance. Clamp it to configured bounds and choose values after auditing existing combat calculations.
- Make rolls controllable in tests; test the formula separately from random outcome resolution.
- Show target identity, relative range, and why an action is unavailable. Preserve keyboard navigation.
- Mouse interaction belongs to Section 15; do not claim it is implemented here.

## Scope boundaries

- No grid pathfinding, tiles, facing, or line-of-sight system.
- No UI-only range, hit-chance, or legal-target logic.
- Do not implement Section 13's full effects/AI or broadly rebalance every encounter.
- Record assumptions so future balance work can revise them on evidence.

## Acceptance criteria

- Initial classes have distinct, testable strengths and weaknesses.
- Speed affects initiative under a documented policy with deterministic tie-breaking.
- Out-of-range actions and illegal targets are rejected before resource use.
- Accuracy varies predictably with class capability and range and remains bounded.
- Target selection and action resolution agree on validity.
- Tests cover range boundaries, invalid targets, initiative order, accuracy bounds, class trade-offs, and existing story/Stronghold encounters.

## Section-specific verification

Use unit tests for pure formulas and domain validation, then let the required CI feedback loop validate the whole crate. Leave Sections 13–14 for their own briefs.
